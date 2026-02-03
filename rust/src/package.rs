use nodejs_semver::{Range, Version};

use crate::parse;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Package {
    name: String,
    source: String,
    version: Option<Version>,
}

impl Package {
    pub fn new(name: String, source: String, version: Option<Version>) -> Self {
        Self {
            name,
            source,
            version,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn version(&self) -> Option<&Version> {
        self.version.as_ref()
    }
}

impl TryFrom<String> for Package {
    type Error = crate::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let (name, tail) = parse::split_name(&s)?;
        let (source, version) = parse::parse_qualified_version(tail)?;
        Ok(Self::new(name.to_owned(), source.to_owned(), version))
    }
}

pub struct Resolution {
    package: Package,
    requests: Vec<Range>,
    original_requests: Vec<String>,
    dependencies: Vec<Dependency>,
}

impl Resolution {
    pub fn new(
        package: Package,
        requests: Vec<Range>,
        original_requests: Vec<String>,
        dependencies: Vec<Dependency>,
    ) -> Self {
        Self {
            package,
            requests,
            original_requests,
            dependencies,
        }
    }

    pub fn package(&self) -> &Package {
        &self.package
    }

    pub fn requests(&self) -> &[Range] {
        &self.requests
    }

    pub fn original(&self, request: &Range) -> Result<&str, crate::Error> {
        self.original_requests
            .iter()
            .find(|original_request| {
                if let Ok(parsed) = parse::parse_range(original_request)
                    && parsed.eq(request)
                {
                    true
                } else {
                    false
                }
            })
            .map(|original_request| original_request.as_str())
            .ok_or_else(|| format!("Original request for {} not found", request).into())
    }

    pub fn dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }
}

pub struct Dependency {
    name: String,
    source: String,
    request: Range,
}

impl Dependency {
    pub fn new(name: String, source: String, request: Range) -> Self {
        Self {
            name,
            source,
            request,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn request(&self) -> &Range {
        &self.request
    }

    pub fn parse(request: &str) -> Result<Self, crate::Error> {
        let (source, tail) = parse::split_qualified(request)?;
        let (name, range) = parse::split_name(tail)?;
        let range = parse::parse_range(range)?;
        Ok(Self::new(name.to_owned(), source.to_owned(), range))
    }

    pub fn from_qualified_range(name: &str, qualified_range: &str) -> Result<Self, crate::Error> {
        if let Ok(dependency) = Self::parse(qualified_range) {
            return Ok(dependency);
        }

        let (source, range) = parse::parse_qualified_range(qualified_range)?;
        Ok(Self::new(name.to_owned(), source.to_owned(), range))
    }
}
