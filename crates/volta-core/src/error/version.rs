//! Error types for version resolution.
//!
//! This module contains errors related to:
//! - Semver parsing and validation
//! - Version requirement matching
//! - Version tag resolution (latest, lts)

use std::fmt;

use super::ExitCode;

/// Errors related to version resolution.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum VersionError {
    /// No matching Node version found.
    NodeNotFound { matching: String },

    /// No matching npm version found.
    NpmNotFound { matching: String },

    /// No matching pnpm version found.
    PnpmNotFound { matching: String },

    /// No matching Yarn version found.
    YarnNotFound { matching: String },

    /// Failed to parse a version string.
    ParseFailed { version: String },

    /// Could not detect bundled npm version.
    NoBundledNpm { command: String },

    /// Yarn version 2 is not supported.
    Yarn2NotSupported,
}

impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodeNotFound { matching } => write!(
                f,
                r#"Could not find Node version matching "{matching}" in the version registry.

Please verify that the version is correct."#
            ),
            Self::NpmNotFound { matching } => write!(
                f,
                r#"Could not find npm version matching "{matching}" in the version registry.

Please verify that the version is correct."#
            ),
            Self::PnpmNotFound { matching } => write!(
                f,
                r#"Could not find pnpm version matching "{matching}" in the version registry.

Please verify that the version is correct."#
            ),
            Self::YarnNotFound { matching } => write!(
                f,
                r#"Could not find Yarn version matching "{matching}" in the version registry.

Please verify that the version is correct."#
            ),
            Self::ParseFailed { version } => write!(
                f,
                r#"Could not parse version "{version}"

Please verify the intended version."#
            ),
            Self::NoBundledNpm { command } => write!(
                f,
                "Could not detect bundled npm version.

Please ensure you have a Node version selected with `volta {command} node` (see `volta help {command}` for more info)."
            ),
            Self::Yarn2NotSupported => write!(
                f,
                "Yarn version 2 is not recommended for use, and not supported by Volta.

Please use version 3 or greater instead."
            ),
        }
    }
}

impl VersionError {
    /// Returns the exit code for this error.
    #[must_use]
    pub const fn exit_code(&self) -> ExitCode {
        match self {
            Self::NodeNotFound { .. }
            | Self::NpmNotFound { .. }
            | Self::PnpmNotFound { .. }
            | Self::YarnNotFound { .. }
            | Self::ParseFailed { .. }
            | Self::Yarn2NotSupported => ExitCode::NoVersionMatch,
            Self::NoBundledNpm { .. } => ExitCode::ConfigurationError,
        }
    }
}
