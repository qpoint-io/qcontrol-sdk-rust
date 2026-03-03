# qcontrol Rust SDK Guidelines

## Build Commands
- `make deps` - Download Frida devkits
- `make build` - Build the SDK
- `make clean` - Clean build artifacts
- `cd examples && make dev` - Start dev container with tools pre-installed
- `cd examples && make build` - Build example plugins

## Dev Environment
- The examples directory contains a Dockerfile that sets up the full dev environment.
- It includes Rust, qcontrol, and the Claude Code CLI.
- Run `make dev` inside the examples directory to drop into this environment.

## Code Style
- Follow standard Rust formatting (`cargo fmt`)
- Run `cargo clippy` for linting
- SDK headers are synced from the Zig SDK source of truth via bindgen
- Use the `export_plugin!` macro for defining plugins
