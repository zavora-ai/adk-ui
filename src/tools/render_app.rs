use crate::application::{UiApplication, UiApplicationPage};
use crate::compat::{Result, Tool, ToolContext};
use crate::schema::{Component, UiResponse};
use crate::tools::{LegacyProtocolOptions, render_ui_response_with_protocol};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::sync::Arc;

/// Parameters for rendering a complete, navigable application.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RenderAppParams {
    pub application: UiApplication,
    /// Optional protocol output configuration. Every page is also emitted as an
    /// independent surface for clients that do not understand application graphs.
    #[serde(flatten)]
    pub protocol: LegacyProtocolOptions,
}

pub struct RenderAppTool;

impl RenderAppTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RenderAppTool {
    fn default() -> Self {
        Self::new()
    }
}

fn page_components(page: &UiApplicationPage) -> Vec<Component> {
    let mut components = Vec::new();
    if let Some(hero) = &page.hero {
        components.extend(hero.content.clone());
        components.extend(hero.actions.clone());
        components.extend(hero.visual.clone());
    }
    for region in &page.regions {
        components.extend(region.components.clone());
    }
    components.extend(page.aside.clone());
    components.extend(page.footer.clone());
    components
}

fn add_application_fields(
    primary: Value,
    application: &UiApplication,
    surfaces: Vec<Value>,
) -> Result<Value> {
    let mut object = match primary {
        Value::Object(object) => object,
        other => {
            let mut object = Map::new();
            object.insert("payload".to_string(), other);
            object
        }
    };
    object.insert(
        "application".to_string(),
        serde_json::to_value(application).map_err(|error| {
            crate::compat::AdkError::tool(format!("Failed to serialize application: {error}"))
        })?,
    );
    object.insert("surfaces".to_string(), Value::Array(surfaces));
    object.insert("application_version".to_string(), json!("1.0"));
    Ok(Value::Object(object))
}

#[async_trait]
impl Tool for RenderAppTool {
    fn name(&self) -> &str {
        "render_app"
    }

    fn description(&self) -> &str {
        "Render a complete multi-page application with brand-aware page composition, navigation, shared state, and optional declarative 3D scenes. Use this instead of render_page for products, sites, dashboards, portals, and end-to-end workflows with multiple routes."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(super::generate_gemini_schema::<RenderAppParams>())
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let params: RenderAppParams = serde_json::from_value(args).map_err(|error| {
            crate::compat::AdkError::tool(format!("Invalid parameters: {error}"))
        })?;
        params.application.validate().map_err(|errors| {
            crate::compat::AdkError::tool(format!("Invalid application: {}", errors.join("; ")))
        })?;

        let mut surfaces = Vec::with_capacity(params.application.pages.len());
        for page in &params.application.pages {
            let mut options = params.protocol.clone();
            options.surface_id = Some(format!("{}-{}", params.application.id, page.id));
            if options.kit_id.is_none() {
                options.kit_id = params.application.kit_id.clone();
            }
            let ui = UiResponse::new(page_components(page))
                .with_id(format!("{}:{}", params.application.id, page.id));
            let ui = match &params.application.kit_id {
                Some(kit_id) => ui.with_kit_id(kit_id),
                None => ui,
            };
            surfaces.push(render_ui_response_with_protocol(ui, &options, &page.id)?);
        }

        let primary_index = params
            .application
            .pages
            .iter()
            .position(|page| page.route == params.application.initial_route)
            .unwrap_or(0);
        let primary = surfaces
            .get(primary_index)
            .cloned()
            .unwrap_or_else(|| json!({}));
        if params.protocol.protocol.is_none() {
            return Ok(json!({
                "application_version": "1.0",
                "application": params.application,
                "surfaces": surfaces,
            }));
        }
        add_application_fields(primary, &params.application, surfaces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::*;
    use crate::compat::{Content, EventActions, ReadonlyContext};
    use crate::interop::UiProtocol;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct TestContext {
        content: Content,
        actions: Mutex<EventActions>,
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
            "call"
        }
        fn actions(&self) -> EventActions {
            self.actions.lock().unwrap().clone()
        }
        fn set_actions(&self, actions: EventActions) {
            *self.actions.lock().unwrap() = actions;
        }
        async fn search_memory(
            &self,
            _: &str,
        ) -> crate::compat::Result<Vec<crate::compat::MemoryEntry>> {
            Ok(vec![])
        }
    }

    fn application() -> UiApplication {
        UiApplication {
            id: "app".into(),
            name: "App".into(),
            kit_id: None,
            initial_route: "/".into(),
            navigation: vec![],
            state: HashMap::new(),
            pages: vec![UiApplicationPage {
                id: "home".into(),
                route: "/".into(),
                title: "Home".into(),
                description: None,
                template: UiPageTemplate::Dashboard,
                eyebrow: None,
                hero: None,
                regions: vec![UiPageRegion {
                    id: "main".into(),
                    title: None,
                    description: None,
                    layout: UiRegionLayout::Stack,
                    columns: 1,
                    tone: UiRegionTone::Default,
                    components: vec![crate::schema::Component::Text(crate::schema::Text {
                        id: None,
                        content: "Hello".into(),
                        variant: crate::schema::TextVariant::Body,
                    })],
                }],
                aside: vec![],
                footer: vec![],
                atmosphere: UiPageAtmosphere::Clean,
            }],
        }
    }

    #[tokio::test]
    async fn emits_application_and_protocol_fallback_surfaces() {
        let args = serde_json::to_value(RenderAppParams {
            application: application(),
            protocol: LegacyProtocolOptions {
                protocol: Some(UiProtocol::A2ui),
                ..Default::default()
            },
        })
        .unwrap();
        let ctx: Arc<dyn ToolContext> = Arc::new(TestContext {
            content: Content::new("user"),
            actions: Mutex::new(EventActions::default()),
        });
        let value = RenderAppTool.execute(ctx, args).await.unwrap();
        assert_eq!(value["protocol"], "a2ui");
        assert_eq!(value["application"]["initial_route"], "/");
        assert_eq!(value["surfaces"].as_array().unwrap().len(), 1);
    }
}
