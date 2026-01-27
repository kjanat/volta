//! Error types for environment variable operations.
//!
//! This module contains errors related to:
//! - Environment variable reading
//! - Path manipulation (`VOLTA_HOME`, `PATH`)
//! - Environment configuration

use std::fmt;
use std::path::PathBuf;

use super::ExitCode;

/// Errors related to environment variable operations.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[allow(clippy::module_name_repetitions)]
pub enum EnvironmentError {
    /// Thrown when building the virtual environment path fails.
    BuildPath,

    /// Thrown when the HOME environment variable is not set.
    NoHome,

    /// Thrown when the install directory could not be determined.
    NoInstallDir,

    /// Thrown when the `LocalAppData` directory is not available (Windows).
    NoLocalData,

    /// Thrown when no shell profiles could be found for setup.
    NoShellProfile {
        env_profile: String,
        bin_dir: PathBuf,
    },

    /// Thrown when unable to acquire a lock on the Volta directory.
    LockAcquire,
}

impl fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuildPath => write!(
                f,
                "Could not create execution environment.

Please ensure your PATH is valid."
            ),
            Self::NoHome => write!(
                f,
                "Could not determine home directory.

Please ensure the environment variable 'HOME' is set."
            ),
            Self::NoInstallDir => write!(
                f,
                "Could not determine Volta install directory.

Please ensure Volta was installed correctly"
            ),
            Self::NoLocalData => write!(
                f,
                "Could not determine LocalAppData directory.

Please ensure the directory is available."
            ),
            Self::NoShellProfile { env_profile, bin_dir } => write!(
                f,
                "Could not locate user profile.
Tried $PROFILE ({}), ~/.bashrc, ~/.bash_profile, ~/.zshenv ~/.zshrc, ~/.profile, and ~/.config/fish/config.fish

Please create one of these and try again; or you can edit your profile manually to add '{}' to your PATH",
                env_profile, bin_dir.display()
            ),
            Self::LockAcquire => write!(f, "Unable to acquire lock on Volta directory"),
        }
    }
}

impl EnvironmentError {
    /// Returns the exit code for this error.
    #[must_use]
    pub const fn exit_code(&self) -> ExitCode {
        match self {
            Self::BuildPath
            | Self::NoHome
            | Self::NoInstallDir
            | Self::NoLocalData
            | Self::NoShellProfile { .. } => ExitCode::EnvironmentError,
            Self::LockAcquire => ExitCode::FileSystemError,
        }
    }
}
