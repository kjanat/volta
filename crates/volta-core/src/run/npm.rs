use std::env;
use std::ffi::OsString;
use std::fs::File;

use super::executor::{Executor, ToolCommand, ToolKind, UninstallCommand};
use super::parser::{CommandArg, InterceptedCommand};
use super::{RECURSION_ENV_VAR, debug_active_image, debug_no_platform};
use crate::error::{BinaryError, ErrorKind, Fallible, PlatformError};
use crate::platform::{Platform, System};
use crate::session::{ActivityKind, Session};
use crate::tool::{PackageManifest, ToolSpec};
use crate::version::VersionSpec;

/// Build an `Executor` for npm
///
/// If the command is a global install or uninstall and we have a default platform available, then
/// we will use custom logic to ensure that the package is correctly installed / uninstalled in the
/// Volta directory.
///
/// If the command is _not_ a global install / uninstall or we don't have a default platform, then
/// we will allow npm to execute the command as usual.
pub(super) fn command(
    args: &[OsString],
    session: &mut Session,
    ignore_recursion: bool,
) -> Fallible<Executor> {
    session.add_event_start(ActivityKind::Npm);
    // Don't re-evaluate the context or global install interception if this is a recursive call
    // (unless ignore_recursion is set)
    let platform = if !ignore_recursion && env::var_os(RECURSION_ENV_VAR).is_some() {
        None
    } else {
        match CommandArg::for_npm(args) {
            CommandArg::Global(cmd) => {
                // For globals, only intercept if the default platform exists
                if let Some(default_platform) = session.default_platform()? {
                    return cmd.executor(default_platform);
                }
            }
            CommandArg::Intercepted(InterceptedCommand::Link(link)) => {
                // For link commands, only intercept if a platform exists
                if let Some(platform) = Platform::current(session)? {
                    return link.executor(platform, current_project_name(session));
                }
            }
            CommandArg::Intercepted(InterceptedCommand::Unlink) => {
                // For unlink, attempt to find the current project name. If successful, treat
                // this as a global uninstall of the current project.
                if let Some(name) = current_project_name(session) {
                    // Same as for link, only intercept if a platform exists
                    if Platform::current(session)?.is_some() {
                        return Ok(UninstallCommand::new(ToolSpec::Package(
                            name,
                            VersionSpec::None,
                        ))
                        .into());
                    }
                }
            }
            _ => {}
        }

        Platform::current(session)?
    };

    Ok(ToolCommand::new("npm", args, platform, ToolKind::Npm).into())
}

/// Determine the execution context (PATH and failure error message) for npm
pub(super) fn execution_context(
    platform: Option<Platform>,
    session: &mut Session,
) -> Fallible<(OsString, ErrorKind)> {
    if let Some(plat) = platform {
        let image = plat.checkout(session)?;
        let path = image.path()?;
        debug_active_image(&image);

        Ok((path, ErrorKind::Binary(BinaryError::ExecError)))
    } else {
        let path = System::path()?;
        debug_no_platform();
        Ok((path, ErrorKind::Platform(PlatformError::NoPlatform)))
    }
}

/// Determine the name of the current project, if possible
fn current_project_name(session: &Session) -> Option<String> {
    let project = session.project().ok()??;
    let manifest_file = File::open(project.manifest_file()).ok()?;
    let manifest: PackageManifest = serde_json::de::from_reader(manifest_file).ok()?;

    Some(manifest.name)
}
