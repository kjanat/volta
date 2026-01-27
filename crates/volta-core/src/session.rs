//! Provides the `Session` type, which represents the user's state during an
//! execution of a Volta tool, including their current directory, Volta
//! hook configuration, and the state of the local inventory.

use std::env;
use std::fmt::{self, Display, Formatter};
use std::process::exit;

use crate::VOLTA_FEATURE_PNPM;
use crate::error::{ExitCode, Fallible, VoltaError};
use crate::event::Log;
use crate::hook::{HookConfig, LazyHookConfig};
use crate::platform::PlatformSpec;
use crate::project::{LazyProject, Project};
use crate::toolchain::{LazyToolchain, Toolchain};
use log::debug;

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub enum ActivityKind {
    Fetch,
    Install,
    Uninstall,
    List,
    Current,
    Default,
    Pin,
    Node,
    Npm,
    Npx,
    Pnpm,
    Yarn,
    Volta,
    Tool,
    Help,
    Version,
    Binary,
    Shim,
    Completions,
    Which,
    Setup,
    Run,
    Args,
}

impl Display for ActivityKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let s = match self {
            Self::Fetch => "fetch",
            Self::Install => "install",
            Self::Uninstall => "uninstall",
            Self::List => "list",
            Self::Current => "current",
            Self::Default => "default",
            Self::Pin => "pin",
            Self::Node => "node",
            Self::Npm => "npm",
            Self::Npx => "npx",
            Self::Pnpm => "pnpm",
            Self::Yarn => "yarn",
            Self::Volta => "volta",
            Self::Tool => "tool",
            Self::Help => "help",
            Self::Version => "version",
            Self::Binary => "binary",
            Self::Setup => "setup",
            Self::Shim => "shim",
            Self::Completions => "completions",
            Self::Which => "which",
            Self::Run => "run",
            Self::Args => "args",
        };
        f.write_str(s)
    }
}

/// Represents the user's state during an execution of a Volta tool. The session
/// encapsulates a number of aspects of the environment in which the tool was
/// invoked, including:
///
/// - the current directory
/// - the Node project tree that contains the current directory (if any)
/// - the Volta hook configuration
/// - the inventory of locally-fetched Volta tools
pub struct Session {
    hooks: LazyHookConfig,
    toolchain: LazyToolchain,
    project: LazyProject,
    event_log: Log,
    /// Cached result of checking `VOLTA_FEATURE_PNPM` environment variable
    pnpm_enabled: bool,
}

impl Session {
    /// Constructs a new `Session`.
    #[must_use]
    pub fn init() -> Self {
        Self {
            hooks: LazyHookConfig::init(),
            toolchain: LazyToolchain::init(),
            project: LazyProject::init(),
            event_log: Log::init(),
            pnpm_enabled: env::var_os(VOLTA_FEATURE_PNPM).is_some(),
        }
    }

    /// Returns whether the pnpm feature flag is enabled.
    #[must_use]
    pub const fn pnpm_enabled(&self) -> bool {
        self.pnpm_enabled
    }

    /// Produces a reference to the current Node project, if any.
    ///
    /// # Errors
    ///
    /// Returns an error if the project cannot be loaded.
    pub fn project(&self) -> Fallible<Option<&Project>> {
        self.project.get()
    }

    /// Produces a mutable reference to the current Node project, if any.
    ///
    /// # Errors
    ///
    /// Returns an error if the project cannot be loaded.
    pub fn project_mut(&mut self) -> Fallible<Option<&mut Project>> {
        self.project.get_mut()
    }

    /// Returns the user's default platform, if any
    ///
    /// # Errors
    ///
    /// Returns an error if the toolchain cannot be loaded.
    pub fn default_platform(&self) -> Fallible<Option<&PlatformSpec>> {
        self.toolchain.get().map(Toolchain::platform)
    }

    /// Returns the current project's pinned platform image, if any.
    ///
    /// # Errors
    ///
    /// Returns an error if the project cannot be loaded.
    pub fn project_platform(&self) -> Fallible<Option<&PlatformSpec>> {
        if let Some(project) = self.project()? {
            return Ok(project.platform());
        }
        Ok(None)
    }

    /// Produces a reference to the current toolchain (default platform specification)
    ///
    /// # Errors
    ///
    /// Returns an error if the toolchain cannot be loaded.
    pub fn toolchain(&self) -> Fallible<&Toolchain> {
        self.toolchain.get()
    }

    /// Produces a mutable reference to the current toolchain
    ///
    /// # Errors
    ///
    /// Returns an error if the toolchain cannot be loaded.
    pub fn toolchain_mut(&mut self) -> Fallible<&mut Toolchain> {
        self.toolchain.get_mut()
    }

    /// Produces a reference to the hook configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the hooks cannot be loaded.
    pub fn hooks(&self) -> Fallible<&HookConfig> {
        self.hooks.get(self.project()?)
    }

    pub fn add_event_start(&mut self, activity_kind: ActivityKind) {
        self.event_log.add_event_start(activity_kind);
    }
    pub fn add_event_end(&mut self, activity_kind: ActivityKind, exit_code: ExitCode) {
        self.event_log.add_event_end(activity_kind, exit_code);
    }
    pub fn add_event_tool_end(&mut self, activity_kind: ActivityKind, exit_code: i32) {
        self.event_log.add_event_tool_end(activity_kind, exit_code);
    }
    pub fn add_event_error(&mut self, activity_kind: ActivityKind, error: &VoltaError) {
        self.event_log.add_event_error(activity_kind, error);
    }

    fn publish_to_event_log(self) {
        let Self {
            project,
            hooks,
            mut event_log,
            ..
        } = self;
        let plugin_res = project
            .get()
            .and_then(|p| hooks.get(p))
            .map(|hooks| hooks.events().and_then(|e| e.publish.as_ref()));
        match plugin_res {
            Ok(plugin) => {
                event_log.add_event_args();
                event_log.publish(plugin);
            }
            Err(e) => {
                debug!("Unable to publish event log.\n{e}");
            }
        }
    }

    pub fn exit(self, code: ExitCode) -> ! {
        self.publish_to_event_log();
        code.exit();
    }

    pub fn exit_tool(self, code: i32) -> ! {
        self.publish_to_event_log();
        exit(code);
    }
}

#[cfg(test)]
pub mod tests {

    use crate::session::Session;
    use std::env;
    use std::path::PathBuf;

    fn fixture_path(fixture_dir: &str) -> PathBuf {
        let mut cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_manifest_dir.push("fixtures");
        cargo_manifest_dir.push(fixture_dir);
        cargo_manifest_dir
    }

    #[test]
    fn test_in_pinned_project() {
        let project_pinned = fixture_path("basic");
        env::set_current_dir(project_pinned).expect("Could not set current directory");
        let pinned_session = Session::init();
        let pinned_platform = pinned_session
            .project_platform()
            .expect("Couldn't create Project");
        assert!(pinned_platform.is_some());

        let project_unpinned = fixture_path("no_toolchain");
        env::set_current_dir(project_unpinned).expect("Could not set current directory");
        let unpinned_session = Session::init();
        let unpinned_platform = unpinned_session
            .project_platform()
            .expect("Couldn't create Project");
        assert!(unpinned_platform.is_none());
    }
}
