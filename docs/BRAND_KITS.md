# Brand kits

A brand kit makes a company's design inputs (colours, type, radius, the chart palette) the
source of truth for a host's own pages. In adk-ui 3 a kit is **the host's**: the agent never
chooses one, a drawing never carries one, and no kit restyles a decision. The ADK-Rust web home
keeps its own tokens; a kit styles hosts of your own that embed `DrawnView`.

| | adk-ui 2 | adk-ui 3 |
|---|---|---|
| Who picks the kit | The agent, with `render_kit` and `kit_id` | The host |
| What a kit defines | Tokens, a component catalog, templates, model instructions | Tokens (and reviewed assets for the host's own chrome) |
| Where it applies | Every protocol surface, as its catalog id | The host's pages: `KitProvider`, or the CSS of `ApprovedKit::surface_css` |
| Assets in drawings | `kit://` references in components | None: a drawing has no images |

## Draft a kit

`KitGenerator` turns brand inputs into a **draft** manifest, its semantic tokens and kit-scoped
CSS. It never activates anything.

```rust
use adk_ui::kit::{KitGenerator, KitSpec};

let spec: KitSpec = serde_json::from_value(serde_json::json!({
    "name": "Acme Operations",
    "version": "1.0.0",
    "brand": { "vibe": "direct, dependable, operational", "industry": "logistics" },
    "colors": { "primary": "#00695c" },
    "typography": { "family": "Source Sans 3" }
}))?;
let draft = KitGenerator::new().generate(&spec);
assert_eq!(draft.manifest.status, adk_ui::kit::KitStatus::Draft);
```

## Approve it

Approval is a human step:

1. Check every colour pair for contrast in light and dark.
2. Import existing company assets first; generated assets go through the same review.
3. Store assets on an approved HTTPS host, or as root-relative application assets.
4. Set `status` to `approved` and fill `reviewedBy` and `approvedAt`.
5. Run `BrandKitManifest::validate()`.

Only an approved, valid kit applies (status `approved`, with `reviewedBy` and `approvedAt`):
`ApprovedKit::try_from` refuses anything else, and
the React `KitProvider` throws for a draft unless `allowDraft` is set for an isolated preview.

## Apply it in a host

React, around the host's `DrawnView`s:

```tsx
import { DrawnView } from "@zavora-ai/adk-surface-react";
import { KitProvider, adkUiRenderers } from "@zavora-ai/adk-ui-react";

<KitProvider kit={approvedManifest}>
  <DrawnView surface={surface} onSend={send} renderers={adkUiRenderers} />
</KitProvider>;
```

Or the CSS rule, from Rust:

```rust
let kit = adk_ui::kit::ApprovedKit::try_from(manifest)?;
let css = kit.surface_css(); // [data-adk-ui-kit="…"] { --card: …; --accent: …; --chart-1: …; }
```

Both set the adk-surface tokens `DrawnView` and adk-ui's renderers read: `--bg`, `--card`,
`--raised`, `--border`, `--text`, `--text-2`, `--label`, `--accent`, `--focus`, `--font`,
`--mono`, `--radius`, `--chart-grid`, `--chart-axis` and `--chart-1` … `--chart-8`. They never set
a decision's or a status's colours (`--decision*`, `--pill-*`, `--waiting`), and a value that is
not a CSS colour is left out.

For a third-party A2UI renderer, the host's transport may add `ApprovedKit::a2ui_theme()` (or
`kitA2uiTheme`) to `createSurface` when it re-sends a drawing: the primary colour only, never an
icon or a display name, which would let a drawing pass for someone else, and never part of the
stored drawing.

## Security boundary

A kit is configuration, not UI:

- No JavaScript, HTML or event handlers in token or asset fields; colours are checked before
  they reach CSS.
- Kit assets are for the host's own chrome (its header logo, say), never referenced by a drawing.
- The model never sees a kit and cannot ask for one: what a drawing may contain comes from
  adk-ui's catalogs alone, and every drawing is checked by `genui`'s rule 5.
