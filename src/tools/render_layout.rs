use crate::a2ui::stable_id;
use crate::compat::{Result, Tool, ToolContext};
use crate::schema::*;
use crate::tools::render_form::{FormField, build_form_content};
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
/// - `"form"`: Uses `fields`, `submit_action`, and `submit_label` fields
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DashboardSection {
    /// Section title displayed as card header
    pub title: String,
    /// Type of content: "stats", "table", "chart", "alert", "text", "key_value", "list", "code_block", "form"
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
    /// For form sections: fields to collect without giving up the surrounding decision context
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FormField>>,
    /// For form sections: action id emitted when the form is submitted
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub submit_action: Option<String>,
    /// For form sections: specific review-oriented button label
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub submit_label: Option<String>,
    /// For form sections: optional data binding path prefix
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path_prefix: Option<String>,
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
/// - `form`: Editable proposal or focused controls inside the wider layout
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
Section types: stats (label/value/status), table, chart, alert, text, key_value, list, code_block, form. Use a form section for editable filters or proposals that need surrounding dashboard context."#
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(super::generate_gemini_schema::<RenderLayoutParams>())
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let params: RenderLayoutParams = serde_json::from_value(args.clone()).map_err(|e| {
            crate::compat::AdkError::tool(format!("Invalid parameters: {}. Got: {}", e, args))
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
                return build_metric_section(section.title, stats);
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
                let page_size = (rows.len() > 10).then_some(10);
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
                    sortable: true,
                    page_size,
                    striped: true,
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
                let normalized_title = section.title.to_ascii_lowercase();
                let represents_metrics = normalized_title.contains("kpi")
                    || normalized_title.contains("key performance")
                    || normalized_title.contains("metric");
                if represents_metrics && (2..=8).contains(&pairs.len()) {
                    let stats = pairs
                        .into_iter()
                        .map(|pair| StatItem {
                            label: pair.key,
                            value: pair.value,
                            status: None,
                        })
                        .collect();
                    return build_metric_section(section.title, stats);
                }

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
        "form" => {
            if let Some(fields) = section.fields {
                let form_id = stable_id(&format!("layout-form:{}", section.title));
                let submit_action = section
                    .submit_action
                    .as_deref()
                    .unwrap_or("review_proposal");
                let submit_label = section.submit_label.as_deref().unwrap_or("Review proposal");
                card_content.extend(build_form_content(
                    &form_id,
                    fields,
                    section.data_path_prefix.as_deref(),
                    submit_action,
                    submit_label,
                ));
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

fn build_metric_section(title: String, stats: Vec<StatItem>) -> Component {
    let columns = stats.len().clamp(1, 4) as u8;
    let metric_cards = stats
        .into_iter()
        .map(|stat| {
            let mut content = vec![Component::Text(Text {
                id: None,
                content: stat.value,
                variant: TextVariant::H2,
            })];
            if let Some(status) = stat.status {
                let variant = match status.as_str() {
                    "operational" | "ok" | "success" => BadgeVariant::Success,
                    "degraded" | "warning" => BadgeVariant::Warning,
                    "down" | "error" | "outage" => BadgeVariant::Error,
                    _ => BadgeVariant::Secondary,
                };
                content.push(Component::Badge(Badge {
                    id: None,
                    label: status.replace(['_', '-'], " "),
                    variant,
                }));
            }
            Component::Card(Card {
                id: None,
                title: Some(stat.label),
                description: None,
                content,
                footer: None,
            })
        })
        .collect();

    Component::Stack(Stack {
        id: None,
        direction: StackDirection::Vertical,
        gap: 3,
        children: vec![
            Component::Text(Text {
                id: None,
                content: title,
                variant: TextVariant::H3,
            }),
            Component::Grid(Grid {
                id: None,
                columns,
                children: metric_cards,
                gap: 3,
            }),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_render_as_responsive_metric_cards() {
        let component = build_section_component(DashboardSection {
            title: "System health".to_string(),
            section_type: "stats".to_string(),
            stats: Some(vec![
                StatItem {
                    label: "Revenue".to_string(),
                    value: "$42k".to_string(),
                    status: Some("success".to_string()),
                },
                StatItem {
                    label: "Incidents".to_string(),
                    value: "3".to_string(),
                    status: Some("warning".to_string()),
                },
            ]),
            text: None,
            message: None,
            severity: None,
            columns: None,
            rows: None,
            chart_type: None,
            data: None,
            x_key: None,
            y_keys: None,
            pairs: None,
            items: None,
            ordered: false,
            code: None,
            language: None,
            fields: None,
            submit_action: None,
            submit_label: None,
            data_path_prefix: None,
        });

        let Component::Stack(stack) = component else {
            panic!("stats section should render as a stack");
        };
        let Component::Grid(grid) = &stack.children[1] else {
            panic!("stats stack should contain a grid");
        };
        assert_eq!(grid.columns, 2);
        assert_eq!(grid.children.len(), 2);
    }

    #[test]
    fn kpi_key_values_are_promoted_to_metric_cards() {
        let component = build_section_component(DashboardSection {
            title: "Key Performance Indicators".to_string(),
            section_type: "key_value".to_string(),
            stats: None,
            text: None,
            message: None,
            severity: None,
            columns: None,
            rows: None,
            chart_type: None,
            data: None,
            x_key: None,
            y_keys: None,
            pairs: Some(vec![
                KeyValueItem {
                    key: "Revenue".to_string(),
                    value: "$42k".to_string(),
                },
                KeyValueItem {
                    key: "Users".to_string(),
                    value: "1,250".to_string(),
                },
                KeyValueItem {
                    key: "Conversion".to_string(),
                    value: "12%".to_string(),
                },
            ]),
            items: None,
            ordered: false,
            code: None,
            language: None,
            fields: None,
            submit_action: None,
            submit_label: None,
            data_path_prefix: None,
        });

        let Component::Stack(stack) = component else {
            panic!("KPI key/value section should render as a metric stack");
        };
        let Component::Grid(grid) = &stack.children[1] else {
            panic!("KPI metric stack should contain a grid");
        };
        assert_eq!(grid.columns, 3);
        assert_eq!(grid.children.len(), 3);
    }

    #[test]
    fn form_section_keeps_editable_controls_in_layout() {
        let component = build_section_component(DashboardSection {
            title: "Rollback proposal".to_string(),
            section_type: "form".to_string(),
            stats: None,
            text: None,
            message: None,
            severity: None,
            columns: None,
            rows: None,
            chart_type: None,
            data: None,
            x_key: None,
            y_keys: None,
            pairs: None,
            items: None,
            ordered: false,
            code: None,
            language: None,
            fields: Some(vec![FormField {
                name: "region".to_string(),
                path: None,
                label: "Region scope".to_string(),
                field_type: "select".to_string(),
                placeholder: None,
                required: true,
                options: vec![SelectOption {
                    label: "us-east-1 only".to_string(),
                    value: "us-east-1".to_string(),
                }],
            }]),
            submit_action: Some("review_rollback".to_string()),
            submit_label: Some("Review guarded rollback".to_string()),
            data_path_prefix: Some("/proposal".to_string()),
        });

        let Component::Card(card) = component else {
            panic!("form section should render as a card");
        };
        assert!(
            matches!(&card.content[0], Component::Select(select) if select.name == "/proposal/region")
        );
        assert!(
            matches!(&card.content[1], Component::Button(button) if button.action_id == "review_rollback")
        );
    }
}
