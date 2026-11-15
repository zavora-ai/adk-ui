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
  `Chart` (bar, line, area, pie, scatter over category, number or time axes), `Sparkline`,
  `Metric` (value, unit, currency or percent, change, status, trend), `KeyValue`, `Progress`
  and `Badge`. Every one only shows, binds its data with `{"path"}`, and says what the
  ADK-Rust web home and terminal show for it: charts as the home's native chart, the rest as
  `forms` components. `genui` checks every string in them and in the data they bind.
- `toolset()` and `toolset_for()`: `genui`'s render tools drawing in an adk-ui catalog, under
  their usual names; `catalogs::register` and `catalogs::ag_ui` register the catalogs with a
  home and with AG-UI.
- Builders for every insight component (`insight::Chart`, `Metric`, …, `bind`), a `Screen`
  that is checked as `genui` checks it and gives `render_screen`'s arguments, and
  `prompts::instructions` for the model.

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
  `agentic_ui`). The protocol migration notes move to `docs/archive/2.x/`.

### Changed

- Builds on `adk-surface` 3 (`genui`) and `schemars` 1.0. Until ADK-Rust 3.0 is on crates.io,
  build against an adk-rust checkout with `.cargo/config.toml.example`.
- A brand kit generates its manifest, tokens and CSS only, never a catalog or instructions for
  the model; its recipes use the `forms` names (`TextField` for 2.x's `TextInput`).

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
