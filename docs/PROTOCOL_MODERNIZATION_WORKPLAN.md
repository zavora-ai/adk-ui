# adk-ui Protocol Modernization Workplan

Status: Complete
Prepared: 2026-03-19
Completed: 2026-03-20
Scope: `a2ui`, `ag_ui`, `mcp_apps`, Rust interop adapters, React client, example apps, tests, and documentation

## Purpose

This document defines the implementation plan for bringing `adk-ui` closer to the current upstream protocol landscape as of 2026-03-19.

The immediate driver is that the three protocol targets have moved materially since the last refresh:

- A2UI `v0.9` is available as a draft and changes the recommended shape and metadata model.
- AG-UI has expanded its event model and now documents reasoning and richer activity/message streams.
- MCP Apps has the largest delta and now centers around extension negotiation and an interactive app bridge.

This plan is intentionally implementation-focused. It is not a product pitch and it is not a migration guide for existing consumers.

For framework follow-on work in the sibling `adk-rust` repo, see `docs/FRAMEWORK_CONTINUITY_ROADMAP.md`.

Historical note:

- The repository-state sections below describe the starting point when this plan was written.
- As of 2026-03-20, the scoped modernization work in this document is complete; remaining items belong to future framework hardening rather than unfinished app integration.

## Progress Comments

### 2026-03-20

- Workstream 0 is effectively complete in both repos. Capability metadata, docs, and example copy now describe `a2ui`, `ag_ui`, and `mcp_apps` as implemented subsets instead of implying full parity.
- Workstream 1 is substantially landed in `adk-ui`. The React package now has protocol-native ingestion and outbound paths for AG-UI and MCP Apps, even though some compatibility parsing remains in place for the bundled example server/runtime.
- Workstream 2 is complete for the scoped modernization pass. The `adk-ui` example stack now uses MCP Apps initialize/message/update-model-context flows, the sibling `adk-rust` repo exposes a canonical MCP Apps constructor layer plus runnable source examples, and the framework bridge now covers notification polling and resource/tool list-changed host flows with fixture-backed tests.
- Workstream 3 is complete for the scoped modernization pass. Stable AG-UI text/tool/activity event parsing and protocol-native AG-UI input transport are in place, the shared React parser and example client reconstruct chunked tool-call args, the framework emits native partial `TEXT_MESSAGE_CHUNK`, `REASONING_MESSAGE_CHUNK`, `TOOL_CALL_CHUNK`, `ACTIVITY_SNAPSHOT`, and `ACTIVITY_DELTA` events for the supported path, and the bundled example stack can exercise those event families live.
- Workstream 4 is complete in `adk-ui`. The shared React package stores `createSurface` metadata, emits metadata-aware A2UI client envelopes, supports `VALIDATION_FAILED`/generic error payloads, executes local A2UI `functionCall` actions, and covers the practical basic-catalog function/check set with test-backed renderer validation behavior.
- Workstream 6 is complete for the scoped modernization pass. Protocol coverage now includes Rust server tests, React package tests, browser validation in the app repo, a framework-side MCP Apps source example, and fixture-backed conformance checks for protocol capabilities, canonical MCP Apps bridge flows, canonical MCP Apps output, and AG-UI native SSE summaries in the sibling framework repo.

### Current Execution Comment

- The scoped execution plan in this document is complete.
- Remaining future work is now framework hardening rather than unfinished `adk-ui` modernization: capability-schema centralization in `adk-rust`, deeper AG-UI runtime-owned semantics below the server serializer, durable MCP Apps bridge persistence for richer deployments, and any extra fixture/browser coverage that accompanies new transport surface area.

## Executive Summary

At the start of this plan, the repository presented broader protocol support than it actually implemented.

- `a2ui` support is partly aligned with the newer flat component shape, but coverage is incomplete.
- `ag_ui` support is mostly a compatibility tunnel that wraps an `adk-ui` surface inside a custom event.
- `mcp_apps` support is currently a static HTML-resource subset, not a modern bridge-aware MCP Apps implementation.

The work should be executed in this order:

1. Make capability claims truthful.
2. Separate protocol-native client handling from the current A2UI normalization path.
3. Implement MCP Apps modernization first.
4. Refresh AG-UI stable event support next.
5. Close the remaining A2UI `v0.9` gaps.

## Upstream Snapshot

### A2UI

- Official `v0.9` spec status: draft
- `v0.9` created: 2025-11-20
- `v0.9` last updated: 2025-12-03
- `v0.8` remains the stable production spec on the official site

Relevant upstream shifts:

- Flat property-based component discriminator is the preferred shape.
- Capability and metadata exchange now matter more at the transport boundary.
- The protocol surface includes client metadata such as `a2uiClientCapabilities`, `inlineCatalogs`, and `a2uiClientDataModel`.
- `VALIDATION_FAILED` and explicit error handling are part of the expected loop.
- The basic function and check catalog is broader than the subset currently implemented in the React package.

### AG-UI

- The current docs define a materially richer event model than this repository supports today.
- Stable concepts now include richer run, message, tool-call, state, activity, and reasoning event families.
- Agent capabilities discovery exists as a draft and should not be treated as a phase-one blocker.

Relevant upstream shifts:

- Newer event families such as `RunError`, `TextMessageContent`, `TextMessageChunk`, `ToolCallChunk`, `MessagesSnapshot`, `ActivitySnapshot`, and `ActivityDelta`
- Reasoning concepts and event handling are documented explicitly
- The protocol has moved beyond a minimal lifecycle-plus-custom-event model

### MCP Apps

- SEP-1865 status: final
- SEP-1865 created: 2025-11-21
- Current extension docs center around negotiated app capabilities and an interactive bridge

Relevant upstream shifts:

- Extension negotiation through the `extensions` capability surface
- Initialization handshake for app hosts and embedded apps
- Host/app communication through a bridge rather than static resource rendering alone
- App-only tool visibility and richer model-context updates
- Host context, theming, and bridge-driven interaction are now first-class

## Initial Repository State

### Capability Signaling

Current capability metadata is defined in `src/protocol_capabilities.rs`.

Current advertised support:

- `a2ui` version `0.9` with features `jsonl`, `createSurface`, `updateComponents`, `updateDataModel`
- `ag_ui` version `0.1` with features `run_lifecycle`, `custom_events`, `event_stream`
- `mcp_apps` version `sep-1865` with features `ui_resource_uri`, `tool_meta`, `html_resource`

This metadata is directionally useful but too coarse. It does not distinguish:

- stable support vs draft-aligned support
- native protocol handling vs compatibility tunneling
- implemented subsets vs aspirational compatibility

### A2UI

Strengths:

- Rust helper code already emits the flat `v0.9`-style component discriminator in `src/a2ui/components.rs`.
- The React client already supports data bindings and a small local function registry in `packages/adk-ui-react/src/a2ui/bindings.ts`.

Gaps:

- The React parser only models `createSurface`, `updateComponents`, `updateDataModel`, and `deleteSurface`.
- The React-side built-in function set is much smaller than the current upstream basic catalog.
- Transport-level handling for newer metadata-driven flows is missing or implicit rather than explicit.
- The project currently presents `v0.9` as if support were broader and cleaner than it is.

### AG-UI

Strengths:

- Rust emits a valid event envelope for the currently implemented subset.
- The adapter layer is already isolated in `src/interop/ag_ui.rs`.

Gaps:

- The event enum does not model the newer stable event families.
- Event field names and payload shapes are older and more limited than the current docs.
- The React client only extracts a `CUSTOM` event named `adk.ui.surface` and converts it back into A2UI JSONL.
- Outbound client events are likewise emitted as a custom compatibility shape rather than a full AG-UI-native client path.

### MCP Apps

Strengths:

- The Rust side already models `ui://` resources, tool metadata, CSP, permissions, and inline HTML payloads.
- Domain validation and resource metadata hardening already exist.

Gaps:

- The implementation is resource-centric and static.
- There is no initialize handshake, negotiated capability model, or bridge-aware event flow.
- The React client parses an embedded `<script id="adk-ui-surface">` payload from HTML instead of participating in an MCP Apps app bridge.
- Outbound client events use a placeholder-style `ui.event` method rather than a modern bridge integration.

### Client Architecture

The largest architecture problem is in `packages/adk-ui-react/src/protocols.ts`.

Today the client does this:

- accept A2UI directly
- convert AG-UI custom events into synthetic A2UI JSONL
- convert MCP Apps HTML resources into synthetic A2UI JSONL

That architecture was acceptable for initial interop, but it is now the main blocker for accurate protocol support. We need protocol-native ingestion paths.

## Goals

### Primary Goals

- Publish truthful support claims for all three protocols
- Introduce protocol-native handling paths in the React package
- Modernize MCP Apps support to the current negotiated app model
- Refresh AG-UI support to current stable event families
- Close the highest-value A2UI `v0.9` gaps without overcommitting to draft-only features

### Secondary Goals

- Preserve compatibility for existing `adk_ui` and current example users
- Improve test coverage so protocol support is verifiable rather than implied
- Make protocol support boundaries clear in docs, examples, and capability endpoints

## Non-Goals

- Implement every upstream draft feature immediately
- Remove the legacy `adk_ui` profile before the published sunset window
- Break existing example flows in order to reach protocol purity
- Turn `adk-ui` into a full host implementation for every external protocol feature in one pass

## Workstreams

### Workstream 0: Truthfulness and Inventory

Objective: make claims, docs, and examples match reality before shipping deeper protocol work.

Tasks:

- Refine `UI_PROTOCOL_CAPABILITIES` so it describes supported subsets, not only protocol names and versions.
- Add feature markers that distinguish `native`, `compat`, and `draft-aligned` behavior where appropriate.
- Audit README, migration docs, example UI copy, and example prompts for overstated protocol support.
- Document the minimum supported subset for each protocol.

Deliverables:

- Updated capability metadata in Rust
- Updated docs and example copy
- Clear support matrix that differentiates native support from compatibility mode

Acceptance criteria:

- `/api/ui/capabilities` no longer implies full parity where only subset support exists.
- README and migration docs do not describe AG-UI and MCP Apps as broader than the code supports.

### Workstream 1: Client Protocol Boundary Refactor

Objective: stop treating A2UI as the universal internal transport for all protocols.

Tasks:

- Introduce protocol-specific parse paths in `packages/adk-ui-react`.
- Separate protocol ingestion from surface rendering.
- Define a protocol-neutral render state that can be produced natively by each protocol path.
- Split outbound event serializers by protocol rather than using compatibility placeholders.
- Preserve the existing A2UI flow as one protocol implementation, not the base abstraction for all others.

Deliverables:

- New protocol parser/reducer boundaries in the React package
- Updated store contract for protocol-native surface state
- Compatibility shims retained only where explicitly intended

Acceptance criteria:

- AG-UI payloads no longer require conversion into synthetic A2UI JSONL to render.
- MCP Apps payloads no longer require HTML script extraction to render.
- A2UI remains supported without regression.

Dependencies:

- Workstream 0 should land first.

### Workstream 2: MCP Apps Modernization

Objective: move from static HTML resource compatibility toward current MCP Apps behavior.

Progress comment:

- Partially landed. The app repo now exercises initialize/message/update-model-context flows, and the framework repo now provides `McpUiBridgeSnapshot`, `McpUiToolResult`, `McpUiToolResultBridge`, and a runnable `mcp_apps_tool_result` example. Remaining work is broader adoption and conformance coverage, not first-use support.

Tasks:

- Add Rust-side types for negotiated MCP Apps capabilities and initialize flows where needed.
- Model bridge-aware metadata separately from static resource metadata.
- Support host/app initialization and model-context update flows in the client package.
- Add app-only tool visibility semantics and structured tool-result handling.
- Keep the current static HTML resource path behind a clearly named compatibility mode until the modern path is stable.
- Decide whether the example app should simulate a host bridge or embed one directly.

Deliverables:

- Updated Rust MCP Apps interop model
- Bridge-aware TypeScript client integration
- Example flow that demonstrates negotiated MCP Apps interaction
- Compatibility fallback path retained only as a documented subset

Acceptance criteria:

- MCP Apps support includes capability negotiation and initialization rather than static resource rendering alone.
- The React example can run a real bridge-aware interaction path.
- Tool visibility and structured tool-result handling are covered by tests.

Dependencies:

- Workstream 1

Risks:

- The implementation can sprawl if we try to build a full host and full app at the same time.
- The bridge API needs to be scoped to the minimal viable supported path first.

### Workstream 3: AG-UI Stable Event Refresh

Objective: align AG-UI support with the current stable event model before taking on draft-only expansion.

Progress comment:

- Substantially landed. The client and example stack now handle richer stable event families, native run input, chunked tool-call arg reconstruction, and AG-UI activity snapshots/deltas, but end-to-end runtime emission is still not fully native in the framework and the app keeps compatibility fallbacks for that reason.
- Substantially landed. The framework now emits native partial text, reasoning, and tool-call chunk events for AG-UI protocol-native streams when the underlying runtime surfaces string deltas, and it emits input-derived `ACTIVITY_SNAPSHOT` plus `ACTIVITY_DELTA` events for frontend-only activity continuity. Richer snapshots and deeper runtime-owned AG-UI emission are still incomplete.

Tasks:

- Expand the Rust AG-UI event enum and payload types to cover the current stable event families that matter for `adk-ui`.
- Update field names and shapes where the upstream protocol has evolved.
- Map existing tool lifecycle behavior to richer AG-UI events.
- Add a native AG-UI reducer/parser in the React client.
- Keep the `adk.ui.surface` custom event only as a compatibility bridge during migration.
- Evaluate reasoning event support and implement the minimum practical subset for UI rendering and observability.

Deliverables:

- Refreshed Rust AG-UI type model
- Updated adapter logic
- Updated React AG-UI client path
- Tests for lifecycle, message, tool-call, activity, and error flows

Acceptance criteria:

- AG-UI support is no longer defined primarily by a custom event tunnel.
- Stable event coverage is explicitly tested.
- Compatibility custom events remain optional rather than mandatory.

Dependencies:

- Workstream 1

Risks:

- AG-UI docs include both stable and draft material; implementation scope must stay disciplined.

### Workstream 4: A2UI `v0.9` Parity Cleanup

Objective: close the practical gaps in our current A2UI support without pretending every draft item is release-critical.

Progress comment:

- Substantially landed. The shared React package now retains `createSurface` metadata, evaluates a much broader subset of the standard catalog function/check set, supports named-argument function calls, emits metadata-aware A2UI client envelopes, exposes `VALIDATION_FAILED` and generic error payload helpers, and surfaces local renderer validation plus local `functionCall` actions with test coverage. Remaining A2UI work is mainly documentation polish and any future draft churn from upstream.

Tasks:

- Audit all Rust producers and React consumers for consistent `v0.9` flat component handling.
- Expand the built-in function and check registry in the React package.
- Add explicit handling for metadata-driven flows such as `a2uiClientCapabilities`, `inlineCatalogs`, and `a2uiClientDataModel` where those flows matter to this project.
- Add support for the validation/error feedback loop expected by the newer spec.
- Verify example prompts and render tools generate catalog/component payloads that remain valid against the copied schema fixtures.

Deliverables:

- Expanded A2UI client/runtime support
- Updated schema and compatibility tests
- Clear statement of which `v0.9` features are implemented and which remain intentionally out of scope

Acceptance criteria:

- A2UI support is described as a concrete implemented subset rather than a generic `v0.9` label.
- Function/check catalog coverage is no longer obviously incomplete.
- Validation and error feedback are test-covered.

Dependencies:

- Workstream 0
- Parts of Workstream 1 if transport metadata needs shared client infrastructure

Risks:

- Because `v0.9` is still a draft, we should avoid overfitting to unstable details that do not improve real integrations.

### Workstream 5: Examples, Prompts, and Documentation

Objective: make the example apps and docs reflect the modernized protocol boundaries.

Tasks:

- Update example prompt instructions so they do not imply unsupported protocol features.
- Refresh the example React UI labels and hints to explain native vs compatibility protocol modes.
- Add example scenarios that exercise MCP Apps and AG-UI through their native paths.
- Update `docs/PROTOCOL_MIGRATION.md` and README guidance after each major protocol milestone.
- Add a small protocol support policy section to the root docs.

Deliverables:

- Updated examples
- Updated README and migration docs
- Protocol-specific example flows

Acceptance criteria:

- Example apps demonstrate the intended protocol architecture, not the deprecated one.
- Documentation matches the shipped implementation.

### Workstream 6: Verification and Release Hardening

Objective: turn protocol support into a tested contract.

Progress comment:

- Complete for the scoped modernization pass. Rust and TypeScript protocol tests are materially broader than when this document was drafted, the framework repo now includes a dedicated MCP Apps source example, and fixture-backed conformance checks cover capability metadata, MCP Apps bridge notification flows, canonical MCP Apps output, and AG-UI native SSE summaries. Additional fixture expansion is now a future maintenance task tied to new protocol surface area, not a blocker for the current release.

Tasks:

- Expand Rust unit and schema tests for each protocol family.
- Expand `packages/adk-ui-react` tests for per-protocol parsing and outbound event generation.
- Add browser-level tests for the example client in each protocol mode.
- Add regression tests that ensure legacy `adk_ui` and current A2UI flows still render.
- Add fixture versioning so upstream protocol updates can be reviewed explicitly.

Deliverables:

- Broader Rust tests
- Broader TypeScript tests
- Browser integration checks for example flows
- Release checklist for protocol compatibility

Acceptance criteria:

- CI catches protocol shape regressions before release.
- The team can point to a concrete compatibility matrix backed by tests.

## Recommended Sequencing

### Phase 1: Truthfulness

Scope:

- Workstream 0

Outcome:

- Public support claims stop getting ahead of implementation.

### Phase 2: Client Boundary Refactor

Scope:

- Workstream 1

Outcome:

- The client becomes capable of native protocol ingestion.

### Phase 3: MCP Apps First

Scope:

- Workstream 2

Outcome:

- The largest protocol gap is addressed first.

### Phase 4: AG-UI Stable Refresh

Scope:

- Workstream 3

Outcome:

- AG-UI stops depending on the custom-event compatibility tunnel.

### Phase 5: A2UI Cleanup

Scope:

- Workstream 4

Outcome:

- A2UI support becomes consistent and credibly documented.

### Phase 6: Example and Release Finish

Scope:

- Workstreams 5 and 6

Outcome:

- The repo, examples, and release process all align.

## Rough Timeline

These are rough estimates for a focused single-stream implementation. Parallel work can compress the schedule, but only after the client boundary refactor is stable.

- Phase 1: 3 to 5 days
- Phase 2: 1 to 1.5 weeks
- Phase 3: 2 to 3 weeks
- Phase 4: 1 to 2 weeks
- Phase 5: about 1 week
- Phase 6: about 1 week

Total rough effort: 6 to 9 weeks, depending on how much MCP Apps bridge work is included in the first release.

## Suggested Ownership Split

- Rust interop and capability metadata: `src/protocol_capabilities.rs`, `src/interop/*`, render tool adapters, schema tests
- React protocol-native client work: `packages/adk-ui-react/src/*`, protocol parsers, stores, outbound events, browser tests
- Example and docs track: `examples/ui_react_client`, `examples/ui_server`, `README.md`, `docs/*`

## Suggested PR Breakdown

PR 1:

- capability truthfulness pass
- README and migration doc corrections
- no protocol behavior change

PR 2:

- React protocol boundary refactor
- native parser/reducer scaffolding for AG-UI and MCP Apps
- compatibility shims retained

PR 3:

- MCP Apps modernization
- bridge-aware client integration
- example host/app flow

PR 4:

- AG-UI stable event expansion
- Rust adapter and React reducer changes

PR 5:

- A2UI `v0.9` parity cleanup
- function/check catalog expansion
- validation/error loop support

PR 6:

- docs polish
- example refresh
- expanded test coverage

## Risks and Mitigations

### Risk: Draft spec churn

Mitigation:

- Treat draft items as opt-in unless they unblock a real integration.
- Mark draft-aligned support explicitly in capability metadata and docs.

### Risk: Compatibility regressions

Mitigation:

- Preserve current compatibility paths until native paths are tested and stable.
- Maintain regression coverage for existing example prompts and legacy consumers.

### Risk: MCP Apps scope explosion

Mitigation:

- Implement the minimal bridge-aware slice first.
- Keep static resource compatibility as a bounded fallback during rollout.

### Risk: AG-UI ambiguity between stable and draft material

Mitigation:

- Separate stable event work from draft capability discovery in planning and code structure.

## Open Questions

- Do we want to keep advertising `a2ui` as `0.9`, or should we describe it as a `v0.9`-aligned subset until parity is stronger?
- Should the static HTML MCP Apps path remain public, or be renamed to a documented compatibility mode?
- How much AG-UI reasoning support is required for the first modernization release?
- Do we want one canonical internal render model for all protocols, or protocol-native stores that converge later in the render layer?

## Exit Criteria

This modernization effort should be considered complete only when all of the following are true:

- Capability metadata reflects actual support boundaries.
- The React client has native protocol ingestion paths for A2UI, AG-UI, and MCP Apps.
- MCP Apps support includes negotiated and bridge-aware behavior.
- AG-UI support covers the intended stable event families without relying on a custom-event tunnel.
- A2UI support has explicit, documented `v0.9` subset coverage with tests.
- Examples and docs match the shipped architecture.

## Sources Reviewed On 2026-03-19

- A2UI `v0.9` spec: <https://a2ui.org/specification/v0.9-a2ui/>
- A2UI `v0.9` evolution guide: <https://a2ui.org/specification/v0.9-evolution-guide/>
- AG-UI events: <https://docs.ag-ui.com/concepts/events>
- AG-UI reasoning: <https://docs.ag-ui.com/concepts/reasoning>
- AG-UI capabilities draft: <https://docs.ag-ui.com/drafts/agent-capabilities>
- MCP Apps SEP-1865: <https://modelcontextprotocol.io/seps/1865-mcp-apps-interactive-user-interfaces-for-mcp>
- MCP Apps API overview: <https://apps.extensions.modelcontextprotocol.io/api/documents/Overview.html>
- MCP Apps patterns: <https://apps.extensions.modelcontextprotocol.io/api/documents/patterns.html>
