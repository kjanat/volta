use std::fmt;
use std::path::PathBuf;

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

const REPORT_BUG_CTA: &str =
    "Please rerun the command that triggered this error with the environment
variable `VOLTA_LOGLEVEL` set to `debug` and open an issue at
https://github.com/volta-cli/volta/issues with the details!";

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

    /// Thrown when the Completions out-dir is not a directory
    CompletionsOutFileError { path: PathBuf },

    /// Thrown when unable to start the migration executable
    CouldNotStartMigration,

    /// Thrown when `volta.extends` keys result in an infinite cycle
    ExtensionCycleError {
        paths: Vec<PathBuf>,
        duplicate: PathBuf,
    },

    /// Thrown when determining the path to an extension manifest fails
    ExtensionPathError { path: PathBuf },

    /// Thrown when unable to parse the node index cache
    ParseNodeIndexCacheError,

    /// Thrown when unable to parse the node index cache expiration
    ParseNodeIndexExpiryError,

    /// Thrown when unable to parse the npm manifest file from a node install
    ParseNpmManifestError,

    /// Thrown when a given feature has not yet been implemented
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
            Self::CompletionsOutFileError { path } => write!(
                f,
                "Completions file `{}` already exists.

Please remove the file or pass `-f` or `--force` to override.",
                path.display()
            ),
            Self::CouldNotStartMigration => write!(
                f,
                "Could not start migration process to upgrade your Volta directory.

Please ensure you have 'volta-migrate' on your PATH and run it directly."
            ),
            Self::ExtensionCycleError { paths, duplicate } => {
                // Detected infinite loop in project workspace:
                //
                // --> /home/user/workspace/project/package.json
                //     /home/user/workspace/package.json
                // --> /home/user/workspace/project/package.json
                //
                // Please ensure that project workspaces do not depend on each other.
                f.write_str("Detected infinite loop in project workspace:\n\n")?;

                for path in paths {
                    if path == duplicate {
                        f.write_str("--> ")?;
                    } else {
                        f.write_str("    ")?;
                    }

                    writeln!(f, "{}", path.display())?;
                }

                writeln!(f, "--> {}", duplicate.display())?;
                writeln!(f)?;

                f.write_str("Please ensure that project workspaces do not depend on each other.")
            }
            Self::ExtensionPathError { path } => write!(
                f,
                "Could not determine path to project workspace: '{}'

Please ensure that the file exists and is accessible.",
                path.display(),
            ),
            Self::ParseNodeIndexCacheError => write!(
                f,
                "Could not parse Node index cache file.

{REPORT_BUG_CTA}"
            ),
            Self::ParseNodeIndexExpiryError => write!(
                f,
                "Could not parse Node index cache expiration file.

{REPORT_BUG_CTA}"
            ),
            Self::ParseNpmManifestError => write!(
                f,
                "Could not parse package.json file for bundled npm.

Please ensure the version of Node is correct."
            ),
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

            // ConfigurationError
            Self::ExtensionCycleError { .. } => ExitCode::ConfigurationError,

            // EnvironmentError
            Self::CouldNotStartMigration => ExitCode::EnvironmentError,

            // FileSystemError
            Self::ExtensionPathError { .. } => ExitCode::FileSystemError,

            // InvalidArguments
            Self::CompletionsOutFileError { .. } => ExitCode::InvalidArguments,

            // UnknownError
            Self::ParseNodeIndexCacheError
            | Self::ParseNodeIndexExpiryError
            | Self::ParseNpmManifestError
            | Self::Unimplemented { .. } => ExitCode::UnknownError,
        }
    }
}
