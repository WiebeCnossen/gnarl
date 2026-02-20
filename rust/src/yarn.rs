use crate::{
    Error,
    audit::{Advisory, Severity},
    locks::Locks,
    out_fix, out_yarn, parse,
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
    lock_path: PathBuf,
    project: Project,
    severity: Severity,
}

const PACKAGE_NOT_FOUND: &str = "package.json not found in current directory";
const LOCK_NOT_FOUND: &str = "yarn.lock not found in current directory";

impl Yarn {
    pub fn new(severity: Severity) -> Result<Self, Error> {
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

        Ok(Self {
            aikido_path,
            yarn_path,
            lock_path,
            project,
            severity,
        })
    }

    pub fn len_dependencies(&self) -> usize {
        self.project.dependencies().len()
    }

    pub fn len_dev_dependencies(&self) -> usize {
        self.project.dev_dependencies().len()
    }

    pub fn len_resolutions(&self) -> usize {
        self.project.resolutions().len()
    }

    fn run(&self, prefer_aikido: bool, args: &[&str]) -> Result<Output, Error> {
        let executable = if prefer_aikido {
            self.aikido_path.as_ref().unwrap_or(&self.yarn_path)
        } else {
            &self.yarn_path
        };
        let name = if prefer_aikido && self.aikido_path.is_some() {
            "aikido install"
        } else if args.len() > 1 {
            "audit"
        } else {
            args[0]
        };
        out_yarn!("{}", name);
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
        let output = self.run(
            false,
            &[
                "npm",
                "audit",
                "--json",
                "--recursive",
                "--severity",
                self.severity.to_string().as_str(),
            ],
        )?;
        let stdout_str = String::from_utf8(output.stdout)?;
        let advisories: Vec<Advisory> = stdout_str
            .lines()
            .map(Advisory::parse)
            .collect::<Result<Vec<Advisory>, Error>>()?;
        Ok(advisories)
    }

    pub fn locks(&self) -> Result<Locks, Error> {
        Locks::read(self.lock_path.clone())
    }

    pub fn reset_resolutions(&mut self) -> Result<bool, Error> {
        let mut dirty = false;
        let resolutions = self.locks()?.all()?;
        for (package, requested) in self.project.resolutions() {
            if parse::parse_range(&requested).is_err() {
                continue;
            }

            let (name, tail) = parse::split_name(&package).unwrap_or((&package, "*"));
            let range = parse::parse_range(tail)?;
            let requested_range = parse::parse_range(&requested)?;
            if range.intersect(&requested_range).is_some()
                || !resolutions.iter().any(|resolution| {
                    resolution.dependencies().iter().any(|dependency| {
                        dependency.name() == name && range.eq(dependency.request())
                    })
                })
            {
                out_fix!("drop resolution for {}", package);
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
