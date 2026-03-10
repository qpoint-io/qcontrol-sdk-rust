//! Access control plugin - blocks access to /tmp/secret* paths
//!
//! Environment variables:
//!   QCONTROL_LOG_FILE - Path to log file (default: /tmp/qcontrol.log)

use qcontrol::prelude::*;

static LOGGER: Logger = Logger::new();

fn init() -> Result<(), Error> {
    LOGGER.init();
    LOGGER.log("[access_control.rs] initializing - blocking /tmp/secret*");
    Ok(())
}

fn on_open(ev: &FileOpenEvent) -> FileOpenResult {
    if ev.path().starts_with("/tmp/secret") {
        LOGGER.log(&format!("[access_control.rs] BLOCKED: {}", ev.path()));
        return FileOpenResult::Block;
    }
    FileOpenResult::Pass
}

export_plugin!(PluginBuilder::new("rust_access_control")
    .on_init(init)
    .on_file_open(on_open));
