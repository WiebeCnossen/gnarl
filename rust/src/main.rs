mod semver;
mod yarn;

use std::env;
use std::process::{Command, exit};
use yarn::{Package, Lock};

const VERSION: &str = "1.0.0-rc-2";

fn help() {
    println!("gnarl {} - the yarn v2/v3 companion tool", VERSION);
    println!("usage: gnarl [<auto | audit | check | fix | help | reset> <args>]");
    println!("> gnarl [auto]");
    println!("> gnarl audit");
    println!("> gnarl check");
    println!("> gnarl fix package-name safe-version-request");
    println!("> gnarl help");
    println!("> gnarl shrink");
    println!("> gnarl reset package-names...");
}

fn must_read_package() -> Package {
    Package::read(".").unwrap_or_else(|e| {
        eprintln!("Error reading package.json: {}", e);
        exit(1);
    })
}

fn must_read_lock() -> Lock {
    Lock::read(".").unwrap_or_else(|e| {
        eprintln!("Error reading yarn.lock: {}", e);
        exit(1);
    })
}

fn must_save_lock(lock: &mut Lock) -> bool {
    lock.save(".").unwrap_or_else(|e| {
        eprintln!("Error saving yarn.lock: {}", e);
        exit(1);
    })
}

fn audit(project: &Package) -> bool {
    let output = Command::new("yarn")
        .arg("--version")
        .output()
        .unwrap_or_else(|e| {
            eprintln!("Error running yarn --version: {}", e);
            exit(1);
        });

    let version_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let version = semver::Version::parse(&version_str).unwrap_or_else(|e| {
        eprintln!("Error parsing yarn version: {}", e);
        exit(1);
    });

    println!("yarn npm audit --recursive");
    let output = Command::new("yarn")
        .args(&["npm", "audit", "--json", "--recursive"])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            if version.major < 4 {
                eprintln!("Error running yarn npm audit: {}", e);
                exit(1);
            } else {
                // Yarn 4+ might return non-zero exit code even with valid output
                Command::new("yarn")
                    .args(&["npm", "audit", "--json", "--recursive"])
                    .output()
                    .unwrap_or_else(|_| {
                        eprintln!("Error running yarn npm audit: {}", e);
                        exit(1);
                    })
            }
        }
    };

    let advisories = yarn::parse_audit(&output.stdout, &version).unwrap_or_else(|e| {
        eprintln!("Error parsing audit: {}", e);
        exit(1);
    });

    let mut lock = must_read_lock();

    for advisory in &advisories {
        let request = semver::Request::parse(&advisory.patched_versions).unwrap_or_else(|e| {
            eprintln!("invalid safe-version-request: {}", e);
            exit(1);
        });

        lock.fix(&advisory.module_name, &request);
    }

    if advisories.is_empty() {
        println!("all packages considered safe");
    }

    if version.major < 4 {
        check(project, &lock);
    }

    must_save_lock(&mut lock)
}

fn check(project: &Package, lock: &Lock) {
    let mut dirty = false;
    
    for (key, r) in &project.resolutions {
        let parts: Vec<&str> = key.split('@').collect();
        let npm_package = parts[0];
        let request = if parts.len() == 1 {
            if let Ok(v) = semver::Request::parse(r) {
                if !v.is_exact() {
                    dirty = true;
                    println!("unrestricted resolution for {}", npm_package);
                }
            }
            "*"
        } else {
            parts[1]
        };

        if !lock.has(npm_package, request) {
            dirty = true;
            println!("superfluous resolution for {}", key);
        }
    }

    if !dirty {
        println!("all resolutions good");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let verb = if args.len() > 1 {
        match args[1].as_str() {
            "audit" | "check" | "fix" | "help" | "reset" | "shrink" => args[1].as_str(),
            _ => {
                eprintln!("unknown verb: {}", args[1]);
                exit(1);
            }
        }
    } else {
        "auto"
    };

    let project = if verb != "help" {
        Some(must_read_package())
    } else {
        None
    };

    match verb {
        "auto" => {
            loop {
                println!("yarn install");
                Command::new("yarn")
                    .arg("install")
                    .output()
                    .unwrap_or_else(|e| {
                        eprintln!("Error running yarn install: {}", e);
                        exit(1);
                    });

                println!("yarn dedupe");
                Command::new("yarn")
                    .arg("dedupe")
                    .output()
                    .unwrap_or_else(|e| {
                        eprintln!("Error running yarn dedupe: {}", e);
                        exit(1);
                    });

                if !audit(project.as_ref().unwrap()) {
                    break;
                }
            }
        }
        "audit" => {
            audit(project.as_ref().unwrap());
        }
        "check" => {
            let lock = must_read_lock();
            check(project.as_ref().unwrap(), &lock);
        }
        "fix" => {
            if args.len() < 4 {
                help();
                eprintln!("insufficient arguments");
                exit(1);
            }

            let npm_package = &args[2];
            let request_str = args[3..].join(" ");
            let request = semver::Request::parse(&request_str).unwrap_or_else(|e| {
                eprintln!("invalid safe-version-request: {}", e);
                exit(1);
            });

            let mut lock = must_read_lock();
            lock.fix(npm_package, &request);
            must_save_lock(&mut lock);
        }
        "help" => {
            help();
        }
        "reset" => {
            let mut lock = must_read_lock();
            for arg in args.iter().skip(1) {
                lock.reset(arg);
            }
            must_save_lock(&mut lock);
        }
        "shrink" => {
            let mut lock = must_read_lock();
            lock.shrink();
            must_save_lock(&mut lock);
        }
        _ => {
            eprintln!("unreachable verb {}", verb);
            exit(1);
        }
    }
}
