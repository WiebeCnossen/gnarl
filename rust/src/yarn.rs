use crate::{lock::Lock, package::Package};
use std::{
    io::Error,
    path::PathBuf,
    process::{Command, Output},
};

const AIKIDO_YARN_NAME: &'static str = "aikido-yarn";
const YARN_NAME: &'static str = "yarn";

pub struct Yarn {
    aikido_path: Option<PathBuf>,
    yarn_path: PathBuf,
    package: Package,
    lock: Lock,
}

const YARN_NOT_FOUND: &'static str = "yarn not installed or not in PATH";
const PACKAGE_NOT_FOUND: &'static str = "package.json not found in current directory";
const LOCK_NOT_FOUND: &'static str = "yarn.lock not found in current directory";

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
        println!("# resolutions: {}", self.package.resolutions().len());
        println!("# dependencies: {}", self.package.dependencies().len());
        println!(
            "# dev dependencies: {}",
            self.package.dev_dependencies().len()
        );
        println!("# lock entries: {}", self.lock.len());
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
}
