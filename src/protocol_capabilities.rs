use serde::Serialize;

/// Default runtime protocol profile for server integrations.
pub const UI_DEFAULT_PROTOCOL: &str = "adk_ui";

/// Tool envelope version used by protocol-aware legacy tool responses.
pub const TOOL_ENVELOPE_VERSION: &str = "1.0";

/// Supported runtime protocol profile values.
pub const SUPPORTED_UI_PROTOCOLS: &[&str] = &["adk_ui", "a2ui", "ag_ui", "mcp_apps"];

/// Planned deprecation metadata for runtime/profile consumers.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiProtocolDeprecationSpec {
    pub stage: &'static str,
    pub announced_on: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sunset_target_on: Option<&'static str>,
    pub replacement_protocols: &'static [&'static str],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<&'static str>,
}

/// Implementation maturity for protocol support exposed to clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UiProtocolImplementationTier {
    Legacy,
    NativeSubset,
    HybridSubset,
    CompatibilitySubset,
}

/// Upstream specification track referenced by the runtime capability signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UiProtocolSpecTrack {
    Internal,
    Stable,
    Draft,
}

/// Static capability contract for each supported UI protocol.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiProtocolCapabilitySpec {
    pub protocol: &'static str,
    pub versions: &'static [&'static str],
    pub implementation_tier: UiProtocolImplementationTier,
    pub spec_track: UiProtocolSpecTrack,
    pub summary: &'static str,
    pub features: &'static [&'static str],
    pub limitations: &'static [&'static str],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation: Option<&'static UiProtocolDeprecationSpec>,
}

pub const ADK_UI_LEGACY_DEPRECATION: UiProtocolDeprecationSpec = UiProtocolDeprecationSpec {
    stage: "planned",
    announced_on: "2026-02-07",
    sunset_target_on: Some("2026-12-31"),
    replacement_protocols: &["a2ui", "ag_ui", "mcp_apps"],
    note: Some("Legacy adk_ui profile remains supported during migration."),
};

pub const UI_PROTOCOL_CAPABILITIES: &[UiProtocolCapabilitySpec] = &[
    UiProtocolCapabilitySpec {
        protocol: "adk_ui",
        versions: &["1.0"],
        implementation_tier: UiProtocolImplementationTier::Legacy,
        spec_track: UiProtocolSpecTrack::Internal,
        summary: "Legacy internal runtime profile retained for backward compatibility during migration.",
        features: &["legacy_components", "theme", "events"],
        limitations: &[
            "Deprecated for new integrations.",
            "Does not represent a standard external protocol surface.",
        ],
        deprecation: Some(&ADK_UI_LEGACY_DEPRECATION),
    },
    UiProtocolCapabilitySpec {
        protocol: "a2ui",
        versions: &["0.9"],
        implementation_tier: UiProtocolImplementationTier::HybridSubset,
        spec_track: UiProtocolSpecTrack::Draft,
        summary: "Core A2UI surface transport with flat-component alignment, metadata-aware client envelopes, and validation-capable renderer behavior.",
        features: &[
            "jsonl",
            "flat_components",
            "createSurface",
            "updateComponents",
            "updateDataModel",
            "client_metadata",
            "validation_feedback",
            "basic_catalog_functions",
            "local_actions",
        ],
        limitations: &[
            "Treat current support as a v0.9-aligned subset while the upstream v0.9 spec remains draft.",
            "The package now covers the practical metadata, validation, and basic catalog flows used by this repo, but it still does not claim every draft-only renderer/runtime feature.",
        ],
        deprecation: None,
    },
    UiProtocolCapabilitySpec {
        protocol: "ag_ui",
        versions: &["0.1"],
        implementation_tier: UiProtocolImplementationTier::CompatibilitySubset,
        spec_track: UiProtocolSpecTrack::Stable,
        summary: "Compatibility-oriented AG-UI subset with native run-input ingestion, lifecycle events, custom surface transport, and stable text/tool event ingestion.",
        features: &[
            "event_stream",
            "native_run_input_ingest",
            "run_lifecycle",
            "custom_surface_event",
            "stable_text_events",
            "stable_tool_events",
            "messages_snapshot_ingest",
            "run_error",
        ],
        limitations: &[
            "The example client now prefers protocol-native AG-UI request input, but the bundled example server still serializes wrapped runtime events rather than a fully native AG-UI SSE envelope.",
            "Activity snapshots/deltas, reasoning streams, and chunk aggregation are accepted as compatibility inputs but are not yet produced end-to-end by the example server.",
        ],
        deprecation: None,
    },
    UiProtocolCapabilitySpec {
        protocol: "mcp_apps",
        versions: &["sep-1865"],
        implementation_tier: UiProtocolImplementationTier::CompatibilitySubset,
        spec_track: UiProtocolSpecTrack::Stable,
        summary: "Compatibility-oriented MCP Apps subset with initialize requests, bridge-aware structured tool results, ui:// resources, and inline HTML fallback surfaces.",
        features: &[
            "ui_resource_uri",
            "tool_meta",
            "structured_content",
            "initialize_request",
            "initialize_bridge_endpoint",
            "message_bridge_endpoint",
            "update_model_context_bridge_endpoint",
            "bridge_host_context",
            "bridge_app_capabilities",
            "inline_html_resource",
        ],
        limitations: &[
            "The example host/client path now supports initialize, message, and model-context bridge endpoints plus bridge-aware response rendering, but the library still exposes MCP Apps primarily through compatibility adapters rather than a standalone embedded app bridge API.",
            "Static HTML resource fallback remains part of the public MCP Apps surface while broader host negotiation, resource notifications, and fully app-native bridge semantics are still partial.",
        ],
        deprecation: None,
    },
];

/// Normalize runtime UI profile aliases to canonical values.
pub fn normalize_runtime_ui_protocol(raw: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "adk_ui" => Some("adk_ui"),
        "a2ui" => Some("a2ui"),
        "ag_ui" | "ag-ui" => Some("ag_ui"),
        "mcp_apps" | "mcp-apps" => Some("mcp_apps"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_runtime_protocol_accepts_aliases() {
        assert_eq!(normalize_runtime_ui_protocol("adk_ui"), Some("adk_ui"));
        assert_eq!(normalize_runtime_ui_protocol("A2UI"), Some("a2ui"));
        assert_eq!(normalize_runtime_ui_protocol("ag-ui"), Some("ag_ui"));
        assert_eq!(normalize_runtime_ui_protocol("mcp-apps"), Some("mcp_apps"));
        assert_eq!(normalize_runtime_ui_protocol("unknown"), None);
    }

    #[test]
    fn capability_specs_cover_supported_protocols() {
        let protocols: Vec<&str> = UI_PROTOCOL_CAPABILITIES
            .iter()
            .map(|spec| spec.protocol)
            .collect();
        assert_eq!(protocols, SUPPORTED_UI_PROTOCOLS);
    }

    #[test]
    fn capability_specs_include_versions() {
        for spec in UI_PROTOCOL_CAPABILITIES {
            assert!(
                !spec.versions.is_empty(),
                "missing versions for {}",
                spec.protocol
            );
            assert!(
                !spec.features.is_empty(),
                "missing features for {}",
                spec.protocol
            );
            assert!(
                !spec.summary.trim().is_empty(),
                "missing summary for {}",
                spec.protocol
            );
            assert!(
                !spec.limitations.is_empty(),
                "missing limitations for {}",
                spec.protocol
            );
        }
    }

    #[test]
    fn legacy_profile_has_deprecation_metadata() {
        let legacy = UI_PROTOCOL_CAPABILITIES
            .iter()
            .find(|spec| spec.protocol == "adk_ui")
            .expect("adk_ui capability");
        let deprecation = legacy.deprecation.expect("adk_ui deprecation metadata");
        assert_eq!(deprecation.announced_on, "2026-02-07");
        assert_eq!(deprecation.sunset_target_on, Some("2026-12-31"));
    }

    #[test]
    fn capability_specs_capture_support_boundaries() {
        let a2ui = UI_PROTOCOL_CAPABILITIES
            .iter()
            .find(|spec| spec.protocol == "a2ui")
            .expect("a2ui capability");
        assert_eq!(
            a2ui.implementation_tier,
            UiProtocolImplementationTier::HybridSubset
        );
        assert_eq!(a2ui.spec_track, UiProtocolSpecTrack::Draft);

        let ag_ui = UI_PROTOCOL_CAPABILITIES
            .iter()
            .find(|spec| spec.protocol == "ag_ui")
            .expect("ag_ui capability");
        assert_eq!(
            ag_ui.implementation_tier,
            UiProtocolImplementationTier::CompatibilitySubset
        );

        let mcp_apps = UI_PROTOCOL_CAPABILITIES
            .iter()
            .find(|spec| spec.protocol == "mcp_apps")
            .expect("mcp_apps capability");
        assert_eq!(
            mcp_apps.implementation_tier,
            UiProtocolImplementationTier::CompatibilitySubset
        );
        assert_eq!(mcp_apps.spec_track, UiProtocolSpecTrack::Stable);
        assert!(
            mcp_apps.features.contains(&"initialize_request"),
            "mcp_apps feature list should include initialize support"
        );
    }

    #[test]
    fn capability_specs_serialize_support_metadata_in_camel_case() {
        let value = serde_json::to_value(UI_PROTOCOL_CAPABILITIES).expect("serialize capabilities");
        let protocols = value.as_array().expect("capabilities array");
        let a2ui = protocols
            .iter()
            .find(|spec| spec["protocol"] == "a2ui")
            .expect("a2ui json capability");

        assert_eq!(a2ui["implementationTier"], "hybrid_subset");
        assert_eq!(a2ui["specTrack"], "draft");
        assert!(a2ui["summary"].as_str().is_some());
        assert!(
            a2ui["limitations"]
                .as_array()
                .is_some_and(|limitations| !limitations.is_empty())
        );
    }
}
