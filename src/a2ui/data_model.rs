use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::messages::{A2uiMessage, UpdateDataModel, UpdateDataModelMessage};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataModelUpdate {
    pub surface_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

#[derive(Debug, Clone)]
pub enum DataModelValue {
    Null,
    Json(Value),
}

#[derive(Debug, Default)]
pub struct UpdateDataModelBuilder {
    surface_id: String,
    path: Option<String>,
    value: Option<Value>,
}

impl UpdateDataModelBuilder {
    pub fn new(surface_id: impl Into<String>) -> Self {
        Self {
            surface_id: surface_id.into(),
            path: None,
            value: None,
        }
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn value(mut self, value: DataModelValue) -> Self {
        self.value = match value {
            DataModelValue::Null => Some(Value::Null),
            DataModelValue::Json(v) => Some(v),
        };
        self
    }

    pub fn build(self) -> A2uiMessage {
        A2uiMessage::UpdateDataModel(UpdateDataModelMessage {
            update_data_model: UpdateDataModel {
                surface_id: self.surface_id,
                path: self.path,
                value: self.value,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn builds_update_data_model_message() {
        let message = UpdateDataModelBuilder::new("main")
            .path("/user")
            .value(DataModelValue::Json(json!({"name": "Alice"})))
            .build();

        let value = serde_json::to_value(&message).unwrap();
        assert_eq!(value["updateDataModel"]["surfaceId"], "main");
        assert_eq!(value["updateDataModel"]["path"], "/user");
    }
}
