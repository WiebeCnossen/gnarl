use crate::{lock::Lock, package::Package};
use std::{
    io::Error,
    path::PathBuf,
    process::{Command, Output},
};

const AIKIDO_YARN_NAME: &str = "aikido-yarn";
const YARN_NAME: &str = "yarn";

pub struct Yarn {
    aikido_path: Option<PathBuf>,
    yarn_path: PathBuf,
    package: Package,
    lock: Lock,
}

const YARN_NOT_FOUND: &str = "yarn not installed or not in PATH";
const PACKAGE_NOT_FOUND: &str = "package.json not found in current directory";
const LOCK_NOT_FOUND: &str = "yarn.lock not found in current directory";

impl From<which::Error> for crate::Error {
    fn from(_: which::Error) -> Self {
        YARN_NOT_FOUND.into()
    }
}

impl Yarn {
    pub fn new() -> Result<Self, crate::Error> {
        let aikido_path = which::which(AIKIDO_YARN_NAME).ok();
        let yarn_path = which::which(YARN_NAME)?;

        let package_path = PathBuf::from("package.json");
        if !package_path.exists() {
            return Err(PACKAGE_NOT_FOUND.into());
        }

        let package = Package::read(package_path)?;

        let lock_path = PathBuf::from("yarn.lock");
        if !lock_path.exists() {
            return Err(LOCK_NOT_FOUND.into());
        }

        let lock = Lock::read(lock_path)?;

        Ok(Self {
            aikido_path,
            yarn_path,
            package,
            lock,
        })
    }

    pub fn print_info(&self) {
        println!("# resolutions: {:11}", self.package.resolutions().len());
        println!("# dependencies: {:10}", self.package.dependencies().len());
        println!(
            "# dev dependencies: {:6}",
            self.package.dev_dependencies().len()
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
        println!("{} {}", name, args.join(" "));
        Command::new(executable).args(args).output()
    }

    pub fn install(&self) -> Result<Output, Error> {
        self.run(true, &["install"])
    }

    pub fn dedupe(&self) -> Result<Output, Error> {
        self.run(false, &["dedupe"])
    }

    pub fn audit(&self) -> Result<Output, Error> {
        self.run(false, &["npm", "audit", "--json", "--recursive"])
    }

    pub fn resolve(
        &mut self,
        package: impl AsRef<str>,
        request: impl AsRef<str>,
    ) -> Result<(), crate::Error> {
        self.package.resolve(package.as_ref(), request.as_ref());
        self.package.save()
    }

    pub fn reset(&mut self, packages: &[impl AsRef<str>]) -> Result<bool, crate::Error> {
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
}
