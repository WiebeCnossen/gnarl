use std::collections::HashMap;

use nodejs_semver::{Range, Version};
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::{
    Error, parse,
    ui::{ActivityKind, SharedReporter, UiEvent, stdout_reporter},
};

#[derive(Deserialize)]
pub struct PackumentVersionDto {
    dependencies: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
struct PackumentDto {
    name: String,
    versions: HashMap<String, PackumentVersionDto>,
    time: HashMap<String, String>,
}

#[derive(Clone)]
pub struct PackumentVersion {
    version: Version,
    dependencies: HashMap<String, Range>,
}

impl PackumentVersion {
    pub fn dependencies(&self) -> impl Iterator<Item = &str> {
        self.dependencies.keys().map(|key| key.as_str())
    }

    pub fn dependency(&self, dependency: &str) -> Result<&Range, crate::Error> {
        self.dependencies
            .get(dependency)
            .ok_or(format!("Dependency {} not found", dependency).into())
    }
}

#[derive(Clone)]
pub struct Packument {
    versions: Vec<PackumentVersion>,
}

impl Packument {
    pub fn versions(&self) -> impl Iterator<Item = &Version> {
        self.all_versions()
            .filter(|version| !version.is_prerelease())
    }

    pub fn all_versions(&self) -> impl Iterator<Item = &Version> {
        self.versions.iter().map(|version| &version.version)
    }

    pub fn version(&self, version: &Version) -> Result<&PackumentVersion, crate::Error> {
        self.versions
            .iter()
            .find(|v| &v.version == version)
            .ok_or(format!("Version {} not found", version).into())
    }

    fn from_dto(dto: PackumentDto, reporter: &dyn crate::ui::Reporter) -> Result<Self, Error> {
        let time = dto
            .time
            .into_iter()
            .map(|(key, value)| {
                let date = chrono::DateTime::parse_from_rfc3339(&value).map_err(
                    |e: chrono::ParseError| Error::String(format!("Invalid time {}: {}", key, e)),
                )?;
                Ok((key, date.into()))
            })
            .collect::<Result<HashMap<String, chrono::DateTime<chrono::Utc>>, crate::Error>>()?;
        let now_minus_1_day = chrono::Utc::now() - chrono::Duration::days(1);
        let mut versions: Vec<_> = dto
            .versions
            .into_iter()
            .flat_map(|(version, ver_dto)| {
                if let Some(&time) = time.get(&version) {
                    if time >= now_minus_1_day {
                        reporter.emit(UiEvent::Activity {
                            kind: ActivityKind::Npm,
                            message: format!(
                                "{}@{} disregarded until {}",
                                dto.name,
                                version,
                                time + chrono::Duration::days(1)
                            ),
                        });
                        return None;
                    }
                } else {
                    reporter.emit(UiEvent::Activity {
                        kind: ActivityKind::Npm,
                        message: format!("{}@{} has no release date", dto.name, version),
                    });
                    return None;
                }

                let ver = parse::parse_version(&version).ok()?;
                let dependencies = match ver_dto.dependencies {
                    Some(deps) => {
                        let mut result = HashMap::new();
                        for (key, value) in deps {
                            if let Ok(range) = parse::parse_range(&value) {
                                result.insert(key, range);
                            }
                        }
                        result
                    }
                    None => HashMap::new(),
                };
                Some(PackumentVersion {
                    version: ver,
                    dependencies,
                })
            })
            .collect::<Vec<_>>();
        versions.sort_unstable_by(|a: &PackumentVersion, b: &PackumentVersion| {
            a.version.cmp(&b.version)
        });
        Ok(Self { versions })
    }
}

pub struct Npm {
    client: Client,
    packuments: HashMap<String, Packument>,
    reporter: SharedReporter,
}

impl Npm {
    pub fn new() -> Result<Self, crate::Error> {
        Self::with_reporter(stdout_reporter())
    }

    pub fn with_reporter(reporter: SharedReporter) -> Result<Self, crate::Error> {
        let client = Client::builder()
            .user_agent("gnarl/2.0.0 (https://github.com/WiebeCnossen/gnarl)")
            .build()?;

        Ok(Self {
            client,
            packuments: HashMap::new(),
            reporter,
        })
    }

    pub fn retrieve_packument(&mut self, package: &str) -> Result<(), crate::Error> {
        if self.packuments.contains_key(package) {
            return Ok(());
        }

        self.reporter.emit(UiEvent::Activity {
            kind: ActivityKind::Npm,
            message: format!("query {package}"),
        });
        let url = format!("https://registry.npmjs.org/{package}");
        let response = self.client.get(&url).send()?.error_for_status()?;
        let packument: PackumentDto = serde_json::from_reader(response)?;
        let name = packument.name.clone();
        let packument = Packument::from_dto(packument, &*self.reporter)?;
        self.packuments.insert(name, packument);
        Ok(())
    }

    pub fn packument(&self, package: &str) -> Result<&Packument, crate::Error> {
        self.packuments
            .get(package)
            .ok_or(format!("Packument for {} not retrieved", package).into())
    }
}
