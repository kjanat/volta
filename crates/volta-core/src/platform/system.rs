use std::ffi::OsString;

use super::build_path_error;
use crate::error::{Context, Fallible};
use crate::layout::env_paths;

/// A lightweight namespace type representing the system environment, i.e. the environment
/// with Volta removed.
pub struct System;

impl System {
    /// Produces a modified version of the current `PATH` environment variable that
    /// removes the Volta shims and binaries, to use for running system node and
    /// executables.
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be computed.
    pub fn path() -> Fallible<OsString> {
        let old_path = envoy::path().unwrap_or_else(|| envoy::Var::from(""));
        Self::path_with_base(&old_path)
    }

    /// Internal helper for path computation, used by tests to avoid mutating environment
    fn path_with_base(old_path: &envoy::Var) -> Fallible<OsString> {
        let mut new_path = old_path.split();

        for remove_path in env_paths()? {
            new_path = new_path.remove(remove_path);
        }

        new_path.join().with_context(build_path_error)
    }

    #[cfg(test)]
    pub fn path_from(base_path: &str) -> Fallible<OsString> {
        Self::path_with_base(&envoy::Var::from(base_path))
    }
}
