package semver

import (
	"fmt"
	"log"
	"strconv"
	"strings"
	"unicode"
)

type Version struct {
	Major, Minor, Patch int
	Pre                 string
}

func MustParseVersion(version string) *Version {
	result, err := ParseVersion(version)
	if err != nil {
		log.Fatalf("failed to parse version %s: %v", version, err)
	}

	return result
}

func ParseVersion(version string) (*Version, error) {
	hash := ""
	if loc := strings.Index(version, "#"); loc >= 0 {
		version, hash = version[:loc], version[loc:]
	}
	parts := strings.SplitN(version, ".", 3)
	major, err := strconv.ParseInt(parts[0], 10, 0)
	if err != nil {
		return nil, fmt.Errorf("invalid major %s: %v", parts[0], err)
	}

	var minor, patch int64
	var pre string
	if len(parts) > 1 {
		minor, err = strconv.ParseInt(parts[1], 10, 0)
		if err != nil {
			return nil, fmt.Errorf("invalid minor %s: %v", parts[1], err)
		}

	}

	if len(parts) > 2 {
		var i int
		for i = 0; i < len(parts[2]) && unicode.IsDigit(rune(parts[2][i])); i++ {
		}

		pre = parts[2][i:]
		patch, err = strconv.ParseInt(parts[2][:i], 10, 0)
		if err != nil {
			return nil, fmt.Errorf("invalid patch %s: %v", parts[1], err)
		}
	}

	return &Version{Major: int(major), Minor: int(minor), Patch: int(patch), Pre: pre + hash}, nil
}

func (v *Version) AtLeast() *Request {
	factors := []RequestFactor{{Constraint: AtLeast, Version: *v}}
	return &Request{terms: []RequestTerm{factors}}
}

func (v *Version) String() string {
	return fmt.Sprintf("%d.%d.%d%s", v.Major, v.Minor, v.Patch, v.Pre)
}
