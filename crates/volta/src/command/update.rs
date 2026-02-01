use nodejs_semver::Version;

use volta_core::error::{CommandError, ExitCode, Fallible};
use volta_core::platform::PlatformSpec;
use volta_core::session::{ActivityKind, Session};
use volta_core::tool::ToolSpec;
use volta_core::version::VersionSpec;

use crate::command::Command;

/// Scope for the update operation.
enum Scope {
    /// Update global default.
    Global,
    /// Update project-pinned version.
    Project,
}

/// Updates one or more tools to the specified or latest versions.
#[derive(clap::Args)]
#[allow(clippy::struct_excessive_bools)] // CLI flags are naturally bools
pub struct Update {
    /// Tools to update, like `node`, `yarn@latest` or `typescript`.
    ///
    /// Note: Version constraints (--major/--minor/--patch) are not supported
    /// for global packages; use explicit versions like `package@^2.0.0` instead.
    #[arg(value_name = "tool[@version]", required = true)]
    tools: Vec<String>,

    /// Update the tool in your global toolchain, even if in a project
    #[arg(long, short = 'g', conflicts_with = "project")]
    global: bool,

    /// Update the tool pinned in the current project (error if not pinned)
    #[arg(long, short = 'p', conflicts_with = "global")]
    project: bool,

    /// Stay within the current major version (e.g., 18.x.x)
    #[arg(long, conflicts_with_all = ["minor", "patch"])]
    major: bool,

    /// Stay within the current minor version (e.g., 18.19.x)
    #[arg(long, conflicts_with_all = ["major", "patch"])]
    minor: bool,

    /// Stay within the current patch version (check for newer builds)
    #[arg(long, conflicts_with_all = ["major", "minor"])]
    patch: bool,
}

impl Command for Update {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Update);

        let result = self.do_update(session);

        let exit_code = match &result {
            Ok(code) => *code,
            Err(err) => err.exit_code(),
        };
        session.add_event_end(ActivityKind::Update, exit_code);

        result
    }
}

impl Update {
    /// Perform the actual update logic.
    fn do_update(self, session: &mut Session) -> Fallible<ExitCode> {
        let in_project = session.project()?.is_some();
        let project_platform = session.project_platform()?.cloned();

        for tool in ToolSpec::from_strings(&self.tools, "update")? {
            let scope = self.determine_scope(&tool, in_project, project_platform.as_ref())?;

            // Determine the version to update to based on constraints
            let version_spec = self.resolve_target_version(&tool, &scope, session)?;

            // Create a new ToolSpec with the resolved version, preserving the variant
            let tool_with_version = with_version_spec(&tool, version_spec);

            match scope {
                Scope::Global => {
                    tool_with_version.resolve_installable(session)?.install(session)?;
                }
                Scope::Project => {
                    tool_with_version.resolve_pinnable(session)?.pin(session)?;
                }
            }
        }

        Ok(ExitCode::Success)
    }

    /// Determine the scope (global vs project) for the update operation.
    ///
    /// # Errors
    ///
    /// Returns `CommandError::NotInProject` if `--project` is specified but not in a project.
    /// Returns `CommandError::NotPinnedInProject` if the tool is not pinned in the project
    /// (either with `--project` flag or during auto-detection in a project context).
    ///
    /// # Panics
    ///
    /// Panics if both `--global` and `--project` flags are set simultaneously.
    /// This should be prevented by clap's `conflicts_with` configuration.
    fn determine_scope(
        &self,
        tool: &ToolSpec,
        in_project: bool,
        project_platform: Option<&PlatformSpec>,
    ) -> Fallible<Scope> {
        match (self.global, self.project, in_project) {
            // Explicit --global or auto-detect outside project → global
            (true, false, _) | (false, false, false) => Ok(Scope::Global),

            // Explicit --project: must be in project with tool pinned
            (false, true, true) => {
                if is_tool_pinned(tool, project_platform) {
                    Ok(Scope::Project)
                } else {
                    Err(CommandError::NotPinnedInProject {
                        tool: tool.name().to_string(),
                    }
                    .into())
                }
            }
            (false, true, false) => Err(CommandError::NotInProject.into()),

            // Auto-detect: in project
            (false, false, true) => {
                if is_tool_pinned(tool, project_platform) {
                    Ok(Scope::Project)
                } else {
                    // Exception: don't silently update global from project context
                    Err(CommandError::NotPinnedInProject {
                        tool: tool.name().to_string(),
                    }
                    .into())
                }
            }

            // Both flags (should be prevented by clap conflicts_with)
            (true, true, _) => unreachable!("clap should prevent --global and --project together"),
        }
    }

    /// Resolve the target version based on constraints (--major, --minor, --patch).
    ///
    /// # Errors
    ///
    /// Returns `CommandError::NoCurrentVersion` if a version constraint is specified
    /// but no current version is installed for the tool.
    /// Returns `CommandError::PackageVersionLookupUnsupported` if a version constraint
    /// is specified for a global package.
    /// Propagates session errors from platform lookup and version parse errors.
    fn resolve_target_version(
        &self,
        tool: &ToolSpec,
        scope: &Scope,
        session: &Session,
    ) -> Fallible<VersionSpec> {
        get_explicit_version(tool).map_or_else(
            || {
                // No explicit version specified, check for constraints
                if !self.major && !self.minor && !self.patch {
                    // No constraints, update to latest
                    return Ok(VersionSpec::default());
                }

                // Get the current version based on scope
                let current_version = get_current_version(tool, scope, session)?;

                // Build a semver range based on constraints
                let range = if self.major {
                    // ^major.0.0 - allows any version with the same major
                    format!("^{}.0.0", current_version.major)
                } else if self.minor {
                    // ~major.minor.0 - allows any version with the same major.minor
                    format!("~{}.{}.0", current_version.major, current_version.minor)
                } else {
                    // ~major.minor.patch - allows patch-level updates (e.g., 18.19.0 -> 18.19.1)
                    format!(
                        "~{}.{}.{}",
                        current_version.major, current_version.minor, current_version.patch
                    )
                };

                range.parse()
            },
            Ok,
        )
    }
}

/// Create a new `ToolSpec` with the given version, preserving the original variant.
#[must_use]
fn with_version_spec(tool: &ToolSpec, version: VersionSpec) -> ToolSpec {
    match tool {
        ToolSpec::Node(_) => ToolSpec::Node(version),
        ToolSpec::Npm(_) => ToolSpec::Npm(version),
        ToolSpec::Pnpm(_) => ToolSpec::Pnpm(version),
        ToolSpec::Yarn(_) => ToolSpec::Yarn(version),
        ToolSpec::Package(name, _) => ToolSpec::Package(name.clone(), version),
    }
}

/// Get explicit version if the user specified one (e.g., node@^20).
#[must_use]
fn get_explicit_version(tool: &ToolSpec) -> Option<VersionSpec> {
    let version = match tool {
        ToolSpec::Node(v)
        | ToolSpec::Npm(v)
        | ToolSpec::Pnpm(v)
        | ToolSpec::Yarn(v)
        | ToolSpec::Package(_, v) => v,
    };

    // If version is not the default (None), user specified something
    if matches!(version, VersionSpec::None) {
        None
    } else {
        Some(version.clone())
    }
}

/// Get the current installed version for the tool based on scope.
///
/// # Errors
///
/// Returns `CommandError::NoCurrentVersion` if no platform is configured or if the
/// specific tool is not installed in the platform.
/// Returns `CommandError::PackageVersionLookupUnsupported` for global packages.
/// Propagates session errors from platform lookup.
fn get_current_version(tool: &ToolSpec, scope: &Scope, session: &Session) -> Fallible<Version> {
    let platform = match scope {
        Scope::Global => session.default_platform()?,
        Scope::Project => session.project_platform()?,
    };

    let platform = platform.ok_or_else(|| CommandError::NoCurrentVersion {
        tool: tool.name().to_string(),
    })?;

    match tool {
        ToolSpec::Node(_) => Ok(platform.node.clone()),
        ToolSpec::Npm(_) => platform.npm.clone().ok_or_else(|| {
            CommandError::NoCurrentVersion {
                tool: "npm".to_string(),
            }
            .into()
        }),
        ToolSpec::Pnpm(_) => platform.pnpm.clone().ok_or_else(|| {
            CommandError::NoCurrentVersion {
                tool: "pnpm".to_string(),
            }
            .into()
        }),
        ToolSpec::Yarn(_) => platform.yarn.clone().ok_or_else(|| {
            CommandError::NoCurrentVersion {
                tool: "yarn".to_string(),
            }
            .into()
        }),
        ToolSpec::Package(name, _) => {
            // Package version lookup is not implemented; inform the user
            Err(CommandError::PackageVersionLookupUnsupported {
                package: name.clone(),
            }
            .into())
        }
    }
}

/// Check if a tool is pinned in the project.
#[must_use]
#[allow(clippy::missing_const_for_fn, reason = "intentionally non-const for future flexibility if PlatformSpec changes")]
fn is_tool_pinned(tool: &ToolSpec, project_platform: Option<&PlatformSpec>) -> bool {
    let Some(platform) = project_platform else {
        return false;
    };

    match tool {
        // Node is always present in a platform spec if there is one
        ToolSpec::Node(_) => true,
        ToolSpec::Npm(_) => platform.npm.is_some(),
        ToolSpec::Pnpm(_) => platform.pnpm.is_some(),
        ToolSpec::Yarn(_) => platform.yarn.is_some(),
        // Packages cannot be pinned in the volta section
        ToolSpec::Package(_, _) => false,
    }
}
