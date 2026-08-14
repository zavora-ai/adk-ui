use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fmt;

pub const BRAND_KIT_SCHEMA_VERSION: &str = "1.0";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitSpec {
    pub name: String,
    pub version: String,
    pub brand: KitBrand,
    pub colors: KitColors,
    pub typography: KitTypography,
    #[serde(default)]
    pub density: KitDensity,
    #[serde(default)]
    pub radius: KitRadius,
    #[serde(default)]
    pub components: KitComponents,
    /// Existing company assets to include in the draft. Generated assets should be reviewed
    /// and then supplied here with provenance before approval.
    #[serde(default)]
    pub assets: Vec<KitAsset>,
    /// Records where the supplied brand inputs came from.
    #[serde(default)]
    pub provenance: KitProvenance,
    #[serde(default)]
    pub templates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitBrand {
    pub vibe: String,
    #[serde(default)]
    pub industry: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitColors {
    pub primary: String,
    #[serde(default)]
    pub accent: Option<String>,
    #[serde(default)]
    pub surface: Option<String>,
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitTypography {
    pub family: String,
    #[serde(default)]
    pub scale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum KitDensity {
    Compact,
    #[default]
    Comfortable,
    Spacious,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum KitRadius {
    None,
    Sm,
    #[default]
    Md,
    Lg,
    Xl,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct KitComponents {
    #[serde(default)]
    pub button: Option<KitComponentButton>,
    #[serde(default)]
    pub card: Option<KitComponentCard>,
    #[serde(default)]
    pub input: Option<KitComponentInput>,
    #[serde(default)]
    pub table: Option<KitComponentTable>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitComponentButton {
    #[serde(default)]
    pub variants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitComponentCard {
    #[serde(default)]
    pub elevation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitComponentInput {
    #[serde(default)]
    pub style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitComponentTable {
    #[serde(default)]
    pub striped: Option<bool>,
}

/// Publication state for a brand kit. Runtime hosts should only auto-select approved kits.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KitStatus {
    #[default]
    Draft,
    Approved,
    Deprecated,
}

/// How the kit was assembled. This is provenance, not a trust decision.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KitSource {
    Imported,
    #[default]
    Generated,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct KitProvenance {
    #[serde(default)]
    pub source: KitSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KitAssetKind {
    Logo,
    Icon,
    Illustration,
    Image,
    Font,
    Model3d,
}

/// An approved, addressable brand asset. Components reference these as `kit://<id>`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KitAsset {
    pub id: String,
    pub kind: KitAssetKind,
    pub uri: String,
    pub mime_type: String,
    pub alt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KitSemanticColors {
    pub primary: String,
    pub primary_foreground: String,
    pub secondary: String,
    pub secondary_foreground: String,
    pub background: String,
    pub foreground: String,
    pub surface: String,
    pub surface_foreground: String,
    pub muted: String,
    pub muted_foreground: String,
    pub border: String,
    pub input: String,
    pub ring: String,
    pub destructive: String,
    pub destructive_foreground: String,
    pub success: String,
    pub warning: String,
    pub info: String,
    #[serde(default)]
    pub chart: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KitSemanticTypography {
    pub body_family: String,
    pub heading_family: String,
    pub mono_family: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KitSemanticTokens {
    pub colors: KitSemanticColors,
    pub typography: KitSemanticTypography,
    pub density: KitDensity,
    pub radius: KitRadius,
    /// Base spacing unit in pixels.
    pub spacing: u8,
    /// Default transition duration in milliseconds.
    pub motion_duration: u16,
}

/// Maps an agent-facing component to a renderer recipe and optional branded asset.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KitComponentRecipe {
    pub component: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub slots: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KitTemplate {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub required_components: Vec<String>,
}

/// Versioned company design contract shared by agents, protocols, and renderers.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BrandKitManifest {
    pub schema_version: String,
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub status: KitStatus,
    #[serde(default)]
    pub provenance: KitProvenance,
    pub brand: KitBrand,
    #[serde(default)]
    pub assets: Vec<KitAsset>,
    pub tokens: KitSemanticTokens,
    #[serde(default)]
    pub components: BTreeMap<String, KitComponentRecipe>,
    #[serde(default)]
    pub templates: Vec<KitTemplate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KitValidationError {
    pub field: String,
    pub message: String,
}

impl KitValidationError {
    fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for KitValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.field, self.message)
    }
}

impl BrandKitManifest {
    pub fn validate(&self) -> Result<(), Vec<KitValidationError>> {
        let mut errors = Vec::new();

        if self.schema_version != BRAND_KIT_SCHEMA_VERSION {
            errors.push(KitValidationError::new(
                "schemaVersion",
                format!("expected {BRAND_KIT_SCHEMA_VERSION}"),
            ));
        }
        if self.id.trim().is_empty() {
            errors.push(KitValidationError::new("id", "must not be empty"));
        }
        if self.name.trim().is_empty() {
            errors.push(KitValidationError::new("name", "must not be empty"));
        }
        if self.version.trim().is_empty() {
            errors.push(KitValidationError::new("version", "must not be empty"));
        }
        if self.status == KitStatus::Approved
            && (self.provenance.reviewed_by.is_none() || self.provenance.approved_at.is_none())
        {
            errors.push(KitValidationError::new(
                "provenance",
                "approved kits require reviewedBy and approvedAt",
            ));
        }
        if let Some(source_uri) = &self.provenance.source_uri
            && !source_uri.trim().starts_with("https://")
        {
            errors.push(KitValidationError::new(
                "provenance.sourceUri",
                "must use HTTPS",
            ));
        }

        validate_colors(&self.tokens.colors, &mut errors);

        let mut asset_ids = HashSet::new();
        for (index, asset) in self.assets.iter().enumerate() {
            let field = format!("assets[{index}]");
            if asset.id.trim().is_empty() {
                errors.push(KitValidationError::new(
                    format!("{field}.id"),
                    "must not be empty",
                ));
            } else if !asset_ids.insert(asset.id.as_str()) {
                errors.push(KitValidationError::new(
                    format!("{field}.id"),
                    "must be unique",
                ));
            }
            if !is_safe_asset_uri(&asset.uri) {
                errors.push(KitValidationError::new(
                    format!("{field}.uri"),
                    "must be an HTTPS URL or a root-relative application asset",
                ));
            }
            if !is_supported_asset_mime(&asset.mime_type) {
                errors.push(KitValidationError::new(
                    format!("{field}.mimeType"),
                    "unsupported brand asset media type",
                ));
            }
            if asset.alt.trim().is_empty() && asset.kind != KitAssetKind::Font {
                errors.push(KitValidationError::new(
                    format!("{field}.alt"),
                    "non-font assets require alternative text",
                ));
            }
            if asset.kind == KitAssetKind::Model3d
                && asset.byte_size.unwrap_or(0) > 25 * 1024 * 1024
            {
                errors.push(KitValidationError::new(
                    format!("{field}.byteSize"),
                    "3D model assets must not exceed 25 MiB",
                ));
            }
            if let Some(sha256) = &asset.sha256
                && (sha256.len() != 64
                    || !sha256
                        .chars()
                        .all(|character| character.is_ascii_hexdigit()))
            {
                errors.push(KitValidationError::new(
                    format!("{field}.sha256"),
                    "must be a 64-character hexadecimal digest",
                ));
            }
        }

        for (name, recipe) in &self.components {
            if recipe.component.trim().is_empty() {
                errors.push(KitValidationError::new(
                    format!("components.{name}.component"),
                    "must not be empty",
                ));
            }
            if let Some(asset_id) = &recipe.asset_id
                && !asset_ids.contains(asset_id.as_str())
            {
                errors.push(KitValidationError::new(
                    format!("components.{name}.assetId"),
                    "must reference an asset declared by this kit",
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn validate_colors(colors: &KitSemanticColors, errors: &mut Vec<KitValidationError>) {
    let values = [
        ("primary", &colors.primary),
        ("primaryForeground", &colors.primary_foreground),
        ("secondary", &colors.secondary),
        ("secondaryForeground", &colors.secondary_foreground),
        ("background", &colors.background),
        ("foreground", &colors.foreground),
        ("surface", &colors.surface),
        ("surfaceForeground", &colors.surface_foreground),
        ("muted", &colors.muted),
        ("mutedForeground", &colors.muted_foreground),
        ("border", &colors.border),
        ("input", &colors.input),
        ("ring", &colors.ring),
        ("destructive", &colors.destructive),
        ("destructiveForeground", &colors.destructive_foreground),
        ("success", &colors.success),
        ("warning", &colors.warning),
        ("info", &colors.info),
    ];

    for (name, value) in values {
        if !is_css_color_token(value) {
            errors.push(KitValidationError::new(
                format!("tokens.colors.{name}"),
                "must be a hex, rgb(), rgba(), hsl(), hsla(), or CSS variable color",
            ));
        }
    }
    for (index, value) in colors.chart.iter().enumerate() {
        if !is_css_color_token(value) {
            errors.push(KitValidationError::new(
                format!("tokens.colors.chart[{index}]"),
                "must be a supported CSS color",
            ));
        }
    }
}

pub(crate) fn is_css_color_token(value: &str) -> bool {
    let value = value.trim();
    let valid_hex = value.strip_prefix('#').is_some_and(|hex| {
        matches!(hex.len(), 3 | 4 | 6 | 8) && hex.chars().all(|ch| ch.is_ascii_hexdigit())
    });
    if valid_hex {
        return true;
    }

    if let Some(variable) = value
        .strip_prefix("var(")
        .and_then(|token| token.strip_suffix(')'))
    {
        return variable.starts_with("--")
            && variable.len() > 2
            && variable.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
            });
    }

    ["rgb(", "rgba(", "hsl(", "hsla("].iter().any(|prefix| {
        value
            .strip_prefix(prefix)
            .and_then(|token| token.strip_suffix(')'))
            .is_some_and(|arguments| {
                !arguments.is_empty()
                    && arguments.chars().all(|character| {
                        character.is_ascii_digit()
                            || character.is_ascii_whitespace()
                            || matches!(character, '.' | ',' | '%' | '/' | '+' | '-')
                    })
            })
    })
}

fn is_safe_asset_uri(uri: &str) -> bool {
    let uri = uri.trim();
    uri.starts_with("https://") || (uri.starts_with('/') && !uri.starts_with("//"))
}

fn is_supported_asset_mime(mime: &str) -> bool {
    matches!(
        mime.trim().to_ascii_lowercase().as_str(),
        "image/png"
            | "image/jpeg"
            | "image/webp"
            | "image/avif"
            | "image/svg+xml"
            | "font/woff"
            | "font/woff2"
            | "model/gltf-binary"
    )
}

#[cfg(test)]
mod manifest_tests {
    use super::*;

    fn valid_manifest() -> BrandKitManifest {
        BrandKitManifest {
            schema_version: BRAND_KIT_SCHEMA_VERSION.to_string(),
            id: "zavora.ai:adk-ui/kit/acme@1.0.0".to_string(),
            name: "Acme".to_string(),
            version: "1.0.0".to_string(),
            status: KitStatus::Draft,
            provenance: KitProvenance::default(),
            brand: KitBrand {
                vibe: "clear".to_string(),
                industry: None,
            },
            assets: vec![KitAsset {
                id: "logo".to_string(),
                kind: KitAssetKind::Logo,
                uri: "/assets/logo.svg".to_string(),
                mime_type: "image/svg+xml".to_string(),
                alt: "Acme".to_string(),
                sha256: None,
                license: None,
                width: None,
                height: None,
                byte_size: None,
            }],
            tokens: KitSemanticTokens {
                colors: KitSemanticColors {
                    primary: "#00695c".to_string(),
                    primary_foreground: "#ffffff".to_string(),
                    secondary: "#e7f4f1".to_string(),
                    secondary_foreground: "#102a27".to_string(),
                    background: "#f8faf9".to_string(),
                    foreground: "#102a27".to_string(),
                    surface: "#ffffff".to_string(),
                    surface_foreground: "#102a27".to_string(),
                    muted: "#eef2f1".to_string(),
                    muted_foreground: "#5b6b68".to_string(),
                    border: "#d9e2e0".to_string(),
                    input: "#d9e2e0".to_string(),
                    ring: "#00695c".to_string(),
                    destructive: "#b42318".to_string(),
                    destructive_foreground: "#ffffff".to_string(),
                    success: "#18794e".to_string(),
                    warning: "#a15c00".to_string(),
                    info: "#175cd3".to_string(),
                    chart: vec!["#00695c".to_string()],
                },
                typography: KitSemanticTypography {
                    body_family: "Source Sans 3".to_string(),
                    heading_family: "Source Sans 3".to_string(),
                    mono_family: "ui-monospace".to_string(),
                    scale: None,
                },
                density: KitDensity::Comfortable,
                radius: KitRadius::Md,
                spacing: 4,
                motion_duration: 160,
            },
            components: BTreeMap::new(),
            templates: Vec::new(),
        }
    }

    #[test]
    fn validates_safe_manifest() {
        assert!(valid_manifest().validate().is_ok());
    }

    #[test]
    fn rejects_unsafe_assets_and_unreviewed_approval() {
        let mut manifest = valid_manifest();
        manifest.status = KitStatus::Approved;
        manifest.assets[0].uri = "javascript:alert(1)".to_string();

        let errors = manifest.validate().expect_err("invalid manifest");
        assert!(errors.iter().any(|error| error.field == "provenance"));
        assert!(errors.iter().any(|error| error.field == "assets[0].uri"));
    }

    #[test]
    fn rejects_css_function_injection() {
        let mut manifest = valid_manifest();
        manifest.tokens.colors.primary = "rgb(0, 0, 0); } body { display:none".to_string();

        let errors = manifest.validate().expect_err("unsafe color");
        assert!(
            errors
                .iter()
                .any(|error| error.field == "tokens.colors.primary")
        );
    }
}
