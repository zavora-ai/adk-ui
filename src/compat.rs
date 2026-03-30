//! Compatibility layer providing standalone versions of types from `adk-core`.
//!
//! When the `adk-core` feature is enabled, these re-export from the real crate.
//! Otherwise, minimal local definitions are used so `adk-ui` compiles independently.

#[cfg(feature = "adk-core")]
pub use adk_core::{
    AdkError, Artifacts, CallbackContext, Content, EventActions, MemoryEntry, Part,
    ReadonlyContext, Result, Tool, ToolContext, Toolset,
};

#[cfg(not(feature = "adk-core"))]
mod standalone {
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use std::fmt;
    use std::sync::Arc;

    // ── Error ──────────────────────────────────────────────────────────

    #[derive(Debug, Clone)]
    pub enum AdkError {
        Tool(String),
        Other(String),
    }

    impl AdkError {
        pub fn tool(msg: impl Into<String>) -> Self {
            AdkError::Tool(msg.into())
        }
    }

    impl fmt::Display for AdkError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                AdkError::Tool(msg) => write!(f, "Tool error: {}", msg),
                AdkError::Other(msg) => write!(f, "{}", msg),
            }
        }
    }

    impl std::error::Error for AdkError {}

    pub type Result<T> = std::result::Result<T, AdkError>;

    // ── Content / Part ─────────────────────────────────────────────────

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Content {
        pub role: String,
        pub parts: Vec<Part>,
    }

    impl Content {
        pub fn new(role: &str) -> Self {
            Self {
                role: role.to_string(),
                parts: Vec::new(),
            }
        }

        pub fn with_text(mut self, text: impl Into<String>) -> Self {
            self.parts.push(Part::Text { text: text.into() });
            self
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Part {
        Text { text: String },
        InlineData { mime_type: String, data: Vec<u8> },
    }

    // ── Traits ─────────────────────────────────────────────────────────

    #[async_trait]
    pub trait Tool: Send + Sync {
        fn name(&self) -> &str;
        fn description(&self) -> &str;
        fn parameters_schema(&self) -> Option<Value> {
            None
        }
        async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value>;
    }

    pub trait ReadonlyContext: Send + Sync {
        fn invocation_id(&self) -> &str {
            ""
        }
        fn agent_name(&self) -> &str {
            ""
        }
        fn user_id(&self) -> &str {
            ""
        }
        fn app_name(&self) -> &str {
            ""
        }
        fn session_id(&self) -> &str {
            ""
        }
        fn branch(&self) -> &str {
            ""
        }
        fn user_content(&self) -> &Content;
        fn state(&self) -> Option<Value> {
            None
        }
    }

    #[async_trait]
    pub trait ToolContext: ReadonlyContext + Send + Sync {
        fn function_call_id(&self) -> &str {
            ""
        }
        fn actions(&self) -> EventActions {
            EventActions::default()
        }
        fn set_actions(&self, actions: EventActions);
        async fn search_memory(&self, query: &str) -> Result<Vec<MemoryEntry>>;
    }

    #[async_trait]
    pub trait Toolset: Send + Sync {
        fn name(&self) -> &str;
        async fn tools(&self, ctx: Arc<dyn ReadonlyContext>) -> Result<Vec<Arc<dyn Tool>>>;
    }

    pub trait CallbackContext: Send + Sync {
        fn artifacts(&self) -> Option<Arc<dyn Artifacts>>;
    }

    pub trait Artifacts: Send + Sync {}

    // ── Supporting types ───────────────────────────────────────────────

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct EventActions {
        #[serde(default)]
        pub transfer_to_agent: Option<String>,
        #[serde(default)]
        pub escalate: Option<String>,
        #[serde(default)]
        pub requested_auth_configs: Option<Value>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MemoryEntry {
        pub content: String,
        #[serde(default)]
        pub metadata: Option<Value>,
    }
}

#[cfg(not(feature = "adk-core"))]
pub use standalone::*;
