use std::env::{args_os, var_os};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use super::VoltaError;
use crate::fs::ensure_containing_dir_exists;
use crate::layout::volta_home;
use crate::style::format_error_cause;
use chrono::Local;
use console::strip_ansi_codes;
use log::{debug, error};

/// Returns true if running in a CI environment.
///
/// Detects CI by checking for the `CI` environment variable, which is set by
/// most CI providers (GitHub Actions, GitLab CI, `CircleCI`, Travis, etc.).
fn is_ci() -> bool {
    var_os("CI").is_some()
}

/// Report an error, both to the console and to error logs
pub fn report_error(volta_version: &str, err: &VoltaError) {
    let message = err.to_string();
    error!("{message}");

    if let Some(details) = compose_error_details(err) {
        if is_ci() {
            // In CI, we write the error details to the log so that they are available in the CI logs
            // A log file may not even exist by the time the user is reviewing a failure
            error!("{details}");
        } else {
            // Outside of CI, we write the error details as Debug (Verbose) information
            // And we write an actual error log that the user can review
            debug!("{details}");

            // Note: Writing the error log info directly to stderr as it is a message for the user
            // Any custom logs will have all of the details already, so showing a message about writing
            // the error log would be redundant
            match write_error_log(volta_version, &message, &details) {
                Ok(log_file) => {
                    eprintln!("Error details written to {}", log_file.to_string_lossy());
                }
                Err(_) => {
                    eprintln!("Unable to write error log!");
                }
            }
        }
    }
}

/// Write an error log with all details about the error
fn write_error_log(
    volta_version: &str,
    message: &str,
    details: &str,
) -> Result<PathBuf, Box<dyn Error>> {
    let file_name = Local::now()
        .format("volta-error-%Y-%m-%d_%H_%M_%S%.3f.log")
        .to_string();
    let log_file_path = volta_home()?.log_dir().join(file_name);

    ensure_containing_dir_exists(&log_file_path)?;
    let mut log_file = File::create(&log_file_path)?;

    writeln!(log_file, "{}", collect_arguments())?;
    writeln!(log_file, "Volta v{volta_version}")?;
    writeln!(log_file)?;
    writeln!(log_file, "{}", strip_ansi_codes(message))?;
    writeln!(log_file)?;
    writeln!(log_file, "{}", strip_ansi_codes(details))?;

    Ok(log_file_path)
}

fn compose_error_details(err: &VoltaError) -> Option<String> {
    // Only compose details if there is an underlying cause for the error
    let mut current = err.source()?;
    let mut details = String::new();

    // Walk up the tree of causes and include all of them
    loop {
        details.push_str(&format_error_cause(current));

        match current.source() {
            Some(cause) => {
                details.push_str("\n\n");
                current = cause;
            }
            None => {
                break;
            }
        }
    }

    Some(details)
}

/// Combines all the arguments into a single String
fn collect_arguments() -> String {
    // Quote arguments that contain spaces or special characters
    args_os()
        .map(|arg| {
            let s = arg.to_string_lossy();
            if s.contains(' ') || s.contains('"') || s.contains('\'') {
                format!("\"{}\"", s.replace('"', "\\\""))
            } else {
                s.into_owned()
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}
