# Moving to adk-ui 3

adk-ui 3 is an extension of ADK-Rust 3's agent-drawn UI, not a second render stack.
`adk-surface`'s `genui` feature draws, versions and proves every drawing, keeps approval
controls, secrets and links out of it (spec 3.3 §7.4 rule 5), and turns a form's **Send** into a
call the person decides. adk-ui 3 adds what `genui`'s `forms` catalog leaves out (charts,
metrics, pages, scenes) as **extension catalogs** that `genui` checks and draws.

adk-ui 2.2 stays on crates.io and npm for ADK-Rust 2, with security fixes only until the
`adk_ui` server profile's sunset on 2026-12-31.

## Rust

```toml
[dependencies]
adk-ui = "3"
adk-surface = { version = "3", features = ["web", "genui"] }
```

```rust
// The agent draws with genui's render tools, in an adk-ui catalog...
let agent = HarnessAgent::builder("operations")
    .toolset(Arc::new(adk_ui::toolset_for(adk_ui::catalogs::studio())))
    .instruction(adk_ui::prompts::instructions(adk_ui::catalogs::studio()))
    // ...
    .build()?;
// ...and the home shows what it draws.
let engine = adk_ui::catalogs::register(SurfaceEngine::builder(source)).build();
let app = ServerBuilder::new(config).with_ag_ui_extension(Arc::new(adk_ui::catalogs::ag_ui()));
```

| adk-ui 2 | adk-ui 3 |
|---|---|
| `UiToolset`, `UiToolset::all_tools()` | `adk_ui::toolset()` / `toolset_for(catalog)`: `genui`'s `DrawToolset` in an adk-ui catalog |
| `render_form`, `render_card`, `render_table` | `genui`'s tools of the same names |
| `render_chart` | `render_screen` with a `Chart` or `Sparkline` (insight catalog) |
| `render_layout` (KPIs, dashboards) | `render_screen` with `Metric`, `KeyValue`, `Chart`, `Badge` |
| `render_progress` | `Progress`, then `patch_surface` as the work moves |
| `render_page`, `render_app` | `render_screen` with `App`, `Page`, `Region` (studio catalog) |
| `render_screen` | `render_screen` in an adk-ui catalog |
| Scene3D components | `Scene3D` (studio): primitives and checked labels only; no models, textures or object actions. For 3D charts, globes, networks and scenes a person can pick from, the spatial catalog (`Bar3D`, `Globe`, `Graph3D`, `Scene`, …) |
| `render_confirm`, `render_modal` | None: a drawing never asks for a yes. A call that needs one is held by the agent's policy and decided on the home's card |
| `render_alert`, `render_toast` | The agent's reply, or a `Badge` or `Text` |
| `render_kit`, `kit_id` | None in a drawing: the host applies an approved kit (`kit::ApprovedKit`, React `KitProvider`); see [BRAND_KITS.md](BRAND_KITS.md) |
| `patch_surface` | `genui`'s `patch_surface` |
| `save_surface`, `load_surface`, `list_surfaces`, `SurfaceStore` | None needed: every drawing is an event of the session, versioned by its surface id |
| Data sources (`binding_path`, `refresh`) | Bind with `{"path": "/rows"}` in `dataModel`; push new data with `patch_surface`. Interval and TTL refresh are gone: a timer must not drive the agent. Live feeds a host owns render adk-ui-react's components directly |
| `interop::ag_ui`, `interop::mcp_apps`, `awp` feature | `genui`'s `GenuiAgUi` (`adk_ui::catalogs::ag_ui()`), `genui-apps`' MCP Apps host; AWP is gone |
| `html` (server-side HTML) | Gone |
| The `adk_ui` protocol profile | Gone from adk-ui; deprecated in `adk-server` until 2026-12-31 |
| Templates | Gone (some asked for passwords, which rule 5 refuses) |
| `prompts::UI_AGENT_PROMPT` | `prompts::instructions(catalog)` |
| Typed builders of `UiResponse` | `adk_ui::insight::{Chart, Metric, …}`, `adk_ui::studio::{App, Page, …}`, `adk_ui::draft::Screen` |

A drawing drawn with adk-ui 2 has no `genui` record, so an ADK-Rust 3 home shows it as an
ordinary tool result.

**What a drawing may contain changed.** Every string, in the components and in the data they
bind, is checked for a decision's words ("Allow once", "Approve", "Deny", "Done" …), links and
length; a component that is not a `forms` button never dispatches anything; there are no
images, password fields or modals. A drawing that breaks a rule is refused, and the agent reads
why.

## React

```bash
npm install @zavora-ai/adk-ui-react@3 @zavora-ai/adk-surface-react
```

```tsx
import { DrawnView } from "@zavora-ai/adk-surface-react";
import { adkUiRenderers } from "@zavora-ai/adk-ui-react";

<DrawnView surface={surface} onSend={send} renderers={adkUiRenderers} />;
```

| `@zavora-ai/adk-ui-react` 2 | 3 |
|---|---|
| `Renderer`, `StreamingRenderer`, `A2uiSurfaceRenderer` | `@zavora-ai/adk-surface-react`'s `DrawnView`, with `adkUiRenderers` |
| `createProtocolClient`, `protocols`, A2UI parser and store | `adk-surface`'s engine API and `DrawnView` |
| `uiEventToMessage` | None: a form's Send posts `submitDrawn`, and the person decides on the home's card |
| `ApplicationRenderer` | `AppShell` (in a drawing, the `App` renderer) |
| `AdkUiKitProvider`, `resolveApprovedUiKit`, slots | `KitProvider` (approved kits only), `kitStyleVariables` |
| `ReasoningPanel` | Gone |
| `react-markdown`, `lucide-react`, `clsx` | No runtime dependencies; `recharts` stays an optional peer. `DrawnView` draws 3D itself, and `three` is a peer only of the deprecated standalone `Scene3D` |

Charts, metrics and the rest are also exported as components (`Chart`, `Metric`,
`KeyValueList`, …) for data a host owns.

## Catalog ids

| Catalog | Id |
|---|---|
| `forms` 1.0 (`genui`) | `https://adk-rust.com/a2ui/catalogs/forms/1.0/catalog.json` |
| adk-ui insight 1.0 | `https://zavora.ai/a2ui/catalogs/adk-ui-insight/1.0/catalog.json` |
| adk-ui studio 1.0 | `https://zavora.ai/a2ui/catalogs/adk-ui-studio/1.0/catalog.json` |

They replace `zavora.ai:adk-ui/extended@0.2.0` and the per-kit catalog ids. The documents are in
`catalogs/`.
