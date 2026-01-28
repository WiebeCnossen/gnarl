use std::fmt;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    pub pre: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constraint {
    Exact,
    MatchMinor,
    MatchMajor,
    AtLeast,
    AtMost,
    Less,
    Greater,
    Any,
}

#[derive(Debug, Clone)]
pub struct RequestFactor {
    pub version: Version,
    pub constraint: Constraint,
}

pub type RequestTerm = Vec<RequestFactor>;

#[derive(Debug, Clone)]
pub struct Request {
    pub terms: Vec<RequestTerm>,
}

impl Version {
    pub fn parse(version: &str) -> Result<Self, String> {
        let mut hash = String::new();
        let version = if let Some(loc) = version.find('#') {
            hash = version[loc..].to_string();
            &version[..loc]
        } else {
            version
        };

        let parts: Vec<&str> = version.splitn(3, '.').collect();
        
        let major = parts[0].parse::<i32>()
            .map_err(|e| format!("invalid major {}: {}", parts[0], e))?;

        let (minor, patch, pre) = if parts.len() > 1 {
            let minor = parts[1].parse::<i32>()
                .map_err(|e| format!("invalid minor {}: {}", parts[1], e))?;
            
            if parts.len() > 2 {
                let mut i = 0;
                let patch_str = parts[2];
                let chars: Vec<char> = patch_str.chars().collect();
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                
                let patch = patch_str[..i].parse::<i32>()
                    .map_err(|e| format!("invalid patch {}: {}", parts[1], e))?;
                let pre = patch_str[i..].to_string();
                
                (minor, patch, pre)
            } else {
                (minor, 0, String::new())
            }
        } else {
            (0, 0, String::new())
        };

        Ok(Version {
            major,
            minor,
            patch,
            pre: pre + &hash,
        })
    }

    pub fn at_least(&self) -> Request {
        Request {
            terms: vec![vec![RequestFactor {
                version: self.clone(),
                constraint: Constraint::AtLeast,
            }]],
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}{}", self.major, self.minor, self.patch, self.pre)
    }
}

impl Request {
    pub fn parse(request: &str) -> Result<Self, String> {
        let mut terms = Vec::new();
        
        for part in request.split("||") {
            let part = part.trim();
            let term = parse_request_term(part)?;
            terms.push(term);
        }

        Ok(Request { terms })
    }

    pub fn matches(&self, version: &Version) -> bool {
        'terms: for term in &self.terms {
            for factor in term {
                if !factor.matches(version) {
                    continue 'terms;
                }
            }
            return true;
        }
        false
    }

    pub fn patches(&self) -> Request {
        let mut patches = Vec::new();

        for term in &self.terms {
            for factor in term {
                if factor.constraint == Constraint::Less {
                    patches.push(vec![RequestFactor {
                        constraint: Constraint::MatchMajor,
                        version: factor.version.clone(),
                    }]);
                }
            }
        }

        if patches.is_empty() {
            patches.push(vec![RequestFactor {
                constraint: Constraint::Less,
                version: Version {
                    major: 0,
                    minor: 0,
                    patch: 0,
                    pre: String::new(),
                },
            }]);
        }

        Request { terms: patches }
    }

    pub fn is_exact(&self) -> bool {
        self.terms.len() == 1 
            && self.terms[0].len() == 1 
            && self.terms[0][0].constraint == Constraint::Exact
    }

    pub fn overlaps(&self, other: &Request) -> (bool, Option<Version>) {
        let from_versions = other.from_versions();
        
        for version in &from_versions {
            if self.matches(version) {
                return (true, None);
            }
        }

        if from_versions.is_empty() {
            return (false, None);
        }

        (false, Some(from_versions[0].clone()))
    }

    fn from_versions(&self) -> Vec<Version> {
        let mut versions = Vec::new();

        for term in &self.terms {
            for factor in term {
                if factor.constraint == Constraint::Greater {
                    let successor = Version {
                        major: factor.version.major,
                        minor: factor.version.minor,
                        patch: factor.version.patch + 1,
                        pre: String::new(),
                    };
                    versions.push(successor);
                } else if factor.constraint != Constraint::Less && factor.constraint != Constraint::Any {
                    versions.push(factor.version.clone());
                }
            }
        }

        versions
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let parts: Vec<String> = self.terms.iter()
            .map(|term| {
                term.iter()
                    .map(|factor| factor.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect();
        write!(f, "{}", parts.join(" || "))
    }
}

impl RequestFactor {
    pub fn matches(&self, version: &Version) -> bool {
        let match_pre = |strict: bool| -> bool {
            self.version.pre == version.pre || (!strict && version.pre.is_empty())
        };

        match self.constraint {
            Constraint::Exact => {
                self.version.major == version.major
                    && self.version.minor == version.minor
                    && self.version.patch == version.patch
                    && match_pre(true)
            }
            Constraint::MatchMinor => {
                self.version.major == version.major
                    && self.version.minor == version.minor
                    && self.version.patch <= version.patch
                    && match_pre(false)
            }
            Constraint::MatchMajor => {
                self.version.major == version.major
                    && (self.version.minor < version.minor
                        || (self.version.minor == version.minor && self.version.patch <= version.patch))
                    && match_pre(false)
            }
            Constraint::AtLeast => {
                (self.version.major < version.major
                    || (self.version.major == version.major
                        && (self.version.minor < version.minor
                            || (self.version.minor == version.minor && self.version.patch <= version.patch))))
                    && match_pre(false)
            }
            Constraint::AtMost => {
                (self.version.major > version.major
                    || (self.version.major == version.major
                        && (self.version.minor > version.minor
                            || (self.version.minor == version.minor && self.version.patch >= version.patch))))
                    && match_pre(false)
            }
            Constraint::Greater => {
                (self.version.major < version.major
                    || (self.version.major == version.major
                        && (self.version.minor < version.minor
                            || (self.version.minor == version.minor && self.version.patch < version.patch))))
                    && match_pre(false)
            }
            Constraint::Less => {
                (self.version.major > version.major
                    || (self.version.major == version.major
                        && (self.version.minor > version.minor
                            || (self.version.minor == version.minor && self.version.patch > version.patch))))
                    && match_pre(false)
            }
            Constraint::Any => true,
        }
    }
}

impl fmt::Display for RequestFactor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let operator = match self.constraint {
            Constraint::Exact => "=",
            Constraint::MatchMinor => "~",
            Constraint::MatchMajor => "^",
            Constraint::AtLeast => ">=",
            Constraint::AtMost => "<=",
            Constraint::Less => "<",
            Constraint::Greater => ">",
            Constraint::Any => "*",
        };
        write!(f, "{}{}", operator, self.version)
    }
}

fn parse_request_term(term: &str) -> Result<RequestTerm, String> {
    let re1 = Regex::new(r"([<>=~^])\s+").unwrap();
    let mut source = re1.replace_all(term, "$1").to_string();
    let re2 = Regex::new(r"\s+").unwrap();
    source = re2.replace_all(&source, " ").to_string();

    let mut factors: Vec<RequestFactor> = Vec::new();
    let mut saw_hyphen = false;
    let parts: Vec<&str> = source.split(' ').collect();
    
    for i in 0..parts.len() {
        let part = parts[i];
        let mut constraint = if part == "-" && i > 0 && i + 1 < parts.len() {
            if let Some(last_factor) = factors.last_mut() {
                if last_factor.constraint == Constraint::Exact {
                    last_factor.constraint = Constraint::AtLeast;
                    saw_hyphen = true;
                    continue;
                }
            }
            return Err(format!("cannot apply range to {}", part));
        } else if part == "*" || part == "latest" {
            factors.push(RequestFactor {
                constraint: Constraint::Any,
                version: Version {
                    major: 0,
                    minor: 0,
                    patch: 0,
                    pre: String::new(),
                },
            });
            continue;
        } else if part.starts_with('^') {
            (Constraint::MatchMajor, &part[1..])
        } else if part.starts_with('~') {
            (Constraint::MatchMinor, &part[1..])
        } else if part.starts_with(">=") {
            (Constraint::AtLeast, &part[2..])
        } else if part.starts_with("<=") {
            (Constraint::AtMost, &part[2..])
        } else if part.starts_with('<') {
            (Constraint::Less, &part[1..])
        } else if part.starts_with('>') {
            (Constraint::Greater, &part[1..])
        } else if part.starts_with('=') {
            (Constraint::Exact, &part[1..])
        } else if part.parse::<i64>().is_ok() {
            (Constraint::MatchMajor, part)
        } else {
            (Constraint::Exact, part)
        };

        if saw_hyphen && constraint.0 != Constraint::Exact {
            return Err(format!("cannot apply range to {}", part));
        }
        
        if saw_hyphen {
            constraint.0 = Constraint::AtMost;
            saw_hyphen = false;
        }

        let mut part_str = constraint.1.to_string();
        
        let minor_match = Regex::new(r"^(\d+)\.[x*]").unwrap();
        if let Some(caps) = minor_match.captures(&part_str) {
            constraint.0 = Constraint::MatchMajor;
            part_str = format!("{}.0.0", &caps[1]);
        }

        let patch_match = Regex::new(r"^(\d+)\.(\d+)\.[x*]").unwrap();
        if let Some(caps) = patch_match.captures(&part_str) {
            constraint.0 = Constraint::MatchMinor;
            part_str = format!("{}.{}.0", &caps[1], &caps[2]);
        }

        let version = Version::parse(&part_str)?;

        let mut final_constraint = constraint.0;
        if version.major == 0 && final_constraint == Constraint::MatchMajor {
            final_constraint = Constraint::MatchMinor;
        }
        if version.major == 0 && version.minor == 0 && final_constraint == Constraint::MatchMinor {
            final_constraint = Constraint::Exact;
        }

        factors.push(RequestFactor {
            constraint: final_constraint,
            version,
        });
    }

    Ok(factors)
}
