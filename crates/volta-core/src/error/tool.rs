//! Error types for tool operations.
//!
//! This module contains errors related to:
//! - Tool fetching and installation
//! - Tool version management
//! - Node, npm, pnpm, and Yarn specific errors

#![allow(unused_imports, dead_code)]

use std::path::PathBuf;

/// Errors related to tool operations.
#[derive(Debug)]
pub enum ToolError {
    // Variants will be moved here from ErrorKind
}
