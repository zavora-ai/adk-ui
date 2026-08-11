use serde::Serialize;

/// Default runtime protocol profile for server integrations.
///
/// New integrations should prefer an explicit protocol (`a2ui`, `ag_ui`,
/// `mcp_apps`, or `awp`). The legacy default remains for compatibility until
/// the 2026-12-31 sunset.
pub const UI_DEFAULT_PROTOCOL: &str = "adk_ui";

/// Preferred default for new integrations (A2UI current production family).
pub const UI_PREFERRED_PROTOCOL: &str = "a2ui";

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
    Candidate,
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
    note: Some(
        "Legacy adk_ui profile remains supported during migration. Prefer a2ui for new integrations.",
    ),
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
        versions: &["0.9", "0.9.1"],
        implementation_tier: UiProtocolImplementationTier::NativeSubset,
        spec_track: UiProtocolSpecTrack::Stable,
        summary: "A2UI v0.9.1 production + v1.0 actionResponse RPC: versioned JSONL, create/update/delete surface, client action with wantResponse/actionId, actionResponse apply-to-data-model, application/a2ui+json.",
        features: &[
            "jsonl",
            "versioned_envelopes_v0_9_1",
            "versioned_envelopes_v1_0",
            "flat_components",
            "createSurface",
            "updateComponents",
            "updateDataModel",
            "deleteSurface",
            "client_action_messages",
            "action_want_response",
            "action_response",
            "call_function",
            "function_response",
            "validation_failed_errors",
            "client_metadata",
            "validation_feedback",
            "basic_catalog_functions",
            "local_actions",
            "mime_application_a2ui_json",
            "optional_bulk_createSurface_fields",
        ],
        limitations: &[
            "A2UI v1.0 multi-catalog mix and full catalog composition constraints are partial; actionResponse/callFunction are implemented end-to-end.",
            "Dedicated A2A transport bindings remain host-owned; this crate emits protocol JSONL and client action envelopes.",
        ],
        deprecation: None,
    },
    UiProtocolCapabilitySpec {
        protocol: "ag_ui",
        versions: &["0.1"],
        implementation_tier: UiProtocolImplementationTier::NativeSubset,
        spec_track: UiProtocolSpecTrack::Stable,
        summary: "AG-UI native event families: run lifecycle, text/tool streams, state snapshot/delta, activity, reasoning, and ADK surface CUSTOM + ACTIVITY_SNAPSHOT transport.",
        features: &[
            "event_stream",
            "run_lifecycle",
            "run_started_parent_run_id",
            "native_run_input_ingest",
            "stable_text_events",
            "text_message_chunk",
            "stable_tool_events",
            "tool_call_chunk",
            "tool_call_result",
            "state_snapshot",
            "state_delta",
            "messages_snapshot",
            "activity_snapshot",
            "activity_delta",
            "reasoning_events",
            "reasoning_message_chunk",
            "custom_surface_event",
            "surface_activity_snapshot",
            "run_error",
            "raw_events",
        ],
        limitations: &[
            "Example hosts may still wrap some ADK runtime events; prefer protocol_native SSE when available from adk-server.",
            "Full bidirectional shared-state / HITL policy is application-defined on top of these event types.",
        ],
        deprecation: None,
    },
    UiProtocolCapabilitySpec {
        protocol: "mcp_apps",
        versions: &["sep-1865", "2025-11-21"],
        implementation_tier: UiProtocolImplementationTier::NativeSubset,
        spec_track: UiProtocolSpecTrack::Stable,
        summary: "MCP Apps (SEP-1865 Final): ui:// resources, text/html;profile=mcp-app, extension id io.modelcontextprotocol/ui, bridge initialize/message/model-context, structured tool results.",
        features: &[
            "extension_id_io_modelcontextprotocol_ui",
            "ui_resource_uri",
            "tool_meta_ui_resource_uri",
            "html_profile_mcp_app",
            "structured_content",
            "initialize_request",
            "initialize_bridge_endpoint",
            "message_bridge_endpoint",
            "update_model_context_bridge_endpoint",
            "bridge_host_context",
            "bridge_app_capabilities",
            "extension_capability_advertisement",
            "resource_list_changed_capability",
            "tool_list_changed_capability",
            "inline_html_resource",
            "sandbox_csp_meta",
        ],
        limitations: &[
            "Full browser postMessage host SDK is host-owned; adk-ui provides payloads + HTTP bridge shapes used with adk-server.",
            "Bridge session durability depends on the host (adk-server in-memory by default).",
        ],
        deprecation: None,
    },
];

/// AWP protocol capability specification.
#[cfg(feature = "awp")]
pub const AWP_PROTOCOL_CAPABILITY: UiProtocolCapabilitySpec = UiProtocolCapabilitySpec {
    protocol: "awp",
    versions: &["1.0"],
    implementation_tier: UiProtocolImplementationTier::CompatibilitySubset,
    spec_track: UiProtocolSpecTrack::Draft,
    summary: "AWP-aligned HTML rendering with capability manifest export and bandwidth-adaptive output.",
    features: &[
        "html_rendering",
        "capability_manifest_export",
        "bandwidth_adaptive",
        "tool_envelope_bridge",
    ],
    limitations: &[
        "HTML renderer produces static markup; interactive behaviors require client-side hydration.",
        "Chart components render as data-attribute placeholders, not visual charts.",
        "Site-level AWP (discovery, trust, rate limits) lives in adk-awp; this crate supplies UI payload/HTML only.",
    ],
    deprecation: None,
};

/// Normalize runtime UI profile aliases to canonical values.
pub fn normalize_runtime_ui_protocol(raw: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "adk_ui" => Some("adk_ui"),
        "a2ui" => Some("a2ui"),
        "ag_ui" | "ag-ui" => Some("ag_ui"),
        "mcp_apps" | "mcp-apps" => Some("mcp_apps"),
        #[cfg(feature = "awp")]
        "awp" => Some("awp"),
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
    fn modern_protocols_are_native_subset() {
        for name in ["a2ui", "ag_ui", "mcp_apps"] {
            let spec = UI_PROTOCOL_CAPABILITIES
                .iter()
                .find(|s| s.protocol == name)
                .unwrap_or_else(|| panic!("missing {name}"));
            assert_eq!(
                spec.implementation_tier,
                UiProtocolImplementationTier::NativeSubset,
                "{name}"
            );
            assert_eq!(spec.spec_track, UiProtocolSpecTrack::Stable, "{name}");
        }
        let a2ui = UI_PROTOCOL_CAPABILITIES
            .iter()
            .find(|s| s.protocol == "a2ui")
            .unwrap();
        assert!(a2ui.versions.contains(&"0.9.1"));
    }

    #[test]
    fn preferred_protocol_is_a2ui() {
        assert_eq!(UI_PREFERRED_PROTOCOL, "a2ui");
        assert!(normalize_runtime_ui_protocol(UI_PREFERRED_PROTOCOL).is_some());
    }
}
