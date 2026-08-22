use crate::compat::{Result, Tool, ToolContext};
use crate::schema::*;
use crate::tools::{LegacyProtocolOptions, render_ui_response_with_protocol};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Parameters for the render_table tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RenderTableParams {
    /// Table title
    #[serde(default)]
    pub title: Option<String>,
    /// Column definitions
    pub columns: Vec<ColumnDef>,
    /// Row data - array of objects with keys matching accessor_key
    #[serde(default)]
    pub data: Vec<HashMap<String, Value>>,
    /// Optional data-model binding. Literal `data` remains the fallback.
    #[serde(default)]
    pub data_source: Option<DataSource>,
    #[serde(default)]
    pub sortable: bool,
    #[serde(default)]
    pub page_size: Option<u32>,
    #[serde(default)]
    pub striped: bool,
    /// Optional protocol output configuration.
    #[serde(flatten)]
    pub protocol: LegacyProtocolOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ColumnDef {
    /// Column header text
    pub header: String,
    /// Key to access data from row objects
    pub accessor_key: String,
    #[serde(default = "default_column_sortable")]
    pub sortable: bool,
}

fn default_column_sortable() -> bool {
    true
}

/// Tool for rendering data tables.
///
/// Creates tabular displays for structured data with customizable columns.
/// Supports optional title, sorting, pagination, and striped styling.
///
/// # Example JSON Parameters
///
/// ```json
/// {
///   "title": "User List",
///   "columns": [
///     { "header": "Name", "accessor_key": "name" },
///     { "header": "Email", "accessor_key": "email" },
///     { "header": "Role", "accessor_key": "role" }
///   ],
///   "data": [
///     { "name": "Alice", "email": "alice@example.com", "role": "Admin" },
///     { "name": "Bob", "email": "bob@example.com", "role": "User" }
///   ]
/// }
/// ```
pub struct RenderTableTool;

impl RenderTableTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RenderTableTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for RenderTableTool {
    fn name(&self) -> &str {
        "render_table"
    }

    fn description(&self) -> &str {
        r#"Render a data table. Output example:
┌───────┬─────────────────────┬───────┐
│ Name  │ Email               │ Role  │
├───────┼─────────────────────┼───────┤
│ Alice │ alice@example.com   │ Admin │
│ Bob   │ bob@example.com     │ User  │
└───────┴─────────────────────┴───────┘
Set sortable=true for clickable column headers. Set page_size for pagination. Set striped=true for alternating row colors."#
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(super::generate_gemini_schema::<RenderTableParams>())
    }

    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let params: RenderTableParams = serde_json::from_value(args)
            .map_err(|e| crate::compat::AdkError::tool(format!("Invalid parameters: {}", e)))?;
        if params.page_size == Some(0) {
            return Err(crate::compat::AdkError::tool(
                "Invalid parameters: page_size must be greater than zero".to_string(),
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

        let columns: Vec<TableColumn> = params
            .columns
            .into_iter()
            .map(|c| TableColumn {
                header: c.header,
                accessor_key: c.accessor_key,
                sortable: c.sortable,
            })
            .collect();

        let mut components = Vec::new();

        // Add title if provided
        if let Some(title) = params.title {
            components.push(Component::Text(Text {
                id: None,
                content: title,
                variant: TextVariant::H3,
            }));
        }

        components.push(Component::Table(Table {
            id: None,
            columns,
            data: params.data,
            data_source: params.data_source,
            sortable: params.sortable,
            page_size: params.page_size,
            striped: params.striped,
        }));

        let ui = UiResponse::new(components);
        let surface_id = protocol_options.resolved_surface_id("table");
        let output = render_ui_response_with_protocol(ui, &protocol_options, "table")?;
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
    fn parameters_expose_table_behavior() {
        let params: RenderTableParams = serde_json::from_value(json!({
            "columns": [{ "header": "Name", "accessor_key": "name", "sortable": false }],
            "data": [],
            "sortable": true,
            "page_size": 25,
            "striped": true
        }))
        .unwrap();
        assert!(params.sortable);
        assert_eq!(params.page_size, Some(25));
        assert!(params.striped);
        assert!(!params.columns[0].sortable);
    }
}
