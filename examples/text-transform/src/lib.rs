//! Text transform plugin - demonstrates custom buffer manipulation
//!
//! Uses FileRwConfig::transform() for advanced buffer operations.
//! Applies different transforms based on file extension:
//!   .upper    - Convert text to uppercase
//!   .rot13    - Apply ROT13 encoding
//!   .bracket  - Wrap content in brackets
//!
//! Environment variables:
//!   QCONTROL_LOG_FILE - Path to log file (default: /tmp/qcontrol.log)

use qcontrol::prelude::*;

/// Transform mode for the file.
#[derive(Debug, Clone, Copy)]
enum TransformMode {
    Upper,
    Rot13,
    Bracket,
}

/// Per-file state tracking transform mode.
struct TransformState {
    path: String,
    mode: TransformMode,
}

impl TransformState {
    fn new(path: &str, mode: TransformMode) -> Self {
        Self {
            path: path.to_string(),
            mode,
        }
    }
}

static LOGGER: Logger = Logger::new();

/// Custom transform function - modifies buffer based on transform mode.
fn read_transform(state: FileState, _ctx: &FileContext, buf: &mut Buffer) -> FileAction {
    let transform_state = match state.downcast_ref::<TransformState>() {
        Some(s) => s,
        None => return FileAction::Pass,
    };

    // Get the current buffer contents
    let data = buf.as_slice();
    if data.is_empty() {
        return FileAction::Pass;
    }

    match transform_state.mode {
        TransformMode::Upper => {
            // Convert to uppercase using buffer operations
            // Replace each lowercase letter with its uppercase equivalent
            for c in b'a'..=b'z' {
                let lower = [c];
                let upper = [c - 32];
                buf.replace_all(&lower, &upper);
            }
        }
        TransformMode::Rot13 => {
            // ROT13: rotate letters by 13 positions
            // Create a transformed copy and set the buffer
            let data = buf.as_slice();
            let transformed: Vec<u8> = data
                .iter()
                .map(|&c| {
                    if c >= b'a' && c <= b'z' {
                        b'a' + ((c - b'a' + 13) % 26)
                    } else if c >= b'A' && c <= b'Z' {
                        b'A' + ((c - b'A' + 13) % 26)
                    } else {
                        c
                    }
                })
                .collect();
            buf.set(&transformed);
        }
        TransformMode::Bracket => {
            // Wrap content in brackets
            buf.prepend(b"[[[ ");
            buf.append(b" ]]]");
        }
    }

    FileAction::Pass
}

fn init() -> Result<(), Error> {
    LOGGER.init();
    LOGGER.log("[text_transform.rs] initializing...");
    Ok(())
}

fn on_open(ev: &FileOpenEvent) -> FileOpenResult {
    if !ev.succeeded() {
        return FileOpenResult::Pass;
    }

    let path = ev.path();

    // Determine transform mode based on extension
    let mode = if path.ends_with(".upper") {
        Some(TransformMode::Upper)
    } else if path.ends_with(".rot13") {
        Some(TransformMode::Rot13)
    } else if path.ends_with(".bracket") {
        Some(TransformMode::Bracket)
    } else {
        None
    };

    if let Some(m) = mode {
        LOGGER.log(&format!(
            "[text_transform.rs] filtering {} with mode {:?}",
            path, m
        ));

        return FileOpenResult::Session(
            FileSession::builder()
                .state(TransformState::new(path, m))
                .read(FileRwConfig::new().transform(read_transform))
                .build(),
        );
    }

    FileOpenResult::Pass
}

fn on_close(state: FileState, _: &FileCloseEvent) {
    if let Some(transform_state) = state.downcast_ref::<TransformState>() {
        LOGGER.log(&format!(
            "[text_transform.rs] closed: {}",
            transform_state.path
        ));
    }
}

fn cleanup() {
    LOGGER.log("[text_transform.rs] cleanup complete");
}

export_plugin!(
    PluginBuilder::new("rust_text_transform")
        .on_init(init)
        .on_cleanup(cleanup)
        .on_file_open(on_open)
        .on_file_close(on_close)
);
