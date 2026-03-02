//! Internal FFI bindings (not part of public API)
//!
//! Uses bindgen-generated bindings from C headers.
//! The C headers are the single source of truth for ABI types.

// Include bindgen-generated bindings from C headers
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
#[allow(improper_ctypes)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// Re-export types from bindgen
pub use bindings::QCONTROL_PLUGIN_VERSION;
pub use bindings::qcontrol_buffer_t;
pub use bindings::qcontrol_file_action_t;
pub use bindings::qcontrol_file_action_type_t;
pub use bindings::qcontrol_file_action__bindgen_ty_1;
pub use bindings::qcontrol_file_close_event_t;
pub use bindings::qcontrol_file_ctx_t;
pub use bindings::qcontrol_file_open_event_t;
pub use bindings::qcontrol_file_pattern_t;
pub use bindings::qcontrol_file_read_event_t;
pub use bindings::qcontrol_file_rw_config_t;
pub use bindings::qcontrol_file_session_t;
pub use bindings::qcontrol_file_transform_fn;
pub use bindings::qcontrol_file_write_event_t;
pub use bindings::qcontrol_plugin_t;

// Buffer functions (implemented by agent)
pub use bindings::qcontrol_buffer_append;
pub use bindings::qcontrol_buffer_clear;
pub use bindings::qcontrol_buffer_contains;
pub use bindings::qcontrol_buffer_data;
pub use bindings::qcontrol_buffer_ends_with;
pub use bindings::qcontrol_buffer_index_of;
pub use bindings::qcontrol_buffer_insert_at;
pub use bindings::qcontrol_buffer_len;
pub use bindings::qcontrol_buffer_prepend;
pub use bindings::qcontrol_buffer_remove;
pub use bindings::qcontrol_buffer_remove_all;
pub use bindings::qcontrol_buffer_remove_range;
pub use bindings::qcontrol_buffer_replace;
pub use bindings::qcontrol_buffer_replace_all;
pub use bindings::qcontrol_buffer_set;
pub use bindings::qcontrol_buffer_starts_with;
