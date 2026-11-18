//! Components under construction, shared by the builders of every catalog.

use serde_json::{Map, Value, json};

/// A component under construction.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Props(pub(crate) Map<String, Value>);

impl Props {
    pub(crate) fn new(id: impl Into<String>, component: &str) -> Self {
        let mut map = Map::new();
        map.insert("id".into(), Value::String(id.into()));
        map.insert("component".into(), Value::String(component.into()));
        Self(map)
    }

    pub(crate) fn set(&mut self, name: &str, value: impl Into<Value>) {
        self.0.insert(name.into(), value.into());
    }

    pub(crate) fn push(&mut self, name: &str, value: Value) {
        match self.0.get_mut(name) {
            Some(Value::Array(items)) => items.push(value),
            _ => self.set(name, json!([value])),
        }
    }
}

macro_rules! component_builder {
    ($name:ident) => {
        impl From<$name> for Value {
            fn from(builder: $name) -> Value {
                builder.build()
            }
        }

        impl $name {
            /// The component, as A2UI JSON.
            pub fn build(self) -> Value {
                Value::Object(self.0.0)
            }
        }
    };
}

pub(crate) use component_builder;
