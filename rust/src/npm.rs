use std::collections::HashMap;

use nodejs_semver::{Range, Version};
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::parse;

#[derive(Deserialize)]
pub struct PackumentVersionDto {
    dependencies: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
struct PackumentDto {
    name: String,
    versions: HashMap<String, PackumentVersionDto>,
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
}

impl TryFrom<PackumentDto> for Packument {
    type Error = crate::Error;
    fn try_from(dto: PackumentDto) -> Result<Self, Self::Error> {
        let mut versions: Vec<_> = dto
            .versions
            .into_iter()
            .map(|(version, ver_dto)| {
                let ver = parse::parse_version(&version)?;
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
                Ok(PackumentVersion {
                    version: ver,
                    dependencies,
                })
            })
            .collect::<Result<Vec<_>, crate::Error>>()?;
        versions.sort_unstable_by(|a: &PackumentVersion, b: &PackumentVersion| {
            a.version.cmp(&b.version)
        });
        Ok(Self { versions })
    }
}

pub struct Npm {
    client: Client,
    packuments: HashMap<String, Packument>,
}

impl Npm {
    pub fn new() -> Result<Self, crate::Error> {
        let client = Client::builder()
            .user_agent("gnarl/2.0.0 (https://github.com/WiebeCnossen/gnarl)")
            .build()?;

        Ok(Self {
            client,
            packuments: HashMap::new(),
        })
    }

    pub fn retrieve_packument(&mut self, package: &str) -> Result<(), crate::Error> {
        let url = format!("https://registry.npmjs.org/{}", package);
        let response = self.client.get(&url).send()?.error_for_status()?;
        let packument: PackumentDto = serde_json::from_reader(response)?;
        self.packuments
            .insert(packument.name.to_owned(), packument.try_into()?);
        Ok(())
    }

    pub fn packument(&self, package: &str) -> Result<&Packument, crate::Error> {
        self.packuments
            .get(package)
            .ok_or(format!("Packument for {} not retrieved", package).into())
    }
}
