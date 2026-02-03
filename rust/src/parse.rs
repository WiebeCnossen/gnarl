use nodejs_semver::{Range, Version};

use crate::Error;

pub fn split_first(reference: &str, separator: impl AsRef<str>) -> &str {
    reference.split(separator.as_ref()).next().unwrap()
}

pub fn split_last<'a>(
    label: &str,
    reference: &'a str,
    separator: impl AsRef<str>,
) -> Result<(&'a str, &'a str), crate::Error> {
    let idx = reference.rfind(separator.as_ref());
    if let Some(i) = idx {
        let (left, right) = reference.split_at(i);
        Ok((left, &right[1..]))
    } else {
        Err(format!("Invalid {}: {}", label, reference).into())
    }
}

pub fn split_name(reference: &str) -> Result<(&str, &str), crate::Error> {
    let (name, tail) = split_last("name", reference, "@")?;
    match name.chars().filter(|&c| c == '@').count() {
        0 => Ok((name, tail)),
        1 if name.starts_with("@") => Ok((name, tail)),
        _ => {
            let name = split_first(reference, "@");
            let tail = &reference[name.len() + 1..];
            Ok((name, tail))
        }
    }
}

pub fn split_qualified(qualified: &str) -> Result<(&str, &str), crate::Error> {
    split_last("qualified", qualified, ":")
        .or_else(|_| split_last("qualified", qualified, "%3A"))
        .or_else(|_| split_last("qualified", qualified, "#"))
}

pub fn parse_qualified_version(
    qualified_version: &str,
) -> Result<(&str, Option<Version>), crate::Error> {
    let (source, version) = split_qualified(qualified_version)?;
    if source == "npm" {
        Ok((source, parse_version(version).ok()))
    } else {
        Ok((source, None))
    }
}

pub fn parse_qualified_range(qualified_range: &str) -> Result<(&str, Range), crate::Error> {
    let (source, range) = split_qualified(qualified_range)?;
    Ok((source, parse_range(range)?))
}

pub fn parse_range(range: &str) -> Result<Range, Error> {
    (match range {
        "latest" => Range::parse("*"),
        _ => Range::parse(range),
    })
    .map_err(|e| format!("Invalid range {}: {}", range, e).into())
}

pub fn parse_version(version: &str) -> Result<Version, Error> {
    Version::parse(version).map_err(|e| format!("Invalid version: {}: {}", version, e).into())
}
