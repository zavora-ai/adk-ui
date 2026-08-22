use crate::compat::{Result, Tool, ToolContext};
use crate::kit::{KitArtifacts, KitGenerator, KitSpec};
use crate::tools::LegacyProtocolOptions;
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;

/// Parameters for the render_kit tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RenderKitParams {
    #[serde(flatten)]
    pub spec: KitSpec,
    /// Optional output format; "json" (default) or "catalog_only"
    #[serde(default)]
    pub output: Option<String>,
    /// Optional protocol output configuration.
    #[serde(flatten)]
    pub protocol: LegacyProtocolOptions,
}

/// Tool for generating a reviewable UI kit draft.
pub struct RenderKitTool;

impl RenderKitTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RenderKitTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for RenderKitTool {
    fn name(&self) -> &str {
        "render_kit"
    }

    fn description(&self) -> &str {
        "Generate a versioned brand-kit draft from a KitSpec. Returns a manifest, supported component catalog, semantic tokens, templates, agent constraints, and scoped theme CSS. Drafts must be reviewed before runtime approval."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(super::generate_gemini_schema::<RenderKitParams>())
    }

    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let params: RenderKitParams = serde_json::from_value(args.clone()).map_err(|e| {
            crate::compat::AdkError::tool(format!("Invalid parameters: {}. Got: {}", e, args))
        })?;

        let generator = KitGenerator::new();
        let artifacts = generator.generate(&params.spec);
        if let Err(errors) = artifacts.manifest.validate() {
            let details = errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ");
            return Err(crate::compat::AdkError::tool(format!(
                "Generated kit failed validation: {details}"
            )));
        }
        let payload = format_output(&artifacts, params.output.as_deref());

        let surface_id = params.protocol.resolved_surface_id("kit");
        let output = match params.protocol.protocol {
            Some(protocol) => {
                let protocol = serde_json::to_value(protocol).unwrap_or_else(|_| json!("a2ui"));
                json!({
                    "protocol": protocol,
                    "surface_id": surface_id.clone(),
                    "payload": payload
                })
            }
            None => payload,
        };
        let surface_ref = crate::surface_runtime::next_surface_ref(&ctx, surface_id);
        crate::surface_runtime::record_surface_ref(&ctx, &surface_ref);
        Ok(output)
    }
}

fn format_output(artifacts: &KitArtifacts, output: Option<&str>) -> Value {
    match output {
        Some("catalog_only") => artifacts.catalog.clone(),
        _ => json!({
            "manifest": artifacts.manifest,
            "catalog": artifacts.catalog,
            "tokens": artifacts.tokens,
            "templates": artifacts.templates,
            "agent_context": artifacts.agent_context,
            "theme_css": artifacts.theme_css,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compat::{Content, EventActions, ReadonlyContext};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    struct TestContext {
        content: Content,
        actions: Mutex<EventActions>,
    }

    impl TestContext {
        fn new() -> Self {
            Self {
                content: Content::new("user"),
                actions: Mutex::new(EventActions::default()),
            }
        }
    }

    #[async_trait]
    impl ReadonlyContext for TestContext {
        fn invocation_id(&self) -> &str {
            "test"
        }
        fn agent_name(&self) -> &str {
            "test"
        }
        fn user_id(&self) -> &str {
            "user"
        }
        fn app_name(&self) -> &str {
            "app"
        }
        fn session_id(&self) -> &str {
            "session"
        }
        fn branch(&self) -> &str {
            ""
        }
        fn user_content(&self) -> &Content {
            &self.content
        }
    }

    #[async_trait]
    impl crate::compat::CallbackContext for TestContext {
        fn artifacts(&self) -> Option<Arc<dyn crate::compat::Artifacts>> {
            None
        }
    }

    #[async_trait]
    impl ToolContext for TestContext {
        fn function_call_id(&self) -> &str {
            "call-123"
        }
        fn actions(&self) -> EventActions {
            self.actions.lock().unwrap().clone()
        }
        fn set_actions(&self, actions: EventActions) {
            *self.actions.lock().unwrap() = actions;
        }
        async fn search_memory(&self, _query: &str) -> Result<Vec<crate::compat::MemoryEntry>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn render_kit_emits_catalog() {
        let tool = RenderKitTool::new();
        let args = serde_json::json!({
            "name": "Fintech Pro",
            "version": "0.1.0",
            "brand": { "vibe": "trustworthy", "industry": "fintech" },
            "colors": { "primary": "#2F6BFF" },
            "typography": { "family": "Source Sans 3" },
            "assets": [{
                "id": "primary-logo",
                "kind": "logo",
                "uri": "/assets/fintech-pro.svg",
                "mimeType": "image/svg+xml",
                "alt": "Fintech Pro"
            }],
            "provenance": {
                "source": "imported",
                "sourceUri": "https://brand.example.com/design-system"
            },
            "templates": ["auth_login"]
        });

        let ctx: Arc<dyn ToolContext> = Arc::new(TestContext::new());
        let value = tool.execute(ctx, args).await.unwrap();
        assert_eq!(value["manifest"]["status"], "draft");
        assert!(value.get("catalog").is_some());
        assert!(value.get("tokens").is_some());
        assert!(value["catalog"]["components"]["Card"].is_object());
        assert_eq!(value["manifest"]["assets"][0]["id"], "primary-logo");
    }

    #[tokio::test]
    async fn render_kit_emits_protocol_envelope() {
        let tool = RenderKitTool::new();
        let args = serde_json::json!({
            "name": "Fintech Pro",
            "version": "0.1.0",
            "brand": { "vibe": "trustworthy", "industry": "fintech" },
            "colors": { "primary": "#2F6BFF" },
            "typography": { "family": "Source Sans 3" },
            "protocol": "mcp_apps"
        });

        let ctx: Arc<dyn ToolContext> = Arc::new(TestContext::new());
        let value = tool.execute(ctx, args).await.unwrap();
        assert_eq!(value["protocol"], "mcp_apps");
        assert!(value["payload"]["catalog"].is_object());
    }
}
