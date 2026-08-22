use crate::compat::{Result, Tool, ToolContext};
use crate::schema::*;
use crate::tools::{LegacyProtocolOptions, render_ui_response_with_protocol};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Parameters for the render_chart tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RenderChartParams {
    /// Chart title
    #[serde(default)]
    pub title: Option<String>,
    /// Chart type: bar, line, area, pie, scatter, or sparkline
    #[serde(rename = "type", default = "default_chart_type")]
    pub chart_type: String,
    /// Data points - array of objects with x_key and y_key values
    #[serde(default)]
    pub data: Vec<HashMap<String, Value>>,
    #[serde(default)]
    pub data_source: Option<DataSource>,
    /// Key for x-axis values
    pub x_key: String,
    /// Keys for y-axis values (can be multiple for multi-series)
    pub y_keys: Vec<String>,
    #[serde(default)]
    pub x_type: ChartXType,
    #[serde(default)]
    pub time_format: Option<String>,
    #[serde(default)]
    pub window: Option<u32>,
    #[serde(default)]
    pub x_label: Option<String>,
    #[serde(default)]
    pub y_label: Option<String>,
    #[serde(default = "default_show_legend")]
    pub show_legend: bool,
    #[serde(default)]
    pub colors: Option<Vec<String>>,
    /// Optional protocol output configuration.
    #[serde(flatten)]
    pub protocol: LegacyProtocolOptions,
}

fn default_chart_type() -> String {
    "bar".to_string()
}

fn default_show_legend() -> bool {
    true
}

/// Tool for rendering charts and data visualizations.
///
/// Creates interactive charts to display data trends, comparisons, and distributions.
/// Supports multiple chart types and customizable axis labels, legends, and colors.
///
/// # Chart Types
///
/// - `bar`: Vertical bar chart (default)
/// - `line`: Line chart for trends
/// - `area`: Filled area chart
/// - `pie`: Pie chart for distributions
///
/// # Example JSON Parameters
///
/// ```json
/// {
///   "title": "Monthly Sales",
///   "type": "line",
///   "data": [
///     { "month": "Jan", "sales": 100 },
///     { "month": "Feb", "sales": 150 },
///     { "month": "Mar", "sales": 120 }
///   ],
///   "x_key": "month",
///   "y_keys": ["sales"]
/// }
/// ```
pub struct RenderChartTool;

impl RenderChartTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RenderChartTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for RenderChartTool {
    fn name(&self) -> &str {
        "render_chart"
    }

    fn description(&self) -> &str {
        "Render a chart to visualize data. Supports bar, line, area, and pie charts. Use this for showing trends, comparisons, or distributions."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(super::generate_gemini_schema::<RenderChartParams>())
    }

    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let params: RenderChartParams = serde_json::from_value(args)
            .map_err(|e| crate::compat::AdkError::tool(format!("Invalid parameters: {}", e)))?;
        if params.window == Some(0) {
            return Err(crate::compat::AdkError::tool(
                "Invalid parameters: window must be greater than zero".to_string(),
            ));
        }
        if let Some(source) = &params.data_source
            && !source.binding_path.starts_with('/')
        {
            return Err(crate::compat::AdkError::tool(
                "Invalid parameters: data_source.binding_path must be an absolute JSON Pointer"
                    .to_string(),
            ));
        }
        let protocol_options = params.protocol.clone();

        let kind = match params.chart_type.as_str() {
            "line" => ChartKind::Line,
            "area" => ChartKind::Area,
            "pie" => ChartKind::Pie,
            "scatter" => ChartKind::Scatter,
            "sparkline" => ChartKind::Sparkline,
            _ => ChartKind::Bar,
        };

        let ui = UiResponse::new(vec![Component::Chart(Chart {
            id: None,
            title: params.title,
            kind,
            data: params.data,
            data_source: params.data_source,
            x_key: params.x_key,
            y_keys: params.y_keys,
            x_type: params.x_type,
            time_format: params.time_format,
            window: params.window,
            x_label: params.x_label,
            y_label: params.y_label,
            show_legend: params.show_legend,
            colors: params.colors,
        })]);

        let surface_id = protocol_options.resolved_surface_id("chart");
        let output = render_ui_response_with_protocol(ui, &protocol_options, "chart")?;
        let surface_ref = crate::surface_runtime::next_surface_ref(&ctx, surface_id);
        crate::surface_runtime::record_surface_ref(&ctx, &surface_ref);
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parameters_expose_time_series_behavior() {
        let params: RenderChartParams = serde_json::from_value(json!({
            "type": "sparkline",
            "x_key": "at",
            "y_keys": ["latency"],
            "x_type": "time",
            "time_format": "time",
            "window": 60,
            "x_label": "Observed",
            "y_label": "Milliseconds",
            "show_legend": false,
            "colors": ["#123456"]
        }))
        .unwrap();
        assert!(matches!(params.x_type, ChartXType::Time));
        assert_eq!(params.window, Some(60));
        assert!(!params.show_legend);
    }
}
