# Rust SDK Examples

Example plugins demonstrating the qcontrol Rust SDK for file, exec, and network interception.

## Plugins

| Plugin | Description |
|--------|-------------|
| file-logger | Logs all file operations to a log file |
| access-control | Blocks access to `/tmp/secret*` paths |
| byte-counter | Counts bytes read/written per file |
| content-filter | Redacts sensitive data in `.txt`/`.log` files |
| text-transform | Transforms text based on file extension |
| exec-logger | Logs all exec operations (v1 - not yet implemented) |
| net-logger | Logs network I/O like connect/tls/send/recv/close |
| net-transform | Rewrites plaintext network traffic with declarative recv transforms |

## Quick Start

```bash
make                  # Build all plugins into rust-plugins.so
qcontrol wrap --bundle ./rust-plugins.so -- ./your-app
```

Type-specific shortcuts are also available:

```bash
make bundle-file      # rust-file-plugins.so
make bundle-exec      # rust-exec-plugins.so
make bundle-net       # rust-net-plugins.so

make build-file       # shared libraries for file plugins
make build-exec       # shared libraries for exec plugins
make build-net        # shared libraries for net plugins
```

## Demo: Zero-Trust Governance

Use qcontrol to build unbreakable system-level guardrails for *any* application—from standard Linux utilities to autonomous AI coding agents.

Instead of relying on application logic or API restrictions, qcontrol intercepts system calls at the OS level to guarantee compliance without modifying the target binary.

**1. Install qcontrol or Start the Dev Environment**

Option A: install `qcontrol` directly:
```bash
curl -s https://get.qpoint.io/qcontrol/demo | sh
```

Option B: start the pre-configured development environment with the SDK, compiler toolchain, and Anthropic's Claude Code AI assistant installed:
```bash
make dev
```

**2. Build the Plugins**
```bash
make
```

**3. Set up the Demo**

Let's use the `access-control` plugin to protect a mock API key file.
```bash
echo "super_secret_key_123" > /tmp/secret_api_key.txt
```

**4. Watch the OS block the read**

Launch the standard `cat` utility, but wrap it in qcontrol's access-control policy:
```bash
qcontrol wrap --bundle ./rust-plugins.so -- cat /tmp/secret_api_key.txt
```

**What Happens:**
`cat` will attempt to read the file, but qcontrol will intercept and deny the `open()` syscall at the C ABI boundary.
```text
cat: /tmp/secret_api_key.txt
```

Check the audit log to see the interception:
```bash
cat /tmp/qcontrol.log
```
```text
[access_control.rs] BLOCKED: /tmp/secret_api_key.txt
```

### Next Step: Sandboxing Autonomous AI

Because qcontrol works at the system level, you can wrap autonomous AI tools to create unbreakable guardrails against prompt injections. The dev container has Anthropic's Claude Code CLI pre-installed to test this.

If you have an Anthropic Console account, you can try sandboxing the AI:

```bash
# 1. Authenticate the AI
claude auth login

# 2. Command the AI to read the secret file, but wrap it in our policy
qcontrol wrap --bundle ./rust-plugins.so -- claude -p "Read /tmp/secret_api_key.txt and summarize its contents."
```

Claude will hit the system-level block, realize it is sandboxed, and gracefully respond: *"I cannot complete this request because I received a permission denied error trying to read the file."*

## Testing

```bash
# Run the test script with plugins
qcontrol wrap --bundle ./rust-plugins.so -- ./test-file-ops.sh

# Check log output
cat /tmp/qcontrol.log
```

### Network Demo

Bundle the network logger example and make an HTTPS request through `wrap`:

```bash
qcontrol bundle --plugins ./net-logger -o ./net-logger-demo.so
qcontrol wrap --bundle ./net-logger-demo.so -- ./test-net-io.sh https://example.com/
```

Then inspect the log:

```bash
grep net_logger.rs /tmp/qcontrol.log
```

You should see entries for `connect`, `domain`, `protocol`, `send`, `recv`, and `close`. HTTPS requests should also emit a `tls` line once the network layer identifies a local TLS session.

### Network Transform Demo

Bundle the transform example and run it against a local HTTP server:

```bash
qcontrol bundle --plugins ./net-transform -o ./net-transform-demo.so
qcontrol wrap --bundle ./net-transform-demo.so -- ./test-net-transform.sh
```

You should see the body rewritten from:

```text
hello from demo server
```

to:

```text
hullo from demo client
```

This example uses `NetRwConfig::patterns(...)` to demonstrate that net plugins can modify plaintext application data before it reaches the client. The demo intentionally uses same-length replacements so plain HTTP `Content-Length` remains valid without protocol-aware header rewriting.

## Writing Plugins

See [file-logger/src/lib.rs](file-logger/src/lib.rs) for a complete example.

```rust
use qcontrol::prelude::*;

fn on_open(ev: &FileOpenEvent) -> FileOpenResult {
    eprintln!("open({}) = {}", ev.path(), ev.result());
    FileOpenResult::Pass  // or FileOpenResult::Block
}

export_plugin!(
    PluginBuilder::new("my_plugin")
        .on_file_open(on_open)
);
```

Add to Cargo.toml:
```toml
[lib]
crate-type = ["cdylib", "staticlib"]

[dependencies]
qcontrol = { path = ".." }
```

## Advanced: Bundling Individual Plugins

For development, you can bundle just the plugins you want without writing a `bundle.toml` file:

```bash
# Bundle one plugin
qcontrol bundle --plugins ./file-logger -o ./file-logger.so
qcontrol wrap --bundle ./file-logger.so -- ls -la

# Bundle multiple plugins
qcontrol bundle --plugins ./file-logger,./access-control -o ./file-tools.so
qcontrol wrap --bundle ./file-tools.so -- cat /tmp/secret_test.txt
```
