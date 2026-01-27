//! Error types for tool operations.
//!
//! This module contains errors related to:
//! - Tool fetching and installation
//! - Tool version management
//! - Tool setup and configuration

use std::fmt;
use std::path::PathBuf;

use super::ExitCode;
use crate::style::text_width;
use textwrap::{fill, indent};

const REPORT_BUG_CTA: &str =
    "Please rerun the command that triggered this error with the environment
variable `VOLTA_LOGLEVEL` set to `debug` and open an issue at
https://github.com/volta-cli/volta/issues with the details!";

const PERMISSIONS_CTA: &str = "Please ensure you have correct permissions to the Volta directory.";

/// Errors related to tool operations.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum ToolError {
    // ==================== Determination Errors ====================
    /// Could not determine the tool name from the command.
    CouldNotDetermine,

    // ==================== Parsing Errors ====================
    /// Could not parse a tool spec (`<tool>[@<version>]`).
    ParseSpec { tool_spec: String },

    /// Invalid tool name per npm's rules.
    InvalidName { name: String, errors: Vec<String> },

    // ==================== Archive Errors ====================
    /// Failed to unpack an archive (tarball or zip).
    UnpackArchive { tool: String, version: String },

    /// Failed to persist archive to inventory cache.
    PersistInventory { tool: String },

    // ==================== Setup Errors ====================
    /// Failed to set a tool to executable.
    SetExecutable { tool: String },

    /// Failed to copy an unpacked tool to the image directory.
    SetupImage {
        tool: String,
        version: String,
        dir: PathBuf,
    },

    // ==================== Serialization Errors ====================
    /// Failed to serialize executable configuration.
    SerializeBinConfig,

    /// Failed to serialize package configuration.
    SerializePackageConfig,

    /// Failed to serialize platform settings.
    SerializePlatform,
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Determination errors
            Self::CouldNotDetermine => write!(
                f,
                "Could not determine tool name

{REPORT_BUG_CTA}"
            ),

            // Parsing errors
            Self::ParseSpec { tool_spec } => write!(
                f,
                "Could not parse tool spec `{tool_spec}`

Please supply a spec in the format `<tool name>[@<version>]`."
            ),
            Self::InvalidName { name, errors } => {
                let indentation = "    ";
                let joined = errors.join("\n");
                let wrapped = text_width().map_or_else(
                    || joined.clone(),
                    |width| fill(&joined, width - indentation.len()),
                );
                let formatted_errs = indent(&wrapped, indentation);

                let call_to_action = if errors.len() > 1 {
                    "Please fix the following errors:"
                } else {
                    "Please fix the following error:"
                };

                write!(
                    f,
                    "Invalid tool name `{name}`

{call_to_action}
{formatted_errs}"
                )
            }

            // Archive errors
            Self::UnpackArchive { tool, version } => write!(
                f,
                "Could not unpack {tool} v{version}

Please ensure the correct version is specified."
            ),
            Self::PersistInventory { tool } => write!(
                f,
                "Could not store {tool} archive in inventory cache

{PERMISSIONS_CTA}"
            ),

            // Setup errors
            Self::SetExecutable { tool } => write!(
                f,
                r#"Could not set "{tool}" to executable

{PERMISSIONS_CTA}"#
            ),
            Self::SetupImage { tool, version, dir } => write!(
                f,
                "Could not create environment for {tool} v{version}
at {}

{PERMISSIONS_CTA}",
                dir.display()
            ),

            // Serialization errors
            Self::SerializeBinConfig => write!(
                f,
                "Could not serialize executable configuration.

{REPORT_BUG_CTA}"
            ),
            Self::SerializePackageConfig => write!(
                f,
                "Could not serialize package configuration.

{REPORT_BUG_CTA}"
            ),
            Self::SerializePlatform => write!(
                f,
                "Could not serialize platform settings.

{REPORT_BUG_CTA}"
            ),
        }
    }
}

impl ToolError {
    /// Returns the appropriate exit code for this error.
    #[must_use]
    pub const fn exit_code(&self) -> ExitCode {
        match self {
            // Internal/unknown errors
            Self::CouldNotDetermine
            | Self::UnpackArchive { .. }
            | Self::SerializeBinConfig
            | Self::SerializePackageConfig
            | Self::SerializePlatform => ExitCode::UnknownError,

            // Invalid arguments
            Self::ParseSpec { .. } | Self::InvalidName { .. } => ExitCode::InvalidArguments,

            // Filesystem errors
            Self::SetExecutable { .. }
            | Self::SetupImage { .. }
            | Self::PersistInventory { .. } => ExitCode::FileSystemError,
        }
    }
}
