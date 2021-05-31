package main

import (
	"gnarl/semver"
	"gnarl/yarn"
	"log"
	"os"
	"strings"
)

func mustReadLock() *yarn.Lock {
	lock, err := yarn.ReadLock(".")
	if err != nil {
		log.Fatal(err)
	}

	return lock
}

func mustSaveLock(lock *yarn.Lock) {
	err := lock.Save(".")
	if err != nil {
		log.Fatal(err)
	}
}

const version string = "1.0.0-beta-1"

func help() {
	log.Printf("gnarl %s - the yarn v2 companion tool", version)
	log.Print("Usage: gnarl <fix | help | reset | shrink> <args>")
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
