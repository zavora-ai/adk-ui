//! Shared types for the Agentic Web Protocol (AWP).
//!
//! This is a local stub crate providing the AWP types referenced by `adk-ui`.
//! It will be replaced by the upstream `awp-types` crate once available.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Current AWP protocol version.
pub const CURRENT_VERSION: &str = "1.0";

/// AWP protocol version descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwpVersion {
    pub major: u32,
    pub minor: u32,
}

impl AwpVersion {
    pub fn current() -> Self {
        Self { major: 1, minor: 0 }
    }
}

/// A single capability entry describing a tool or endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityEntry {
    pub name: String,
    pub description: String,
    pub endpoint: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<String>,
}

/// A manifest listing all capabilities exposed to agent visitors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityManifest {
    pub version: String,
    pub capabilities: Vec<CapabilityEntry>,
}

impl CapabilityManifest {
    pub fn new(capabilities: Vec<CapabilityEntry>) -> Self {
        Self {
            version: CURRENT_VERSION.to_string(),
            capabilities,
        }
    }
}

/// An AWP response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwpResponse {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    pub payload: Value,
}

impl AwpResponse {
    pub fn new(payload: Value) -> Self {
        Self {
            version: CURRENT_VERSION.to_string(),
            request_id: None,
            payload,
        }
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}
