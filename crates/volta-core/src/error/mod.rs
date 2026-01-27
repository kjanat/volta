use std::error::Error;
use std::fmt;
use std::process::exit;

mod binary;
mod command;
mod environment;
mod filesystem;
mod hook;
mod kind;
mod network;
mod package;
mod platform;
mod reporter;
mod shim;
mod tool;
mod version;

#[allow(clippy::module_name_repetitions)]
pub use binary::BinaryError;
#[allow(clippy::module_name_repetitions)]
pub use filesystem::FilesystemError;
#[allow(clippy::module_name_repetitions)]
pub use kind::ErrorKind;
#[allow(clippy::module_name_repetitions)]
pub use reporter::report_error;
#[allow(clippy::module_name_repetitions)]
pub use shim::ShimError;

pub type Fallible<T> = Result<T, VoltaError>;

/// Error type for Volta
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct VoltaError {
    inner: Box<Inner>,
}

#[derive(Debug)]
struct Inner {
    kind: ErrorKind,
    source: Option<Box<dyn Error>>,
}

impl VoltaError {
    /// The exit code Volta should use when this error stops execution
    #[must_use]
    pub fn exit_code(&self) -> ExitCode {
        self.inner.kind.exit_code()
    }

    /// Create a new `VoltaError` instance including a source error
    pub fn from_source<E>(source: E, kind: ErrorKind) -> Self
    where
        E: Into<Box<dyn Error>>,
    {
        Self {
            inner: Box::new(Inner {
                kind,
                source: Some(source.into()),
            }),
        }
    }

    /// Get a reference to the `ErrorKind` for this error
    #[must_use]
    pub fn kind(&self) -> &ErrorKind {
        &self.inner.kind
    }
}

impl fmt::Display for VoltaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.kind.fmt(f)
    }
}

impl Error for VoltaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.inner.source.as_ref().map(std::convert::AsRef::as_ref)
    }
}

impl From<ErrorKind> for VoltaError {
    fn from(kind: ErrorKind) -> Self {
        Self {
            inner: Box::new(Inner { kind, source: None }),
        }
    }
}

/// Trait providing the `with_context` method to easily convert any Result error into a `VoltaError`
pub trait Context<T> {
    /// # Errors
    ///
    /// Returns a `VoltaError` if the underlying result is an error.
    fn with_context<F>(self, f: F) -> Fallible<T>
    where
        F: FnOnce() -> ErrorKind;
}

impl<T, E> Context<T> for Result<T, E>
where
    E: Error + 'static,
{
    fn with_context<F>(self, f: F) -> Fallible<T>
    where
        F: FnOnce() -> ErrorKind,
    {
        self.map_err(|e| VoltaError::from_source(e, f()))
    }
}

/// Exit codes supported by Volta Errors
#[derive(Copy, Clone, Debug)]
pub enum ExitCode {
    /// No error occurred.
    Success = 0,

    /// An unknown error occurred.
    UnknownError = 1,

    /// An invalid combination of command-line arguments was supplied.
    InvalidArguments = 3,

    /// No match could be found for the requested version string.
    NoVersionMatch = 4,

    /// A network error occurred.
    NetworkError = 5,

    /// A required environment variable was unset or invalid.
    EnvironmentError = 6,

    /// A file could not be read or written.
    FileSystemError = 7,

    /// Package configuration is missing or incorrect.
    ConfigurationError = 8,

    /// The command or feature is not yet implemented.
    NotYetImplemented = 9,

    /// The requested executable could not be run.
    ExecutionFailure = 126,

    /// The requested executable is not available.
    ExecutableNotFound = 127,
}

impl ExitCode {
    pub fn exit(self) -> ! {
        exit(self as i32);
    }
}
