package yarn

import (
	"fmt"
	"gnarl/semver"
	"io/fs"
	"io/ioutil"
	"log"
	"os"
	"sort"
	"strings"

	yaml2 "gopkg.in/yaml.v2"
)

type Lock struct {
	dirty       bool
	resolutions map[string]Resolution
	suggestions map[string]*semver.Version
}

type Resolution struct {
	Version              string                    `yaml:"version,omitempty"`
	Resolution           string                    `yaml:"resolution,omitempty"`
	CacheKey             string                    `yaml:"cacheKey,omitempty"`
	Dependencies         map[string]string         `yaml:"dependencies,omitempty"`
	DependenciesMeta     map[string]DependencyMeta `yaml:"dependenciesMeta,omitempty"`
	PeerDependencies     map[string]string         `yaml:"peerDependencies,omitempty"`
	PeerDependenciesMeta map[string]DependencyMeta `yaml:"peerDependenciesMeta,omitempty"`
	Bin                  map[string]string         `yaml:"bin,omitempty"`
	Checksum             string                    `yaml:"checksum,omitempty"`
	LanguageName         string                    `yaml:"languageName,omitempty"`
	LinkType             string                    `yaml:"linkType,omitempty"`
}

type DependencyMeta struct {
	Optional bool `yaml:"optional,omitempty"`
}

func yarnLock(directory string) string {
	return fmt.Sprintf("%s/yarn.lock", directory)
}

func ReadLock(directory string) (*Lock, error) {
	reader, err := os.Open(yarnLock(directory))
	if err != nil {
		return nil, fmt.Errorf("cannot open yarn.lock: %v", err)
	}

	defer reader.Close()

	yaml, err := ioutil.ReadAll(reader)
	if err != nil {
		return nil, fmt.Errorf("cannot read yarn.lock: %v", err)
	}

	lock := Lock{resolutions: map[string]Resolution{}, suggestions: map[string]*semver.Version{}}
	err = yaml2.Unmarshal(yaml, lock.resolutions)
	if err != nil {
		return nil, fmt.Errorf("cannot deserialize yarn.lock: %v", err)
	}

	if len(lock.resolutions) == 0 {
		return nil, fmt.Errorf("no entries found in yarn.lock")
	}

	return &lock, nil
}

func (lock *Lock) Fix(npmPackage string, safeVersions *semver.Request) {
	resolutions, _ := lock.read(npmPackage)
	if len(resolutions) == 0 {
		return
	}

	var needsReset bool
	for key, resolution := range resolutions {
		if safeVersions.Matches(semver.MustParseVersion(resolution.Version)) {
			continue
		}

		requested := requestFromKey(key)
		request := semver.MustParseRequest(requested)
		overlaps, closest := request.Overlaps(safeVersions)
		npmPackageRequest := fmt.Sprintf("%s@%s", npmPackage, requested)
		switch {
		case overlaps:
			needsReset = true
		case closest == nil:
			log.Printf(`No fix for %s`, npmPackageRequest)
		case lock.suggestions[npmPackageRequest] == nil:
			lock.suggestions[npmPackageRequest] = closest
		case lock.suggestions[npmPackageRequest].AtLeast().Matches(closest):
			lock.suggestions[npmPackageRequest] = closest
		default:
		}
	}

	if needsReset {
		lock.Reset(npmPackage)
	}
}

func (lock *Lock) Reset(npmPackage string) {
	var keys []string

	for key := range lock.resolutions {
		if strings.HasPrefix(key, npmPackage) && key[len(npmPackage)] == '@' {
			keys = append(keys, key)
		}
	}

	if len(keys) > 0 {
		log.Printf("Reset %s", npmPackage)
	}

	for _, key := range keys {
		lock.dirty = true
		delete(lock.resolutions, key)
	}
}

func (lock *Lock) Shrink() {
	npmPackages := make(map[string]int)
	for key := range lock.resolutions {
		npmPackages[key[:1+strings.Index(key[1:], "@")]] += 1
	}

	for npmPackage, count := range npmPackages {
		if count > 1 {
			lock.shrink(npmPackage)
		}
	}
}

func (lock *Lock) read(npmPackage string) (map[string]Resolution, map[string]Resolution) {
	resolutions := make(map[string]Resolution)
	versions := make(map[string]Resolution)

	for key, resolution := range lock.resolutions {
		if strings.HasPrefix(key, npmPackage) && key[len(npmPackage)] == '@' {
			for _, sub := range strings.Split(key, ", ") {
				resolutions[sub] = resolution
				versions[resolution.Version] = resolution
			}
		}
	}

	return resolutions, versions
}

func requestFromKey(key string) string {
	afterColon := key[strings.LastIndex(key, ":")+1:]
	if loc := strings.LastIndex(afterColon, "%3A"); loc >= 0 {
		return afterColon[strings.LastIndex(afterColon, "%3A")+3:]
	} else {
		return afterColon
	}
}

func (lock *Lock) shrink(npmPackage string) {
	resolutions, versions := lock.read(npmPackage)

	for key, resolution := range lock.resolutions {
		if strings.HasPrefix(key, npmPackage) && key[len(npmPackage)] == '@' {
			for _, sub := range strings.Split(key, ", ") {
				resolutions[sub] = resolution
				versions[resolution.Version] = resolution
			}
		}
	}

	for key, value := range resolutions {
		requested := requestFromKey(key)

		if strings.HasPrefix(requested, npmPackage+"@") {
			requested = requested[len(npmPackage)+1:]
		}

		request, version := semver.MustParseRequest(requested), semver.MustParseVersion(value.Version)
		for presentSource := range versions {
			present := semver.MustParseVersion(presentSource)
			if version.AtLeast().Matches(present) && request.Matches(present) {
				version = present
			}
		}

		if value.Version != version.String() {
			resolutions[key] = versions[version.String()]
		}
	}

	next := make(map[string]Resolution)
	dirty := false
	for version, resolution := range versions {
		var keys []string
		for key, resolution := range resolutions {
			if resolution.Version == version {
				keys = append(keys, key)
			}
		}

		if len(keys) == 0 {
			dirty = true
			log.Printf("Drop %s %s", npmPackage, resolution.Version)
			continue
		}

		sort.Slice(keys, func(p, q int) bool { return keys[p] < keys[q] })
		keyCsv := strings.Join(keys, ", ")
		next[keyCsv] = resolution
		if lock.resolutions[keyCsv].Version != resolution.Version {
			dirty = true
			log.Printf("Save %s %s", npmPackage, resolution.Version)
		}
	}

	if dirty {
		lock.Reset(npmPackage)
		for keyCsv, resolution := range next {
			lock.resolutions[keyCsv] = resolution
		}
	}
}

func (lock *Lock) printSuggestions() {
	if len(lock.suggestions) == 0 {
		return
	}

	var keys []string

	for key := range lock.suggestions {
		keys = append(keys, key)
	}

	sort.Slice(keys, func(p, q int) bool { return keys[p] < keys[q] })

	log.Printf("Suggested resolutions")

	for _, key := range keys {
		fmt.Printf("    \"%s\": \"^%s\",\n", key, lock.suggestions[key].String())
	}
}

func (lock *Lock) Save(directory string) (bool, error) {
	lock.printSuggestions()

	if !lock.dirty {
		log.Printf("yarn.lock stable")
		return false, nil
	}

	yaml, err := yaml2.Marshal(lock.resolutions)
	if err != nil {
		return true, fmt.Errorf("cannot serialize yarn.lock: %v", err)
	}

	log.Printf("yarn.lock dirty, needs `yarn install`")
	return true, ioutil.WriteFile(yarnLock(directory), yaml, fs.ModePerm)
}
