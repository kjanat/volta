//! Error types for network operations.
//!
//! This module contains errors related to:
//! - HTTP requests and responses
//! - Download failures
//! - Registry communication
//! - Network connectivity issues

use std::fmt;

use super::ExitCode;
use crate::tool::ToolSpec;

/// Errors related to network operations.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum NetworkError {
    /// Thrown when a network error occurs while downloading a tool.
    DownloadTool { tool: ToolSpec, from_url: String },

    /// Thrown when the public registry for Node or Yarn could not be downloaded.
    RegistryFetch { tool: String, from_url: String },

    /// Thrown when there is an error fetching the latest version of Yarn.
    YarnLatestFetch { from_url: String },

    /// Thrown when unable to parse the node index.
    ParseNodeIndex { from_url: String },
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DownloadTool { tool, from_url } => write!(
                f,
                "Could not download {tool}
from {from_url}

Please verify your internet connection and ensure the correct version is specified."
            ),
            Self::RegistryFetch { tool, from_url } => write!(
                f,
                "Could not download {tool} version registry
from {from_url}

Please verify your internet connection."
            ),
            Self::YarnLatestFetch { from_url } => write!(
                f,
                "Could not fetch latest version of Yarn
from {from_url}

Please verify your internet connection."
            ),
            Self::ParseNodeIndex { from_url } => write!(
                f,
                "Could not parse Node version index
from {from_url}

Please verify your internet connection."
            ),
        }
    }
}

impl NetworkError {
    /// Returns the exit code for this error.
    #[must_use]
    pub const fn exit_code(&self) -> ExitCode {
        ExitCode::NetworkError
    }
}
