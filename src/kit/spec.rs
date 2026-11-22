//! A brand kit's formats: the brand inputs a draft is made from ([`KitSpec`]) and the manifest
//! a host reviews, approves and applies ([`BrandKitManifest`]).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fmt;

/// The `schemaVersion` of the manifests this crate writes, and the only one
/// [`BrandKitManifest::validate`] accepts.
pub const BRAND_KIT_SCHEMA_VERSION: &str = "1.0";

/// The brand inputs [`KitGenerator::generate`](super::KitGenerator::generate) drafts a kit
/// from.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitSpec {
    /// The kit's name; its slug goes into the kit's id. `Untitled Kit` when blank.
    pub name: String,
    /// The kit's version, the end of its id. `0.1.0` when blank.
    pub version: String,
    /// The brand's character, copied to the manifest.
    pub brand: KitBrand,
    /// The brand colours the kit's colours are derived from.
    pub colors: KitColors,
    /// The brand's type.
    pub typography: KitTypography,
    /// How tightly the kit spaces things; it sets the spacing unit.
    #[serde(default)]
    pub density: KitDensity,
    /// How rounded corners are.
    #[serde(default)]
    pub radius: KitRadius,
    /// The brand's component styles, which become the variants of the kit's recipes.
    #[serde(default)]
    pub components: KitComponents,
    /// Existing company assets to include in the draft. Generated assets should be reviewed
    /// and then supplied here with provenance before approval.
    #[serde(default)]
    pub assets: Vec<KitAsset>,
    /// Records where the supplied brand inputs came from.
    #[serde(default)]
    pub provenance: KitProvenance,
    /// Ids of the templates the manifest lists; blank ones are dropped.
    #[serde(default)]
    pub templates: Vec<String>,
}

/// The brand's character and industry.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitBrand {
    /// Words for the brand's character, such as `direct, dependable, operational`.
    pub vibe: String,
    /// The brand's industry, such as `logistics`.
    #[serde(default)]
    pub industry: Option<String>,
}

/// The brand colours. The generator derives the kit's colours from them and puts a default in
/// place of any value that is not a CSS colour.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitColors {
    /// The main brand colour.
    pub primary: String,
    /// A second brand colour, for the kit's secondary colour and focus ring; the primary when
    /// absent.
    #[serde(default)]
    pub accent: Option<String>,
    /// The colour of cards and panels.
    #[serde(default)]
    pub surface: Option<String>,
    /// The page background.
    #[serde(default)]
    pub background: Option<String>,
    /// The text colour on the background and on surfaces.
    #[serde(default)]
    pub text: Option<String>,
}

/// The brand's type.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitTypography {
    /// The font family for body text and headings.
    pub family: String,
    /// A type scale, carried over to the kit's typography as given.
    #[serde(default)]
    pub scale: Option<String>,
}

/// How much space a kit puts between things. The generator turns it into the spacing unit:
/// 3px, 4px or 6px.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum KitDensity {
    /// Tight spacing.
    Compact,
    /// Medium spacing; the default.
    #[default]
    Comfortable,
    /// Loose spacing.
    Spacious,
}

/// How rounded corners are: the `--radius` a kit sets.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum KitRadius {
    /// Square corners, `0px`.
    None,
    /// Small, `4px`.
    Sm,
    /// Medium, `8px`; the default.
    #[default]
    Md,
    /// Large, `12px`.
    Lg,
    /// Extra large, `18px`.
    Xl,
}

/// The brand's component styles, which become the variants of the kit's recipes.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct KitComponents {
    /// Button variants; the first is the `Button` recipe's variant.
    #[serde(default)]
    pub button: Option<KitComponentButton>,
    /// Card style, the `Card` recipe's variant.
    #[serde(default)]
    pub card: Option<KitComponentCard>,
    /// Text field style, the `TextField` recipe's variant.
    #[serde(default)]
    pub input: Option<KitComponentInput>,
    /// Table style. The generator makes no table recipe.
    #[serde(default)]
    pub table: Option<KitComponentTable>,
}

/// The brand's buttons.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitComponentButton {
    /// Button variant names; the first is the `Button` recipe's variant.
    #[serde(default)]
    pub variants: Vec<String>,
}

/// The brand's cards.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitComponentCard {
    /// The card elevation, used as the `Card` recipe's variant.
    #[serde(default)]
    pub elevation: Option<String>,
}

/// The brand's text fields.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitComponentInput {
    /// The field style, used as the `TextField` recipe's variant.
    #[serde(default)]
    pub style: Option<String>,
}

/// The brand's tables.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitComponentTable {
    /// Whether table rows are striped.
    #[serde(default)]
    pub striped: Option<bool>,
}

/// Whether a kit is a draft, approved or deprecated. Only an approved kit applies.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KitStatus {
    /// Not approved yet. Every generated kit starts here.
    #[default]
    Draft,
    /// Reviewed and approved; the only status a host applies. Needs `reviewedBy` and
    /// `approvedAt` in the kit's provenance.
    Approved,
    /// No longer to be applied.
    Deprecated,
}

/// How the kit was assembled. This is provenance, not a trust decision.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KitSource {
    /// Taken from the company's existing brand material.
    Imported,
    /// Made by a generator; the default.
    #[default]
    Generated,
    /// Partly imported, partly generated.
    Hybrid,
}

/// Where a kit came from, and who approved it and when.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct KitProvenance {
    /// How the kit was assembled.
    #[serde(default)]
    pub source: KitSource,
    /// Where the brand inputs came from, as an `https://` URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_uri: Option<String>,
    /// What generated the kit. The generator sets `adk-ui/KitGenerator` when it is empty and
    /// the source is generated or hybrid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_by: Option<String>,
    /// Who reviewed and approved the kit; an approved kit needs it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_by: Option<String>,
    /// When the kit was approved; an approved kit needs it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<String>,
}

/// What a brand asset is.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KitAssetKind {
    /// A logo.
    Logo,
    /// An icon.
    Icon,
    /// An illustration.
    Illustration,
    /// Any other image.
    Image,
    /// A font file; it needs no alternative text.
    Font,
    /// A 3D model, at most 25 MiB.
    Model3d,
}

/// A reviewed brand asset for the host's own pages. A drawing never references one.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KitAsset {
    /// The asset's id, unique in the kit; a recipe's `assetId` refers to it.
    pub id: String,
    /// What the asset is.
    pub kind: KitAssetKind,
    /// Where the asset is: an `https://` URL or a root-relative path such as
    /// `/assets/logo.svg`.
    pub uri: String,
    /// The asset's media type: `image/png`, `image/jpeg`, `image/webp`, `image/avif`,
    /// `image/svg+xml`, `font/woff`, `font/woff2` or `model/gltf-binary`.
    pub mime_type: String,
    /// Alternative text; every asset but a font needs it.
    pub alt: String,
    /// The asset's SHA-256 digest, as 64 hexadecimal characters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// The license the asset is used under.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// The asset's width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// The asset's height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// The asset's size in bytes; a 3D model may be at most 25 MiB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_size: Option<u64>,
}

/// The kit's colours by role. Each must be a hex, `rgb()`, `rgba()`, `hsl()`, `hsla()` or
/// `var(--name)` colour. [`ApprovedKit`](super::ApprovedKit) sets the adk-surface tokens named
/// on a field and nothing from the others.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KitSemanticColors {
    /// The main brand colour; sets `--accent`.
    pub primary: String,
    /// Text on the primary colour.
    pub primary_foreground: String,
    /// The second brand colour.
    pub secondary: String,
    /// Text on the secondary colour.
    pub secondary_foreground: String,
    /// The page background; sets `--bg`.
    pub background: String,
    /// Text on the background.
    pub foreground: String,
    /// Cards and panels; sets `--card`.
    pub surface: String,
    /// Text on surfaces; sets `--text`.
    pub surface_foreground: String,
    /// Subdued areas; sets `--raised` and `--chart-grid`.
    pub muted: String,
    /// Secondary text; sets `--text-2`, `--label` and `--chart-axis`.
    pub muted_foreground: String,
    /// Borders; sets `--border`.
    pub border: String,
    /// The borders of input fields.
    pub input: String,
    /// The focus ring; sets `--focus`.
    pub ring: String,
    /// Destructive actions and errors.
    pub destructive: String,
    /// Text on the destructive colour.
    pub destructive_foreground: String,
    /// Success.
    pub success: String,
    /// Warnings.
    pub warning: String,
    /// Information.
    pub info: String,
    /// Chart series colours, in order; the first eight set `--chart-1` to `--chart-8`.
    #[serde(default)]
    pub chart: Vec<String>,
}

/// The kit's font families and type scale.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KitSemanticTypography {
    /// The font family for body text; sets `--font`.
    pub body_family: String,
    /// The font family for headings.
    pub heading_family: String,
    /// The monospace font family; sets `--mono`.
    pub mono_family: String,
    /// A type scale, carried over from the spec.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<String>,
}

/// The kit's design tokens: colours, type, density, corner radius, spacing and motion.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KitSemanticTokens {
    /// Colours by role.
    pub colors: KitSemanticColors,
    /// Font families and type scale.
    pub typography: KitSemanticTypography,
    /// How tightly things are spaced.
    pub density: KitDensity,
    /// How rounded corners are; sets `--radius`.
    pub radius: KitRadius,
    /// Base spacing unit in pixels.
    pub spacing: u8,
    /// Default transition duration in milliseconds.
    pub motion_duration: u16,
}

/// How the brand styles one `forms` component, with an optional brand asset.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KitComponentRecipe {
    /// The component it styles, by its `forms` name, such as `TextField`; must not be empty.
    pub component: String,
    /// The variant the brand uses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    /// The id of a kit asset for the component; it must be one of the kit's assets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
    /// Named slot values. The generator leaves them empty.
    #[serde(default)]
    pub slots: BTreeMap<String, String>,
}

/// A template a kit lists.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KitTemplate {
    /// The template's id.
    pub id: String,
    /// What the template is for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The components the template needs.
    #[serde(default)]
    pub required_components: Vec<String>,
}

/// A versioned brand kit: the design tokens a host applies to its own pages once it is
/// approved. The model never picks one and a drawing never carries one.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BrandKitManifest {
    /// The manifest format version; [`validate`](Self::validate) accepts only
    /// [`BRAND_KIT_SCHEMA_VERSION`].
    pub schema_version: String,
    /// The kit's id; the generator writes `zavora.ai:adk-ui/kit/<name slug>@<version>`. It names
    /// the kit in `[data-adk-ui-kit]` and must not be empty.
    pub id: String,
    /// The kit's display name; must not be empty.
    pub name: String,
    /// The kit's version; must not be empty.
    pub version: String,
    /// Whether the kit is a draft, approved or deprecated. A host applies only an approved kit.
    #[serde(default)]
    pub status: KitStatus,
    /// Where the kit came from, and who approved it and when.
    #[serde(default)]
    pub provenance: KitProvenance,
    /// The brand's character and industry.
    pub brand: KitBrand,
    /// Brand assets for the host's own chrome, such as its header logo.
    #[serde(default)]
    pub assets: Vec<KitAsset>,
    /// The design tokens a host applies.
    pub tokens: KitSemanticTokens,
    /// Component recipes, keyed by component name.
    #[serde(default)]
    pub components: BTreeMap<String, KitComponentRecipe>,
    /// The templates the kit lists.
    #[serde(default)]
    pub templates: Vec<KitTemplate>,
}

/// A problem [`BrandKitManifest::validate`] found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KitValidationError {
    /// The field at fault, as a path such as `tokens.colors.primary` or `assets[0].uri`.
    pub field: String,
    /// What is wrong with it.
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
    /// Checks the manifest and returns every problem found: a schema version other than
    /// [`BRAND_KIT_SCHEMA_VERSION`]; a blank id, name or version; an approved kit without
    /// `reviewedBy` and `approvedAt`; a source URI that is not HTTPS; a colour that is not a CSS
    /// colour; an asset that breaks a rule given on [`KitAsset`]'s fields; and a recipe with no
    /// component or with an asset the kit does not declare.
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
