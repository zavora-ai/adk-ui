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

/// Version of the machine-readable capability document exposed by examples.
pub const UI_CAPABILITY_SCHEMA_VERSION: &str = "2026-08-13";

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

/// Truthful implementation status for an individual protocol capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UiCapabilityStatus {
    Supported,
    Partial,
    HostOwned,
    Candidate,
    Planned,
    Legacy,
}

/// Layer responsible for an individual capability claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UiCapabilityScope {
    Runtime,
    Client,
    Transport,
    Host,
    Security,
}

/// Maturity and implementation status for one upstream protocol version.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiProtocolVersionSpec {
    pub version: &'static str,
    pub spec_track: UiProtocolSpecTrack,
    pub status: UiCapabilityStatus,
    pub specification_url: &'static str,
}

/// How clients and servers discover or negotiate a protocol profile.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiProtocolNegotiationSpec {
    pub mode: &'static str,
    pub capability_keys: &'static [&'static str],
    pub content_types: &'static [&'static str],
    pub transports: &'static [&'static str],
    pub fallback_protocols: &'static [&'static str],
}

/// Machine-readable, scoped capability claim. Unlike `features`, this makes
/// partial and host-owned behavior explicit so clients can adapt safely.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiProtocolCapabilityClaim {
    pub id: &'static str,
    pub status: UiCapabilityStatus,
    pub scope: UiCapabilityScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<&'static str>,
}

/// Static capability contract for each supported UI protocol.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiProtocolCapabilitySpec {
    pub protocol: &'static str,
    pub versions: &'static [&'static str],
    pub implementation_tier: UiProtocolImplementationTier,
    pub spec_track: UiProtocolSpecTrack,
    pub reviewed_on: &'static str,
    pub version_details: &'static [UiProtocolVersionSpec],
    pub negotiation: UiProtocolNegotiationSpec,
    pub capability_claims: &'static [UiProtocolCapabilityClaim],
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
        reviewed_on: UI_CAPABILITY_SCHEMA_VERSION,
        version_details: &[UiProtocolVersionSpec {
            version: "1.0",
            spec_track: UiProtocolSpecTrack::Internal,
            status: UiCapabilityStatus::Legacy,
            specification_url: "https://github.com/zavora-ai/adk-ui",
        }],
        negotiation: UiProtocolNegotiationSpec {
            mode: "profile_selection",
            capability_keys: &["x-adk-ui-protocol", "uiProtocol"],
            content_types: &["application/json", "text/event-stream"],
            transports: &["http", "sse"],
            fallback_protocols: &[],
        },
        capability_claims: &[
            UiProtocolCapabilityClaim {
                id: "legacy_surface_runtime",
                status: UiCapabilityStatus::Legacy,
                scope: UiCapabilityScope::Runtime,
                note: Some("Retained only for compatibility through the published sunset window."),
            },
            UiProtocolCapabilityClaim {
                id: "trusted_component_catalog",
                status: UiCapabilityStatus::Supported,
                scope: UiCapabilityScope::Security,
                note: None,
            },
        ],
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
        versions: &["0.9.1", "1.0"],
        implementation_tier: UiProtocolImplementationTier::NativeSubset,
        spec_track: UiProtocolSpecTrack::Stable,
        reviewed_on: UI_CAPABILITY_SCHEMA_VERSION,
        version_details: &[
            UiProtocolVersionSpec {
                version: "0.9.1",
                spec_track: UiProtocolSpecTrack::Stable,
                status: UiCapabilityStatus::Supported,
                specification_url: "https://a2ui.org/specification/v0.9.1-a2ui/",
            },
            UiProtocolVersionSpec {
                version: "1.0",
                spec_track: UiProtocolSpecTrack::Candidate,
                status: UiCapabilityStatus::Candidate,
                specification_url: "https://a2ui.org/specification/v1.0-a2ui/",
            },
        ],
        negotiation: UiProtocolNegotiationSpec {
            mode: "catalog_capability_exchange",
            capability_keys: &[
                "a2uiClientCapabilities.v0.9",
                "a2uiRendererCapabilities.v1.0",
            ],
            content_types: &["application/a2ui+json", "application/json+a2ui"],
            transports: &["jsonl", "sse", "a2a_host_binding"],
            fallback_protocols: &["adk_ui"],
        },
        capability_claims: &[
            UiProtocolCapabilityClaim {
                id: "surface_lifecycle",
                status: UiCapabilityStatus::Supported,
                scope: UiCapabilityScope::Runtime,
                note: Some("create, update components/data, and delete are supported."),
            },
            UiProtocolCapabilityClaim {
                id: "catalog_negotiation",
                status: UiCapabilityStatus::Partial,
                scope: UiCapabilityScope::Transport,
                note: Some(
                    "Versioned capability shapes are emitted; transport-specific selection remains host-owned.",
                ),
            },
            UiProtocolCapabilityClaim {
                id: "v1_correlated_rpc",
                status: UiCapabilityStatus::Candidate,
                scope: UiCapabilityScope::Client,
                note: Some(
                    "actionResponse, callFunction, and functionResponse are implemented against the v1.0 release candidate.",
                ),
            },
            UiProtocolCapabilityClaim {
                id: "declarative_security",
                status: UiCapabilityStatus::Supported,
                scope: UiCapabilityScope::Security,
                note: Some(
                    "Generated payloads are constrained to trusted component and function catalogs.",
                ),
            },
            UiProtocolCapabilityClaim {
                id: "a2a_transport_binding",
                status: UiCapabilityStatus::HostOwned,
                scope: UiCapabilityScope::Transport,
                note: None,
            },
        ],
        summary: "A2UI v0.9.1 production surfaces plus an explicitly candidate v1.0 RPC subset, with versioned catalog capability exchange and declarative rendering.",
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
        reviewed_on: UI_CAPABILITY_SCHEMA_VERSION,
        version_details: &[UiProtocolVersionSpec {
            version: "0.1",
            spec_track: UiProtocolSpecTrack::Stable,
            status: UiCapabilityStatus::Supported,
            specification_url: "https://docs.ag-ui.com/",
        }],
        negotiation: UiProtocolNegotiationSpec {
            mode: "dynamic_discovery",
            capability_keys: &["getCapabilities", "agentCapabilities"],
            content_types: &["text/event-stream", "application/json"],
            transports: &["sse", "http"],
            fallback_protocols: &["adk_ui"],
        },
        capability_claims: &[
            UiProtocolCapabilityClaim {
                id: "typed_agent_capabilities",
                status: UiCapabilityStatus::Partial,
                scope: UiCapabilityScope::Runtime,
                note: Some(
                    "The example emits a truthful server baseline; loaded-agent tool, modality, sub-agent, and limit derivation remains framework-owned.",
                ),
            },
            UiProtocolCapabilityClaim {
                id: "event_stream",
                status: UiCapabilityStatus::Supported,
                scope: UiCapabilityScope::Transport,
                note: None,
            },
            UiProtocolCapabilityClaim {
                id: "state_and_activity_sync",
                status: UiCapabilityStatus::Supported,
                scope: UiCapabilityScope::Client,
                note: None,
            },
            UiProtocolCapabilityClaim {
                id: "encrypted_reasoning_continuity",
                status: UiCapabilityStatus::Supported,
                scope: UiCapabilityScope::Client,
                note: Some(
                    "Opaque REASONING_ENCRYPTED_VALUE events are preserved without exposing chain-of-thought.",
                ),
            },
            UiProtocolCapabilityClaim {
                id: "interrupt_resume",
                status: UiCapabilityStatus::Partial,
                scope: UiCapabilityScope::Runtime,
                note: Some(
                    "Wire types and resume ingestion are supported; durable pause/checkpoint policy is agent-owned.",
                ),
            },
            UiProtocolCapabilityClaim {
                id: "resumable_stream_sequence",
                status: UiCapabilityStatus::Planned,
                scope: UiCapabilityScope::Transport,
                note: None,
            },
        ],
        summary: "AG-UI event streaming with a typed capability model, state/activity synchronization, encrypted reasoning continuity, and interrupt-aware wire contracts.",
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
            "reasoning_encrypted_value",
            "agent_capability_discovery",
            "interrupt_resume_wire_types",
            "custom_surface_event",
            "push_notification_channel",
            "surface_activity_snapshot",
            "run_error",
            "raw_events",
        ],
        limitations: &[
            "Example hosts may still wrap some ADK runtime events; prefer protocol_native SSE when available from adk-server.",
            "Durable checkpointing and approval policy for interrupt/resume remain application-defined.",
            "WebSocket, binary HTTP, and resumable sequence transports are not implemented by the example server; hosts must bridge the agent notification channel to their chosen push transport.",
        ],
        deprecation: None,
    },
    UiProtocolCapabilitySpec {
        protocol: "mcp_apps",
        versions: &["2026-01-26", "2025-11-21"],
        implementation_tier: UiProtocolImplementationTier::NativeSubset,
        spec_track: UiProtocolSpecTrack::Stable,
        reviewed_on: UI_CAPABILITY_SCHEMA_VERSION,
        version_details: &[
            UiProtocolVersionSpec {
                version: "2026-01-26",
                spec_track: UiProtocolSpecTrack::Stable,
                status: UiCapabilityStatus::Supported,
                specification_url: "https://apps.extensions.modelcontextprotocol.io/",
            },
            UiProtocolVersionSpec {
                version: "2025-11-21",
                spec_track: UiProtocolSpecTrack::Candidate,
                status: UiCapabilityStatus::Legacy,
                specification_url: "https://github.com/modelcontextprotocol/ext-apps",
            },
        ],
        negotiation: UiProtocolNegotiationSpec {
            mode: "mcp_extension_negotiation",
            capability_keys: &["capabilities.extensions.io.modelcontextprotocol/ui.mimeTypes"],
            content_types: &["text/html;profile=mcp-app"],
            transports: &["mcp_jsonrpc", "postmessage", "sandbox_proxy"],
            fallback_protocols: &["text", "structured_content"],
        },
        capability_claims: &[
            UiProtocolCapabilityClaim {
                id: "progressive_enhancement",
                status: UiCapabilityStatus::Supported,
                scope: UiCapabilityScope::Runtime,
                note: Some(
                    "UI-enabled tools retain meaningful text and structured-content fallbacks.",
                ),
            },
            UiProtocolCapabilityClaim {
                id: "extension_mime_negotiation",
                status: UiCapabilityStatus::Partial,
                scope: UiCapabilityScope::Transport,
                note: Some(
                    "Capability shapes are emitted; host registration policy remains integration-owned.",
                ),
            },
            UiProtocolCapabilityClaim {
                id: "app_lifecycle",
                status: UiCapabilityStatus::Partial,
                scope: UiCapabilityScope::Host,
                note: Some(
                    "Initialize, initialized, message, and model-context flows are implemented; teardown/display-mode flows are typed but host-owned.",
                ),
            },
            UiProtocolCapabilityClaim {
                id: "sandbox_and_csp",
                status: UiCapabilityStatus::HostOwned,
                scope: UiCapabilityScope::Security,
                note: Some(
                    "Resource requirements are emitted and validated; the embedding host must enforce iframe isolation and CSP.",
                ),
            },
            UiProtocolCapabilityClaim {
                id: "tool_lifecycle_notifications",
                status: UiCapabilityStatus::Partial,
                scope: UiCapabilityScope::Client,
                note: Some(
                    "Tool input/result shapes are supported; complete postMessage orchestration remains host-owned.",
                ),
            },
            UiProtocolCapabilityClaim {
                id: "host_mediated_downloads_and_links",
                status: UiCapabilityStatus::HostOwned,
                scope: UiCapabilityScope::Host,
                note: None,
            },
        ],
        summary: "MCP Apps stable 2026-01-26 resource, negotiation, sandbox metadata, progressive fallback, and interactive app lifecycle subset.",
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
            "progressive_enhancement_fallback",
            "tool_lifecycle_notifications",
            "host_context_changed",
            "resource_teardown",
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
    reviewed_on: UI_CAPABILITY_SCHEMA_VERSION,
    version_details: &[UiProtocolVersionSpec {
        version: "1.0",
        spec_track: UiProtocolSpecTrack::Draft,
        status: UiCapabilityStatus::Partial,
        specification_url: "https://github.com/zavora-ai/adk-rust",
    }],
    negotiation: UiProtocolNegotiationSpec {
        mode: "capability_manifest",
        capability_keys: &["capabilities"],
        content_types: &["text/html", "application/json"],
        transports: &["http"],
        fallback_protocols: &["adk_ui"],
    },
    capability_claims: &[
        UiProtocolCapabilityClaim {
            id: "capability_manifest",
            status: UiCapabilityStatus::Supported,
            scope: UiCapabilityScope::Runtime,
            note: None,
        },
        UiProtocolCapabilityClaim {
            id: "interactive_hydration",
            status: UiCapabilityStatus::HostOwned,
            scope: UiCapabilityScope::Client,
            note: None,
        },
    ],
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
            assert_eq!(spec.reviewed_on, UI_CAPABILITY_SCHEMA_VERSION);
            assert!(
                !spec.version_details.is_empty(),
                "missing version details for {}",
                spec.protocol
            );
            assert!(
                !spec.capability_claims.is_empty(),
                "missing scoped claims for {}",
                spec.protocol
            );
            assert!(
                !spec.negotiation.transports.is_empty(),
                "missing transports for {}",
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
        assert!(a2ui.version_details.iter().any(|version| {
            version.version == "1.0"
                && version.spec_track == UiProtocolSpecTrack::Candidate
                && version.status == UiCapabilityStatus::Candidate
        }));

        let mcp_apps = UI_PROTOCOL_CAPABILITIES
            .iter()
            .find(|s| s.protocol == "mcp_apps")
            .unwrap();
        assert_eq!(mcp_apps.versions.first(), Some(&"2026-01-26"));

        let ag_ui = UI_PROTOCOL_CAPABILITIES
            .iter()
            .find(|s| s.protocol == "ag_ui")
            .unwrap();
        assert!(ag_ui.capability_claims.iter().any(|claim| {
            claim.id == "interrupt_resume" && claim.status == UiCapabilityStatus::Partial
        }));
    }

    #[test]
    fn preferred_protocol_is_a2ui() {
        assert_eq!(UI_PREFERRED_PROTOCOL, "a2ui");
        assert!(normalize_runtime_ui_protocol(UI_PREFERRED_PROTOCOL).is_some());
    }
}
