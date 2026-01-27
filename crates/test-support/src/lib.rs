//! Utilities to use with acceptance tests in Volta.

#[macro_export]
macro_rules! ok_or_panic {
    { $e:expr } => {
        match $e {
            Ok(x) => x,
            Err(err) => panic!("{} failed with {}", stringify!($e), err),
        }
    };
}

pub mod matchers;
pub mod paths;
pub mod process;

/// Re-export `process::error` as `process_error` for backwards compatibility.
pub use process::error as process_error;
/// Re-export `process::Builder` as `ProcessBuilder` for backwards compatibility.
pub use process::Builder as ProcessBuilder;
/// Re-export `process::Error` as `ProcessError` for backwards compatibility.
pub use process::Error as ProcessError;
