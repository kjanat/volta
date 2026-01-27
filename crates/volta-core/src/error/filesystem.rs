//! Error types for filesystem operations.
//!
//! This module contains errors related to:
//! - File reading and writing
//! - Directory creation and deletion
//! - Path operations and resolution
//! - Symlink operations

#![allow(unused_imports, dead_code)]

use std::path::PathBuf;

/// Errors related to filesystem operations.
#[derive(Debug)]
pub enum FilesystemError {
    // Variants will be moved here from ErrorKind
}
