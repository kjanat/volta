use std::collections::HashMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::Path;
use std::process::{Command, ExitStatus, Output};
use std::str;

use thiserror::Error;

/// A builder object for an external process, similar to `std::process::Command`.
#[derive(Clone, Debug)]
pub struct Builder {
    /// The program to execute.
    program: OsString,
    /// A list of arguments to pass to the program.
    args: Vec<OsString>,
    /// Any environment variables that should be set for the program.
    env: HashMap<String, Option<OsString>>,
    /// Which directory to run the program from.
    cwd: Option<OsString>,
}

impl fmt::Display for Builder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}", self.program.to_string_lossy())?;

        for arg in &self.args {
            write!(f, " {}", arg.to_string_lossy())?;
        }

        write!(f, "`")
    }
}

impl Builder {
    /// (chainable) Set the executable for the process.
    pub fn program<T: AsRef<OsStr>>(&mut self, program: T) -> &mut Self {
        self.program = program.as_ref().to_os_string();
        self
    }

    /// (chainable) Add an arg to the args list.
    pub fn arg<T: AsRef<OsStr>>(&mut self, arg: T) -> &mut Self {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    /// (chainable) Add many args to the args list.
    pub fn args<T: AsRef<OsStr>>(&mut self, arguments: &[T]) -> &mut Self {
        self.args
            .extend(arguments.iter().map(|t| t.as_ref().to_os_string()));
        self
    }

    /// (chainable) Replace args with new args list
    pub fn args_replace<T: AsRef<OsStr>>(&mut self, arguments: &[T]) -> &mut Self {
        self.args = arguments
            .iter()
            .map(|t| t.as_ref().to_os_string())
            .collect();
        self
    }

    /// (chainable) Set the current working directory of the process
    pub fn cwd<T: AsRef<OsStr>>(&mut self, path: T) -> &mut Self {
        self.cwd = Some(path.as_ref().to_os_string());
        self
    }

    /// (chainable) Set an environment variable for the process.
    pub fn env<T: AsRef<OsStr>>(&mut self, key: &str, val: T) -> &mut Self {
        self.env
            .insert(key.to_string(), Some(val.as_ref().to_os_string()));
        self
    }

    /// (chainable) Unset an environment variable for the process.
    pub fn env_remove(&mut self, key: &str) -> &mut Self {
        self.env.insert(key.to_string(), None);
        self
    }

    /// Get the executable name.
    #[must_use]
    pub const fn get_program(&self) -> &OsString {
        &self.program
    }

    /// Get the program arguments
    #[must_use]
    pub fn get_args(&self) -> &[OsString] {
        &self.args
    }

    /// Get the current working directory for the process
    pub fn get_cwd(&self) -> Option<&Path> {
        self.cwd.as_ref().map(Path::new)
    }

    /// Get an environment variable as the process will see it (will inherit from environment
    /// unless explicitally unset).
    #[must_use]
    pub fn get_env(&self, var: &str) -> Option<OsString> {
        self.env
            .get(var)
            .cloned()
            .or_else(|| Some(env::var_os(var)))
            .and_then(|s| s)
    }

    /// Get all environment variables explicitly set or unset for the process (not inherited
    /// vars).
    #[must_use]
    pub const fn get_envs(&self) -> &HashMap<String, Option<OsString>> {
        &self.env
    }

    /// Run the process, waiting for completion, and mapping non-success exit codes to an error.
    ///
    /// # Errors
    ///
    /// Returns an error if the process fails to execute or returns a non-zero exit code.
    pub fn exec(&self) -> Result<(), Error> {
        let mut command = self.build_command();

        let Ok(exit) = command.status() else {
            return Err(error(
                &format!("could not execute process {self}"),
                None,
                None,
            ));
        };

        if exit.success() {
            Ok(())
        } else {
            Err(error(
                &format!("process didn't exit successfully: {self}"),
                Some(exit),
                None,
            ))
        }
    }

    /// Execute the process, returning the stdio output, or an error if non-zero exit status.
    ///
    /// # Errors
    ///
    /// Returns an error if the process fails to execute or returns a non-zero exit code.
    pub fn exec_with_output(&self) -> Result<Output, Error> {
        let mut command = self.build_command();

        let Ok(output) = command.output() else {
            return Err(error(
                &format!("could not execute process {self}"),
                None,
                None,
            ));
        };

        if output.status.success() {
            Ok(output)
        } else {
            Err(error(
                &format!("process didn't exit successfully: {self}"),
                Some(output.status),
                Some(&output),
            ))
        }
    }

    /// Converts `ProcessBuilder` into a `std::process::Command`
    #[must_use]
    pub fn build_command(&self) -> Command {
        let mut command = Command::new(&self.program);
        if let Some(cwd) = self.get_cwd() {
            command.current_dir(cwd);
        }
        for arg in &self.args {
            command.arg(arg);
        }
        for (k, v) in &self.env {
            match *v {
                Some(ref v) => {
                    command.env(k, v);
                }
                None => {
                    command.env_remove(k);
                }
            }
        }
        command
    }
}

/// A helper function to create a `Builder`.
pub fn process<T: AsRef<OsStr>>(cmd: T) -> Builder {
    Builder {
        program: cmd.as_ref().to_os_string(),
        args: Vec::new(),
        cwd: None,
        env: HashMap::new(),
    }
}

/// Error type for process execution failures.
#[derive(Debug, Error)]
#[error("{desc}")]
pub struct Error {
    /// Description of the error.
    pub desc: String,
    /// Exit status if the process ran.
    pub exit: Option<ExitStatus>,
    /// Captured output if available.
    pub output: Option<Output>,
}

fn status_to_string(status: ExitStatus) -> String {
    status.to_string()
}

/// Creates an error for process execution failures.
#[must_use]
pub fn error(msg: &str, status: Option<ExitStatus>, output: Option<&Output>) -> Error {
    let exit = status.map_or_else(|| "never executed".to_string(), status_to_string);
    let mut desc = format!("{} ({})", &msg, exit);

    if let Some(out) = output {
        match str::from_utf8(&out.stdout) {
            Ok(s) if !s.trim().is_empty() => {
                desc.push_str("\n--- stdout\n");
                desc.push_str(s);
            }
            Ok(_) | Err(_) => {}
        }
        match str::from_utf8(&out.stderr) {
            Ok(s) if !s.trim().is_empty() => {
                desc.push_str("\n--- stderr\n");
                desc.push_str(s);
            }
            Ok(_) | Err(_) => {}
        }
    }

    Error {
        desc,
        exit: status,
        output: output.cloned(),
    }
}
