use std::env;
use std::ffi::OsString;

use super::executor::{Executor, ToolCommand, ToolKind};
use super::{RECURSION_ENV_VAR, debug_active_image, debug_no_platform};
use crate::error::{BinaryError, ErrorKind, Fallible, PlatformError};
use crate::platform::{Platform, System};
use crate::session::{ActivityKind, Session};

/// Build a `ToolCommand` for Node
pub(super) fn command(
    args: &[OsString],
    session: &mut Session,
    ignore_recursion: bool,
) -> Fallible<Executor> {
    session.add_event_start(ActivityKind::Node);
    // Don't re-evaluate the platform if this is a recursive call (unless ignore_recursion is set)
    let platform = if !ignore_recursion && env::var_os(RECURSION_ENV_VAR).is_some() {
        None
    } else {
        Platform::current(session)?
    };

    Ok(ToolCommand::new("node", args, platform, ToolKind::Node).into())
}

/// Determine the execution context (PATH and failure error message) for Node
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
