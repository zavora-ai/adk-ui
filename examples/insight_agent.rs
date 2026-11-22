//! A real agent, on Claude, that answers with adk-ui's charts, metrics and pages.
//!
//! Nothing here is scripted: the agent is a harness agent on an Anthropic model, its data is
//! whatever its workspace tools find in the directory you point it at (files, search, and a
//! sandboxed shell that asks you first), and what it draws is its own choice, made with
//! `genui`'s render tools in an adk-ui catalog. The ADK-Rust web home shows every drawing
//! (charts natively), and any form it draws asks you on the home's card before a tool runs.
//!
//! With `--features apps` and `ADK_UI_3D_APPS=1`, the agent can also write three.js with
//! `show_3d_app` (adk-ui's MCP Apps server, in process). The home frames each app sandboxed
//! on an origin of its own under `http://localhost:APPS_PORT` (a subdomain per MCP server, and
//! a different site from the home's `127.0.0.1`), under "App from 3D sandbox · it can't
//! approve anything".
//!
//! Configuration, all from the environment:
//!
//! | Variable | Meaning | Default |
//! |---|---|---|
//! | `ANTHROPIC_API_KEY` | The Anthropic key (the variable `ModelProvider::Anthropic` names) | required |
//! | `ANTHROPIC_MODEL` | The Claude model | the framework's default for Anthropic |
//! | `ANTHROPIC_EFFORT` | `low`, `medium`, `high`, `xhigh` or `max` | the model's default |
//! | `AGENT_WORKSPACE` | The directory the agent studies | the current directory |
//! | `AGENT_MODE` | `read-only`, `plan`, `ask` or `auto` | `ask` |
//! | `AGENT_INSTRUCTIONS` | Extra instructions appended to adk-ui's | none |
//! | `AGENT_CATALOG` | `insight`, `studio` or `spatial` | `spatial` |
//! | `ADK_UI_3D_APPS` | `1` to give the agent `show_3d_app` (needs `--features apps`) | off |
//! | `PORT` | Where the home listens on 127.0.0.1 | `8807` |
//! | `APPS_PORT` | Where the apps origin listens, with `ADK_UI_3D_APPS` | `PORT` + 1 |
//!
//! ```sh
//! ANTHROPIC_MODEL=claude-opus-5-5 AGENT_WORKSPACE=$PWD cargo run --example insight_agent
//! # open http://127.0.0.1:8807/ui/ and ask it something about the workspace
//!
//! ADK_UI_3D_APPS=1 cargo run --features apps --example insight_agent
//! ```

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use adk_core::audit_log::{AuditTrail, InMemoryAuditRecorder};
use adk_core::{
    ApprovalPolicy, ApprovalService, ApproverSelector, DurableRunStore, InMemoryApprovalStore,
    InMemoryRunStore, RequestContext, SingleAgentLoader, StoreApprovalService,
};
use adk_harness::{Harness, HarnessAgent, HarnessMode, InstructionsConfig};
use adk_kernel::control::ControlPlaneConfig;
use adk_kernel::{InMemoryProcessStore, KernelConfig, PartitionMode, ProcessStore};
use adk_model::ModelProvider;
use adk_model::anthropic::{AnthropicClient, AnthropicConfig, Effort, ThinkingMode};
use adk_runner::Runner;
use adk_server::auth_bridge::{RequestContextError, RequestContextExtractor};
use adk_server::background::DurableWorkerConfig;
use adk_server::{ServerBuilder, ServerConfig};
use adk_session::{InMemorySessionService, SessionService};
use adk_surface::genui::{Catalog, GenuiWiring};
use adk_surface::{GrantedUse, LocalSource, Place, SurfaceEngine, words};
use async_trait::async_trait;

const APP: &str = "home";

/// Admits every request as the person at this computer: the home listens on loopback only.
struct Owner;

#[async_trait]
impl RequestContextExtractor for Owner {
    async fn extract(
        &self,
        _parts: &axum::http::request::Parts,
    ) -> Result<RequestContext, RequestContextError> {
        Ok(RequestContext {
            user_id: std::env::var("USER").unwrap_or_else(|_| "owner".into()),
            scopes: vec!["owner".into(), "operator".into()],
            metadata: Default::default(),
        })
    }
}

fn env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn effort(value: &str) -> Result<Effort, String> {
    Ok(match value.to_ascii_lowercase().as_str() {
        "low" => Effort::Low,
        "medium" => Effort::Medium,
        "high" => Effort::High,
        "xhigh" => Effort::XHigh,
        "max" => Effort::Max,
        other => {
            return Err(format!(
                "ANTHROPIC_EFFORT '{other}' is not low, medium, high, xhigh or max"
            ));
        }
    })
}

fn catalog(name: &str) -> Result<&'static Catalog, String> {
    match name {
        "insight" => Ok(adk_ui::catalogs::insight()),
        "studio" => Ok(adk_ui::catalogs::studio()),
        "spatial" => Ok(adk_ui::catalogs::spatial()),
        other => Err(format!(
            "AGENT_CATALOG '{other}' is not insight, studio or spatial"
        )),
    }
}

/// The agent's `show_3d_app` toolset and the apps host's view of the same server.
#[cfg(feature = "apps")]
struct Sandbox {
    toolset: Arc<dyn adk_core::Toolset>,
    servers: Arc<dyn adk_surface::genui::AppServers>,
}

/// adk-ui's three.js server in process, as the agent's MCP toolset "3D-sandbox".
#[cfg(feature = "apps")]
async fn sandbox() -> Result<Sandbox, Box<dyn std::error::Error>> {
    use adk_tool::mcp::rmcp::ServiceExt;
    use adk_tool::mcp::{AdkClientHandler, AutoDeclineElicitationHandler, McpToolset};

    let (server_io, client_io) = tokio::io::duplex(1 << 20);
    tokio::spawn(async move {
        if let Ok(running) = adk_ui::apps::UiApps::three_d().serve(server_io).await {
            let _ = running.waiting().await;
        }
    });
    let client = AdkClientHandler::new(Arc::new(AutoDeclineElicitationHandler))
        .with_apps()
        .serve(client_io)
        .await?;
    let toolset = Arc::new(McpToolset::new(client).with_name("3D-sandbox"));
    Ok(Sandbox {
        toolset: toolset.clone(),
        servers: toolset,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = ModelProvider::Anthropic;
    let key = env(provider.env_var())
        .ok_or_else(|| format!("set {} to run the agent on Claude", provider.env_var()))?;
    let model_id = env("ANTHROPIC_MODEL")
        .or_else(|| provider.default_model().map(str::to_string))
        .ok_or("set ANTHROPIC_MODEL")?;
    let mut config = AnthropicConfig::new(key, &model_id)
        .with_thinking_mode(ThinkingMode::Adaptive)
        .with_prompt_caching(true);
    if let Some(level) = env("ANTHROPIC_EFFORT") {
        config = config.with_effort(effort(&level)?);
    }
    let model = Arc::new(AnthropicClient::new(config)?);

    let workspace = env("AGENT_WORKSPACE")
        .map_or_else(std::env::current_dir, |path| Ok(PathBuf::from(path)))?;
    let mode: HarnessMode = env("AGENT_MODE").as_deref().unwrap_or("ask").parse()?;
    let port: u16 = env("PORT").as_deref().unwrap_or("8807").parse()?;
    let catalog = catalog(env("AGENT_CATALOG").as_deref().unwrap_or("spatial"))?;
    let apps = env("ADK_UI_3D_APPS").is_some_and(|value| value == "1" || value == "true");
    #[cfg(not(feature = "apps"))]
    if apps {
        return Err("ADK_UI_3D_APPS needs show_3d_app: rebuild with --features apps".into());
    }
    #[cfg(feature = "apps")]
    let sandbox = if apps { Some(sandbox().await?) } else { None };
    let apps_port: u16 = match env("APPS_PORT") {
        Some(value) => value.parse()?,
        None => port.checked_add(1).ok_or("set APPS_PORT")?,
    };
    let mut instruction = adk_ui::prompts::instructions(catalog);
    if apps {
        instruction.push_str("\n\n");
        instruction.push_str(&adk_ui::prompts::sandboxed_3d_apps());
    }
    if let Some(extra) = env("AGENT_INSTRUCTIONS") {
        instruction.push_str("\n\n");
        instruction.push_str(&extra);
    }

    let runs: Arc<dyn DurableRunStore> = Arc::new(InMemoryRunStore::new());
    let approvals: Arc<dyn ApprovalService> = Arc::new(
        StoreApprovalService::new(Arc::new(InMemoryApprovalStore::new()))
            .with_run_store(Arc::clone(&runs))
            .with_router(
                ApprovalPolicy::new([ApproverSelector::group("owner")]).allow_self_approval(),
            ),
    );
    #[allow(unused_mut)]
    let mut builder = HarnessAgent::builder("analyst")
        .model(model)
        .workspace(adk_devtools::Workspace::new(&workspace))
        .project_instructions(InstructionsConfig {
            user_file: None,
            require_trust: false,
            ..InstructionsConfig::default()
        })
        .without_default_members()
        .mode(mode)
        .approval_service(Arc::clone(&approvals))
        .instruction(instruction)
        .toolset(Arc::new(adk_ui::toolset_for(catalog)));
    #[cfg(feature = "apps")]
    if let Some(sandbox) = &sandbox {
        // show_3d_app reads only, as its server says: in Ask it runs without asking, and
        // what it shows is framed in the sandbox, where it can't approve anything.
        builder = builder
            .toolset(Arc::clone(&sandbox.toolset))
            .trust_mcp_read_only_hints(true);
    }
    let agent = Arc::new(builder.build()?);
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
                        .with_worker_id("analyst")
                        .with_poll_interval(Duration::from_millis(100)),
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
    let trail = AuditTrail::new(Arc::new(InMemoryAuditRecorder::new()));
    let engine = adk_ui::catalogs::register(SurfaceEngine::builder(Arc::new(source)))
        .audit(trail.clone())
        .places([Place::web(), Place::phone()])
        .agent_name(APP, "Analyst")
        .grant(GrantedUse::new("genui", words::SHOW_FORMS_AND_TABLES))
        .build();

    let config = ServerConfig::new(Arc::new(SingleAgentLoader::new(agent)), sessions)
        .with_request_context(Arc::new(Owner));
    let wiring = GenuiWiring {
        ag_ui: true,
        mcp_apps: apps,
    };
    #[allow(unused_mut)]
    let mut server = ServerBuilder::new(config)
        .with_ag_ui_extension(Arc::new(adk_ui::catalogs::ag_ui()))
        .add_api_routes(adk_kernel::control::router(kernel))
        .add_api_routes(adk_surface::web::router(engine.clone()))
        .add_api_routes(adk_surface::web::genui::router_wired(
            engine.clone(),
            wiring,
        ));
    let apps_origin = format!("http://localhost:{apps_port}");
    #[cfg(feature = "apps")]
    if let Some(sandbox) = sandbox {
        let host = adk_surface::genui::McpAppsHost::builder()
            .servers(sandbox.servers)
            .apps_origin(&apps_origin)
            .home_origin(format!("http://127.0.0.1:{port}"))
            .build();
        // The apps origin's own listener, on both loopback addresses `localhost` names.
        let frames = host.sandbox_router();
        for address in ["127.0.0.1", "::1"] {
            if let Ok(listener) = tokio::net::TcpListener::bind((address, apps_port)).await {
                let frames = frames.clone();
                tokio::spawn(async move {
                    let _ = axum::serve(listener, frames).await;
                });
            }
        }
        server = server.add_api_routes(adk_surface::web::genui::apps_router(engine.clone(), host));
    }
    let home = if apps {
        adk_surface::web::home_with_apps_origin(&apps_origin)
    } else {
        adk_surface::web::home()
    };
    let mut app = server.add_root_routes(home).build();
    if apps {
        app = adk_surface::web::genui::refuse_apps_origin_recorded(app, &apps_origin, trail);
    }
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    println!(
        "adk-ui insight agent ({model_id}, {mode:?}, {}) studying {} on http://127.0.0.1:{port}/ui/{}",
        catalog.label(),
        workspace.display(),
        if apps {
            format!(" (3D apps on {apps_origin})")
        } else {
            String::new()
        }
    );
    axum::serve(listener, app).await?;
    Ok(())
}
