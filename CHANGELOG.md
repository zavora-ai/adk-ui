# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] — 3.0.0

adk-ui 3 is an extension of ADK-Rust 3's agent-drawn UI (`adk-surface`, feature `genui`), not
a second render stack. `genui` draws, versions and proves every drawing, keeps approval
controls, secrets and links out of it (spec 3.3 §7.4 rule 5), and turns a form's **Send** into
a call the person decides. adk-ui 3 adds extension catalogs `genui` checks and draws.

### Added

- **The insight catalog** (`catalogs::insight`, id
  `https://zavora.ai/a2ui/catalogs/adk-ui-insight/1.0/catalog.json`): `forms` 1.0 plus
  `Chart` (bar, line, area, pie, scatter over category, number or time axes), `VegaLite`,
  `Sparkline`, `Metric` (value, unit, currency or percent, change, status, trend),
  `KeyValue`, `Progress` and `Badge`. Every one only shows, binds its data with
  `{"@path": …}`, and says what the ADK-Rust web home and terminal show for it: charts as the
  home's native chart, the rest as `forms` components. `genui` checks every string in them and
  in the data they bind.
- `VegaLite`: a Vega-Lite subset drawn as the home's chart (one mark: bar, line, area, point
  or arc; encodings `x`, `y`, `color` as a series per value, and `theta`). Transforms, layers,
  facets, concatenation, params, selections, URL data, `aggregate`, `bin` and `timeUnit` are
  refused by name, and every string in the spec is checked as shown text.
- `toolset()` and `toolset_for()`: `genui`'s render tools drawing in an adk-ui catalog, under
  their usual names; `catalogs::register` and `catalogs::ag_ui` register the catalogs with a
  home and with AG-UI.
- **The studio catalog** (`catalogs::studio`, id
  `https://zavora.ai/a2ui/catalogs/adk-ui-studio/1.0/catalog.json`): the insight catalog plus
  `App` (pages the host navigates; the web home shows them as tabs), `Page`, `Region` and
  `Scene3D` (primitive shapes with checked labels and `#rrggbb` colours, at most 64; no models,
  textures or actions). The home draws a scene in 3D: each hex colour as the nearest chart
  colour, and a plane upright in x-y as three.js draws one. The containers declare what they
  hold, so `genui` checks every held component, its fields and buttons included. Builders in
  `studio`.
- `kit::ApprovedKit`: only an approved, valid kit applies. `surface_css` sets the adk-surface
  tokens a custom host's `DrawnView` and adk-ui's renderers read (card, border, text, accent,
  focus, fonts, radius, the eight chart colours), never a decision's or a status's;
  `a2ui_theme` gives a third-party A2UI renderer the primary colour alone.
- Builders for every insight component (`insight::Chart`, `Metric`, …, `bind`), a `Screen`
  that is checked as `genui` checks it and gives `render_screen`'s arguments, and
  `prompts::instructions` for the model.
- **The spatial catalog** (`catalogs::spatial`, id
  `https://zavora.ai/a2ui/catalogs/adk-ui-spatial/1.0/catalog.json`): the studio catalog plus
  - 3D charts: `Bar3D`, `Scatter3D`, `SurfacePlot`, `Graph3D` and `Globe`;
  - `Scene`: groups, lines, arrows, text, palette colours and states, lights, cameras,
    motion, panels beside objects, and `pickInto`, where picking an object fills a choice
    field the drawing shows and never sends;
  - `DepthCard` (tilt, flip, layers) and `Stage` (stage, wall, carousel).

  Each one projects onto `genui`'s own scenes and depth effects. The ADK-Rust web home draws
  them natively (WebGL, in its palette), and the terminal shows their words. Every limit is
  checked again on bound data, which the schema can't see. Builders are in `spatial`.
  `catalogs::HOME_DRAWN` lists what the home draws, and `prompts::instructions` tells a spatial
  agent when 3D carries information.
- **`@zavora-ai/adk-ui-react` 3**: renderers for `@zavora-ai/adk-surface-react`'s `DrawnView`
  (`adkUiRenderers`, `insightRenderers`, `studioRenderers`, and `spatialRenderers` for the
  spatial catalog's 2D components).
  - A renderer draws from the server's resolved, checked properties and never gets a way to
    send; forms inside an app's pages stay `DrawnView`'s own.
  - `Chart` (recharts) loads lazily as an optional peer and falls back to the home's own view.
  - Vega-Lite charts, 3D scenes, cards with depth and 3D layouts have no renderer:
    `DrawnView` draws them.
  - `Sparkline`, `Metric`, `KeyValueList`, `ProgressBar`, `Badge`, `AppShell`, `PageView` and
    `RegionView` need no peer and also draw a host's own data.
  - `KitProvider` applies approved kits only.
  - The package ships its LICENSE, and names its repository directory, issues page and
    Node 20 floor.
- **Drawing evaluation**: `eval::score` checks a drawing as genui will and reports whether it
  is shown (with the refusal the model is told), its dark-pattern warnings, the expected
  components it lacks, unlabelled fields, size and depth, folded into 0 to 100 points;
  `eval::scenarios` are prompts across the three catalogs, and `examples/drawing_eval.rs`
  scores the sample drawings or a JSONL file of a model's drawings, exiting non-zero on a
  refusal.
- **MCP Apps** (feature `apps`): `apps::UiApps`, an MCP server with two tools
  (`adk-ui-apps` serves both over stdio; `UiApps::three_d()` serves the first alone).
  - `show_drawing` shows a drawing in adk-ui's catalogs read-only in any MCP Apps host:
    checked by rule 5 and resolved on the server (`genui::drawn_tree`), drawn by a page that
    sets text only and reaches no site. It declares no actions, so a button is refused.
- **3D apps**: `show_3d_app` shows three.js the model writes in genui-apps' sandbox, on its
  own origin.
  - The page loads three.js 0.185.1 from its one declared site. It runs the code in a block
    of its own, given `THREE`, `scene`, `camera`, `onFrame`, `label`, `theme` and more, so
    code that declares a given name again still runs.
  - The page owns the render loop and its Pause button.
  - The server refuses code that shows a decision's words or reaches outside its frame.
  - `insight_agent` serves it with `ADK_UI_3D_APPS=1`. The spatial prompt states every scene
    limit, a motion's 0.2 to 120 seconds included, so a model knows when to use an app
    instead.
- `catalogs/`: the catalog documents and an index, written from the Rust catalogs and
  drift-checked by `cargo test`; the React registry is checked against them. A2UI 1.0
  documents are in `catalogs/a2ui-1.0/`: only the components each catalog adds, which a 1.0
  surface mixes with `forms`, its rules as `instructions` (`prompts::catalog_rules`) and
  `App`/`Page` composition rules, checked against the pinned A2UI 1.0 meta-schema.
- Agents are told to bind data in A2UI 1.0's syntax (`{"@path": "/sales"}`); the render
  tools read v0.9.1's too.
- Eleven sample drawings (`fixtures/drawings/`, six in studio and five in spatial), the
  `insight_home` example (the ADK-Rust web home over a scripted agent that draws them), and
  `examples/custom_host` (a Vite app drawing them with `DrawnView`, adk-ui's renderers and two
  approved kits).
- The `capabilities_tour` example (feature `apps`): the web home over a scripted agent that
  walks through Vega-Lite charts, a live update, a held and sealed send, dark-pattern warnings
  and a refusal, both MCP Apps tools and two MCP elicitations, one stop a message; `--wire`
  prints what has no screen.
- The `insight_agent` example: a harness agent on an Anthropic model (adaptive thinking, prompt
  caching) that studies a directory with its own workspace tools and answers with adk-ui's
  catalogs (spatial by default), everything configured from the environment and nothing
  scripted.
- End-to-end tests:
  - a harness agent draws every sample through `toolset_for`, in studio and in spatial;
  - a home that registers the catalogs shows every one, its scenes with the server's
    summaries;
  - a form inside a drawing is held for the person with only its fields;
  - a dock picked in a 3D scene is sent as a field and asks first.
- `docs/MIGRATING_TO_3.md` maps every 2.x tool, module and React export to its replacement,
  and `docs/ARCHITECTURE.md` says how adk-ui fits on genui and how to add a component.
- docs.rs builds every feature and marks `apps` as feature-gated; public items are documented
  (`#![warn(missing_docs)]`).

### Removed

- Every render tool and `UiToolset` (`render_form`, `render_card`, `render_table`,
  `render_chart`, `render_layout`, `render_page`, `render_app`, `render_screen`,
  `render_confirm`, `render_alert`, `render_toast`, `render_modal`, `render_progress`,
  `render_kit`, `patch_surface` and the surface persistence tools): agents draw with
  `genui`'s render tools; a confirm dialog an agent draws is never an approval.
- The protocol adapters and profiles (`interop` for AG-UI, MCP Apps and AWP, the legacy
  `adk_ui` profile, `protocol_capabilities`), the A2UI model (`a2ui`), the HTML renderer
  (`html`), templates (some asked for passwords), notifications, the legacy schema and
  validation, the catalog registry and the 2.x catalogs: `genui` owns A2UI v0.9.1, AG-UI and
  MCP Apps.
- The `adk-core`, `awp` and `remote-catalogs` features, and the 2.x examples (`ui_server`,
  `agentic_ui`, the Agent UI Studio client). The 2.x protocol migration notes, roadmaps and
  release notes stay on the `release/2.x` branch.
- The crate-root re-exports of the kit types (`adk_ui::KitGenerator` and the rest) and the
  public `kit::spec` and `kit::generator` modules: every kit type is at `adk_ui::kit`.
- From `@zavora-ai/adk-ui-react`: the standalone `Scene3D` and its `three` peer (`DrawnView`
  draws every scene natively), `Renderer`, `StreamingRenderer`, the A2UI renderer, parser
  and store, the protocol client, `uiEventToMessage` (UI events no longer reach the model as the
  person's words), `ApplicationRenderer`, `ReasoningPanel`, kit slots and `kit://` assets, and
  the `react-markdown`, `lucide-react` and `clsx` dependencies.

### Changed

- Builds on `adk-surface` 3 (`genui`, with its scenes and depth effects) and `schemars` 1.0;
  Rust 1.95. `.cargo/config.toml.example` builds against an unreleased adk-rust checkout.
- A brand kit generates its manifest, tokens and CSS only, never a catalog or instructions for
  the model; its recipes use the `forms` names (`TextField` for 2.x's `TextInput`). A kit is the
  host's: the agent never picks one and a drawing never carries one (`docs/BRAND_KITS.md`).
  `KitArtifacts::theme_css` sets the adk-surface tokens an approved kit sets (`--card`,
  `--accent`, `--chart-1` …), so a draft previews as it will look once approved; 2.x's
  `--adk-ui-*` variables, which nothing reads any more, are gone. A generated kit says
  `generatedBy: "adk-ui/KitGenerator"`.
- The crate ships only its library, catalog documents, docs and LICENSE (`include`).
- The crate and `@zavora-ai/adk-ui-react` are both 3.0.0, released after ADK-Rust 3.0
  (`adk-surface`, `@zavora-ai/adk-surface-react`) is published ([RELEASING.md](RELEASING.md)).
- The 2026 generative UI market brief, written for 2.x, stays on the `release/2.x` branch.

## [2.2.1] - Unreleased (`release/2.x`)

The 2.x line is in maintenance for ADK-Rust 2: security fixes only, until the `adk_ui` server
profile's sunset on 2026-12-31.

### Fixed

- `ApplicationRenderer` passes the application's `state` to every rendered component as its data
  model, so tables, charts and key-value views bound with `data_source.binding_path` show the
  application's data instead of an empty view.

## [2.2.0] - 2026-08-22

### Added

- Versioned surface references are now written to ADK `EventActions.state_delta` by every render tool.
- `SurfaceStore` with optimistic revisions, owner scoping, in-memory and atomic filesystem backends, plus save/load/list/delete tools enabled by `UiToolset::with_persistence`.
- `patch_surface`, `UiUpdate::patch`, typed UI event envelopes, and an agent-owned notification channel with AG-UI custom-event projection.
- Declarative data sources for charts, tables, and key-value views with manual, interval, and push refresh policies.
- Numeric and time axes, time formatting, rolling windows, scatter plots, and sparklines.
- Version, owner, and update timestamps for multi-page applications with persisted page-upsert semantics.

### Changed

- Chart tools now expose axis labels, colors, legend visibility, time-series options, and data bindings.
- Table tools now honor sortable, per-column sorting, page size, striped rows, and data bindings.
- React renderers resolve JSON Pointer data bindings and can request interval or TTL refreshes through `DataSourceProvider`.
- UI-to-agent messages retain human-readable fallbacks while carrying typed, surface-correlated event envelopes.

### Fixed

- Application page route changes now keep navigation and the initial route consistent.
- AG-UI capability discovery now reports the available notification channel and its host-bridge boundary.

## [2.1.0] - 2026-08-14

### Added

- Versioned 2026 capability discovery with protocol-specific versions, maturity, negotiation keys, content types, transports, fallbacks, and scoped support claims.
- A2UI v0.9.1 stable and v1.0 release-candidate capability models, including catalog exchange and correlated RPC support boundaries.
- Typed AG-UI capability snapshots plus interrupt, resume, and human-in-the-loop wire types.
- MCP Apps 2026-01-26 extension, host-capability, lifecycle, and progressive-fallback models.
- Lazy-loaded chart rendering for the shared React package and example client.
- Versioned brand-kit manifests with provenance, approval state, validated assets, semantic tokens, component recipes, templates, and agent constraints.
- React brand-kit provider, approved-kit selection, typed component slots, and `kit://` asset resolution for both legacy and native A2UI renderers.

### Changed

- Agent UI Studio now exposes honest per-feature capability status instead of a single coarse compatibility label.
- OpenAI demos now default to `gpt-5.6-sol` through the Responses API with medium reasoning, while preserving provider and model overrides.
- Generated layouts use responsive KPI grids, stable component identity, safer nested layouts, and clearer empty/error states.
- Forms preserve client-side values and submission semantics while generated charts are split into an on-demand bundle.
- Protocol documentation and migration guidance now distinguish implemented, partial, candidate, host-owned, planned, and legacy behavior.
- Approved `kit_id` values now propagate through legacy responses and protocol catalog identity so one company design contract drives all rendering paths.

### Fixed

- Corrected A2UI conversion paths that could drop component identity or produce malformed nested layouts.
- Prevented generated form controls and action flows from losing state during rerenders.

### Packaging

- `@zavora-ai/adk-ui-react` moves from its own `0.x` line to **2.1.0** so the React package and the `adk-ui` crate share one version. The public API stays a strict superset of the published `0.3.2`.
- `three` and `recharts` become **optional peer dependencies**. Both are only reachable from lazily loaded renderers, so hosts that render neither charts nor 3D scenes drop roughly 63 MB from a fresh install.
- A missing optional peer now degrades the affected component to an accessible note naming the package to install instead of propagating the failed chunk import to the host application.
- Added a `prepublishOnly` gate that runs lint, typecheck, and the test suite so a publish cannot ship an unbuilt or unverified `dist`.

## [2.0.0] - 2026-08-13

### Breaking

- Bumped crate to **2.0.0** for ADK-Rust 2.0 lockstep.
- MSRV raised from **1.85** to **1.95** (matches ADK-Rust 2.0).
- Dependencies target the published `adk-*` / `awp-types` **2.x** crates from crates.io.
- Standalone `compat` `Part::InlineData` gains optional `uri` / `annotations` fields for parity with adk-core 2.0.
- Example `ui_server` updated for adk-core 2.0 `Part` fields and `RunnerConfig` intra-compaction slots.
- **AG-UI wire shapes** align with the current protocol: `STATE_SNAPSHOT.snapshot`, step `stepName`, run-error optional lineage, surface streams emit `ACTIVITY_SNAPSHOT` before `CUSTOM`.

### Changed

- **Agent UI Studio**: replaced the example client test-harness layout with a scenario-led, responsive workspace that clearly separates goals, live surfaces, and run diagnostics.
- **Protocol discovery**: added a visual protocol matrix with explicit support tiers and compact server capability signals.
- **Responsive preview**: added desktop/mobile surface modes and a mobile-first stacked studio layout.
- **Run inspector**: moved events, assistant output, AG-UI activity, reasoning, and A2UI responses into focused tabs.
- **A2UI**: every envelope stamps `version: "v0.9.1"`; MIME `application/a2ui+json`; client action + `VALIDATION_FAILED` envelopes; catalog schema accepts version + optional bulk createSurface fields.
- **AG-UI**: full reasoning event family (`REASONING_*`), activity/state official field names, `run_started` / `run_finished` helpers; tier **NativeSubset**.
- **MCP Apps**: extension id `io.modelcontextprotocol/ui`, SEP Final status, and capability advertisement limited to implemented bridge behavior; tier **NativeSubset**.
- **React package**: Recharts 3.10 and React Markdown 10, React 18+ peer contract, and a direct esbuild 0.28 build path with CommonJS/ESM/declaration output.
- Demo default LLM provider is **OpenAI** (`OPENAI_API_KEY`); switch with `ADK_UI_PROVIDER=gemini` (+ `GOOGLE_API_KEY`).
- Added `UI_PREFERRED_PROTOCOL` (`a2ui`) for new integrations while keeping legacy default `adk_ui` until the 2026-12-31 sunset.
- Added repo `rust-toolchain.toml` pinning **1.95.0**.

### Added

- A zero-setup local feature demo that renders a complete support surface, reasoning trace, and correlated A2UI `actionResponse` without a running server or API key.
- A v2 landing experience and in-product release narrative explaining the user-facing upgrade.
- `compat::inline_data_part()` helper for constructing `Part::InlineData` under both standalone and `adk-core` feature paths.
- **Agentic examples**: `examples/agentic_ui` (A2UI / AG-UI / MCP Apps apps) and three agentic agents on `ui_server`.
- Shared multi-provider factory (OpenAI default) under `examples/common/provider.rs`.
- **A2UI v1.0 actionResponse** end-to-end: Rust `ActionResponseMessage` / client `wantResponse`+`actionId`, React pending-action tracking, example server correlated response, data-model write via `responsePath`.
- **AG-UI reasoning UI**: `ReasoningPanel` + `applyReasoningEvent` in `@zavora-ai/adk-ui-react`, wired in the example client event stream.

### Fixed

- Local feature-demo actions no longer attempt a server request; the client completes and displays the correlated action response in-browser.

## [1.0.0] - 2026-05-15

### Changed

- Upgraded all `adk-*` dependencies to 1.0.0 (adk-core, adk-agent, adk-model, adk-cli, adk-runner, adk-server, adk-session).
- Upgraded `awp-types` to 1.0.0.
- Bumped adk-ui to 1.0.0 — stable release aligned with the ADK-Rust 1.0 ecosystem.

## [0.8.0] - 2026-05-07

### Changed

- Upgraded all `adk-*` dependencies to 0.8.0 (adk-core, adk-agent, adk-model, adk-cli, adk-runner, adk-server, adk-session).
- Switched `awp-types` from local stub to published crate (v0.8.0 on crates.io).
- Updated `ToolEnvelope::to_awp_response()` to match the published `AwpResponse` struct (uses `Uuid`, `AwpVersion`, `status` fields).
- Added `uuid` as optional dependency (gated behind `awp` feature).
- Removed local `awp-types/` stub directory.
- Clean upgrade — no other API changes required.

## [0.7.0] - 2026-04-25

### Added

- **AWP (Agentic Web Protocol) integration** behind an optional `awp` Cargo feature flag — zero cost for non-AWP consumers.
- **HTML renderer** (`src/html.rs`) — converts all 30+ typed `Component` variants into clean, embeddable HTML with inline styles. Available unconditionally (no feature gate).
- **`BandwidthMode` enum** (`Full` / `Low`) — adaptive rendering that strips Chart, Image, Skeleton, and Spinner components and inline styles for constrained connections (2G/3G per AWP Section 41.4). Marked `#[non_exhaustive]` for future `Medium` variant.
- **`HtmlRenderOptions`** struct with `bandwidth_mode` and `class_prefix` for namespaced CSS classes.
- **`AwpAdapter`** implementing `UiProtocolAdapter` — produces JSON payloads with `protocol: "awp"`, component tree, and optionally rendered HTML. Configurable `include_html`, `bandwidth_mode`, and `class_prefix`.
- **`UiProtocol::Awp`** variant (conditional on `awp` feature).
- **`ToolEnvelopeProtocol::Awp`** variant with conditional `awp_version` and `request_id` fields, plus `to_awp_response()` conversion method.
- **`UiToolset::to_capability_entries()`** — exports enabled render tools as `awp_types::CapabilityEntry` values for AWP manifest generation.
- **`AWP_PROTOCOL_CAPABILITY`** constant — `UiProtocolCapabilitySpec` for AWP (version 1.0, CompatibilitySubset tier, Draft spec track) exposed for downstream crates.
- **AWP compat module** (`src/awp_compat.rs`) — re-exports `CapabilityEntry`, `CapabilityManifest`, `AwpResponse`, `AwpVersion`, `CURRENT_VERSION` from `awp-types`.
- **AWP protocol in React client** — protocol selector, prompt instruction, HTML iframe rendering, capability signal display.
- **AWP protocol in example server** — `UiProfile::Awp`, capabilities endpoint includes AWP when feature is active, AWP-specific prompt instruction.
- Local `awp-types` stub crate for development (to be replaced by upstream when published).
- 36 new unit tests for HTML renderer covering all component types, bandwidth modes, class prefixes, and HTML escaping.
- 5 new AWP adapter tests.

### Changed

- `render_ui_response_with_protocol` AWP branch renders HTML from original typed `UiResponse.components` (not from A2UI-format surface summaries).
- React client `isLowFidelitySurface` skips quality check for AWP surfaces with HTML (prevents false retry triggers).
- React client `extractSurfaceFromToolResponse` extracts `html` field from AWP payloads for iframe rendering.

## [0.6.0] - 2026-04-17

### Changed

- Upgraded all `adk-*` dependencies to 0.6.0 (adk-core, adk-agent, adk-model, adk-cli, adk-runner, adk-server, adk-session).
- Updated example server `FunctionResponseData` initialization to include new `file_data` and `inline_data` fields from adk-core 0.6.

## [0.5.0] - 2026-03-30

### Changed

- Upgraded all `adk-*` dependencies to 0.5.0 (adk-core, adk-agent, adk-model, adk-cli, adk-runner, adk-server, adk-session).
- Migrated `AdkError::Tool(...)` enum variant to `AdkError::tool(...)` constructor across all render tools, protocol output, interop adapters, and MCP Apps validation to match the adk-core 0.5 API.
- Updated example server `runner.run()` to use typed `UserId` and `SessionId` parameters.
- Revised README with professional structure, consistent formatting, and improved scannability.

### Added

- `AdkError::tool()` constructor on the standalone compat fallback for parity with adk-core 0.5.
- Root `package.json` for npm workspace support across the React client and shared renderer package.
- Verification screenshot for A2UI dashboard render.

### Fixed

- Compilation under `--features adk-core` against adk-core 0.5.0.

## [0.4.0] - 2026-03-20

### Added

- Initial public release with 30 component types and 13 high-level render tools.
- Protocol support for A2UI (v0.9 draft-aligned), AG-UI (stable 0.1 subset), MCP Apps (SEP-1865 subset), and legacy adk_ui.
- Full render tool × protocol matrix coverage (39/39 combinations tested).
- A2UI validator with JSON Schema-based message validation.
- Interop adapters for A2UI JSONL, AG-UI event streams, and MCP Apps bridge payloads.
- React reference client (`@zavora-ai/adk-ui-react`) with protocol profile selector.
- Rust example server with SSE streaming and multi-agent demo apps.
- Runtime capability metadata exposed via `/api/ui/capabilities`.
- Protocol migration guide and deprecation timeline for legacy `adk_ui` profile.
