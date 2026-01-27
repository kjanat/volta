use std::fmt;
use std::path::PathBuf;

use super::binary::BinaryError;
use super::shim::ShimError;
use super::ExitCode;
use crate::style::{text_width, tool_version};
use crate::tool;
use crate::tool::package::PackageManager;
use textwrap::{fill, indent};

const REPORT_BUG_CTA: &str =
    "Please rerun the command that triggered this error with the environment
variable `VOLTA_LOGLEVEL` set to `debug` and open an issue at
https://github.com/volta-cli/volta/issues with the details!";

const PERMISSIONS_CTA: &str = "Please ensure you have correct permissions to the Volta directory.";

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum ErrorKind {
    /// Wrapper for binary-related errors.
    Binary(BinaryError),

    /// Wrapper for shim-related errors.
    Shim(ShimError),

    /// Thrown when building the virtual environment path fails
    BuildPathError,

    /// Thrown when unable to launch a command with `VOLTA_BYPASS` set
    BypassError {
        command: String,
    },

    /// Thrown when a user tries to `volta fetch` something other than node/yarn/npm.
    CannotFetchPackage {
        package: String,
    },

    /// Thrown when a user tries to `volta pin` something other than node/yarn/npm.
    CannotPinPackage {
        package: String,
    },

    /// Thrown when the Completions out-dir is not a directory
    CompletionsOutFileError {
        path: PathBuf,
    },

    /// Thrown when the containing directory could not be determined
    ContainingDirError {
        path: PathBuf,
    },

    CouldNotDetermineTool,

    /// Thrown when unable to start the migration executable
    CouldNotStartMigration,

    CreateDirError {
        dir: PathBuf,
    },

    /// Thrown when unable to create the layout file
    CreateLayoutFileError {
        file: PathBuf,
    },

    /// Thrown when unable to create a link to the shared global library directory
    CreateSharedLinkError {
        name: String,
    },

    /// Thrown when creating a temporary directory fails
    CreateTempDirError {
        in_dir: PathBuf,
    },

    /// Thrown when creating a temporary file fails
    CreateTempFileError {
        in_dir: PathBuf,
    },

    CurrentDirError,

    /// Thrown when deleting a directory fails
    DeleteDirectoryError {
        directory: PathBuf,
    },

    /// Thrown when deleting a file fails
    DeleteFileError {
        file: PathBuf,
    },

    DeprecatedCommandError {
        command: String,
        advice: String,
    },

    DownloadToolNetworkError {
        tool: tool::ToolSpec,
        from_url: String,
    },

    /// Thrown when unable to execute a hook command
    ExecuteHookError {
        command: String,
    },

    /// Thrown when `volta.extends` keys result in an infinite cycle
    ExtensionCycleError {
        paths: Vec<PathBuf>,
        duplicate: PathBuf,
    },

    /// Thrown when determining the path to an extension manifest fails
    ExtensionPathError {
        path: PathBuf,
    },

    /// Thrown when a hook command returns a non-zero exit code
    HookCommandFailed {
        command: String,
    },

    /// Thrown when a hook contains multiple fields (prefix, template, or bin)
    HookMultipleFieldsSpecified,

    /// Thrown when a hook doesn't contain any of the known fields (prefix, template, or bin)
    HookNoFieldsSpecified,

    /// Thrown when determining the path to a hook fails
    HookPathError {
        command: String,
    },

    /// Thrown when determining the name of a newly-installed package fails
    InstalledPackageNameError,

    InvalidHookCommand {
        command: String,
    },

    /// Thrown when output from a hook command could not be read
    InvalidHookOutput {
        command: String,
    },

    /// Thrown when a user does e.g. `volta install node 12` instead of
    /// `volta install node@12`.
    InvalidInvocation {
        action: String,
        name: String,
        version: String,
    },

    /// Thrown when a user does e.g. `volta install 12` instead of
    /// `volta install node@12`.
    InvalidInvocationOfBareVersion {
        action: String,
        version: String,
    },

    /// Thrown when a format other than "npm" or "github" is given for yarn.index in the hooks
    InvalidRegistryFormat {
        format: String,
    },

    /// Thrown when a tool name is invalid per npm's rules.
    InvalidToolName {
        name: String,
        errors: Vec<String>,
    },

    /// Thrown when unable to acquire a lock on the Volta directory
    LockAcquireError,

    /// Thrown when pinning or installing npm@bundled and couldn't detect the bundled version
    NoBundledNpm {
        command: String,
    },

    /// Thrown when pnpm is not set at the command-line
    NoCommandLinePnpm,

    /// Thrown when Yarn is not set at the command-line
    NoCommandLineYarn,

    /// Thrown when a user tries to install a Yarn or npm version before installing a Node version.
    NoDefaultNodeVersion {
        tool: String,
    },

    /// Thrown when there is no Node version matching a requested semver specifier.
    NodeVersionNotFound {
        matching: String,
    },

    NoHomeEnvironmentVar,

    /// Thrown when the install dir could not be determined
    NoInstallDir,

    NoLocalDataDir,

    /// Thrown when a user tries to pin a npm, pnpm, or Yarn version before pinning a Node version.
    NoPinnedNodeVersion {
        tool: String,
    },

    /// Thrown when the platform (Node version) could not be determined
    NoPlatform,

    /// Thrown when parsing the project manifest and there is a `"volta"` key without Node
    NoProjectNodeInManifest,

    /// Thrown when Yarn is not set in a project
    NoProjectYarn,

    /// Thrown when pnpm is not set in a project
    NoProjectPnpm,

    /// Thrown when no shell profiles could be found
    NoShellProfile {
        env_profile: String,
        bin_dir: PathBuf,
    },

    /// Thrown when the user tries to pin Node or Yarn versions outside of a package.
    NotInPackage,

    /// Thrown when default Yarn is not set
    NoDefaultYarn,

    /// Thrown when default pnpm is not set
    NoDefaultPnpm,

    /// Thrown when `npm link` is called with a package that isn't available
    NpmLinkMissingPackage {
        package: String,
    },

    /// Thrown when `npm link` is called with a package that was not installed / linked with npm
    NpmLinkWrongManager {
        package: String,
    },

    /// Thrown when there is no npm version matching the requested Semver/Tag
    NpmVersionNotFound {
        matching: String,
    },

    NpxNotAvailable {
        version: String,
    },

    /// Thrown when the command to install a global package is not successful
    PackageInstallFailed {
        package: String,
    },

    /// Thrown when parsing the package manifest fails
    PackageManifestParseError {
        package: String,
    },

    /// Thrown when reading the package manifest fails
    PackageManifestReadError {
        package: String,
    },

    /// Thrown when a specified package could not be found on the npm registry
    PackageNotFound {
        package: String,
    },

    /// Thrown when parsing a package manifest fails
    PackageParseError {
        file: PathBuf,
    },

    /// Thrown when reading a package manifest fails
    PackageReadError {
        file: PathBuf,
    },

    /// Thrown when a package has been unpacked but is not formed correctly.
    PackageUnpackError,

    /// Thrown when writing a package manifest fails
    PackageWriteError {
        file: PathBuf,
    },

    /// Thrown when unable to parse a hooks.json file
    ParseHooksError {
        file: PathBuf,
    },

    /// Thrown when unable to parse the node index cache
    ParseNodeIndexCacheError,

    /// Thrown when unable to parse the node index
    ParseNodeIndexError {
        from_url: String,
    },

    /// Thrown when unable to parse the node index cache expiration
    ParseNodeIndexExpiryError,

    /// Thrown when unable to parse the npm manifest file from a node install
    ParseNpmManifestError,

    /// Thrown when unable to parse a package configuration
    ParsePackageConfigError,

    /// Thrown when unable to parse the platform.json file
    ParsePlatformError,

    /// Thrown when unable to parse a tool spec (`<tool>[@<version>]`)
    ParseToolSpecError {
        tool_spec: String,
    },

    /// Thrown when persisting an archive to the inventory fails
    PersistInventoryError {
        tool: String,
    },

    /// Thrown when there is no pnpm version matching a requested semver specifier.
    PnpmVersionNotFound {
        matching: String,
    },

    /// Thrown when a publish hook contains both the url and bin fields
    PublishHookBothUrlAndBin,

    /// Thrown when a publish hook contains neither url nor bin fields
    PublishHookNeitherUrlNorBin,

    /// Thrown when unable to read the default npm version file
    ReadDefaultNpmError {
        file: PathBuf,
    },

    /// Thrown when unable to read the contents of a directory
    ReadDirError {
        dir: PathBuf,
    },

    /// Thrown when there was an error opening a hooks.json file
    ReadHooksError {
        file: PathBuf,
    },

    /// Thrown when there was an error reading the Node Index Cache
    ReadNodeIndexCacheError {
        file: PathBuf,
    },

    /// Thrown when there was an error reading the Node Index Cache Expiration
    ReadNodeIndexExpiryError {
        file: PathBuf,
    },

    /// Thrown when there was an error reading the npm manifest file
    ReadNpmManifestError,

    /// Thrown when there was an error reading a package configuration file
    ReadPackageConfigError {
        file: PathBuf,
    },

    /// Thrown when there was an error opening the user platform file
    ReadPlatformError {
        file: PathBuf,
    },

    /// Thrown when unable to read the user Path environment variable from the registry
    #[cfg(windows)]
    ReadUserPathError,

    /// Thrown when the public registry for Node or Yarn could not be downloaded.
    RegistryFetchError {
        tool: String,
        from_url: String,
    },

    /// Thrown when there was an error setting a tool to executable
    SetToolExecutable {
        tool: String,
    },

    /// Thrown when there was an error copying an unpacked tool to the image directory
    SetupToolImageError {
        tool: String,
        version: String,
        dir: PathBuf,
    },

    /// Thrown when serializing a bin config to JSON fails
    StringifyBinConfigError,

    /// Thrown when serializing a package config to JSON fails
    StringifyPackageConfigError,

    /// Thrown when serializing the platform to JSON fails
    StringifyPlatformError,

    /// Thrown when a given feature has not yet been implemented
    Unimplemented {
        feature: String,
    },

    /// Thrown when unpacking an archive (tarball or zip) fails
    UnpackArchiveError {
        tool: String,
        version: String,
    },

    /// Thrown when a package to upgrade was not found
    UpgradePackageNotFound {
        package: String,
        manager: PackageManager,
    },

    /// Thrown when a package to upgrade was installed with a different package manager
    UpgradePackageWrongManager {
        package: String,
        manager: PackageManager,
    },

    VersionParseError {
        version: String,
    },

    /// Thrown when there was an error writing a bin config file
    WriteBinConfigError {
        file: PathBuf,
    },

    /// Thrown when there was an error writing the default npm to file
    WriteDefaultNpmError {
        file: PathBuf,
    },

    /// Thrown when there was an error writing the npm launcher
    WriteLauncherError {
        tool: String,
    },

    /// Thrown when there was an error writing the node index cache
    WriteNodeIndexCacheError {
        file: PathBuf,
    },

    /// Thrown when there was an error writing the node index expiration
    WriteNodeIndexExpiryError {
        file: PathBuf,
    },

    /// Thrown when there was an error writing a package config
    WritePackageConfigError {
        file: PathBuf,
    },

    /// Thrown when writing the platform.json file fails
    WritePlatformError {
        file: PathBuf,
    },

    /// Thrown when unable to write the user PATH environment variable
    #[cfg(windows)]
    WriteUserPathError,

    /// Thrown when a user attempts to install a version of Yarn2
    Yarn2NotSupported,

    /// Thrown when there is an error fetching the latest version of Yarn
    YarnLatestFetchError {
        from_url: String,
    },

    /// Thrown when there is no Yarn version matching a requested semver specifier.
    YarnVersionNotFound {
        matching: String,
    },
}

impl fmt::Display for ErrorKind {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Binary(e) => e.fmt(f),
            Self::Shim(e) => e.fmt(f),
            Self::BuildPathError => write!(
                f,
                "Could not create execution environment.

Please ensure your PATH is valid."
            ),
            Self::BypassError { command } => write!(
                f,
                "Could not execute command '{command}'

VOLTA_BYPASS is enabled, please ensure that the command exists on your system or unset VOLTA_BYPASS",
            ),
            Self::CannotFetchPackage { package } => write!(
                f,
                "Fetching packages without installing them is not supported.

Use `volta install {package}` to update the default version."
            ),
            Self::CannotPinPackage { package } => write!(
                f,
                "Only node and yarn can be pinned in a project

Use `npm install` or `yarn add` to select a version of {package} for this project."
            ),
            Self::CompletionsOutFileError { path } => write!(
                f,
                "Completions file `{}` already exists.

Please remove the file or pass `-f` or `--force` to override.",
                path.display()
            ),
            Self::ContainingDirError { path } => write!(
                f,
                "Could not create the containing directory for {}

{}",
                path.display(),
                PERMISSIONS_CTA
            ),
            Self::CouldNotDetermineTool => write!(
                f,
                "Could not determine tool name

{REPORT_BUG_CTA}"
            ),
            Self::CouldNotStartMigration => write!(
                f,
                "Could not start migration process to upgrade your Volta directory.

Please ensure you have 'volta-migrate' on your PATH and run it directly."
            ),
            Self::CreateDirError { dir } => write!(
                f,
                "Could not create directory {}

Please ensure that you have the correct permissions.",
                dir.display()
            ),
            Self::CreateLayoutFileError { file } => write!(
                f,
                "Could not create layout file {}

{}",
                file.display(), PERMISSIONS_CTA
            ),
            Self::CreateSharedLinkError { name } => write!(
                f,
                "Could not create shared environment for package '{name}'

{PERMISSIONS_CTA}"
            ),
            Self::CreateTempDirError { in_dir } => write!(
                f,
                "Could not create temporary directory
in {}

{}",
                in_dir.display(),
                PERMISSIONS_CTA
            ),
            Self::CreateTempFileError { in_dir } => write!(
                f,
                "Could not create temporary file
in {}

{}",
                in_dir.display(),
                PERMISSIONS_CTA
            ),
            Self::CurrentDirError => write!(
                f,
                "Could not determine current directory

Please ensure that you have the correct permissions."
            ),
            Self::DeleteDirectoryError { directory } => write!(
                f,
                "Could not remove directory
at {}

{}",
                directory.display(),
                PERMISSIONS_CTA
            ),
            Self::DeleteFileError { file } => write!(
                f,
                "Could not remove file
at {}

{}",
                file.display(),
                PERMISSIONS_CTA
            ),
            Self::DeprecatedCommandError { command, advice } => {
                write!(f, "The subcommand `{command}` is deprecated.\n{advice}")
            }
            Self::DownloadToolNetworkError { tool, from_url } => write!(
                f,
                "Could not download {tool}
from {from_url}

Please verify your internet connection and ensure the correct version is specified."
            ),
            Self::ExecuteHookError { command } => write!(
                f,
                "Could not execute hook command: '{command}'

Please ensure that the correct command is specified."
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
            Self::HookCommandFailed { command } => write!(
                f,
                "Hook command '{command}' indicated a failure.

Please verify the requested tool and version."
            ),
            Self::HookMultipleFieldsSpecified => write!(
                f,
                "Hook configuration includes multiple hook types.

Please include only one of 'bin', 'prefix', or 'template'"
            ),
            Self::HookNoFieldsSpecified => write!(
                f,
                "Hook configuration includes no hook types.

Please include one of 'bin', 'prefix', or 'template'"
            ),
            Self::HookPathError { command } => write!(
                f,
                "Could not determine path to hook command: '{command}'

Please ensure that the correct command is specified."
            ),
            Self::InstalledPackageNameError => write!(
                f,
                "Could not determine the name of the package that was just installed.

{REPORT_BUG_CTA}"
            ),
            Self::InvalidHookCommand { command } => write!(
                f,
                "Invalid hook command: '{command}'

Please ensure that the correct command is specified."
            ),
            Self::InvalidHookOutput { command } => write!(
                f,
                "Could not read output from hook command: '{command}'

Please ensure that the command output is valid UTF-8 text."
            ),

            Self::InvalidInvocation {
                action,
                name,
                version,
            } => {
                let error = format!(
                    "`volta {action} {name} {version}` is not supported."
                );

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

            Self::InvalidInvocationOfBareVersion {
                action,
                version,
            } => {
                let error = format!(
                    "`volta {action} {version}` is not supported."
                );

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

            Self::InvalidRegistryFormat { format } => write!(
                f,
                "Unrecognized index registry format: '{format}'

Please specify either 'npm' or 'github' for the format."
            ),

            Self::InvalidToolName { name, errors } => {
                let indentation = "    ";
                let joined = errors.join("\n");
                let wrapped = text_width()
                    .map_or_else(|| joined.clone(), |width| fill(&joined, width - indentation.len()));
                let formatted_errs = indent(&wrapped, indentation);

                let call_to_action = if errors.len() > 1 {
                    "Please fix the following errors:"
                } else {
                    "Please fix the following error:"
                };

                write!(
                    f,
                    "Invalid tool name `{name}`\n\n{call_to_action}\n{formatted_errs}"
                )
            }
            // Note: No CTA as this error is purely informational and shouldn't be exposed to the user
            Self::LockAcquireError => write!(
                f,
                "Unable to acquire lock on Volta directory"
            ),
            Self::NoBundledNpm { command } => write!(
                f,
                "Could not detect bundled npm version.

Please ensure you have a Node version selected with `volta {command} node` (see `volta help {command}` for more info)."
            ),
            Self::NoCommandLinePnpm => write!(
                f,
                "No pnpm version specified.

Use `volta run --pnpm` to select a version (see `volta help run` for more info)."
            ),
            Self::NoCommandLineYarn => write!(
                f,
                "No Yarn version specified.

Use `volta run --yarn` to select a version (see `volta help run` for more info)."
            ),
            Self::NoDefaultNodeVersion { tool } => write!(
                f,
                "Cannot install {tool} because the default Node version is not set.

Use `volta install node` to select a default Node first, then install a {tool} version."
            ),
            Self::NodeVersionNotFound { matching } => write!(
                f,
                r#"Could not find Node version matching "{matching}" in the version registry.

Please verify that the version is correct."#
            ),
            Self::NoHomeEnvironmentVar => write!(
                f,
                "Could not determine home directory.

Please ensure the environment variable 'HOME' is set."
            ),
            Self::NoInstallDir => write!(
                f,
                "Could not determine Volta install directory.

Please ensure Volta was installed correctly"
            ),
            Self::NoLocalDataDir => write!(
                f,
                "Could not determine LocalAppData directory.

Please ensure the directory is available."
            ),
            Self::NoPinnedNodeVersion { tool } => write!(
                f,
                "Cannot pin {tool} because the Node version is not pinned in this project.

Use `volta pin node` to pin Node first, then pin a {tool} version."
            ),
            Self::NoPlatform => write!(
                f,
                "Node is not available.

To run any Node command, first set a default version using `volta install node`"
            ),
            Self::NoProjectNodeInManifest => write!(
                f,
                "No Node version found in this project.

Use `volta pin node` to select a version (see `volta help pin` for more info)."
            ),
            Self::NoProjectPnpm => write!(
                f,
                "No pnpm version found in this project.

Use `volta pin pnpm` to select a version (see `volta help pin` for more info)."
            ),
            Self::NoProjectYarn => write!(
                f,
                "No Yarn version found in this project.

Use `volta pin yarn` to select a version (see `volta help pin` for more info)."
            ),
            Self::NoShellProfile { env_profile, bin_dir } => write!(
                f,
                "Could not locate user profile.
Tried $PROFILE ({}), ~/.bashrc, ~/.bash_profile, ~/.zshenv ~/.zshrc, ~/.profile, and ~/.config/fish/config.fish

Please create one of these and try again; or you can edit your profile manually to add '{}' to your PATH",
                env_profile, bin_dir.display()
            ),
            Self::NotInPackage => write!(
                f,
                "Not in a node package.

Use `volta install` to select a default version of a tool."
            ),
            Self::NoDefaultPnpm => write!(
                f,
                "pnpm is not available.

Use `volta install pnpm` to select a default version (see `volta help install` for more info)."
            ),
            Self::NoDefaultYarn => write!(
                f,
                "Yarn is not available.

Use `volta install yarn` to select a default version (see `volta help install` for more info)."
            ),
            Self::NpmLinkMissingPackage { package } => write!(
                f,
                "Could not locate the package '{package}'

Please ensure it is available by running `npm link` in its source directory."
            ),
            Self::NpmLinkWrongManager { package } => write!(
                f,
                "The package '{package}' was not installed using npm and cannot be linked with `npm link`

Please ensure it is linked with `npm link` or installed with `npm i -g {package}`."
            ),
            Self::NpmVersionNotFound { matching } => write!(
                f,
                r#"Could not find Node version matching "{matching}" in the version registry.

Please verify that the version is correct."#
            ),
            Self::NpxNotAvailable { version } => write!(
                f,
                "'npx' is only available with npm >= 5.2.0

This project is configured to use version {version} of npm."
            ),
            Self::PackageInstallFailed { package } => write!(
                f,
                "Could not install package '{package}'

Please confirm the package is valid and run with `--verbose` for more diagnostics."
            ),
            Self::PackageManifestParseError { package } => write!(
                f,
                "Could not parse package.json manifest for {package}

Please ensure the package includes a valid manifest file."
            ),
            Self::PackageManifestReadError { package } => write!(
                f,
                "Could not read package.json manifest for {package}

Please ensure the package includes a valid manifest file."
            ),
            Self::PackageNotFound { package } => write!(
                f,
                "Could not find '{package}' in the package registry.

Please verify the requested package is correct."
            ),
            Self::PackageParseError { file } => write!(
                f,
                "Could not parse project manifest
at {}

Please ensure that the file is correctly formatted.",
                file.display()
            ),
            Self::PackageReadError { file } => write!(
                f,
                "Could not read project manifest
from {}

Please ensure that the file exists.",
                file.display()
            ),
            Self::PackageUnpackError => write!(
                f,
                "Could not determine package directory layout.

Please ensure the package is correctly formatted."
            ),
            Self::PackageWriteError { file } => write!(
                f,
                "Could not write project manifest
to {}

Please ensure you have correct permissions.",
                file.display()
            ),
            Self::ParseHooksError { file } => write!(
                f,
                "Could not parse hooks configuration file.
from {}

Please ensure the file is correctly formatted.",
                file.display()
            ),
            Self::ParseNodeIndexCacheError => write!(
                f,
                "Could not parse Node index cache file.

{REPORT_BUG_CTA}"
            ),
            Self::ParseNodeIndexError { from_url } => write!(
                f,
                "Could not parse Node version index
from {from_url}

Please verify your internet connection."
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
            Self::ParsePackageConfigError => write!(
                f,
                "Could not parse package configuration file.

{REPORT_BUG_CTA}"
            ),
            Self::ParsePlatformError => write!(
                f,
                "Could not parse platform settings file.

{REPORT_BUG_CTA}"
            ),
            Self::ParseToolSpecError { tool_spec } => write!(
                f,
                "Could not parse tool spec `{tool_spec}`

Please supply a spec in the format `<tool name>[@<version>]`."
            ),
            Self::PersistInventoryError { tool } => write!(
                f,
                "Could not store {tool} archive in inventory cache

{PERMISSIONS_CTA}"
            ),
            Self::PnpmVersionNotFound { matching } => write!(
                f,
                r#"Could not find pnpm version matching "{matching}" in the version registry.

Please verify that the version is correct."#
            ),
            Self::PublishHookBothUrlAndBin => write!(
                f,
                "Publish hook configuration includes both hook types.

Please include only one of 'bin' or 'url'"
            ),
            Self::PublishHookNeitherUrlNorBin => write!(
                f,
                "Publish hook configuration includes no hook types.

Please include one of 'bin' or 'url'"
            ),
            Self::ReadDefaultNpmError { file } => write!(
                f,
                "Could not read default npm version
from {}

{}",
                file.display(),
                PERMISSIONS_CTA
            ),
            Self::ReadDirError { dir } => write!(
                f,
                "Could not read contents from directory {}

{}",
                dir.display(), PERMISSIONS_CTA
            ),
            Self::ReadHooksError { file } => write!(
                f,
                "Could not read hooks file
from {}

{}",
                file.display(),
                PERMISSIONS_CTA
            ),
            Self::ReadNodeIndexCacheError { file } => write!(
                f,
                "Could not read Node index cache
from {}

{}",
                file.display(),
                PERMISSIONS_CTA
            ),
            Self::ReadNodeIndexExpiryError { file } => write!(
                f,
                "Could not read Node index cache expiration
from {}

{}",
                file.display(),
                PERMISSIONS_CTA
            ),
            Self::ReadNpmManifestError => write!(
                f,
                "Could not read package.json file for bundled npm.

Please ensure the version of Node is correct."
            ),
            Self::ReadPackageConfigError { file } => write!(
                f,
                "Could not read package configuration file
from {}

{}",
                file.display(),
                PERMISSIONS_CTA
            ),
            Self::ReadPlatformError { file } => write!(
                f,
                "Could not read default platform file
from {}

{}",
                file.display(),
                PERMISSIONS_CTA
            ),
            #[cfg(windows)]
            ErrorKind::ReadUserPathError => write!(
                f,
                "Could not read user Path environment variable.

Please ensure you have access to the your environment variables."
            ),
            Self::RegistryFetchError { tool, from_url } => write!(
                f,
                "Could not download {tool} version registry
from {from_url}

Please verify your internet connection."
            ),
            Self::SetToolExecutable { tool } => write!(
                f,
                r#"Could not set "{tool}" to executable

{PERMISSIONS_CTA}"#
            ),
            Self::SetupToolImageError { tool, version, dir } => write!(
                f,
                "Could not create environment for {} v{}
at {}

{}",
                tool,
                version,
                dir.display(),
                PERMISSIONS_CTA
            ),
            Self::StringifyBinConfigError => write!(
                f,
                "Could not serialize executable configuration.

{REPORT_BUG_CTA}"
            ),
            Self::StringifyPackageConfigError => write!(
                f,
                "Could not serialize package configuration.

{REPORT_BUG_CTA}"
            ),
            Self::StringifyPlatformError => write!(
                f,
                "Could not serialize platform settings.

{REPORT_BUG_CTA}"
            ),
            Self::Unimplemented { feature } => {
                write!(f, "{feature} is not supported yet.")
            }
            Self::UnpackArchiveError { tool, version } => write!(
                f,
                "Could not unpack {tool} v{version}

Please ensure the correct version is specified."
            ),
            Self::UpgradePackageNotFound { package, manager } => write!(
                f,
                r"Could not locate the package '{}' to upgrade.

Please ensure it is installed with `{} {0}`",
                package,
                match manager {
                    PackageManager::Npm => "npm i -g",
                    PackageManager::Pnpm => "pnpm add -g",
                    PackageManager::Yarn => "yarn global add",
                }
            ),
            Self::UpgradePackageWrongManager { package, manager } => {
                let (name, command) = match manager {
                    PackageManager::Npm => ("npm", "npm update -g"),
                    PackageManager::Pnpm => ("pnpm", "pnpm update -g"),
                    PackageManager::Yarn => ("Yarn", "yarn global upgrade"),
                };
                write!(
                    f,
                    r"The package '{package}' was installed using {name}.

To upgrade it, please use the command `{command} {package}`"
                )
            }
            Self::VersionParseError { version } => write!(
                f,
                r#"Could not parse version "{version}"

Please verify the intended version."#
            ),
            Self::WriteBinConfigError { file } => write!(
                f,
                "Could not write executable configuration
to {}

{}",
                file.display(),
                PERMISSIONS_CTA
            ),
            Self::WriteDefaultNpmError { file } => write!(
                f,
                "Could not write bundled npm version
to {}

{}",
                file.display(),
                PERMISSIONS_CTA
            ),
            Self::WriteLauncherError { tool } => write!(
                f,
                "Could not set up launcher for {tool}

This is most likely an intermittent failure, please try again."
            ),
            Self::WriteNodeIndexCacheError { file } => write!(
                f,
                "Could not write Node index cache
to {}

{}",
                file.display(),
                PERMISSIONS_CTA
            ),
            Self::WriteNodeIndexExpiryError { file } => write!(
                f,
                "Could not write Node index cache expiration
to {}

{}",
                file.display(),
                PERMISSIONS_CTA
            ),
            Self::WritePackageConfigError { file } => write!(
                f,
                "Could not write package configuration
to {}

{}",
                file.display(),
                PERMISSIONS_CTA
            ),
            Self::WritePlatformError { file } => write!(
                f,
                "Could not save platform settings
to {}

{}",
                file.display(),
                PERMISSIONS_CTA
            ),
            #[cfg(windows)]
            ErrorKind::WriteUserPathError => write!(
                f,
                "Could not write Path environment variable.

Please ensure you have permissions to edit your environment variables."
            ),
            Self::Yarn2NotSupported => write!(
                f,
                "Yarn version 2 is not recommended for use, and not supported by Volta.

Please use version 3 or greater instead."
            ),
            Self::YarnLatestFetchError { from_url } => write!(
                f,
                "Could not fetch latest version of Yarn
from {from_url}

Please verify your internet connection."
            ),
            Self::YarnVersionNotFound { matching } => write!(
                f,
                r#"Could not find Yarn version matching "{matching}" in the version registry.

Please verify that the version is correct."#
            ),
        }
    }
}

impl ErrorKind {
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub const fn exit_code(&self) -> ExitCode {
        match self {
            Self::Binary(e) => e.exit_code(),
            Self::Shim(e) => e.exit_code(),
            Self::BuildPathError => ExitCode::EnvironmentError,
            Self::BypassError { .. } => ExitCode::ExecutionFailure,
            Self::CannotFetchPackage { .. } => ExitCode::InvalidArguments,
            Self::CannotPinPackage { .. } => ExitCode::InvalidArguments,
            Self::CompletionsOutFileError { .. } => ExitCode::InvalidArguments,
            Self::ContainingDirError { .. } => ExitCode::FileSystemError,
            Self::CouldNotDetermineTool => ExitCode::UnknownError,
            Self::CouldNotStartMigration => ExitCode::EnvironmentError,
            Self::CreateDirError { .. } => ExitCode::FileSystemError,
            Self::CreateLayoutFileError { .. } => ExitCode::FileSystemError,
            Self::CreateSharedLinkError { .. } => ExitCode::FileSystemError,
            Self::CreateTempDirError { .. } => ExitCode::FileSystemError,
            Self::CreateTempFileError { .. } => ExitCode::FileSystemError,
            Self::CurrentDirError => ExitCode::EnvironmentError,
            Self::DeleteDirectoryError { .. } => ExitCode::FileSystemError,
            Self::DeleteFileError { .. } => ExitCode::FileSystemError,
            Self::DeprecatedCommandError { .. } => ExitCode::InvalidArguments,
            Self::DownloadToolNetworkError { .. } => ExitCode::NetworkError,
            Self::ExecuteHookError { .. } => ExitCode::ExecutionFailure,
            Self::ExtensionCycleError { .. } => ExitCode::ConfigurationError,
            Self::ExtensionPathError { .. } => ExitCode::FileSystemError,
            Self::HookCommandFailed { .. } => ExitCode::ConfigurationError,
            Self::HookMultipleFieldsSpecified => ExitCode::ConfigurationError,
            Self::HookNoFieldsSpecified => ExitCode::ConfigurationError,
            Self::HookPathError { .. } => ExitCode::ConfigurationError,
            Self::InstalledPackageNameError => ExitCode::UnknownError,
            Self::InvalidHookCommand { .. } => ExitCode::ExecutableNotFound,
            Self::InvalidHookOutput { .. } => ExitCode::ExecutionFailure,
            Self::InvalidInvocation { .. } => ExitCode::InvalidArguments,
            Self::InvalidInvocationOfBareVersion { .. } => ExitCode::InvalidArguments,
            Self::InvalidRegistryFormat { .. } => ExitCode::ConfigurationError,
            Self::InvalidToolName { .. } => ExitCode::InvalidArguments,
            Self::LockAcquireError => ExitCode::FileSystemError,
            Self::NoBundledNpm { .. } => ExitCode::ConfigurationError,
            Self::NoCommandLinePnpm => ExitCode::ConfigurationError,
            Self::NoCommandLineYarn => ExitCode::ConfigurationError,
            Self::NoDefaultNodeVersion { .. } => ExitCode::ConfigurationError,
            Self::NodeVersionNotFound { .. } => ExitCode::NoVersionMatch,
            Self::NoHomeEnvironmentVar => ExitCode::EnvironmentError,
            Self::NoInstallDir => ExitCode::EnvironmentError,
            Self::NoLocalDataDir => ExitCode::EnvironmentError,
            Self::NoPinnedNodeVersion { .. } => ExitCode::ConfigurationError,
            Self::NoPlatform => ExitCode::ConfigurationError,
            Self::NoProjectNodeInManifest => ExitCode::ConfigurationError,
            Self::NoProjectPnpm => ExitCode::ConfigurationError,
            Self::NoProjectYarn => ExitCode::ConfigurationError,
            Self::NoShellProfile { .. } => ExitCode::EnvironmentError,
            Self::NotInPackage => ExitCode::ConfigurationError,
            Self::NoDefaultPnpm => ExitCode::ConfigurationError,
            Self::NoDefaultYarn => ExitCode::ConfigurationError,
            Self::NpmLinkMissingPackage { .. } => ExitCode::ConfigurationError,
            Self::NpmLinkWrongManager { .. } => ExitCode::ConfigurationError,
            Self::NpmVersionNotFound { .. } => ExitCode::NoVersionMatch,
            Self::NpxNotAvailable { .. } => ExitCode::ExecutableNotFound,
            Self::PackageInstallFailed { .. } => ExitCode::UnknownError,
            Self::PackageManifestParseError { .. } => ExitCode::ConfigurationError,
            Self::PackageManifestReadError { .. } => ExitCode::FileSystemError,
            Self::PackageNotFound { .. } => ExitCode::InvalidArguments,
            Self::PackageParseError { .. } => ExitCode::ConfigurationError,
            Self::PackageReadError { .. } => ExitCode::FileSystemError,
            Self::PackageUnpackError => ExitCode::ConfigurationError,
            Self::PackageWriteError { .. } => ExitCode::FileSystemError,
            Self::ParseHooksError { .. } => ExitCode::ConfigurationError,
            Self::ParseToolSpecError { .. } => ExitCode::InvalidArguments,
            Self::ParseNodeIndexCacheError => ExitCode::UnknownError,
            Self::ParseNodeIndexError { .. } => ExitCode::NetworkError,
            Self::ParseNodeIndexExpiryError => ExitCode::UnknownError,
            Self::ParseNpmManifestError => ExitCode::UnknownError,
            Self::ParsePackageConfigError => ExitCode::UnknownError,
            Self::ParsePlatformError => ExitCode::ConfigurationError,
            Self::PersistInventoryError { .. } => ExitCode::FileSystemError,
            Self::PnpmVersionNotFound { .. } => ExitCode::NoVersionMatch,
            Self::PublishHookBothUrlAndBin => ExitCode::ConfigurationError,
            Self::PublishHookNeitherUrlNorBin => ExitCode::ConfigurationError,
            Self::ReadDefaultNpmError { .. } => ExitCode::FileSystemError,
            Self::ReadDirError { .. } => ExitCode::FileSystemError,
            Self::ReadHooksError { .. } => ExitCode::FileSystemError,
            Self::ReadNodeIndexCacheError { .. } => ExitCode::FileSystemError,
            Self::ReadNodeIndexExpiryError { .. } => ExitCode::FileSystemError,
            Self::ReadNpmManifestError => ExitCode::UnknownError,
            Self::ReadPackageConfigError { .. } => ExitCode::FileSystemError,
            Self::ReadPlatformError { .. } => ExitCode::FileSystemError,
            #[cfg(windows)]
            ErrorKind::ReadUserPathError => ExitCode::EnvironmentError,
            Self::RegistryFetchError { .. } => ExitCode::NetworkError,
            Self::SetupToolImageError { .. } => ExitCode::FileSystemError,
            Self::SetToolExecutable { .. } => ExitCode::FileSystemError,
            Self::StringifyBinConfigError => ExitCode::UnknownError,
            Self::StringifyPackageConfigError => ExitCode::UnknownError,
            Self::StringifyPlatformError => ExitCode::UnknownError,
            Self::Unimplemented { .. } => ExitCode::UnknownError,
            Self::UnpackArchiveError { .. } => ExitCode::UnknownError,
            Self::UpgradePackageNotFound { .. } => ExitCode::ConfigurationError,
            Self::UpgradePackageWrongManager { .. } => ExitCode::ConfigurationError,
            Self::VersionParseError { .. } => ExitCode::NoVersionMatch,
            Self::WriteBinConfigError { .. } => ExitCode::FileSystemError,
            Self::WriteDefaultNpmError { .. } => ExitCode::FileSystemError,
            Self::WriteLauncherError { .. } => ExitCode::FileSystemError,
            Self::WriteNodeIndexCacheError { .. } => ExitCode::FileSystemError,
            Self::WriteNodeIndexExpiryError { .. } => ExitCode::FileSystemError,
            Self::WritePackageConfigError { .. } => ExitCode::FileSystemError,
            Self::WritePlatformError { .. } => ExitCode::FileSystemError,
            #[cfg(windows)]
            ErrorKind::WriteUserPathError => ExitCode::EnvironmentError,
            Self::Yarn2NotSupported => ExitCode::NoVersionMatch,
            Self::YarnLatestFetchError { .. } => ExitCode::NetworkError,
            Self::YarnVersionNotFound { .. } => ExitCode::NoVersionMatch,
        }
    }
}
