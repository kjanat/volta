//! Error types for shim operations.
//!
//! This module contains errors related to:
//! - Shim creation and deletion
//! - Shim symlink operations
//! - Shim directory management

use std::fmt;

use super::ExitCode;

const PERMISSIONS_CTA: &str = "Please ensure you have correct permissions to the Volta directory.";

/// Errors related to shim operations.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum ShimError {
    /// Thrown when Volta is unable to create a shim.
    CreateFailed { name: String },

    /// Thrown when the shim binary is called directly, not through a symlink.
    DirectInvocation,

    /// Thrown when Volta is unable to remove a shim.
    RemoveFailed { name: String },
}

impl fmt::Display for ShimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateFailed { name } => write!(
                f,
                r#"Could not create shim for "{name}"

{PERMISSIONS_CTA}"#
            ),
            Self::DirectInvocation => write!(
                f,
                "'volta-shim' should not be called directly.

Please use the existing shims provided by Volta (node, yarn, etc.) to run tools."
            ),
            Self::RemoveFailed { name } => write!(
                f,
                r#"Could not remove shim for "{name}"

{PERMISSIONS_CTA}"#
            ),
        }
    }
}

impl ShimError {
    /// Returns the appropriate exit code for this error.
    #[must_use]
    pub const fn exit_code(&self) -> ExitCode {
        match self {
            Self::CreateFailed { .. } | Self::RemoveFailed { .. } => ExitCode::FileSystemError,
            Self::DirectInvocation => ExitCode::InvalidArguments,
        }
    }
}
