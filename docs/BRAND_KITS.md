# Brand Kits

Brand kits make company design inputs the source of truth for generated interfaces. An agent chooses the right approved components and content, but it does not invent a logo, pick arbitrary colors, or restyle the product after generation.

## What A Kit Controls

One versioned `BrandKitManifest` is shared across the stack:

- **Agents** receive the allowed component catalog, templates, asset references, and design policy.
- **Protocol surfaces** carry the kit id as their catalog identity.
- **React renderers** apply semantic color, typography, spacing, radius, and motion tokens.
- **Assets** are referenced by stable `kit://<asset-id>` identifiers instead of model-generated URLs.
- **Hosts** decide which manifests are approved and may replace component rendering through typed slots.

This follows the useful separation seen in current generative-UI systems: controlled components for product-critical flows, declarative protocol surfaces for portable UI, and open-ended rendering only for explicitly sandboxed use cases. ADK UI keeps brand kits on the controlled/declarative side of that boundary.

## Draft First

`render_kit` always produces a draft. It does not publish or activate a kit.

```json
{
  "name": "Acme Operations",
  "version": "1.0.0",
  "brand": {
    "vibe": "direct, dependable, operational",
    "industry": "logistics"
  },
  "colors": {
    "primary": "#00695c",
    "accent": "#d97706",
    "background": "#f7f9f8",
    "surface": "#ffffff",
    "text": "#17211f"
  },
  "typography": {
    "family": "Source Sans 3"
  },
  "assets": [
    {
      "id": "primary-logo",
      "kind": "logo",
      "uri": "/brand/acme-logo.svg",
      "mimeType": "image/svg+xml",
      "alt": "Acme"
    }
  ],
  "provenance": {
    "source": "imported",
    "sourceUri": "https://design.acme.example/brand"
  },
  "templates": ["incident_command", "shipment_exception"]
}
```

The result contains:

- `manifest`: versioned tokens, assets, recipes, provenance, and lifecycle state
- `catalog`: every component supported by the renderer, not an empty placeholder
- `tokens`: web-ready semantic token values
- `templates`: named workflow starting points
- `agent_context`: allowed components and anti-brand-invention instructions
- `theme_css`: static, kit-scoped CSS for non-React hosts

## Existing And Generated Assets

Existing company assets should be imported first. If a required derivative does not exist, a separate generation pipeline may create it, but the generated file must enter the same review path as an imported file.

Use provenance values consistently:

- `imported`: supplied by a company design system or brand repository
- `generated`: created by a model or asset pipeline
- `hybrid`: imported source material with generated derivatives

ADK UI does not silently call an image model from `render_kit`. Generation is host-owned so companies can choose their model, storage, legal review, and approval process. Once reviewed, generated assets are listed in `assets` and addressed through `kit://` references like any other asset.

## Approval

Before changing a manifest to `approved`:

1. Verify asset ownership and licensing.
2. Sanitize SVG content; MIME validation alone does not make an SVG trusted.
3. Store assets on an approved HTTPS host or as root-relative application assets.
4. Record a SHA-256 digest for immutable production assets where possible.
5. Check alternative text, contrast, keyboard behavior, responsive layouts, and reduced-motion behavior.
6. Set `reviewedBy` and `approvedAt`, then run `BrandKitManifest::validate()`.

The React provider refuses draft and deprecated kits by default. `allowDraft` exists only for an isolated preview workflow.

## React Host

```tsx
import {
  AdkUiKitProvider,
  Renderer,
  defaultUiKit,
  resolveApprovedUiKit,
  type UiKitManifest,
  type UiResponse,
} from '@zavora-ai/adk-ui-react';

function AgentSurface({ response, kits }: {
  response: UiResponse;
  kits: UiKitManifest[];
}) {
  const kit = resolveApprovedUiKit(response.kit_id, kits, defaultUiKit);

  return (
    <AdkUiKitProvider kit={kit}>
      {response.components.map((component, index) => (
        <Renderer key={component.id ?? index} component={component} />
      ))}
    </AdkUiKitProvider>
  );
}
```

`A2uiSurfaceRenderer` consumes the same provider, so native A2UI and legacy `UiResponse` rendering use one visual contract.

### Approved Asset References

Agents use a stable asset id:

```json
{
  "type": "image",
  "src": "kit://primary-logo",
  "alt": "Acme"
}
```

The renderer resolves the id against the active manifest, then runs the resolved URI through its URL security policy. Missing or malformed kit references are not rendered.

### Component Slots

Hosts can replace a catalog component without teaching the model a new rendering implementation:

```tsx
<AdkUiKitProvider
  kit={kit}
  slots={{
    button: ({ component, renderDefault }) =>
      component.type === 'button' && component.variant === 'primary'
        ? <CompanyPrimaryButton label={component.label} />
        : renderDefault(),
  }}
>
  {children}
</AdkUiKitProvider>
```

Use slots for real design-system components. Keep the default renderer for component types the company has not replaced.

## Protocol Selection

For legacy-style tools such as `render_form`, `render_layout`, and `render_card`, pass the approved manifest id as `kit_id`:

```json
{
  "protocol": "ag_ui",
  "kit_id": "zavora.ai:adk-ui/kit/acme-operations@1.0.0"
}
```

The identifier is copied into `UiResponse.kit_id` and becomes the projected A2UI, AG-UI, or MCP Apps catalog id. `render_screen` and `render_page` already accept their catalog identity directly as `catalog_id`.

Hosts must resolve that identity against their own approved manifest registry. A server-provided draft, unknown id, or deprecated id must not silently activate new brand code or assets.

## Security Boundary

A kit is configuration, not executable UI. Keep these boundaries:

- Do not place JavaScript, arbitrary HTML, or event-handler source in token or asset fields.
- Permit only reviewed component slots compiled into the host application.
- Use CSP and an asset-host allowlist in production.
- Keep remote catalog loading disabled unless the catalog registry is configured with trusted HTTPS hosts.
- Sandbox genuinely open-ended MCP Apps or HTML output separately from the controlled kit renderer.
- Preserve the fallback approved kit when a requested manifest cannot be resolved.

