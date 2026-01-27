use std::collections::HashMap;
use std::env::{self, ArgsOs};
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::ExitStatus;

use crate::error::{ErrorKind, Fallible};
use crate::platform::{Overrides, Image, Sourced};
use crate::session::Session;
use crate::VOLTA_FEATURE_PNPM;
use log::debug;
use nodejs_semver::Version;

pub mod binary;
mod executor;
mod node;
mod npm;
mod npx;
mod parser;
mod pnpm;
mod yarn;

/// Environment variable set internally when a shim has been executed and the context evaluated
///
/// This is set when executing a shim command. If this is already, then the built-in shims (Node,
/// npm, npx, pnpm and Yarn) will assume that the context has already been evaluated & the PATH has
/// already been modified, so they will use the pass-through behavior.
///
/// Shims should only be called recursively when the environment is misconfigured, so this will
/// prevent infinite recursion as the pass-through logic removes the shim directory from the PATH.
///
/// Note: This is explicitly _removed_ when calling a command through `volta run`, as that will
/// never happen due to the Volta environment.
const RECURSION_ENV_VAR: &str = "_VOLTA_TOOL_RECURSION";
const VOLTA_BYPASS: &str = "VOLTA_BYPASS";

/// Execute a shim command, based on the command-line arguments to the current process
///
/// # Errors
///
/// Returns an error if the shim cannot be executed.
pub fn execute_shim(session: &mut Session) -> Fallible<ExitStatus> {
    let mut native_args = env::args_os();
    let exe = get_tool_name(&mut native_args)?;
    let args: Vec<_> = native_args.collect();

    get_executor(&exe, &args, session, false)?.execute(session)
}

/// Execute a tool with the provided arguments
///
/// # Errors
///
/// Returns an error if the tool cannot be executed.
pub fn execute_tool<K, V, S>(
    exe: &OsStr,
    args: &[OsString],
    envs: &HashMap<K, V, S>,
    cli: Overrides,
    session: &mut Session,
) -> Fallible<ExitStatus>
where
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    // Pass ignore_recursion=true so the platform is re-evaluated even if RECURSION_ENV_VAR is set
    // This is needed when `volta run` is called from within a Node script
    let mut runner = get_executor(exe, args, session, true)?;
    runner.cli_platform(cli);
    runner.envs(envs);

    runner.execute(session)
}

/// Get the appropriate Tool command, based on the requested executable and arguments
///
/// When `ignore_recursion` is true, the recursion environment variable check is skipped,
/// allowing the platform to be re-evaluated. This is used by `volta run` to ensure fresh
/// platform evaluation even when called from within a Node script.
fn get_executor(
    exe: &OsStr,
    args: &[OsString],
    session: &mut Session,
    ignore_recursion: bool,
) -> Fallible<executor::Executor> {
    if env::var_os(VOLTA_BYPASS).is_some() {
        Ok(executor::ToolCommand::new(
            exe,
            args,
            None,
            executor::ToolKind::Bypass(exe.to_string_lossy().to_string()),
        )
        .into())
    } else {
        match exe.to_str() {
            Some("volta-shim") => Err(ErrorKind::RunShimDirectly.into()),
            Some("node") => node::command(args, session, ignore_recursion),
            Some("npm") => npm::command(args, session, ignore_recursion),
            Some("npx") => npx::command(args, session, ignore_recursion),
            Some("pnpm") => {
                // If the pnpm feature flag variable is set, delegate to the pnpm handler
                // If not, use the binary handler as a fallback (prior to pnpm support, installing
                // pnpm would be handled the same as any other global binary)
                if env::var_os(VOLTA_FEATURE_PNPM).is_some() {
                    pnpm::command(args, session, ignore_recursion)
                } else {
                    binary::command(exe, args, session)
                }
            }
            Some("yarn" | "yarnpkg") => yarn::command(args, session, ignore_recursion),
            _ => binary::command(exe, args, session),
        }
    }
}

/// Determine the name of the command to run by inspecting the first argument to the active process
fn get_tool_name(args: &mut ArgsOs) -> Fallible<OsString> {
    args.next()
        .and_then(|arg0| Path::new(&arg0).file_name().map(tool_name_from_file_name))
        .ok_or_else(|| ErrorKind::CouldNotDetermineTool.into())
}

#[cfg(unix)]
fn tool_name_from_file_name(file_name: &OsStr) -> OsString {
    file_name.to_os_string()
}

#[cfg(windows)]
fn tool_name_from_file_name(file_name: &OsStr) -> OsString {
    // On Windows PowerShell, the file name includes the .exe suffix,
    // and the Windows file system is case-insensitive
    // We need to remove that to get the raw tool name
    match file_name.to_str() {
        Some(file) => OsString::from(file.to_ascii_lowercase().trim_end_matches(".exe")),
        None => OsString::from(file_name),
    }
}

/// Write a debug message that there is no platform available
#[inline]
fn debug_no_platform() {
    debug!("Could not find Volta-managed platform, delegating to system");
}

/// Write a debug message with the full image that will be used to execute a command
#[inline]
fn debug_active_image(image: &Image) {
    debug!(
        "Active Image:
    Node: {}
    npm: {}
    pnpm: {}
    Yarn: {}",
        format_tool_version(&image.node),
        image
            .resolve_npm()
            .ok()
            .as_ref().map_or_else(|| "Bundled with Node".into(), format_tool_version),
        image
            .pnpm
            .as_ref().map_or_else(|| "None".into(), format_tool_version),
        image
            .yarn
            .as_ref().map_or_else(|| "None".into(), format_tool_version),
    );
}

fn format_tool_version(version: &Sourced<Version>) -> String {
    format!("{} from {} configuration", version.value, version.source)
}
