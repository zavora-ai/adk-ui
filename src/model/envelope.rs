use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Standard envelope protocol marker for render tool outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ToolEnvelopeProtocol {
    AdkUi,
    A2ui,
    AgUi,
    McpApps,
    #[cfg(feature = "awp")]
    Awp,
}

/// Canonical tool output envelope.
///
/// Payload fields are flattened to preserve backward-compatible response shapes
/// while still attaching protocol metadata.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolEnvelope<P> {
    pub protocol: ToolEnvelopeProtocol,
    pub version: String,
    pub surface_id: String,
    #[serde(flatten)]
    pub payload: P,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
    #[cfg(feature = "awp")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub awp_version: Option<String>,
    #[cfg(feature = "awp")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl<P> ToolEnvelope<P> {
    pub fn new(protocol: ToolEnvelopeProtocol, surface_id: impl Into<String>, payload: P) -> Self {
        Self {
            protocol,
            version: "1.0".to_string(),
            surface_id: surface_id.into(),
            payload,
            meta: None,
            #[cfg(feature = "awp")]
            awp_version: None,
            #[cfg(feature = "awp")]
            request_id: None,
        }
    }

    pub fn with_meta(mut self, meta: Option<Value>) -> Self {
        self.meta = meta;
        self
    }
}

#[cfg(feature = "awp")]
impl<P: Serialize> ToolEnvelope<P> {
    /// Set the AWP version on this envelope.
    pub fn with_awp_version(mut self, version: impl Into<String>) -> Self {
        self.awp_version = Some(version.into());
        self
    }

    /// Set the AWP request ID on this envelope.
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Convert this envelope into an AWP response.
    pub fn to_awp_response(&self) -> Result<awp_types::AwpResponse, crate::compat::AdkError> {
        let payload = serde_json::to_value(&self.payload).map_err(|e| {
            crate::compat::AdkError::tool(format!(
                "Failed to serialize envelope payload for AWP response: {}",
                e
            ))
        })?;
        Ok(awp_types::AwpResponse {
            id: uuid::Uuid::now_v7(),
            version: awp_types::CURRENT_VERSION,
            status: "ok".to_string(),
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde_json::json;

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    struct Payload {
        value: String,
    }

    #[test]
    fn envelope_serializes_flattened_payload() {
        let envelope = ToolEnvelope::new(
            ToolEnvelopeProtocol::A2ui,
            "main",
            Payload {
                value: "ok".to_string(),
            },
        )
        .with_meta(Some(json!({"trace_id": "abc"})));

        let value = serde_json::to_value(envelope).expect("serialize");
        assert_eq!(value["protocol"], "a2ui");
        assert_eq!(value["surface_id"], "main");
        assert_eq!(value["version"], "1.0");
        assert_eq!(value["value"], "ok");
        assert_eq!(value["meta"]["trace_id"], "abc");
    }
}
