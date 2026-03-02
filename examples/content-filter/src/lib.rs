//! Content filter plugin - redacts sensitive data in .txt and .log files
//!
//! Demonstrates session configuration with buffer transforms.
//! Uses FileOpenResult::Session with FileRwConfig for pattern replacement.
//!
//! Environment variables:
//!   QCONTROL_LOG_FILE - Path to log file (default: /tmp/qcontrol.log)

use qcontrol::prelude::*;

/// Per-file state for tracking filter activity.
struct FilterState {
    path: String,
}

impl FilterState {
    fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

static LOGGER: Logger = Logger::new();

fn init() -> Result<(), Error> {
    LOGGER.init();
    LOGGER.log("[content_filter.rs] initializing...");
    Ok(())
}

fn on_open(ev: &FileOpenEvent) -> FileOpenResult {
    // Only filter successfully opened files
    if !ev.succeeded() {
        return FileOpenResult::Pass;
    }

    let path = ev.path();

    // Only filter .txt and .log files
    let is_txt = path.ends_with(".txt");
    let is_log = path.ends_with(".log");

    if !is_txt && !is_log {
        return FileOpenResult::Pass;
    }

    LOGGER.log(&format!("[content_filter.rs] filtering: {}", path));

    // Return Session with read transforms
    FileOpenResult::Session(
        FileSession::builder()
            .state(FilterState::new(path))
            .read(
                FileRwConfig::new()
                    // Static prefix added to all reads
                    .prefix_str("[FILTERED]\n")
                    // Pattern replacements for sensitive data
                    .replace("password", "********")
                    .replace("secret", "[REDACTED]")
                    .replace("api_key", "[HIDDEN]")
                    .replace("token", "[HIDDEN]")
            )
            .build()
    )
}

fn on_close(state: FileState, _: &FileCloseEvent) {
    if let Some(filter_state) = state.downcast_ref::<FilterState>() {
        LOGGER.log(&format!("[content_filter.rs] closed: {}", filter_state.path));
    }
}

fn cleanup() {
    LOGGER.log("[content_filter.rs] cleanup complete");
}

export_plugin!(
    PluginBuilder::new("rust_content_filter")
        .on_init(init)
        .on_cleanup(cleanup)
        .on_file_open(on_open)
        .on_file_close(on_close)
    // Note: on_file_read/on_file_write not needed - transforms are
    // handled declaratively via the session config returned from on_file_open
);
