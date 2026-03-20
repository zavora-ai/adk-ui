use crate::compat::{Result, Tool, ToolContext};
use crate::schema::*;
use crate::tools::{LegacyProtocolOptions, render_ui_response_with_protocol};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// A section in a dashboard layout.
///
/// Each section has a `type` field that determines which other fields are used:
/// - `"stats"`: Uses `stats` field for label/value/status items
/// - `"text"`: Uses `text` field for plain text content
/// - `"alert"`: Uses `message` and `severity` fields
/// - `"table"`: Uses `columns` and `rows` fields
/// - `"chart"`: Uses `chart_type`, `data`, `x_key`, `y_keys` fields
/// - `"key_value"`: Uses `pairs` field for key-value display
/// - `"list"`: Uses `items` and `ordered` fields
/// - `"code_block"`: Uses `code` and `language` fields
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DashboardSection {
    /// Section title displayed as card header
    pub title: String,
    /// Type of content: "stats", "table", "chart", "alert", "text", "key_value", "list", "code_block"
    #[serde(rename = "type")]
    pub section_type: String,
    /// For stats sections: list of label/value pairs with optional status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stats: Option<Vec<StatItem>>,
    /// For text sections: the text content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// For alert sections: the message to display
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// For alert sections: severity level ("info", "success", "warning", "error")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    /// For table sections: column definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<ColumnSpec>>,
    /// For table sections: row data as key-value maps
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<HashMap<String, Value>>>,
    /// For chart sections: chart type ("bar", "line", "area", "pie")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chart_type: Option<String>,
    /// For chart sections: data points as key-value maps
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<HashMap<String, Value>>>,
    /// For chart sections: key for x-axis values
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_key: Option<String>,
    /// For chart sections: keys for y-axis values
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y_keys: Option<Vec<String>>,
    /// For key_value sections: list of key-value pairs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pairs: Option<Vec<KeyValueItem>>,
    /// For list sections: list of text items
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<String>>,
    /// For list sections: whether to display as ordered list (default: false)
    #[serde(default)]
    pub ordered: bool,
    /// For code_block sections: the code content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// For code_block sections: programming language for syntax highlighting
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StatItem {
    /// Label displayed for this stat
    pub label: String,
    /// Value displayed for this stat
    pub value: String,
    /// Optional status indicator: "operational"/"ok"/"success" (green), "degraded"/"warning" (yellow), "down"/"error"/"outage" (red)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ColumnSpec {
    /// Column header text
    pub header: String,
    /// Key to access data from row objects
    pub key: String,
}

/// Key-value pair for key_value sections
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KeyValueItem {
    /// Display label
    pub key: String,
    /// Display value
    pub value: String,
}

/// Parameters for the render_layout tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RenderLayoutParams {
    /// Dashboard/layout title
    pub title: String,
    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
    /// Sections to display
    pub sections: Vec<DashboardSection>,
    /// Theme: "light", "dark", or "system" (default: "light")
    #[serde(default)]
    pub theme: Option<String>,
    /// Optional protocol output configuration.
    #[serde(flatten)]
    pub protocol: LegacyProtocolOptions,
}

/// Tool for rendering complex multi-component layouts.
///
/// Creates dashboard-style layouts with multiple sections, each containing
/// different types of content. Ideal for status pages, admin dashboards,
/// and multi-section displays.
///
/// # Supported Section Types
///
/// - `stats`: Status indicators with labels, values, and optional status colors
/// - `text`: Plain text content
/// - `alert`: Notification banners with severity levels
/// - `table`: Tabular data with columns and rows
/// - `chart`: Data visualizations (bar, line, area, pie)
/// - `key_value`: Key-value pair displays
/// - `list`: Ordered or unordered lists
/// - `code_block`: Code snippets with syntax highlighting
///
/// # Example JSON Parameters
///
/// ```json
/// {
///   "title": "System Status",
///   "sections": [
///     {
///       "title": "Services",
///       "type": "stats",
///       "stats": [
///         { "label": "API", "value": "Healthy", "status": "operational" },
///         { "label": "Database", "value": "Degraded", "status": "warning" }
///       ]
///     },
///     {
///       "title": "Configuration",
///       "type": "key_value",
///       "pairs": [
///         { "key": "Version", "value": "1.2.3" },
///         { "key": "Region", "value": "us-east-1" }
///       ]
///     }
///   ]
/// }
/// ```
pub struct RenderLayoutTool;

impl RenderLayoutTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RenderLayoutTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for RenderLayoutTool {
    fn name(&self) -> &str {
        "render_layout"
    }

    fn description(&self) -> &str {
        r#"Render a dashboard layout with multiple sections. Output example:
┌─────────────────────────────────────────────┐
│ System Status                               │
├─────────────────────────────────────────────┤
│ CPU: 45% ✓  │ Memory: 78% ⚠  │ Disk: 92% ✗ │
├─────────────────────────────────────────────┤
│ [Chart: Usage over time]                    │
├─────────────────────────────────────────────┤
│ Region: us-east-1  │  Version: 1.2.3        │
└─────────────────────────────────────────────┘
Section types: stats (label/value/status), table, chart, alert, text, key_value, list, code_block."#
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(super::generate_gemini_schema::<RenderLayoutParams>())
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let params: RenderLayoutParams = serde_json::from_value(args.clone()).map_err(|e| {
            crate::compat::AdkError::Tool(format!("Invalid parameters: {}. Got: {}", e, args))
        })?;
        let protocol_options = params.protocol.clone();

        let mut components = Vec::new();

        // Title
        components.push(Component::Text(Text {
            id: None,
            content: params.title,
            variant: TextVariant::H2,
        }));

        // Description
        if let Some(desc) = params.description {
            components.push(Component::Text(Text {
                id: None,
                content: desc,
                variant: TextVariant::Caption,
            }));
        }

        // Build sections
        for section in params.sections {
            let section_component = build_section_component(section);
            components.push(section_component);
        }

        let mut ui = UiResponse::new(components);

        // Apply theme if specified
        if let Some(theme_str) = params.theme {
            let theme = match theme_str.to_lowercase().as_str() {
                "dark" => Theme::Dark,
                "system" => Theme::System,
                _ => Theme::Light,
            };
            ui = ui.with_theme(theme);
        }

        render_ui_response_with_protocol(ui, &protocol_options, "layout")
    }
}

fn build_section_component(section: DashboardSection) -> Component {
    let mut card_content: Vec<Component> = Vec::new();

    match section.section_type.as_str() {
        "stats" => {
            if let Some(stats) = section.stats {
                // Create a nice stats display
                for stat in stats {
                    let status_indicator = match stat.status.as_deref() {
                        Some("operational") | Some("ok") | Some("success") => "🟢 ",
                        Some("degraded") | Some("warning") => "🟡 ",
                        Some("down") | Some("error") | Some("outage") => "🔴 ",
                        _ => "",
                    };
                    card_content.push(Component::Text(Text {
                        id: None,
                        content: format!("{}{}: {}", status_indicator, stat.label, stat.value),
                        variant: TextVariant::Body,
                    }));
                }
            }
        }
        "text" => {
            if let Some(text) = section.text {
                card_content.push(Component::Text(Text {
                    id: None,
                    content: text,
                    variant: TextVariant::Body,
                }));
            }
        }
        "alert" => {
            let variant = match section.severity.as_deref() {
                Some("success") => AlertVariant::Success,
                Some("warning") => AlertVariant::Warning,
                Some("error") => AlertVariant::Error,
                _ => AlertVariant::Info,
            };
            return Component::Alert(Alert {
                id: None,
                title: section.title,
                description: section.message,
                variant,
            });
        }
        "table" => {
            if let (Some(cols), Some(rows)) = (section.columns, section.rows) {
                let table_columns: Vec<TableColumn> = cols
                    .into_iter()
                    .map(|c| TableColumn {
                        header: c.header,
                        accessor_key: c.key,
                        sortable: true,
                    })
                    .collect();
                card_content.push(Component::Table(Table {
                    id: None,
                    columns: table_columns,
                    data: rows,
                    sortable: false,
                    page_size: None,
                    striped: false,
                }));
            }
        }
        "chart" => {
            if let (Some(data), Some(x), Some(y)) = (section.data, section.x_key, section.y_keys) {
                let kind = match section.chart_type.as_deref() {
                    Some("line") => ChartKind::Line,
                    Some("area") => ChartKind::Area,
                    Some("pie") => ChartKind::Pie,
                    _ => ChartKind::Bar,
                };
                card_content.push(Component::Chart(Chart {
                    id: None,
                    title: None,
                    kind,
                    data,
                    x_key: x,
                    y_keys: y,
                    x_label: None,
                    y_label: None,
                    show_legend: true,
                    colors: None,
                }));
            }
        }
        "key_value" => {
            if let Some(pairs) = section.pairs {
                let kv_pairs: Vec<KeyValuePair> = pairs
                    .into_iter()
                    .map(|p| KeyValuePair {
                        key: p.key,
                        value: p.value,
                    })
                    .collect();
                card_content.push(Component::KeyValue(KeyValue {
                    id: None,
                    pairs: kv_pairs,
                }));
            }
        }
        "list" => {
            if let Some(items) = section.items {
                card_content.push(Component::List(List {
                    id: None,
                    items,
                    ordered: section.ordered,
                }));
            }
        }
        "code_block" => {
            if let Some(code) = section.code {
                card_content.push(Component::CodeBlock(CodeBlock {
                    id: None,
                    code,
                    language: section.language,
                }));
            }
        }
        _ => {
            // Fallback: show raw text for unknown section types
            card_content.push(Component::Text(Text {
                id: None,
                content: format!("Unknown section type: {}", section.section_type),
                variant: TextVariant::Caption,
            }));
        }
    }

    // If no content was added, add a placeholder
    if card_content.is_empty() {
        card_content.push(Component::Text(Text {
            id: None,
            content: "(No content)".to_string(),
            variant: TextVariant::Caption,
        }));
    }

    Component::Card(Card {
        id: None,
        title: Some(section.title),
        description: None,
        content: card_content,
        footer: None,
    })
}
