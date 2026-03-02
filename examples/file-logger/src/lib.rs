//! File logger plugin - logs all file operations to a file
//!
//! Environment variables:
//!   QCONTROL_LOG_FILE - Path to log file (default: /tmp/qcontrol.log)

use qcontrol::prelude::*;

static LOGGER: Logger = Logger::new();

fn init() -> Result<(), Error> {
    LOGGER.init();
    LOGGER.log("[file_logger.rs] initializing...");
    Ok(())
}

fn on_open(ev: &FileOpenEvent) -> FileOpenResult {
    LOGGER.log(&format!(
        "[file_logger.rs] open(\"{}\", 0x{:x}) = {}",
        ev.path(),
        ev.flags(),
        ev.result()
    ));
    FileOpenResult::Pass
}

fn on_read(_: FileState, ev: &FileReadEvent) -> FileAction {
    LOGGER.log(&format!(
        "[file_logger.rs] read({}, buf, {}) = {}",
        ev.fd(),
        ev.count(),
        ev.result()
    ));
    FileAction::Pass
}

fn on_write(_: FileState, ev: &FileWriteEvent) -> FileAction {
    LOGGER.log(&format!(
        "[file_logger.rs] write({}, buf, {}) = {}",
        ev.fd(),
        ev.count(),
        ev.result()
    ));
    FileAction::Pass
}

fn on_close(_: FileState, ev: &FileCloseEvent) {
    LOGGER.log(&format!(
        "[file_logger.rs] close({}) = {}",
        ev.fd(),
        ev.result()
    ));
}

export_plugin!(
    PluginBuilder::new("rust_file_logger")
        .on_init(init)
        .on_file_open(on_open)
        .on_file_read(on_read)
        .on_file_write(on_write)
        .on_file_close(on_close)
);
