//! Agentic UI demos for **A2UI**, **AG-UI**, and **MCP Apps**.
//!
//! Default LLM provider is **OpenAI**. Switch with:
//!
//! ```bash
//! export OPENAI_API_KEY=sk-...
//! cargo run --example agentic_ui --features adk-core
//!
//! # Gemini instead:
//! export ADK_UI_PROVIDER=gemini
//! export GOOGLE_API_KEY=...
//! cargo run --example agentic_ui --features adk-core
//! ```
//!
//! Apps (pick via appName / agent name):
//! - `agentic_a2ui_support` — multi-step support intake (A2UI)
//! - `agentic_ag_ui_ops` — ops dashboard + confirm (AG-UI)
//! - `agentic_mcp_apps_confirm` — MCP Apps ui:// confirm flow

#[path = "../common/provider.rs"]
mod provider;

use adk_agent::LlmAgentBuilder;
use adk_core::{Agent, Llm, MultiAgentLoader, Tool};
use adk_server::{ServerConfig, create_app, shutdown_signal};
use adk_session::{InMemorySessionService, SessionService};
use adk_ui::{UiToolset, a2ui::A2UI_AGENT_PROMPT};
use anyhow::Result;
use provider::{build_default_model, require_any_provider_env};
use std::sync::Arc;

const A2UI_INSTRUCTION: &str = r#"
You are an agentic A2UI support assistant.

Always use protocol=a2ui on tools. Drive a multi-turn agent loop with UI:
1) render_form — title, description, severity (low/med/high), environment, deadline
2) render_card + render_confirm — summarize and ask before escalate
3) render_toast or render_alert — success/failure after user action

Use render_screen with root id "root", Column layout, and Button action.event.name.
Prefer A2UI surfaces over plain text.
"#;

const AG_UI_INSTRUCTION: &str = r#"
You are an agentic AG-UI operations assistant.

Always use protocol=ag_ui on tools. Typical turn:
1) render_layout / render_table / render_chart / render_alert — ops dashboard
2) render_confirm — risky failover / scale action
3) render_progress then render_toast — action progress and result

The event stream is the primary UX; keep chat text short.
"#;

const MCP_APPS_INSTRUCTION: &str = r#"
You are an agentic MCP Apps assistant for embedded hosts.

Always use protocol=mcp_apps so tool results include ui:// resources and structured content.
Flow:
1) render_confirm — approve access / destructive action
2) render_form — collect remaining fields
3) render_card — confirmation summary

Keep resource URIs under ui://adk-ui/... and prefer structured confirmations.
"#;

fn full_instruction(extra: &str) -> String {
    format!("{A2UI_AGENT_PROMPT}\n\n{extra}")
}

fn build_agent(
    name: &str,
    description: &str,
    instruction: &str,
    model: Arc<dyn Llm>,
    tools: &[Arc<dyn Tool>],
) -> Result<Arc<dyn Agent>> {
    let mut builder = LlmAgentBuilder::new(name)
        .description(description)
        .instruction(full_instruction(instruction))
        .model(model);
    for tool in tools.iter().cloned() {
        builder = builder.tool(tool);
    }
    Ok(Arc::new(builder.build()?))
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    require_any_provider_env()?;
    let (model, model_name, provider) = build_default_model()?;
    let tools = UiToolset::all_tools();

    let a2ui = build_agent(
        "agentic_a2ui_support",
        "Agentic A2UI multi-step support intake",
        A2UI_INSTRUCTION,
        model.clone(),
        &tools,
    )?;
    let ag_ui = build_agent(
        "agentic_ag_ui_ops",
        "Agentic AG-UI operations command center",
        AG_UI_INSTRUCTION,
        model.clone(),
        &tools,
    )?;
    let mcp_apps = build_agent(
        "agentic_mcp_apps_confirm",
        "Agentic MCP Apps confirm/form host demo",
        MCP_APPS_INSTRUCTION,
        model,
        &tools,
    )?;

    let loader = Arc::new(MultiAgentLoader::new(vec![a2ui, ag_ui, mcp_apps])?);
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8081);
    let sessions: Arc<dyn SessionService> = Arc::new(InMemorySessionService::new());
    let config = ServerConfig::new(loader, sessions);
    let app = create_app(config);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    println!("=== ADK-UI Agentic Protocol Demos ===");
    println!("http://localhost:{port}");
    println!("Provider: {} | Model: {}", provider.as_str(), model_name);
    println!();
    println!("Apps:");
    println!("  agentic_a2ui_support       — A2UI form → confirm → toast");
    println!("  agentic_ag_ui_ops          — AG-UI dashboard → confirm → progress");
    println!("  agentic_mcp_apps_confirm   — MCP Apps ui:// confirm → form");
    println!();
    println!("Client: open examples/ui_react_client and select the matching protocol profile.");
    println!(
        "Env: ADK_UI_PROVIDER=openai|gemini  ADK_UI_MODEL=...  OPENAI_API_KEY / GOOGLE_API_KEY"
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}
