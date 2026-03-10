# Qcontrol Rust SDK

Intercept. Observe. Transform. Build secure sandboxes for AI agents and control any application from the inside.

---

**Overview:** [Introduction](#introduction) · [What Can You Build?](#what-can-you-build) · [Quick Start](#quick-start) · [Core Concepts](#core-concepts) · [Examples](#examples)

**API Reference:** [File Operations](#file-operations) · [Exec Operations](#exec-operations) · [Network Operations](#network-operations)

**Development:** [Building Plugins](#building-plugins) · [Bundling Plugins](#bundling-plugins) · [Environment Variables](#environment-variables)

---

## Introduction

The Qcontrol Rust SDK lets you build plugins that hook directly into system calls, giving you precise control over how applications interact with files, processes, and the network.

This makes Qcontrol ideal for building **AI agent sandboxes and runtimes**. As agents gain autonomy to read files, execute commands, and make network requests, you need visibility and control at the syscall level. Qcontrol gives you that:

- **Intercept file operations** — See every open, read, write, and close. Block access to sensitive paths or transform data as it flows through.
- **Intercept exec operations** — Monitor process spawning, modify arguments, capture stdin/stdout/stderr.
- **Intercept network operations** — Watch connections form, inspect send/recv traffic, detect TLS and protocols.

Your plugins run inside the target process via native function hooking. Observe silently, block operations, or transform data in transit. No application changes required.

The SDK handles all FFI internally. You write safe, idiomatic Rust.

## What Can You Build?

| Category | What You Can Do | Example Plugin |
|----------|-----------------|----------------|
| **AI Agent Sandboxes** | Constrain file access, limit network destinations, audit all agent actions | [access-control](examples/access-control/) |
| **Agent Runtimes** | Build secure execution environments with fine-grained syscall control | [file-logger](examples/file-logger/) |
| **Security** | Enforce allowlists, block sensitive paths, create application sandboxes | [access-control](examples/access-control/) |
| **Observability** | Trace all I/O, log syscalls, build audit trails, count bytes | [file-logger](examples/file-logger/), [byte-counter](examples/byte-counter/) |
| **Compliance** | Redact PII from output, mask credentials, filter sensitive data | [content-filter](examples/content-filter/) |
| **Development** | Mock file systems, inject test data, transform formats on the fly | [text-transform](examples/text-transform/) |

## Quick Start

A plugin that logs every file open:

```rust
use qcontrol::prelude::*;

fn on_open(ev: &FileOpenEvent) -> FileOpenResult {
    eprintln!("open: {}", ev.path());
    FileOpenResult::Pass
}

export_plugin!(
    PluginBuilder::new("hello-plugin")
        .on_file_open(on_open)
);
```

Bundle and run:

```bash
qcontrol bundle --plugins . -o hello-plugin-bundle.so
qcontrol wrap --bundle ./hello-plugin-bundle.so -- cat /etc/passwd
```

That's it. Your plugin now intercepts every file open in the wrapped process.

## Core Concepts

### Hooks

Qcontrol intercepts operations at three levels:

| Domain | Operations | Status |
|--------|------------|--------|
| **File** | open, read, write, close | Fully implemented |
| **Exec** | spawn, stdin, stdout, stderr, exit | SDK ready, agent coming soon |
| **Network** | connect, accept, send, recv, close | SDK ready, agent coming soon |

### Actions

Every callback returns an action that controls what happens next:

```rust
// Let the operation proceed normally
FileOpenResult::Pass

// Block with EACCES
FileOpenResult::Block

// Block with custom errno
FileOpenResult::BlockErrno(libc::ENOENT)

// Full interception with transforms
FileOpenResult::Session(
    FileSession::builder()
        .state(my_state)
        .read(FileRwConfig::new().prefix_str("[LOG] "))
        .build()
)
```

### Sessions

Sessions are where Qcontrol gets powerful. Return a session from your open/connect callback to configure automatic transforms:

```rust
FileOpenResult::Session(
    FileSession::builder()
        .state(my_state)
        .read(FileRwConfig::new()
            .prefix_str("[LOG] ")           // Prepend to every read
            .suffix_str("\n")               // Append to every read
            .replace("password", "********") // Pattern replacements
            .replace("api_key", "[HIDDEN]")
            .transform(my_custom_transform)) // Full control
        .build()
)
```

**Transform pipeline:** `prefix` → `replace` → `transform` → `suffix`

### Progressive Examples

**Observe** — Log operations without affecting them:

```rust
fn on_open(ev: &FileOpenEvent) -> FileOpenResult {
    eprintln!("open({}) = {}", ev.path(), ev.result());
    FileOpenResult::Pass  // No interception
}
```

**Block** — Deny access to specific paths:

```rust
fn on_open(ev: &FileOpenEvent) -> FileOpenResult {
    if ev.path().starts_with("/tmp/secret") {
        return FileOpenResult::Block;  // EACCES
    }
    FileOpenResult::Pass
}
```

**Transform** — Modify data in transit:

```rust
fn on_open(ev: &FileOpenEvent) -> FileOpenResult {
    if ev.path().ends_with(".log") {
        return FileOpenResult::Session(
            FileSession::builder()
                .read(FileRwConfig::new()
                    .replace("ERROR", "[REDACTED]"))
                .build()
        );
    }
    FileOpenResult::Pass
}
```

## Examples

| Plugin | Description | Key Concepts |
|--------|-------------|--------------|
| [file-logger](examples/file-logger/) | Logs all file operations to `/tmp/qcontrol.log` | Basic callbacks, Logger utility |
| [access-control](examples/access-control/) | Blocks access to `/tmp/secret*` paths | Blocking with `FileOpenResult::Block` |
| [byte-counter](examples/byte-counter/) | Tracks total bytes read and written | State tracking with sessions |
| [content-filter](examples/content-filter/) | Redacts "password" and "secret" from reads | Sessions with pattern replacement |
| [text-transform](examples/text-transform/) | Uppercases all file reads | Custom transform functions |
| [exec-logger](examples/exec-logger/) | Logs process spawns and exits | Exec API |
| [net-logger](examples/net-logger/) | Logs network connections and traffic | Network API |
| [net-transform](examples/net-transform/) | Rewrites plaintext network traffic | Network transform configuration |

## File Operations

File operations are fully implemented. Use these to observe, block, or transform file I/O.

### Callbacks

| Callback | Signature | Phase | Purpose |
|----------|-----------|-------|---------|
| `on_file_open` | `fn(&FileOpenEvent) -> FileOpenResult` | After open()/openat() | Decide interception |
| `on_file_read` | `fn(FileState, &FileReadEvent) -> FileAction` | After read() | Observe or block |
| `on_file_write` | `fn(FileState, &FileWriteEvent) -> FileAction` | Before write() | Observe or block |
| `on_file_close` | `fn(FileState, &FileCloseEvent)` | After close() | Cleanup state |

The `FileState` parameter provides access to your custom state returned from `on_file_open`.

### Events

**FileOpenEvent** — passed to `on_file_open`:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `path()` | `&str` | File path being opened |
| `path_bytes()` | `&[u8]` | File path as raw bytes |
| `flags()` | `i32` | Open flags (O_RDONLY, O_WRONLY, etc.) |
| `mode()` | `u32` | File mode (for O_CREAT) |
| `result()` | `i32` | Result fd on success, negative errno on failure |
| `succeeded()` | `bool` | Whether open succeeded |
| `fd()` | `Option<i32>` | File descriptor if succeeded |

**FileReadEvent** — passed to `on_file_read`:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `fd()` | `i32` | File descriptor |
| `count()` | `usize` | Requested byte count |
| `result()` | `isize` | Bytes read or negative errno |
| `data()` | `Option<&[u8]>` | Data that was read (if result > 0) |
| `data_str()` | `Option<&str>` | Data as UTF-8 string |

**FileWriteEvent** — passed to `on_file_write`:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `fd()` | `i32` | File descriptor |
| `count()` | `usize` | Byte count to write |
| `result()` | `isize` | Bytes written or negative errno |
| `data()` | `&[u8]` | Data being written |
| `data_str()` | `Option<&str>` | Data as UTF-8 string |

**FileCloseEvent** — passed to `on_file_close`:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `fd()` | `i32` | File descriptor |
| `result()` | `i32` | Result (0 or negative errno) |
| `succeeded()` | `bool` | Whether close succeeded |

### Actions

**FileOpenResult** — return from `on_file_open`:

| Variant | Description |
|---------|-------------|
| `FileOpenResult::Pass` | No interception, continue normally |
| `FileOpenResult::Block` | Block with EACCES |
| `FileOpenResult::BlockErrno(i32)` | Block with custom errno |
| `FileOpenResult::Session(FileSession)` | Intercept with transform config |

**FileAction** — return from `on_file_read`/`on_file_write`:

| Variant | Description |
|---------|-------------|
| `FileAction::Pass` | Continue normally |
| `FileAction::Block` | Block with EACCES |
| `FileAction::BlockErrno(i32)` | Block with custom errno |

### Sessions and Transforms

Return a `FileSession` from `on_file_open` to configure read/write transforms:

```rust
fn on_open(ev: &FileOpenEvent) -> FileOpenResult {
    if !ev.succeeded() {
        return FileOpenResult::Pass;
    }

    FileOpenResult::Session(
        FileSession::builder()
            .state(MyState { count: 0 })  // Optional: custom state
            .read(FileRwConfig::new()     // Optional: read transform config
                .prefix_str("[LOG] ")
                .suffix_str("\n")
                .replace("password", "********")
                .replace("secret", "[REDACTED]")
                .transform(my_transform_fn))
            .write(FileRwConfig::new())   // Optional: write transform config
            .set_path("/redirected/path") // Optional: redirect to different path
            .set_flags(libc::O_RDONLY)    // Optional: override open flags
            .set_mode(0o644)              // Optional: override file mode
            .build()
    )
}
```

**FileRwConfig methods:**

| Method | Description |
|--------|-------------|
| `prefix(impl Into<Vec<u8>>)` | Static prefix to prepend |
| `prefix_str(&str)` | Static prefix string |
| `suffix(impl Into<Vec<u8>>)` | Static suffix to append |
| `suffix_str(&str)` | Static suffix string |
| `replace(&str, &str)` | Add pattern replacement |
| `patterns(Vec<FilePattern>)` | Add multiple patterns |
| `transform(FileTransformFn)` | Custom transform function |

**Transform pipeline order:** `prefix` → `replace` → `transform` → `suffix`

**Custom transform function:**

```rust
fn my_transform(
    state: FileState,
    ctx: &FileContext,
    buf: &mut Buffer
) -> FileAction {
    // ctx provides: fd(), path(), flags()
    // buf provides: read and modify methods

    if buf.contains_str("sensitive") {
        buf.replace_all_str("sensitive", "[HIDDEN]");
    }

    FileAction::Pass  // or FileAction::Block
}
```

### Buffer API

The `Buffer` type provides methods for inspecting and modifying data:

**Read operations:**

| Method | Description |
|--------|-------------|
| `as_slice()` | Get read-only slice of contents |
| `as_str()` | Get contents as UTF-8 string |
| `len()` | Get buffer length |
| `is_empty()` | Check if buffer is empty |
| `contains(&[u8])` | Check if buffer contains needle |
| `contains_str(&str)` | Check if buffer contains string |
| `starts_with(&[u8])` | Check if buffer starts with prefix |
| `starts_with_str(&str)` | Check if buffer starts with string |
| `ends_with(&[u8])` | Check if buffer ends with suffix |
| `ends_with_str(&str)` | Check if buffer ends with string |
| `find(&[u8])` | Find position of needle (None if not found) |
| `find_str(&str)` | Find position of string |

**Write operations:**

| Method | Description |
|--------|-------------|
| `prepend(&[u8])` | Add data to beginning |
| `prepend_str(&str)` | Add string to beginning |
| `append(&[u8])` | Add data to end |
| `append_str(&str)` | Add string to end |
| `replace(&[u8], &[u8])` | Replace first occurrence (returns bool) |
| `replace_str(&str, &str)` | Replace first string (returns bool) |
| `replace_all(&[u8], &[u8])` | Replace all occurrences (returns count) |
| `replace_all_str(&str, &str)` | Replace all strings (returns count) |
| `remove(&[u8])` | Remove first occurrence (returns bool) |
| `remove_str(&str)` | Remove first string (returns bool) |
| `remove_all(&[u8])` | Remove all occurrences (returns count) |
| `remove_all_str(&str)` | Remove all strings (returns count) |
| `clear()` | Clear buffer contents |
| `set(&[u8])` | Replace entire buffer contents |
| `set_str(&str)` | Replace with string |
| `insert_at(usize, &[u8])` | Insert data at position |
| `insert_at_str(usize, &str)` | Insert string at position |
| `remove_range(usize, usize)` | Remove byte range |

### Patterns

Use the `patterns!` macro for declarative string replacements:

```rust
use qcontrol::patterns;

let pats = patterns![
    "password" => "********",
    "secret" => "[REDACTED]",
    "api_key" => "[HIDDEN]",
];

FileOpenResult::Session(
    FileSession::builder()
        .read(FileRwConfig::new().patterns(pats))
        .build()
)
```

Or create patterns manually:

```rust
let my_patterns = vec![
    FilePattern::from_str("foo", "bar"),
    FilePattern::from_str("baz", "qux"),
];
```

## Exec Operations

> **Note:** Agent support coming soon. The SDK is stable, so you can write plugins now.

### Callbacks

| Callback | Signature | Phase | Purpose |
|----------|-----------|-------|---------|
| `on_exec` | `fn(&ExecEvent) -> ExecResult` | Before exec | Decide interception |
| `on_exec_stdin` | `fn(FileState, &StdinEvent) -> ExecAction` | Before stdin write | Observe or block |
| `on_exec_stdout` | `fn(FileState, &StdoutEvent) -> ExecAction` | After stdout read | Observe or block |
| `on_exec_stderr` | `fn(FileState, &StderrEvent) -> ExecAction` | After stderr read | Observe or block |
| `on_exec_exit` | `fn(FileState, &ExitEvent)` | After exit | Cleanup state |

### Events

**ExecEvent** — passed to `on_exec`:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `path()` | `&str` | Executable path |
| `path_bytes()` | `&[u8]` | Executable path as bytes |
| `argc()` | `usize` | Argument count |
| `argv()` | `impl Iterator<Item = &str>` | Iterator over arguments |
| `envc()` | `usize` | Environment variable count |
| `envp()` | `impl Iterator<Item = &str>` | Iterator over env vars (KEY=VALUE) |
| `cwd()` | `Option<&str>` | Working directory (if set) |

**StdinEvent** — passed to `on_exec_stdin`:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `pid()` | `i32` | Child process ID |
| `data()` | `&[u8]` | Data being written to stdin |
| `data_str()` | `Option<&str>` | Data as UTF-8 string |
| `count()` | `usize` | Byte count |

**StdoutEvent** / **StderrEvent** — passed to `on_exec_stdout`/`on_exec_stderr`:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `pid()` | `i32` | Child process ID |
| `data()` | `Option<&[u8]>` | Data read (if result > 0) |
| `data_str()` | `Option<&str>` | Data as UTF-8 string |
| `count()` | `usize` | Requested byte count |
| `result()` | `isize` | Bytes read or negative errno |

**ExitEvent** — passed to `on_exec_exit`:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `pid()` | `i32` | Child process ID |
| `exit_code()` | `i32` | Exit code (if normal exit) |
| `exit_signal()` | `i32` | Signal number (0 if normal) |
| `exited_normally()` | `bool` | Whether process exited normally |

### Actions

**ExecResult** — return from `on_exec`:

| Variant | Description |
|---------|-------------|
| `ExecResult::Pass` | No interception |
| `ExecResult::Block` | Block with EACCES |
| `ExecResult::BlockErrno(i32)` | Block with custom errno |
| `ExecResult::Session(ExecSession)` | Intercept with config |
| `ExecResult::State(*mut c_void)` | Track state only |

**ExecAction** — return from stdin/stdout/stderr callbacks:

| Variant | Description |
|---------|-------------|
| `ExecAction::Pass` | Continue normally |
| `ExecAction::Block` | Block operation |
| `ExecAction::BlockErrno(i32)` | Block with custom errno |

### Sessions

```rust
fn on_exec(ev: &ExecEvent) -> ExecResult {
    ExecResult::Session(
        ExecSession::builder()
            .state(MyState::new())  // Custom state

            // Exec modifications
            .set_path("/usr/bin/safe-wrapper")
            .set_argv(&["wrapper", "--safe"])
            .prepend_argv(&["--verbose"])
            .append_argv(&["--", "extra"])
            .set_env(&[("SAFE_MODE", "1")])
            .unset_env(&["DEBUG"])
            .set_cwd("/tmp/sandbox")

            // I/O transforms
            .stdin(ExecRwConfig::new().replace("secret", "***"))
            .stdout(ExecRwConfig::new().prefix_str("[OUT] "))
            .stderr(ExecRwConfig::new().prefix_str("[ERR] "))
            .build()
    )
}
```

## Network Operations

> **Note:** Proxy-backed wrap mode already exercises this ABI today. Native agent-side network hooks are still coming.

### Callbacks

| Callback | Signature | Phase | Purpose |
|----------|-----------|-------|---------|
| `on_net_connect` | `fn(&ConnectEvent) -> ConnectResult` | After connect() | Decide interception |
| `on_net_accept` | `fn(&AcceptEvent) -> AcceptResult` | After accept() | Decide interception |
| `on_net_tls` | `fn(FileState, &TlsEvent)` | After TLS handshake | Observe |
| `on_net_domain` | `fn(FileState, &DomainEvent)` | Domain discovered | Observe |
| `on_net_protocol` | `fn(FileState, &ProtocolEvent)` | Protocol detected | Observe |
| `on_net_send` | `fn(FileState, &SendEvent) -> NetAction` | Before send | Observe or block |
| `on_net_recv` | `fn(FileState, &RecvEvent) -> NetAction` | After recv | Observe or block |
| `on_net_close` | `fn(FileState, &NetCloseEvent)` | After close | Cleanup state |

### Events

**ConnectEvent** — passed to `on_net_connect`:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `fd()` | `i32` | Socket file descriptor |
| `dst_addr()` | `&str` | Destination IP address |
| `dst_port()` | `u16` | Destination port |
| `src_addr()` | `&str` | Local source address (empty if not bound) |
| `src_port()` | `u16` | Local source port |
| `result()` | `i32` | 0 on success, negative errno |
| `succeeded()` | `bool` | Whether connect succeeded |

**AcceptEvent** — passed to `on_net_accept`:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `fd()` | `i32` | Accepted socket fd |
| `listen_fd()` | `i32` | Listening socket fd |
| `src_addr()` | `&str` | Remote client address |
| `src_port()` | `u16` | Remote client port |
| `dst_addr()` | `&str` | Local server address |
| `dst_port()` | `u16` | Local server port |
| `result()` | `i32` | fd on success, negative errno |
| `succeeded()` | `bool` | Whether accept succeeded |

**TlsEvent** — passed to `on_net_tls`:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `fd()` | `i32` | Socket fd |
| `version()` | `&str` | TLS version (e.g., "TLSv1.3") |
| `cipher()` | `Option<&str>` | Cipher suite |

**DomainEvent** — passed to `on_net_domain`:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `fd()` | `i32` | Socket fd |
| `domain()` | `&str` | Domain name |

**ProtocolEvent** — passed to `on_net_protocol`:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `fd()` | `i32` | Socket fd |
| `protocol()` | `&str` | Protocol (e.g., "http/1.1", "h2") |

**SendEvent** — passed to `on_net_send`:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `fd()` | `i32` | Socket fd |
| `data()` | `&[u8]` | Data being sent |
| `data_str()` | `Option<&str>` | Data as UTF-8 string |
| `count()` | `usize` | Byte count |

**RecvEvent** — passed to `on_net_recv`:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `fd()` | `i32` | Socket fd |
| `data()` | `Option<&[u8]>` | Data received (if result > 0) |
| `data_str()` | `Option<&str>` | Data as UTF-8 string |
| `count()` | `usize` | Requested byte count |
| `result()` | `isize` | Bytes received or negative errno |

**NetCloseEvent** — passed to `on_net_close`:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `fd()` | `i32` | Socket fd |
| `result()` | `i32` | 0 on success, negative errno |
| `succeeded()` | `bool` | Whether close succeeded |

### Actions

**ConnectResult** / **AcceptResult** — return from connect/accept callbacks:

| Variant | Description |
|---------|-------------|
| `::Pass` | No interception |
| `::Block` | Block with EACCES |
| `::BlockErrno(i32)` | Block with custom errno |
| `::Session(NetSession)` | Intercept with config |
| `::State(*mut c_void)` | Track state only |

**NetAction** — return from send/recv callbacks:

| Variant | Description |
|---------|-------------|
| `NetAction::Pass` | Continue normally |
| `NetAction::Block` | Block operation |
| `NetAction::BlockErrno(i32)` | Block with custom errno |

### Sessions

```rust
fn on_net_connect(ev: &ConnectEvent) -> ConnectResult {
    ConnectResult::Session(
        NetSession::builder()
            .state(MyState::new())  // Custom state

            // Connection modifications (connect only)
            .set_addr("proxy.example.com")
            .set_port(8080)

            // I/O transforms
            .send(NetRwConfig::new().replace("secret", "***"))
            .recv(NetRwConfig::new().prefix_str("[RECV] "))
            .build()
    )
}
```

### Context

The `NetContext` type in transform functions provides connection metadata:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `fd()` | `i32` | Socket fd |
| `direction()` | `NetDirection` | `Outbound` or `Inbound` |
| `src_addr()` | `&str` | Source address |
| `src_port()` | `u16` | Source port |
| `dst_addr()` | `&str` | Destination address |
| `dst_port()` | `u16` | Destination port |
| `is_tls()` | `bool` | Whether TLS is active |
| `tls_version()` | `Option<&str>` | TLS version |
| `domain()` | `Option<&str>` | Domain name (if discovered) |
| `protocol()` | `Option<&str>` | Protocol (if detected) |

## Building Plugins

### Project Setup

Create the following directory structure:

```
my-plugin/
  Cargo.toml
  src/
    lib.rs
```

**Cargo.toml** — Package configuration:

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
# cdylib: shared library for dynamic loading (QCONTROL_PLUGINS)
# staticlib: static archive for bundling (qcontrol bundle)
crate-type = ["cdylib", "staticlib"]

[dependencies]
qcontrol = { git = "https://github.com/qpoint-io/qcontrol-sdk-rust" }
```

### Plugin Structure

Plugins export a single `qcontrol_plugin` descriptor using the `export_plugin!` macro:

```rust
use qcontrol::prelude::*;

fn init() -> Result<(), Error> {
    // Called on load
    Ok(())
}

fn cleanup() {
    // Called on unload
}

export_plugin!(
    PluginBuilder::new("my-plugin")
        .on_init(init)                    // Optional: called on load
        .on_cleanup(cleanup)              // Optional: called on unload
        // File callbacks (optional)
        .on_file_open(on_file_open)
        .on_file_read(on_file_read)
        .on_file_write(on_file_write)
        .on_file_close(on_file_close)
        // Exec callbacks (optional)
        .on_exec(on_exec)
        .on_exec_stdin(on_exec_stdin)
        .on_exec_stdout(on_exec_stdout)
        .on_exec_stderr(on_exec_stderr)
        .on_exec_exit(on_exec_exit)
        // Net callbacks (optional)
        .on_net_connect(on_net_connect)
        .on_net_accept(on_net_accept)
        .on_net_tls(on_net_tls)
        .on_net_domain(on_net_domain)
        .on_net_protocol(on_net_protocol)
        .on_net_send(on_net_send)
        .on_net_recv(on_net_recv)
        .on_net_close(on_net_close)
);
```

All callbacks are optional. Only implement what you need.

### Building

```bash
# Debug build
cargo build

# Release build (recommended for production)
cargo build --release
```

Output locations:
- Shared library: `target/release/libmy_plugin.so`
- Static archive: `target/release/libmy_plugin.a`

### Using Bundles

Bundle plugins directly from plugin directories:

```bash
qcontrol bundle --plugins ./my-plugin -o my-plugin-bundle.so
qcontrol wrap --bundle ./my-plugin-bundle.so -- ./target

# Multiple plugins
qcontrol bundle --plugins ./logger,./blocker -o my-plugins.so
qcontrol wrap --bundle ./my-plugins.so -- ./target
```

## Bundling Plugins

For distribution, bundle plugins with the agent core into a single `.so` file.

### Creating a Bundle

Create the bundle directly from plugin directories:

```bash
qcontrol bundle --plugins ./file-logger,./access-control -o my-bundle.so
```

You can also pass prebuilt static archives, or use a `bundle.toml` file when you want to describe larger bundles declaratively:

```toml
[bundle]
output = "my-plugins.so"

[[plugins]]
source = "./file-logger"

[[plugins]]
source = "./access-control"
```

```bash
qcontrol bundle --config bundle.toml
```

### Using Bundles

```bash
# Via command line flag
qcontrol wrap --bundle my-bundle.so -- ./target

# Via environment variable
QCONTROL_BUNDLE=./my-bundle.so qcontrol wrap -- ./target
```

## API Reference

### Plugin Lifecycle

```rust
fn init() -> Result<(), Error> {
    // Called after plugin is loaded
    // Initialize resources, open log files, etc.
    Ok(())
}

fn cleanup() {
    // Called before plugin is unloaded
    // Clean up resources, close files, etc.
}
```

### Logger Utility

Thread-safe file logger with reentrancy protection:

```rust
use qcontrol::prelude::*;

static LOGGER: Logger = Logger::new();

fn init() -> Result<(), Error> {
    LOGGER.init();
    LOGGER.log("[my-plugin] started");
    Ok(())
}

fn cleanup() {
    LOGGER.log("[my-plugin] stopped");
}

fn on_open(ev: &FileOpenEvent) -> FileOpenResult {
    LOGGER.log(&format!("open: {} flags=0x{:x}", ev.path(), ev.flags()));
    FileOpenResult::Pass
}
```

The log file path is controlled by `QCONTROL_LOG_FILE` (default: `/tmp/qcontrol.log`).

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `QCONTROL_PLUGINS` | (none) | Comma-separated paths to plugin `.so` files |
| `QCONTROL_BUNDLE` | (none) | Path to bundled plugin `.so` file |
| `QCONTROL_LOG_FILE` | `/tmp/qcontrol.log` | Log file path for Logger utility |

## License

Apache License 2.0
