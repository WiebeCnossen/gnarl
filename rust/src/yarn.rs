use crate::{
    Error,
    audit::Advisory,
    lock::Lock,
    package::{Dependency, Resolution},
    parse,
    project::Project,
};
use std::{
    path::PathBuf,
    process::{Command, Output},
};

const AIKIDO_YARN_NAME: &str = "aikido-yarn";
const YARN_NAME: &str = "yarn";

pub struct Yarn {
    aikido_path: Option<PathBuf>,
    yarn_path: PathBuf,
    project: Project,
    lock: Lock,
}

const PACKAGE_NOT_FOUND: &str = "package.json not found in current directory";
const LOCK_NOT_FOUND: &str = "yarn.lock not found in current directory";

impl Yarn {
    pub fn new() -> Result<Self, Error> {
        let aikido_path = which::which(AIKIDO_YARN_NAME).ok();
        let yarn_path = which::which(YARN_NAME)?;

        let package_path = PathBuf::from("package.json");
        if !package_path.exists() {
            return Err(PACKAGE_NOT_FOUND.into());
        }

        let project = Project::read(package_path)?;

        let lock_path = PathBuf::from("yarn.lock");
        if !lock_path.exists() {
            return Err(LOCK_NOT_FOUND.into());
        }

        let lock = Lock::read(lock_path)?;

        Ok(Self {
            aikido_path,
            yarn_path,
            project,
            lock,
        })
    }

    pub fn print_info(&self) {
        println!("# resolutions: {:11}", self.project.resolutions().len());
        println!("# dependencies: {:10}", self.project.dependencies().len());
        println!(
            "# dev dependencies: {:6}",
            self.project.dev_dependencies().len()
        );
        println!("# lock entries: {:10}", self.lock.len());
    }

    fn run(&self, prefer_aikido: bool, args: &[&str]) -> Result<Output, Error> {
        let executable = if prefer_aikido {
            self.aikido_path.as_ref().unwrap_or(&self.yarn_path)
        } else {
            &self.yarn_path
        };
        let name = if prefer_aikido && self.aikido_path.is_some() {
            AIKIDO_YARN_NAME
        } else {
            YARN_NAME
        };
        println!(
            "{} {}",
            name,
            args.iter()
                .take_while(|arg| !arg.starts_with("-"))
                .map(|arg| arg.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        );
        let output = Command::new(executable).args(args).output()?;
        if !output.status.success() && output.stdout.is_empty() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string().into());
        }
        Ok(output)
    }

    pub fn install(&self) -> Result<Output, Error> {
        self.run(true, &["install"])
    }

    pub fn dedupe(&self) -> Result<Output, Error> {
        self.run(false, &["dedupe"])
    }

    pub fn audit(&self) -> Result<Vec<Advisory>, Error> {
        let output = self.run(false, &["npm", "audit", "--json", "--recursive"])?;
        let stdout_str = String::from_utf8(output.stdout)?;
        let advisories: Vec<Advisory> = stdout_str
            .lines()
            .map(Advisory::parse)
            .collect::<Result<Vec<Advisory>, Error>>()?;
        Ok(advisories)
    }

    pub fn reset(&mut self, packages: &[impl AsRef<str>]) -> Result<bool, Error> {
        let mut dirty = false;
        for package in packages {
            dirty = self.lock.reset(package.as_ref()) || dirty;
        }

        if !dirty {
            return Ok(false);
        }

        self.lock.save()?;
        Ok(true)
    }

    pub fn dependents(&self, name: impl AsRef<str>) -> Result<Vec<Dependency>, Error> {
        self.lock.dependents(name.as_ref())
    }

    pub fn resolutions(&self, name: impl AsRef<str>) -> Result<Vec<Resolution>, Error> {
        self.lock.resolutions(name.as_ref())
    }

    pub fn reset_resolutions(&mut self) -> Result<bool, Error> {
        let mut dirty = false;
        for (package, requested) in self.project.resolutions() {
            if parse::parse_range(&requested).is_err() {
                continue;
            }

            let (name, tail) = parse::split_name(&package).unwrap_or((&package, "*"));
            let range = parse::parse_range(tail)?;
            let resolutions = self.lock.all_resolutions()?;
            if !resolutions.iter().any(|resolution| {
                resolution
                    .dependencies()
                    .iter()
                    .any(|dependency| dependency.name() == name && range.eq(dependency.request()))
            }) {
                println!("Drop resolution for {}", package);
                self.project.reset_resolution(&package);
                dirty = true;
            }
        }

        if !dirty {
            return Ok(false);
        }

        self.project.save()?;
        Ok(true)
    }
}
