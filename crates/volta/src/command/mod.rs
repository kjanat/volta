pub mod completions;
pub mod fetch;
pub mod install;
pub mod list;
pub mod pin;
pub mod run;
pub mod setup;
pub mod uninstall;
pub mod r#use;
pub mod which;

pub use self::which::Which;
pub use completions::Completions;
pub use fetch::Fetch;
pub use install::Install;
pub use list::List;
pub use pin::Pin;
pub use r#use::Use;
pub use run::Run;
pub use setup::Setup;
pub use uninstall::Uninstall;

use volta_core::error::{ExitCode, Fallible};
use volta_core::session::Session;

/// A Volta command.
pub trait Command: Sized {
    /// Executes the command. Returns `Ok(true)` if the process should return 0,
    /// `Ok(false)` if the process should return 1, and `Err(e)` if the process
    /// should return `e.exit_code()`.
    fn run(self, session: &mut Session) -> Fallible<ExitCode>;
}
