//! Error types for filesystem operations.
//!
//! This module contains errors related to:
//! - File reading and writing
//! - Directory creation and deletion
//! - Path operations and resolution
//! - Symlink operations

use std::fmt;
use std::path::PathBuf;

use super::ExitCode;

const PERMISSIONS_CTA: &str = "Please ensure you have correct permissions to the Volta directory.";

/// Errors related to filesystem operations.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum FilesystemError {
    // ==================== Create Operations ====================
    /// Could not create a directory.
    CreateDir { dir: PathBuf },

    /// Could not create the layout file.
    CreateLayoutFile { file: PathBuf },

    /// Could not create a link to the shared global library directory.
    CreateSharedLink { name: String },

    /// Could not create a temporary directory.
    CreateTempDir { in_dir: PathBuf },

    /// Could not create a temporary file.
    CreateTempFile { in_dir: PathBuf },

    /// Could not determine the containing directory.
    ContainingDir { path: PathBuf },

    // ==================== Read Operations ====================
    /// Could not determine the current directory.
    CurrentDir,

    /// Could not read contents of a directory.
    ReadDir { dir: PathBuf },

    /// Could not read hooks file.
    ReadHooks { file: PathBuf },

    /// Could not read Node index cache.
    ReadNodeIndexCache { file: PathBuf },

    /// Could not read Node index cache expiration.
    ReadNodeIndexExpiry { file: PathBuf },

    /// Could not read npm manifest file.
    ReadNpmManifest,

    /// Could not read package configuration file.
    ReadPackageConfig { file: PathBuf },

    /// Could not read platform file.
    ReadPlatform { file: PathBuf },

    /// Could not read default npm version file.
    ReadDefaultNpm { file: PathBuf },

    /// Could not read user Path environment variable (Windows only).
    #[cfg(windows)]
    ReadUserPath,

    // ==================== Write Operations ====================
    /// Could not write executable configuration.
    WriteBinConfig { file: PathBuf },

    /// Could not write default npm version.
    WriteDefaultNpm { file: PathBuf },

    /// Could not write launcher.
    WriteLauncher { tool: String },

    /// Could not write Node index cache.
    WriteNodeIndexCache { file: PathBuf },

    /// Could not write Node index cache expiration.
    WriteNodeIndexExpiry { file: PathBuf },

    /// Could not write package configuration.
    WritePackageConfig { file: PathBuf },

    /// Could not write platform settings.
    WritePlatform { file: PathBuf },

    /// Could not write user Path environment variable (Windows only).
    #[cfg(windows)]
    WriteUserPath,

    /// Could not write project manifest.
    WritePackage { file: PathBuf },

    // ==================== Delete Operations ====================
    /// Could not delete a directory.
    DeleteDir { dir: PathBuf },

    /// Could not delete a file.
    DeleteFile { file: PathBuf },
}

impl fmt::Display for FilesystemError {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Create operations
            Self::CreateDir { dir } => write!(
                f,
                "Could not create directory {}

Please ensure that you have the correct permissions.",
                dir.display()
            ),
            Self::CreateLayoutFile { file } => write!(
                f,
                "Could not create layout file {}

{PERMISSIONS_CTA}",
                file.display()
            ),
            Self::CreateSharedLink { name } => write!(
                f,
                "Could not create shared environment for package '{name}'

{PERMISSIONS_CTA}"
            ),
            Self::CreateTempDir { in_dir } => write!(
                f,
                "Could not create temporary directory
in {}

{PERMISSIONS_CTA}",
                in_dir.display()
            ),
            Self::CreateTempFile { in_dir } => write!(
                f,
                "Could not create temporary file
in {}

{PERMISSIONS_CTA}",
                in_dir.display()
            ),
            Self::ContainingDir { path } => write!(
                f,
                "Could not create the containing directory for {}

{PERMISSIONS_CTA}",
                path.display()
            ),

            // Read operations
            Self::CurrentDir => write!(
                f,
                "Could not determine current directory

Please ensure that you have the correct permissions."
            ),
            Self::ReadDir { dir } => write!(
                f,
                "Could not read contents from directory {}

{PERMISSIONS_CTA}",
                dir.display()
            ),
            Self::ReadHooks { file } => write!(
                f,
                "Could not read hooks file
from {}

{PERMISSIONS_CTA}",
                file.display()
            ),
            Self::ReadNodeIndexCache { file } => write!(
                f,
                "Could not read Node index cache
from {}

{PERMISSIONS_CTA}",
                file.display()
            ),
            Self::ReadNodeIndexExpiry { file } => write!(
                f,
                "Could not read Node index cache expiration
from {}

{PERMISSIONS_CTA}",
                file.display()
            ),
            Self::ReadNpmManifest => write!(
                f,
                "Could not read package.json file for bundled npm.

Please ensure the version of Node is correct."
            ),
            Self::ReadPackageConfig { file } => write!(
                f,
                "Could not read package configuration file
from {}

{PERMISSIONS_CTA}",
                file.display()
            ),
            Self::ReadPlatform { file } => write!(
                f,
                "Could not read default platform file
from {}

{PERMISSIONS_CTA}",
                file.display()
            ),
            Self::ReadDefaultNpm { file } => write!(
                f,
                "Could not read default npm version
from {}

{PERMISSIONS_CTA}",
                file.display()
            ),
            #[cfg(windows)]
            Self::ReadUserPath => write!(
                f,
                "Could not read user Path environment variable.

Please ensure you have access to the your environment variables."
            ),

            // Write operations
            Self::WriteBinConfig { file } => write!(
                f,
                "Could not write executable configuration
to {}

{PERMISSIONS_CTA}",
                file.display()
            ),
            Self::WriteDefaultNpm { file } => write!(
                f,
                "Could not write bundled npm version
to {}

{PERMISSIONS_CTA}",
                file.display()
            ),
            Self::WriteLauncher { tool } => write!(
                f,
                "Could not set up launcher for {tool}

This is most likely an intermittent failure, please try again."
            ),
            Self::WriteNodeIndexCache { file } => write!(
                f,
                "Could not write Node index cache
to {}

{PERMISSIONS_CTA}",
                file.display()
            ),
            Self::WriteNodeIndexExpiry { file } => write!(
                f,
                "Could not write Node index cache expiration
to {}

{PERMISSIONS_CTA}",
                file.display()
            ),
            Self::WritePackageConfig { file } => write!(
                f,
                "Could not write package configuration
to {}

{PERMISSIONS_CTA}",
                file.display()
            ),
            Self::WritePlatform { file } => write!(
                f,
                "Could not save platform settings
to {}

{PERMISSIONS_CTA}",
                file.display()
            ),
            #[cfg(windows)]
            Self::WriteUserPath => write!(
                f,
                "Could not write Path environment variable.

Please ensure you have permissions to edit your environment variables."
            ),
            Self::WritePackage { file } => write!(
                f,
                "Could not write project manifest
to {}

Please ensure you have correct permissions.",
                file.display()
            ),

            // Delete operations
            Self::DeleteDir { dir } => write!(
                f,
                "Could not remove directory
at {}

{PERMISSIONS_CTA}",
                dir.display()
            ),
            Self::DeleteFile { file } => write!(
                f,
                "Could not remove file
at {}

{PERMISSIONS_CTA}",
                file.display()
            ),
        }
    }
}

impl FilesystemError {
    /// Returns the appropriate exit code for this error.
    #[must_use]
    pub const fn exit_code(&self) -> ExitCode {
        match self {
            // Create operations - all filesystem errors
            Self::CreateDir { .. }
            | Self::CreateLayoutFile { .. }
            | Self::CreateSharedLink { .. }
            | Self::CreateTempDir { .. }
            | Self::CreateTempFile { .. }
            | Self::ContainingDir { .. } => ExitCode::FileSystemError,

            // Read operations
            Self::CurrentDir => ExitCode::EnvironmentError,
            Self::ReadDir { .. }
            | Self::ReadHooks { .. }
            | Self::ReadNodeIndexCache { .. }
            | Self::ReadNodeIndexExpiry { .. }
            | Self::ReadPackageConfig { .. }
            | Self::ReadPlatform { .. }
            | Self::ReadDefaultNpm { .. } => ExitCode::FileSystemError,
            Self::ReadNpmManifest => ExitCode::UnknownError,
            #[cfg(windows)]
            Self::ReadUserPath => ExitCode::EnvironmentError,

            // Write operations - all filesystem errors except WriteLauncher
            Self::WriteBinConfig { .. }
            | Self::WriteDefaultNpm { .. }
            | Self::WriteNodeIndexCache { .. }
            | Self::WriteNodeIndexExpiry { .. }
            | Self::WritePackageConfig { .. }
            | Self::WritePlatform { .. }
            | Self::WritePackage { .. } => ExitCode::FileSystemError,
            Self::WriteLauncher { .. } => ExitCode::FileSystemError,
            #[cfg(windows)]
            Self::WriteUserPath => ExitCode::EnvironmentError,

            // Delete operations - all filesystem errors
            Self::DeleteDir { .. } | Self::DeleteFile { .. } => ExitCode::FileSystemError,
        }
    }
}
