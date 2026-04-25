# Implementation Plan: AWP Protocol Alignment

## Overview

Integrate the Agentic Web Protocol (AWP) into the `adk-ui` crate behind an optional `awp` Cargo feature flag. Implementation proceeds in dependency order: Cargo.toml changes first, then the HTML renderer (largest piece, no feature gate), AWP compat re-exports, protocol adapter, envelope bridge, capability export, protocol metadata, and finally wiring up exports. Property-based tests and backward compatibility verification close out the plan.

## Tasks

- [x] 1. Add `awp-types` dependency and `awp` feature flag to Cargo.toml
  - Add `awp-types` as an optional dependency via git: `awp-types = { git = "https://github.com/zavora-ai/adk-rust", optional = true }`
  - Add `awp` feature in `[features]` that enables `dep:awp-types`
  - Add `proptest` to `[dev-dependencies]` for property-based testing
  - Verify the crate compiles with and without the `awp` feature
  - _Requirements: 1.1, 1.2, 1.4_

- [-] 2. Implement BandwidthMode enum and HTML renderer (`src/html.rs`)
  - [x] 2.1 Create `src/html.rs` with `BandwidthMode` enum, `HtmlRenderOptions` struct, and `escape_html` helper
    - Define `#[non_exhaustive]` `BandwidthMode` with `Full` (default) and `Low` variants, deriving `Serialize`, `Deserialize`, `Default`
    - Define `HtmlRenderOptions` struct with `bandwidth_mode: BandwidthMode` and `class_prefix: Option<String>`
    - Implement `escape_html` to escape `<`, `>`, `&`, `"`, `'` characters
    - _Requirements: 5.1, 5.7, 2.12, 2.14_

  - [x] 2.2 Implement `render_component_html` for all 30+ Component variants
    - Map each `schema::Component` variant to its HTML element per the design's HTML Component Mapping table
    - Text variants → `<p>`, `<h1>`–`<h4>`, `<small>`, `<code>`; Button → `<button>`; form inputs → `<input>`, `<select>`, `<textarea>`; layout → `<div>` with CSS classes; Table → `<table>`; Alert → `<div role="alert">`; Progress → `<progress>`; Chart → `<div data-chart>`; etc.
    - In `BandwidthMode::Low`, omit Chart, Image, Skeleton, Spinner components (return empty string)
    - In `BandwidthMode::Low`, strip all `style="..."` attributes from output
    - When `class_prefix` is set in options, prefix all generated CSS class names
    - Escape all user-provided text content via `escape_html`
    - Handle recursive component trees (Card.content, Stack.children, Grid.children, Modal.content, Tabs.tabs[].content)
    - _Requirements: 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9, 2.10, 2.12, 2.14, 5.2, 5.3, 5.4, 5.5, 5.6_

  - [x] 2.3 Implement `render_components_html` and `render_surface_html` public entry points
    - `render_components_html(&[Component], &HtmlRenderOptions) -> String` — typed path for `UiResponse.components`
    - `render_surface_html(&UiSurface, &HtmlRenderOptions) -> String` — deserializes each `Value` from `surface.components` into `schema::Component` via `serde_json::from_value`, renders successes, emits `<!-- unknown component -->` for failures
    - Note: `UiSurface.components` is `Vec<Value>`, not `Vec<Component>` — A2UI-format components with nested `component` objects will fail deserialization and render as unknown
    - Wrap output in a minimal self-contained HTML shell (inline styles only, no external CSS/JS)
    - Produce embeddable HTML without `<link rel="stylesheet">`, external `<script src=...>`, or `@import` rules
    - _Requirements: 2.1, 2.2, 2.11, 2.13, 2.14_

  - [x] 2.4 Register `html` module in `src/lib.rs`
    - Add `pub mod html;` to `src/lib.rs`
    - Re-export `html::BandwidthMode` and `html::render_surface_html`
    - _Requirements: 2.13_

  - [ ] 2.5 Write property test: Component-to-HTML mapping correctness (Property 1)
    - **Property 1: Component-to-HTML mapping correctness**
    - Generate arbitrary `Component` variants, render with `render_component_html`, assert correct HTML element type and content presence
    - **Validates: Requirements 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9, 2.10**

  - [ ] 2.6 Write property test: HTML injection prevention (Property 2)
    - **Property 2: HTML injection prevention**
    - Generate strings with HTML special characters, embed in Text component, render, assert escaped output
    - **Validates: Requirements 2.12**

  - [ ] 2.7 Write property test: Embeddable HTML output (Property 3)
    - **Property 3: Embeddable HTML output**
    - Generate arbitrary `UiSurface`, render via `render_surface_html`, assert no external resource references
    - **Validates: Requirements 2.11**

  - [ ] 2.8 Write property test: Full bandwidth preserves all components (Property 7)
    - **Property 7: Full bandwidth preserves all components**
    - Generate arbitrary `UiSurface` with N components, render with `BandwidthMode::Full`, assert every component has HTML representation
    - **Validates: Requirements 5.2**

  - [ ] 2.9 Write property test: Low bandwidth omits sensitive components (Property 8)
    - **Property 8: Low bandwidth omits bandwidth-sensitive components**
    - Generate surfaces with Chart, Image, Skeleton, Spinner components, render with `BandwidthMode::Low`, assert those absent
    - **Validates: Requirements 5.3, 5.4, 5.5**

  - [ ] 2.10 Write property test: Low bandwidth strips inline styles (Property 9)
    - **Property 9: Low bandwidth strips inline styles**
    - Generate arbitrary `UiSurface`, render with `BandwidthMode::Low`, assert no `style="` attributes
    - **Validates: Requirements 5.6**

- [x] 3. Checkpoint — Ensure HTML renderer compiles and tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Implement AWP compat module (`src/awp_compat.rs`)
  - Create `src/awp_compat.rs` with `#[cfg(feature = "awp")]` re-exports of `CapabilityEntry`, `CapabilityManifest`, `AwpResponse`, `AwpVersion`, `CURRENT_VERSION` from `awp-types`
  - Register module in `src/lib.rs` with conditional `#[cfg(feature = "awp")] pub mod awp_compat;`
  - _Requirements: 1.3, 1.4_

- [ ] 5. Implement `UiProtocol::Awp` variant and `AwpAdapter`
  - [x] 5.1 Add conditional `Awp` variant to `UiProtocol` enum in `src/interop/surface.rs`
    - Add `#[cfg(feature = "awp")] Awp` variant to the `UiProtocol` enum
    - _Requirements: 3.3, 3.4_

  - [x] 5.2 Create `src/interop/awp.rs` with `AwpAdapter` implementing `UiProtocolAdapter`
    - Implement `AwpAdapter` struct with `bandwidth_mode: BandwidthMode`, `include_html: bool` (default true), and `class_prefix: Option<String>` fields
    - Implement `UiProtocolAdapter` trait: `protocol()` returns `UiProtocol::Awp`, `to_protocol_payload()` produces JSON with `"protocol": "awp"`, `surface_id`, `components`, and optionally `html` field (when `include_html` is true)
    - Use `render_surface_html` from `src/html.rs` with `HtmlRenderOptions` for the HTML field
    - Gate entire file behind `#[cfg(feature = "awp")]`
    - _Requirements: 3.1, 3.2, 3.3, 3.4_

  - [x] 5.3 Register AWP adapter in `src/interop/mod.rs`
    - Add conditional `#[cfg(feature = "awp")] pub mod awp;` and re-export `AwpAdapter`
    - _Requirements: 3.1_

  - [x] 5.4 Add AWP branch to `render_ui_response_with_protocol` in `src/tools/protocol_output.rs`
    - Add `#[cfg(feature = "awp")] UiProtocol::Awp => { ... }` match arm using `AwpAdapter`
    - _Requirements: 3.2_

  - [ ] 5.5 Write property test: AWP adapter payload completeness (Property 4)
    - **Property 4: AWP adapter payload completeness**
    - Generate arbitrary `UiSurface`, convert via `AwpAdapter::to_protocol_payload`, assert payload contains `"protocol": "awp"`, `surface_id`, `components` array, and non-empty `html` string
    - **Validates: Requirements 3.2**

- [ ] 6. Implement ToolEnvelope AWP bridge (`src/model/envelope.rs`)
  - Add conditional `#[cfg(feature = "awp")] Awp` variant to `ToolEnvelopeProtocol` enum
  - Add conditional `awp_version: Option<String>` and `request_id: Option<String>` fields to `ToolEnvelope` with `#[serde(skip_serializing_if = "Option::is_none")]`
  - Implement conditional `to_awp_response(&self) -> Result<AwpResponse, AdkError>` method on `ToolEnvelope`
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

  - [ ] 6.1 Write property test: ToolEnvelope AWP response conversion (Property 10)
    - **Property 10: ToolEnvelope AWP response conversion**
    - Generate `ToolEnvelope` with random serializable payloads and `awp_version` set, call `to_awp_response()`, assert `Ok`
    - **Validates: Requirements 6.2**

- [ ] 7. Implement capability export on `UiToolset` (`src/toolset.rs`)
  - Add `#[cfg(feature = "awp")] pub fn to_capability_entries(&self) -> Vec<awp_types::CapabilityEntry>` method
  - Use the per-tool include flags (include_screen, include_form, etc.) to determine which tools to export — does NOT require a `ReadonlyContext` or async context
  - For each enabled tool, instantiate it (e.g. `RenderFormTool::new()`), call `name()`, `description()`, `parameters_schema()` to populate the `CapabilityEntry`
  - Exclude disabled tools from the output
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

  - [ ] 7.1 Write property test: Capability entry completeness (Property 5)
    - **Property 5: Capability entry completeness**
    - Iterate all 13 tools, convert each to `CapabilityEntry`, assert non-empty `name`, `description`, and `Some` `input_schema`
    - **Validates: Requirements 4.2**

  - [ ] 7.2 Write property test: Disabled tool exclusion (Property 6)
    - **Property 6: Disabled tool exclusion from capability export**
    - Generate random boolean vectors for tool enable/disable, create `UiToolset`, assert entry count matches enabled count and no disabled tool names appear
    - **Validates: Requirements 4.4**

- [x] 8. Implement AWP protocol capability constant (`src/protocol_capabilities.rs`)
  - Add conditional `#[cfg(feature = "awp")] pub const AWP_PROTOCOL_CAPABILITY: UiProtocolCapabilitySpec` with version `"1.0"`, `CompatibilitySubset` tier, `Draft` spec track, and features/limitations from the design
  - Do NOT modify `SUPPORTED_UI_PROTOCOLS` or `UI_PROTOCOL_CAPABILITIES` arrays — downstream crates (adk-server, adk-gateway) consume the constant and include it in their own arrays
  - Add conditional `"awp" | "AWP" => Some("awp")` arm to `normalize_runtime_ui_protocol`
  - Existing tests (`capability_specs_cover_supported_protocols`, etc.) must continue passing unchanged since the arrays are not modified
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

- [x] 9. Wire up exports in `src/lib.rs`
  - Ensure `pub mod html` is registered (done in task 2.4)
  - Ensure conditional `pub mod awp_compat` is registered (done in task 4)
  - Add conditional re-exports for `AwpAdapter`, AWP protocol capability types
  - Verify all public API items are accessible from the crate root
  - _Requirements: 1.3, 1.4_

- [ ] 10. Checkpoint — Ensure all tests pass with `awp` feature enabled
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 11. Backward compatibility unit tests (cross-cutting)
  - [ ] 11.1 Write unit test: Existing adapters produce expected output shapes with `awp` feature enabled
    - **Validates Property 11 (backward compatibility)** — verified as unit test within `awp`-enabled build, not as property test (cross-feature comparison requires CI matrix)
    - Run A2uiAdapter, AgUiAdapter, McpAppsAdapter on a known UiSurface, assert output structure matches expected shapes
    - **Validates: Requirements 8.3**
    - Note: Full cross-feature verification (with vs without `awp`) is a CI concern — run `cargo test` and `cargo test --features awp` in CI matrix

- [x] 12. Backward compatibility verification
  - Run `cargo test` without `awp` feature — all 93 existing tests must pass
  - Run `cargo test --features awp` — all 93 existing tests plus new AWP tests must pass
  - Verify existing adapters produce identical output with and without `awp` feature
  - _Requirements: 8.1, 8.2, 8.3, 8.4_

- [ ] 13. Final checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document (Properties 1–11)
- The HTML renderer (`src/html.rs`) is the largest implementation piece — it maps 30+ component variants to HTML elements
- All AWP-specific code uses `#[cfg(feature = "awp")]` for clean feature isolation
- The `proptest` crate is used for property-based testing with minimum 100 iterations per property
