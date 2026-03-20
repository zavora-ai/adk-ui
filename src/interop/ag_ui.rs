use super::surface::UiSurface;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Event name used for surface payload transport via AG-UI custom events.
pub const ADK_UI_SURFACE_EVENT_NAME: &str = "adk.ui.surface";

/// AG-UI event types from the protocol event model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgUiEventType {
    RunStarted,
    RunFinished,
    RunError,
    StepStarted,
    StepFinished,
    TextMessageStart,
    TextMessageContent,
    TextMessageDelta,
    TextMessageEnd,
    TextMessageChunk,
    ToolCallStart,
    ToolCallArgs,
    ToolCallEnd,
    ToolCallResult,
    ToolCallChunk,
    StateSnapshot,
    StateDelta,
    MessagesSnapshot,
    ActivitySnapshot,
    ActivityDelta,
    Error,
    Raw,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiRunStartedEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiRunFinishedEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiRunErrorEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiCustomEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub name: String,
    pub value: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_event: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiStepEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    pub step_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiTextMessageStartEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    pub message_id: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiTextMessageDeltaEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    pub message_id: String,
    pub delta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiTextMessageChunkEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiTextMessageEndEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiToolCallStartEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    pub tool_call_id: String,
    pub tool_call_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_message_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiToolCallArgsEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    pub tool_call_id: String,
    pub delta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiToolCallEndEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    pub tool_call_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiToolCallResultEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    pub tool_call_id: String,
    pub message_id: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiToolCallChunkEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiStateSnapshotEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    pub state: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiStateDeltaEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    pub delta: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiMessagesSnapshotEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    pub messages: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiActivitySnapshotEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    pub message_id: String,
    pub activity_type: String,
    pub content: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiActivityDeltaEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    pub message_id: String,
    pub activity_type: String,
    pub patch: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiErrorEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub thread_id: String,
    pub run_id: String,
    pub message: String,
    pub recoverable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiRawEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub event: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AgUiEvent {
    RunStarted(AgUiRunStartedEvent),
    RunError(AgUiRunErrorEvent),
    StepStarted(AgUiStepEvent),
    StepFinished(AgUiStepEvent),
    TextMessageStart(AgUiTextMessageStartEvent),
    TextMessageContent(AgUiTextMessageDeltaEvent),
    TextMessageDelta(AgUiTextMessageDeltaEvent),
    TextMessageChunk(AgUiTextMessageChunkEvent),
    TextMessageEnd(AgUiTextMessageEndEvent),
    ToolCallStart(AgUiToolCallStartEvent),
    ToolCallArgs(AgUiToolCallArgsEvent),
    ToolCallEnd(AgUiToolCallEndEvent),
    ToolCallResult(AgUiToolCallResultEvent),
    ToolCallChunk(AgUiToolCallChunkEvent),
    StateSnapshot(AgUiStateSnapshotEvent),
    StateDelta(AgUiStateDeltaEvent),
    MessagesSnapshot(AgUiMessagesSnapshotEvent),
    ActivitySnapshot(AgUiActivitySnapshotEvent),
    ActivityDelta(AgUiActivityDeltaEvent),
    Error(AgUiErrorEvent),
    Raw(AgUiRawEvent),
    Custom(AgUiCustomEvent),
    RunFinished(AgUiRunFinishedEvent),
}

pub fn step_started_event(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    step_id: impl Into<String>,
    name: Option<String>,
) -> AgUiEvent {
    AgUiEvent::StepStarted(AgUiStepEvent {
        event_type: AgUiEventType::StepStarted,
        thread_id: thread_id.into(),
        run_id: run_id.into(),
        step_id: step_id.into(),
        name,
    })
}

pub fn step_finished_event(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    step_id: impl Into<String>,
    name: Option<String>,
) -> AgUiEvent {
    AgUiEvent::StepFinished(AgUiStepEvent {
        event_type: AgUiEventType::StepFinished,
        thread_id: thread_id.into(),
        run_id: run_id.into(),
        step_id: step_id.into(),
        name,
    })
}

pub fn text_message_events(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    message_id: impl Into<String>,
    role: impl Into<String>,
    delta: impl Into<String>,
) -> Vec<AgUiEvent> {
    let thread_id = thread_id.into();
    let run_id = run_id.into();
    let message_id = message_id.into();
    let role = role.into();
    let delta = delta.into();

    vec![
        AgUiEvent::TextMessageStart(AgUiTextMessageStartEvent {
            event_type: AgUiEventType::TextMessageStart,
            thread_id: thread_id.clone(),
            run_id: run_id.clone(),
            message_id: message_id.clone(),
            role,
        }),
        AgUiEvent::TextMessageContent(AgUiTextMessageDeltaEvent {
            event_type: AgUiEventType::TextMessageContent,
            thread_id: thread_id.clone(),
            run_id: run_id.clone(),
            message_id: message_id.clone(),
            delta,
        }),
        AgUiEvent::TextMessageEnd(AgUiTextMessageEndEvent {
            event_type: AgUiEventType::TextMessageEnd,
            thread_id,
            run_id,
            message_id,
        }),
    ]
}

pub fn text_message_chunk_event(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    message_id: Option<String>,
    role: Option<String>,
    delta: Option<String>,
) -> AgUiEvent {
    AgUiEvent::TextMessageChunk(AgUiTextMessageChunkEvent {
        event_type: AgUiEventType::TextMessageChunk,
        thread_id: thread_id.into(),
        run_id: run_id.into(),
        message_id,
        role,
        delta,
    })
}

pub fn tool_call_events(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    tool_call_id: impl Into<String>,
    name: impl Into<String>,
    args: Value,
    result: Value,
    is_error: bool,
) -> Vec<AgUiEvent> {
    let thread_id = thread_id.into();
    let run_id = run_id.into();
    let tool_call_id = tool_call_id.into();
    let name = name.into();
    let args_delta = serde_json::to_string(&args).unwrap_or_else(|_| args.to_string());
    let result_content = serde_json::to_string(&if is_error {
        json!({
            "is_error": true,
            "result": result,
        })
    } else {
        result
    })
    .unwrap_or_else(|_| "\"\"".to_string());
    let message_id = format!("msg-{}", tool_call_id);

    vec![
        AgUiEvent::ToolCallStart(AgUiToolCallStartEvent {
            event_type: AgUiEventType::ToolCallStart,
            thread_id: thread_id.clone(),
            run_id: run_id.clone(),
            tool_call_id: tool_call_id.clone(),
            tool_call_name: name,
            parent_message_id: None,
        }),
        AgUiEvent::ToolCallArgs(AgUiToolCallArgsEvent {
            event_type: AgUiEventType::ToolCallArgs,
            thread_id: thread_id.clone(),
            run_id: run_id.clone(),
            tool_call_id: tool_call_id.clone(),
            delta: args_delta,
        }),
        AgUiEvent::ToolCallEnd(AgUiToolCallEndEvent {
            event_type: AgUiEventType::ToolCallEnd,
            thread_id: thread_id.clone(),
            run_id: run_id.clone(),
            tool_call_id: tool_call_id.clone(),
        }),
        AgUiEvent::ToolCallResult(AgUiToolCallResultEvent {
            event_type: AgUiEventType::ToolCallResult,
            thread_id,
            run_id,
            tool_call_id,
            message_id,
            content: result_content,
            role: Some("tool".to_string()),
        }),
    ]
}

pub fn tool_call_chunk_event(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    tool_call_id: Option<String>,
    tool_call_name: Option<String>,
    parent_message_id: Option<String>,
    delta: Option<String>,
) -> AgUiEvent {
    AgUiEvent::ToolCallChunk(AgUiToolCallChunkEvent {
        event_type: AgUiEventType::ToolCallChunk,
        thread_id: thread_id.into(),
        run_id: run_id.into(),
        tool_call_id,
        tool_call_name,
        parent_message_id,
        delta,
    })
}

pub fn state_snapshot_event(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    state: Value,
) -> AgUiEvent {
    AgUiEvent::StateSnapshot(AgUiStateSnapshotEvent {
        event_type: AgUiEventType::StateSnapshot,
        thread_id: thread_id.into(),
        run_id: run_id.into(),
        state,
    })
}

pub fn state_delta_event(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    delta: Value,
) -> AgUiEvent {
    AgUiEvent::StateDelta(AgUiStateDeltaEvent {
        event_type: AgUiEventType::StateDelta,
        thread_id: thread_id.into(),
        run_id: run_id.into(),
        delta,
    })
}

pub fn error_event(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    message: impl Into<String>,
    code: Option<String>,
    recoverable: bool,
) -> AgUiEvent {
    AgUiEvent::Error(AgUiErrorEvent {
        event_type: AgUiEventType::Error,
        thread_id: thread_id.into(),
        run_id: run_id.into(),
        message: message.into(),
        recoverable,
        code,
    })
}

pub fn run_error_event(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    message: impl Into<String>,
    code: Option<String>,
) -> AgUiEvent {
    AgUiEvent::RunError(AgUiRunErrorEvent {
        event_type: AgUiEventType::RunError,
        thread_id: thread_id.into(),
        run_id: run_id.into(),
        message: message.into(),
        code,
    })
}

pub fn messages_snapshot_event(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    messages: Vec<Value>,
) -> AgUiEvent {
    AgUiEvent::MessagesSnapshot(AgUiMessagesSnapshotEvent {
        event_type: AgUiEventType::MessagesSnapshot,
        thread_id: thread_id.into(),
        run_id: run_id.into(),
        messages,
    })
}

pub fn activity_snapshot_event(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    message_id: impl Into<String>,
    activity_type: impl Into<String>,
    content: Value,
    replace: Option<bool>,
) -> AgUiEvent {
    AgUiEvent::ActivitySnapshot(AgUiActivitySnapshotEvent {
        event_type: AgUiEventType::ActivitySnapshot,
        thread_id: thread_id.into(),
        run_id: run_id.into(),
        message_id: message_id.into(),
        activity_type: activity_type.into(),
        content,
        replace,
    })
}

pub fn activity_delta_event(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    message_id: impl Into<String>,
    activity_type: impl Into<String>,
    patch: Value,
) -> AgUiEvent {
    AgUiEvent::ActivityDelta(AgUiActivityDeltaEvent {
        event_type: AgUiEventType::ActivityDelta,
        thread_id: thread_id.into(),
        run_id: run_id.into(),
        message_id: message_id.into(),
        activity_type: activity_type.into(),
        patch,
    })
}

pub fn raw_event(event: Value, source: Option<String>) -> AgUiEvent {
    AgUiEvent::Raw(AgUiRawEvent {
        event_type: AgUiEventType::Raw,
        event,
        source,
    })
}

pub fn surface_to_custom_event(surface: &UiSurface) -> AgUiCustomEvent {
    AgUiCustomEvent {
        event_type: AgUiEventType::Custom,
        name: ADK_UI_SURFACE_EVENT_NAME.to_string(),
        value: json!({
            "format": "adk-ui-surface-v1",
            "surface": surface
        }),
        timestamp: None,
        raw_event: None,
    }
}

pub fn surface_to_event_stream(
    surface: &UiSurface,
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
) -> Vec<AgUiEvent> {
    let thread_id = thread_id.into();
    let run_id = run_id.into();

    vec![
        AgUiEvent::RunStarted(AgUiRunStartedEvent {
            event_type: AgUiEventType::RunStarted,
            thread_id: thread_id.clone(),
            run_id: run_id.clone(),
        }),
        AgUiEvent::Custom(surface_to_custom_event(surface)),
        AgUiEvent::RunFinished(AgUiRunFinishedEvent {
            event_type: AgUiEventType::RunFinished,
            thread_id,
            run_id,
            result: None,
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn surface_custom_event_is_well_formed() {
        let surface = UiSurface::new(
            "main",
            "catalog",
            vec![json!({"id":"root","component":{"Column":{"children":[]}}})],
        );
        let event = surface_to_custom_event(&surface);
        assert_eq!(event.event_type, AgUiEventType::Custom);
        assert_eq!(event.name, ADK_UI_SURFACE_EVENT_NAME);
        assert!(event.value.get("surface").is_some());
    }

    #[test]
    fn event_stream_wraps_custom_event_with_lifecycle() {
        let surface = UiSurface::new(
            "main",
            "catalog",
            vec![json!({"id":"root","component":{"Column":{"children":[]}}})],
        );
        let stream = surface_to_event_stream(&surface, "thread-1", "run-1");
        assert_eq!(stream.len(), 3);

        let first = serde_json::to_value(&stream[0]).unwrap();
        let second = serde_json::to_value(&stream[1]).unwrap();
        let third = serde_json::to_value(&stream[2]).unwrap();

        assert_eq!(first["type"], "RUN_STARTED");
        assert_eq!(second["type"], "CUSTOM");
        assert_eq!(third["type"], "RUN_FINISHED");
    }

    #[test]
    fn text_message_helpers_emit_start_content_end() {
        let events = text_message_events("thread-1", "run-1", "msg-1", "assistant", "hello");
        assert_eq!(events.len(), 3);

        let start = serde_json::to_value(&events[0]).unwrap();
        let content = serde_json::to_value(&events[1]).unwrap();
        let end = serde_json::to_value(&events[2]).unwrap();

        assert_eq!(start["type"], "TEXT_MESSAGE_START");
        assert_eq!(content["type"], "TEXT_MESSAGE_CONTENT");
        assert_eq!(content["delta"], "hello");
        assert_eq!(end["type"], "TEXT_MESSAGE_END");
    }

    #[test]
    fn tool_call_helpers_emit_lifecycle_and_result() {
        let events = tool_call_events(
            "thread-1",
            "run-1",
            "tool-1",
            "lookup_weather",
            json!({"city": "Nairobi"}),
            json!({"temp": 23}),
            false,
        );

        assert_eq!(events.len(), 4);
        let start = serde_json::to_value(&events[0]).unwrap();
        let args = serde_json::to_value(&events[1]).unwrap();
        let end = serde_json::to_value(&events[2]).unwrap();
        let result = serde_json::to_value(&events[3]).unwrap();

        assert_eq!(start["type"], "TOOL_CALL_START");
        assert_eq!(start["toolCallName"], "lookup_weather");
        assert_eq!(args["type"], "TOOL_CALL_ARGS");
        assert_eq!(args["delta"], "{\"city\":\"Nairobi\"}");
        assert_eq!(end["type"], "TOOL_CALL_END");
        assert_eq!(result["type"], "TOOL_CALL_RESULT");
        assert_eq!(result["content"], "{\"temp\":23}");
        assert_eq!(result["messageId"], "msg-tool-1");
        assert_eq!(result["role"], "tool");
    }

    #[test]
    fn state_and_error_helpers_emit_expected_shapes() {
        let snapshot = state_snapshot_event("thread-1", "run-1", json!({"phase": "planning"}));
        let delta = state_delta_event("thread-1", "run-1", json!({"phase": "acting"}));
        let error = error_event(
            "thread-1",
            "run-1",
            "tool timeout",
            Some("TIMEOUT".to_string()),
            true,
        );

        let snapshot_json = serde_json::to_value(snapshot).unwrap();
        let delta_json = serde_json::to_value(delta).unwrap();
        let error_json = serde_json::to_value(error).unwrap();

        assert_eq!(snapshot_json["type"], "STATE_SNAPSHOT");
        assert_eq!(snapshot_json["state"]["phase"], "planning");
        assert_eq!(delta_json["type"], "STATE_DELTA");
        assert_eq!(delta_json["delta"]["phase"], "acting");
        assert_eq!(error_json["type"], "ERROR");
        assert_eq!(error_json["code"], "TIMEOUT");
        assert_eq!(error_json["recoverable"], true);
    }

    #[test]
    fn stable_ag_ui_helper_events_emit_expected_shapes() {
        let run_error = run_error_event("thread-1", "run-1", "boom", Some("FAIL".to_string()));
        let text_chunk = text_message_chunk_event(
            "thread-1",
            "run-1",
            Some("msg-1".to_string()),
            Some("assistant".to_string()),
            Some("partial".to_string()),
        );
        let tool_chunk = tool_call_chunk_event(
            "thread-1",
            "run-1",
            Some("tool-1".to_string()),
            Some("lookup_weather".to_string()),
            Some("msg-1".to_string()),
            Some("{\"city\":\"Nairobi\"}".to_string()),
        );
        let messages_snapshot = messages_snapshot_event(
            "thread-1",
            "run-1",
            vec![json!({"role":"assistant","content":"hello"})],
        );
        let activity_snapshot = activity_snapshot_event(
            "thread-1",
            "run-1",
            "activity-1",
            "PLAN",
            json!({"steps":[{"title":"Research"}]}),
            Some(true),
        );
        let activity_delta = activity_delta_event(
            "thread-1",
            "run-1",
            "activity-1",
            "PLAN",
            json!([{"op":"add","path":"/steps/1","value":{"title":"Implement"}}]),
        );
        let raw = raw_event(
            json!({"source":"legacy"}),
            Some("legacy-system".to_string()),
        );

        let run_error_json = serde_json::to_value(run_error).unwrap();
        let text_chunk_json = serde_json::to_value(text_chunk).unwrap();
        let tool_chunk_json = serde_json::to_value(tool_chunk).unwrap();
        let messages_snapshot_json = serde_json::to_value(messages_snapshot).unwrap();
        let activity_snapshot_json = serde_json::to_value(activity_snapshot).unwrap();
        let activity_delta_json = serde_json::to_value(activity_delta).unwrap();
        let raw_json = serde_json::to_value(raw).unwrap();

        assert_eq!(run_error_json["type"], "RUN_ERROR");
        assert_eq!(run_error_json["message"], "boom");
        assert_eq!(text_chunk_json["type"], "TEXT_MESSAGE_CHUNK");
        assert_eq!(tool_chunk_json["type"], "TOOL_CALL_CHUNK");
        assert_eq!(messages_snapshot_json["type"], "MESSAGES_SNAPSHOT");
        assert_eq!(activity_snapshot_json["type"], "ACTIVITY_SNAPSHOT");
        assert_eq!(activity_delta_json["type"], "ACTIVITY_DELTA");
        assert_eq!(raw_json["type"], "RAW");
    }
}
