//! `adk-ui-apps`: adk-ui's MCP Apps server over stdio, with `show_3d_app` (model-written
//! three.js, sandboxed) and `show_drawing` (a checked drawing of adk-ui's catalogs, read-only).
//! Give it to an agent whose home runs genui-apps, or to any client that frames MCP Apps.
//!
//! ```sh
//! cargo install adk-ui --features apps --bin adk-ui-apps
//! ```

use rmcp::ServiceExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let running = adk_ui::apps::UiApps::default()
        .serve(rmcp::transport::stdio())
        .await?;
    running.waiting().await?;
    Ok(())
}
