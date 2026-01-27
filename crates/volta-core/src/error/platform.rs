//! Error types for platform and default configuration.
//!
//! This module contains errors related to:
//! - Default toolchain configuration
//! - Platform detection
//! - Project platform settings

use std::fmt;

use super::ExitCode;

/// Errors related to platform and default configuration.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum PlatformError {
    /// Thrown when the platform (Node version) could not be determined.
    NoPlatform,

    /// Thrown when a user tries to install a Yarn, npm, or pnpm version before installing a
    /// default Node version.
    NoDefaultNode { tool: String },

    /// Thrown when default Yarn is not set.
    NoDefaultYarn,

    /// Thrown when default pnpm is not set.
    NoDefaultPnpm,

    /// Thrown when a user tries to pin npm, pnpm, or Yarn before pinning a Node version.
    NoPinnedNode { tool: String },

    /// Thrown when parsing the project manifest and there is a `"volta"` key without Node.
    NoProjectNode,

    /// Thrown when Yarn is not set in a project.
    NoProjectYarn,

    /// Thrown when pnpm is not set in a project.
    NoProjectPnpm,

    /// Thrown when unable to parse the platform.json file.
    ParsePlatform,

    /// Thrown when the user tries to pin Node or Yarn versions outside of a package.
    NotInPackage,
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoPlatform => write!(
                f,
                "Node is not available.

To run any Node command, first set a default version using `volta install node`"
            ),
            Self::NoDefaultNode { tool } => write!(
                f,
                "Cannot install {tool} because the default Node version is not set.

Use `volta install node` to select a default Node first, then install a {tool} version."
            ),
            Self::NoDefaultYarn => write!(
                f,
                "Yarn is not available.

Use `volta install yarn` to select a default version (see `volta help install` for more info)."
            ),
            Self::NoDefaultPnpm => write!(
                f,
                "pnpm is not available.

Use `volta install pnpm` to select a default version (see `volta help install` for more info)."
            ),
            Self::NoPinnedNode { tool } => write!(
                f,
                "Cannot pin {tool} because the Node version is not pinned in this project.

Use `volta pin node` to pin Node first, then pin a {tool} version."
            ),
            Self::NoProjectNode => write!(
                f,
                "No Node version found in this project.

Use `volta pin node` to select a version (see `volta help pin` for more info)."
            ),
            Self::NoProjectYarn => write!(
                f,
                "No Yarn version found in this project.

Use `volta pin yarn` to select a version (see `volta help pin` for more info)."
            ),
            Self::NoProjectPnpm => write!(
                f,
                "No pnpm version found in this project.

Use `volta pin pnpm` to select a version (see `volta help pin` for more info)."
            ),
            Self::ParsePlatform => {
                write!(
                    f,
                    "Could not parse platform settings file.

Please rerun the command that triggered this error with the environment
variable `VOLTA_LOGLEVEL` set to `debug` and open an issue at
https://github.com/volta-cli/volta/issues with the details!"
                )
            }
            Self::NotInPackage => write!(
                f,
                "Not in a node package.

Use `volta install` to select a default version of a tool."
            ),
        }
    }
}

impl PlatformError {
    /// Returns the appropriate exit code for this error.
    #[must_use]
    pub const fn exit_code(&self) -> ExitCode {
        ExitCode::ConfigurationError
    }
}
