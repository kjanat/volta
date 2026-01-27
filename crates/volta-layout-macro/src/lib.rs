//! Procedural macro for defining Volta directory layout hierarchies.
//!
//! This crate provides the [`layout!`] macro, which generates type-safe structs
//! representing filesystem directory trees. Each struct field corresponds to a
//! path within the hierarchy, with automatic path construction from a root directory.
//!
//! # Generated Code
//!
//! For each struct defined in `layout!`, the macro generates:
//!
//! - A struct with `PathBuf` fields for each entry (files and directories)
//! - A `new(root: PathBuf) -> Self` constructor
//! - Accessor methods returning `&Path` for each field
//! - A `root() -> &Path` method returning the root directory
//! - A `create() -> io::Result<()>` method that creates all subdirectories
//!
//! # DSL Syntax
//!
//! ```text
//! layout! {
//!     [attributes]
//!     [visibility] struct StructName {
//!         "filename": field_name;              // File entry
//!         "dirname": field_name {}             // Empty directory
//!         "dirname": field_name {              // Directory with contents
//!             "nested": nested_field;          // Nested entries...
//!         }
//!         "name[.exe]": field_name;            // Executable (platform-aware)
//!     }
//! }
//! ```
//!
//! ## Entry Types
//!
//! - **Files**: Declared with `"filename": field_name;` (semicolon terminator)
//! - **Directories**: Declared with `"dirname": field_name { ... }` (braces, may be empty)
//! - **Executables**: Use `[.exe]` suffix (e.g., `"volta[.exe]"`) - expands to `.exe` on
//!   Windows, empty string on Unix via [`std::env::consts::EXE_SUFFIX`]
//!
//! # Example
//!
//! See [`volta_layout::v1`](../volta_layout/v1/index.html) for real-world usage defining
//! `VoltaHome` and `VoltaInstall` directory structures.

#![recursion_limit = "128"]

extern crate proc_macro;

mod ast;
mod ir;

use crate::ast::Ast;
use proc_macro::TokenStream;
use syn::parse_macro_input;

/// A macro for defining Volta directory layout hierarchies.
///
/// The syntax of `layout!` takes the form:
///
/// ```text,no_run
/// layout! {
///     LayoutStruct*
/// }
/// ```
///
/// The syntax of a `LayoutStruct` takes the form:
///
/// ```text,no_run
/// Attribute* Visibility "struct" Ident Directory
/// ```
///
/// The syntax of a `Directory` takes the form:
///
/// ```text,no_run
/// {
///     (FieldPrefix)FieldContents*
/// }
/// ```
///
/// The syntax of a `FieldPrefix` takes the form:
///
/// ```text,no_run
/// LitStr ":" Ident
/// ```
///
/// The syntax of a `FieldContents` is either:
///
/// ```text,no_run
/// ";"
/// ```
///
/// or:
///
/// ```text,no_run
/// Directory
/// ```
#[proc_macro]
pub fn layout(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as Ast);
    let expanded = ast.compile();
    TokenStream::from(expanded)
}
