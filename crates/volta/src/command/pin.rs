use volta_core::error::{ExitCode, Fallible};
use volta_core::session::{ActivityKind, Session};
use volta_core::tool::ToolSpec;

use crate::command::Command;

#[derive(clap::Args)]
pub struct Pin {
    /// Tools to pin, like `node@lts` or `yarn@^1.14`.
    #[arg(value_name = "tool[@version]", required = true)]
    tools: Vec<String>,
}

impl Command for Pin {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Pin);

        for tool in ToolSpec::from_strings(&self.tools, "pin")? {
            tool.resolve(session)?.pin(session)?;
        }

        session.add_event_end(ActivityKind::Pin, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
