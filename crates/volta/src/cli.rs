use clap::{Parser, builder::styling};

use crate::command::{self, Command};
use volta_core::error::{ExitCode, Fallible};
use volta_core::session::Session;
use volta_core::style::{MAX_WIDTH, text_width};

#[derive(Parser)]
#[command(
    name = "volta",
    about = "The JavaScript Launcher ⚡",
    long_about = "The JavaScript Launcher ⚡

    To install a tool in your toolchain, use `volta install`.
    To pin your project's runtime or package manager, use `volta pin`.",
    version,
    disable_version_flag = true,
    styles = styles(),
    term_width = text_width().unwrap_or(MAX_WIDTH),
)]
#[allow(clippy::struct_excessive_bools)] // CLI flags are naturally bools
pub struct Volta {
    #[command(subcommand)]
    pub(crate) command: Option<Subcommand>,

    /// Enables verbose diagnostics
    #[arg(long, global = true)]
    pub(crate) verbose: bool,

    /// Enables trace-level diagnostics.
    #[arg(long, global = true, requires = "verbose")]
    pub(crate) very_verbose: bool,

    /// Prevents unnecessary output
    #[arg(
        long,
        global = true,
        conflicts_with = "verbose",
        aliases = &["silent"]
    )]
    pub(crate) quiet: bool,

    /// Prints the current version of Volta
    #[arg(short, long)]
    pub(crate) version: bool,
}

impl Volta {
    pub(crate) fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        if self.version {
            // suffix indicator for dev build
            if cfg!(debug_assertions) {
                println!("{}-dev", env!("CARGO_PKG_VERSION"));
            } else {
                println!("{}", env!("CARGO_PKG_VERSION"));
            }
            Ok(ExitCode::Success)
        } else if let Some(command) = self.command {
            command.run(session)
        } else {
            Self::parse_from(["volta", "help"].iter()).run(session)
        }
    }
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// Fetches a tool to the local machine
    Fetch(command::Fetch),

    /// Installs a tool in your toolchain
    Install(command::Install),

    /// Uninstalls a tool from your toolchain
    Uninstall(command::Uninstall),

    /// Updates a tool in your toolchain to a newer version
    Update(command::Update),

    /// Pins your project's runtime or package manager
    Pin(command::Pin),

    /// Displays the current toolchain
    #[command(alias = "ls")]
    List(command::List),

    /// Generates Volta completions
    ///
    /// By default, completions will be generated for the value of your current shell,
    /// shell, i.e. the value of `SHELL`. If you set the `<shell>` option, completions
    /// will be generated for that shell instead.
    ///
    /// If you specify a directory, the completions will be written to a file there;
    /// otherwise, they will be written to `stdout`.
    #[command(arg_required_else_help = true)]
    Completions(command::Completions),

    /// Locates the actual binary that will be called by Volta
    Which(command::Which),

    #[command(long_about = crate::command::r#use::USAGE, hide = true)]
    Use(command::Use),

    /// Enables Volta for the current user / shell
    Setup(command::Setup),

    /// Run a command with custom Node, npm, pnpm, and/or Yarn versions
    Run(command::Run),
}

impl Subcommand {
    pub(crate) fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        match self {
            Self::Fetch(fetch) => fetch.run(session),
            Self::Install(install) => install.run(session),
            Self::Uninstall(uninstall) => uninstall.run(session),
            Self::Update(update) => update.run(session),
            Self::Pin(pin) => pin.run(session),
            Self::List(list) => list.run(session),
            Self::Completions(completions) => completions.run(session),
            Self::Which(which) => which.run(session),
            Self::Use(r#use) => r#use.run(session),
            Self::Setup(setup) => setup.run(session),
            Self::Run(run) => run.run(session),
        }
    }
}

fn styles() -> styling::Styles {
    styling::Styles::plain()
        .header(
            styling::AnsiColor::Yellow.on_default()
                | styling::Effects::BOLD
                | styling::Effects::ITALIC,
        )
        .usage(
            styling::AnsiColor::Yellow.on_default()
                | styling::Effects::BOLD
                | styling::Effects::ITALIC,
        )
        .literal(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
        .placeholder(styling::AnsiColor::BrightBlue.on_default())
}
