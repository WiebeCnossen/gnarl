pub enum Error {
    Str(&'static str),
    String(String),
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::Str(s)
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::String(s)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::String(error.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::String(error.to_string())
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(error: serde_yaml::Error) -> Self {
        Error::String(error.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::String(error.to_string())
    }
}

impl From<nodejs_semver::SemverError> for Error {
    fn from(error: nodejs_semver::SemverError) -> Self {
        Error::String(error.to_string())
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Str(s) => write!(f, "{}", s),
            Error::String(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for Error {}
