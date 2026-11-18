# @zavora-ai/adk-ui-react

Renderers of [adk-ui](https://github.com/zavora-ai/adk-ui)'s extension catalogs (charts,
metrics, key-value lists, progress, badges, apps of pages and regions) for
`@zavora-ai/adk-surface-react`'s `DrawnView`, and the same components for a host's own data.
3D scenes, cards with depth and 3D layouts need no renderer: `DrawnView` draws them itself.

ADK-Rust 3 draws agent UI with `adk-surface`'s `genui`: every drawing is recorded, checked by
rule 5 (no approval controls, secrets or links) and shown by `DrawnView`. In a drawing, a
component of an adk-ui catalog arrives with its properties already resolved and checked on the
server, and with the home's own view of it as a fallback. These renderers draw it richly.

## Install

```bash
npm install @zavora-ai/adk-ui-react@3 @zavora-ai/adk-surface-react@3
# Optional: rich charts
npm install recharts
```

Without `recharts`, a chart in a drawing is drawn as the ADK-Rust home draws it (an SVG chart).
Studio's `Scene3D` and every 3D component of the spatial catalog are drawn by `DrawnView`
natively, with WebGL loaded only for a drawing that has one, in the home's palette, with picking
into the drawing's form, and in words where WebGL is missing.

## In a DrawnView

```tsx
import { DrawnView } from "@zavora-ai/adk-surface-react";
import { adkUiRenderers } from "@zavora-ai/adk-ui-react";
import "@zavora-ai/adk-surface-react/tokens.css";
import "@zavora-ai/adk-surface-react/style.css";
import "@zavora-ai/adk-ui-react/style.css";

<DrawnView surface={surface} onSend={send} renderers={adkUiRenderers} />;
```

A renderer gets the component's checked properties, the nodes it holds and `renderChild`; it
never gets a way to send or change a value. Forms inside an app's pages are drawn by
`DrawnView` itself, so their **Send** still asks the person before any tool runs.

| Export | Catalog |
|---|---|
| `insightRenderers` | `https://zavora.ai/a2ui/catalogs/adk-ui-insight/1.0/catalog.json` |
| `studioRenderers` | `https://zavora.ai/a2ui/catalogs/adk-ui-studio/1.0/catalog.json` |
| `spatialRenderers` | `https://zavora.ai/a2ui/catalogs/adk-ui-spatial/1.0/catalog.json` (its 2D components) |
| `adkUiRenderers` | All three |

## With your own data

The same components draw data your host owns, outside a drawing: a live metrics feed, say.

```tsx
import { Chart, Metric } from "@zavora-ai/adk-ui-react";

<Metric label="Open incidents" value={3} delta={{ value: -2, goodWhen: "down" }} />
<Chart kind="line" data={rows} x={{ key: "time", type: "time" }} series={[{ key: "p95", label: "p95 latency" }]} />
```

`Chart`, `Sparkline`, `Metric`, `KeyValueList`, `ProgressBar`, `Badge`, `AppShell`, `PageView`
and `RegionView` take the catalog's properties (see the exported types). The standalone
`Scene3D` (with `three` as an optional peer) is deprecated and goes in 4.0.

## Brand kits

`KitProvider` applies an approved adk-ui brand kit to everything inside it, `DrawnView`
included, by setting the adk-surface tokens (`--card`, `--text`, `--accent`, `--chart-1` …).
A draft or unsigned kit throws unless `allowDraft` is set, which a production host never does,
and no kit sets a decision's colours. The ADK-Rust web home keeps its own tokens.

```tsx
<KitProvider kit={approvedManifest}>
  <DrawnView surface={surface} onSend={send} renderers={adkUiRenderers} />
</KitProvider>
```

## From 2.x

2.x shipped its own renderer, protocol client and A2UI parser; 3.0 ships renderers for
`DrawnView` instead. See [MIGRATING_TO_3.md](https://github.com/zavora-ai/adk-ui/blob/main/docs/MIGRATING_TO_3.md).

## License

Apache-2.0
