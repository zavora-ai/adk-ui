use crate::compat::{Result, Tool, ToolContext};
use crate::schema::*;
use crate::tools::{LegacyProtocolOptions, render_ui_response_with_protocol};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Parameters for the render_progress tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RenderProgressParams {
    /// Title or label for the progress
    pub title: String,
    /// Progress percentage (0-100)
    pub value: u8,
    /// Optional description of current step
    #[serde(default)]
    pub description: Option<String>,
    /// List of steps with their completion status
    #[serde(default)]
    pub steps: Option<Vec<ProgressStep>>,
    /// Optional protocol output configuration.
    #[serde(flatten)]
    pub protocol: LegacyProtocolOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProgressStep {
    /// Step label
    pub label: String,
    /// Whether step is completed
    #[serde(default)]
    pub completed: bool,
    /// Whether this is the current step
    #[serde(default)]
    pub current: bool,
}

/// Tool for rendering progress indicators and loading states
pub struct RenderProgressTool;

impl RenderProgressTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RenderProgressTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for RenderProgressTool {
    fn name(&self) -> &str {
        "render_progress"
    }

    fn description(&self) -> &str {
        "Render a progress indicator to show task completion status. Use this when performing multi-step operations or to show loading progress. Can show a progress bar with optional steps."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(super::generate_gemini_schema::<RenderProgressParams>())
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let params: RenderProgressParams = serde_json::from_value(args)
            .map_err(|e| crate::compat::AdkError::tool(format!("Invalid parameters: {}", e)))?;
        let protocol_options = params.protocol.clone();

        let mut components = Vec::new();

        // Title
        components.push(Component::Text(Text {
            id: None,
            content: params.title,
            variant: TextVariant::H3,
        }));

        // Description
        if let Some(desc) = params.description {
            components.push(Component::Text(Text {
                id: None,
                content: desc,
                variant: TextVariant::Caption,
            }));
        }

        // Progress bar
        components.push(Component::Progress(Progress {
            id: None,
            value: params.value,
            label: Some(format!("{}%", params.value)),
        }));

        // Steps if provided
        if let Some(steps) = params.steps {
            for step in steps {
                let prefix = if step.completed {
                    "✅"
                } else if step.current {
                    "⏳"
                } else {
                    "⬜"
                };
                components.push(Component::Text(Text {
                    id: None,
                    content: format!("{} {}", prefix, step.label),
                    variant: TextVariant::Body,
                }));
            }
        }

        let ui = UiResponse::new(vec![Component::Card(Card {
            id: None,
            title: None,
            description: None,
            content: components,
            footer: None,
        })]);

        render_ui_response_with_protocol(ui, &protocol_options, "progress")
    }
}
