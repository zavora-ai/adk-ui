use crate::compat::ToolContext;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

pub const ACTIVE_SURFACE_STATE_KEY: &str = "adk_ui.active_surface";
pub const SURFACE_REFS_STATE_KEY: &str = "adk_ui.surface_refs";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SurfaceRef {
    pub id: String,
    pub version: u64,
}

impl SurfaceRef {
    pub fn new(id: impl Into<String>, version: u64) -> Self {
        Self {
            id: id.into(),
            version,
        }
    }
}

fn revisions() -> &'static Mutex<HashMap<String, u64>> {
    static REVISIONS: OnceLock<Mutex<HashMap<String, u64>>> = OnceLock::new();
    REVISIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn revision_key(ctx: &dyn ToolContext, surface_id: &str) -> String {
    format!(
        "{}\u{1f}{}\u{1f}{}\u{1f}{}",
        ctx.app_name(),
        ctx.user_id(),
        ctx.session_id(),
        surface_id
    )
}

pub(crate) fn next_surface_ref(
    ctx: &Arc<dyn ToolContext>,
    surface_id: impl Into<String>,
) -> SurfaceRef {
    let surface_id = surface_id.into();
    let key = revision_key(ctx.as_ref(), &surface_id);
    let mut values = revisions()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let version = values
        .entry(key)
        .and_modify(|value| *value += 1)
        .or_insert(1);
    SurfaceRef::new(surface_id, *version)
}

pub(crate) fn observe_surface_version(
    ctx: &Arc<dyn ToolContext>,
    surface_id: impl Into<String>,
    version: u64,
) -> SurfaceRef {
    let surface_id = surface_id.into();
    let key = revision_key(ctx.as_ref(), &surface_id);
    let mut values = revisions()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    values
        .entry(key)
        .and_modify(|current| *current = (*current).max(version))
        .or_insert(version);
    SurfaceRef::new(surface_id, version)
}

pub(crate) fn record_surface_ref(ctx: &Arc<dyn ToolContext>, surface_ref: &SurfaceRef) {
    let mut actions = ctx.actions();
    actions.state_delta.insert(
        ACTIVE_SURFACE_STATE_KEY.to_string(),
        serde_json::to_value(surface_ref).unwrap_or_else(|_| {
            json!({
                "id": surface_ref.id,
                "version": surface_ref.version,
            })
        }),
    );
    ctx.set_actions(actions);
}

pub(crate) fn record_surface_refs(
    ctx: &Arc<dyn ToolContext>,
    active: &SurfaceRef,
    refs: &[SurfaceRef],
) {
    let mut actions = ctx.actions();
    actions.state_delta.insert(
        ACTIVE_SURFACE_STATE_KEY.to_string(),
        serde_json::to_value(active).unwrap_or(Value::Null),
    );
    actions.state_delta.insert(
        SURFACE_REFS_STATE_KEY.to_string(),
        serde_json::to_value(refs).unwrap_or(Value::Array(vec![])),
    );
    ctx.set_actions(actions);
}

pub(crate) fn surface_owner(ctx: &dyn ToolContext) -> String {
    let agent = ctx.agent_name().trim();
    if !agent.is_empty() {
        agent.to_string()
    } else {
        ctx.user_id().to_string()
    }
}
