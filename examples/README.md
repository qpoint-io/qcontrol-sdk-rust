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

## Building

```bash
make build   # Build all plugins (.so for dynamic loading)
make dist    # Build all plugins (.a for bundling)
make clean   # Remove all built plugins
```

Each plugin outputs to `target/release/` and `dist/`:
- **Shared library** — `lib<name>.so` (in target/release)
- **Static archive** — `dist/<name>-<arch>.a` (for bundling)

## Usage

```bash
ARCH=$(uname -m)-$(uname -s | tr A-Z a-z)

# Single plugin
QCONTROL_PLUGINS=./file-logger/dist/file-logger-$ARCH.so qcontrol wrap -- ls -la

# Multiple plugins
QCONTROL_PLUGINS=./file-logger/dist/file-logger-$ARCH.so,./access-control/dist/access-control-$ARCH.so \
  qcontrol wrap -- cat /tmp/secret_test.txt
```

## Bundling

```bash
# Build plugins
make dist

# Bundle a single plugin (see Known Limitations below)
qcontrol bundle --plugins ./file-logger/dist/file-logger-$ARCH.a -o rust-plugin.so

# Use the bundle
qcontrol wrap --bundle rust-plugin.so -- ./target_app
```

## Testing

```bash
# Run the test script with plugins
ARCH=$(uname -m)-$(uname -s | tr A-Z a-z)
QCONTROL_PLUGINS=./file-logger/dist/file-logger-$ARCH.so \
  qcontrol wrap -- ./test-file-ops.sh

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

## Known Limitations

### Multiple Plugin Bundling

**Multiple Rust plugins cannot be bundled together.** Each staticlib includes the full Rust standard library, causing duplicate symbol errors when linking.

Workarounds:
- Use dynamic loading (`QCONTROL_PLUGINS`) for multiple Rust plugins
- Bundle only one Rust plugin at a time
- Combine multiple plugins into a single crate before bundling
- Use C or Zig plugins when bundling multiple plugins is required
