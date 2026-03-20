# Framework Continuity Roadmap

Date: 2026-03-20
Primary repo: `../adk-rust`
Companion repo: `./`

## Purpose

This roadmap keeps the `adk-ui` app work aligned with the remaining framework work in `adk-rust` so the current app-side bridge logic can be reduced safely instead of rewritten again later.

Status note:

- The scoped continuity work needed for the current `adk-ui` modernization pass is complete as of 2026-03-20.
- The remaining items in this document are future framework hardening and cleanup opportunities, not blockers for the current app integration.

## Current Baseline

What is already landed and should be treated as the active contract:

- `adk-rust/adk-server` exposes truthful UI protocol capabilities.
- `adk-rust/adk-server` supports additive MCP Apps bridge endpoints:
  - `/api/ui/initialize`
  - `/api/ui/message`
  - `/api/ui/update-model-context`
  - `/api/ui/notifications/poll`
  - `/api/ui/notifications/resources-list-changed`
  - `/api/ui/notifications/tools-list-changed`
- `adk-rust/adk-server` supports additive AG-UI protocol-native transport on `/api/run_sse`.
- `adk-rust/adk-server` accepts dual-path AG-UI input and additive MCP Apps runtime bridge envelopes.
- `adk-ui` example client now prefers:
  - AG-UI native run input plus `protocol_native` transport
  - direct MCP Apps bridge endpoints for initialize, message, model-context updates, and host-notification flows
  - framework-owned MCP Apps session state over the earlier runtime initialized shim

## App Assumptions To Preserve

The framework should keep these contracts additive until `adk-ui` removes its remaining compatibility paths:

- `/api/run_sse` default behavior stays backward compatible.
- `uiProtocol` negotiation remains supported.
- `uiTransport=protocol_native` remains opt-in.
- MCP Apps bridge endpoints remain stable even if a richer postMessage host bridge is added later.
- MCP Apps bridge responses keep returning enough host metadata for app-side diagnostics.

## Phase 1: Normalize Shared Contracts

Target: remove duplicated protocol metadata and response conventions.

Framework work:

- Move UI capability metadata to one shared source consumed by both `adk-server` and `adk-ui`.
- Define a canonical MCP Apps tool-result helper for:
  - `structuredContent`
  - `appInfo`
  - `appCapabilities`
  - `hostInfo`
  - `hostContext`
  - `initialized`
- Publish one stable AG-UI transport/input contract example in framework docs.

Progress comment:

- Complete for the current modernization scope. The framework now has the canonical MCP Apps helper path (`McpUiBridgeSnapshot`, `McpUiToolResult`, `McpUiToolResultBridge`), a runnable source example in `../adk-rust/examples/mcp_apps_tool_result/`, fixture-backed output coverage, truthful capability signaling, and additive bridge endpoint support. The only remaining follow-on item in this phase is future shared-source metadata centralization inside `adk-rust`.

App impact after completion:

- Delete local capability wording overrides where possible.
- Stop carrying repo-specific MCP Apps bridge metadata shaping in render payload handling.

## Phase 2: Push AG-UI Native Events Deeper

Target: reduce server-boundary reconstruction.

Framework work:

- Emit more stable AG-UI event families from `adk-agent` / runtime internals instead of translating generic events only at the server edge.
- Preserve tool call IDs, names, and message snapshots end to end.
- Add fixture coverage for:
  - `RUN_STARTED`
  - `STATE_SNAPSHOT`
  - `MESSAGES_SNAPSHOT`
  - `TEXT_MESSAGE_*`
  - `TOOL_CALL_*`
  - `RUN_ERROR`
  - `RUN_FINISHED`

Progress comment:

- Complete for the current modernization scope. The framework now emits native `RUN_STARTED`, `MESSAGES_SNAPSHOT`, `TEXT_MESSAGE_CHUNK`, `REASONING_MESSAGE_CHUNK`, `TOOL_CALL_CHUNK`, `ACTIVITY_SNAPSHOT`, and `ACTIVITY_DELTA` coverage for the shipped `protocol_native` AG-UI path, with fixture-backed tests in `adk-rust` and matching client-side parsing in `adk-ui`. Future work here is deeper runtime-owned semantics below the server serializer, not missing integration required by the app.

App impact after completion:

- Remove more call-side render projection heuristics from the example client.
- Prefer response-side and event-side surfaces only.

## Phase 3: Complete MCP Apps Host Lifecycle

Target: finish the framework-owned bridge so the app can stop compensating for it.

Framework work:

- Add remaining lifecycle coverage:
  - initialized notification handling as a first-class host flow
  - resource/tool list change notifications
  - documented postMessage mapping for embedded hosts
- Move bridge state from in-memory demo behavior toward runtime/session-aware persistence.
- Define notification and revision semantics clearly.

Progress comment:

- Complete for the current modernization scope. The framework now exposes first-class lifecycle helpers for initialize, message, model-context updates, resource/tool list-changed notifications, and notification polling, with revision semantics and fixture-backed bridge-flow tests. The remaining work is future persistence and full browser `postMessage` parity, not missing additive lifecycle coverage.

App impact after completion:

- Remove the remaining runtime notification shim from initial MCP Apps runs.
- Treat the framework bridge as the sole source of host/app session state.

Status in `adk-ui`:

- Done. The example client now initializes MCP Apps through the framework-owned `/api/ui/*` bridge and no longer injects its own runtime initialized shim into prompt runs.

## Phase 4: Conformance And Deprecation Gates

Target: make cleanup safe.

Framework work:

- Add conformance fixtures for protocol capabilities, AG-UI native SSE, and MCP Apps bridge flows.
- Version any breaking transport changes explicitly.
- Publish a deprecation checklist for compatibility-only paths.

Progress comment:

- Complete for the current modernization scope. Fixture-backed summaries now cover protocol capabilities, canonical MCP Apps tool-result output, MCP Apps bridge notification flows, and AG-UI native SSE summaries. The remaining follow-on here is broader release engineering polish, not missing protocol contract coverage for the implemented paths.

App cleanup gate:

Do not remove compatibility parsing in `adk-ui` until the framework has:

- stable AG-UI native fixtures
- stable MCP Apps bridge fixtures
- a canonical MCP Apps tool-result contract
- published migration guidance for downstream hosts

Status:

- Gate satisfied for the scoped modernization pass. `adk-ui` can keep compatibility parsing where useful, but it is no longer blocked on missing framework contracts for the implemented AG-UI and MCP Apps paths.

## Recommended Sequence From Here

1. Centralize UI capability metadata inside `adk-rust` so `adk-server` and downstream crates stop duplicating support text.
2. Deepen AG-UI semantics below the server serializer if the framework wants truly runtime-native stable-event production.
3. Add durable MCP Apps bridge persistence and richer browser `postMessage` parity only if a full embedded-host product path needs it.

## Handoff References

- App workplan: `docs/PROTOCOL_MODERNIZATION_WORKPLAN.md`
- Framework backlog: `../adk-rust/docs/implementation/adk-ui-framework-issue-list.md`
