use super::surface::UiSurface;
use crate::notifications::UiNotification;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Event name used for surface payload transport via AG-UI custom events.
pub const ADK_UI_SURFACE_EVENT_NAME: &str = "adk.ui.surface";

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgUiIdentityCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgUiTransportCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub websocket: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_binary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_notifications: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resumable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgUiToolsCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_provided: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgUiOutputCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured_output: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_mime_types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgUiStateCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshots: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deltas: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistent_state: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgUiSubAgentInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgUiMultiAgentCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handoffs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_agents: Option<Vec<AgUiSubAgentInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgUiReasoningCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgUiMultimodalInputCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgUiMultimodalOutputCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgUiMultimodalCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<AgUiMultimodalInputCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<AgUiMultimodalOutputCapabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgUiExecutionCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_execution: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandboxed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_iterations: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_execution_time: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgUiHumanInTheLoopCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approvals: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interventions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approve_with_edits: Option<bool>,
}

/// Current AG-UI capability snapshot. All fields are optional by design:
/// absence means unknown rather than unsupported.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgUiAgentCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<AgUiIdentityCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<AgUiTransportCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<AgUiToolsCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<AgUiOutputCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<AgUiStateCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_agent: Option<AgUiMultiAgentCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<AgUiReasoningCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multimodal: Option<AgUiMultimodalCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution: Option<AgUiExecutionCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub human_in_the_loop: Option<AgUiHumanInTheLoopCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<Value>,
}

pub fn default_ag_ui_agent_capabilities() -> AgUiAgentCapabilities {
    AgUiAgentCapabilities {
        identity: Some(AgUiIdentityCapabilities {
            name: Some("ADK UI example agent".to_string()),
            agent_type: Some("adk-rust".to_string()),
            description: Some("Protocol-aware generative UI agent".to_string()),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            provider: Some("Zavora AI".to_string()),
            documentation_url: Some("https://github.com/zavora-ai/adk-ui".to_string()),
            metadata: None,
        }),
        transport: Some(AgUiTransportCapabilities {
            streaming: Some(true),
            websocket: Some(false),
            http_binary: Some(false),
            push_notifications: Some(true),
            resumable: Some(false),
        }),
        tools: Some(AgUiToolsCapabilities {
            supported: Some(true),
            items: None,
            parallel_calls: None,
            client_provided: Some(false),
        }),
        output: Some(AgUiOutputCapabilities {
            structured_output: Some(true),
            supported_mime_types: Some(vec![
                "text/event-stream".to_string(),
                "application/json".to_string(),
            ]),
        }),
        state: Some(AgUiStateCapabilities {
            snapshots: Some(true),
            deltas: Some(true),
            memory: None,
            persistent_state: Some(true),
        }),
        reasoning: Some(AgUiReasoningCapabilities {
            supported: Some(true),
            streaming: Some(true),
            encrypted: Some(false),
        }),
        human_in_the_loop: Some(AgUiHumanInTheLoopCapabilities {
            supported: Some(true),
            approvals: None,
            interventions: None,
            feedback: Some(true),
            interrupts: Some(false),
            approve_with_edits: Some(false),
        }),
        custom: Some(json!({
            "adkUiSurfaceEvents": ["ACTIVITY_SNAPSHOT", "CUSTOM"],
            "interruptWireTypes": true,
            "durableInterruptRuntime": false,
            "adkUiNotificationChannel": true,
        })),
        ..Default::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgUiInterrupt {
    pub id: String,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_schema: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgUiResumeStatus {
    Resolved,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgUiResumeEntry {
    pub interrupt_id: String,
    pub status: AgUiResumeStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgUiRunAgentInput {
    pub thread_id: String,
    pub run_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_run_id: Option<String>,
    pub state: Value,
    pub messages: Vec<Value>,
    pub tools: Vec<Value>,
    pub context: Vec<Value>,
    pub forwarded_props: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume: Option<Vec<AgUiResumeEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AgUiRunFinishedOutcome {
    Success,
    Interrupt { interrupts: Vec<AgUiInterrupt> },
}

/// AG-UI event types from the protocol event model (current stable + reasoning).
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
    ReasoningStart,
    ReasoningMessageStart,
    ReasoningMessageContent,
    ReasoningMessageEnd,
    ReasoningMessageChunk,
    ReasoningEnd,
    ReasoningEncryptedValue,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<Value>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<AgUiRunFinishedOutcome>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiRunErrorEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Optional lineage fields retained for multi-run hosts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
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
    /// Official AG-UI field (`stepName`).
    pub step_name: String,
    /// Optional run lineage for multi-thread hosts (not required by core AG-UI).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
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
    /// Official AG-UI field name.
    pub snapshot: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiStateDeltaEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    /// JSON Patch operations (RFC 6902).
    pub delta: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
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
    pub message_id: String,
    pub activity_type: String,
    pub content: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiActivityDeltaEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub message_id: String,
    pub activity_type: String,
    pub patch: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiReasoningStartEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiReasoningMessageStartEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub message_id: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiReasoningMessageContentEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub message_id: String,
    pub delta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiReasoningMessageEndEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiReasoningMessageChunkEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiReasoningEndEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiReasoningEncryptedValueEvent {
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    pub subtype: String,
    pub entity_id: String,
    pub encrypted_value: String,
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
    ReasoningStart(AgUiReasoningStartEvent),
    ReasoningMessageStart(AgUiReasoningMessageStartEvent),
    ReasoningMessageContent(AgUiReasoningMessageContentEvent),
    ReasoningMessageEnd(AgUiReasoningMessageEndEvent),
    ReasoningMessageChunk(AgUiReasoningMessageChunkEvent),
    ReasoningEnd(AgUiReasoningEndEvent),
    ReasoningEncryptedValue(AgUiReasoningEncryptedValueEvent),
    Error(AgUiErrorEvent),
    Raw(AgUiRawEvent),
    Custom(AgUiCustomEvent),
    RunFinished(AgUiRunFinishedEvent),
}

pub fn run_started_event(thread_id: impl Into<String>, run_id: impl Into<String>) -> AgUiEvent {
    AgUiEvent::RunStarted(AgUiRunStartedEvent {
        event_type: AgUiEventType::RunStarted,
        thread_id: thread_id.into(),
        run_id: run_id.into(),
        parent_run_id: None,
        input: None,
    })
}

pub fn run_finished_event(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    result: Option<Value>,
) -> AgUiEvent {
    AgUiEvent::RunFinished(AgUiRunFinishedEvent {
        event_type: AgUiEventType::RunFinished,
        thread_id: thread_id.into(),
        run_id: run_id.into(),
        result,
        outcome: None,
    })
}

pub fn run_interrupted_event(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    interrupts: Vec<AgUiInterrupt>,
) -> AgUiEvent {
    AgUiEvent::RunFinished(AgUiRunFinishedEvent {
        event_type: AgUiEventType::RunFinished,
        thread_id: thread_id.into(),
        run_id: run_id.into(),
        result: None,
        outcome: Some(AgUiRunFinishedOutcome::Interrupt { interrupts }),
    })
}

pub fn step_started_event(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    step_id: impl Into<String>,
    name: Option<String>,
) -> AgUiEvent {
    let step_id = step_id.into();
    let step_name = name.unwrap_or_else(|| step_id.clone());
    AgUiEvent::StepStarted(AgUiStepEvent {
        event_type: AgUiEventType::StepStarted,
        step_name,
        thread_id: Some(thread_id.into()),
        run_id: Some(run_id.into()),
        step_id: Some(step_id),
    })
}

pub fn step_finished_event(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    step_id: impl Into<String>,
    name: Option<String>,
) -> AgUiEvent {
    let step_id = step_id.into();
    let step_name = name.unwrap_or_else(|| step_id.clone());
    AgUiEvent::StepFinished(AgUiStepEvent {
        event_type: AgUiEventType::StepFinished,
        step_name,
        thread_id: Some(thread_id.into()),
        run_id: Some(run_id.into()),
        step_id: Some(step_id),
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
        snapshot: state,
        thread_id: Some(thread_id.into()),
        run_id: Some(run_id.into()),
    })
}

pub fn state_delta_event(
    thread_id: impl Into<String>,
    run_id: impl Into<String>,
    delta: Value,
) -> AgUiEvent {
    AgUiEvent::StateDelta(AgUiStateDeltaEvent {
        event_type: AgUiEventType::StateDelta,
        delta,
        thread_id: Some(thread_id.into()),
        run_id: Some(run_id.into()),
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
        message: message.into(),
        code,
        thread_id: Some(thread_id.into()),
        run_id: Some(run_id.into()),
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
        message_id: message_id.into(),
        activity_type: activity_type.into(),
        content,
        replace,
        thread_id: Some(thread_id.into()),
        run_id: Some(run_id.into()),
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
        message_id: message_id.into(),
        activity_type: activity_type.into(),
        patch,
        thread_id: Some(thread_id.into()),
        run_id: Some(run_id.into()),
    })
}

/// Emit a full reasoning message triad (start → content → end).
pub fn reasoning_message_events(
    message_id: impl Into<String>,
    delta: impl Into<String>,
) -> Vec<AgUiEvent> {
    let message_id = message_id.into();
    let delta = delta.into();
    vec![
        AgUiEvent::ReasoningStart(AgUiReasoningStartEvent {
            event_type: AgUiEventType::ReasoningStart,
            message_id: message_id.clone(),
        }),
        AgUiEvent::ReasoningMessageStart(AgUiReasoningMessageStartEvent {
            event_type: AgUiEventType::ReasoningMessageStart,
            message_id: message_id.clone(),
            role: "reasoning".to_string(),
        }),
        AgUiEvent::ReasoningMessageContent(AgUiReasoningMessageContentEvent {
            event_type: AgUiEventType::ReasoningMessageContent,
            message_id: message_id.clone(),
            delta,
        }),
        AgUiEvent::ReasoningMessageEnd(AgUiReasoningMessageEndEvent {
            event_type: AgUiEventType::ReasoningMessageEnd,
            message_id: message_id.clone(),
        }),
        AgUiEvent::ReasoningEnd(AgUiReasoningEndEvent {
            event_type: AgUiEventType::ReasoningEnd,
            message_id,
        }),
    ]
}

pub fn reasoning_message_chunk_event(
    message_id: Option<String>,
    delta: Option<String>,
) -> AgUiEvent {
    AgUiEvent::ReasoningMessageChunk(AgUiReasoningMessageChunkEvent {
        event_type: AgUiEventType::ReasoningMessageChunk,
        message_id,
        delta,
    })
}

pub fn raw_event(event: Value, source: Option<String>) -> AgUiEvent {
    AgUiEvent::Raw(AgUiRawEvent {
        event_type: AgUiEventType::Raw,
        event,
        source,
    })
}

/// Convert an out-of-turn UI notification into an AG-UI custom event.
pub fn notification_event(notification: &UiNotification) -> AgUiEvent {
    AgUiEvent::Custom(AgUiCustomEvent {
        event_type: AgUiEventType::Custom,
        name: "adk.ui.notification".to_string(),
        value: serde_json::to_value(notification).unwrap_or(Value::Null),
        timestamp: None,
        raw_event: None,
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
            parent_run_id: None,
            input: None,
        }),
        // Prefer native activity for surface lifecycle; keep CUSTOM for legacy clients.
        activity_snapshot_event(
            thread_id.clone(),
            run_id.clone(),
            format!("activity-{}", surface.surface_id),
            "ADK_UI_SURFACE",
            json!({
                "format": "adk-ui-surface-v1",
                "surface": surface,
            }),
            Some(true),
        ),
        AgUiEvent::Custom(surface_to_custom_event(surface)),
        AgUiEvent::RunFinished(AgUiRunFinishedEvent {
            event_type: AgUiEventType::RunFinished,
            thread_id,
            run_id,
            result: None,
            outcome: Some(AgUiRunFinishedOutcome::Success),
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
        assert_eq!(stream.len(), 4);

        let first = serde_json::to_value(&stream[0]).unwrap();
        let second = serde_json::to_value(&stream[1]).unwrap();
        let third = serde_json::to_value(&stream[2]).unwrap();
        let fourth = serde_json::to_value(&stream[3]).unwrap();

        assert_eq!(first["type"], "RUN_STARTED");
        assert_eq!(second["type"], "ACTIVITY_SNAPSHOT");
        assert_eq!(second["activityType"], "ADK_UI_SURFACE");
        assert_eq!(third["type"], "CUSTOM");
        assert_eq!(fourth["type"], "RUN_FINISHED");
    }

    #[test]
    fn state_snapshot_uses_official_snapshot_field() {
        let event = state_snapshot_event("t", "r", json!({"ui": "ok"}));
        let value = serde_json::to_value(event).unwrap();
        assert_eq!(value["type"], "STATE_SNAPSHOT");
        assert_eq!(value["snapshot"]["ui"], "ok");
        assert!(value.get("state").is_none());
    }

    #[test]
    fn reasoning_helpers_emit_official_event_types() {
        let events = reasoning_message_events("msg-r1", "thinking...");
        let types: Vec<_> = events
            .iter()
            .map(|e| {
                serde_json::to_value(e).unwrap()["type"]
                    .as_str()
                    .unwrap()
                    .to_string()
            })
            .collect();
        assert!(types.contains(&"REASONING_START".to_string()));
        assert!(types.contains(&"REASONING_MESSAGE_CONTENT".to_string()));
        assert!(types.contains(&"REASONING_END".to_string()));
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
        assert_eq!(snapshot_json["snapshot"]["phase"], "planning");
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

    #[test]
    fn capability_snapshot_uses_current_typed_categories() {
        let capabilities = serde_json::to_value(default_ag_ui_agent_capabilities()).unwrap();

        assert_eq!(capabilities["transport"]["streaming"], true);
        assert_eq!(capabilities["transport"]["resumable"], false);
        assert_eq!(capabilities["state"]["snapshots"], true);
        assert_eq!(capabilities["reasoning"]["encrypted"], false);
        assert_eq!(capabilities["humanInTheLoop"]["interrupts"], false);
        assert_eq!(capabilities["custom"]["interruptWireTypes"], true);
    }

    #[test]
    fn interrupt_outcome_and_resume_entry_match_ag_ui_wire_shape() {
        let event = run_interrupted_event(
            "thread-1",
            "run-1",
            vec![AgUiInterrupt {
                id: "interrupt-1".to_string(),
                reason: "confirmation".to_string(),
                message: Some("Continue?".to_string()),
                tool_call_id: None,
                response_schema: Some(json!({"type":"boolean"})),
                expires_at: None,
                metadata: None,
            }],
        );
        let event_json = serde_json::to_value(event).unwrap();
        assert_eq!(event_json["type"], "RUN_FINISHED");
        assert_eq!(event_json["outcome"]["type"], "interrupt");
        assert_eq!(event_json["outcome"]["interrupts"][0]["id"], "interrupt-1");

        let resume: AgUiResumeEntry = serde_json::from_value(json!({
            "interruptId": "interrupt-1",
            "status": "resolved",
            "payload": {"approved": true}
        }))
        .unwrap();
        assert_eq!(resume.status, AgUiResumeStatus::Resolved);
    }
}
