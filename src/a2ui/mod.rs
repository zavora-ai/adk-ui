pub mod bindings;
pub mod capabilities;
pub mod components;
pub mod data_model;
pub mod encoding;
pub mod events;
pub mod ids;
pub mod messages;
pub mod prompts;
pub mod validator;

pub use bindings::DynamicString;
pub use capabilities::{
    A2uiAgentCapabilities, A2uiAgentCapabilitySet, A2uiRendererCapabilities,
    A2uiRendererCapabilitySet, default_a2ui_agent_capabilities, default_a2ui_renderer_capabilities,
};
pub use components::{button, column, divider, image, row, text};
pub use data_model::{DataModelUpdate, DataModelValue, UpdateDataModelBuilder};
pub use encoding::{encode_jsonl, encode_jsonl_bytes, encode_message_line};
pub use events::{A2uiActionEvent, A2uiActionMetadata, UiEventMapper};
pub use ids::{stable_child_id, stable_id, stable_indexed_id};
pub use messages::{
    A2UI_BASIC_CATALOG_ID, A2UI_BASIC_CATALOG_ID_V1, A2UI_MIME_TYPE, A2UI_MIME_TYPE_LEGACY,
    A2UI_PROTOCOL_VERSION, A2uiActionResponseError, A2uiClientAction, A2uiClientActionMessage,
    A2uiClientMessage, A2uiErrorMessage, A2uiMessage, A2uiValidationFailedError,
    ActionResponseBody, ActionResponseMessage, CallFunctionBody, CallFunctionMessage,
    CreateSurface, CreateSurfaceMessage, DeleteSurface, DeleteSurfaceMessage, FunctionResponseBody,
    FunctionResponseMessage, UpdateComponents, UpdateComponentsMessage, UpdateDataModel,
    UpdateDataModelMessage, apply_action_response_value,
};
pub use prompts::A2UI_AGENT_PROMPT;
pub use validator::{A2uiSchemaVersion, A2uiValidationError, A2uiValidator};
