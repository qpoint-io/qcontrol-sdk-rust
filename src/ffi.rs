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
pub use bindings::qcontrol_buffer_ops_t;
pub use bindings::qcontrol_buffer_t;
pub use bindings::qcontrol_netstat_endpoint_t;
pub use bindings::qcontrol_netstat_event_t;
pub use bindings::qcontrol_netstat_fn;
pub use bindings::qcontrol_netstat_protocol_t;
pub use bindings::qcontrol_netstat_protocol_t_QCONTROL_NETSTAT_PROTO_TCP as QCONTROL_NETSTAT_PROTO_TCP;
pub use bindings::qcontrol_netstat_protocol_t_QCONTROL_NETSTAT_PROTO_UDP as QCONTROL_NETSTAT_PROTO_UDP;
pub use bindings::qcontrol_plugin_t;
pub use bindings::qcontrol_string_view_t;
pub use bindings::{gid_t, pid_t, uid_t};
pub use bindings::{
    qcontrol_netstat_addr_family_t_QCONTROL_NETSTAT_ADDR_IPV4 as QCONTROL_NETSTAT_ADDR_IPV4,
    qcontrol_netstat_addr_family_t_QCONTROL_NETSTAT_ADDR_IPV6 as QCONTROL_NETSTAT_ADDR_IPV6,
    qcontrol_netstat_addr_family_t_QCONTROL_NETSTAT_ADDR_UNSPEC as QCONTROL_NETSTAT_ADDR_UNSPEC,
    qcontrol_netstat_direction_t_QCONTROL_NETSTAT_DIR_RECV as QCONTROL_NETSTAT_DIR_RECV,
    qcontrol_netstat_direction_t_QCONTROL_NETSTAT_DIR_SEND as QCONTROL_NETSTAT_DIR_SEND,
    qcontrol_netstat_event_type_t_QCONTROL_NETSTAT_EVENT_CONNECTED as QCONTROL_NETSTAT_EVENT_CONNECTED,
    qcontrol_netstat_event_type_t_QCONTROL_NETSTAT_EVENT_CONNECTING as QCONTROL_NETSTAT_EVENT_CONNECTING,
    qcontrol_netstat_event_type_t_QCONTROL_NETSTAT_EVENT_CONNECT_ABANDONED as QCONTROL_NETSTAT_EVENT_CONNECT_ABANDONED,
    qcontrol_netstat_event_type_t_QCONTROL_NETSTAT_EVENT_CONNECT_FAILED as QCONTROL_NETSTAT_EVENT_CONNECT_FAILED,
    qcontrol_netstat_event_type_t_QCONTROL_NETSTAT_EVENT_ENDED as QCONTROL_NETSTAT_EVENT_ENDED,
    qcontrol_netstat_event_type_t_QCONTROL_NETSTAT_EVENT_LOCAL_ENDPOINT as QCONTROL_NETSTAT_EVENT_LOCAL_ENDPOINT,
    qcontrol_netstat_event_type_t_QCONTROL_NETSTAT_EVENT_LOSSES as QCONTROL_NETSTAT_EVENT_LOSSES,
    qcontrol_netstat_event_type_t_QCONTROL_NETSTAT_EVENT_NAMES_CLOSED as QCONTROL_NETSTAT_EVENT_NAMES_CLOSED,
    qcontrol_netstat_event_type_t_QCONTROL_NETSTAT_EVENT_OPENED as QCONTROL_NETSTAT_EVENT_OPENED,
    qcontrol_netstat_event_type_t_QCONTROL_NETSTAT_EVENT_PEER_ENDPOINT as QCONTROL_NETSTAT_EVENT_PEER_ENDPOINT,
    qcontrol_netstat_event_type_t_QCONTROL_NETSTAT_EVENT_PROCESS_ARMED as QCONTROL_NETSTAT_EVENT_PROCESS_ARMED,
    qcontrol_netstat_event_type_t_QCONTROL_NETSTAT_EVENT_PROCESS_ENDED as QCONTROL_NETSTAT_EVENT_PROCESS_ENDED,
    qcontrol_netstat_event_type_t_QCONTROL_NETSTAT_EVENT_STOPPED as QCONTROL_NETSTAT_EVENT_STOPPED,
    qcontrol_netstat_event_type_t_QCONTROL_NETSTAT_EVENT_TRANSFERRED as QCONTROL_NETSTAT_EVENT_TRANSFERRED,
    qcontrol_netstat_event_type_t_QCONTROL_NETSTAT_EVENT_TRANSFER_FAILED as QCONTROL_NETSTAT_EVENT_TRANSFER_FAILED,
    qcontrol_netstat_event_type_t_QCONTROL_NETSTAT_EVENT_UNKNOWN as QCONTROL_NETSTAT_EVENT_UNKNOWN,
    qcontrol_netstat_event_type_t_QCONTROL_NETSTAT_EVENT_WATERMARK as QCONTROL_NETSTAT_EVENT_WATERMARK,
    qcontrol_netstat_instance_end_t_QCONTROL_NETSTAT_INSTANCE_CLOSED as QCONTROL_NETSTAT_INSTANCE_CLOSED,
    qcontrol_netstat_instance_end_t_QCONTROL_NETSTAT_INSTANCE_REPLACED as QCONTROL_NETSTAT_INSTANCE_REPLACED,
    qcontrol_netstat_unknown_cause_t_QCONTROL_NETSTAT_UNKNOWN_RECORDS_LOST as QCONTROL_NETSTAT_UNKNOWN_RECORDS_LOST,
    QCONTROL_NETSTAT_EVENT_HAS_PEER, QCONTROL_NETSTAT_EVENT_LATE,
    QCONTROL_NETSTAT_EVENT_NAME_NOT_OPEN, QCONTROL_NETSTAT_EVENT_PREEXISTING,
};
pub use bindings::{QCONTROL_CONFIG_INVALID, QCONTROL_CONFIG_OK, QCONTROL_PLUGIN_VERSION};

// ============================================================================
// File types
// ============================================================================
pub use bindings::qcontrol_file_action__bindgen_ty_1;
pub use bindings::qcontrol_file_action_t;
pub use bindings::qcontrol_file_action_type_t;
pub use bindings::qcontrol_file_close_event_t;
pub use bindings::qcontrol_file_ctx_t;
pub use bindings::qcontrol_file_op_action__bindgen_ty_1;
pub use bindings::qcontrol_file_op_action_t;
pub use bindings::qcontrol_file_op_action_type_t;
pub use bindings::qcontrol_file_op_action_type_t_QCONTROL_FILE_OP_ACTION_BLOCK;
pub use bindings::qcontrol_file_op_action_type_t_QCONTROL_FILE_OP_ACTION_BLOCK_ERRNO;
pub use bindings::qcontrol_file_op_action_type_t_QCONTROL_FILE_OP_ACTION_PASS;
pub use bindings::qcontrol_file_op_event_t;
pub use bindings::qcontrol_file_op_kind_t;
pub use bindings::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_ACCESS;
pub use bindings::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_CHMOD;
pub use bindings::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_CHOWN;
pub use bindings::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_LINK;
pub use bindings::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_MKDIR;
pub use bindings::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_OPENDIR;
pub use bindings::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_READLINK;
pub use bindings::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_RENAME;
pub use bindings::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_RMDIR;
pub use bindings::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_STAT;
pub use bindings::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_SYMLINK;
pub use bindings::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_UNLINK;
pub use bindings::qcontrol_file_open_event_t;
pub use bindings::qcontrol_file_pattern_t;
pub use bindings::qcontrol_file_policy_class_t;
pub use bindings::qcontrol_file_policy_class_t_QCONTROL_FILE_POLICY_CLASS_READ;
pub use bindings::qcontrol_file_policy_class_t_QCONTROL_FILE_POLICY_CLASS_WRITE;
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
pub use bindings::qcontrol_net_inspection_mode_t;
pub use bindings::qcontrol_net_pattern_t;
pub use bindings::qcontrol_net_phase_action__bindgen_ty_1;
pub use bindings::qcontrol_net_phase_action_t;
pub use bindings::qcontrol_net_phase_action_type_t;
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
pub use bindings::qcontrol_net_inspection_mode_t_QCONTROL_NET_INSPECTION_DEFAULT;
pub use bindings::qcontrol_net_inspection_mode_t_QCONTROL_NET_INSPECTION_INTERCEPT;
pub use bindings::qcontrol_net_inspection_mode_t_QCONTROL_NET_INSPECTION_TUNNEL;
pub use bindings::qcontrol_net_phase_action_type_t_QCONTROL_NET_PHASE_ACTION_BLOCK;
pub use bindings::qcontrol_net_phase_action_type_t_QCONTROL_NET_PHASE_ACTION_BLOCK_ERRNO;
pub use bindings::qcontrol_net_phase_action_type_t_QCONTROL_NET_PHASE_ACTION_PASS;

// Net direction constants
pub use bindings::qcontrol_net_direction_t_QCONTROL_NET_INBOUND;
pub use bindings::qcontrol_net_direction_t_QCONTROL_NET_OUTBOUND;

// ============================================================================
// Process types
// ============================================================================
pub use bindings::qcontrol_process_identity_t;
pub use bindings::qcontrol_process_lineage_entry_t;

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
pub use bindings::qcontrol_http_headers_ops_t;
pub use bindings::qcontrol_http_headers_t;
pub use bindings::qcontrol_http_message_done_event_t;
pub use bindings::qcontrol_http_message_kind_t;
pub use bindings::qcontrol_http_request_event_t;
pub use bindings::qcontrol_http_request_head_ops_t;
pub use bindings::qcontrol_http_request_head_t;
pub use bindings::qcontrol_http_response_event_t;
pub use bindings::qcontrol_http_response_head_ops_t;
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
pub use bindings::qcontrol_http_close_reason_t_QCONTROL_HTTP_CLOSE_UPGRADED;

// HTTP exchange flag constants
pub use bindings::qcontrol_http_exchange_flag_t_QCONTROL_HTTP_EXCHANGE_FLAG_NONE;
pub use bindings::qcontrol_http_exchange_flag_t_QCONTROL_HTTP_EXCHANGE_FLAG_REQUEST_DONE;
pub use bindings::qcontrol_http_exchange_flag_t_QCONTROL_HTTP_EXCHANGE_FLAG_RESPONSE_DONE;

// ============================================================================
// Stream types
// ============================================================================
pub use bindings::qcontrol_stream_action_t;
pub use bindings::qcontrol_stream_action_type_t;
pub use bindings::qcontrol_stream_ctx_t;
pub use bindings::qcontrol_stream_direction_t;
pub use bindings::qcontrol_stream_protocol_t;

pub use bindings::qcontrol_stream_action_type_t_QCONTROL_STREAM_ACTION_BLOCK;
pub use bindings::qcontrol_stream_action_type_t_QCONTROL_STREAM_ACTION_PASS;
pub use bindings::qcontrol_stream_action_type_t_QCONTROL_STREAM_ACTION_STATE;
pub use bindings::qcontrol_stream_direction_t_QCONTROL_STREAM_CLIENT_TO_SERVER;
pub use bindings::qcontrol_stream_direction_t_QCONTROL_STREAM_DIRECTION_UNKNOWN;
pub use bindings::qcontrol_stream_direction_t_QCONTROL_STREAM_SERVER_TO_CLIENT;
pub use bindings::qcontrol_stream_protocol_t_QCONTROL_STREAM_PROTOCOL_SSE;
pub use bindings::qcontrol_stream_protocol_t_QCONTROL_STREAM_PROTOCOL_WEBSOCKET;

// ============================================================================
// SSE types
// ============================================================================
pub use bindings::qcontrol_sse_close_event_t;
pub use bindings::qcontrol_sse_event_ops_t;
pub use bindings::qcontrol_sse_event_record_t;
pub use bindings::qcontrol_sse_event_t;
pub use bindings::qcontrol_sse_open_event_t;

// ============================================================================
// WebSocket types
// ============================================================================
pub use bindings::qcontrol_websocket_close_event_t;
pub use bindings::qcontrol_websocket_message_event_t;
pub use bindings::qcontrol_websocket_message_type_t;
pub use bindings::qcontrol_websocket_open_event_t;

pub use bindings::qcontrol_websocket_message_type_t_QCONTROL_WEBSOCKET_MESSAGE_BINARY;
pub use bindings::qcontrol_websocket_message_type_t_QCONTROL_WEBSOCKET_MESSAGE_TEXT;
