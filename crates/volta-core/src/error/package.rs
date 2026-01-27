//! Error types for package operations.
//!
//! This module contains errors related to:
//! - Package manifest parsing (package.json)
//! - Package name validation
//! - Package installation and configuration
//! - Package registry operations

use std::fmt;
use std::path::PathBuf;

use super::ExitCode;
use crate::tool::package::PackageManager;

const REPORT_BUG_CTA: &str =
    "Please rerun the command that triggered this error with the environment
variable `VOLTA_LOGLEVEL` set to `debug` and open an issue at
https://github.com/volta-cli/volta/issues with the details!";

/// Errors related to package operations.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum PackageError {
    /// Thrown when a user tries to `volta fetch` a package (not supported).
    FetchNotSupported { package: String },

    /// Thrown when a user tries to `volta pin` a package (not supported).
    PinNotSupported { package: String },

    /// Thrown when the command to install a global package is not successful.
    InstallFailed { package: String },

    /// Thrown when parsing the package manifest fails.
    ManifestParse { package: String },

    /// Thrown when reading the package manifest fails.
    ManifestRead { package: String },

    /// Thrown when a specified package could not be found on the npm registry.
    NotFound { package: String },

    /// Thrown when parsing a project manifest (package.json) fails.
    ProjectManifestParse { file: PathBuf },

    /// Thrown when reading a project manifest (package.json) fails.
    ProjectManifestRead { file: PathBuf },

    /// Thrown when a package has been unpacked but is not formed correctly.
    UnpackLayout,

    /// Thrown when determining the name of a newly-installed package fails.
    InstalledNameUnknown,

    /// Thrown when unable to parse a package configuration file.
    ConfigParse,

    /// Thrown when `npm link` is called with a package that is not available.
    LinkMissing { package: String },

    /// Thrown when `npm link` is called with a package not installed/linked with npm.
    LinkWrongManager { package: String },

    /// Thrown when a package to upgrade was not found.
    UpgradeNotFound {
        package: String,
        manager: PackageManager,
    },

    /// Thrown when a package to upgrade was installed with a different package manager.
    UpgradeWrongManager {
        package: String,
        manager: PackageManager,
    },

    /// Thrown when `volta.extends` keys result in an infinite cycle.
    WorkspaceCycle {
        paths: Vec<PathBuf>,
        duplicate: PathBuf,
    },

    /// Thrown when determining the path to a workspace manifest fails.
    WorkspacePathInvalid { path: PathBuf },
}

impl fmt::Display for PackageError {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FetchNotSupported { package } => write!(
                f,
                "Fetching packages without installing them is not supported.

Use `volta install {package}` to update the default version."
            ),
            Self::PinNotSupported { package } => write!(
                f,
                "Only node and yarn can be pinned in a project

Use `npm install` or `yarn add` to select a version of {package} for this project."
            ),
            Self::InstallFailed { package } => write!(
                f,
                "Could not install package '{package}'

Please confirm the package is valid and run with `--verbose` for more diagnostics."
            ),
            Self::ManifestParse { package } => write!(
                f,
                "Could not parse package.json manifest for {package}

Please ensure the package includes a valid manifest file."
            ),
            Self::ManifestRead { package } => write!(
                f,
                "Could not read package.json manifest for {package}

Please ensure the package includes a valid manifest file."
            ),
            Self::NotFound { package } => write!(
                f,
                "Could not find '{package}' in the package registry.

Please verify the requested package is correct."
            ),
            Self::ProjectManifestParse { file } => write!(
                f,
                "Could not parse project manifest
at {}

Please ensure that the file is correctly formatted.",
                file.display()
            ),
            Self::ProjectManifestRead { file } => write!(
                f,
                "Could not read project manifest
from {}

Please ensure that the file exists.",
                file.display()
            ),
            Self::UnpackLayout => write!(
                f,
                "Could not determine package directory layout.

Please ensure the package is correctly formatted."
            ),
            Self::InstalledNameUnknown => write!(
                f,
                "Could not determine the name of the package that was just installed.

{REPORT_BUG_CTA}"
            ),
            Self::ConfigParse => write!(
                f,
                "Could not parse package configuration file.

{REPORT_BUG_CTA}"
            ),
            Self::LinkMissing { package } => write!(
                f,
                "Could not locate the package '{package}'

Please ensure it is available by running `npm link` in its source directory."
            ),
            Self::LinkWrongManager { package } => write!(
                f,
                "The package '{package}' was not installed using npm and cannot be linked with `npm link`

Please ensure it is linked with `npm link` or installed with `npm i -g {package}`."
            ),
            Self::UpgradeNotFound { package, manager } => write!(
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
            Self::UpgradeWrongManager { package, manager } => {
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
            Self::WorkspaceCycle { paths, duplicate } => {
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
            Self::WorkspacePathInvalid { path } => write!(
                f,
                "Could not determine path to project workspace: '{}'

Please ensure that the file exists and is accessible.",
                path.display(),
            ),
        }
    }
}

impl PackageError {
    #[must_use]
    pub const fn exit_code(&self) -> ExitCode {
        match self {
            // ConfigurationError
            Self::ManifestParse { .. }
            | Self::ProjectManifestParse { .. }
            | Self::UnpackLayout
            | Self::LinkMissing { .. }
            | Self::LinkWrongManager { .. }
            | Self::UpgradeNotFound { .. }
            | Self::UpgradeWrongManager { .. }
            | Self::WorkspaceCycle { .. } => ExitCode::ConfigurationError,

            // FileSystemError
            Self::ManifestRead { .. }
            | Self::ProjectManifestRead { .. }
            | Self::WorkspacePathInvalid { .. } => ExitCode::FileSystemError,

            // InvalidArguments
            Self::FetchNotSupported { .. }
            | Self::PinNotSupported { .. }
            | Self::NotFound { .. } => ExitCode::InvalidArguments,

            // UnknownError
            Self::InstallFailed { .. } | Self::InstalledNameUnknown | Self::ConfigParse => {
                ExitCode::UnknownError
            }
        }
    }
}
