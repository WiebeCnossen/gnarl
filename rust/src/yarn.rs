use crate::semver;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Cursor};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Package {
    #[serde(default)]
    pub resolutions: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Lock {
    #[serde(flatten)]
    resolutions: HashMap<String, Resolution>,
    #[serde(skip)]
    dirty: bool,
    #[serde(skip)]
    suggestions: HashMap<String, semver::Version>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Resolution {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies_meta: Option<HashMap<String, DependencyMeta>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_dependencies: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_dependencies_meta: Option<HashMap<String, DependencyMeta>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DependencyMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Advisory {
    #[serde(rename = "module_name")]
    pub module_name: String,
    #[serde(rename = "patched_versions")]
    pub patched_versions: String,
}

impl Package {
    pub fn read(directory: &str) -> Result<Self, String> {
        let path = Path::new(directory).join("package.json");
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("cannot open package.json: {}", e))?;
        
        serde_json::from_str(&content)
            .map_err(|e| format!("cannot deserialize package.json: {}", e))
    }
}

impl Lock {
    pub fn read(directory: &str) -> Result<Self, String> {
        let path = Path::new(directory).join("yarn.lock");
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("cannot open yarn.lock: {}", e))?;
        
        let resolutions: HashMap<String, Resolution> = serde_yaml::from_str(&content)
            .map_err(|e| format!("cannot deserialize yarn.lock: {}", e))?;
        
        if resolutions.is_empty() {
            return Err("no entries found in yarn.lock".to_string());
        }

        Ok(Lock {
            resolutions,
            dirty: false,
            suggestions: HashMap::new(),
        })
    }

    pub fn has(&self, npm_package: &str, request: &str) -> bool {
        for resolution in self.resolutions.values() {
            if let Some(ref deps) = resolution.dependencies {
                for (key, value) in deps {
                    if key != npm_package {
                        continue;
                    }
                    if request == "*" || request == value {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn fix(&mut self, npm_package: &str, safe_versions: &semver::Request) {
        let (resolutions, _) = self.read_package(npm_package);
        if resolutions.is_empty() {
            return;
        }

        let mut needs_reset = false;
        for (key, resolution) in &resolutions {
            if let Some(ref version_str) = resolution.version {
                let version = semver::Version::parse(version_str)
                    .unwrap_or_else(|_| {
                        eprintln!("Error parsing version: {}", version_str);
                        std::process::exit(1);
                    });
                
                if safe_versions.matches(&version) {
                    continue;
                }

                let requested = request_from_key(key);
                let request = semver::Request::parse(&requested)
                    .unwrap_or_else(|_| {
                        eprintln!("Error parsing request: {}", requested);
                        std::process::exit(1);
                    });
                
                let npm_package_request = format!("{}@{}", npm_package, requested);
                let (overlaps, closest) = request.overlaps(safe_versions);
                
                match (overlaps, closest) {
                    (true, _) => {
                        needs_reset = true;
                    }
                    (false, None) => {
                        println!("No fix for {}", npm_package_request);
                    }
                    (false, Some(ref closest_version)) => {
                        if let Some(existing) = self.suggestions.get(&npm_package_request) {
                            if closest_version.at_least().matches(existing) {
                                self.suggestions.insert(npm_package_request, closest_version.clone());
                            }
                        } else {
                            self.suggestions.insert(npm_package_request, closest_version.clone());
                        }
                    }
                }
            }
        }

        if needs_reset {
            self.reset(npm_package);
        }
    }

    pub fn reset(&mut self, npm_package: &str) {
        let mut keys: Vec<String> = Vec::new();

        for key in self.resolutions.keys() {
            if key.starts_with(npm_package) && key.chars().nth(npm_package.len()) == Some('@') {
                keys.push(key.clone());
            }
        }

        if !keys.is_empty() {
            println!("Reset {}", npm_package);
        }

        for key in keys {
            self.dirty = true;
            self.resolutions.remove(&key);
        }
    }

    pub fn shrink(&mut self) {
        let mut npm_packages: HashMap<String, i32> = HashMap::new();
        
        for key in self.resolutions.keys() {
            if let Some(pos) = key[1..].find('@') {
                let npm_package = &key[..=pos];
                *npm_packages.entry(npm_package.to_string()).or_insert(0) += 1;
            }
        }

        for (npm_package, count) in npm_packages {
            if count > 1 {
                self.shrink_package(&npm_package);
            }
        }
    }

    fn read_package(&self, npm_package: &str) -> (HashMap<String, Resolution>, HashMap<String, Resolution>) {
        let mut resolutions = HashMap::new();
        let mut versions = HashMap::new();

        for (key, resolution) in &self.resolutions {
            if key.starts_with(npm_package) && key.chars().nth(npm_package.len()) == Some('@') {
                for sub in key.split(", ") {
                    resolutions.insert(sub.to_string(), resolution.clone());
                    if let Some(ref version) = resolution.version {
                        versions.insert(version.clone(), resolution.clone());
                    }
                }
            }
        }

        (resolutions, versions)
    }

    fn shrink_package(&mut self, npm_package: &str) {
        let (resolutions, versions) = self.read_package(npm_package);

        let mut updated_resolutions = resolutions.clone();
        for (key, value) in &resolutions {
            let requested = request_from_key(key);
            let requested = if requested.starts_with(&format!("{}@", npm_package)) {
                &requested[npm_package.len() + 1..]
            } else {
                &requested
            };

            let request = semver::Request::parse(requested)
                .unwrap_or_else(|_| {
                    eprintln!("Error parsing request: {}", requested);
                    std::process::exit(1);
                });
            
            let version = if let Some(ref v) = value.version {
                semver::Version::parse(v).unwrap_or_else(|_| {
                    eprintln!("Error parsing version: {}", v);
                    std::process::exit(1);
                })
            } else {
                continue;
            };

            let mut best_version = version.clone();
            for (present_source, _) in &versions {
                let present = semver::Version::parse(present_source)
                    .unwrap_or_else(|_| {
                        eprintln!("Error parsing version: {}", present_source);
                        std::process::exit(1);
                    });
                
                if version.at_least().matches(&present) && request.matches(&present) {
                    best_version = present;
                }
            }

            if value.version.as_ref().map(|s| s.as_str()) != Some(best_version.to_string().as_str()) {
                if let Some(new_resolution) = versions.get(&best_version.to_string()) {
                    updated_resolutions.insert(key.clone(), new_resolution.clone());
                }
            }
        }

        let mut next = HashMap::new();
        let mut dirty = false;
        
        for (version, resolution) in &versions {
            let mut keys: Vec<String> = Vec::new();
            for (key, res) in &updated_resolutions {
                if res.version.as_ref().map(|s| s.as_str()) == Some(version.as_str()) {
                    keys.push(key.clone());
                }
            }

            if keys.is_empty() {
                dirty = true;
                println!("Drop {} {}", npm_package, version);
                continue;
            }

            keys.sort();
            let key_csv = keys.join(", ");
            next.insert(key_csv.clone(), resolution.clone());
            
            if let Some(existing) = self.resolutions.get(&key_csv) {
                if existing.version != resolution.version {
                    dirty = true;
                    println!("Save {} {}", npm_package, version);
                }
            } else {
                dirty = true;
                println!("Save {} {}", npm_package, version);
            }
        }

        if dirty {
            self.reset(npm_package);
            for (key_csv, resolution) in next {
                self.resolutions.insert(key_csv, resolution);
            }
        }
    }

    fn print_suggestions(&self) {
        if self.suggestions.is_empty() {
            return;
        }

        let mut keys: Vec<String> = self.suggestions.keys().cloned().collect();
        keys.sort();

        println!("Suggested resolutions");
        for key in keys {
            if let Some(version) = self.suggestions.get(&key) {
                println!("    \"{}\": \"^{}\",", key, version);
            }
        }
    }

    pub fn save(&mut self, directory: &str) -> Result<bool, String> {
        self.print_suggestions();

        if !self.dirty {
            println!("yarn.lock stable");
            return Ok(false);
        }

        let yaml = serde_yaml::to_string(&self.resolutions)
            .map_err(|e| format!("cannot serialize yarn.lock: {}", e))?;

        println!("yarn.lock dirty, needs `yarn install`");
        let path = Path::new(directory).join("yarn.lock");
        fs::write(&path, yaml)
            .map_err(|e| format!("cannot write yarn.lock: {}", e))?;

        Ok(true)
    }
}

fn request_from_key(key: &str) -> String {
    if let Some(pos) = key.rfind(':') {
        let after_colon = &key[pos + 1..];
        if let Some(loc) = after_colon.rfind("%3A") {
            after_colon[loc + 3..].to_string()
        } else {
            after_colon.to_string()
        }
    } else {
        key.to_string()
    }
}

#[derive(Debug, Deserialize)]
struct Audit {
    #[serde(default)]
    advisories: HashMap<String, Advisory>,
}

#[derive(Debug, Deserialize)]
struct Yarn4Advisory {
    #[serde(rename = "value")]
    module_name: String,
    children: Yarn4AdvisoryChildren,
}

#[derive(Debug, Deserialize)]
struct Yarn4AdvisoryChildren {
    #[serde(rename = "ID")]
    id: serde_json::Value,
    #[serde(rename = "Vulnerable Versions")]
    vulnerable_versions: String,
}

impl Yarn4Advisory {
    fn to_advisory(&self) -> Advisory {
        let request = semver::Request::parse(&self.children.vulnerable_versions)
            .unwrap_or_else(|_| {
                eprintln!("Error parsing vulnerable versions: {}", self.children.vulnerable_versions);
                std::process::exit(1);
            });
        
        Advisory {
            module_name: self.module_name.clone(),
            patched_versions: request.patches().to_string(),
        }
    }
}

pub fn parse_audit(output: &[u8], version: &semver::Version) -> Result<Vec<Advisory>, String> {
    match version.major {
        2 | 3 => parse_audit_yarn2(output),
        4 => parse_audit_yarn4(output),
        _ => Err(format!("unsupported yarn version: {}", version.major)),
    }
}

fn parse_audit_yarn2(output: &[u8]) -> Result<Vec<Advisory>, String> {
    let audit: Audit = serde_json::from_slice(output)
        .map_err(|e| format!("cannot deserialize audit json: {}", e))?;

    Ok(audit.advisories.into_values().collect())
}

fn parse_audit_yarn4(output: &[u8]) -> Result<Vec<Advisory>, String> {
    let mut advisories = Vec::new();
    let reader = BufReader::new(Cursor::new(output));
    
    for line in reader.lines() {
        let line = line.map_err(|e| format!("cannot read line: {}", e))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        match serde_json::from_str::<Yarn4Advisory>(line) {
            Ok(issue) => {
                if let serde_json::Value::String(id) = &issue.children.id {
                    if id.contains(" (deprecation)") {
                        continue;
                    }
                }
                advisories.push(issue.to_advisory());
            }
            Err(_) => {
                // Try to continue parsing other lines
                continue;
            }
        }
    }

    Ok(advisories)
}
