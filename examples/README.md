# Rust SDK Examples

Example plugins demonstrating the qcontrol Rust SDK for file operation filtering.

## Plugins

| Plugin | Description |
|--------|-------------|
| file-logger | Logs all file operations to a log file |
| access-control | Blocks access to `/tmp/secret*` paths |
| byte-counter | Counts bytes read/written per file |
| content-filter | Redacts sensitive data in `.txt`/`.log` files |
| text-transform | Transforms text based on file extension |

## Quick Start

```bash
make                  # Build all plugins into rust-plugins.so
qcontrol wrap --bundle ./rust-plugins.so -- ./your-app
```

## Demo: Zero-Trust Governance

Use qcontrol to build unbreakable system-level guardrails for *any* application—from standard Linux utilities to autonomous AI coding agents.

Instead of relying on application logic or API restrictions, qcontrol intercepts system calls at the OS level to guarantee compliance without modifying the target binary.

**1. Start the Dev Environment**

We have pre-configured a development container with the SDK, compiler toolchain, and Anthropic's Claude Code AI assistant installed.
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

## Advanced: Dynamic Loading

For development or when you need to load individual plugins without bundling:

```bash
# Build shared libraries
make build

# Load plugins dynamically
QCONTROL_PLUGINS=./target/release/libfile_logger.so qcontrol wrap -- ls -la

# Multiple plugins (comma-separated)
QCONTROL_PLUGINS=./target/release/libfile_logger.so,./target/release/libaccess_control.so \
  qcontrol wrap -- cat /tmp/secret_test.txt
```
