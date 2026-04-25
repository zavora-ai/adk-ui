# Requirements Document

## Introduction

This feature aligns the `adk-ui` crate with the Agentic Web Protocol (AWP) specification. AWP defines how websites serve both human and agent visitors; Section 32 covers the ADK-UI rendering layer. The integration adds AWP awareness to adk-ui behind an optional `awp` feature flag, enabling HTML rendering, capability manifest exposure, bandwidth-adaptive output, and AWP envelope bridging — while preserving full backward compatibility for users who do not need AWP.

The architectural boundary is: `adk-awp` decides WHAT to render (HTML vs JSON based on requester type), `adk-ui` decides HOW to render it (component tree → target format). This feature makes adk-ui's output consumable by both paths.

## Glossary

- **ADK_UI**: The `adk-ui` Rust crate providing 13 render tools and 30 component types for AI agent UI generation.
- **AWP**: Agentic Web Protocol — the specification defining how websites serve human and agent visitors.
- **AWP_Types**: The `awp-types` crate providing shared protocol types (RequesterType, TrustLevel, CapabilityManifest, CapabilityEntry, AwpRequest, AwpResponse).
- **CapabilityEntry**: A struct from AWP_Types describing a single capability (tool) with typed input/output schemas, used in capability manifests.
- **CapabilityManifest**: A struct from AWP_Types listing all capabilities a site exposes to agent visitors.
- **Component_Tree**: The in-memory representation of UI components. In `schema.rs`, this is the typed `Component` enum (30 variants). In `UiSurface`, components are stored as `Vec<Value>` (raw JSON) which may be in either the flat `schema::Component` format or the nested A2UI format (with `component` object wrappers).
- **Feature_Flag**: A Cargo feature gate (`awp`) that conditionally compiles AWP-related code.
- **HTML_Renderer**: A module that converts typed `Component` values into clean, embeddable HTML markup. Accepts both `Vec<Component>` directly and `Vec<Value>` (with best-effort deserialization).
- **BandwidthMode**: An enum representing connection quality levels (Full, Low) that controls adaptive rendering behavior.
- **Render_Tool**: One of the 13 high-level tools (render_form, render_card, render_table, etc.) that produce `UiResponse` values containing typed `Vec<Component>`.
- **ToolEnvelope**: The canonical output wrapper for render tool responses, carrying protocol metadata and payload.
- **UiSurface**: The protocol-neutral UI surface representation. Components are stored as `Vec<Value>` (raw JSON), not as typed `Component` enums. Created from `UiResponse` via `project_ui_response_to_surface` in `protocol_output.rs`.
- **UiProtocol**: An enum of supported interoperability protocols (`A2ui`, `AgUi`, `McpApps`). The legacy `adk_ui` protocol is handled separately in `protocol_output.rs`, not through the adapter pattern.
- **UiProtocolAdapter**: A trait for converting `UiSurface` instances into protocol-specific payloads. Implemented by `A2uiAdapter`, `AgUiAdapter`, `McpAppsAdapter`.
- **UiToolset**: Manages which of the 13 render tools are enabled. `all_tools()` is a static method returning all tools. The `Toolset::tools()` trait method uses per-tool include flags and requires a `ReadonlyContext`.

## Requirements

### Requirement 1: AWP Feature Flag and Optional Dependency

**User Story:** As a library consumer, I want AWP integration to be behind an optional feature flag, so that I can use adk-ui without pulling in awp-types when I do not need AWP support.

#### Acceptance Criteria

1. THE ADK_UI crate SHALL declare `awp-types` as an optional dependency gated behind an `awp` Cargo feature flag.
2. WHEN the `awp` feature flag is not enabled, THE ADK_UI crate SHALL compile and pass all existing tests without any AWP_Types dependency in the dependency tree.
3. WHEN the `awp` feature flag is enabled, THE ADK_UI crate SHALL re-export relevant AWP_Types types through a dedicated `awp` compatibility module.
4. THE ADK_UI crate SHALL use conditional compilation (`#[cfg(feature = "awp")]`) to isolate all AWP-specific code paths.

### Requirement 2: HTML Render Pass

**User Story:** As an AWP gateway developer, I want to convert a UiSurface into clean, embeddable HTML, so that human visitors receive rendered pages and agents can request embeddable HTML components.

#### Acceptance Criteria

1. THE HTML_Renderer SHALL accept a `UiResponse` (typed `Vec<Component>`) and produce a valid HTML string.
2. THE HTML_Renderer SHALL also accept a `UiSurface` (containing `Vec<Value>`) and attempt best-effort deserialization of each `Value` into a `Component` before rendering. Values that cannot be deserialized SHALL be rendered as an HTML comment `<!-- unknown component -->`.
3. WHEN a UiSurface contains Text components, THE HTML_Renderer SHALL render each Text component as an appropriate HTML element with its content.
3. WHEN a UiSurface contains Button components, THE HTML_Renderer SHALL render each Button as an HTML `<button>` element with its label.
4. WHEN a UiSurface contains layout components (Stack, Grid, Container), THE HTML_Renderer SHALL render them as structural HTML elements (div with appropriate CSS classes).
5. WHEN a UiSurface contains Card components, THE HTML_Renderer SHALL render each Card as a styled container with its title and child content.
6. WHEN a UiSurface contains Table components, THE HTML_Renderer SHALL render each Table as an HTML `<table>` element with headers and rows.
7. WHEN a UiSurface contains form input components (TextInput, NumberInput, Select, Switch, DateInput, Slider, Textarea), THE HTML_Renderer SHALL render each as its corresponding HTML form element.
8. WHEN a UiSurface contains Alert components, THE HTML_Renderer SHALL render each Alert as a styled container with its severity level and message.
9. WHEN a UiSurface contains Chart components, THE HTML_Renderer SHALL render a placeholder container with the chart data embedded as a JSON data attribute.
10. WHEN a UiSurface contains Progress components, THE HTML_Renderer SHALL render each as an HTML `<progress>` element with its current value.
11. THE HTML_Renderer SHALL produce HTML that is embeddable within a parent document without requiring external stylesheets or scripts.
12. THE HTML_Renderer SHALL escape all user-provided text content to prevent HTML injection.
13. THE HTML_Renderer SHALL be available regardless of the `awp` feature flag, as it serves general HTML rendering needs beyond AWP.
14. THE HTML_Renderer SHALL accept an optional CSS class prefix parameter, so that embedders can namespace generated class names to avoid style collisions with the parent document.

### Requirement 3: AWP Protocol Adapter

**User Story:** As a gateway developer, I want an AWP-specific protocol adapter, so that UiSurface payloads can be converted to AWP-compatible format alongside existing A2UI, AG-UI, and MCP Apps adapters.

#### Acceptance Criteria

1. WHEN the `awp` feature flag is enabled, THE ADK_UI crate SHALL provide an AwpAdapter that implements the UiProtocolAdapter trait.
2. WHEN the AwpAdapter converts a UiSurface, THE AwpAdapter SHALL produce a JSON payload containing the protocol identifier "awp", the surface_id, and the component tree.
3. WHEN the AwpAdapter has HTML rendering enabled (the default), THE payload SHALL also include an "html" field with the rendered HTML string.
4. THE AwpAdapter SHALL provide an `include_html` configuration flag (defaulting to true) that controls whether the HTML rendering is included in the payload, allowing gateways to avoid payload doubling when only the component tree is needed.
5. THE UiProtocol enum SHALL include an `Awp` variant when the `awp` feature flag is enabled.
6. WHEN the `awp` feature flag is not enabled, THE existing UiProtocol enum and adapters SHALL remain unchanged.

### Requirement 4: Render Tool Capability Exposure

**User Story:** As an AWP manifest builder, I want each render tool to describe itself as a CapabilityEntry, so that adk-awp can automatically include adk-ui tools in the site's capability manifest.

#### Acceptance Criteria

1. WHEN the `awp` feature flag is enabled, THE ADK_UI crate SHALL provide a trait or function that converts each Render_Tool into a CapabilityEntry from AWP_Types.
2. WHEN a Render_Tool is converted to a CapabilityEntry, THE CapabilityEntry SHALL include the tool name, a human-readable description, and the tool's input JSON schema.
3. WHEN the UiToolset generates its tool list, THE UiToolset SHALL provide a method to export all enabled tools as a Vec of CapabilityEntry values.
4. WHEN a Render_Tool is disabled in the UiToolset configuration, THE capability export method SHALL exclude that tool from the resulting CapabilityEntry list.

### Requirement 5: Bandwidth-Adaptive Rendering

**User Story:** As an AWP gateway serving constrained connections, I want adk-ui to strip non-essential components when low bandwidth is signaled, so that pages load acceptably on 2G/3G connections per AWP Section 41.4.

#### Acceptance Criteria

1. THE ADK_UI crate SHALL define a `#[non_exhaustive]` BandwidthMode enum with at least two variants: Full and Low.
2. WHEN BandwidthMode is Full, THE HTML_Renderer SHALL include all components from the UiSurface without modification.
3. WHEN BandwidthMode is Low, THE HTML_Renderer SHALL omit Chart components from the rendered output.
4. WHEN BandwidthMode is Low, THE HTML_Renderer SHALL omit Image components from the rendered output.
5. WHEN BandwidthMode is Low, THE HTML_Renderer SHALL omit Skeleton and Spinner components from the rendered output.
6. WHEN BandwidthMode is Low, THE HTML_Renderer SHALL omit inline styles and render minimal structural HTML.
7. THE HTML_Renderer SHALL accept BandwidthMode as an optional parameter, defaulting to Full when not specified.

### Requirement 6: ToolEnvelope AWP Bridge

**User Story:** As a gateway developer, I want ToolEnvelope to carry AWP envelope fields when the awp feature is active, so that render tool output can be directly used in AWP responses without manual conversion.

#### Acceptance Criteria

1. WHEN the `awp` feature flag is enabled, THE ToolEnvelope SHALL include optional `awp_version` and `request_id` fields.
2. WHEN the `awp` feature flag is enabled, THE ToolEnvelope SHALL provide a conversion method to produce an AwpResponse value from AWP_Types.
3. WHEN the `awp` feature flag is not enabled, THE ToolEnvelope SHALL retain its current structure with no additional fields.
4. THE ToolEnvelopeProtocol enum SHALL include an `Awp` variant when the `awp` feature flag is enabled.

### Requirement 7: AWP Protocol Capability Constant

**User Story:** As a gateway or server developer, I want adk-ui to expose an AWP capability spec constant, so that downstream crates (adk-server, adk-gateway) can conditionally include it in their runtime protocol arrays.

#### Acceptance Criteria

1. WHEN the `awp` feature flag is enabled, THE ADK_UI crate SHALL expose a public `AWP_PROTOCOL_CAPABILITY` constant of type `UiProtocolCapabilitySpec` describing the AWP protocol with version, implementation tier, spec track, summary, features, and limitations.
2. WHEN the `awp` feature flag is enabled, THE normalize_runtime_ui_protocol function SHALL recognize "awp" as a valid protocol alias.
3. WHEN the `awp` feature flag is not enabled, THE AWP_PROTOCOL_CAPABILITY constant SHALL not be compiled, and normalize_runtime_ui_protocol SHALL remain unchanged.
4. THE ADK_UI crate SHALL NOT modify the SUPPORTED_UI_PROTOCOLS or UI_PROTOCOL_CAPABILITIES arrays — downstream crates are responsible for including the AWP entry in their runtime arrays.

### Requirement 8: Backward Compatibility

**User Story:** As an existing adk-ui consumer, I want all current functionality to remain unchanged after the AWP integration, so that my existing integrations continue working without modification.

#### Acceptance Criteria

1. THE ADK_UI crate SHALL pass all 93 existing Rust tests when compiled without the `awp` feature flag.
2. THE ADK_UI crate SHALL pass all 93 existing Rust tests when compiled with the `awp` feature flag.
3. THE existing A2uiAdapter, AgUiAdapter, and McpAppsAdapter SHALL produce identical output before and after the AWP integration.
4. THE existing 13 Render_Tools SHALL produce identical ToolEnvelope output when the `awp` feature flag is not enabled.
5. THE 34 existing Node.js tests SHALL continue passing without modification.
