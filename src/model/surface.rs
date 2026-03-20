use crate::interop::UiSurface;
use crate::model::CanonicalComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Canonical surface model used by protocol adapters and envelope projection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalSurface {
    pub surface_id: String,
    pub catalog_id: String,
    pub components: Vec<CanonicalComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_model: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<Value>,
    pub send_data_model: bool,
}

impl CanonicalSurface {
    pub fn new(
        surface_id: impl Into<String>,
        catalog_id: impl Into<String>,
        components: Vec<CanonicalComponent>,
    ) -> Self {
        Self {
            surface_id: surface_id.into(),
            catalog_id: catalog_id.into(),
            components,
            data_model: None,
            theme: None,
            send_data_model: true,
        }
    }

    pub fn with_data_model(mut self, data_model: Option<Value>) -> Self {
        self.data_model = data_model;
        self
    }

    pub fn with_theme(mut self, theme: Option<Value>) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_send_data_model(mut self, send_data_model: bool) -> Self {
        self.send_data_model = send_data_model;
        self
    }
}

impl From<UiSurface> for CanonicalSurface {
    fn from(surface: UiSurface) -> Self {
        Self {
            surface_id: surface.surface_id,
            catalog_id: surface.catalog_id,
            components: surface
                .components
                .into_iter()
                .map(CanonicalComponent::from)
                .collect(),
            data_model: surface.data_model,
            theme: surface.theme,
            send_data_model: surface.send_data_model,
        }
    }
}

impl From<CanonicalSurface> for UiSurface {
    fn from(surface: CanonicalSurface) -> Self {
        UiSurface {
            surface_id: surface.surface_id,
            catalog_id: surface.catalog_id,
            components: surface.components.into_iter().map(Value::from).collect(),
            data_model: surface.data_model,
            theme: surface.theme,
            send_data_model: surface.send_data_model,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonical_surface_round_trips_with_ui_surface() {
        let ui_surface = UiSurface::new(
            "main",
            "catalog",
            vec![
                json!({ "id": "root", "component": "Column", "children": [] }),
                json!({ "id": "title", "component": "Text", "text": "hello" }),
            ],
        )
        .with_data_model(Some(json!({ "ok": true })))
        .with_theme(Some(json!({ "mode": "dark" })))
        .with_send_data_model(false);

        let canonical: CanonicalSurface = ui_surface.clone().into();
        assert_eq!(canonical.surface_id, "main");
        assert_eq!(canonical.components.len(), 2);
        assert_eq!(canonical.components[0].id(), Some("root"));

        let restored: UiSurface = canonical.into();
        assert_eq!(restored.surface_id, ui_surface.surface_id);
        assert_eq!(restored.catalog_id, ui_surface.catalog_id);
        assert_eq!(restored.components, ui_surface.components);
        assert_eq!(restored.data_model, ui_surface.data_model);
        assert_eq!(restored.theme, ui_surface.theme);
        assert_eq!(restored.send_data_model, ui_surface.send_data_model);
    }
}
