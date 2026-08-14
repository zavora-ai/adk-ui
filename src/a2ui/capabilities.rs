use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{A2UI_BASIC_CATALOG_ID, A2UI_BASIC_CATALOG_ID_V1};

fn is_false(value: &bool) -> bool {
    !*value
}

/// Catalog support advertised by an A2UI renderer.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct A2uiRendererCapabilitySet {
    pub supported_catalog_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_catalogs: Option<Vec<Value>>,
}

/// Versioned A2UI renderer capabilities. The upstream schemas intentionally
/// keep the family key (`v0.9`) for the production v0.9.1 patch release.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct A2uiRendererCapabilities {
    #[serde(rename = "v0.9", skip_serializing_if = "Option::is_none")]
    pub v0_9: Option<A2uiRendererCapabilitySet>,
    #[serde(rename = "v1.0", skip_serializing_if = "Option::is_none")]
    pub v1_0: Option<A2uiRendererCapabilitySet>,
}

/// Catalog support advertised by an A2UI agent.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct A2uiAgentCapabilitySet {
    pub supported_catalog_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub accepts_inline_catalogs: bool,
}

/// Versioned A2UI agent capabilities for production and candidate renderers.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct A2uiAgentCapabilities {
    #[serde(rename = "v0.9", skip_serializing_if = "Option::is_none")]
    pub v0_9: Option<A2uiAgentCapabilitySet>,
    #[serde(rename = "v1.0", skip_serializing_if = "Option::is_none")]
    pub v1_0: Option<A2uiAgentCapabilitySet>,
}

pub fn default_a2ui_renderer_capabilities() -> A2uiRendererCapabilities {
    A2uiRendererCapabilities {
        v0_9: Some(A2uiRendererCapabilitySet {
            supported_catalog_ids: vec![A2UI_BASIC_CATALOG_ID.to_string()],
            inline_catalogs: None,
        }),
        v1_0: Some(A2uiRendererCapabilitySet {
            supported_catalog_ids: vec![A2UI_BASIC_CATALOG_ID_V1.to_string()],
            inline_catalogs: None,
        }),
    }
}

pub fn default_a2ui_agent_capabilities() -> A2uiAgentCapabilities {
    A2uiAgentCapabilities {
        v0_9: Some(A2uiAgentCapabilitySet {
            supported_catalog_ids: vec![A2UI_BASIC_CATALOG_ID.to_string()],
            accepts_inline_catalogs: false,
        }),
        v1_0: Some(A2uiAgentCapabilitySet {
            supported_catalog_ids: vec![A2UI_BASIC_CATALOG_ID_V1.to_string()],
            accepts_inline_catalogs: false,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_use_upstream_version_family_keys() {
        let renderer = serde_json::to_value(default_a2ui_renderer_capabilities()).unwrap();
        let agent = serde_json::to_value(default_a2ui_agent_capabilities()).unwrap();

        assert_eq!(
            renderer["v0.9"]["supportedCatalogIds"][0],
            A2UI_BASIC_CATALOG_ID
        );
        assert_eq!(
            renderer["v1.0"]["supportedCatalogIds"][0],
            A2UI_BASIC_CATALOG_ID_V1
        );
        assert_eq!(agent["v1.0"]["acceptsInlineCatalogs"], Value::Null);
    }
}
