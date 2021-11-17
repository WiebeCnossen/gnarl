package main

import (
	"gnarl/semver"
	"gnarl/yarn"
	"log"
	"os"
	"os/exec"
	"strings"
)

func mustReadPackage() *yarn.Package {
	lock, err := yarn.ReadPackage(".")
	if err != nil {
		log.Fatal(err)
	}

	return lock
}

func mustReadLock() *yarn.Lock {
	lock, err := yarn.ReadLock(".")
	if err != nil {
		log.Fatal(err)
	}

	return lock
}

func mustSaveLock(lock *yarn.Lock) bool {
	dirty, err := lock.Save(".")
	if err != nil {
		log.Fatal(err)
	}

	return dirty
}

const version string = "1.0.0-rc-1"

func help() {
	log.Printf("gnarl %s - the yarn v2/v3 companion tool", version)
	log.Print("usage: gnarl [<auto | audit | check | fix | help | reset> <args>]")
	log.Print("> gnarl [auto]")
	log.Print("> gnarl audit")
	log.Print("> gnarl check")
	log.Print("> gnarl fix package-name safe-version-request")
	log.Print("> gnarl help")
	log.Print("> gnarl shrink")
	log.Print("> gnarl reset package-names...")
}

func main() {
	var verb string
	if len(os.Args) > 1 {
		switch os.Args[1] {
		case "audit":
		case "check":
		case "fix":
		case "help":
		case "reset":
		case "shrink":
		default:
			log.Fatalf("unknown verb: %s", os.Args[1])
		}
		verb = os.Args[1]
	} else {
		verb = "auto"
	}

	project := mustReadPackage()

	switch verb {
	case "auto":
		for {
			log.Print("yarn install")
			_, err := exec.Command("yarn", "install").Output()
			if err != nil {
				log.Fatal(err)
			}

			log.Print("yarn dedupe")
			_, err = exec.Command("yarn", "dedupe").Output()
			if err != nil {
				log.Fatal(err)
			}

			if !audit(project) {
				break
			}
		}

	case "audit":
		audit(project)

	case "check":
		lock := mustReadLock()
		check(project, lock)

	case "fix":
		if len(os.Args) < 4 {
			help()
			log.Fatal("insufficient arguments")
		}

		npmPackage := os.Args[2]
		request, err := semver.ParseRequest(strings.Join(os.Args[3:], " "))
		if err != nil {
			log.Fatalf("invalid safe-version-request: %v", err)
		}

		lock := mustReadLock()
		lock.Fix(npmPackage, request)
		mustSaveLock(lock)

	case "help":
		help()

	case "reset":
		lock := mustReadLock()

		for _, arg := range os.Args[1:] {
			lock.Reset(arg)
		}

		mustSaveLock(lock)

	case "shrink":
		lock := mustReadLock()
		lock.Shrink()
		mustSaveLock(lock)

	default:
		log.Fatalf("unreachable verb %s", verb)
	}
}

func audit(project *yarn.Package) bool {
	log.Print("yarn npm audit --recursive")
	out, err := exec.Command("yarn", "npm", "audit", "--json", "--recursive").Output()
	if err != nil {
		log.Fatal(err)
	}

	advisories, err := yarn.ParseAudit(out)
	if err != nil {
		log.Fatal(err)
	}

	lock := mustReadLock()

	for _, advisory := range advisories {
		request, err := semver.ParseRequest(advisory.PatchedVersions)
		if err != nil {
			log.Fatalf("invalid safe-version-request: %v", err)
		}

		lock.Fix(advisory.ModuleName, request)
	}

	if len(advisories) == 0 {
		log.Print("all packages considered safe")
	}

	check(project, lock)

	return mustSaveLock(lock)
}

func check(project *yarn.Package, lock *yarn.Lock) {
	dirty := false
	for key, r := range project.Resolutions {
		parts := strings.Split(key, "@")
		npmPackage := parts[0]
		request := "*"
		switch len(parts) {
		case 1:
			if v, err := semver.ParseRequest(r); err == nil && !v.IsExact() {
				dirty = true
				log.Printf("unrestricted resolution for %s", npmPackage)
			}
		default:
			request = parts[1]
		}

		if !lock.Has(npmPackage, request) {
			dirty = true
			log.Printf("superfluous resolution for %s", key)
		}
	}

	if !dirty {
		log.Print("all resolutions good")
	}
}
