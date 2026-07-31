use nodejs_semver::{OutsideDirection, Range};

use crate::{
    Error,
    audit::{Advisory, Severity},
    locks::Locks,
    out_fix, out_hit, out_info, out_yarn, parse,
    package::Dependency,
    project::Project,
    yarnrc::YarnRc,
};
use std::{
    cmp::Ordering,
    path::PathBuf,
    process::{Command, Output},
};

const AIKIDO_YARN_NAME: &str = "aikido-yarn";
const SAFE_CHAIN_NAME: &str = "safe-chain";
const YARN_NAME: &str = "yarn";

pub struct Yarn {
    aikido_path: Option<PathBuf>,
    safe_path: Option<PathBuf>,
    yarn_path: PathBuf,
    lock_path: PathBuf,
    project: Project,
    severity: Severity,
    locks: Option<Locks>,
}

const PACKAGE_NOT_FOUND: &str = "package.json not found in current directory";
const LOCK_NOT_FOUND: &str = "yarn.lock not found in current directory";

impl Yarn {
    pub fn new(severity: Severity) -> Result<Self, Error> {
        let aikido_path = which::which(AIKIDO_YARN_NAME).ok();
        let safe_path = which::which(SAFE_CHAIN_NAME).ok();
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
            safe_path,
            yarn_path,
            lock_path,
            project,
            severity,
            locks: None,
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
        let mut args = args.to_vec();
        let (executable, name) = match prefer_aikido {
            true if let Some(ref path) = self.safe_path => {
                args.insert(0, "yarn");
                (path, "safe-chain install")
            }
            true if let Some(ref path) = self.aikido_path => (path, "aikido-yarn install"),
            _ if args.len() > 1 => (&self.yarn_path, "audit"),
            _ => (&self.yarn_path, args[0]),
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

    pub fn yarnrc(&self) -> Result<YarnRc, Error> {
        YarnRc::read(PathBuf::from(".yarnrc.yml"))
    }

    pub fn audit(&self) -> Result<Vec<Advisory>, Error> {
        Ok(self.filter_by_severity(self.parse_audit(self.run_audit()?)?))
    }

    /// Full audit with `npmAuditIgnoreAdvisories` temporarily cleared and no severity
    /// threshold applied, so ignored IDs of any severity remain visible for hygiene.
    /// Restores the previous ignore list even if the audit fails.
    pub fn audit_unfiltered(&self) -> Result<Vec<Advisory>, Error> {
        let mut yarnrc = self.yarnrc()?;
        let saved = yarnrc.npm_audit_ignore_advisories();
        if saved.is_empty() {
            return self.parse_audit(self.run_audit()?);
        }

        yarnrc.set_npm_audit_ignore_advisories(&[]);
        yarnrc.save()?;

        let result = self.run_audit().and_then(|output| self.parse_audit(output));

        yarnrc.set_npm_audit_ignore_advisories(&saved);
        let restore = yarnrc.save();

        match (result, restore) {
            (Ok(advisories), Ok(())) => Ok(advisories),
            (Err(err), _) => Err(err),
            (Ok(_), Err(err)) => Err(err),
        }
    }

    fn run_audit(&self) -> Result<Output, Error> {
        // Fetch all severities; gnarl applies `-s` itself so ignore hygiene can see
        // below-threshold advisories and not treat them as orphans.
        self.run(false, &["npm", "audit", "--json", "--recursive"])
    }

    fn parse_audit(&self, output: Output) -> Result<Vec<Advisory>, Error> {
        let stdout_str = String::from_utf8(output.stdout)?;
        stdout_str
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(Advisory::parse)
            .collect()
    }

    fn filter_by_severity(&self, advisories: Vec<Advisory>) -> Vec<Advisory> {
        advisories
            .into_iter()
            .filter(|advisory| advisory.severity().meets_threshold(self.severity))
            .collect()
    }

    pub fn locks(&mut self) -> Result<&mut Locks, Error> {
        if self.locks.is_none() {
            self.locks = Some(Locks::read(self.lock_path.clone())?);
        }
        Ok(self.locks.as_mut().unwrap())
    }

    pub fn reset_resolutions(&mut self) -> Result<bool, Error> {
        let mut dirty = false;
        let lock_dependencies: Vec<Dependency> = self
            .locks()?
            .all()
            .iter()
            .flat_map(|resolution| resolution.dependencies().iter().cloned())
            .collect();
        for (package, requested) in self.project.resolutions() {
            if parse::parse_range(&requested).is_err() {
                continue;
            }

            let (name, tail) = parse::split_name(&package).unwrap_or((&package, "*"));
            // Bare package keys (no `@…`) apply to every request range for that name.
            let match_all_ranges = tail == "*";
            // Yarn Berry descriptor keys look like `pkg@npm:^1.1.7`; strip the protocol.
            let range = match parse::parse_qualified_range(tail) {
                Ok((_, range)) => range,
                Err(_) => parse::parse_range(tail)?,
            };
            let requested_range = parse::parse_range(&requested)?;
            let mut needed = false;

            // warn if any resolution has a dependency with a higher minimum version than range
            if let Some(range_min_version) = requested_range.min_version() {
                for dependency in lock_dependencies.iter().filter(|d| {
                    d.name() == name && (d.request().eq(&range) || match_all_ranges)
                }) {
                    if matches!(
                        dependency.request().outside(
                            &range_min_version,
                            OutsideDirection::Lower,
                            false,
                        ),
                        Ok(true)
                    ) {
                        out_hit!(
                            "{} resolved to {} but request is {}",
                            name,
                            range_min_version,
                            dependency.request()
                        );
                        continue;
                    }

                    if matches!(
                        dependency.request().outside(
                            &range_min_version,
                            OutsideDirection::Higher,
                            false,
                        ),
                        Ok(true)
                    ) {
                        out_info!(
                            "{}@{} forced to {}",
                            name,
                            dependency.request(),
                            requested_range
                        );
                        needed = true;
                        continue;
                    }

                    // Check if the upper bound of requested range is less than the lower bound of dependency.request()
                    match is_capped(dependency.request(), &requested_range) {
                        Ordering::Less => {
                            out_info!(
                                "{}@{} capped to {}",
                                name,
                                dependency.request(),
                                requested_range
                            );
                            needed = true;
                        }
                        Ordering::Greater => {
                            out_info!(
                                "{}@{} expanded to {}",
                                name,
                                dependency.request(),
                                requested_range
                            );
                        }
                        _ => {}
                    }
                }
            }

            if !needed {
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

fn is_capped(range: &Range, cap: &Range) -> Ordering {
    let min_versions = range
        .min_version()
        .iter()
        .chain(cap.min_version().iter())
        .cloned()
        .collect::<Vec<_>>();
    if let Some(min_version) = cap.min_satisfying(&min_versions)
        && let Ok(cut) = Range::parse(format!("< {}", min_version))
    {
        if let Some(r) = range.difference(&cut)
            && r.difference(cap).is_some()
        {
            return Ordering::Less;
        }

        if let Some(r) = cap.difference(&cut)
            && r.difference(range).is_some()
        {
            return Ordering::Greater;
        }
    }

    Ordering::Equal
}
