use super::surface::UiSurface;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const MCP_APPS_HTML_MIME_TYPE: &str = "text/html;profile=mcp-app";
pub const MCP_APPS_PROTOCOL_VERSION: &str = "2025-11-21";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum McpToolVisibility {
    Model,
    App,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct McpUiResourceCsp {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_uri_domains: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PermissionGrant {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct McpUiPermissions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera: Option<PermissionGrant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub microphone: Option<PermissionGrant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geolocation: Option<PermissionGrant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clipboard_write: Option<PermissionGrant>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct McpUiResourceMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csp: Option<McpUiResourceCsp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<McpUiPermissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefers_border: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpResourceMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui: Option<McpUiResourceMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpUiResource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub mime_type: String,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<McpResourceMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct McpUiToolMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Vec<McpToolVisibility>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpToolMetaEnvelope {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui: Option<McpUiToolMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpToolMeta {
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<McpToolMetaEnvelope>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpUiResourceContent {
    pub uri: String,
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<McpResourceMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceReadResponse {
    pub contents: Vec<McpUiResourceContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsPartyInfo {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icons: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsAppCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_display_modes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsInitializeRequestParams {
    pub protocol_version: String,
    pub app_info: McpAppsPartyInfo,
    pub app_capabilities: McpAppsAppCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsInitializeRequest {
    pub method: String,
    pub params: McpAppsInitializeRequestParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsInitializeResult {
    pub protocol_version: String,
    pub host_info: McpAppsPartyInfo,
    pub host_capabilities: Value,
    pub host_context: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsBridgeMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_info: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_capabilities: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_context: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_capabilities: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_info: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialized: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsSurfacePayload {
    pub resource: McpUiResource,
    pub resource_read_response: McpResourceReadResponse,
    pub tool_meta: McpToolMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bridge: Option<McpAppsBridgeMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpAppsRenderOptions {
    #[serde(skip_serializing_if = "Option::is_none", alias = "resourceUri")]
    pub resource_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", alias = "resourceName")]
    pub resource_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", alias = "resourceDescription")]
    pub resource_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", alias = "prefersBorder")]
    pub prefers_border: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csp: Option<McpUiResourceCsp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<McpUiPermissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Vec<McpToolVisibility>>,
}

fn is_allowed_domain(domain: &str) -> bool {
    domain.starts_with("https://")
        || domain.starts_with("http://localhost")
        || domain.starts_with("http://127.0.0.1")
}

fn validate_domain_list(
    domains: Option<&Vec<String>>,
    field: &str,
) -> Result<(), crate::compat::AdkError> {
    let Some(domains) = domains else {
        return Ok(());
    };

    for domain in domains {
        if !is_allowed_domain(domain) {
            return Err(crate::compat::AdkError::Tool(format!(
                "Invalid mcp_apps option '{}': unsupported domain '{}'",
                field, domain
            )));
        }
    }
    Ok(())
}

pub fn validate_mcp_apps_render_options(
    options: &McpAppsRenderOptions,
) -> Result<(), crate::compat::AdkError> {
    if let Some(domain) = options.domain.as_deref() {
        if !is_allowed_domain(domain) {
            return Err(crate::compat::AdkError::Tool(format!(
                "Invalid mcp_apps option 'domain': unsupported domain '{}'",
                domain
            )));
        }
    }

    if let Some(csp) = &options.csp {
        validate_domain_list(csp.connect_domains.as_ref(), "csp.connect_domains")?;
        validate_domain_list(csp.resource_domains.as_ref(), "csp.resource_domains")?;
        validate_domain_list(csp.frame_domains.as_ref(), "csp.frame_domains")?;
        validate_domain_list(csp.base_uri_domains.as_ref(), "csp.base_uri_domains")?;
    }

    Ok(())
}

fn sanitize_resource_token(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '/' {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "surface".to_string()
    } else {
        trimmed.to_string()
    }
}

fn build_inline_html(surface: &UiSurface) -> String {
    let payload = serde_json::to_string(surface).unwrap_or_else(|_| "{}".to_string());
    let escaped_payload = payload.replace("</script>", "<\\/script>");

    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>ADK UI Surface</title>
    <style>
      body {{ font-family: ui-sans-serif, system-ui, -apple-system, Segoe UI, sans-serif; margin: 0; padding: 1rem; }}
      pre {{ white-space: pre-wrap; background: #f6f8fa; border: 1px solid #d1d9e0; border-radius: 8px; padding: 0.75rem; }}
    </style>
  </head>
  <body>
    <h3>ADK UI Surface Payload</h3>
    <pre id="payload"></pre>
    <script id="adk-ui-surface" type="application/json">{}</script>
    <script>
      const raw = document.getElementById("adk-ui-surface")?.textContent ?? "{{}}";
      const parsed = JSON.parse(raw);
      document.getElementById("payload").textContent = JSON.stringify(parsed, null, 2);
    </script>
  </body>
</html>"#,
        escaped_payload
    )
}

pub fn default_mcp_apps_host_info() -> McpAppsPartyInfo {
    McpAppsPartyInfo {
        name: "adk-ui".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        title: Some("ADK UI Host".to_string()),
        description: Some("Compatibility host for ADK UI MCP Apps examples.".to_string()),
        website_url: Some("https://github.com/zavora-ai/adk-ui".to_string()),
        icons: None,
    }
}

pub fn default_mcp_apps_host_capabilities() -> Value {
    json!({
        "message": {
            "text": {},
            "structuredContent": {},
            "resource": {},
            "resourceLink": {},
        },
        "openLinks": {},
        "sandbox": {},
        "serverResources": {
            "listChanged": false,
        },
        "serverTools": {
            "listChanged": false,
        },
        "updateModelContext": {
            "text": {},
            "structuredContent": {},
        },
    })
}

pub fn default_mcp_apps_host_context(resource_uri: &str, domain: Option<&str>) -> Value {
    json!({
        "theme": "light",
        "locale": "en-US",
        "timeZone": "UTC",
        "platform": "web",
        "displayMode": "inline",
        "availableDisplayModes": ["inline"],
        "deviceCapabilities": {
            "hover": true,
            "touch": false,
        },
        "containerDimensions": {
            "maxWidth": 960,
            "maxHeight": 720,
        },
        "toolInfo": {
            "id": "adk-ui-example-tool",
            "tool": {
                "name": "render_screen",
                "title": "ADK UI render_screen",
                "description": "Renders an ADK UI screen surface.",
                "inputSchema": {
                    "type": "object",
                },
                "_meta": {
                    "ui": {
                        "resourceUri": resource_uri,
                    },
                },
            },
            "resourceUri": resource_uri,
            "domain": domain,
        },
        "userAgent": "adk-ui-example-host",
    })
}

pub fn build_default_mcp_apps_initialize_result(
    resource_uri: &str,
    domain: Option<&str>,
) -> McpAppsInitializeResult {
    McpAppsInitializeResult {
        protocol_version: MCP_APPS_PROTOCOL_VERSION.to_string(),
        host_info: default_mcp_apps_host_info(),
        host_capabilities: default_mcp_apps_host_capabilities(),
        host_context: default_mcp_apps_host_context(resource_uri, domain),
    }
}

fn build_bridge_metadata(
    surface: &UiSurface,
    resource_uri: &str,
    domain: Option<&str>,
) -> McpAppsBridgeMetadata {
    let initialize_result = build_default_mcp_apps_initialize_result(resource_uri, domain);
    McpAppsBridgeMetadata {
        protocol_version: Some(initialize_result.protocol_version.clone()),
        structured_content: Some(json!({
            "surface": surface,
        })),
        host_info: serde_json::to_value(initialize_result.host_info).ok(),
        host_capabilities: Some(initialize_result.host_capabilities),
        host_context: Some(initialize_result.host_context),
        app_capabilities: Some(json!({
            "availableDisplayModes": ["inline"],
            "tools": {
                "listChanged": false,
            },
        })),
        app_info: None,
        initialized: None,
    }
}

pub fn surface_to_mcp_apps_payload(
    surface: &UiSurface,
    options: McpAppsRenderOptions,
) -> McpAppsSurfacePayload {
    let resource_uri = options.resource_uri.unwrap_or_else(|| {
        format!(
            "ui://adk-rust/{}",
            sanitize_resource_token(surface.surface_id.as_str())
        )
    });
    let resource_name = options.resource_name.unwrap_or_else(|| {
        format!(
            "adk-ui-{}",
            sanitize_resource_token(surface.surface_id.as_str())
        )
    });
    let resource_description = options
        .resource_description
        .or_else(|| Some("ADK UI surface rendered via MCP Apps resource".to_string()));
    let bridge = build_bridge_metadata(surface, &resource_uri, options.domain.as_deref());

    let ui_meta = McpUiResourceMeta {
        csp: options.csp,
        permissions: options.permissions,
        domain: options.domain,
        prefers_border: options.prefers_border,
    };
    let meta = McpResourceMeta { ui: Some(ui_meta) };

    let resource = McpUiResource {
        uri: resource_uri.clone(),
        name: resource_name,
        description: resource_description,
        mime_type: MCP_APPS_HTML_MIME_TYPE.to_string(),
        meta: Some(meta.clone()),
    };

    let html = build_inline_html(surface);
    let resource_read_response = McpResourceReadResponse {
        contents: vec![McpUiResourceContent {
            uri: resource_uri.clone(),
            mime_type: MCP_APPS_HTML_MIME_TYPE.to_string(),
            text: Some(html),
            blob: None,
            meta: Some(meta),
        }],
    };

    let visibility = options
        .visibility
        .unwrap_or_else(|| vec![McpToolVisibility::Model, McpToolVisibility::App]);
    let tool_meta = McpToolMeta {
        meta: Some(McpToolMetaEnvelope {
            ui: Some(McpUiToolMeta {
                resource_uri: Some(resource_uri),
                visibility: Some(visibility),
            }),
        }),
    };

    McpAppsSurfacePayload {
        resource,
        resource_read_response,
        tool_meta,
        bridge: Some(bridge),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn payload_contains_ui_scheme_and_tool_binding() {
        let surface = UiSurface::new(
            "main",
            "catalog",
            vec![json!({"id":"root","component":{"Column":{"children":[]}}})],
        );
        let payload = surface_to_mcp_apps_payload(&surface, McpAppsRenderOptions::default());

        assert!(payload.resource.uri.starts_with("ui://"));
        let tool_meta_value = serde_json::to_value(payload.tool_meta).unwrap();
        assert_eq!(
            tool_meta_value["_meta"]["ui"]["resourceUri"]
                .as_str()
                .unwrap(),
            payload.resource.uri
        );
    }

    #[test]
    fn payload_contains_bridge_structured_surface() {
        let surface = UiSurface::new(
            "main",
            "catalog",
            vec![json!({"id":"root","component":{"Column":{"children":[]}}})],
        )
        .with_data_model(Some(json!({"phase":"ready"})));
        let payload = surface_to_mcp_apps_payload(&surface, McpAppsRenderOptions::default());
        let bridge = payload.bridge.as_ref().expect("bridge metadata");

        assert_eq!(
            bridge.structured_content.as_ref().unwrap()["surface"]["surfaceId"],
            "main"
        );
        assert_eq!(
            bridge.host_context.as_ref().unwrap()["displayMode"],
            "inline"
        );
        assert_eq!(
            bridge.protocol_version.as_deref(),
            Some(MCP_APPS_PROTOCOL_VERSION)
        );
    }

    #[test]
    fn payload_html_embeds_surface_json() {
        let surface = UiSurface::new(
            "main",
            "catalog",
            vec![json!({"id":"root","component":{"Column":{"children":[]}}})],
        );
        let payload = surface_to_mcp_apps_payload(&surface, McpAppsRenderOptions::default());
        let html = payload.resource_read_response.contents[0]
            .text
            .as_ref()
            .unwrap();
        assert!(html.contains("ADK UI Surface Payload"));
        assert!(html.contains("adk-ui-surface"));
    }

    #[test]
    fn validate_options_rejects_invalid_domain() {
        let options = McpAppsRenderOptions {
            domain: Some("ftp://example.com".to_string()),
            ..Default::default()
        };
        let result = validate_mcp_apps_render_options(&options);
        assert!(result.is_err());
    }

    #[test]
    fn validate_options_rejects_invalid_csp_domain() {
        let options = McpAppsRenderOptions {
            csp: Some(McpUiResourceCsp {
                connect_domains: Some(vec![
                    "https://example.com".to_string(),
                    "javascript:alert(1)".to_string(),
                ]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let result = validate_mcp_apps_render_options(&options);
        assert!(result.is_err());
    }
}
