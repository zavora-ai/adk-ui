# ADK UI React Client

React frontend for rendering dynamic UI components from ADK agents.

## Quick Start

```bash
# Start the UI server (in adk-rust root)
GOOGLE_API_KEY=... cargo run --example ui_server

# In another terminal, start this client
cd examples/ui_react_client
npm install
npm run dev
```

Open http://localhost:5173 to interact with the agent.

## What This Does

This client connects to the ADK UI server via SSE and renders UI components that agents generate through `render_*` tool calls:

- **Forms** - User input with text fields, selects, switches, etc.
- **Cards** - Information display with action buttons
- **Alerts** - Success, warning, error, and info notifications
- **Tables** - Tabular data display
- **Charts** - Bar, line, area, and pie charts
- **Progress** - Step-by-step task progress
- **Layouts** - Dashboard-style multi-section views

## Architecture

```
┌─────────────────┐     SSE      ┌──────────────┐
│  React Client   │◄────────────│  ui_server   │
│   (Vite)        │             │  (Rust)      │
│                 │────POST────►│              │
└─────────────────┘ /api/run_sse └──────────────┘
         │                               │
         ▼                               ▼
   Renderer.tsx                   LlmAgent + UiToolset
         │
         └── UI actions use the shared tri-protocol client bridge:
             - A2UI / legacy ADK UI: event payloads
             - AG-UI: native run-input envelope plus compatibility event;
               the client also ingests stable text, tool-result, run-error,
               and message-snapshot events when present
             - MCP Apps: `ui/initialize`, `ui/notifications/initialized`,
               `ui/message`, and `ui/update-model-context`
             The example still includes a text fallback in `newMessage`
             so current ADK servers can handle follow-up actions.
             Protocol metadata is sent in the JSON body, not custom HTTP headers,
             to avoid CORS preflight failures against the demo server.
             MCP Apps response payloads prefer bridge `structuredContent`
             and negotiated host/app metadata before falling back to inline HTML.
```

## Key Files

- `src/adk-ui-renderer/types.ts` - TypeScript types matching Rust schema
- `src/adk-ui-renderer/Renderer.tsx` - Component renderer (23 components)
- `src/App.tsx` - Main app with SSE connection and outbound protocol bridge

## Customization

The renderer uses Tailwind CSS. Modify `Renderer.tsx` to customize styling or add new component types.

## Production Build

```bash
npm run build
# Output in dist/
```
