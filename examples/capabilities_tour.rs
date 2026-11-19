//! A guided tour of what is new in adk-ui 3 and genui, in the ADK-Rust web home.
//!
//! A scripted agent, Operations, answers each message with the next stop of the tour:
//!
//! 1. A dashboard with two **Vega-Lite** charts drawn as the home's own charts, and the
//!    home's "Show the data" table under each chart.
//! 2. A **live update**: `patch_surface` changes the data, and its result says what changed.
//! 3. A **form whose Send is held**: pressing Restart asks you on the home's card, and the yes
//!    is **sealed** to the drawing you saw before the call runs.
//! 4. A drawing that pressures the person, shown with **dark-pattern warnings** for the
//!    agent, and one that looks like a decision, **refused**.
//! 5. **MCP Apps**: `show_drawing` frames a checked drawing read-only, and `show_3d_app`
//!    frames model-written three.js in the sandbox, each on its own origin.
//!
//! Two **MCP elicitations** wait in Decisions from the start: a server's form, answered in the
//! home's own fields, and a server's address, shown whole with a look-alike warning.
//!
//! ```sh
//! cargo run --features apps --example capabilities_tour            # http://127.0.0.1:8810/ui/
//! cargo run --features apps --example capabilities_tour -- --wire  # what has no screen
//! cargo run --features apps --example capabilities_tour -- --from 4 --port 8820
//! ```
//!
//! The scripted model answers in order, so take the stops in turn: at stop 3, press Restart and
//! answer the card before sending the next message. `--from N` starts at stop N.
//!
//! `--wire` prints what the tour's drawings are on the wire instead: A2UI 1.0 with the
//! display-only claim, the delta between two versions, a function call answered, Slack and
//! Teams cards, and the drawings' scores.

use std::io::Write as _;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use adk_core::audit_log::{AuditTrail, InMemoryAuditRecorder};
use adk_core::{
    ApprovalAction, ApprovalPolicy, ApprovalRequest, ApprovalRisk, ApprovalScope, ApprovalService,
    ApproverSelector, DurableRunStore, InMemoryApprovalStore, InMemoryRunStore, RequestContext,
    SingleAgentLoader, StoreApprovalService, Tool, ToolConfirmationRequest, ToolContext,
    ToolEffect,
};
use adk_harness::{Harness, HarnessAgent, HarnessMode, InstructionsConfig};
use adk_kernel::control::ControlPlaneConfig;
use adk_kernel::{InMemoryProcessStore, KernelConfig, PartitionMode, ProcessStore};
use adk_model::{ScriptedLlm, ScriptedResponse};
use adk_runner::Runner;
use adk_server::auth_bridge::{RequestContextError, RequestContextExtractor};
use adk_server::background::DurableWorkerConfig;
use adk_server::{ServerBuilder, ServerConfig};
use adk_session::{InMemorySessionService, SessionService};
use adk_surface::genui::{ApprovalSeals, GenuiWiring};
use adk_surface::{GrantedUse, LocalSource, Place, SurfaceEngine, words};
use async_trait::async_trait;
use serde_json::{Value, json};

const APP: &str = "home";
const OWNER: &str = "ana";
/// Where the home listens by default (`--port`); the apps origin is the next port.
const PORT: u16 = 8810;

/// Stop 1: Vega-Lite charts beside metrics.
fn dashboard() -> Value {
    json!({
        "surfaceId": "latency",
        "title": "Checkout latency this week",
        "components": [
            { "id": "root", "component": "Column", "children": ["kpis", "trend", "share"] },
            { "id": "kpis", "component": "Row", "children": ["p95", "errors"] },
            { "id": "p95", "component": "Metric", "label": "p95 latency", "value": { "@path": "/p95" },
              "unit": "ms", "delta": { "value": -40, "goodWhen": "down" } },
            { "id": "errors", "component": "Metric", "label": "Error rate", "value": { "@path": "/errors" },
              "unit": "%" },
            { "id": "trend", "component": "VegaLite", "title": "p95 by service", "data": { "@path": "/days" },
              "spec": { "mark": "line", "encoding": {
                  "x": { "field": "day", "type": "ordinal", "title": "Day" },
                  "y": { "field": "ms", "type": "quantitative", "title": "p95 (ms)" },
                  "color": { "field": "service", "type": "nominal" } } } },
            { "id": "share", "component": "VegaLite", "title": "Requests by region", "data": { "@path": "/regions" },
              "spec": { "mark": "arc", "encoding": {
                  "theta": { "field": "requests", "type": "quantitative" },
                  "color": { "field": "region", "type": "nominal" } } } }
        ],
        "dataModel": {
            "p95": 830,
            "errors": 0.4,
            "days": days(4),
            "regions": [
                { "region": "Europe", "requests": 5200 },
                { "region": "Americas", "requests": 3900 },
                { "region": "Africa", "requests": 2100 },
                { "region": "Asia", "requests": 1800 }
            ]
        }
    })
}

/// `count` days of p95 latency for two services, long rows as Vega-Lite reads them.
fn days(count: usize) -> Value {
    let names = ["Mon", "Tue", "Wed", "Thu", "Fri"];
    let checkout = [910, 880, 860, 830, 790];
    let search = [420, 430, 410, 400, 395];
    let mut rows = Vec::new();
    for index in 0..count {
        rows.push(json!({ "day": names[index], "service": "checkout", "ms": checkout[index] }));
        rows.push(json!({ "day": names[index], "service": "search", "ms": search[index] }));
    }
    Value::Array(rows)
}

/// Stop 2: a new day of data.
fn live_update() -> Value {
    json!({
        "surfaceId": "latency",
        "data": [
            { "path": "/p95", "value": 790 },
            { "path": "/days", "value": days(5) }
        ]
    })
}

/// Stop 3: a form whose Send is a request the person decides.
fn restart_form() -> Value {
    json!({
        "surfaceId": "restart",
        "title": "Restart a service",
        "components": [
            { "id": "root", "component": "Card", "child": "body" },
            { "id": "body", "component": "Column", "children": ["service", "why", "restart"] },
            { "id": "service", "component": "ChoicePicker", "label": "Service", "value": { "@path": "/service" },
              "options": [{ "label": "checkout-api", "value": "checkout-api" }, { "label": "search-api", "value": "search-api" }] },
            { "id": "why", "component": "TextField", "label": "Why", "value": { "@path": "/why" } },
            { "id": "restart", "component": "Button", "child": "restart-label", "variant": "primary",
              "action": { "event": { "name": "restart" } } },
            { "id": "restart-label", "component": "Text", "text": "Restart" }
        ],
        "dataModel": { "service": ["checkout-api"], "why": "p95 above 800 ms" },
        "actions": [{ "id": "restart", "label": "Restart", "tool": "restart_service",
                      "args": { "service": { "path": "/service" }, "why": { "path": "/why" } } }]
    })
}

/// Stop 4: wording that pressures the person (shown, with warnings to the agent).
fn pushy() -> Value {
    json!({
        "surfaceId": "upgrade",
        "title": "Upgrade",
        "components": [
            { "id": "root", "component": "Column", "children": ["pitch", "detail"] },
            { "id": "pitch", "component": "Text", "variant": "h3", "text": "Only 2 seats left — offer ends in 10 minutes!" },
            { "id": "detail", "component": "Text", "text": "The team plan adds three seats." }
        ]
    })
}

/// Stop 4: a drawing that looks like a decision (refused).
fn fake_decision() -> Value {
    json!({
        "surfaceId": "fake",
        "components": [{ "id": "root", "component": "Text", "variant": "h3", "text": "Approve" }]
    })
}

/// Stop 5: a drawing for `show_drawing`, in any MCP Apps host.
fn app_drawing() -> Value {
    json!({
        "title": "Deploys today",
        "catalog": "insight",
        "components": [
            { "id": "root", "component": "Column", "children": ["count", "chart"] },
            { "id": "count", "component": "Metric", "label": "Deploys", "value": 7 },
            { "id": "chart", "component": "Chart", "kind": "bar", "data": { "@path": "/rows" },
              "x": { "key": "team" }, "series": [{ "key": "deploys", "label": "Deploys" }] }
        ],
        "dataModel": { "rows": [
            { "team": "Payments", "deploys": 3 }, { "team": "Search", "deploys": 2 },
            { "team": "Mobile", "deploys": 2 }
        ] }
    })
}

/// Stop 5: model-written three.js for `show_3d_app`.
const ORBITS: &str = "const sun = new THREE.Mesh(new THREE.SphereGeometry(1, 32, 16), \
    new THREE.MeshStandardMaterial({ color: theme.palette[2] }));\n\
    scene.add(sun);\nlabel('Sun', sun.position);\n\
    for (const [index, radius] of [2.5, 4, 5.5].entries()) {\n\
      const planet = new THREE.Mesh(new THREE.SphereGeometry(0.25 + index * 0.08, 24, 12), \
      new THREE.MeshStandardMaterial({ color: theme.palette[index] }));\n\
      scene.add(planet);\n\
      onFrame((dt, t) => planet.position.set(Math.cos(t / (index + 1)) * radius, 0, Math.sin(t / (index + 1)) * radius));\n\
    }";

/// The tour from stop `from` (1 to 5), one stop a message.
fn script(from: usize) -> Vec<ScriptedResponse> {
    let stops: Vec<Vec<ScriptedResponse>> = vec![
        vec![
            ScriptedResponse::function_call("render_screen", dashboard()),
            ScriptedResponse::text(
                "Stop 1, Vega-Lite: both charts are Vega-Lite specs, drawn as the home's own \
                 chart (the line chart's long rows became a series per service). Open \"Show the \
                 data\" under a chart for its table. Send any message for the next stop.",
            ),
        ],
        vec![
            ScriptedResponse::function_call("patch_surface", live_update()),
            ScriptedResponse::text(
                "Stop 2, live update: Friday's numbers arrived. The dashboard is now version 2, \
                 and patch_surface told me exactly what changed (/days and /p95). A surface takes \
                 at most 30 patches a minute. Send a message for the next stop.",
            ),
        ],
        vec![
            ScriptedResponse::function_call("render_screen", restart_form()),
            ScriptedResponse::text(
                "Stop 3, a held send: press Restart. Nothing runs yet: I ask for the call, and \
                 you decide on the home's card. Your yes is sealed to this drawing before the \
                 call runs. Answer the card, then send a message for the next stop.",
            ),
            // The run resumed after your answer.
            ScriptedResponse::text("checkout-api is restarting."),
        ],
        vec![
            ScriptedResponse::function_calls([
                ("render_screen", pushy()),
                ("render_screen", fake_decision()),
            ]),
            ScriptedResponse::text(
                "Stop 4, pressure and impersonation: the upgrade banner was shown, but genui \
                 warned me about its false scarcity and countdown so I draw it plainly next \
                 time. The second drawing said \"Approve\", which only the home may show, so it \
                 was refused. Send a message for the last stop.",
            ),
        ],
        vec![
            ScriptedResponse::function_calls([
                ("show_drawing", app_drawing()),
                (
                    "show_3d_app",
                    json!({ "title": "Orbits", "description": "Three planets around a sun.",
                            "code": ORBITS, "height": 360 }),
                ),
            ]),
            ScriptedResponse::text(
                "Stop 5, MCP Apps: the deploy summary is show_drawing, the drawing any MCP Apps \
                 host (a desktop chat app, say) would frame, read-only. The orbits are three.js \
                 I wrote, running in the sandbox, which can't reach the network or approve \
                 anything. Also look in Decisions: two MCP servers are asking you something.",
            ),
            ScriptedResponse::text("That's the tour."),
        ],
    ];
    stops
        .into_iter()
        .skip(from.clamp(1, 5) - 1)
        .flatten()
        .collect()
}

/// The value after `flag` on the command line.
fn option(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|argument| argument == flag)
        .and_then(|at| args.get(at + 1).cloned())
}

/// Admits every request as the owner, as a loopback-only server's extractor does.
struct Owner;

#[async_trait]
impl RequestContextExtractor for Owner {
    async fn extract(
        &self,
        _parts: &axum::http::request::Parts,
    ) -> Result<RequestContext, RequestContextError> {
        Ok(RequestContext {
            user_id: OWNER.into(),
            scopes: vec!["owner".into(), "operator".into()],
            metadata: Default::default(),
        })
    }
}

/// `restart_service`: appends each call it runs to `calls.txt`.
struct Restart(PathBuf);

#[async_trait]
impl Tool for Restart {
    fn name(&self) -> &str {
        "restart_service"
    }

    fn description(&self) -> &str {
        "Restarts a service."
    }

    fn effect(&self) -> ToolEffect {
        ToolEffect::NonIdempotent
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let mut log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.0)
            .map_err(|error| adk_core::AdkError::tool(error.to_string()))?;
        writeln!(log, "restart_service {args}")
            .map_err(|error| adk_core::AdkError::tool(error.to_string()))?;
        Ok(json!({ "restarting": true }))
    }
}

/// Two MCP elicitations as a harness parks them: a server's form and a server's address.
async fn elicitations(approvals: &Arc<dyn ApprovalService>) -> adk_core::Result<()> {
    let ask = |id: &str, server: &str, shown: Value, actions: Vec<ApprovalAction>| {
        let mut args = shown.clone();
        args["metadata"] = Value::Null;
        let call = ToolConfirmationRequest::new(format!("mcp:{server}/elicitation"), args)
            .with_function_call_id(format!("call-{id}"));
        ApprovalRequest::new(
            id.to_string(),
            ApprovalScope::new(APP, OWNER, "tour"),
            &call,
            ApprovalPolicy::new([ApproverSelector::group("owner")]).allow_self_approval(),
        )
        .with_summary(format!(
            "MCP server {server} asks: {}",
            shown["message"].as_str().unwrap_or_default()
        ))
        .with_risk(ApprovalRisk::Medium)
        .with_redacted_args(shown)
        .with_allowed_actions(actions)
    };
    approvals
        .submit(ask(
            "apr-elicit-form",
            "billing",
            json!({
                "message": "Which account should this month's invoices go to?",
                "schema": {
                    "type": "object",
                    "properties": {
                        "account": { "type": "string", "title": "Account", "minLength": 3,
                                     "description": "The billing account's name" },
                        "currency": { "type": "string", "title": "Currency", "enum": ["USD", "EUR", "KES"],
                                      "enumNames": ["US dollar", "Euro", "Kenyan shilling"] },
                        "seats": { "type": "integer", "title": "Seats", "minimum": 1, "maximum": 50 },
                        "notify": { "type": "boolean", "title": "Email the finance team", "default": true }
                    },
                    "required": ["account", "currency"]
                }
            }),
            vec![ApprovalAction::Edit, ApprovalAction::Reject],
        ))
        .await?;
    approvals
        .submit(ask(
            "apr-elicit-url",
            "payments",
            json!({ "message": "Connect your payments account", "url": "https://xn--pple-43d.com/connect?client=adk" }),
            vec![ApprovalAction::Approve, ApprovalAction::Reject],
        ))
        .await?;
    Ok(())
}

/// adk-ui's MCP Apps server in process, as the agent's toolset and the apps host's servers.
async fn apps_server() -> Result<
    (
        Arc<dyn adk_core::Toolset>,
        Arc<dyn adk_surface::genui::AppServers>,
    ),
    Box<dyn std::error::Error>,
> {
    use adk_tool::mcp::rmcp::ServiceExt;
    use adk_tool::mcp::{AdkClientHandler, AutoDeclineElicitationHandler, McpToolset};

    let (server_io, client_io) = tokio::io::duplex(1 << 20);
    tokio::spawn(async move {
        if let Ok(running) = adk_ui::apps::UiApps::default().serve(server_io).await {
            let _ = running.waiting().await;
        }
    });
    let client = AdkClientHandler::new(Arc::new(AutoDeclineElicitationHandler))
        .with_apps()
        .serve(client_io)
        .await?;
    let toolset = Arc::new(McpToolset::new(client).with_name("adk-ui apps"));
    Ok((toolset.clone(), toolset))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|argument| argument == "--wire") {
        return wire();
    }
    let port: u16 = option("--port").map_or(Ok(PORT), |port| port.parse())?;
    let from: usize = option("--from").map_or(Ok(1), |from| from.parse())?;
    let workspace = std::env::temp_dir().join("adk-ui-tour");
    std::fs::create_dir_all(&workspace)?;
    let apps_port = port + 1;
    let apps_origin = format!("http://localhost:{apps_port}");

    let runs: Arc<dyn DurableRunStore> = Arc::new(InMemoryRunStore::new());
    let approvals: Arc<dyn ApprovalService> = Arc::new(
        StoreApprovalService::new(Arc::new(InMemoryApprovalStore::new()))
            .with_run_store(Arc::clone(&runs))
            .with_router(
                ApprovalPolicy::new([ApproverSelector::group("owner")]).allow_self_approval(),
            ),
    );
    let seals = ApprovalSeals::new();
    let (apps_toolset, app_servers) = apps_server().await?;
    let agent = Arc::new(
        HarnessAgent::builder("operations")
            .model(Arc::new(ScriptedLlm::new("scripted", script(from))))
            .workspace(adk_devtools::Workspace::new(&workspace))
            .project_instructions(InstructionsConfig {
                user_file: None,
                require_trust: false,
                ..InstructionsConfig::default()
            })
            .without_compaction()
            .without_default_members()
            .without_shell()
            .mode(HarnessMode::Ask)
            .approval_service(Arc::clone(&approvals))
            .requested_call_seal(Arc::new(seals.clone()))
            .tool(Arc::new(Restart(workspace.join("calls.txt"))))
            .toolset(Arc::new(adk_ui::toolset_for(adk_ui::catalogs::spatial())))
            .toolset(apps_toolset)
            .trust_mcp_read_only_hints(true)
            .build()?,
    );
    let sessions: Arc<dyn SessionService> = Arc::new(InMemorySessionService::new());
    let runner = agent
        .configure_runner(
            Runner::builder()
                .app_name(APP)
                .agent(agent.clone())
                .session_service(sessions.clone()),
        )
        .build()?;
    let harness =
        Arc::new(Harness::with_runner(agent.clone(), runner)?.with_run_store(Arc::clone(&runs)));
    let kernel = harness
        .kernel()
        .config(
            KernelConfig::default()
                .with_app_name(APP)
                .with_worker(
                    DurableWorkerConfig::default()
                        .with_worker_id("tour")
                        .with_poll_interval(Duration::from_millis(50)),
                )
                .with_control(
                    ControlPlaneConfig::default()
                        .without_socket()
                        .with_start_mode("default", PartitionMode::Ask),
                ),
        )
        .process_store(Arc::new(InMemoryProcessStore::new()) as Arc<dyn ProcessStore>)
        .build()?;
    kernel.start().await?;
    // The elicitations' own thread, so a decision's page opens into a conversation.
    sessions
        .create(adk_session::CreateRequest {
            app_name: APP.into(),
            user_id: OWNER.into(),
            session_id: Some("tour".into()),
            state: Default::default(),
        })
        .await?;
    let mut note = adk_core::Event::new("tour-elicitations");
    note.author = "operations".into();
    note.llm_response.content = Some(adk_core::Content::new("model").with_text(
        "While I worked, two MCP servers asked you something: billing wants a form filled in, \
         and payments wants you to open an address. Their cards are in Decisions; the home draws \
         both in its own controls.",
    ));
    sessions.append_event("tour", note).await?;
    elicitations(&approvals).await?;

    let source = LocalSource::builder(APP)
        .sessions(sessions.clone())
        .approvals(approvals)
        .harness(harness)
        .kernel(kernel.clone(), APP)
        .build()?;
    let trail = AuditTrail::new(Arc::new(InMemoryAuditRecorder::new()));
    let engine = adk_ui::catalogs::register(SurfaceEngine::builder(Arc::new(source)))
        .audit(trail.clone())
        .approval_seals(seals)
        .places([Place::web(), Place::phone()])
        .agent_name(APP, "Operations")
        .grant(GrantedUse::new("genui", words::SHOW_FORMS_AND_TABLES))
        .build();

    let host = adk_surface::genui::McpAppsHost::builder()
        .servers(app_servers)
        .apps_origin(&apps_origin)
        .home_origin(format!("http://127.0.0.1:{port}"))
        .build();
    let frames = host.sandbox_router();
    for address in ["127.0.0.1", "::1"] {
        if let Ok(listener) = tokio::net::TcpListener::bind((address, apps_port)).await {
            let frames = frames.clone();
            tokio::spawn(async move {
                let _ = axum::serve(listener, frames).await;
            });
        }
    }

    let config = ServerConfig::new(Arc::new(SingleAgentLoader::new(agent)), sessions)
        .with_request_context(Arc::new(Owner));
    let app = ServerBuilder::new(config)
        .with_ag_ui_extension(Arc::new(adk_ui::catalogs::ag_ui()))
        .add_api_routes(adk_kernel::control::router(kernel))
        .add_api_routes(adk_surface::web::router(engine.clone()))
        .add_api_routes(adk_surface::web::genui::router_wired(
            engine.clone(),
            GenuiWiring {
                ag_ui: true,
                mcp_apps: true,
            },
        ))
        .add_api_routes(adk_surface::web::genui::apps_router(engine, host))
        .add_root_routes(adk_surface::web::home_with_apps_origin(&apps_origin))
        .build();
    let app = adk_surface::web::genui::refuse_apps_origin_recorded(app, &apps_origin, trail);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    println!(
        "adk-ui capabilities tour on http://127.0.0.1:{port}/ui/ (from stop {from}) — send any \
         message to start"
    );
    axum::serve(listener, app).await?;
    Ok(())
}

/// What the tour's drawings are on the wire.
fn wire() -> Result<(), Box<dyn std::error::Error>> {
    use adk_surface::genui::{
        A2uiMessage, A2uiVersion, DataModel, DrawnActionView, DrawnEnvelope, DrawnKind,
        DrawnSurface, Validator, adaptive_card, capabilities_wired, drawn_tree,
        read_function_message, slack_blocks, to_wire,
    };

    let spatial = adk_ui::catalogs::spatial();
    let section = |title: &str| println!("\n━━ {title} ━━\n");
    let pretty = |value: &Value| serde_json::to_string_pretty(value).unwrap_or_default();

    // A drawing as the store keeps it, at a version.
    let envelope = |args: &Value, version: u64| -> Result<DrawnEnvelope, String> {
        let draft = adk_ui::eval::draft_of(args, spatial)?;
        let surface = draft.surface_id.clone();
        Ok(DrawnEnvelope {
            surface_id: surface.clone(),
            version,
            kind: DrawnKind::Screen,
            catalog_id: spatial.id().to_string(),
            title: draft.title.clone(),
            messages: vec![
                A2uiMessage::create_surface(surface.clone()),
                A2uiMessage::update_components(surface.clone(), draft.components.clone()),
                A2uiMessage::update_data_model(surface, None, Some(draft.data_model.clone())),
            ],
            actions: draft.actions.clone(),
            deleted: false,
        })
    };

    section(
        "1. The dashboard in A2UI 1.0: one createSurface, @path bindings, the display-only claim",
    );
    let first = envelope(&dashboard(), 1)?;
    let wire = to_wire(&first.messages, spatial, A2uiVersion::V1_0);
    let create = &wire[0]["createSurface"];
    println!("metadata: {}", pretty(&create["metadata"]));
    println!("a Metric: {}", create["components"][2]);
    println!("a VegaLite: {}", create["components"][4]);

    section("2. The live update as a delta: only what changed between version 1 and 2");
    let mut patched = adk_ui::eval::draft_of(&dashboard(), spatial)?;
    let mut model = DataModel::new(patched.data_model.clone());
    for change in live_update()["data"].as_array().into_iter().flatten() {
        model.apply(change["path"].as_str(), Some(change["value"].clone()))?;
    }
    patched.data_model = model.value().clone();
    let mut second = envelope(&dashboard(), 2)?;
    second.messages[2] = A2uiMessage::update_data_model("latency", None, Some(patched.data_model));
    let delta = second.delta_from(&first).ok_or("no delta")?;
    for message in to_wire(&delta, spatial, A2uiVersion::V1_0) {
        println!("{message}");
    }

    section("3. A2UI 1.0 function calls: an agent asking the home to run a function is refused");
    let call = json!({ "version": "1.0", "callRendererFunction": {
        "functionCallId": "c1", "callFunction": { "@call": "formatString", "catalogId": spatial.id() } } });
    if let Some(adk_surface::genui::FunctionMessage::Call(call)) = read_function_message(&call)? {
        println!("{}", pretty(&call.refusal(spatial).to_wire()));
    }

    section("4. The restart form as a Slack message and a Teams card");
    let form = adk_ui::eval::draft_of(&restart_form(), spatial)?;
    let validated = Validator::new(spatial)
        .validate(&form)
        .map_err(|r| r.technical())?;
    let mut surface: DrawnSurface = serde_json::from_value(json!({
        "surfaceId": "restart", "version": 1, "kind": "screen", "catalog": spatial.label(),
        "agent": { "name": "operations", "displayName": "Operations", "avatar": { "monogram": "O", "color": "#2563eb" } },
        "drawnAt": "2026-11-19T10:02:00Z", "atText": "10:02", "drawnText": "Drawn by Operations · 10:02",
        "changedText": "", "earlierText": "", "title": "Restart a service"
    }))?;
    surface.root = Some(drawn_tree(spatial, &validated));
    surface.actions = vec![DrawnActionView {
        id: "restart".into(),
        label: "Restart".into(),
        tool: "restart_service".into(),
        note: "Sending runs restart_service and asks you first".into(),
        args: Vec::new(),
    }];
    let slack = slack_blocks(&surface);
    println!(
        "Slack blocks: {}",
        slack["blocks"].as_array().map_or(0, Vec::len)
    );
    for block in slack["blocks"].as_array().into_iter().flatten() {
        println!("  {}", block);
    }
    let teams = adaptive_card(&surface);
    println!("\nTeams Adaptive Card body:");
    for item in teams["body"].as_array().into_iter().flatten() {
        println!("  {item}");
    }
    println!("  actions: {}", teams["actions"]);

    section("5. The home's claims (Developer panel → Protocols)");
    let claims = capabilities_wired(GenuiWiring {
        ag_ui: true,
        mcp_apps: true,
    });
    for claim in &claims.protocols[0].claims {
        println!(
            "  A2UI · {} — {:?}{}",
            claim.label,
            claim.status,
            claim
                .note
                .as_deref()
                .map(|note| format!(" ({note})"))
                .unwrap_or_default()
        );
    }

    section("6. The tour's drawings, scored the way genui will judge them");
    let mut report = adk_ui::eval::Report::default();
    for (name, args, expected) in [
        ("dashboard", dashboard(), vec!["VegaLite", "Metric"]),
        ("restart form", restart_form(), vec![]),
        ("upgrade (pushy)", pushy(), vec![]),
        ("fake decision", fake_decision(), vec![]),
    ] {
        report.add(name, adk_ui::eval::score(spatial, &args, &expected));
    }
    println!("{report}");
    Ok(())
}
