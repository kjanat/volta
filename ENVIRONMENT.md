# Environment Variables

Volta supports several environment variables that can be used to customize its behavior. This document provides a comprehensive reference for all user-facing environment variables.

## Shim Behavior

### `VOLTA_BYPASS`

Temporarily disables Volta's shim behavior and runs commands directly from the system PATH.

**Use cases:**
- Working on a tool's codebase where the compiler needs to resolve the actual binary path (e.g., building bun, node, or other tools managed by Volta)
- Debugging issues where you need to verify behavior without Volta's interception
- Running a system-installed version of a tool instead of the Volta-managed version

**Example:**
```bash
# Run a single command bypassing Volta
VOLTA_BYPASS=1 node --version

# Or export for multiple commands
export VOLTA_BYPASS=1
node --version
npm --version
unset VOLTA_BYPASS
```

When enabled, Volta removes its directories from `PATH` and executes commands using your system's native tools. If the command doesn't exist on your system PATH, you'll see an error message suggesting to unset `VOLTA_BYPASS`.

*Added in v0.6.7*

---

### `VOLTA_UNSAFE_GLOBAL`

Disables Volta's interception of global package manager commands (`npm install -g`, `npm link`, `pnpm add -g`, etc.).

**Use cases:**
- Installing global packages directly through npm/pnpm/yarn without Volta's management
- Troubleshooting global package installation issues

**Example:**
```bash
VOLTA_UNSAFE_GLOBAL=1 npm install -g some-package
```

Note: This only affects how Volta parses global install commands. The shim itself still runs through Volta (use `VOLTA_BYPASS` to skip the shim entirely).

---

## Directory Configuration

### `VOLTA_HOME`

Overrides the default Volta home directory where tools, shims, and cache are stored.

**Default locations:**
- Unix/macOS: `~/.volta`
- Windows: `%LOCALAPPDATA%\Volta`

**Example:**
```bash
export VOLTA_HOME=/custom/path/to/volta
```

---

### `VOLTA_INSTALL_DIR`

Overrides the directory where Volta binaries (`volta`, `volta-shim`) are installed.

**Default:** Determined from the location of the currently running executable.

**Example:**
```bash
export VOLTA_INSTALL_DIR=/custom/bin
```

---

## Logging

### `VOLTA_LOGLEVEL`

Sets the logging verbosity level.

**Values:** `error`, `warn`, `info`, `debug`, `trace`

**Default behavior:**
- If stdout is a TTY: `info`
- If stdout is not a TTY (e.g., in scripts): `error`

**Example:**
```bash
# Enable debug logging
VOLTA_LOGLEVEL=debug volta install node

# Quiet mode - errors only
VOLTA_LOGLEVEL=error volta install node
```

*Added in v0.5.4*

---

## Feature Flags

### `VOLTA_FEATURE_PNPM`

Enables experimental support for pnpm.

**Example:**
```bash
export VOLTA_FEATURE_PNPM=1
volta install pnpm
```

*Added in v1.1.1 (experimental)*

---

## Internal Variables

The following variables are used internally by Volta and should not be set manually:

| Variable | Purpose |
|----------|---------|
| `_VOLTA_TOOL_RECURSION` | Prevents infinite recursion when shims call other shims |
| `VOLTA_WRITE_EVENTS_FILE` | Used internally for event monitoring |
