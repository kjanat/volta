//! Error types for command invocation.
//!
//! This module contains errors related to:
//! - Command-line argument validation
//! - Subcommand execution
//! - External command invocation

use std::fmt;

use super::ExitCode;
use crate::style::{text_width, tool_version};
use textwrap::fill;

/// Errors related to command invocation.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum CommandError {
    /// Failed to execute command with `VOLTA_BYPASS` set.
    Bypass { command: String },

    /// User invoked a deprecated command.
    Deprecated { command: String, advice: String },

    /// Invalid invocation like `volta install node 12` instead of `volta install node@12`.
    InvalidToolVersion {
        action: String,
        name: String,
        version: String,
    },

    /// Invalid invocation like `volta install 12` instead of `volta install node@12`.
    InvalidBareVersion { action: String, version: String },

    /// pnpm not specified at command-line when required.
    NoPnpmSpecified,

    /// Yarn not specified at command-line when required.
    NoYarnSpecified,

    /// npx requires npm >= 5.2.0.
    NpxUnavailable { version: String },
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bypass { command } => write!(
                f,
                "Could not execute command '{command}'

VOLTA_BYPASS is enabled, please ensure that the command exists on your system or unset VOLTA_BYPASS",
            ),
            Self::Deprecated { command, advice } => {
                write!(f, "The subcommand `{command}` is deprecated.\n{advice}")
            }
            Self::InvalidToolVersion {
                action,
                name,
                version,
            } => {
                let error = format!("`volta {action} {name} {version}` is not supported.");

                let call_to_action = format!(
"To {action} '{name}' version '{version}', please run `volta {action} {formatted}`. \
To {action} the packages '{name}' and '{version}', please {action} them in separate commands, or with explicit versions.",
                    action=action,
                    name=name,
                    version=version,
                    formatted=tool_version(name, version)
                );

                let wrapped_cta = match text_width() {
                    Some(width) => fill(&call_to_action, width),
                    None => call_to_action,
                };

                write!(f, "{error}\n\n{wrapped_cta}")
            }
            Self::InvalidBareVersion { action, version } => {
                let error = format!("`volta {action} {version}` is not supported.");

                let call_to_action = format!(
"To {action} node version '{version}', please run `volta {action} {formatted}`. \
To {action} the package '{version}', please use an explicit version such as '{version}@latest'.",
                    action=action,
                    version=version,
                    formatted=tool_version("node", version)
                );

                let wrapped_cta = match text_width() {
                    Some(width) => fill(&call_to_action, width),
                    None => call_to_action,
                };

                write!(f, "{error}\n\n{wrapped_cta}")
            }
            Self::NoPnpmSpecified => write!(
                f,
                "No pnpm version specified.

Use `volta run --pnpm` to select a version (see `volta help run` for more info)."
            ),
            Self::NoYarnSpecified => write!(
                f,
                "No Yarn version specified.

Use `volta run --yarn` to select a version (see `volta help run` for more info)."
            ),
            Self::NpxUnavailable { version } => write!(
                f,
                "'npx' is only available with npm >= 5.2.0

This project is configured to use version {version} of npm."
            ),
        }
    }
}

impl CommandError {
    /// Returns the appropriate exit code for this error.
    #[must_use]
    pub const fn exit_code(&self) -> ExitCode {
        match self {
            // ConfigurationError
            Self::NoPnpmSpecified | Self::NoYarnSpecified => ExitCode::ConfigurationError,

            // ExecutableNotFound
            Self::NpxUnavailable { .. } => ExitCode::ExecutableNotFound,

            // ExecutionFailure
            Self::Bypass { .. } => ExitCode::ExecutionFailure,

            // InvalidArguments
            Self::Deprecated { .. }
            | Self::InvalidToolVersion { .. }
            | Self::InvalidBareVersion { .. } => ExitCode::InvalidArguments,
        }
    }
}
