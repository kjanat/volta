use std::fmt;
use std::str::FromStr;

use crate::error::{Context, ErrorKind, Fallible, VersionError, VoltaError};
use nodejs_semver::{Range, Version};

mod serial;

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[allow(clippy::module_name_repetitions)]
pub enum VersionSpec {
    /// No version specified (default)
    #[default]
    None,

    /// `SemVer` Range
    Semver(Range),

    /// Exact Version
    Exact(Version),

    /// Arbitrary Version Tag
    Tag(Tag),
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum Tag {
    /// The 'latest' tag, a special case that exists for all packages
    Latest,

    /// The 'lts' tag, a special case for Node
    Lts,

    /// An arbitrary tag version
    Custom(String),
}

impl fmt::Display for VersionSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "<default>"),
            Self::Semver(req) => req.fmt(f),
            Self::Exact(version) => version.fmt(f),
            Self::Tag(tag) => tag.fmt(f),
        }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Latest => write!(f, "latest"),
            Self::Lts => write!(f, "lts"),
            Self::Custom(s) => s.fmt(f),
        }
    }
}

impl FromStr for VersionSpec {
    type Err = VoltaError;

    fn from_str(s: &str) -> Fallible<Self> {
        parse(s).map_or_else(
            |_| {
                parse_requirements(s)
                    .map_or_else(|_| s.parse().map(Self::Tag), |req| Ok(Self::Semver(req)))
            },
            |version| Ok(Self::Exact(version)),
        )
    }
}

impl FromStr for Tag {
    type Err = VoltaError;

    fn from_str(s: &str) -> Fallible<Self> {
        if s == "latest" {
            Ok(Self::Latest)
        } else if s == "lts" {
            Ok(Self::Lts)
        } else {
            Ok(Self::Custom(s.into()))
        }
    }
}

/// # Errors
///
/// Returns an error if the string cannot be parsed as a semver range.
pub fn parse_requirements(s: impl AsRef<str>) -> Fallible<Range> {
    let s = s.as_ref();
    serial::parse_requirements(s)
        .with_context(|| ErrorKind::Version(VersionError::ParseFailed { version: s.into() }))
}

/// # Errors
///
/// Returns an error if the string cannot be parsed as a semver version.
pub fn parse(s: impl AsRef<str>) -> Fallible<Version> {
    let s = s.as_ref();
    s.parse()
        .with_context(|| ErrorKind::Version(VersionError::ParseFailed { version: s.into() }))
}

// remove the leading 'v' from the version string, if present
fn trim_version(s: &str) -> &str {
    let s = s.trim();
    s.strip_prefix('v').unwrap_or(s)
}

// custom serialization and de-serialization for Version
// because Version doesn't work with serde out of the box
#[allow(clippy::module_name_repetitions)]
pub mod version_serde {
    use nodejs_semver::Version;
    use serde::de::{Error, Visitor};
    use serde::{self, Deserializer, Serializer};
    use std::fmt;

    struct VersionVisitor;

    impl Visitor<'_> for VersionVisitor {
        type Value = Version;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("string")
        }

        // parse the version from the string
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Version::parse(super::trim_version(value)).map_err(Error::custom)
        }
    }

    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn serialize<S>(version: &Version, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&version.to_string())
    }

    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(VersionVisitor)
    }
}

// custom serialization and de-serialization for Option<Version>
// because Version doesn't work with serde out of the box
pub mod option_version_serde {
    use nodejs_semver::Version;
    use serde::de::Error;
    use serde::{self, Deserialize, Deserializer, Serializer};

    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn serialize<S>(version: &Option<Version>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match version {
            Some(v) => s.serialize_str(&v.to_string()),
            None => s.serialize_none(),
        }
    }

    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Version>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        if let Some(v) = s {
            return Ok(Some(
                Version::parse(super::trim_version(&v)).map_err(Error::custom)?,
            ));
        }
        Ok(None)
    }
}

// custom deserialization for HashMap<String, Version>
// because Version doesn't work with serde out of the box
pub mod hashmap_version_serde {
    use super::version_serde;
    use nodejs_semver::Version;
    use serde::{self, Deserialize, Deserializer};
    use std::collections::HashMap;

    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "version_serde::deserialize")] Version);

    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<String, Version>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let m = HashMap::<String, Wrapper>::deserialize(deserializer)?;
        Ok(m.into_iter().map(|(k, Wrapper(v))| (k, v)).collect())
    }
}
