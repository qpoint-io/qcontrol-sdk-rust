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

// Re-export common types
pub use bindings::qcontrol_buffer_t;
pub use bindings::qcontrol_plugin_t;
pub use bindings::QCONTROL_PLUGIN_VERSION;

// ============================================================================
// File types
// ============================================================================
pub use bindings::qcontrol_file_action__bindgen_ty_1;
pub use bindings::qcontrol_file_action_t;
pub use bindings::qcontrol_file_action_type_t;
pub use bindings::qcontrol_file_close_event_t;
pub use bindings::qcontrol_file_ctx_t;
pub use bindings::qcontrol_file_open_event_t;
pub use bindings::qcontrol_file_pattern_t;
pub use bindings::qcontrol_file_read_event_t;
pub use bindings::qcontrol_file_rw_config_t;
pub use bindings::qcontrol_file_session_t;
pub use bindings::qcontrol_file_transform_fn;
pub use bindings::qcontrol_file_write_event_t;
pub use bindings::QCONTROL_FILE_FLAGS_UNCHANGED;

// ============================================================================
// Exec types
// ============================================================================
pub use bindings::qcontrol_exec_action__bindgen_ty_1;
pub use bindings::qcontrol_exec_action_t;
pub use bindings::qcontrol_exec_action_type_t;
pub use bindings::qcontrol_exec_ctx_t;
pub use bindings::qcontrol_exec_event_t;
pub use bindings::qcontrol_exec_exit_event_t;
pub use bindings::qcontrol_exec_pattern_t;
pub use bindings::qcontrol_exec_rw_config_t;
pub use bindings::qcontrol_exec_session_t;
pub use bindings::qcontrol_exec_stderr_event_t;
pub use bindings::qcontrol_exec_stdin_event_t;
pub use bindings::qcontrol_exec_stdout_event_t;
pub use bindings::qcontrol_exec_transform_fn;

// Exec action type constants
pub use bindings::qcontrol_exec_action_type_t_QCONTROL_EXEC_ACTION_BLOCK;
pub use bindings::qcontrol_exec_action_type_t_QCONTROL_EXEC_ACTION_BLOCK_ERRNO;
pub use bindings::qcontrol_exec_action_type_t_QCONTROL_EXEC_ACTION_PASS;
pub use bindings::qcontrol_exec_action_type_t_QCONTROL_EXEC_ACTION_SESSION;
pub use bindings::qcontrol_exec_action_type_t_QCONTROL_EXEC_ACTION_STATE;

// ============================================================================
// Net types
// ============================================================================
pub use bindings::qcontrol_net_accept_event_t;
pub use bindings::qcontrol_net_action__bindgen_ty_1;
pub use bindings::qcontrol_net_action_t;
pub use bindings::qcontrol_net_action_type_t;
pub use bindings::qcontrol_net_close_event_t;
pub use bindings::qcontrol_net_connect_event_t;
pub use bindings::qcontrol_net_ctx_t;
pub use bindings::qcontrol_net_direction_t;
pub use bindings::qcontrol_net_domain_event_t;
pub use bindings::qcontrol_net_pattern_t;
pub use bindings::qcontrol_net_protocol_event_t;
pub use bindings::qcontrol_net_recv_event_t;
pub use bindings::qcontrol_net_rw_config_t;
pub use bindings::qcontrol_net_send_event_t;
pub use bindings::qcontrol_net_session_t;
pub use bindings::qcontrol_net_tls_event_t;
pub use bindings::qcontrol_net_transform_fn;

// Net action type constants
pub use bindings::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_BLOCK;
pub use bindings::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_BLOCK_ERRNO;
pub use bindings::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_PASS;
pub use bindings::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_SESSION;
pub use bindings::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_STATE;

// Net direction constants
pub use bindings::qcontrol_net_direction_t_QCONTROL_NET_INBOUND;
pub use bindings::qcontrol_net_direction_t_QCONTROL_NET_OUTBOUND;

// ============================================================================
// HTTP types
// ============================================================================
pub use bindings::qcontrol_http_action__bindgen_ty_1;
pub use bindings::qcontrol_http_action_t;
pub use bindings::qcontrol_http_action_type_t;
pub use bindings::qcontrol_http_body_event_t;
pub use bindings::qcontrol_http_body_flag_t;
pub use bindings::qcontrol_http_body_mode_t;
pub use bindings::qcontrol_http_close_reason_t;
pub use bindings::qcontrol_http_ctx_t;
pub use bindings::qcontrol_http_exchange_close_event_t;
pub use bindings::qcontrol_http_exchange_flag_t;
pub use bindings::qcontrol_http_header_t;
pub use bindings::qcontrol_http_headers_t;
pub use bindings::qcontrol_http_message_done_event_t;
pub use bindings::qcontrol_http_message_kind_t;
pub use bindings::qcontrol_http_request_event_t;
pub use bindings::qcontrol_http_request_head_t;
pub use bindings::qcontrol_http_response_event_t;
pub use bindings::qcontrol_http_response_head_t;
pub use bindings::qcontrol_http_trailers_event_t;
pub use bindings::qcontrol_http_version_t;

// HTTP action type constants
pub use bindings::qcontrol_http_action_type_t_QCONTROL_HTTP_ACTION_BLOCK;
pub use bindings::qcontrol_http_action_type_t_QCONTROL_HTTP_ACTION_PASS;
pub use bindings::qcontrol_http_action_type_t_QCONTROL_HTTP_ACTION_STATE;

// HTTP version constants
pub use bindings::qcontrol_http_version_t_QCONTROL_HTTP_VERSION_1_0;
pub use bindings::qcontrol_http_version_t_QCONTROL_HTTP_VERSION_1_1;
pub use bindings::qcontrol_http_version_t_QCONTROL_HTTP_VERSION_2;
pub use bindings::qcontrol_http_version_t_QCONTROL_HTTP_VERSION_UNKNOWN;

// HTTP message kind constants
pub use bindings::qcontrol_http_message_kind_t_QCONTROL_HTTP_MESSAGE_REQUEST;
pub use bindings::qcontrol_http_message_kind_t_QCONTROL_HTTP_MESSAGE_RESPONSE;

// HTTP body flag constants
pub use bindings::qcontrol_http_body_flag_t_QCONTROL_HTTP_BODY_FLAG_CONTENT_DECODED;
pub use bindings::qcontrol_http_body_flag_t_QCONTROL_HTTP_BODY_FLAG_NONE;
pub use bindings::qcontrol_http_body_flag_t_QCONTROL_HTTP_BODY_FLAG_TRANSFER_DECODED;

// HTTP body mode constants
pub use bindings::qcontrol_http_body_mode_t_QCONTROL_HTTP_BODY_MODE_BUFFER;
pub use bindings::qcontrol_http_body_mode_t_QCONTROL_HTTP_BODY_MODE_DEFAULT;
pub use bindings::qcontrol_http_body_mode_t_QCONTROL_HTTP_BODY_MODE_STREAM;

// HTTP close reason constants
pub use bindings::qcontrol_http_close_reason_t_QCONTROL_HTTP_CLOSE_ABORTED;
pub use bindings::qcontrol_http_close_reason_t_QCONTROL_HTTP_CLOSE_COMPLETE;
pub use bindings::qcontrol_http_close_reason_t_QCONTROL_HTTP_CLOSE_CONNECTION_CLOSED;
pub use bindings::qcontrol_http_close_reason_t_QCONTROL_HTTP_CLOSE_PARSE_ERROR;

// HTTP exchange flag constants
pub use bindings::qcontrol_http_exchange_flag_t_QCONTROL_HTTP_EXCHANGE_FLAG_NONE;
pub use bindings::qcontrol_http_exchange_flag_t_QCONTROL_HTTP_EXCHANGE_FLAG_REQUEST_DONE;
pub use bindings::qcontrol_http_exchange_flag_t_QCONTROL_HTTP_EXCHANGE_FLAG_RESPONSE_DONE;

// ============================================================================
// Buffer functions (implemented by agent)
// ============================================================================
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

// HTTP helper functions (implemented by the host/runtime)
pub use bindings::qcontrol_http_headers_add;
pub use bindings::qcontrol_http_headers_count;
pub use bindings::qcontrol_http_headers_data;
pub use bindings::qcontrol_http_headers_remove;
pub use bindings::qcontrol_http_headers_set;
pub use bindings::qcontrol_http_request_authority;
pub use bindings::qcontrol_http_request_authority_len;
pub use bindings::qcontrol_http_request_headers;
pub use bindings::qcontrol_http_request_method;
pub use bindings::qcontrol_http_request_method_len;
pub use bindings::qcontrol_http_request_path;
pub use bindings::qcontrol_http_request_path_len;
pub use bindings::qcontrol_http_request_raw_target;
pub use bindings::qcontrol_http_request_raw_target_len;
pub use bindings::qcontrol_http_request_scheme;
pub use bindings::qcontrol_http_request_scheme_len;
pub use bindings::qcontrol_http_request_set_authority;
pub use bindings::qcontrol_http_request_set_method;
pub use bindings::qcontrol_http_request_set_path;
pub use bindings::qcontrol_http_request_set_scheme;
pub use bindings::qcontrol_http_response_headers;
pub use bindings::qcontrol_http_response_reason;
pub use bindings::qcontrol_http_response_reason_len;
pub use bindings::qcontrol_http_response_set_reason;
pub use bindings::qcontrol_http_response_set_status_code;
pub use bindings::qcontrol_http_response_status_code;
