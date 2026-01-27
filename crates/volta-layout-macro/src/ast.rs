use crate::ir::{Entry, Ir};
use proc_macro2::TokenStream;
use std::collections::HashMap;
use syn::parse::{self, Parse, ParseStream};
use syn::{braced, Attribute, Ident, LitStr, Token, Visibility};

pub type Result<T> = ::std::result::Result<T, TokenStream>;

/// Check if filename has .exe extension (case-insensitive for DSL validation).
fn has_exe_extension(filename: &str) -> bool {
    filename
        .rsplit_once('.')
        .is_some_and(|(_, ext)| ext.eq_ignore_ascii_case("exe"))
}

/// Check if filename has [.exe] suffix (conditional exe marker).
fn has_conditional_exe_suffix(filename: &str) -> bool {
    filename.ends_with("[.exe]")
}

/// Abstract syntax tree (AST) for the surface syntax of the `layout!` macro.
///
/// The surface syntax of the `layout!` macro takes the form:
///
/// ```text,no_run
/// Attribute* Visibility "struct" Ident Directory
/// ```
///
/// This AST gets lowered by the `flatten` method to a vector of intermediate
/// representation (IR) trees. See the `Ir` type for details.
pub struct Ast {
    decls: Vec<LayoutStruct>,
}

impl Parse for Ast {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let mut decls = Vec::new();
        while !input.is_empty() {
            let decl = input.call(LayoutStruct::parse)?;
            decls.push(decl);
        }
        Ok(Self { decls })
    }
}

impl Ast {
    /// Compiles (macro-expands) the AST.
    pub(crate) fn compile(self) -> TokenStream {
        self.decls
            .into_iter()
            .map(|decl| match decl.flatten() {
                Ok(ir) => ir.codegen(),
                Err(err) => err,
            })
            .collect()
    }
}

/// Represents a single type `LayoutStruct` in the AST, which takes the form:
///
/// ```text,no_run
/// Attribute* Visibility "struct" Ident Directory
/// ```
///
/// This AST gets lowered by the `flatten` method to a flat list of entries,
/// organized by entry type. See the `Ir` type for details.
pub struct LayoutStruct {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    name: Ident,
    directory: Directory,
}

impl Parse for LayoutStruct {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;
        let visibility: Visibility = input.parse()?;
        input.parse::<Token![struct]>()?;
        let name: Ident = input.parse()?;
        let directory: Directory = input.parse()?;
        Ok(Self {
            attrs,
            visibility,
            name,
            directory,
        })
    }
}

impl LayoutStruct {
    /// Lowers the AST to a flattened intermediate representation.
    fn flatten(self) -> Result<Ir> {
        let mut results = Ir {
            name: self.name,
            attrs: self.attrs,
            visibility: self.visibility,
            dirs: vec![],
            files: vec![],
            exes: vec![],
        };

        self.directory.flatten(&mut results, &[])?;

        Ok(results)
    }
}

/// Represents a directory entry in the AST, which can recursively contain
/// more entries.
///
/// The surface syntax of a directory takes the form:
///
/// ```text,no_run
/// {
///     (FieldPrefix)FieldContents*
/// }
/// ```
struct Directory {
    entries: Vec<(FieldPrefix, FieldContents)>,
}

impl Parse for Directory {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let content;
        braced!(content in input);
        let mut entries = Vec::new();
        while !content.is_empty() {
            let prefix: FieldPrefix = content.parse()?;
            let contents: FieldContents = content.parse()?;
            entries.push((prefix, contents));
        }
        Ok(Self { entries })
    }
}

enum EntryKind {
    Exe,
    File,
    Dir,
}

impl Directory {
    /// Lowers the directory to a flattened intermediate representation.
    fn flatten(self, results: &mut Ir, context: &[LitStr]) -> Result<()> {
        let mut visited_entries = HashMap::new();

        for (prefix, contents) in self.entries {
            let mut entry = Entry {
                name: prefix.name,
                context: context.to_owned(),
                filename: prefix.filename.clone(),
            };

            let filename = prefix.filename.value();

            match contents {
                FieldContents::Dir(dir) => {
                    if has_exe_extension(&filename) || has_conditional_exe_suffix(&filename) {
                        let error = syn::Error::new(
                            prefix.filename.span(),
                            "the `.exe` extension is not allowed for directory names",
                        );
                        return Err(error.to_compile_error());
                    }

                    if let Some(kind) = visited_entries.get(&filename) {
                        let message = match kind {
                            EntryKind::Exe => {
                                format!("filename `{filename}` is a duplicate of `{filename}` executable on non-Windows operating systems")
                            }
                            EntryKind::File | EntryKind::Dir => {
                                format!("duplicate filename `{filename}`")
                            }
                        };
                        let error = syn::Error::new(prefix.filename.span(), message);
                        return Err(error.to_compile_error());
                    }

                    visited_entries.insert(filename.clone(), EntryKind::Dir);

                    results.dirs.push(entry);
                    let mut sub_context = context.to_owned();
                    sub_context.push(prefix.filename);
                    dir.flatten(results, &sub_context)?;
                }
                FieldContents::File(()) => {
                    if has_conditional_exe_suffix(&filename) {
                        let basename = &filename[0..filename.len() - 6];

                        if let Some(kind) = visited_entries.get(basename) {
                            let message = match kind {
                                EntryKind::Exe => {
                                    format!("duplicate filename `{basename}.exe`")
                                }
                                EntryKind::File => {
                                    format!("executable `{basename}` (on non-Windows operating systems) is a duplicate of `{basename}` filename")
                                }
                                EntryKind::Dir => {
                                    format!("executable `{basename}` (on non-Windows operating systems) is a duplicate of `{basename}` directory name")
                                }
                            };
                            let error = syn::Error::new(prefix.filename.span(), message);
                            return Err(error.to_compile_error());
                        }

                        visited_entries.insert(basename.to_string(), EntryKind::Exe);
                        entry.filename = LitStr::new(basename, prefix.filename.span());
                        results.exes.push(entry);
                    } else {
                        if let Some(kind) = visited_entries.get(&filename) {
                            let message = match kind {
                                EntryKind::Exe => {
                                    format!("filename `{filename}` is a duplicate of `{filename}` executable on non-Windows operating systems")
                                }
                                EntryKind::File | EntryKind::Dir => {
                                    format!("duplicate filename `{filename}`")
                                }
                            };
                            let error = syn::Error::new(prefix.filename.span(), message);
                            return Err(error.to_compile_error());
                        }

                        visited_entries.insert(filename, EntryKind::File);
                        results.files.push(entry);
                    }
                }
            }
        }
        Ok(())
    }
}

/// AST for the common prefix of a single field in a `layout!` struct declaration,
/// which is of the form:
///
/// ```text,no_run
/// LitStr ":" Ident
/// ```
///
/// This is followed either by a semicolon (`;`), indicating that the field is a
/// file, or a braced directory entry, indicating that the field is a directory.
///
/// If the `LitStr` contains the suffix `"[.exe]"` it is treated specially as an
/// executable file, whose suffix (or lack thereof) is determined by the current
/// operating system (using the `std::env::consts::EXE_SUFFIX` constant).
struct FieldPrefix {
    filename: LitStr,
    name: Ident,
}

impl Parse for FieldPrefix {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let filename = input.parse()?;
        input.parse::<Token![:]>()?;
        let name = input.parse()?;
        Ok(Self { filename, name })
    }
}

/// AST for the suffix of a field in a `layout!` struct declaration.
enum FieldContents {
    /// A file field suffix, which consists of a single semicolon (`;`).
    File(()),

    /// A directory field suffix, which consists of a braced directory.
    Dir(Directory),
}

impl Parse for FieldContents {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let lookahead = input.lookahead1();
        Ok(if lookahead.peek(Token![;]) {
            let _semi: Token![;] = input.parse()?;
            Self::File(())
        } else {
            let directory = input.parse()?;
            Self::Dir(directory)
        })
    }
}
