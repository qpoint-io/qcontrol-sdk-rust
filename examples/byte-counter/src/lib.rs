//! Byte counter plugin - tracks bytes read/written per file
//!
//! Demonstrates per-file state tracking without transforms.
//! Uses FileOpenResult::Session with state to attach custom state to each file.
//!
//! Environment variables:
//!   QCONTROL_LOG_FILE - Path to log file (default: /tmp/qcontrol.log)

use std::sync::atomic::{AtomicUsize, Ordering};

use qcontrol::prelude::*;

/// Per-file statistics tracked from open to close.
struct FileStats {
    path: String,
    bytes_read: AtomicUsize,
    bytes_written: AtomicUsize,
    read_calls: AtomicUsize,
    write_calls: AtomicUsize,
}

impl FileStats {
    fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            bytes_read: AtomicUsize::new(0),
            bytes_written: AtomicUsize::new(0),
            read_calls: AtomicUsize::new(0),
            write_calls: AtomicUsize::new(0),
        }
    }
}

static LOGGER: Logger = Logger::new();

fn init() -> Result<(), Error> {
    LOGGER.init();
    LOGGER.log("[byte_counter.rs] initializing...");
    Ok(())
}

fn on_open(ev: &FileOpenEvent) -> FileOpenResult {
    // Only track successfully opened files
    if !ev.succeeded() {
        return FileOpenResult::Pass;
    }

    let path = ev.path();

    // Skip common paths to reduce noise
    if path.starts_with("/proc/")
        || path.starts_with("/sys/")
        || path.starts_with("/dev/")
    {
        return FileOpenResult::Pass;
    }

    LOGGER.log(&format!("[byte_counter.rs] tracking: {}", path));

    // Return Session with state to track this file
    FileOpenResult::Session(
        FileSession::builder()
            .state(FileStats::new(path))
            .build()
    )
}

fn on_read(state: FileState, ev: &FileReadEvent) -> FileAction {
    if let Some(stats) = state.downcast_ref::<FileStats>() {
        let bytes = ev.result();
        if bytes > 0 {
            stats.bytes_read.fetch_add(bytes as usize, Ordering::Relaxed);
            stats.read_calls.fetch_add(1, Ordering::Relaxed);
        }
    }
    FileAction::Pass
}

fn on_write(state: FileState, ev: &FileWriteEvent) -> FileAction {
    if let Some(stats) = state.downcast_ref::<FileStats>() {
        let bytes = ev.result();
        if bytes > 0 {
            stats.bytes_written.fetch_add(bytes as usize, Ordering::Relaxed);
            stats.write_calls.fetch_add(1, Ordering::Relaxed);
        }
    }
    FileAction::Pass
}

fn on_close(state: FileState, _: &FileCloseEvent) {
    if let Some(stats) = state.downcast_ref::<FileStats>() {
        LOGGER.log(&format!(
            "[byte_counter.rs] {}: read {} bytes ({} calls), wrote {} bytes ({} calls)",
            stats.path,
            stats.bytes_read.load(Ordering::Relaxed),
            stats.read_calls.load(Ordering::Relaxed),
            stats.bytes_written.load(Ordering::Relaxed),
            stats.write_calls.load(Ordering::Relaxed),
        ));
    }
}

fn cleanup() {
    LOGGER.log("[byte_counter.rs] cleanup complete");
}

export_plugin!(
    PluginBuilder::new("rust_byte_counter")
        .on_init(init)
        .on_cleanup(cleanup)
        .on_file_open(on_open)
        .on_file_read(on_read)
        .on_file_write(on_write)
        .on_file_close(on_close)
);
