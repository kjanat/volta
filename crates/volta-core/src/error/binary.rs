//! Error types for binary and executable operations.
//!
//! This module contains errors related to:
//! - Binary resolution and lookup
//! - Executable not found scenarios
//! - Binary execution failures

use std::fmt;
use std::path::PathBuf;

use super::ExitCode;

const REPORT_BUG_CTA: &str =
    "Please rerun the command that triggered this error with the environment
variable `VOLTA_LOGLEVEL` set to `debug` and open an issue at
https://github.com/volta-cli/volta/issues with the details!";

const PERMISSIONS_CTA: &str = "Please ensure you have correct permissions to the Volta directory.";

/// Errors related to binary and executable operations.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum BinaryError {
    /// Thrown when package tries to install a binary that is already installed.
    AlreadyInstalled {
        bin_name: String,
        existing_package: String,
        new_package: String,
    },

    /// Thrown when executing an external binary fails
    ExecError,

    /// Thrown when a binary could not be found in the local inventory
    NotFound { name: String },

    /// Thrown when executing a project-local binary fails
    ProjectLocalExecError { command: String },

    /// Thrown when a project-local binary could not be found
    ProjectLocalNotFound { command: String },

    /// Thrown when unable to parse a bin config file
    ParseConfigError,

    /// Thrown when there was an error reading the config for a binary
    ReadConfigError { file: PathBuf },

    /// Thrown when there was an error reading the user bin directory
    ReadConfigDirError { dir: PathBuf },
}

impl fmt::Display for BinaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyInstalled {
                bin_name,
                existing_package,
                new_package,
            } => write!(
                f,
                "Executable '{bin_name}' is already installed by {existing_package}

Please remove {existing_package} before installing {new_package}"
            ),
            Self::ExecError => write!(
                f,
                "Could not execute command.

See `volta help install` and `volta help pin` for info about making tools available."
            ),
            Self::NotFound { name } => write!(
                f,
                r#"Could not find executable "{name}"

Use `volta install` to add a package to your toolchain (see `volta help install` for more info)."#
            ),
            Self::ProjectLocalExecError { command } => write!(
                f,
                "Could not execute `{command}`

Please ensure you have correct permissions to access the file."
            ),
            Self::ProjectLocalNotFound { command } => write!(
                f,
                "Could not locate executable `{command}` in your project.

Please ensure that all project dependencies are installed with `npm install` or `yarn install`"
            ),
            Self::ParseConfigError => write!(
                f,
                "Could not parse executable configuration file.

{REPORT_BUG_CTA}"
            ),
            Self::ReadConfigError { file } => write!(
                f,
                "Could not read executable configuration
from {}

{PERMISSIONS_CTA}",
                file.display()
            ),
            Self::ReadConfigDirError { dir } => write!(
                f,
                "Could not read executable metadata directory
at {}

{PERMISSIONS_CTA}",
                dir.display()
            ),
        }
    }
}

impl BinaryError {
    /// Returns the appropriate exit code for this error.
    #[must_use]
    pub const fn exit_code(&self) -> ExitCode {
        match self {
            Self::ExecError | Self::ProjectLocalExecError { .. } => ExitCode::ExecutionFailure,
            Self::NotFound { .. } => ExitCode::ExecutableNotFound,
            Self::ParseConfigError => ExitCode::UnknownError,
            Self::AlreadyInstalled { .. }
            | Self::ProjectLocalNotFound { .. }
            | Self::ReadConfigError { .. }
            | Self::ReadConfigDirError { .. } => ExitCode::FileSystemError,
        }
    }
}
