//! AWP protocol adapter.
//!
//! Converts a `UiSurface` into an AWP-compatible JSON payload containing
//! the component tree and optionally rendered HTML.

use crate::html::{BandwidthMode, HtmlRenderOptions, render_surface_html};
use crate::interop::adapter::UiProtocolAdapter;
use crate::interop::surface::{UiProtocol, UiSurface};
use serde_json::{Value, json};

/// AWP protocol adapter producing JSON payloads with optional HTML rendering.
pub struct AwpAdapter {
    bandwidth_mode: BandwidthMode,
    include_html: bool,
    class_prefix: Option<String>,
}

impl AwpAdapter {
    /// Create a new AwpAdapter with default settings (Full bandwidth, HTML included).
    pub fn new() -> Self {
        Self {
            bandwidth_mode: BandwidthMode::Full,
            include_html: true,
            class_prefix: None,
        }
    }

    /// Set the bandwidth mode for HTML rendering.
    pub fn with_bandwidth_mode(mut self, mode: BandwidthMode) -> Self {
        self.bandwidth_mode = mode;
        self
    }

    /// Control whether HTML rendering is included in the payload.
    pub fn with_include_html(mut self, include: bool) -> Self {
        self.include_html = include;
        self
    }

    /// Set a CSS class prefix for namespacing generated class names.
    pub fn with_class_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.class_prefix = Some(prefix.into());
        self
    }
}

impl Default for AwpAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl UiProtocolAdapter for AwpAdapter {
    fn protocol(&self) -> UiProtocol {
        UiProtocol::Awp
    }

    fn to_protocol_payload(&self, surface: &UiSurface) -> Result<Value, crate::compat::AdkError> {
        let mut payload = json!({
            "protocol": "awp",
            "surface_id": surface.surface_id,
            "components": surface.components,
        });

        if self.include_html {
            let options = HtmlRenderOptions {
                bandwidth_mode: self.bandwidth_mode,
                class_prefix: self.class_prefix.clone(),
            };
            let html = render_surface_html(surface, &options);
            payload["html"] = json!(html);
        }

        Ok(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_surface() -> UiSurface {
        UiSurface::new(
            "main",
            "catalog",
            vec![json!({"id": "root", "component": "Column", "children": []})],
        )
    }

    #[test]
    fn awp_adapter_protocol_returns_awp() {
        let adapter = AwpAdapter::new();
        assert_eq!(adapter.protocol(), UiProtocol::Awp);
    }

    #[test]
    fn awp_adapter_payload_contains_required_fields() {
        let adapter = AwpAdapter::new();
        let payload = adapter
            .to_protocol_payload(&test_surface())
            .expect("awp payload");
        assert_eq!(payload["protocol"], "awp");
        assert_eq!(payload["surface_id"], "main");
        assert!(payload["components"].is_array());
        assert!(payload["html"].as_str().is_some());
    }

    #[test]
    fn awp_adapter_without_html_omits_html_field() {
        let adapter = AwpAdapter::new().with_include_html(false);
        let payload = adapter
            .to_protocol_payload(&test_surface())
            .expect("awp payload");
        assert_eq!(payload["protocol"], "awp");
        assert!(payload.get("html").is_none());
    }

    #[test]
    fn awp_adapter_with_low_bandwidth() {
        let adapter = AwpAdapter::new().with_bandwidth_mode(BandwidthMode::Low);
        let payload = adapter
            .to_protocol_payload(&test_surface())
            .expect("awp payload");
        let html = payload["html"].as_str().unwrap();
        assert!(!html.contains("style=\""));
    }

    #[test]
    fn awp_adapter_with_class_prefix() {
        let adapter = AwpAdapter::new().with_class_prefix("adk-");
        let payload = adapter
            .to_protocol_payload(&test_surface())
            .expect("awp payload");
        assert!(payload["html"].as_str().is_some());
    }
}
