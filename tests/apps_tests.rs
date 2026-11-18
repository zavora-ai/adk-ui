//! `show_3d_app` end to end (feature `apps`): adk-ui's three.js server, an agent that calls
//! it, and genui-apps showing what it returns, framed sandboxed on a second origin.
//!
//! The server runs in process over `tokio::io::duplex`, and once as the real `adk-ui-apps`
//! binary over stdio. The harness, its sessions and genui-apps' host are real; the model is
//! scripted.

use std::sync::Arc;
use std::time::Duration;

use adk_core::{RequestContext, SessionId, SingleAgentLoader, UserId};
use adk_harness::{Harness, HarnessAgent, HarnessMode, InstructionsConfig};
use adk_model::{ScriptedLlm, ScriptedResponse};
use adk_runner::Runner;
use adk_server::auth_bridge::{RequestContextError, RequestContextExtractor};
use adk_server::{SecurityConfig, ServerBuilder, ServerConfig};
use adk_session::{InMemorySessionService, SessionService};
use adk_surface::genui::{AppOpen, AppServers, AppView, McpAppsHost};
use adk_surface::{Caller, LocalSource, Origin, SurfaceEngine, SurfaceKind, ThreadItem};
use adk_tool::mcp::rmcp::ServiceExt;
use adk_tool::mcp::rmcp::model::CallToolRequestParams;
use adk_tool::mcp::rmcp::transport::TokioChildProcess;
use adk_tool::mcp::{AdkClientHandler, AutoDeclineElicitationHandler, McpAppResource, McpToolset};
use adk_ui::apps::{CDN, RESOURCE_URI, THREE_VERSION, TOOL, ThreeApps};
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use futures::StreamExt;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt as _;

const APP: &str = "home";
const SESSION: &str = "s1";
const OWNER: &str = "ana";
const APPS_ORIGIN: &str = "http://localhost:41234";
const HOME_ORIGIN: &str = "http://127.0.0.1:8080";

const ORBITS: &str = "const sun = new THREE.Mesh(new THREE.SphereGeometry(1, 32, 16), \
    new THREE.MeshStandardMaterial({ color: theme.palette[2] }));\n\
    scene.add(sun);\nlabel('Sun', sun.position);\n\
    const earth = new THREE.Mesh(new THREE.SphereGeometry(0.3, 24, 12), \
    new THREE.MeshStandardMaterial({ color: theme.palette[0] }));\n\
    scene.add(earth);\n\
    onFrame((dt, t) => { earth.position.set(Math.cos(t) * 4, 0, Math.sin(t) * 4); });";

/// `Bearer ana` is the owner.
struct Owner;

#[async_trait]
impl RequestContextExtractor for Owner {
    async fn extract(
        &self,
        parts: &axum::http::request::Parts,
    ) -> Result<RequestContext, RequestContextError> {
        let user = parts
            .headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(RequestContextError::MissingAuth)?;
        Ok(RequestContext {
            user_id: user.to_string(),
            scopes: vec!["owner".into()],
            metadata: Default::default(),
        })
    }
}

/// The three.js server in process, as an agent's MCP toolset named "3D-sandbox".
async fn sandbox() -> (Arc<dyn adk_core::Toolset>, Arc<dyn AppServers>) {
    let (server_io, client_io) = tokio::io::duplex(1 << 20);
    tokio::spawn(async move {
        if let Ok(running) = ThreeApps::default().serve(server_io).await {
            let _ = running.waiting().await;
        }
    });
    let client = AdkClientHandler::new(Arc::new(AutoDeclineElicitationHandler))
        .with_apps()
        .serve(client_io)
        .await
        .unwrap();
    let toolset = Arc::new(McpToolset::new(client).with_name("3D-sandbox"));
    (
        toolset.clone() as Arc<dyn adk_core::Toolset>,
        toolset as Arc<dyn AppServers>,
    )
}

struct Home {
    engine: SurfaceEngine,
    host: McpAppsHost,
    api: axum::Router,
    _dir: tempfile::TempDir,
}

/// An agent in Ask first whose scripted model calls `show_3d_app` with each of `calls`, then
/// says "Here it is."; the engine over its sessions, and genui-apps' host and routes.
async fn home(calls: Vec<Value>) -> Home {
    let dir = tempfile::tempdir().unwrap();
    let (toolset, servers) = sandbox().await;
    let mut script: Vec<ScriptedResponse> = calls
        .into_iter()
        .map(|args| ScriptedResponse::function_call(TOOL, args))
        .collect();
    script.push(ScriptedResponse::text("Here it is."));
    let agent = Arc::new(
        HarnessAgent::builder("analyst")
            .model(Arc::new(ScriptedLlm::new("scripted", script)))
            .workspace(adk_devtools::Workspace::new(dir.path()))
            .project_instructions(InstructionsConfig {
                user_file: None,
                require_trust: false,
                ..InstructionsConfig::default()
            })
            .without_compaction()
            .without_default_members()
            .without_shell()
            .mode(HarnessMode::Ask)
            // `show_3d_app` reads only, as its server says, so Ask runs it unasked.
            .trust_mcp_read_only_hints(true)
            .toolset(toolset)
            .build()
            .unwrap(),
    );
    let sessions: Arc<dyn SessionService> = Arc::new(InMemorySessionService::new());
    let runner = agent
        .configure_runner(
            Runner::builder()
                .app_name(APP)
                .agent(agent.clone())
                .session_service(sessions.clone()),
        )
        .build()
        .unwrap();
    let harness = Harness::with_runner(agent.clone(), runner).unwrap();
    let user = UserId::new(OWNER).unwrap();
    let session = harness
        .open_session(&user, Some(&SessionId::new(SESSION).unwrap()))
        .await
        .unwrap();
    let stream = harness
        .send(&user, &session, "show me the planets moving")
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(20), stream.collect::<Vec<_>>())
        .await
        .expect("the turn finishes");

    let source = LocalSource::builder(APP)
        .sessions(sessions.clone())
        .build()
        .unwrap();
    let engine = adk_ui::catalogs::register(SurfaceEngine::builder(Arc::new(source)))
        .agent_name(APP, "Analyst")
        .build();
    let host = McpAppsHost::builder()
        .servers(servers)
        .apps_origin(APPS_ORIGIN)
        .home_origin(HOME_ORIGIN)
        .build();
    let config = ServerConfig::new(
        Arc::new(SingleAgentLoader::new(agent as Arc<dyn adk_core::Agent>)),
        sessions,
    )
    .with_security(SecurityConfig::development())
    .with_request_context(Arc::new(Owner));
    let api = ServerBuilder::new(config)
        .add_api_routes(adk_surface::web::router(engine.clone()))
        .add_api_routes(adk_surface::web::genui::apps_router(
            engine.clone(),
            host.clone(),
        ))
        .build();
    Home {
        engine,
        host,
        api,
        _dir: dir,
    }
}

fn web() -> Caller {
    Caller::human(OWNER, SurfaceKind::Web, Origin::Pointer).with_groups(["owner"])
}

async fn apps(home: &Home) -> Vec<AppView> {
    home.engine
        .thread(&web(), SESSION)
        .await
        .unwrap()
        .into_iter()
        .flat_map(|entry| entry.items)
        .filter_map(|item| match item {
            ThreadItem::App { app } => Some(*app),
            _ => None,
        })
        .collect()
}

async fn open(home: &Home, call_id: &str, surface: &str) -> AppOpen {
    let request = Request::builder()
        .uri(format!(
            "/api/surface/genui/sessions/{SESSION}/apps/{call_id}?surface={surface}"
        ))
        .header("host", "127.0.0.1:8080")
        .header("origin", HOME_ORIGIN)
        .header("authorization", format!("Bearer {OWNER}"))
        .body(Body::empty())
        .unwrap();
    let response = home.api.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(
        status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&bytes)
    );
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn the_server_publishes_one_model_only_tool_and_its_page() {
    let (_, servers) = sandbox().await;
    let bindings = servers.ui_bindings("3D-sandbox").await.unwrap().unwrap();
    assert_eq!(bindings.len(), 1);
    assert_eq!(
        (bindings[0].tool.as_str(), bindings[0].resource_uri.as_str()),
        (TOOL, RESOURCE_URI)
    );
    assert!(bindings[0].visibility.model && !bindings[0].visibility.app);

    let contents = servers
        .read_resource("3D-sandbox", RESOURCE_URI)
        .await
        .unwrap();
    let page = McpAppResource::from_contents(RESOURCE_URI, &contents).unwrap();
    assert_eq!(page.csp.resource_domains, vec![CDN.to_string()]);
    assert!(
        page.csp.connect_domains.is_empty(),
        "the page reaches no sites"
    );
    assert!(
        page.html
            .contains(&format!("three@{THREE_VERSION}/build/three.module.js"))
    );
}

#[tokio::test]
async fn a_3d_app_is_framed_sandboxed_on_the_apps_origin_and_never_on_the_phone() {
    let home = home(vec![
        json!({ "title": "The inner planets", "code": ORBITS, "height": 480 }),
    ])
    .await;
    let shown = apps(&home).await;
    assert_eq!(shown.len(), 1);
    let app = &shown[0];
    assert_eq!(
        app.header_text,
        "App from 3D sandbox · it can't approve anything"
    );
    assert!(
        app.result_text.contains("“The inner planets”"),
        "{}",
        app.result_text
    );
    assert!(
        !app.result_text.contains("THREE"),
        "the model's code is never echoed"
    );

    let opened = open(&home, &app.call_id, "web").await;
    let AppOpen::Shown {
        frame_url,
        frame_origin,
        tool_input,
        ..
    } = opened
    else {
        panic!("not shown: {opened:?}");
    };
    assert_eq!(frame_origin, APPS_ORIGIN);
    assert_eq!(
        tool_input["code"], ORBITS,
        "the code reaches the frame as the call's input"
    );

    let path = frame_url.strip_prefix(APPS_ORIGIN).unwrap();
    let page = home
        .host
        .sandbox_router()
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(page.status(), StatusCode::OK);
    let policy = page.headers()["content-security-policy"]
        .to_str()
        .unwrap()
        .to_string();
    let script = policy
        .split(';')
        .map(str::trim)
        .find(|d| d.starts_with("script-src"))
        .unwrap();
    assert!(
        script.contains("'unsafe-inline'") && script.contains(CDN),
        "{policy}"
    );
    for part in ["connect-src 'none'", "worker-src 'none'"] {
        assert!(policy.contains(part), "{part} in {policy}");
    }
    assert!(
        policy.contains(&format!("frame-ancestors {HOME_ORIGIN}")),
        "{policy}"
    );
    let body = page.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8_lossy(&body);
    assert!(body.contains("importmap"), "the template is what is framed");
    assert!(
        !body.contains("earth.position"),
        "the code is never part of the page"
    );

    let elsewhere = open(&home, &app.call_id, "phone").await;
    assert!(
        matches!(&elsewhere, AppOpen::Elsewhere { result_text, .. } if result_text == &app.result_text),
        "{elsewhere:?}"
    );
}

#[tokio::test]
async fn refused_code_shows_nothing() {
    let home = home(vec![
        json!({ "title": "Approve", "code": ORBITS }),
        json!({ "title": "Planets", "code": "fetch('https://example.com/a.glb')" }),
        json!({ "title": "Planets", "code": "label('Allow once', sun.position)" }),
        json!({ "title": "The inner planets", "code": ORBITS }),
    ])
    .await;
    let shown = apps(&home).await;
    assert_eq!(
        shown.len(),
        1,
        "a refused call writes no app record; the good one does"
    );
    assert!(shown[0].result_text.contains("“The inner planets”"));
}

#[tokio::test]
async fn the_binary_serves_the_tool_over_stdio() {
    let command = tokio::process::Command::new(env!("CARGO_BIN_EXE_adk-ui-apps"));
    let client = ().serve(TokioChildProcess::new(command).unwrap()).await.unwrap();
    let tools = client.list_all_tools().await.unwrap();
    assert_eq!(
        tools
            .iter()
            .map(|tool| tool.name.as_ref())
            .collect::<Vec<_>>(),
        vec![TOOL]
    );
    let arguments = json!({ "title": "Field", "code": "new WebSocket(u)" });
    let refused = client
        .call_tool(
            CallToolRequestParams::new(TOOL)
                .with_arguments(serde_json::from_value(arguments).unwrap()),
        )
        .await
        .unwrap();
    assert_eq!(refused.is_error, Some(true));
    client.cancel().await.unwrap();
}
