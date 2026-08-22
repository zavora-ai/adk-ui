use crate::compat::{AdkError, Result, Tool, ToolContext};
use crate::persistence::{SavedSurface, SurfaceStore};
use crate::schema::{Component, UiUpdate};
use crate::surface_runtime::{
    next_surface_ref, observe_surface_version, record_surface_ref, surface_owner,
};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;

fn store_error(error: impl std::fmt::Display) -> AdkError {
    AdkError::tool(error.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SaveSurfaceParams {
    pub id: String,
    pub name: String,
    pub payload: Value,
    #[serde(default)]
    pub expected_version: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SurfaceIdParams {
    pub id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ListSurfacesParams {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeleteSurfaceParams {
    pub id: String,
    #[serde(default)]
    pub expected_version: Option<u64>,
}

macro_rules! store_tool {
    ($name:ident) => {
        pub struct $name {
            store: Arc<dyn SurfaceStore>,
        }
        impl $name {
            pub fn new(store: Arc<dyn SurfaceStore>) -> Self {
                Self { store }
            }
        }
    };
}

store_tool!(SaveSurfaceTool);
store_tool!(LoadSurfaceTool);
store_tool!(ListSurfacesTool);
store_tool!(DeleteSurfaceTool);

#[async_trait]
impl Tool for SaveSurfaceTool {
    fn name(&self) -> &str {
        "save_surface"
    }
    fn description(&self) -> &str {
        "Save a named UI surface. Use expected_version to prevent overwriting a newer revision."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(super::generate_gemini_schema::<SaveSurfaceParams>())
    }
    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let params: SaveSurfaceParams = serde_json::from_value(args).map_err(store_error)?;
        let owner = surface_owner(ctx.as_ref());
        let saved = self
            .store
            .save(
                &owner,
                &params.id,
                &params.name,
                params.payload,
                params.expected_version,
            )
            .await
            .map_err(store_error)?;
        let surface_ref = observe_surface_version(&ctx, saved.id.clone(), saved.version);
        record_surface_ref(&ctx, &surface_ref);
        serde_json::to_value(saved).map_err(store_error)
    }
}

#[async_trait]
impl Tool for LoadSurfaceTool {
    fn name(&self) -> &str {
        "load_surface"
    }
    fn description(&self) -> &str {
        "Load a saved UI surface owned by the current agent."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(super::generate_gemini_schema::<SurfaceIdParams>())
    }
    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let params: SurfaceIdParams = serde_json::from_value(args).map_err(store_error)?;
        let saved = self
            .store
            .load(&surface_owner(ctx.as_ref()), &params.id)
            .await
            .map_err(store_error)?;
        let surface_ref = observe_surface_version(&ctx, saved.id.clone(), saved.version);
        record_surface_ref(&ctx, &surface_ref);
        serde_json::to_value(saved).map_err(store_error)
    }
}

#[async_trait]
impl Tool for ListSurfacesTool {
    fn name(&self) -> &str {
        "list_surfaces"
    }
    fn description(&self) -> &str {
        "List saved UI surfaces owned by the current agent."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(super::generate_gemini_schema::<ListSurfacesParams>())
    }
    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let _: ListSurfacesParams = serde_json::from_value(args).map_err(store_error)?;
        let surfaces = self
            .store
            .list(&surface_owner(ctx.as_ref()))
            .await
            .map_err(store_error)?;
        serde_json::to_value(surfaces).map_err(store_error)
    }
}

#[async_trait]
impl Tool for DeleteSurfaceTool {
    fn name(&self) -> &str {
        "delete_surface"
    }
    fn description(&self) -> &str {
        "Delete a saved UI surface. Use expected_version to prevent deleting a newer revision."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(super::generate_gemini_schema::<DeleteSurfaceParams>())
    }
    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let params: DeleteSurfaceParams = serde_json::from_value(args).map_err(store_error)?;
        let deleted = self
            .store
            .delete(
                &surface_owner(ctx.as_ref()),
                &params.id,
                params.expected_version,
            )
            .await
            .map_err(store_error)?;
        let mut actions = ctx.actions();
        actions.state_delta.insert(
            "adk_ui.deleted_surface".to_string(),
            json!({ "id": params.id, "deleted": deleted }),
        );
        ctx.set_actions(actions);
        Ok(json!({ "id": params.id, "deleted": deleted }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PatchSurfaceParams {
    #[serde(default = "default_surface_id")]
    pub surface_id: String,
    pub target_id: String,
    pub component: Component,
    #[serde(default)]
    pub expected_version: Option<u64>,
}

fn default_surface_id() -> String {
    "main".to_string()
}

pub struct PatchSurfaceTool {
    store: Option<Arc<dyn SurfaceStore>>,
}

impl PatchSurfaceTool {
    pub fn new() -> Self {
        Self { store: None }
    }
    pub fn with_store(store: Arc<dyn SurfaceStore>) -> Self {
        Self { store: Some(store) }
    }
}

impl Default for PatchSurfaceTool {
    fn default() -> Self {
        Self::new()
    }
}

fn merge_value(target: &mut Value, patch: &Value) {
    match (target, patch) {
        (Value::Object(target), Value::Object(patch)) => {
            for (key, value) in patch {
                target.insert(key.clone(), value.clone());
            }
        }
        (target, patch) => *target = patch.clone(),
    }
}

fn patch_component(value: &mut Value, target_id: &str, patch: &Value) -> bool {
    match value {
        Value::Object(object) => {
            if object.get("id").and_then(Value::as_str) == Some(target_id) {
                merge_value(value, patch);
                return true;
            }
            object
                .values_mut()
                .any(|child| patch_component(child, target_id, patch))
        }
        Value::Array(values) => values
            .iter_mut()
            .any(|child| patch_component(child, target_id, patch)),
        _ => false,
    }
}

#[async_trait]
impl Tool for PatchSurfaceTool {
    fn name(&self) -> &str {
        "patch_surface"
    }
    fn description(&self) -> &str {
        "Patch a component on an existing surface and emit an incremental UiUpdate. When persistence is configured, the saved surface is updated atomically."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(super::generate_gemini_schema::<PatchSurfaceParams>())
    }
    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let params: PatchSurfaceParams = serde_json::from_value(args).map_err(store_error)?;
        let update = UiUpdate::patch(params.target_id.clone(), params.component.clone());
        let surface_ref = if let Some(store) = &self.store {
            let owner = surface_owner(ctx.as_ref());
            let current = store
                .load(&owner, &params.surface_id)
                .await
                .map_err(store_error)?;
            let mut payload = current.payload.clone();
            let patch = serde_json::to_value(&params.component).map_err(store_error)?;
            if !patch_component(&mut payload, &params.target_id, &patch) {
                return Err(AdkError::tool(format!(
                    "component not found: {}",
                    params.target_id
                )));
            }
            let expected = params.expected_version.or(Some(current.version));
            let saved: SavedSurface = store
                .save(&owner, &current.id, &current.name, payload, expected)
                .await
                .map_err(store_error)?;
            observe_surface_version(&ctx, saved.id, saved.version)
        } else {
            next_surface_ref(&ctx, params.surface_id)
        };
        record_surface_ref(&ctx, &surface_ref);
        Ok(json!({ "surface_ref": surface_ref, "update": update }))
    }
}
