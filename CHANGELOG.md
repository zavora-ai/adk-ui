# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2026-08-09

### Breaking

- Bumped crate to **2.0.0** for ADK-Rust 2.0 lockstep.
- MSRV raised from **1.85** to **1.95** (matches ADK-Rust 2.0).
- Dependencies target the published `adk-*` / `awp-types` **2.x** crates from crates.io.
- Standalone `compat` `Part::InlineData` gains optional `uri` / `annotations` fields for parity with adk-core 2.0.
- Example `ui_server` updated for adk-core 2.0 `Part` fields and `RunnerConfig` intra-compaction slots.
- **AG-UI wire shapes** align with the current protocol: `STATE_SNAPSHOT.snapshot`, step `stepName`, run-error optional lineage, surface streams emit `ACTIVITY_SNAPSHOT` before `CUSTOM`.

### Changed

- **A2UI**: every envelope stamps `version: "v0.9.1"`; MIME `application/a2ui+json`; client action + `VALIDATION_FAILED` envelopes; catalog schema accepts version + optional bulk createSurface fields.
- **AG-UI**: full reasoning event family (`REASONING_*`), activity/state official field names, `run_started` / `run_finished` helpers; tier **NativeSubset**.
- **MCP Apps**: extension id `io.modelcontextprotocol/ui`, SEP Final status, and capability advertisement limited to implemented bridge behavior; tier **NativeSubset**.
- **React package**: Recharts 3.10 and React Markdown 10, React 18+ peer contract, and a direct esbuild 0.28 build path with CommonJS/ESM/declaration output.
- Demo default LLM provider is **OpenAI** (`OPENAI_API_KEY`); switch with `ADK_UI_PROVIDER=gemini` (+ `GOOGLE_API_KEY`).
- Added `UI_PREFERRED_PROTOCOL` (`a2ui`) for new integrations while keeping legacy default `adk_ui` until the 2026-12-31 sunset.
- Added repo `rust-toolchain.toml` pinning **1.95.0**.

### Added

- `compat::inline_data_part()` helper for constructing `Part::InlineData` under both standalone and `adk-core` feature paths.
- **Agentic examples**: `examples/agentic_ui` (A2UI / AG-UI / MCP Apps apps) and three agentic agents on `ui_server`.
- Shared multi-provider factory (OpenAI default) under `examples/common/provider.rs`.
- **A2UI v1.0 actionResponse** end-to-end: Rust `ActionResponseMessage` / client `wantResponse`+`actionId`, React pending-action tracking, example server correlated response, data-model write via `responsePath`.
- **AG-UI reasoning UI**: `ReasoningPanel` + `applyReasoningEvent` in `@zavora-ai/adk-ui-react`, wired in the example client event stream.

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
