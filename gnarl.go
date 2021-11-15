package main

import (
	"gnarl/semver"
	"gnarl/yarn"
	"log"
	"os"
	"os/exec"
	"strings"
)

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

const version string = "1.0.0-beta-8"

func help() {
	log.Printf("gnarl %s - the yarn v2/v3 companion tool", version)
	log.Print("Usage: gnarl [<auto | fix | help | reset> <args>]")
	log.Print("> gnarl [auto] [reset-package-names...]")
	log.Print("> gnarl fix package-name safe-version-request")
	log.Print("> gnarl help")
	log.Print("> gnarl reset package-names...")
}

func main() {
	var verb string
	if len(os.Args) > 1 {
		switch os.Args[1] {
		case "deprecated-shrink":
		case "fix":
		case "help":
		case "reset":
			verb = os.Args[1]
		default:
			verb = "auto"
		}
	} else {
		verb = "auto"
	}

	var lock *yarn.Lock
	switch verb {
	case "auto":
		first := true
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

			log.Print("yarn npm audit --recursive")
			out, err := exec.Command("yarn", "npm", "audit", "--json", "--recursive").Output()
			if err != nil {
				log.Fatal(err)
			}

			advisories, err := yarn.ParseAudit(out)
			if err != nil {
				log.Fatal(err)
			}

			lock = mustReadLock()

			if first {
				first = false
				for _, arg := range os.Args {
					lock.Reset(arg)
				}
			}

			for _, advisory := range advisories {
				request, err := semver.ParseRequest(advisory.PatchedVersions)
				if err != nil {
					log.Fatalf("Invalid safe-version-request: %v", err)
				}

				lock.Fix(advisory.ModuleName, request)
			}

			if !mustSaveLock(lock) {
				break
			}
		}
	case "deprecated-shrink":
		lock = mustReadLock()
		lock.Shrink()
		mustSaveLock(lock)
	case "fix":
		if len(os.Args) < 4 {
			help()
			log.Fatal("Insufficient arguments")
		}

		npmPackage := os.Args[2]
		request, err := semver.ParseRequest(strings.Join(os.Args[3:], " "))
		if err != nil {
			log.Fatalf("Invalid safe-version-request: %v", err)
		}

		lock = mustReadLock()
		lock.Fix(npmPackage, request)
		mustSaveLock(lock)
	case "help":
		help()
	case "reset":
		lock = mustReadLock()

		for _, arg := range os.Args[1:] {
			lock.Reset(arg)
		}

		mustSaveLock(lock)
	default:
		log.Fatalf("Unreachable verb %s", verb)
	}
}
