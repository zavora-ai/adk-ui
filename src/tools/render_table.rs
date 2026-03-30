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
    pub data: Vec<HashMap<String, Value>>,
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

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let params: RenderTableParams = serde_json::from_value(args)
            .map_err(|e| crate::compat::AdkError::tool(format!("Invalid parameters: {}", e)))?;
        let protocol_options = params.protocol.clone();

        let columns: Vec<TableColumn> = params
            .columns
            .into_iter()
            .map(|c| TableColumn {
                header: c.header,
                accessor_key: c.accessor_key,
                sortable: true,
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
            sortable: false,
            page_size: None,
            striped: false,
        }));

        let ui = UiResponse::new(components);
        render_ui_response_with_protocol(ui, &protocol_options, "table")
    }
}
