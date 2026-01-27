//! Error types for hook operations.
//!
//! This module contains errors related to:
//! - Hook configuration parsing
//! - Hook execution
//! - Hook template processing

use std::fmt;
use std::path::PathBuf;

use super::ExitCode;

/// Errors related to hook operations.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum HookError {
    /// Unable to execute a hook command.
    ExecutionFailed { command: String },

    /// Hook command returned a non-zero exit code.
    CommandFailed { command: String },

    /// Hook configuration includes multiple hook types.
    MultipleFieldsSpecified,

    /// Hook configuration includes no hook types.
    NoFieldsSpecified,

    /// Unable to determine path to hook command.
    PathResolutionFailed { command: String },

    /// Invalid hook command (e.g., empty command).
    InvalidCommand { command: String },

    /// Unable to read output from hook command.
    InvalidOutput { command: String },

    /// Unable to parse hooks configuration file.
    ParseFailed { file: PathBuf },

    /// Publish hook configuration includes both url and bin fields.
    PublishBothUrlAndBin,

    /// Publish hook configuration includes neither url nor bin fields.
    PublishNeitherUrlNorBin,

    /// Unrecognized index registry format.
    InvalidRegistryFormat { format: String },
}

impl fmt::Display for HookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExecutionFailed { command } => write!(
                f,
                "Could not execute hook command: '{command}'

Please ensure that the correct command is specified."
            ),
            Self::CommandFailed { command } => write!(
                f,
                "Hook command '{command}' indicated a failure.

Please verify the requested tool and version."
            ),
            Self::MultipleFieldsSpecified => write!(
                f,
                "Hook configuration includes multiple hook types.

Please include only one of 'bin', 'prefix', or 'template'"
            ),
            Self::NoFieldsSpecified => write!(
                f,
                "Hook configuration includes no hook types.

Please include one of 'bin', 'prefix', or 'template'"
            ),
            Self::PathResolutionFailed { command } => write!(
                f,
                "Could not determine path to hook command: '{command}'

Please ensure that the correct command is specified."
            ),
            Self::InvalidCommand { command } => write!(
                f,
                "Invalid hook command: '{command}'

Please ensure that the correct command is specified."
            ),
            Self::InvalidOutput { command } => write!(
                f,
                "Could not read output from hook command: '{command}'

Please ensure that the command output is valid UTF-8 text."
            ),
            Self::ParseFailed { file } => write!(
                f,
                "Could not parse hooks configuration file.
from {}

Please ensure the file is correctly formatted.",
                file.display()
            ),
            Self::PublishBothUrlAndBin => write!(
                f,
                "Publish hook configuration includes both hook types.

Please include only one of 'bin' or 'url'"
            ),
            Self::PublishNeitherUrlNorBin => write!(
                f,
                "Publish hook configuration includes no hook types.

Please include one of 'bin' or 'url'"
            ),
            Self::InvalidRegistryFormat { format } => write!(
                f,
                "Unrecognized index registry format: '{format}'

Please specify either 'npm' or 'github' for the format."
            ),
        }
    }
}

impl HookError {
    #[must_use]
    pub const fn exit_code(&self) -> ExitCode {
        match self {
            // ConfigurationError - configuration issues
            Self::CommandFailed { .. }
            | Self::MultipleFieldsSpecified
            | Self::NoFieldsSpecified
            | Self::PathResolutionFailed { .. }
            | Self::InvalidRegistryFormat { .. }
            | Self::ParseFailed { .. }
            | Self::PublishBothUrlAndBin
            | Self::PublishNeitherUrlNorBin => ExitCode::ConfigurationError,

            // ExecutableNotFound - invalid command
            Self::InvalidCommand { .. } => ExitCode::ExecutableNotFound,

            // ExecutionFailure - execution issues
            Self::ExecutionFailed { .. } | Self::InvalidOutput { .. } => ExitCode::ExecutionFailure,
        }
    }
}
