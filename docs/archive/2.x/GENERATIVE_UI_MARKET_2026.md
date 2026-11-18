# Generative UI Market Brief (2026)

Reviewed: 2026-08-14

This brief defines the product standard used by the adk-ui demo prompts and agent instructions. It focuses on behavior users can evaluate, not protocol checklists.

## Market Direction

The leading 2026 implementations converge on six expectations:

1. **Outcome-first generation.** The agent builds a decision surface for a real task instead of translating a list of component names into a sparse mockup.
2. **Shared, editable state.** User edits, agent updates, filters, and intermediate decisions persist across turns.
3. **Progressive interaction.** Lightweight decisions stay inline; exploration and multi-step work use a larger composed workspace.
4. **Human control.** Consequential actions expose scope and evidence, allow edits, and require explicit approval.
5. **Production runtime behavior.** Streaming, tool errors, approvals, durability, sandboxing, capability negotiation, and observability are part of the UX.
6. **Native design-system rendering.** Agents generate trusted declarative components from the host's catalog rather than arbitrary front-end code.

## Protocol Implications

| Ecosystem | 2026 signal | adk-ui product requirement |
| --- | --- | --- |
| A2UI | Declarative, trusted catalogs; data-model binding; native rendering; client and agent validation | Generate against an explicit catalog, preserve bound state, validate both ends, and support safe fallback components |
| AG-UI | Shared state, lifecycle and tool streaming, frontend/backend tools, interrupts, steering, human approval | Render live workspaces that expose progress and evidence while keeping risky actions editable and approval-gated |
| MCP Apps | Interactive `ui://` resources inside conversation with a bidirectional host bridge and sandboxed execution | Preserve host context, use compact cards for single decisions, and use richer embedded apps for filtering, drill-down, and editing |
| OpenAI Apps | Accessible host-native components, predictable inline/fullscreen modes, persisted edits, at most two primary card actions | Match the host design system, keep inline surfaces focused, and reserve multi-step exploration for a larger display mode |
| Vercel AI SDK | Typed UI messages, streaming tool inputs, explicit tool errors, approvals, MCP, durable/sandboxed agents, telemetry | Treat structured UI state, errors, approvals, and runtime instrumentation as first-class production features |

## Demo Prompt Standard

Every featured prompt should contain:

- a recognizable user or operator with a concrete goal
- known context the surface must retain
- a decision, exception, or workflow state that makes UI useful
- realistic data requirements or clearly identified assumptions
- an interaction beyond passive display, such as edit, filter, compare, review, or approve
- a safety boundary for destructive, financial, privileged, or externally visible actions

Avoid prompts such as "create three KPI cards and a chart." They reward component counting and produce interchangeable demos. Prefer prompts such as "investigate a checkout latency spike, compare recent deploys and affected regions, then prepare an editable rollback proposal for approval."

## Surface Acceptance Criteria

### Focused Forms

- Carry supplied facts into the title or description.
- Collect only missing information.
- Use specific field labels, useful options, sensible order, and evidence/detail input where relevant.
- Explain what happens next.
- Use a review-oriented CTA instead of `Submit`.
- Render review and success on later interaction turns, never in advance.

### Decision Workspaces

- Show current status and meaningful KPIs, including deltas or risk where useful.
- Populate a chart or comparison table with credible domain data.
- Surface exceptions, evidence, ownership, and unresolved decisions.
- Provide one clear next action and no decorative controls.

### Embedded Apps

- Preserve filters, edits, and model context across conversation turns.
- Do not duplicate controls already owned by the host.
- Keep inline cards single-purpose with no more than one primary and one secondary action.
- Use approval gates for consequential tool calls.

## Sources

- A2UI overview: <https://a2ui.org/introduction/what-is-a2ui/>
- A2UI catalogs: <https://a2ui.org/catalogs/>
- AG-UI overview and capabilities: <https://docs.ag-ui.com/>, <https://docs.ag-ui.com/concepts/capabilities>
- AG-UI tools and human-in-the-loop guidance: <https://docs.ag-ui.com/concepts/tools>
- MCP Apps announcement and examples: <https://blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/>
- MCP Apps architecture: <https://modelcontextprotocol.io/extensions/apps/overview>
- OpenAI app UI guidelines: <https://developers.openai.com/plugins/concepts/ui-guidelines>
- Vercel AI SDK 7: <https://vercel.com/blog/ai-sdk-7>
- Vercel Ship 2026 recap: <https://vercel.com/blog/vercel-ship-2026-recap>

