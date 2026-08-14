# ADK UI Showcase Fixtures

Each file in this directory is an independent, zero-server example built with the public `@zavora-ai/adk-ui-react` API.

| Fixture | Demonstrates |
|---------|--------------|
| `incident-command.ts` | Data-dense decision support and guarded actions |
| `support-workflow.ts` | Context-aware forms, validation, and review |
| `release-room.ts` | Multi-page generation and preserved local state |
| `brand-commerce.ts` | Approved brand kits and semantic design tokens |
| `spatial-logistics.ts` | Bounded declarative 3D with labelled actions |
| `streaming-operations.ts` | Replace, patch, append, and remove updates |

Run the containing React client with `npm run dev`, then open `/?example=<fixture-name>`.

Application fixtures export a `UiApplication` plus an approved `UiKitManifest`. The streaming fixture exports an initial `Component` and ordered `UiUpdate` operations. None of the examples bypasses the renderer with handcrafted product markup.
