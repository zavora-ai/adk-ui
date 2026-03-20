# adk-ui Protocol Migration Guide

For the March 2026 modernization sequencing and implementation backlog, see `docs/PROTOCOL_MODERNIZATION_WORKPLAN.md`.

This guide maps legacy `adk_ui` tool outputs to protocol-aware outputs for `a2ui`, `ag_ui`, and `mcp_apps`.

## Why Migrate

Protocol-aware outputs let the same agent/toolset serve:

- A2UI clients
- AG-UI consumers
- MCP Apps hosts

without maintaining separate tool implementations.

## Legacy vs Protocol-Aware Outputs

### Legacy default (`protocol` omitted)

- `render_screen`: returns A2UI payload object
- `render_page`: returns A2UI JSONL string
- `render_kit`: returns kit artifact JSON
- other render tools: return legacy `UiResponse` JSON

### Protocol-aware (`protocol` set)

- `protocol="a2ui"`: A2UI payloads or protocol envelopes
- `protocol="ag_ui"`: AG-UI event payloads
- `protocol="mcp_apps"`: MCP Apps resource payloads with `ui://` URIs

## Tool Mapping

### `render_screen` / `render_page`

No behavior change is required for existing A2UI consumers.

- Keep `protocol` omitted for existing behavior.
- Set `protocol` explicitly when targeting AG-UI or MCP Apps.

### `render_kit`

When `protocol` is set, output is wrapped:

```json
{
  "protocol": "a2ui",
  "surface_id": "kit",
  "payload": { "...": "kit artifacts" }
}
```

### Legacy-style render tools

`render_form`, `render_card`, `render_alert`, `render_confirm`, `render_table`, `render_chart`, `render_layout`, `render_progress`, `render_modal`, and `render_toast` keep legacy `UiResponse` behavior by default.

When `protocol` is set, these tools emit protocol envelopes:

```json
{
  "protocol": "ag_ui",
  "version": "1.0",
  "surface_id": "form",
  "payload": {
    "events": [ "... AG-UI events ..." ]
  }
}
```

## Recommended Rollout

1. Keep existing calls unchanged and verify no regressions.
2. Add explicit `protocol` only at integration boundaries that need AG-UI or MCP Apps.
3. For MCP Apps, provide optional validated metadata under `mcp_apps`.
4. Pin tests to protocol contracts using `adk-ui/tests/tool_protocol_matrix_tests.rs`.

## Deprecation Notes

- Legacy default outputs are still supported.
- No forced cutover is required for existing `UiResponse` consumers.
- New integrations should prefer explicit `protocol` selection to avoid ambiguous defaults.

### Timeline (Current Plan)

- Announced: `2026-02-07`
- Legacy runtime profile: `adk_ui`
- Sunset target for new integrations: `2026-12-31`
- Preferred profiles: `a2ui`, `ag_ui`, `mcp_apps`

### Runtime Notices

- `adk-server` emits warning logs when requests explicitly or implicitly use `adk_ui`.
- `/api/ui/capabilities` now includes deprecation metadata for `adk_ui` so clients can surface migration prompts.
