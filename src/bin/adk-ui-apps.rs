//! `adk-ui-apps`: the `show_3d_app` MCP server over stdio. Give it to an agent whose home runs
//! genui-apps, and the model's three.js shows sandboxed in the person's thread.
//!
//! ```sh
//! cargo install adk-ui --features apps --bin adk-ui-apps
//! ```

use rmcp::ServiceExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let running = adk_ui::apps::ThreeApps::default()
        .serve(rmcp::transport::stdio())
        .await?;
    running.waiting().await?;
    Ok(())
}
