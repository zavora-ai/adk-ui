use crate::compat::{Result, Tool, ToolContext};
use crate::schema::*;
use crate::tools::{LegacyProtocolOptions, render_ui_response_with_protocol};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Parameters for the render_card tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RenderCardParams {
    /// Title of the card
    pub title: String,
    /// Optional description/subtitle
    #[serde(default)]
    pub description: Option<String>,
    /// Main content text (supports markdown-like formatting)
    pub content: String,
    /// Optional action buttons
    #[serde(default)]
    pub actions: Vec<CardAction>,
    /// Optional protocol output configuration.
    #[serde(flatten)]
    pub protocol: LegacyProtocolOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CardAction {
    /// Button label
    pub label: String,
    /// Action ID triggered when clicked
    pub action_id: String,
    /// Button variant: primary, secondary, danger, ghost
    #[serde(default = "default_variant")]
    pub variant: String,
}

fn default_variant() -> String {
    "primary".to_string()
}

/// Tool for rendering information cards.
///
/// Creates styled card components to display content with optional action buttons.
/// Cards are ideal for status updates, summaries, or any structured information.
///
/// # Example JSON Parameters
///
/// ```json
/// {
///   "title": "Welcome",
///   "description": "Getting started with your account",
///   "content": "Your account has been created successfully. Click below to continue.",
///   "actions": [
///     { "label": "Get Started", "action_id": "start", "variant": "primary" }
///   ]
/// }
/// ```
pub struct RenderCardTool;

impl RenderCardTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RenderCardTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for RenderCardTool {
    fn name(&self) -> &str {
        "render_card"
    }

    fn description(&self) -> &str {
        r#"Render an information card. Output example:
┌─────────────────────────────┐
│ Welcome                     │
│ Your account is ready       │
│ ─────────────────────────── │
│ Click below to get started. │
│      [Get Started]          │
└─────────────────────────────┘
Use for status updates, summaries, or any structured info with optional action buttons."#
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(super::generate_gemini_schema::<RenderCardParams>())
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let params: RenderCardParams = serde_json::from_value(args)
            .map_err(|e| crate::compat::AdkError::Tool(format!("Invalid parameters: {}", e)))?;
        let protocol_options = params.protocol.clone();

        // Build card content
        let content = vec![Component::Text(Text {
            id: None,
            content: params.content,
            variant: TextVariant::Body,
        })];

        // Build footer with action buttons
        let footer = if params.actions.is_empty() {
            None
        } else {
            Some(
                params
                    .actions
                    .into_iter()
                    .map(|action| {
                        let variant = match action.variant.as_str() {
                            "secondary" => ButtonVariant::Secondary,
                            "danger" => ButtonVariant::Danger,
                            "ghost" => ButtonVariant::Ghost,
                            "outline" => ButtonVariant::Outline,
                            _ => ButtonVariant::Primary,
                        };
                        Component::Button(Button {
                            id: None,
                            label: action.label,
                            action_id: action.action_id,
                            variant,
                            disabled: false,
                            icon: None,
                        })
                    })
                    .collect(),
            )
        };

        let ui = UiResponse::new(vec![Component::Card(Card {
            id: None,
            title: Some(params.title),
            description: params.description,
            content,
            footer,
        })]);

        render_ui_response_with_protocol(ui, &protocol_options, "card")
    }
}
