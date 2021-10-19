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

const version string = "1.0.0-beta-5"

func help() {
	log.Printf("gnarl %s - the yarn v2/v3 companion tool", version)
	log.Print("Usage: gnarl <fix | help | reset | shrink> <args>")
	log.Print("> gnarl auto")
	log.Print("> gnarl fix package-name safe-version-request")
	log.Print("> gnarl help")
	log.Print("> gnarl reset package-names...")
	log.Print("> gnarl shrink")
}

func main() {
	var verb string
	if len(os.Args) > 1 {
		verb = os.Args[1]
	} else {
		help()
		log.Fatal("No verb specified")
	}

	var lock *yarn.Lock
	switch verb {
	case "auto":
		for {
			log.Print("yarn install")
			_, err := exec.Command("yarn", "install").Output()
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

		for i, arg := range os.Args {
			if i > 1 {
				lock.Reset(arg)
			}
		}

		mustSaveLock(lock)
	case "shrink":
		lock = mustReadLock()
		lock.Shrink()
		mustSaveLock(lock)
	default:
		help()
		log.Fatalf("Unknown verb %s", verb)
	}

}
