pub mod a2ui;
pub mod catalog_registry;
pub mod compat;
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
pub use catalog_registry::{CatalogArtifact, CatalogError, CatalogRegistry, CatalogSource};
pub use interop::*;
pub use kit::{KitArtifacts, KitGenerator, KitSpec};
pub use model::{ToolEnvelope, ToolEnvelopeProtocol};
pub use prompts::{UI_AGENT_PROMPT, UI_AGENT_PROMPT_SHORT};
pub use protocol_capabilities::{
    ADK_UI_LEGACY_DEPRECATION, SUPPORTED_UI_PROTOCOLS, TOOL_ENVELOPE_VERSION, UI_DEFAULT_PROTOCOL,
    UI_PROTOCOL_CAPABILITIES, UiProtocolCapabilitySpec, UiProtocolDeprecationSpec,
    UiProtocolImplementationTier, UiProtocolSpecTrack, normalize_runtime_ui_protocol,
};
pub use schema::*;
pub use templates::{StatItem, TemplateData, UiTemplate, UserData, render_template};
pub use tools::*;
pub use toolset::UiToolset;
pub use validation::{Validate, ValidationError, validate_ui_response};
