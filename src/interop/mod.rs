mod adapter;
pub mod ag_ui;
pub mod mcp_apps;
pub mod surface;

pub use adapter::{A2uiAdapter, AgUiAdapter, McpAppsAdapter, UiProtocolAdapter};
pub use ag_ui::{
    ADK_UI_SURFACE_EVENT_NAME, AgUiActivityDeltaEvent, AgUiActivitySnapshotEvent, AgUiCustomEvent,
    AgUiErrorEvent, AgUiEvent, AgUiEventType, AgUiMessagesSnapshotEvent, AgUiRawEvent,
    AgUiRunErrorEvent, AgUiRunFinishedEvent, AgUiRunStartedEvent, AgUiStateDeltaEvent,
    AgUiStateSnapshotEvent, AgUiStepEvent, AgUiTextMessageChunkEvent, AgUiTextMessageDeltaEvent,
    AgUiTextMessageEndEvent, AgUiTextMessageStartEvent, AgUiToolCallArgsEvent,
    AgUiToolCallChunkEvent, AgUiToolCallEndEvent, AgUiToolCallResultEvent, AgUiToolCallStartEvent,
    activity_delta_event, activity_snapshot_event, error_event, messages_snapshot_event, raw_event,
    run_error_event, state_delta_event, state_snapshot_event, step_finished_event,
    step_started_event, surface_to_event_stream, text_message_chunk_event, text_message_events,
    tool_call_chunk_event, tool_call_events,
};
pub use mcp_apps::{
    MCP_APPS_HTML_MIME_TYPE, MCP_APPS_PROTOCOL_VERSION, McpAppsAppCapabilities,
    McpAppsBridgeMetadata, McpAppsInitializeRequest, McpAppsInitializeRequestParams,
    McpAppsInitializeResult, McpAppsPartyInfo, McpAppsRenderOptions, McpAppsSurfacePayload,
    McpToolVisibility, build_default_mcp_apps_initialize_result,
    default_mcp_apps_host_capabilities, default_mcp_apps_host_context, default_mcp_apps_host_info,
    surface_to_mcp_apps_payload, validate_mcp_apps_render_options,
};
pub use surface::{UiProtocol, UiSurface};
