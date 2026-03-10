//! Thread-safe file logger for qcontrol plugins.
//!
//! Reads log path from QCONTROL_LOG_FILE environment variable,
//! defaulting to /tmp/qcontrol.log.

use std::cell::Cell;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};

const DEFAULT_LOG_PATH: &str = "/tmp/qcontrol.log";

thread_local! {
    static IN_LOGGING: Cell<bool> = const { Cell::new(false) };
}

/// Thread-safe file logger with reentrancy protection.
pub struct Logger {
    file: OnceLock<Mutex<File>>,
}

impl Logger {
    /// Create a new uninitialized logger.
    pub const fn new() -> Self {
        Self {
            file: OnceLock::new(),
        }
    }

    /// Initialize the logger, opening the log file.
    ///
    /// Reads path from QCONTROL_LOG_FILE env var, defaults to /tmp/qcontrol.log.
    pub fn init(&self) {
        let log_path = std::env::var("QCONTROL_LOG_FILE")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_LOG_PATH.to_string());

        if let Ok(file) = OpenOptions::new().create(true).append(true).open(&log_path) {
            self.file.get_or_init(|| Mutex::new(file));
        }
    }

    /// Log a message to the file.
    ///
    /// Includes reentrancy protection to prevent infinite loops when
    /// logging operations that trigger more file operations.
    pub fn log(&self, msg: &str) {
        // Prevent reentrancy
        if IN_LOGGING.with(|f| f.get()) {
            return;
        }
        IN_LOGGING.with(|f| f.set(true));

        if let Some(file_mutex) = self.file.get() {
            if let Ok(mut guard) = file_mutex.lock() {
                let _ = writeln!(guard, "{}", msg);
                let _ = guard.flush();
            }
        }

        IN_LOGGING.with(|f| f.set(false));
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}
