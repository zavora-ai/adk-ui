use serde::{Deserialize, Serialize};

/// A2UI v0.9 createSurface payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSurface {
    pub surface_id: String,
    pub catalog_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_data_model: Option<bool>,
}

/// A2UI v0.9 updateComponents payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponents {
    pub surface_id: String,
    pub components: Vec<serde_json::Value>,
}

/// A2UI v0.9 updateDataModel payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDataModel {
    pub surface_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

/// A2UI v0.9 deleteSurface payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSurface {
    pub surface_id: String,
}

/// Envelope: createSurface message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSurfaceMessage {
    #[serde(rename = "createSurface")]
    pub create_surface: CreateSurface,
}

/// Envelope: updateComponents message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateComponentsMessage {
    #[serde(rename = "updateComponents")]
    pub update_components: UpdateComponents,
}

/// Envelope: updateDataModel message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDataModelMessage {
    #[serde(rename = "updateDataModel")]
    pub update_data_model: UpdateDataModel,
}

/// Envelope: deleteSurface message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSurfaceMessage {
    #[serde(rename = "deleteSurface")]
    pub delete_surface: DeleteSurface,
}

/// A2UI v0.9 message envelope (exactly one of the variants).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum A2uiMessage {
    CreateSurface(CreateSurfaceMessage),
    UpdateComponents(UpdateComponentsMessage),
    UpdateDataModel(UpdateDataModelMessage),
    DeleteSurface(DeleteSurfaceMessage),
}
