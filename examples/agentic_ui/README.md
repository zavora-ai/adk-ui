# Agentic UI protocol demos

Three production-style agents, one protocol each:

| App name | Protocol | Journey |
|----------|----------|---------|
| `agentic_a2ui_support` | **A2UI** v0.9.1 | Form → confirm → toast |
| `agentic_ag_ui_ops` | **AG-UI** | Dashboard → confirm → progress/toast |
| `agentic_mcp_apps_confirm` | **MCP Apps** (SEP-1865 Final) | Confirm → form with `ui://` resources |

## Provider (OpenAI default)

```bash
# Default — OpenAI
export OPENAI_API_KEY=sk-...
# optional: export ADK_UI_MODEL=gpt-4o-mini

cargo run --example agentic_ui --features adk-core

# Gemini instead
export ADK_UI_PROVIDER=gemini
export GOOGLE_API_KEY=...
cargo run --example agentic_ui --features adk-core
```

Server listens on **http://localhost:8081** (override with `PORT`).

## React client

```bash
# terminal 1 — agentic_ui on 8081
cargo run --example agentic_ui --features adk-core

# terminal 2
cd examples/ui_react_client
npm run dev -- --host 127.0.0.1
```

Point the client at `http://127.0.0.1:8081`, pick protocol profile **a2ui** / **ag_ui** / **mcp_apps**, and select the matching agent app.

## Sample prompts

**A2UI support**

> My payroll export failed in production and finance needs it today. Open a high-severity ticket.

**AG-UI ops**

> Show cluster health and let me approve a failover if node-3 is degraded.

**MCP Apps**

> I need to revoke API keys for the staging project. Confirm first, then collect reason and ticket ID.

## Full multi-app server

For all demos plus legacy working apps (default port 8080):

```bash
cargo run --example ui_server --features adk-core
```
