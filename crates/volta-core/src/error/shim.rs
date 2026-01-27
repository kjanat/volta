//! Error types for shim operations.
//!
//! This module contains errors related to:
//! - Shim creation and deletion
//! - Shim symlink operations
//! - Shim directory management

#![allow(unused_imports, dead_code)]

use std::path::PathBuf;

/// Errors related to shim operations.
#[derive(Debug)]
pub enum ShimError {
    // Variants will be moved here from ErrorKind
}
