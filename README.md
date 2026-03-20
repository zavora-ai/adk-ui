# adk-ui

Build agent-driven interfaces that feel like real products, not debug payloads.

`adk-ui` gives Rust agents a practical UI vocabulary: forms, dashboards, confirmations, alerts, tables, charts, progress states, modals, toasts, and protocol-aware surfaces that can be rendered by real clients. If your agent can reason about a workflow, `adk-ui` helps it express that workflow as an interface people can use.

[![Crates.io](https://img.shields.io/crates/v/adk-ui.svg)](https://crates.io/crates/adk-ui)
[![Documentation](https://docs.rs/adk-ui/badge.svg)](https://docs.rs/adk-ui)
[![License](https://img.shields.io/crates/l/adk-ui.svg)](LICENSE)

## Why People Pick adk-ui

- Start simple: give an agent high-level render tools instead of hand-authoring a UI framework from scratch.
- Stay honest at protocol boundaries: A2UI, AG-UI, and MCP Apps are supported with explicit capability signaling rather than vague compatibility claims.
- Ship faster: this repo includes a Rust demo server, a React reference client, protocol adapters, examples, tests, and migration docs.
- Keep the loop agentic: the same system can ask for input, render a result, request confirmation, and react to follow-up actions.
- Grow without repainting everything: use high-level tools first, then choose the protocol surface that matches your host or client architecture.

## What It Looks Like

### A2UI dashboard

![A2UI dashboard](docs/screenshots/a2ui-dashboard.png)

### AG-UI operations flow

![AG-UI operations flow](docs/screenshots/ag-ui-command-center.png)

### MCP Apps bridge-driven confirm flow

![MCP Apps confirm flow](docs/screenshots/mcp-apps-confirm.png)

## What You Can Build

- A support intake assistant that turns open-ended requests into structured forms, triage queues, and escalation confirms.
- An operations agent that renders dashboards, alerts, and approval prompts instead of dumping raw JSON into chat.
- A scheduling assistant that shows availability, collects preferences, and confirms bookings.
- An inventory or facilities workflow that moves from dashboard to form to approval to toast in one agent session.

## 5-Minute Quick Start

### 1. Add the crate

```toml
[dependencies]
adk-ui = "0.4.0"
```

### 2. Add the UI toolset to your agent

```rust
use adk_agent::LlmAgentBuilder;
use adk_ui::{UiToolset, UI_AGENT_PROMPT};

let tools = UiToolset::all_tools();

let mut builder = LlmAgentBuilder::new("assistant")
    .model(model)
    .instruction(UI_AGENT_PROMPT);

for tool in tools {
    builder = builder.tool(tool);
}

let agent = builder.build()?;
```

### 3. Run the example stack in this repo

The bundled demo uses the Rust example server plus the React client.

```bash
# In the repo root
export GOOGLE_API_KEY=...
cargo run --example ui_server --features adk-core
```

```bash
# In another terminal
cd examples/ui_react_client
npm install
npm run dev -- --host 127.0.0.1
```

Then open [http://127.0.0.1:5173/](http://127.0.0.1:5173/), choose a protocol profile, and run one of the built-in prompts.

## Beginner-Friendly Mental Model

Think of `adk-ui` as a UI layer for agents, not as a replacement web framework.

Your agent decides:

- what the user needs next
- which UI pattern fits that moment
- what data should be shown or collected
- what action should happen after the user responds

`adk-ui` gives the agent structured tools to express those decisions safely.

That means the same conversation can naturally move through:

1. a prompt from the user
2. a rendered form or dashboard
3. a follow-up action such as confirm, submit, or retry
4. a new surface, update, or toast

## Agentic UI Examples

### Example 1: Support intake

User says:

```text
My payroll export failed and finance needs it today.
```

Agent flow:

- render a support intake form with severity, environment, screenshots, and deadline
- summarize the issue back to the user
- ask for confirmation before escalating to the on-call queue

### Example 2: Operations command center

User says:

```text
Show me cluster health and let me approve a failover if needed.
```

Agent flow:

- render a dashboard with alerts, node tables, and traffic charts
- surface a confirmation card when a risky action is needed
- render a toast or follow-up status panel after approval

### Example 3: Scheduling assistant

User says:

```text
Book me the earliest available appointment next week.
```

Agent flow:

- show available time slots
- collect preferences or missing constraints
- confirm the final selection
- render a success state with the booked details

## Protocols, Explained Simply

You do not need to master every protocol on day one. Start with the shape that fits your client boundary.

### A2UI

Choose `a2ui` when you want a practical structured surface transport between your agent/server and a renderer. It is the cleanest starting point in this repo and the most direct way to render agent-produced surfaces in the React client.

### AG-UI

Choose `ag_ui` when your consumer cares about event streams, lifecycle updates, and stable message/tool semantics. In this repo, AG-UI support is intentionally explicit about its boundaries: native run-input ingestion is in place, while some runtime emission still passes through compatibility-oriented paths.

### MCP Apps

Choose `mcp_apps` when your integration boundary looks like a host/app bridge and you want `ui://` resources, structured content, and bridge-aware host metadata. In this repo, MCP Apps support is centered on the practical bridge flows we actually run today, not a claim of full embedded-host parity.

### Legacy `adk_ui`

The legacy internal profile is still available for backward compatibility during migration, but new integrations should prefer `a2ui`, `ag_ui`, or `mcp_apps`.

## Compliance Snapshot

This section is deliberately concrete. It is meant to help integrators understand what is implemented today.

### Implementation metrics

- `30` component types
- `13` high-level render tools
- `39 / 39` render tool x protocol combinations covered by Rust matrix tests
- `4 / 4` selectable runtime profiles smoke-tested in the live example client on `2026-03-20`
- capability metadata is exposed at runtime through `/api/ui/capabilities`

### Protocol support snapshot

| Protocol | Upstream Target | Implementation Tier | What Works Well Today | Live Validation |
|----------|------------------|---------------------|-----------------------|-----------------|
| `a2ui` | `v0.9` draft-aligned | Hybrid subset | `jsonl`, flat components, `createSurface`, `updateComponents`, `updateDataModel`, client metadata, validation feedback, local actions | Dashboard render validated in browser |
| `ag_ui` | Stable `0.1` subset | Compatibility subset | native run-input ingestion, run lifecycle, stable text/tool event ingestion, message snapshot ingestion, action loop support in the React client | Render + confirm action validated in browser |
| `mcp_apps` | `SEP-1865` subset | Compatibility subset | `ui://` resources, structured tool results, initialize/message/model-context bridge endpoints, host context, inline HTML fallback | Initialize + render + confirm action validated in browser |
| `adk_ui` | Internal legacy profile | Legacy | backward-compatible runtime behavior during migration | Smoke-tested in browser |

### Important honesty note

`adk-ui` does not present AG-UI or MCP Apps as fully native, fully complete implementations. The runtime capability signal and docs are intentionally explicit about hybrid and compatibility subsets so downstream clients can make correct decisions.

## How The Pieces Fit Together

At a high level:

```text
User prompt
  -> agent decides what UI to render
  -> adk-ui tool emits a surface or protocol-aware payload
  -> client renders the surface
  -> user acts on the interface
  -> action goes back to the agent
  -> agent updates, confirms, or completes the workflow
```

This repo includes:

- Rust-side UI models, validation, prompts, templates, and protocol adapters
- a React reference client that can render and act on those surfaces
- protocol boundary code for A2UI, AG-UI, and MCP Apps
- tests and examples for real integration paths

## What Is In The Box

### Render tools

| Tool | Purpose |
|------|---------|
| `render_screen` | Emit protocol-aware screen surfaces from component definitions |
| `render_page` | Build multi-section pages and emit protocol-aware payloads |
| `render_kit` | Generate A2UI kit/catalog artifacts |
| `render_form` | Collect structured user input |
| `render_card` | Show information-rich cards with actions |
| `render_alert` | Surface status and severity messages |
| `render_confirm` | Ask the user to approve a risky or important action |
| `render_table` | Display sortable tabular data |
| `render_chart` | Display line, bar, area, and pie charts |
| `render_layout` | Build dashboard-style layouts |
| `render_progress` | Show progress and step flows |
| `render_modal` | Display modal dialogs |
| `render_toast` | Show temporary notifications |

### Core strengths

- type-safe Rust schema plus TypeScript-friendly rendering surface
- server-side validation before bad UI reaches the browser
- streaming updates via `UiUpdate`
- tested system prompts for reliable tool use
- prebuilt templates for common business flows
- protocol adapters that reduce per-tool drift

## Examples In This Repo

| Example | Description | Command |
|---------|-------------|---------|
| `ui_server` | Rust server with SSE and protocol-aware UI tool output | `cargo run --example ui_server --features adk-core` |
| `ui_react_client` | React reference client with protocol profile selector | `cd examples/ui_react_client && npm run dev -- --host 127.0.0.1` |
| `ui_protocol_profiles` | Coverage demo for tool/protocol output combinations | `cargo run --example ui_protocol_profiles` |
| `streaming_demo` | Streaming progress and incremental updates | `cargo run --example streaming_demo` |

## Choosing The Right Protocol

If you want a practical default in this repo:

- choose `a2ui` for the most direct structured surface path
- choose `ag_ui` when the consumer wants event semantics
- choose `mcp_apps` when the host/app bridge model matters

If you are unsure, start with `a2ui`, validate the user journey, then introduce `ag_ui` or `mcp_apps` at the integration boundary that actually needs them.

## Migration And Deprecation

The legacy `adk_ui` runtime profile is on a planned migration path:

- announced: `2026-02-07`
- sunset target: `2026-12-31`
- preferred profiles: `a2ui`, `ag_ui`, `mcp_apps`

Detailed migration guidance lives in [docs/PROTOCOL_MIGRATION.md](docs/PROTOCOL_MIGRATION.md).

## Additional Docs

- [Protocol migration guide](docs/PROTOCOL_MIGRATION.md)
- [Protocol modernization workplan](docs/PROTOCOL_MODERNIZATION_WORKPLAN.md)
- [Framework continuity roadmap](docs/FRAMEWORK_CONTINUITY_ROADMAP.md)
- [React client notes](examples/ui_react_client/README.md)

## License

Apache-2.0

## Part Of ADK-Rust

`adk-ui` is part of the [ADK-Rust](https://adk-rust.com) ecosystem for building AI agents in Rust.
