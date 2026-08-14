pub mod a2ui;
pub mod application;
#[cfg(feature = "awp")]
pub mod awp_compat;
pub mod catalog_registry;
pub mod compat;
pub mod html;
pub mod interop;
pub mod kit;
pub mod model;
pub mod prompts;
pub mod protocol_capabilities;
pub mod schema;
pub mod templates;
pub mod tools;
pub mod toolset;
pub mod validation;

pub use a2ui::*;
pub use application::*;
pub use catalog_registry::{CatalogArtifact, CatalogError, CatalogRegistry, CatalogSource};
pub use interop::*;
pub use kit::{BrandKitManifest, KitArtifacts, KitGenerator, KitSpec};
pub use model::{ToolEnvelope, ToolEnvelopeProtocol};
pub use prompts::{UI_AGENT_PROMPT, UI_AGENT_PROMPT_SHORT};
#[cfg(feature = "awp")]
pub use protocol_capabilities::AWP_PROTOCOL_CAPABILITY;
pub use protocol_capabilities::{
    ADK_UI_LEGACY_DEPRECATION, SUPPORTED_UI_PROTOCOLS, TOOL_ENVELOPE_VERSION,
    UI_CAPABILITY_SCHEMA_VERSION, UI_DEFAULT_PROTOCOL, UI_PREFERRED_PROTOCOL,
    UI_PROTOCOL_CAPABILITIES, UiCapabilityScope, UiCapabilityStatus, UiProtocolCapabilityClaim,
    UiProtocolCapabilitySpec, UiProtocolDeprecationSpec, UiProtocolImplementationTier,
    UiProtocolNegotiationSpec, UiProtocolSpecTrack, UiProtocolVersionSpec,
    normalize_runtime_ui_protocol,
};

// Re-export frequently used protocol constants at the crate root for discoverability.
pub use a2ui::{A2UI_BASIC_CATALOG_ID, A2UI_MIME_TYPE, A2UI_PROTOCOL_VERSION};
pub use interop::{MCP_APPS_EXTENSION_ID, MCP_APPS_SEP_STATUS};
pub use schema::*;
pub use templates::{StatItem, TemplateData, UiTemplate, UserData, render_template};
pub use tools::*;
pub use toolset::UiToolset;
pub use validation::{Validate, ValidationError, validate_ui_response};

pub use html::{BandwidthMode, HtmlRenderOptions, render_components_html, render_surface_html};
