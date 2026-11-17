//! Model-written three.js, shown only in genui-apps' sandbox (feature `apps`).
//!
//! Some 3D is beyond what a catalog can declare, such as a solar system in motion, a particle
//! field or a procedural landscape. [`ThreeApps`] is an MCP server with one tool,
//! `show_3d_app`, which takes the model's three.js code and shows it as an MCP App
//! (2026-01-26). The code is never a drawing: genui-apps frames it sandboxed on a second origin,
//! under "App from 3D sandbox · it can't approve anything", where:
//! - it can't reach the network (`connect-src 'none'`) or start a worker;
//! - it can't touch the home; every tool call it could ask for is held for the person;
//! - this server publishes no tool the app may call (`visibility: ["model"]`).
//!
//! Before an app record is written, the server refuses code that shows a decision's words or
//! reaches outside its frame ([`check`]). The page loads three.js
//! ([`THREE_VERSION`]) and its addons from [`CDN`], the one site it declares, and runs the code
//! as the body of an async function given `{THREE, addons, scene, camera, renderer, controls,
//! onFrame, label, size, theme, reducedMotion}`. The page owns the render loop, which pauses
//! off screen, honours reduced motion and slows down for heavy scenes.
//!
//! ```rust,no_run
//! # async fn serve() -> Result<(), Box<dyn std::error::Error>> {
//! use rmcp::ServiceExt;
//!
//! let running = adk_ui::apps::ThreeApps::default().serve(rmcp::transport::stdio()).await?;
//! running.waiting().await?;
//! # Ok(())
//! # }
//! ```

pub mod check;

use std::sync::Arc;

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, ErrorData,
    Implementation, InitializeResult, JsonObject, ListResourcesResult, ListToolsResult, MetaObject,
    PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResponse, ReadResourceResult,
    Resource, ResourceContents, ServerCapabilities, Tool, ToolAnnotations,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler};
use serde_json::{Value, json};

pub use check::{AppInput, refusal};

/// The tool's name.
pub const TOOL: &str = "show_3d_app";
/// The page's resource.
pub const RESOURCE_URI: &str = "ui://adk-ui/three-app";
/// MCP Apps' page type.
pub const APP_MIME_TYPE: &str = "text/html;profile=mcp-app";
/// The three.js release the page loads.
pub const THREE_VERSION: &str = "0.185.1";
/// The one site the page declares: three.js and its addons, pinned to [`THREE_VERSION`].
pub const CDN: &str = "https://cdn.jsdelivr.net";
/// The page's template. The code arrives as the call's input, after it starts.
pub const PAGE: &str = include_str!("three_app.html");

/// The MCP server behind `show_3d_app`. It serves over any rmcp transport: stdio
/// (`adk-ui-apps`), or in process over `tokio::io::duplex`.
#[derive(Debug, Clone, Default)]
pub struct ThreeApps {
    _private: (),
}

impl ThreeApps {
    /// The `show_3d_app` tool as `tools/list` lists it.
    pub fn tool() -> Tool {
        let schema: JsonObject = serde_json::from_value(json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["title", "code"],
            "properties": {
                "title": { "type": "string", "minLength": 1, "maxLength": check::MAX_TITLE_CHARS,
                           "description": "What the app shows, in a few words." },
                "description": { "type": "string", "maxLength": check::MAX_DESCRIPTION_CHARS,
                                 "description": "One line under the title." },
                "code": { "type": "string", "minLength": 1, "maxLength": check::MAX_CODE_BYTES,
                          "description": "The body of an async JavaScript function (see the tool description)." },
                "height": { "type": "integer", "minimum": check::HEIGHTS.start(),
                            "maximum": check::HEIGHTS.end(), "default": check::DEFAULT_HEIGHT },
            },
        }))
        .expect("the input schema is an object");
        let mut tool = Tool::new(TOOL, description(), Arc::new(schema)).annotate(
            ToolAnnotations::new()
                .read_only(true)
                .destructive(false)
                .idempotent(true)
                .open_world(false),
        );
        tool.title = Some("Show a 3D app".into());
        tool.meta = Some(meta(json!({
            "ui": { "resourceUri": RESOURCE_URI, "visibility": ["model"] }
        })));
        tool
    }

    /// The page as `resources/read` returns it.
    pub fn page() -> ResourceContents {
        ResourceContents::text(PAGE, RESOURCE_URI)
            .with_mime_type(APP_MIME_TYPE)
            .with_meta(meta(page_meta()))
    }

    /// Runs one `show_3d_app` call: the text the model reads, or why it was refused.
    pub fn show(arguments: Option<JsonObject>) -> CallToolResult {
        let input: AppInput =
            match serde_json::from_value(Value::Object(arguments.unwrap_or_default())) {
                Ok(input) => input,
                Err(error) => return refused(&format!("its arguments don't fit: {error}")),
            };
        if let Some(reason) = refusal(&input) {
            return refused(&reason);
        }
        CallToolResult::success(vec![ContentBlock::text(format!(
            "A 3D app, “{}”, is in the person's thread. It shows in the web home on this computer; \
             elsewhere the person sees this text.",
            input.title.trim()
        ))])
    }
}

/// The tool's description: what the code is given and what it may not do.
fn description() -> String {
    format!(
        "Shows the person a 3D scene you write in three.js ({THREE_VERSION}), framed in their \
         thread as an app that can't approve anything. Use it only for 3D the catalog's \
         components can't declare (motion, particles, procedural shapes); for charts, \
         networks, globes and laid-out scenes draw with render_screen instead.\n\
         `code` is the body of an async function given {{ THREE, addons, scene, camera, \
         renderer, controls, onFrame, label, size, theme, reducedMotion }}:\n\
         - add objects to `scene`; the camera starts at (6, 5, 8) looking at the origin, with \
         orbit controls and soft lights already in place;\n\
         - `onFrame((dt, t) => ...)` animates; the page runs the loop, pauses it off screen and \
         under reduced motion;\n\
         - `label(text, position, parent?)` puts readable text at a point;\n\
         - `theme.palette` holds 8 colours for the person's light or dark scheme; \
         `theme.background` and `theme.text` too;\n\
         - `addons` has OrbitControls, CSS2DRenderer, CSS2DObject, BufferGeometryUtils, \
         ParametricGeometry, ConvexGeometry, RoundedBoxGeometry, ImprovedNoise, SimplexNoise, \
         Line2, LineMaterial and LineGeometry.\n\
         The app reaches no sites: no URLs, imports, fetch, fonts, textures from the web, \
         workers, storage or messages to the page. Keep it under {} KiB and light enough for a \
         laptop. Never show a decision's words such as Allow once or Approve.",
        check::MAX_CODE_BYTES / 1024
    )
}

/// The page's `_meta`: the CDN it loads three.js from, and a border.
fn page_meta() -> Value {
    json!({ "ui": { "csp": { "resourceDomains": [CDN] }, "prefersBorder": true } })
}

fn meta(value: Value) -> MetaObject {
    MetaObject(serde_json::from_value(value).expect("_meta is an object"))
}

fn refused(reason: &str) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(format!(
        "Not shown: {reason}. Change the code and call {TOOL} again."
    ))])
}

impl ServerHandler for ThreeApps {
    fn get_info(&self) -> InitializeResult {
        InitializeResult::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new(
            "adk-ui-apps",
            env!("CARGO_PKG_VERSION"),
        ))
        .with_instructions(
            "show_3d_app shows model-written three.js, sandboxed, in the person's thread.",
        )
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(vec![Self::tool()]))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        if request.name != TOOL {
            return Err(ErrorData::invalid_params(
                format!("no tool {}", request.name),
                None,
            ));
        }
        Ok(CallToolResponse::Complete(Self::show(request.arguments)))
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult::with_all_items(vec![
            Resource::new(RESOURCE_URI, "three-app")
                .with_title("A 3D app")
                .with_mime_type(APP_MIME_TYPE)
                .with_meta(meta(page_meta())),
        ]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        if request.uri != RESOURCE_URI {
            return Err(ErrorData::resource_not_found(
                format!("no resource {}", request.uri),
                None,
            ));
        }
        Ok(ReadResourceResult::new(vec![Self::page()]).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adk_surface::genui::reserved;

    #[test]
    fn the_page_shows_no_decision_words_and_reaches_only_the_cdn() {
        for line in PAGE.lines() {
            assert_eq!(reserved(line), None, "{line}");
        }
        for url in PAGE
            .split(['"', '\'', '`', ' '])
            .filter(|part| part.starts_with("http"))
        {
            assert!(
                url.starts_with(&format!("{CDN}/npm/three@{THREE_VERSION}/")),
                "{url}"
            );
        }
        assert!(
            !PAGE.contains("import("),
            "the page imports nothing at run time"
        );
    }

    #[test]
    fn the_tool_declares_its_page_for_the_model_only() {
        let tool = ThreeApps::tool();
        let ui = &tool.meta.as_ref().unwrap().0["ui"];
        assert_eq!(ui["resourceUri"], RESOURCE_URI);
        assert_eq!(ui["visibility"], json!(["model"]));
        assert!(description().len() < 8 * 1024);
        assert_eq!(
            reserved(&description()),
            Some("Allow once"),
            "it names what never to show"
        );
    }

    #[test]
    fn a_refused_call_is_an_error_and_never_echoes_the_code() {
        let code = "fetch('https://example.com/secret-data')";
        let result = ThreeApps::show(Some(
            serde_json::from_value(json!({ "title": "Field", "code": code })).unwrap(),
        ));
        assert_eq!(result.is_error, Some(true));
        let text = serde_json::to_string(&result.content).unwrap();
        assert!(!text.contains("secret-data"));
        let shown = ThreeApps::show(Some(
            serde_json::from_value(
                json!({ "title": "Field", "code": "scene.add(new THREE.Mesh())" }),
            )
            .unwrap(),
        ));
        assert_ne!(shown.is_error, Some(true));
        assert!(
            !serde_json::to_string(&shown.content)
                .unwrap()
                .contains("scene.add")
        );
    }
}
