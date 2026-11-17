# adk-ui

Charts, metrics, pages and scenes for agent-drawn UI in [ADK-Rust](https://adk-rust.com) 3.

ADK-Rust draws agent UI with `adk-surface`'s **genui** feature: the render tools, A2UI v0.9.1, a
version and digest record for every drawing, and **rule 5**: a drawing never contains approval
controls, secrets or links, and a form's **Send** asks the person before any tool runs. genui's
own `forms` catalog draws forms, cards and tables. adk-ui 3 adds the rest as **extension
catalogs** that genui checks and draws:

| Catalog | Adds to `forms` |
|---|---|
| **insight** | `Chart` (bar, line, area, pie, scatter over category, number or time axes), `Sparkline`, `Metric`, `KeyValue`, `Progress`, `Badge` |
| **studio** | insight, plus `App` (pages the host navigates), `Page`, `Region` and `Scene3D` (primitive 3D shapes with labels) |
| **spatial** | studio, plus 3D charts (`Bar3D`, `Scatter3D`, `SurfacePlot`, `Graph3D`, `Globe`), `Scene` (groups, lines, lights, motion, panels beside objects; picking an object fills a choice field the drawing shows), `DepthCard` (tilt, flip, layers) and `Stage` (stage, wall, carousel) |

adk-ui adds no render tool name, protocol, provenance record or approval path. Every component
only shows: its strings, and every string of the data it binds, are checked before anything is
drawn. The web home draws charts and 3D scenes natively, in its own palette (scenes with WebGL,
loaded only for a drawing that has one), and the rest as its own components; a host of your own
draws them richly with `@zavora-ai/adk-ui-react`.

> adk-ui 2.x, with its own render tools and protocol adapters, stays on crates.io and npm for
> ADK-Rust 2. Moving over: [docs/MIGRATING_TO_3.md](docs/MIGRATING_TO_3.md).

## Use it

```toml
[dependencies]
adk-ui = "3"
adk-surface = { version = "3", features = ["web", "genui"] }
```

```rust
use std::sync::Arc;
use adk_surface::SurfaceEngine;

// The agent draws with genui's render tools, in adk-ui's studio catalog.
let agent = HarnessAgent::builder("operations")
    .model(model)
    .toolset(Arc::new(adk_ui::toolset_for(adk_ui::catalogs::studio())))
    .instruction(adk_ui::prompts::instructions(adk_ui::catalogs::studio()))
    .tool(Arc::new(start_rollback))
    .build()?;

// The home shows what it draws; AG-UI clients get it too.
let engine = adk_ui::catalogs::register(SurfaceEngine::builder(Arc::new(source))).build();
let app = ServerBuilder::new(config)
    .with_ag_ui_extension(Arc::new(adk_ui::catalogs::ag_ui()))
    .add_api_routes(adk_surface::web::router(engine.clone()))
    .add_api_routes(adk_surface::web::genui::router(engine))
    .add_root_routes(adk_surface::web::home())
    .build();
```

The model draws a screen with `render_screen`:

```json
{
  "surfaceId": "incident-2041",
  "title": "Incident INC-2041",
  "components": [
    { "id": "root", "component": "Column", "children": ["p95", "latency", "start"] },
    { "id": "p95", "component": "Metric", "label": "p95 latency", "value": { "path": "/p95" }, "unit": "ms" },
    { "id": "latency", "component": "Chart", "kind": "line", "data": { "path": "/latency" },
      "x": { "key": "at", "type": "time" }, "series": [{ "key": "checkout", "label": "Checkout" }] },
    { "id": "start", "component": "Button", "child": "start-label", "action": { "event": { "name": "rollback" } } },
    { "id": "start-label", "component": "Text", "text": "Start rollback" }
  ],
  "dataModel": { "p95": 840, "latency": [{ "at": "2026-11-16T09:00:00Z", "checkout": 410 }] },
  "actions": [{ "id": "rollback", "label": "Start rollback", "tool": "start_rollback", "args": { "service": "checkout-api" } }]
}
```

Pressing **Start rollback** never runs `start_rollback`: the agent holds the call, and the person
decides on the home's card (Allow once, Edit or Deny).

Builders write the same JSON in Rust (`adk_ui::insight::{Chart, Metric, …}`,
`adk_ui::studio::{App, Page, …}`, `adk_ui::spatial::{Bar3D, Scene, …}`, `adk_ui::draft::Screen`),
and `Screen::validate` checks a drawing exactly as genui will.

## In a host of your own

```tsx
import { DrawnView } from "@zavora-ai/adk-surface-react";
import { adkUiRenderers, KitProvider } from "@zavora-ai/adk-ui-react";

<KitProvider kit={approvedKit}>
  <DrawnView surface={surface} onSend={send} renderers={adkUiRenderers} />
</KitProvider>;
```

`recharts` is an optional peer: without it a chart is drawn as the home draws it. 3D scenes,
cards with depth and 3D layouts have no renderer here: `DrawnView` draws them itself, natively,
with picking into the drawing's form (`three` stays an optional peer only for the deprecated
standalone `Scene3D`). See [packages/adk-ui-react](packages/adk-ui-react/README.md) and
[docs/BRAND_KITS.md](docs/BRAND_KITS.md).

## Examples

| | |
|---|---|
| [`examples/insight_agent.rs`](examples/insight_agent.rs) | A real agent on Claude that studies a directory with its own tools and answers with adk-ui's charts, metrics and pages; configured from the environment (`ANTHROPIC_MODEL`, `ANTHROPIC_EFFORT`, `AGENT_WORKSPACE`, `AGENT_MODE`, `PORT`): `ANTHROPIC_MODEL=claude-opus-5-5 AGENT_WORKSPACE=$PWD cargo run --example insight_agent` |
| [`examples/insight_home.rs`](examples/insight_home.rs) | The ADK-Rust web home over a scripted agent that draws the eleven sample drawings: `cargo run --example insight_home -- 8806 /tmp/insight-home`, open `http://127.0.0.1:8806/ui/` and send a message |
| [`examples/custom_host`](examples/custom_host) | A Vite app drawing the same drawings with `DrawnView`: the home's view, adk-ui's renderers, and two approved brand kits |
| [`fixtures/drawings`](fixtures/drawings) | The eleven sample drawings: incident command, support request, release room, store week, warehouse bays and live throughput in studio; revenue in 3D bars, a fleet on a globe, a service network, a yard to pick a dock in, and terrain in spatial |

## Building before ADK-Rust 3.0 is on crates.io

```bash
cp .cargo/config.toml.example .cargo/config.toml   # untracked; points adk-* at ../adk-rust
cargo test
npm install && npm test                            # packages/adk-ui-react
```

`catalogs/` holds the catalog documents, written from the Rust catalogs and drift-checked by
`cargo test`; the React package's registry is checked against them.

## License

Apache-2.0
