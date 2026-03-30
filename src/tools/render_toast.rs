use crate::compat::{Result, Tool, ToolContext};
use crate::schema::*;
use crate::tools::{LegacyProtocolOptions, render_ui_response_with_protocol};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Parameters for the render_toast tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RenderToastParams {
    /// Toast message to display
    pub message: String,
    /// Toast variant: info, success, warning, error
    #[serde(default = "default_variant")]
    pub variant: String,
    /// Duration in milliseconds before auto-dismiss (default 5000)
    #[serde(default = "default_duration")]
    pub duration: u32,
    /// Whether the toast can be manually dismissed
    #[serde(default = "default_true")]
    pub dismissible: bool,
    /// Optional protocol output configuration.
    #[serde(flatten)]
    pub protocol: LegacyProtocolOptions,
}

fn default_variant() -> String {
    "info".to_string()
}

fn default_duration() -> u32 {
    5000
}

fn default_true() -> bool {
    true
}

/// Tool for rendering toast notifications
pub struct RenderToastTool;

impl RenderToastTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RenderToastTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for RenderToastTool {
    fn name(&self) -> &str {
        "render_toast"
    }

    fn description(&self) -> &str {
        "Render a temporary toast notification. Use for brief status updates, success messages, or non-blocking alerts that auto-dismiss."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(super::generate_gemini_schema::<RenderToastParams>())
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let params: RenderToastParams = serde_json::from_value(args)
            .map_err(|e| crate::compat::AdkError::tool(format!("Invalid parameters: {}", e)))?;
        let protocol_options = params.protocol.clone();

        let variant = match params.variant.as_str() {
            "success" => AlertVariant::Success,
            "warning" => AlertVariant::Warning,
            "error" => AlertVariant::Error,
            _ => AlertVariant::Info,
        };

        let ui = UiResponse::new(vec![Component::Toast(Toast {
            id: None,
            message: params.message,
            variant,
            duration: params.duration,
            dismissible: params.dismissible,
        })]);

        render_ui_response_with_protocol(ui, &protocol_options, "toast")
    }
}
