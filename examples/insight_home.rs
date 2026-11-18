//! The ADK-Rust web home over a scripted agent that draws in adk-ui's catalogs.
//!
//! A kernel hosts the harness agent Operations in "Ask first", with `genui`'s render tools
//! drawing in adk-ui's spatial catalog (`adk_ui::toolset_for`), which holds studio and insight,
//! and three tools a drawing may ask for: `start_rollback`, `open_ticket` and `reserve_dock`,
//! each appending its calls to `calls.txt` in the workspace. Its model is `ScriptedLlm`: the
//! first turn draws the eleven sample drawings of `fixtures/drawings/` and their patches. The
//! home registers adk-ui's catalogs (`adk_ui::catalogs::register`), so it shows charts and 3D
//! scenes natively and the rest as its own components; pressing **Start rollback**, or
//! **Reserve this dock** after picking a dock in the yard, asks you first, on the home's card.
//!
//! ```sh
//! cargo run --example insight_home -- 8806 /tmp/insight-home
//! # then open http://127.0.0.1:8806/ui/ and send any message
//! ```

use std::io::Write as _;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use adk_core::{
    ApprovalPolicy, ApprovalService, ApproverSelector, DurableRunStore, InMemoryApprovalStore,
    InMemoryRunStore, RequestContext, SingleAgentLoader, StoreApprovalService, Tool, ToolContext,
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
use adk_surface::{GrantedUse, LocalSource, Place, SurfaceEngine, words};
use async_trait::async_trait;
use serde_json::{Value, json};

const APP: &str = "home";
const OWNER: &str = "ana";

/// The sample drawings, as `render_screen`'s arguments and `patch_surface`'s changes.
const FIXTURES: &[&str] = &[
    include_str!("../fixtures/drawings/incident-command.json"),
    include_str!("../fixtures/drawings/release-room.json"),
    include_str!("../fixtures/drawings/brand-commerce.json"),
    include_str!("../fixtures/drawings/support-workflow.json"),
    include_str!("../fixtures/drawings/spatial-logistics.json"),
    include_str!("../fixtures/drawings/streaming-operations.json"),
    include_str!("../fixtures/drawings/spatial-revenue-bars.json"),
    include_str!("../fixtures/drawings/spatial-fleet-globe.json"),
    include_str!("../fixtures/drawings/spatial-service-graph.json"),
    include_str!("../fixtures/drawings/spatial-warehouse-pick.json"),
    include_str!("../fixtures/drawings/spatial-terrain.json"),
];

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

/// A tool a drawing may ask for: it appends each call it runs to `calls.txt`.
struct Logged {
    name: &'static str,
    description: &'static str,
    log: PathBuf,
}

#[async_trait]
impl Tool for Logged {
    fn name(&self) -> &str {
        self.name
    }

    fn description(&self) -> &str {
        self.description
    }

    fn effect(&self) -> ToolEffect {
        ToolEffect::NonIdempotent
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        let mut log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log)
            .map_err(|error| adk_core::AdkError::tool(error.to_string()))?;
        writeln!(log, "{} {args}", self.name)
            .map_err(|error| adk_core::AdkError::tool(error.to_string()))?;
        Ok(json!({ "done": true }))
    }
}

/// The scripted turn: every sample drawing, the throughput patches, then a reply.
fn script() -> Vec<ScriptedResponse> {
    let mut script = Vec::new();
    for fixture in FIXTURES {
        let fixture: Value = serde_json::from_str(fixture).expect("a sample drawing is JSON");
        script.push(ScriptedResponse::function_call(
            "render_screen",
            fixture["screen"].clone(),
        ));
        for patch in fixture["patches"].as_array().into_iter().flatten() {
            script.push(ScriptedResponse::function_call(
                "patch_surface",
                patch.clone(),
            ));
        }
    }
    script.push(ScriptedResponse::text(
        "Here is where things stand. Checkout latency doubled after the last deploy; press \
         Start rollback if you want me to roll it back.",
    ));
    script.push(ScriptedResponse::text(
        "Rolling back checkout-api to 2026.11.16-1.",
    ));
    script
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let port: u16 = args.next().map_or(Ok(8806), |port| port.parse())?;
    let workspace = args.next().map_or_else(std::env::temp_dir, PathBuf::from);
    std::fs::create_dir_all(&workspace)?;
    let log = workspace.join("calls.txt");

    let runs: Arc<dyn DurableRunStore> = Arc::new(InMemoryRunStore::new());
    let approvals: Arc<dyn ApprovalService> = Arc::new(
        StoreApprovalService::new(Arc::new(InMemoryApprovalStore::new()))
            .with_run_store(Arc::clone(&runs))
            .with_router(
                ApprovalPolicy::new([ApproverSelector::group("owner")]).allow_self_approval(),
            ),
    );
    let tool = |name: &'static str, description: &'static str| -> Arc<dyn Tool> {
        Arc::new(Logged {
            name,
            description,
            log: log.clone(),
        })
    };
    let agent = Arc::new(
        HarnessAgent::builder("operations")
            .model(Arc::new(ScriptedLlm::new("scripted", script())))
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
            .tool(tool(
                "start_rollback",
                "Rolls a service back to an earlier version.",
            ))
            .tool(tool("open_ticket", "Opens a support ticket."))
            .tool(tool("reserve_dock", "Reserves a loading dock."))
            .toolset(Arc::new(adk_ui::toolset_for(adk_ui::catalogs::spatial())))
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
                        .with_worker_id("insight")
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

    let source = LocalSource::builder(APP)
        .sessions(sessions.clone())
        .approvals(approvals)
        .harness(harness)
        .kernel(kernel.clone(), APP)
        .build()?;
    let engine = adk_ui::catalogs::register(SurfaceEngine::builder(Arc::new(source)))
        .places([Place::web(), Place::phone()])
        .agent_name(APP, "Operations")
        .grant(GrantedUse::new("genui", words::SHOW_FORMS_AND_TABLES))
        .build();

    let config = ServerConfig::new(Arc::new(SingleAgentLoader::new(agent)), sessions)
        .with_request_context(Arc::new(Owner));
    let app = ServerBuilder::new(config)
        .with_ag_ui_extension(Arc::new(adk_ui::catalogs::ag_ui()))
        .add_api_routes(adk_kernel::control::router(kernel))
        .add_api_routes(adk_surface::web::router(engine.clone()))
        .add_api_routes(adk_surface::web::genui::router(engine))
        .add_root_routes(adk_surface::web::home())
        .build();
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    println!("adk-ui insight home on http://127.0.0.1:{port}/ui/");
    axum::serve(listener, app).await?;
    Ok(())
}
