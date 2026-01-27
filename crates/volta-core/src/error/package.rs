//! Error types for package operations.
//!
//! This module contains errors related to:
//! - Package manifest parsing (package.json)
//! - Package name validation
//! - Package installation and configuration
//! - Package registry operations

#![allow(unused_imports, dead_code)]

use std::path::PathBuf;

/// Errors related to package operations.
#[derive(Debug)]
pub enum PackageError {
    // Variants will be moved here from ErrorKind
}
