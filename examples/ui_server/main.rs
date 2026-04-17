use adk_agent::LlmAgentBuilder;
use adk_core::{Agent, Content, Event, FunctionResponseData, MultiAgentLoader, Part, SessionId, Tool, UserId};
use adk_model::gemini::GeminiModel;
use adk_runner::{Runner, RunnerConfig};
use adk_server::{ServerConfig, create_app, shutdown_signal};
use adk_session::{CreateRequest, GetRequest, InMemorySessionService, SessionService};
use adk_ui::{
    MCP_APPS_PROTOCOL_VERSION, McpAppsAppCapabilities, McpAppsInitializeRequest,
    McpAppsInitializeRequestParams, McpAppsInitializeResult, McpAppsPartyInfo,
    SUPPORTED_UI_PROTOCOLS, UI_PROTOCOL_CAPABILITIES, UiToolset, a2ui::A2UI_AGENT_PROMPT,
    build_default_mcp_apps_initialize_result, normalize_runtime_ui_protocol,
};
use anyhow::Result;
use axum::{
    body::{Body, to_bytes},
    extract::State,
    http::{HeaderMap, HeaderValue, Method, Request, StatusCode, header},
    middleware::{self, Next},
    response::{
        IntoResponse, Response,
        sse::{Event as SseEvent, KeepAlive, Sse},
    },
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use futures::{StreamExt, stream};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    convert::Infallible,
    sync::{Arc, OnceLock, RwLock},
};
use tracing::{info, warn};

const UI_DEMO_INSTRUCTION: &str = r#"
You are a general-purpose UI assistant.

Render clear, production-style interfaces for dashboards, forms, tables, charts, alerts, and modals.
Prefer high-level tools (`render_layout`, `render_table`, `render_chart`, `render_form`, `render_card`) over plain text-only outputs.
For dashboard prompts, include at minimum:
- 3 KPI cards
- 1 table or list
- 1 chart
- 1 alert or status badge cluster
Do not satisfy dashboard prompts using text paragraphs plus buttons only.
Always return complete surfaces with stable ids and actionable controls.
"#;

const SUPPORT_INSTRUCTION: &str = r#"
You are a support intake assistant.

When the user starts, immediately render a support ticket form with:
- Title input
- Description textarea
- Priority select (Low, Medium, High)
- Submit button

Use `render_form` for intake and `render_alert` for success/error feedback.
Use render_screen with a root Column layout.
"#;

const APPOINTMENT_INSTRUCTION: &str = r#"
You are a clinic scheduling assistant that renders working UIs.

Use render_layout for overviews and render_page for supporting sections (services, hours, policies).
Use render_card for service options and render_table for schedule availability.
Use render_screen for booking flows and ensure:
- root component id "root"
- layout with Column/Row
- Button actions include action.event.name

After a booking submission, render a confirmation screen with the appointment details.
"#;

const EVENTS_INSTRUCTION: &str = r#"
You are an event RSVP assistant with working UI flows.

Always use render_table for agenda timeline rows with columns: time, session, speaker, room.
Always use render_card for featured speakers with non-empty title and description fields.
Use render_layout for page structure and venue summary sections.
Use render_screen to collect RSVP details (name, guests, dietary, sessions).
Ensure A2UI components include root id "root" and valid Button actions.
Do not compress agenda data into plain text lines.

After submission, render a confirmation screen and a calendar link button.
"#;

const FACILITIES_INSTRUCTION: &str = r#"
You are a facilities maintenance assistant.

Use render_layout for command-center style screens with alerts, KPI cards, and queues.
Use render_table for work-order lists and render_confirm for high-risk actions.
Use render_screen to intake work orders (location, issue type, urgency, contact).
Use render_page for maintenance guidelines or status summaries.
Ensure A2UI components include root id "root".

After intake, render a confirmation with next steps and an emergency contact action.
"#;

const INVENTORY_INSTRUCTION: &str = r#"
You are an inventory restock assistant.

Use render_layout for inventory command views and render_table for stock/reorder grids.
Use render_chart for trend/forecast visuals and render_card for supplier summaries.
Inventory monitor responses must include:
- a stock table with SKU, quantity, threshold, and status
- at least one alert
- at least one chart or summary card row
Use render_screen to collect restock requests (SKU, qty, priority, notes).
Use render_page for inventory summaries and reorder recommendations.
Ensure A2UI components include a root id "root" and explicit child ids.

On submit, show a confirmation card or alert with the request summary.
"#;

const UI_PROTOCOL_HEADER: &str = "x-adk-ui-protocol";
const MAX_REQUEST_BODY_BYTES: usize = 10 * 1024 * 1024;

type RuntimeError = (StatusCode, String);

#[derive(Clone)]
struct ExampleRuntimeState {
    config: ServerConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UiProfile {
    AdkUi,
    A2ui,
    AgUi,
    McpApps,
}

#[derive(Debug, Clone)]
struct McpAppsNegotiationState {
    initialize_request: Option<McpAppsInitializeRequest>,
    initialize_result: McpAppsInitializeResult,
    initialized: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct McpAppsBridgeSessionKey {
    app_name: String,
    user_id: String,
    session_id: String,
}

#[derive(Debug, Clone)]
struct McpAppsBridgeSessionEntry {
    initialize_request: Option<McpAppsInitializeRequest>,
    initialize_result: McpAppsInitializeResult,
    initialized: bool,
    message_count: u64,
    last_message: Option<Value>,
    model_context: Vec<Value>,
    model_context_revision: u64,
}

static MCP_APPS_BRIDGE_REGISTRY: OnceLock<
    RwLock<HashMap<McpAppsBridgeSessionKey, McpAppsBridgeSessionEntry>>,
> = OnceLock::new();

impl UiProfile {
    fn as_str(self) -> &'static str {
        match self {
            Self::AdkUi => "adk_ui",
            Self::A2ui => "a2ui",
            Self::AgUi => "ag_ui",
            Self::McpApps => "mcp_apps",
        }
    }
}

fn mcp_apps_bridge_registry()
-> &'static RwLock<HashMap<McpAppsBridgeSessionKey, McpAppsBridgeSessionEntry>> {
    MCP_APPS_BRIDGE_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

fn mcp_apps_bridge_key(app_name: &str, user_id: &str, session_id: &str) -> McpAppsBridgeSessionKey {
    McpAppsBridgeSessionKey {
        app_name: app_name.to_string(),
        user_id: user_id.to_string(),
        session_id: session_id.to_string(),
    }
}

fn ensure_mcp_apps_bridge_session<'a>(
    registry: &'a mut HashMap<McpAppsBridgeSessionKey, McpAppsBridgeSessionEntry>,
    app_name: &str,
    user_id: &str,
    session_id: &str,
) -> &'a mut McpAppsBridgeSessionEntry {
    registry
        .entry(mcp_apps_bridge_key(app_name, user_id, session_id))
        .or_insert_with(|| McpAppsBridgeSessionEntry {
            initialize_request: None,
            initialize_result: build_default_mcp_apps_initialize_result(
                "ui://adk-ui/main",
                Some("https://example.com"),
            ),
            initialized: false,
            message_count: 0,
            last_message: None,
            model_context: vec![],
            model_context_revision: 0,
        })
}

fn negotiation_from_bridge_entry(entry: &McpAppsBridgeSessionEntry) -> McpAppsNegotiationState {
    McpAppsNegotiationState {
        initialize_request: entry.initialize_request.clone(),
        initialize_result: entry.initialize_result.clone(),
        initialized: entry.initialized,
    }
}

fn full_instruction(extra: &str) -> String {
    format!("{A2UI_AGENT_PROMPT}\n\n{extra}")
}

fn build_ui_agent(
    name: &str,
    description: &str,
    instruction: &str,
    api_key: &str,
    model_name: &str,
    ui_tools: &[Arc<dyn Tool>],
) -> Result<Arc<dyn Agent>> {
    let mut builder = LlmAgentBuilder::new(name)
        .description(description)
        .instruction(full_instruction(instruction))
        .model(Arc::new(GeminiModel::new(api_key, model_name)?));

    for tool in ui_tools.iter().cloned() {
        builder = builder.tool(tool);
    }

    Ok(Arc::new(builder.build()?))
}

fn parse_ui_profile(raw: &str) -> Option<UiProfile> {
    match normalize_runtime_ui_protocol(raw)? {
        "adk_ui" => Some(UiProfile::AdkUi),
        "a2ui" => Some(UiProfile::A2ui),
        "ag_ui" => Some(UiProfile::AgUi),
        "mcp_apps" => Some(UiProfile::McpApps),
        _ => None,
    }
}

fn pick_value<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    let object = value.as_object()?;
    keys.iter().find_map(|key| object.get(*key))
}

fn pick_string(value: &Value, keys: &[&str]) -> Option<String> {
    pick_value(value, keys)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn bridge_body(value: &Value) -> &Value {
    pick_value(value, &["params"]).unwrap_or(value)
}

fn bridge_identity(body: &Value) -> Result<(String, String, String), RuntimeError> {
    let params = bridge_body(body);
    let app_name = pick_string(params, &["appName", "app_name"])
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "appName is required".to_string()))?;
    let user_id = pick_string(params, &["userId", "user_id"])
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "userId is required".to_string()))?;
    let session_id = pick_string(params, &["sessionId", "session_id"])
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "sessionId is required".to_string()))?;
    Ok((app_name, user_id, session_id))
}

fn resolve_ui_profile(headers: &HeaderMap, body: &Value) -> Result<UiProfile, RuntimeError> {
    let header_value = headers
        .get(UI_PROTOCOL_HEADER)
        .and_then(|value| value.to_str().ok());
    let body_value = pick_value(body, &["uiProtocol", "ui_protocol"]).and_then(Value::as_str);
    let candidate = header_value.or(body_value);

    let Some(raw) = candidate else {
        return Ok(UiProfile::AdkUi);
    };

    parse_ui_profile(raw).ok_or_else(|| {
        warn!(
            requested = %raw,
            header = %UI_PROTOCOL_HEADER,
            "unsupported ui protocol requested",
        );
        (
            StatusCode::BAD_REQUEST,
            format!(
                "Unsupported ui protocol '{}'. Supported profiles: {}",
                raw,
                SUPPORTED_UI_PROTOCOLS.join(", "),
            ),
        )
    })
}

fn serialize_runtime_event(event: &Event, profile: UiProfile) -> Option<String> {
    if profile == UiProfile::McpApps {
        return serialize_runtime_event_with_mcp_bridge(event, profile, None);
    }

    if profile == UiProfile::AdkUi {
        return serde_json::to_string(event).ok();
    }

    serde_json::to_string(&json!({
        "ui_protocol": profile.as_str(),
        "event": event,
    }))
    .ok()
}

fn merge_json(target: &mut Value, overlay: &Value) {
    match (target, overlay) {
        (Value::Object(target_map), Value::Object(overlay_map)) => {
            for (key, value) in overlay_map {
                if let Some(existing) = target_map.get_mut(key) {
                    merge_json(existing, value);
                } else {
                    target_map.insert(key.clone(), value.clone());
                }
            }
        }
        (target_slot, overlay_value) => {
            *target_slot = overlay_value.clone();
        }
    }
}

fn looks_like_mcp_apps_payload(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };

    object.contains_key("bridge")
        || object.contains_key("resourceReadResponse")
        || object.contains_key("resource_read_response")
        || object.contains_key("toolResult")
        || object.contains_key("tool_result")
        || object.contains_key("toolMeta")
        || object.contains_key("tool_meta")
}

fn enrich_mcp_apps_payload(value: &mut Value, negotiation: &McpAppsNegotiationState) {
    match value {
        Value::Array(entries) => {
            for entry in entries {
                enrich_mcp_apps_payload(entry, negotiation);
            }
        }
        Value::Object(object) => {
            for nested in object.values_mut() {
                enrich_mcp_apps_payload(nested, negotiation);
            }

            if !looks_like_mcp_apps_payload(&Value::Object(object.clone())) {
                return;
            }

            let bridge = object
                .entry("bridge".to_string())
                .or_insert_with(|| Value::Object(Default::default()));
            if !bridge.is_object() {
                *bridge = Value::Object(Default::default());
            }

            let bridge_object = bridge.as_object_mut().expect("bridge object");
            bridge_object.insert(
                "protocolVersion".to_string(),
                json!(negotiation.initialize_result.protocol_version),
            );
            bridge_object.insert(
                "hostInfo".to_string(),
                serde_json::to_value(&negotiation.initialize_result.host_info)
                    .unwrap_or_else(|_| json!({})),
            );
            bridge_object.insert(
                "hostCapabilities".to_string(),
                negotiation.initialize_result.host_capabilities.clone(),
            );

            let mut host_context = negotiation.initialize_result.host_context.clone();
            if let Some(existing_host_context) = bridge_object.get("hostContext") {
                merge_json(&mut host_context, existing_host_context);
            }
            bridge_object.insert("hostContext".to_string(), host_context);

            if let Some(initialize_request) = &negotiation.initialize_request {
                bridge_object.insert(
                    "appInfo".to_string(),
                    serde_json::to_value(&initialize_request.params.app_info)
                        .unwrap_or_else(|_| json!({})),
                );
                let mut app_capabilities = bridge_object
                    .get("appCapabilities")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                let request_capabilities =
                    serde_json::to_value(&initialize_request.params.app_capabilities)
                        .unwrap_or_else(|_| json!({}));
                merge_json(&mut app_capabilities, &request_capabilities);
                bridge_object.insert("appCapabilities".to_string(), app_capabilities);
            }

            bridge_object.insert("initialized".to_string(), json!(negotiation.initialized));
        }
        _ => {}
    }
}

fn serialize_runtime_event_with_mcp_bridge(
    event: &Event,
    profile: UiProfile,
    negotiation: Option<&McpAppsNegotiationState>,
) -> Option<String> {
    let mut serialized = serde_json::to_value(event).ok()?;
    if let Some(negotiation) = negotiation {
        enrich_mcp_apps_payload(&mut serialized, negotiation);
    }

    if profile == UiProfile::AdkUi {
        return serde_json::to_string(&serialized).ok();
    }

    serde_json::to_string(&json!({
        "ui_protocol": profile.as_str(),
        "event": serialized,
    }))
    .ok()
}

fn log_profile_deprecation(profile: UiProfile) {
    if profile != UiProfile::AdkUi {
        return;
    }

    let Some(spec) = UI_PROTOCOL_CAPABILITIES
        .iter()
        .find(|capability| capability.protocol == profile.as_str())
        .and_then(|capability| capability.deprecation)
    else {
        return;
    };

    warn!(
        protocol = %profile.as_str(),
        stage = %spec.stage,
        announced_on = %spec.announced_on,
        sunset_target_on = ?spec.sunset_target_on,
        replacements = ?spec.replacement_protocols,
        "legacy ui protocol profile selected",
    );
}

fn parse_inline_data(part: &Value) -> Result<Option<Part>, RuntimeError> {
    let Some(inline_data) = pick_value(part, &["inlineData", "inline_data"]) else {
        return Ok(None);
    };
    let Some(data) = pick_string(inline_data, &["data"]) else {
        return Err((
            StatusCode::BAD_REQUEST,
            "inlineData.data is required".to_string(),
        ));
    };
    let Some(mime_type) = pick_string(inline_data, &["mimeType", "mime_type"]) else {
        return Err((
            StatusCode::BAD_REQUEST,
            "inlineData.mimeType is required".to_string(),
        ));
    };

    let decoded = BASE64_STANDARD.decode(data).map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid base64 data in inlineData: {}", error),
        )
    })?;

    if decoded.len() > adk_core::MAX_INLINE_DATA_SIZE {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "inlineData exceeds max inline size of {} bytes",
                adk_core::MAX_INLINE_DATA_SIZE,
            ),
        ));
    }

    Ok(Some(Part::InlineData {
        mime_type,
        data: decoded,
    }))
}

fn parse_function_call_part(part: &Value) -> Result<Option<Part>, RuntimeError> {
    let Some(function_call) = pick_value(part, &["functionCall", "function_call"]) else {
        return Ok(None);
    };
    let Some(name) = pick_string(function_call, &["name"]) else {
        return Err((
            StatusCode::BAD_REQUEST,
            "functionCall.name is required".to_string(),
        ));
    };

    Ok(Some(Part::FunctionCall {
        name,
        args: pick_value(function_call, &["args", "arguments"])
            .cloned()
            .unwrap_or(Value::Null),
        id: pick_string(function_call, &["id"]),
        thought_signature: pick_string(function_call, &["thoughtSignature", "thought_signature"]),
    }))
}

fn parse_function_response_part(part: &Value) -> Result<Option<Part>, RuntimeError> {
    let Some(function_response) = pick_value(part, &["functionResponse", "function_response"])
    else {
        return Ok(None);
    };
    let payload = pick_value(
        function_response,
        &["functionResponse", "function_response"],
    )
    .unwrap_or(function_response);
    let Some(name) = pick_string(payload, &["name"]) else {
        return Err((
            StatusCode::BAD_REQUEST,
            "functionResponse.name is required".to_string(),
        ));
    };

    Ok(Some(Part::FunctionResponse {
        function_response: FunctionResponseData {
            name,
            response: pick_value(payload, &["response", "result", "data", "payload"])
                .cloned()
                .unwrap_or(Value::Null),
            file_data: vec![],
            inline_data: vec![],
        },
        id: pick_string(function_response, &["id"]),
    }))
}

fn parse_message_parts(parts: &[Value]) -> Result<Vec<Part>, RuntimeError> {
    let mut parsed = Vec::new();

    for part in parts {
        if let Some(text) = pick_string(part, &["text"]) {
            parsed.push(Part::Text { text });
            continue;
        }

        if let Some(inline_data) = parse_inline_data(part)? {
            parsed.push(inline_data);
            continue;
        }

        if let Some(function_call) = parse_function_call_part(part)? {
            parsed.push(function_call);
            continue;
        }

        if let Some(function_response) = parse_function_response_part(part)? {
            parsed.push(function_response);
        }
    }

    Ok(parsed)
}

fn extract_text_snippets(value: &Value) -> Vec<String> {
    match value {
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                Vec::new()
            } else {
                vec![trimmed.to_string()]
            }
        }
        Value::Array(entries) => entries.iter().flat_map(extract_text_snippets).collect(),
        Value::Object(_) => {
            if let Some(text) = pick_string(value, &["text"]) {
                return vec![text];
            }
            if let Some(content) = pick_value(value, &["content"]) {
                return extract_text_snippets(content);
            }
            Vec::new()
        }
        _ => Vec::new(),
    }
}

fn ag_ui_input_value<'a>(body: &'a Value) -> Option<&'a Value> {
    pick_value(body, &["input", "agUiInput", "ag_ui_input"])
}

fn ag_ui_activity_id(message: &Value, index: usize) -> String {
    pick_string(message, &["id"])
        .unwrap_or_else(|| format!("activity-{}-{}", chrono::Utc::now().timestamp_millis(), index))
}

fn ag_ui_activity_type(message: &Value) -> String {
    pick_string(message, &["activityType", "activity_type", "name"])
        .unwrap_or_else(|| "CUSTOM".to_string())
}

fn ag_ui_messages_snapshot_value(body: &Value) -> Option<Value> {
    let messages = ag_ui_input_value(body)
        .and_then(|input| pick_value(input, &["messages"]))
        .and_then(Value::as_array)?;

    let filtered: Vec<Value> = messages
        .iter()
        .filter(|message| {
            !(pick_string(message, &["role"]).as_deref() == Some("activity")
                && pick_value(message, &["patch"]).is_some())
        })
        .cloned()
        .collect();
    if filtered.is_empty() {
        return None;
    }

    Some(Value::Array(filtered))
}

fn ag_ui_activity_events(body: &Value) -> Vec<Value> {
    let Some(messages) = ag_ui_input_value(body)
        .and_then(|input| pick_value(input, &["messages"]))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };

    messages
        .iter()
        .enumerate()
        .filter(|(_, message)| pick_string(message, &["role"]).as_deref() == Some("activity"))
        .map(|(index, message)| {
            let message_id = ag_ui_activity_id(message, index);
            let activity_type = ag_ui_activity_type(message);
            let timestamp = chrono::Utc::now().timestamp_millis().max(0);

            if let Some(patch) = pick_value(message, &["patch"]).cloned() {
                json!({
                    "type": "ACTIVITY_DELTA",
                    "messageId": message_id,
                    "activityType": activity_type,
                    "patch": patch,
                    "timestamp": timestamp,
                })
            } else {
                let content = pick_value(message, &["content"]).cloned().unwrap_or_else(|| json!({}));
                let content = match content {
                    Value::Object(object) => Value::Object(object),
                    other => json!({ "value": other }),
                };

                let mut event = json!({
                    "type": "ACTIVITY_SNAPSHOT",
                    "messageId": message_id,
                    "activityType": activity_type,
                    "content": content,
                    "timestamp": timestamp,
                });
                if let Some(replace) = pick_value(message, &["replace"]).and_then(Value::as_bool) {
                    if let Some(object) = event.as_object_mut() {
                        object.insert("replace".to_string(), Value::Bool(replace));
                    }
                }
                event
            }
        })
        .collect()
}

fn ag_ui_startup_events(body: &Value, session_id: &str) -> Vec<String> {
    let Some(ag_ui_input) = ag_ui_input_value(body) else {
        return Vec::new();
    };

    let thread_id =
        pick_string(ag_ui_input, &["threadId", "thread_id"]).unwrap_or_else(|| format!("thread-{}", session_id));
    let run_id =
        pick_string(ag_ui_input, &["runId", "run_id"]).unwrap_or_else(|| format!("run-{}", session_id));

    let mut events = vec![json!({
        "type": "RUN_STARTED",
        "threadId": thread_id,
        "runId": run_id,
    })];

    if let Some(parent_run_id) = pick_string(ag_ui_input, &["parentRunId", "parent_run_id"]) {
        if let Some(object) = events[0].as_object_mut() {
            object.insert("parentRunId".to_string(), Value::String(parent_run_id));
        }
    }

    if let Ok(value) = serde_json::to_value(ag_ui_input) {
        if let Some(object) = events[0].as_object_mut() {
            object.insert("input".to_string(), value);
        }
    }

    if let Some(snapshot) = pick_value(ag_ui_input, &["state"]) {
        events.push(json!({
            "type": "STATE_SNAPSHOT",
            "snapshot": snapshot,
        }));
    }

    if let Some(messages) = ag_ui_messages_snapshot_value(body) {
        events.push(json!({
            "type": "MESSAGES_SNAPSHOT",
            "messages": messages,
        }));
    }

    events.extend(ag_ui_activity_events(body));

    events.into_iter().filter_map(|event| serde_json::to_string(&event).ok()).collect()
}

fn build_content_from_request(body: &Value) -> Result<Content, RuntimeError> {
    let new_message = pick_value(body, &["newMessage", "new_message"]);
    let role = new_message
        .and_then(|value| pick_string(value, &["role"]))
        .unwrap_or_else(|| "user".to_string());
    let mut content = Content::new(role);

    if let Some(parts_value) = new_message.and_then(|value| pick_value(value, &["parts"])) {
        if let Some(parts) = parts_value.as_array() {
            let parsed_parts = parse_message_parts(parts)?;
            if !parsed_parts.is_empty() {
                content.parts = parsed_parts;
                return Ok(content);
            }
        }
    }

    let mut snippets = Vec::new();

    if let Some(ag_ui_input) = pick_value(body, &["input", "agUiInput", "ag_ui_input"]) {
        if let Some(messages) = pick_value(ag_ui_input, &["messages"]).and_then(Value::as_array) {
            for message in messages {
                if pick_string(message, &["role"]).as_deref() == Some("activity") {
                    continue;
                }
                if let Some(content_value) = pick_value(message, &["content"]) {
                    snippets.extend(extract_text_snippets(content_value));
                }
            }
        }
    }

    if let Some(mcp_apps_request) = pick_value(body, &["mcpAppsRequest", "mcp_apps_request"]) {
        if let Some(params) = pick_value(mcp_apps_request, &["params"]) {
            if let Some(content_value) = pick_value(params, &["content"]) {
                snippets.extend(extract_text_snippets(content_value));
            }
        }
    }

    if let Some(protocol_envelope) = pick_value(body, &["protocolEnvelope", "protocol_envelope"]) {
        if let Some(input) = pick_value(protocol_envelope, &["input"]) {
            if let Some(messages) = pick_value(input, &["messages"]).and_then(Value::as_array) {
                for message in messages {
                    if let Some(content_value) = pick_value(message, &["content"]) {
                        snippets.extend(extract_text_snippets(content_value));
                    }
                }
            }
        }

        if let Some(params) = pick_value(protocol_envelope, &["params"]) {
            if let Some(content_value) = pick_value(params, &["content"]) {
                snippets.extend(extract_text_snippets(content_value));
            }
        }
    }

    for snippet in snippets {
        content.parts.push(Part::Text { text: snippet });
    }

    if content.parts.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Request did not contain any renderable message parts.".to_string(),
        ));
    }

    Ok(content)
}

fn build_state_delta_from_request(body: &Value, profile: UiProfile) -> HashMap<String, Value> {
    let mut state_delta = HashMap::new();

    state_delta.insert("ui_protocol".to_string(), json!(profile.as_str()));

    if let Some(delta) = pick_value(body, &["stateDelta", "state_delta"]).and_then(Value::as_object)
    {
        for (key, value) in delta {
            state_delta.insert(key.clone(), value.clone());
        }
    }

    if let Some(ui_context) = pick_value(body, &["uiContext", "ui_context"]) {
        state_delta.insert("ui_context".to_string(), ui_context.clone());
    }
    if let Some(ui_event) = pick_value(body, &["uiEvent", "ui_event"]) {
        state_delta.insert("ui_event".to_string(), ui_event.clone());
    }
    if let Some(protocol_event) = pick_value(body, &["protocolEvent", "protocol_event"]) {
        state_delta.insert("protocol_event".to_string(), protocol_event.clone());
    }
    if let Some(protocol_envelope) = pick_value(body, &["protocolEnvelope", "protocol_envelope"]) {
        state_delta.insert("protocol_envelope".to_string(), protocol_envelope.clone());
    }

    if let Some(ag_ui_input) = pick_value(body, &["input", "agUiInput", "ag_ui_input"]) {
        if let Some(state) = pick_value(ag_ui_input, &["state"]) {
            state_delta.insert("ag_ui_state".to_string(), state.clone());
        }
        if let Some(forwarded_props) =
            pick_value(ag_ui_input, &["forwardedProps", "forwarded_props"])
        {
            state_delta.insert("ag_ui_forwarded_props".to_string(), forwarded_props.clone());
        }
        if let Some(thread_id) = pick_value(ag_ui_input, &["threadId", "thread_id"]) {
            state_delta.insert("ag_ui_thread_id".to_string(), thread_id.clone());
        }
        if let Some(run_id) = pick_value(ag_ui_input, &["runId", "run_id"]) {
            state_delta.insert("ag_ui_run_id".to_string(), run_id.clone());
        }
        if let Some(parent_run_id) = pick_value(ag_ui_input, &["parentRunId", "parent_run_id"]) {
            state_delta.insert("ag_ui_parent_run_id".to_string(), parent_run_id.clone());
        }
    }

    if let Some(compatibility_event) = pick_value(
        body,
        &["agUiCompatibilityEvent", "ag_ui_compatibility_event"],
    ) {
        state_delta.insert(
            "ag_ui_compatibility_event".to_string(),
            compatibility_event.clone(),
        );
    }

    if let Some(mcp_apps_request) = pick_value(body, &["mcpAppsRequest", "mcp_apps_request"]) {
        state_delta.insert("mcp_apps_request".to_string(), mcp_apps_request.clone());
    }

    if let Some(mcp_apps_initialize) =
        pick_value(body, &["mcpAppsInitialize", "mcp_apps_initialize"])
    {
        state_delta.insert(
            "mcp_apps_initialize".to_string(),
            mcp_apps_initialize.clone(),
        );
    }
    if let Some(mcp_apps_initialized) =
        pick_value(body, &["mcpAppsInitialized", "mcp_apps_initialized"])
    {
        state_delta.insert(
            "mcp_apps_initialized".to_string(),
            mcp_apps_initialized.clone(),
        );
    }

    state_delta
}

fn parse_mcp_apps_initialize_request(
    body: &Value,
) -> Result<Option<McpAppsInitializeRequest>, RuntimeError> {
    let Some(raw) = pick_value(body, &["mcpAppsInitialize", "mcp_apps_initialize"]) else {
        return Ok(None);
    };

    let request =
        serde_json::from_value::<McpAppsInitializeRequest>(raw.clone()).map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                format!("invalid mcpAppsInitialize payload: {}", error),
            )
        })?;

    if request.method != "ui/initialize" {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "unsupported MCP Apps initialize method '{}'",
                request.method
            ),
        ));
    }

    Ok(Some(request))
}

fn parse_mcp_apps_initialized(body: &Value) -> Result<bool, RuntimeError> {
    let Some(raw) = pick_value(body, &["mcpAppsInitialized", "mcp_apps_initialized"]) else {
        return Ok(false);
    };

    let method = pick_string(raw, &["method"]).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "mcpAppsInitialized.method is required".to_string(),
        )
    })?;

    if method != "ui/notifications/initialized" {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("unsupported MCP Apps initialized method '{}'", method),
        ));
    }

    Ok(true)
}

fn build_mcp_apps_negotiation_state(
    profile: UiProfile,
    app_name: &str,
    user_id: &str,
    session_id: &str,
    body: &Value,
) -> Result<Option<McpAppsNegotiationState>, RuntimeError> {
    if profile != UiProfile::McpApps {
        return Ok(None);
    }

    let initialized = parse_mcp_apps_initialized(body)?;
    let initialize_request = parse_mcp_apps_initialize_request(body)?;
    let mut registry = mcp_apps_bridge_registry().write().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "bridge registry poisoned".to_string(),
        )
    })?;
    let session = ensure_mcp_apps_bridge_session(&mut registry, app_name, user_id, session_id);

    if let Some(request) = initialize_request {
        session.initialize_result.protocol_version = request.params.protocol_version.clone();
        session.initialize_request = Some(request);
    }
    if initialized {
        session.initialized = true;
    }

    Ok(Some(negotiation_from_bridge_entry(session)))
}

async fn ensure_session(
    config: &ServerConfig,
    app_name: &str,
    user_id: &str,
    session_id: &str,
    state_delta: &HashMap<String, Value>,
) -> Result<(), RuntimeError> {
    let session_exists = config
        .session_service
        .get(GetRequest {
            app_name: app_name.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            num_recent_events: None,
            after: None,
        })
        .await
        .is_ok();

    if !session_exists {
        config
            .session_service
            .create(CreateRequest {
                app_name: app_name.to_string(),
                user_id: user_id.to_string(),
                session_id: Some(session_id.to_string()),
                state: state_delta.clone(),
            })
            .await
            .map_err(|error| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("failed to create session: {}", error),
                )
            })?;
        return Ok(());
    }

    if state_delta.is_empty() {
        return Ok(());
    }

    let mut context_event = Event::new(format!("ui-context-{}", session_id));
    context_event.author = "user".to_string();
    context_event.llm_response.turn_complete = true;
    context_event.actions.skip_summarization = true;
    context_event.actions.state_delta = state_delta.clone();

    config
        .session_service
        .append_event(session_id, context_event)
        .await
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to persist native ui context: {}", error),
            )
        })?;

    Ok(())
}

fn with_cors_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static(
            "content-type, authorization, x-request-id, x-adk-ui-protocol, x-adk-ui-transport",
        ),
    );
    response
}

fn json_error_response(status: StatusCode, message: impl Into<String>) -> Response {
    let response = (status, axum::Json(json!({ "error": message.into() }))).into_response();
    with_cors_headers(response)
}

fn capabilities_response() -> Response {
    let response = axum::Json(json!({
        "protocols": UI_PROTOCOL_CAPABILITIES,
    }))
    .into_response();
    with_cors_headers(response)
}

fn mcp_apps_initialize_response(body: &Value) -> Result<Response, RuntimeError> {
    let params = bridge_body(body);
    let (app_name, user_id, session_id) = bridge_identity(body)?;
    let protocol_version = pick_string(params, &["protocolVersion", "protocol_version"])
        .unwrap_or_else(|| MCP_APPS_PROTOCOL_VERSION.to_string());
    let app_info_value = pick_value(params, &["appInfo", "app_info"])
        .cloned()
        .unwrap_or_else(|| {
            json!({
                "name": "adk-ui-react-example",
                "version": "0.4.0",
            })
        });
    let app_capabilities_value = pick_value(params, &["appCapabilities", "app_capabilities"])
        .cloned()
        .unwrap_or_else(|| {
            json!({
                "availableDisplayModes": ["inline"],
                "tools": {
                    "listChanged": false,
                }
            })
        });

    let app_info =
        serde_json::from_value::<McpAppsPartyInfo>(app_info_value.clone()).map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                format!("invalid appInfo payload: {}", error),
            )
        })?;
    let app_capabilities = serde_json::from_value::<McpAppsAppCapabilities>(
        app_capabilities_value.clone(),
    )
    .map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            format!("invalid appCapabilities payload: {}", error),
        )
    })?;

    let mut registry = mcp_apps_bridge_registry().write().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "bridge registry poisoned".to_string(),
        )
    })?;
    let session = ensure_mcp_apps_bridge_session(&mut registry, &app_name, &user_id, &session_id);

    session.initialize_request = Some(McpAppsInitializeRequest {
        method: "ui/initialize".to_string(),
        params: McpAppsInitializeRequestParams {
            protocol_version: protocol_version.clone(),
            app_info,
            app_capabilities,
        },
    });
    session.initialize_result.protocol_version = protocol_version;

    if let Some(host_context) = pick_value(params, &["hostContext", "host_context"]) {
        merge_json(&mut session.initialize_result.host_context, host_context);
    }

    let response = axum::Json(json!({
        "initialized": session.initialized,
        "protocolVersion": session.initialize_result.protocol_version,
        "appInfo": app_info_value,
        "appCapabilities": app_capabilities_value,
        "hostInfo": serde_json::to_value(&session.initialize_result.host_info).unwrap_or_else(|_| json!({})),
        "hostCapabilities": session.initialize_result.host_capabilities,
        "hostContext": session.initialize_result.host_context,
        "messageCount": session.message_count,
        "modelContext": session.model_context,
        "modelContextRevision": session.model_context_revision,
    }))
    .into_response();

    Ok(with_cors_headers(response))
}

fn mcp_apps_message_response(body: &Value) -> Result<Response, RuntimeError> {
    let params = bridge_body(body);
    let (app_name, user_id, session_id) = bridge_identity(body)?;

    let mut registry = mcp_apps_bridge_registry().write().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "bridge registry poisoned".to_string(),
        )
    })?;
    let session = ensure_mcp_apps_bridge_session(&mut registry, &app_name, &user_id, &session_id);

    if let Some(host_context) = pick_value(params, &["hostContext", "host_context"]) {
        merge_json(&mut session.initialize_result.host_context, host_context);
    }

    session.message_count += 1;
    let mut message = json!({
        "role": pick_string(params, &["role"]).unwrap_or_else(|| "user".to_string()),
        "content": pick_value(params, &["content"]).cloned().unwrap_or_else(|| json!([])),
    });
    if let Some(metadata) = pick_value(params, &["metadata"]) {
        if let Some(object) = message.as_object_mut() {
            object.insert("metadata".to_string(), metadata.clone());
        }
    }
    session.last_message = Some(message.clone());

    let response = axum::Json(json!({
        "accepted": true,
        "initialized": session.initialized,
        "protocolVersion": session.initialize_result.protocol_version,
        "messageCount": session.message_count,
        "lastMessage": message,
        "hostInfo": serde_json::to_value(&session.initialize_result.host_info).unwrap_or_else(|_| json!({})),
        "hostCapabilities": session.initialize_result.host_capabilities,
        "hostContext": session.initialize_result.host_context,
    }))
    .into_response();

    Ok(with_cors_headers(response))
}

fn mcp_apps_update_model_context_response(body: &Value) -> Result<Response, RuntimeError> {
    let params = bridge_body(body);
    let (app_name, user_id, session_id) = bridge_identity(body)?;

    let mut registry = mcp_apps_bridge_registry().write().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "bridge registry poisoned".to_string(),
        )
    })?;
    let session = ensure_mcp_apps_bridge_session(&mut registry, &app_name, &user_id, &session_id);

    if let Some(host_context) = pick_value(params, &["hostContext", "host_context"]) {
        merge_json(&mut session.initialize_result.host_context, host_context);
    }

    let mode = pick_string(params, &["mode"]).unwrap_or_else(|| "replace".to_string());
    let mut next_model_context = if mode.eq_ignore_ascii_case("append") {
        session.model_context.clone()
    } else {
        Vec::new()
    };

    if let Some(content) = pick_value(params, &["content"]).and_then(Value::as_array) {
        next_model_context.extend(content.iter().cloned());
    }
    if let Some(structured_content) =
        pick_value(params, &["structuredContent", "structured_content"])
    {
        next_model_context.push(json!({
            "type": "structuredContent",
            "structuredContent": structured_content,
        }));
    }

    session.model_context = next_model_context.clone();
    session.model_context_revision += 1;

    let response = axum::Json(json!({
        "accepted": true,
        "initialized": session.initialized,
        "protocolVersion": session.initialize_result.protocol_version,
        "modelContext": next_model_context,
        "modelContextRevision": session.model_context_revision,
        "hostInfo": serde_json::to_value(&session.initialize_result.host_info).unwrap_or_else(|_| json!({})),
        "hostCapabilities": session.initialize_result.host_capabilities,
        "hostContext": session.initialize_result.host_context,
    }))
    .into_response();

    Ok(with_cors_headers(response))
}

async fn handle_protocol_native_run_sse(
    state: &ExampleRuntimeState,
    headers: &HeaderMap,
    body: Value,
) -> Result<Response, RuntimeError> {
    let app_name = pick_string(&body, &["appName", "app_name"])
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "appName is required".to_string()))?;
    let user_id = pick_string(&body, &["userId", "user_id"])
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "userId is required".to_string()))?;
    let session_id = pick_string(&body, &["sessionId", "session_id"])
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "sessionId is required".to_string()))?;
    let streaming = pick_value(&body, &["streaming"])
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let ui_profile = resolve_ui_profile(headers, &body)?;
    let mcp_apps_negotiation =
        build_mcp_apps_negotiation_state(ui_profile, &app_name, &user_id, &session_id, &body)?;

    log_profile_deprecation(ui_profile);
    info!(
        app_name = %app_name,
        user_id = %user_id,
        session_id = %session_id,
        ui_protocol = %ui_profile.as_str(),
        "protocol-aware /api/run_sse request received",
    );

    let content = build_content_from_request(&body)?;
    let state_delta = build_state_delta_from_request(&body, ui_profile);
    ensure_session(
        &state.config,
        &app_name,
        &user_id,
        &session_id,
        &state_delta,
    )
    .await?;

    let agent = state
        .config
        .agent_loader
        .load_agent(&app_name)
        .await
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load agent: {}", error),
            )
        })?;

    let streaming_mode = if streaming {
        adk_core::StreamingMode::SSE
    } else {
        adk_core::StreamingMode::None
    };

    let runner = Runner::new(RunnerConfig {
        app_name: app_name.clone(),
        agent,
        session_service: state.config.session_service.clone(),
        artifact_service: state.config.artifact_service.clone(),
        memory_service: state.config.memory_service.clone(),
        plugin_manager: None,
        run_config: Some(adk_core::RunConfig {
            streaming_mode,
            ..adk_core::RunConfig::default()
        }),
        compaction_config: state.config.compaction_config.clone(),
        context_cache_config: state.config.context_cache_config.clone(),
        cache_capable: state.config.cache_capable.clone(),
        request_context: None,
        cancellation_token: None,
    })
    .map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to create runner: {}", error),
        )
    })?;

    let event_stream = runner
        .run(
            UserId::try_from(user_id.as_str()).map_err(|e| {
                (StatusCode::BAD_REQUEST, format!("invalid userId: {}", e))
            })?,
            SessionId::try_from(session_id.as_str()).map_err(|e| {
                (StatusCode::BAD_REQUEST, format!("invalid sessionId: {}", e))
            })?,
            content,
        )
        .await
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to run agent: {}", error),
            )
        })?;

    let selected_profile = ui_profile;
    let mcp_bridge_state = mcp_apps_negotiation.clone();
    let startup_stream = if selected_profile == UiProfile::AgUi {
        stream::iter(
            ag_ui_startup_events(&body, &session_id)
                .into_iter()
                .map(|json| Ok::<_, Infallible>(SseEvent::default().data(json))),
        )
        .left_stream()
    } else {
        stream::empty::<Result<SseEvent, Infallible>>().right_stream()
    };
    let runtime_stream = event_stream.filter_map(move |item| {
        let mcp_bridge_state = mcp_bridge_state.clone();
        async move {
            match item {
                Ok(event) => {
                    let json = if selected_profile == UiProfile::McpApps {
                        serialize_runtime_event_with_mcp_bridge(
                            &event,
                            selected_profile,
                            mcp_bridge_state.as_ref(),
                        )?
                    } else {
                        serialize_runtime_event(&event, selected_profile)?
                    };
                    Some(Ok::<_, Infallible>(SseEvent::default().data(json)))
                }
                Err(_) => None,
            }
        }
    });
    let sse_stream = startup_stream.chain(runtime_stream);

    let response = Sse::new(sse_stream)
        .keep_alive(KeepAlive::default())
        .into_response();
    Ok(with_cors_headers(response))
}

async fn protocol_native_run_sse_middleware(
    State(state): State<ExampleRuntimeState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let is_options = request.method() == Method::OPTIONS;
    let path = request.uri().path().to_string();
    let should_intercept_run_sse = request.method() == Method::POST && path == "/api/run_sse";
    let should_intercept_capabilities =
        request.method() == Method::GET && path == "/api/ui/capabilities";
    let should_intercept_mcp_initialize =
        request.method() == Method::POST && path == "/api/ui/initialize";
    let should_intercept_mcp_message =
        request.method() == Method::POST && path == "/api/ui/message";
    let should_intercept_mcp_model_context =
        request.method() == Method::POST && path == "/api/ui/update-model-context";
    let should_intercept_preflight = is_options
        && matches!(
            path.as_str(),
            "/api/run_sse"
                | "/api/ui/capabilities"
                | "/api/ui/initialize"
                | "/api/ui/message"
                | "/api/ui/update-model-context"
        );

    if should_intercept_preflight {
        return with_cors_headers(StatusCode::OK.into_response());
    }

    if should_intercept_capabilities {
        return capabilities_response();
    }

    if !should_intercept_run_sse
        && !should_intercept_mcp_initialize
        && !should_intercept_mcp_message
        && !should_intercept_mcp_model_context
    {
        return next.run(request).await;
    }

    let (parts, body) = request.into_parts();
    let headers = parts.headers.clone();

    let body_bytes = match to_bytes(body, MAX_REQUEST_BODY_BYTES).await {
        Ok(bytes) => bytes,
        Err(error) => {
            return json_error_response(
                StatusCode::BAD_REQUEST,
                format!("failed to read request body: {}", error),
            );
        }
    };

    let body_value = match serde_json::from_slice::<Value>(&body_bytes) {
        Ok(value) => value,
        Err(error) => {
            return json_error_response(
                StatusCode::BAD_REQUEST,
                format!("invalid request body: {}", error),
            );
        }
    };

    if should_intercept_mcp_initialize {
        return match mcp_apps_initialize_response(&body_value) {
            Ok(response) => response,
            Err((status, message)) => json_error_response(status, message),
        };
    }

    if should_intercept_mcp_message {
        return match mcp_apps_message_response(&body_value) {
            Ok(response) => response,
            Err((status, message)) => json_error_response(status, message),
        };
    }

    if should_intercept_mcp_model_context {
        return match mcp_apps_update_model_context_response(&body_value) {
            Ok(response) => response,
            Err((status, message)) => json_error_response(status, message),
        };
    }

    match handle_protocol_native_run_sse(&state, &headers, body_value).await {
        Ok(response) => response,
        Err((status, message)) => json_error_response(status, message),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let api_key = std::env::var("GOOGLE_API_KEY")
        .or_else(|_| std::env::var("GEMINI_API_KEY"))
        .expect("GOOGLE_API_KEY or GEMINI_API_KEY must be set");
    let model_name =
        std::env::var("UI_DEMO_MODEL").unwrap_or_else(|_| "gemini-2.5-flash".to_string());

    let ui_tools = UiToolset::all_tools();

    let ui_demo = build_ui_agent(
        "ui_demo",
        "General purpose multi-surface demo agent",
        UI_DEMO_INSTRUCTION,
        &api_key,
        &model_name,
        &ui_tools,
    )?;
    let ui_working_support = build_ui_agent(
        "ui_working_support",
        "Support intake agent with working UI flows",
        SUPPORT_INSTRUCTION,
        &api_key,
        &model_name,
        &ui_tools,
    )?;
    let ui_working_appointment = build_ui_agent(
        "ui_working_appointment",
        "Appointment scheduling agent with working UI flows",
        APPOINTMENT_INSTRUCTION,
        &api_key,
        &model_name,
        &ui_tools,
    )?;
    let ui_working_events = build_ui_agent(
        "ui_working_events",
        "Event RSVP agent with working UI flows",
        EVENTS_INSTRUCTION,
        &api_key,
        &model_name,
        &ui_tools,
    )?;
    let ui_working_facilities = build_ui_agent(
        "ui_working_facilities",
        "Facilities maintenance agent with working UI flows",
        FACILITIES_INSTRUCTION,
        &api_key,
        &model_name,
        &ui_tools,
    )?;
    let ui_working_inventory = build_ui_agent(
        "ui_working_inventory",
        "Inventory restock agent with working UI flows",
        INVENTORY_INSTRUCTION,
        &api_key,
        &model_name,
        &ui_tools,
    )?;

    let agent_loader = Arc::new(MultiAgentLoader::new(vec![
        ui_demo,
        ui_working_support,
        ui_working_appointment,
        ui_working_events,
        ui_working_facilities,
        ui_working_inventory,
    ])?);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8080);

    let session_service: Arc<dyn SessionService> = Arc::new(InMemorySessionService::new());
    let config = ServerConfig::new(agent_loader, session_service);
    let app = create_app(config.clone()).layer(middleware::from_fn_with_state(
        ExampleRuntimeState { config },
        protocol_native_run_sse_middleware,
    ));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    println!("=== ADK UI Aggregated Server ===");
    println!("Server running on http://localhost:{}", port);
    println!("Model: {}", model_name);
    println!();
    println!("Loaded apps:");
    println!("  - ui_demo");
    println!("  - ui_working_support");
    println!("  - ui_working_appointment");
    println!("  - ui_working_events");
    println!("  - ui_working_facilities");
    println!("  - ui_working_inventory");
    println!();
    println!("React client: http://localhost:5173");
    println!();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_content_falls_back_to_ag_ui_messages() {
        let body = json!({
            "agUiInput": {
                "messages": [
                    { "content": "Approve the request" }
                ]
            }
        });

        let content = build_content_from_request(&body).expect("ag-ui content");
        assert_eq!(content.role, "user");
        assert_eq!(content.parts.len(), 1);
        assert!(matches!(&content.parts[0], Part::Text { text } if text == "Approve the request"));
    }

    #[test]
    fn build_content_falls_back_to_ag_ui_input_alias() {
        let body = json!({
            "input": {
                "messages": [
                    {
                        "content": [
                            { "type": "text", "text": "Review the dashboard" }
                        ]
                    }
                ]
            }
        });

        let content = build_content_from_request(&body).expect("ag-ui alias content");
        assert_eq!(content.role, "user");
        assert_eq!(content.parts.len(), 1);
        assert!(matches!(&content.parts[0], Part::Text { text } if text == "Review the dashboard"));
    }

    #[test]
    fn build_content_supports_function_response_parts() {
        let body = json!({
            "newMessage": {
                "role": "user",
                "parts": [
                    {
                        "functionResponse": {
                            "name": "approve_request",
                            "response": { "approved": true }
                        }
                    }
                ]
            }
        });

        let content = build_content_from_request(&body).expect("function response content");
        assert_eq!(content.parts.len(), 1);
        assert!(matches!(
            &content.parts[0],
            Part::FunctionResponse {
                function_response: FunctionResponseData { name, response },
                ..
            } if name == "approve_request" && response == &json!({ "approved": true })
        ));
    }

    #[test]
    fn build_state_delta_captures_native_protocol_context() {
        let body = json!({
            "uiContext": {
                "surfaceId": "main"
            },
            "uiEvent": {
                "action": "button_click",
                "action_id": "approve_request"
            },
            "agUiInput": {
                "state": {
                    "adkUi": {
                        "surfaceId": "main"
                    }
                },
                "forwardedProps": {
                    "source": "adk-ui-react"
                },
                "threadId": "thread-123",
                "runId": "run-456"
            }
        });

        let state_delta = build_state_delta_from_request(&body, UiProfile::AgUi);
        assert_eq!(state_delta.get("ui_protocol"), Some(&json!("ag_ui")));
        assert_eq!(
            state_delta.get("ui_context"),
            Some(&json!({ "surfaceId": "main" })),
        );
        assert_eq!(
            state_delta.get("ag_ui_thread_id"),
            Some(&json!("thread-123")),
        );
        assert_eq!(state_delta.get("ag_ui_run_id"), Some(&json!("run-456")),);
    }

    #[test]
    fn build_mcp_apps_negotiation_state_parses_initialize_request() {
        let body = json!({
            "mcpAppsInitialize": {
                "method": "ui/initialize",
                "params": {
                    "protocolVersion": "2025-11-21",
                    "appInfo": {
                        "name": "adk-ui-react-example",
                        "version": "0.4.0",
                    },
                    "appCapabilities": {
                        "availableDisplayModes": ["inline"],
                        "tools": {
                            "listChanged": false,
                        }
                    }
                }
            },
            "mcpAppsInitialized": {
                "method": "ui/notifications/initialized",
                "params": {}
            }
        });

        let negotiation = build_mcp_apps_negotiation_state(
            UiProfile::McpApps,
            "ui_demo",
            "user1",
            "session-1",
            &body,
        )
        .expect("negotiation");
        let negotiation = negotiation.expect("mcp apps negotiation state");

        assert!(negotiation.initialized);
        assert_eq!(
            negotiation
                .initialize_request
                .as_ref()
                .expect("initialize request")
                .params
                .app_info
                .name,
            "adk-ui-react-example"
        );
        assert_eq!(negotiation.initialize_result.protocol_version, "2025-11-21");
    }

    #[test]
    fn mcp_apps_direct_initialize_is_reused_by_runtime_negotiation() {
        let body = json!({
            "appName": "ui_demo",
            "userId": "user1",
            "sessionId": "session-direct",
            "protocolVersion": "2025-11-21",
            "appInfo": {
                "name": "adk-ui-react-example",
                "version": "0.4.0"
            },
            "appCapabilities": {
                "availableDisplayModes": ["inline"]
            }
        });
        let _ = mcp_apps_initialize_response(&body).expect("initialize response");

        let negotiation = build_mcp_apps_negotiation_state(
            UiProfile::McpApps,
            "ui_demo",
            "user1",
            "session-direct",
            &json!({
                "mcpAppsInitialized": {
                    "method": "ui/notifications/initialized"
                }
            }),
        )
        .expect("negotiation")
        .expect("mcp apps negotiation state");

        assert!(negotiation.initialized);
        assert_eq!(
            negotiation
                .initialize_request
                .as_ref()
                .expect("initialize request")
                .params
                .app_info
                .name,
            "adk-ui-react-example"
        );
    }
}
