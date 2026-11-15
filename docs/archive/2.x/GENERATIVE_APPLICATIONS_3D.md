# Generative Applications and 3D

ADK UI supports three output scales:

1. `UiResponse` for a component tree.
2. `UiSurface` for a protocol-addressable screen.
3. `UiApplication` for a complete product with routes, page composition, shared state, and optional spatial interfaces.

The application graph is additive. Existing consumers do not need to migrate until they want multi-page behavior.

## When to use `render_app`

Use `render_app` for a product, portal, website, command center, end-to-end workflow, or any request that needs more than one meaningful route. Continue using focused tools such as `render_form`, `render_confirm`, and `render_layout` for single decisions and bounded tasks.

An application contains:

- An approved `kit_id` that drives the complete visual system.
- A unique initial route and optional navigation model.
- Up to 24 pages, each with a route, page template, atmosphere, hero, regions, aside, and footer.
- Up to 16 regions per page using stack, grid, bento, split, editorial, or stats composition.
- Shared serializable state for agent and host coordination.

The renderer includes marketing, dashboard, workspace, detail, form, and immersive page templates. These are art-direction constraints rather than rigid page templates: the agent still provides domain content and components, while the host guarantees hierarchy, spacing, responsiveness, and brand consistency.

## Protocol behavior

`render_app` includes the canonical application graph in its response. It also emits every page as an independently addressable surface:

- A2UI receives page JSONL and the application graph.
- AG-UI receives page lifecycle/custom-surface events and the application graph.
- MCP Apps receives page resources/structured content and the application graph.
- AWP, when enabled, receives page HTML fallbacks and the application graph.

This progressive shape allows application-aware clients to render navigation while older clients can still display the initial page surface.

## React integration

```tsx
import {
  ApplicationRenderer,
  parseUiApplication,
} from '@zavora-ai/adk-ui-react';

const application = parseUiApplication(protocolToolResponse);

if (application) {
  return (
    <ApplicationRenderer
      application={application}
      kits={approvedKits}
      onAction={sendEventToAgent}
      onNavigate={(route, page) => recordNavigation(route, page.id)}
    />
  );
}
```

Use `navigate:/route` as a button action for host-owned navigation. Other actions remain agent events. Pages are mounted when first visited and then hidden rather than unmounted, preserving uncontrolled input values and local renderer state across navigation.

Application updates support `navigate`, `replace`, `upsert_page`, `remove_page`, and `patch_state`. Hosts should validate external updates before applying them.

## Declarative 3D scenes

`scene_3d` is a component in both the typed ADK UI schema and the extended A2UI catalog. A scene can include:

- Box, sphere, cylinder, torus, and plane primitives.
- Up to 64 total scene objects.
- Up to 8 approved GLB models.
- Semantic material roles such as primary, secondary, surface, success, warning, and info.
- A bounded camera, optional orbit controls, labelled actions, and accessible fallback text.

Agents cannot provide JavaScript, JSX, shader source, arbitrary material graphs, or raw model URLs. Models reference an asset ID from the active approved brand kit. The asset must use `model_3d`, `model/gltf-binary`, a safe URI, and a maximum declared size of 25 MiB.

## Runtime and performance

The React renderer isolates 3D in a lazy module. Three.js is a declared runtime dependency and remains external in the published library build, preventing duplication in CommonJS and ESM consumers. Applications that never render `scene_3d` do not evaluate the scene adapter.

At runtime the scene renderer:

- Uses WebGPU when available and falls back to WebGL.
- Caps pixel ratio at 2.
- Resizes through `ResizeObserver`.
- Pauses rendering while outside the viewport through `IntersectionObserver`.
- Disables automatic motion when the user requests reduced motion.
- Disposes geometries, textures, materials, controls, and renderers on unmount.
- Provides labelled HTML actions so interactions do not depend only on pointer picking.

Hosts should still set product-specific model, texture, memory, and battery budgets. A high-end 3D scene is not a replacement for a fast text, table, or chart interface when spatial context adds no user value.

## Design quality

Brand kits and page composition solve different problems. The kit defines approved identity, tokens, assets, and component recipes. The application graph defines page-level hierarchy, rhythm, route intent, and regional composition. Both are required for generated products to feel consistent with their host application.

Production agents should be instructed to:

- Choose a page template based on user intent, not component count.
- Give every route one clear job.
- Use realistic populated content instead of placeholder dashboards.
- Prefer two to five purposeful pages over many shallow routes.
- Use 3D only for spatial reasoning, product exploration, simulation, or topology where it improves a decision.
- Keep risky operations editable and explicitly confirmed.
