# Volta Agent Guidelines

Volta is a JavaScript toolchain manager written in Rust. It manages Node.js, npm, pnpm, and Yarn versions.

## Project Structure

```
crates/
  volta/           # Main CLI binary (commands: fetch, install, list, pin, run, setup, uninstall, use, which)
  volta-core/      # Core library (error, platform, tools, session, hooks, etc.)
  volta-migrate/   # Migration between Volta versions
  volta-layout/    # Filesystem layout definitions
  volta-layout-macro/ # Proc macros for layout
  archive/         # Archive extraction (tar, zip)
  test-support/    # Test utilities, matchers, and process helpers
  fs-utils/        # Filesystem utilities
  progress-read/   # Progress bar for reads
  validate-npm-package-name/  # NPM package name validation
```

## Build Commands

```bash
cargo check --workspace          # Fast compilation check
cargo build --workspace          # Debug build
cargo build --release --workspace # Release build (LTO enabled)
cargo clippy --workspace         # Required before commits
cargo fmt --all                  # Format code
cargo fmt --all -- --check       # Check formatting
```

## Test Commands

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p volta-core

# Run single test by exact name (nested modules use :: separator)
cargo test -p volta command::list::human::tests::active::no_runtimes

# Run tests matching pattern
cargo test -p volta-core version

# List available tests
cargo test --workspace -- --list

# Run ignored tests
cargo test --workspace -- --ignored
```

## Code Style

### Imports

Group imports in order, separated by blank lines:

```rust
use std::path::PathBuf;

use clap::Parser;
use log::debug;

use volta_core::error::Fallible;

use super::Command;
use crate::cli::Volta;
```

### Error Handling

- Use `Fallible<T>` (alias for `Result<T, VoltaError>`) for fallible operations
- Define error variants in `volta_core::error::ErrorKind`
- Use `.with_context()` from `Context` trait to wrap errors
- Document errors with `# Errors` section

```rust
use volta_core::error::{Context, ErrorKind, Fallible};

fn read_config(path: &Path) -> Fallible<Config> {
    std::fs::read_to_string(path)
        .with_context(|| ErrorKind::ReadFileError { file: path.to_owned() })
}
```

### Naming Conventions

- `snake_case` for functions, variables, modules
- `PascalCase` for types, traits, enums
- Prefix lazy types with `Lazy` (e.g., `LazyProject`)
- Use `Sourced<T>` for values with origin tracking

### Documentation

- Doc comments (`///`) on all public items
- First paragraph: single concise sentence
- Add `# Errors` for fallible functions
- Add `# Panics` if function can panic

### Functions

- Prefer `&str` over `String`, `&Path` over `PathBuf` for params
- Use `#[must_use]` on pure functions returning values
- Keep functions under 100 lines

### Match & Options

- Merge identical arms with `|`
- Use `map_or`, `map_or_else`, `is_some_and` over `if let`
- Prefer `let else` for early returns

## Clippy Configuration

Strict lints enforced (see `Cargo.toml`):

```toml
unsafe_code = "deny"
pedantic = "warn"
nursery = "warn"
module_name_repetitions = "deny"
needless_pass_by_value = "deny"
option_if_let_else = "deny"
manual_let_else = "deny"
doc_markdown = "deny"
missing_panics_doc = "deny"
```

Add targeted `#[allow]` when legitimately needed:

```rust
#[allow(clippy::module_name_repetitions)]
pub struct ErrorKind { ... }
```

## Type Patterns

### Version Types

- `VersionSpec` - User input: `None`, `Semver(Range)`, `Exact(Version)`, `Tag(Tag)`
- `Tag` - Version tag: `Latest`, `Lts`, `Custom(String)`
- `Version` - Resolved semver (from `nodejs_semver`)

### Platform Types

- `Image` - Runtime image with node/npm/pnpm/yarn versions
- `Sourced<T>` - Value with source tracking (default/project)
- `Platform` - Resolved platform with sourced versions

### Tool Types

- `Tool` trait - Interface with `fetch`, `install`, `pin` methods
- `Node`, `Npm`, `Pnpm`, `Yarn` - Tool implementations
- `Package` - Third-party npm packages

## Testing

Tests use `#[cfg(test)]` modules, often with nested organization:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod parsing {
        use super::*;

        #[test]
        fn parses_valid_input() {
            assert_eq!(parse("1.2.3").unwrap(), expected);
        }
    }
}
```

Use `test-support` crate utilities:

```rust
use test_support::{ok_or_panic, matchers, ProcessBuilder};
```

Platform-specific tests use `#[cfg(unix)]` / `#[cfg(windows)]`.

## Commit Guidelines

1. `cargo fmt --all`
2. `cargo clippy --workspace` - fix all errors
3. `cargo test --workspace` - ensure tests pass
4. Keep commits focused and atomic
