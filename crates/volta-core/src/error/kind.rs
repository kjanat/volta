use std::fmt;

use super::ExitCode;
use super::binary::BinaryError;
use super::command::CommandError;
use super::environment::EnvironmentError;
use super::filesystem::FilesystemError;
use super::hook::HookError;
use super::network::NetworkError;
use super::package::PackageError;
use super::platform::PlatformError;
use super::shim::ShimError;
use super::tool::ToolError;
use super::version::VersionError;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum ErrorKind {
    /// Wrapper for binary-related errors.
    Binary(BinaryError),

    /// Wrapper for command-related errors.
    Command(CommandError),

    /// Wrapper for environment-related errors.
    Environment(EnvironmentError),

    /// Wrapper for filesystem-related errors.
    Filesystem(FilesystemError),

    /// Wrapper for hook-related errors.
    Hook(HookError),

    /// Wrapper for network-related errors.
    Network(NetworkError),

    /// Wrapper for shim-related errors.
    Shim(ShimError),

    /// Wrapper for tool-related errors.
    Tool(ToolError),

    /// Wrapper for version-related errors.
    Version(VersionError),

    /// Wrapper for platform-related errors.
    Platform(PlatformError),

    /// Wrapper for package-related errors.
    Package(PackageError),

    /// Thrown when a given feature has not yet been implemented.
    Unimplemented { feature: String },
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Binary(e) => e.fmt(f),
            Self::Command(e) => e.fmt(f),
            Self::Environment(e) => e.fmt(f),
            Self::Filesystem(e) => e.fmt(f),
            Self::Hook(e) => e.fmt(f),
            Self::Network(e) => e.fmt(f),
            Self::Platform(e) => e.fmt(f),
            Self::Package(e) => e.fmt(f),
            Self::Shim(e) => e.fmt(f),
            Self::Tool(e) => e.fmt(f),
            Self::Version(e) => e.fmt(f),
            Self::Unimplemented { feature } => {
                write!(f, "{feature} is not supported yet.")
            }
        }
    }
}

impl From<CommandError> for ErrorKind {
    fn from(error: CommandError) -> Self {
        Self::Command(error)
    }
}

impl From<PlatformError> for ErrorKind {
    fn from(error: PlatformError) -> Self {
        Self::Platform(error)
    }
}

impl From<PackageError> for ErrorKind {
    fn from(error: PackageError) -> Self {
        Self::Package(error)
    }
}

impl From<ToolError> for ErrorKind {
    fn from(error: ToolError) -> Self {
        Self::Tool(error)
    }
}

impl From<EnvironmentError> for ErrorKind {
    fn from(error: EnvironmentError) -> Self {
        Self::Environment(error)
    }
}

impl From<FilesystemError> for ErrorKind {
    fn from(error: FilesystemError) -> Self {
        Self::Filesystem(error)
    }
}

impl From<NetworkError> for ErrorKind {
    fn from(error: NetworkError) -> Self {
        Self::Network(error)
    }
}

impl From<VersionError> for ErrorKind {
    fn from(error: VersionError) -> Self {
        Self::Version(error)
    }
}

impl ErrorKind {
    #[must_use]
    pub const fn exit_code(&self) -> ExitCode {
        match self {
            // Delegated errors
            Self::Binary(e) => e.exit_code(),
            Self::Command(e) => e.exit_code(),
            Self::Environment(e) => e.exit_code(),
            Self::Filesystem(e) => e.exit_code(),
            Self::Hook(e) => e.exit_code(),
            Self::Network(e) => e.exit_code(),
            Self::Package(e) => e.exit_code(),
            Self::Platform(e) => e.exit_code(),
            Self::Shim(e) => e.exit_code(),
            Self::Tool(e) => e.exit_code(),
            Self::Version(e) => e.exit_code(),

            // UnknownError
            Self::Unimplemented { .. } => ExitCode::UnknownError,
        }
    }
}
