//! Drafting a brand kit from brand inputs ([`KitGenerator`]).

use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::spec::{
    BRAND_KIT_SCHEMA_VERSION, BrandKitManifest, KitComponentRecipe, KitDensity, KitRadius,
    KitSemanticColors, KitSemanticTokens, KitSemanticTypography, KitSource, KitSpec, KitStatus,
    KitTemplate, is_css_color_token,
};

/// What a brand kit generates for a host: its manifest, its semantic tokens and the CSS that
/// applies them. A kit never defines a catalog: the components a drawing may use come from
/// adk-ui's catalogs alone, and the host, not the model, chooses the kit.
#[derive(Debug, Clone)]
pub struct KitArtifacts {
    /// The draft manifest, for the host to review and approve.
    pub manifest: BrandKitManifest,
    /// The manifest's semantic tokens, as JSON.
    pub tokens: Value,
    /// The CSS rule [`ApprovedKit::surface_css`](super::ApprovedKit::surface_css) gives for the
    /// kit, so a draft previews as it will look once approved.
    pub theme_css: String,
}

/// The components a kit's recipes style, by their `forms` names.
const RECIPE_COMPONENTS: &[&str] = &["Button", "Card", "TextField"];

/// Drafts brand kits from brand inputs.
#[derive(Debug, Default)]
pub struct KitGenerator;

impl KitGenerator {
    /// A generator.
    pub fn new() -> Self {
        Self
    }

    /// Drafts a kit from `spec`. Approving it is the host's step: set `status`, `reviewedBy`
    /// and `approvedAt`, then [`ApprovedKit::try_from`](super::ApprovedKit) checks it.
    pub fn generate(&self, spec: &KitSpec) -> KitArtifacts {
        let kit_name = non_empty(&spec.name, "Untitled Kit");
        let kit_version = non_empty(&spec.version, "0.1.0");
        let slug = {
            let value = slugify(kit_name);
            if value.is_empty() {
                "untitled-kit".to_string()
            } else {
                value
            }
        };
        let kit_id = format!("zavora.ai:adk-ui/kit/{slug}@{kit_version}");
        let recipes = component_recipes(spec);
        let semantic_tokens = semantic_tokens(spec);
        let templates = spec
            .templates
            .iter()
            .filter(|template| !template.trim().is_empty())
            .map(|template| KitTemplate {
                id: template.clone(),
                description: None,
                required_components: Vec::new(),
            })
            .collect::<Vec<_>>();
        let mut provenance = spec.provenance.clone();
        if provenance.generated_by.is_none()
            && matches!(provenance.source, KitSource::Generated | KitSource::Hybrid)
        {
            provenance.generated_by = Some("adk-ui/KitGenerator".to_string());
        }

        let manifest = BrandKitManifest {
            schema_version: BRAND_KIT_SCHEMA_VERSION.to_string(),
            id: kit_id.clone(),
            name: kit_name.to_string(),
            version: kit_version.to_string(),
            status: KitStatus::Draft,
            provenance,
            brand: spec.brand.clone(),
            assets: spec.assets.clone(),
            tokens: semantic_tokens.clone(),
            components: recipes,
            templates: templates.clone(),
        };

        let tokens = serde_json::to_value(&semantic_tokens).unwrap_or_else(|_| json!({}));
        let theme_css = theme_css(&kit_id, &semantic_tokens);

        KitArtifacts {
            manifest,
            tokens,
            theme_css,
        }
    }
}

fn component_recipes(spec: &KitSpec) -> BTreeMap<String, KitComponentRecipe> {
    RECIPE_COMPONENTS
        .iter()
        .map(|component| {
            let component = (*component).to_string();
            let variant = match component.as_str() {
                "Button" => spec
                    .components
                    .button
                    .as_ref()
                    .and_then(|button| button.variants.first().cloned()),
                "Card" => spec
                    .components
                    .card
                    .as_ref()
                    .and_then(|card| card.elevation.clone()),
                "TextField" => spec
                    .components
                    .input
                    .as_ref()
                    .and_then(|input| input.style.clone()),
                _ => None,
            };
            (
                component.clone(),
                KitComponentRecipe {
                    component: component.clone(),
                    variant,
                    asset_id: None,
                    slots: BTreeMap::new(),
                },
            )
        })
        .collect()
}

fn semantic_tokens(spec: &KitSpec) -> KitSemanticTokens {
    let primary = color_or(&spec.colors.primary, "#0f766e");
    let accent = spec
        .colors
        .accent
        .as_deref()
        .map(|value| color_or(value, &primary))
        .unwrap_or_else(|| primary.clone());
    let background = spec
        .colors
        .background
        .as_deref()
        .map(|value| color_or(value, "#f7f9f8"))
        .unwrap_or_else(|| "#f7f9f8".to_string());
    let surface = spec
        .colors
        .surface
        .as_deref()
        .map(|value| color_or(value, "#ffffff"))
        .unwrap_or_else(|| "#ffffff".to_string());
    let foreground = spec
        .colors
        .text
        .as_deref()
        .map(|value| color_or(value, "#17211f"))
        .unwrap_or_else(|| "#17211f".to_string());
    let body_family = non_empty(&spec.typography.family, "Avenir Next, sans-serif").to_string();

    KitSemanticTokens {
        colors: KitSemanticColors {
            primary_foreground: contrast_foreground(&primary),
            primary,
            secondary_foreground: contrast_foreground(&accent),
            secondary: accent.clone(),
            background,
            foreground: foreground.clone(),
            surface,
            surface_foreground: foreground,
            muted: "#edf2f1".to_string(),
            muted_foreground: "#5b6b68".to_string(),
            border: "#d8e1df".to_string(),
            input: "#c7d3d0".to_string(),
            ring: accent,
            destructive: "#b42318".to_string(),
            destructive_foreground: "#ffffff".to_string(),
            success: "#18794e".to_string(),
            warning: "#a15c00".to_string(),
            info: "#175cd3".to_string(),
            chart: vec![
                "#0f766e".to_string(),
                "#175cd3".to_string(),
                "#a15c00".to_string(),
                "#7a5af8".to_string(),
                "#c4320a".to_string(),
            ],
        },
        typography: KitSemanticTypography {
            body_family: body_family.clone(),
            heading_family: body_family,
            mono_family: "ui-monospace, SFMono-Regular, Menlo, monospace".to_string(),
            scale: spec.typography.scale.clone(),
        },
        density: spec.density.clone(),
        radius: spec.radius.clone(),
        spacing: density_spacing(&spec.density),
        motion_duration: 160,
    }
}

/// The CSS that applies `tokens` to a host's `DrawnView`s: the adk-surface tokens an approved
/// kit sets ([`ApprovedKit::surface_css`](super::ApprovedKit::surface_css)), so a draft
/// previews exactly as it will look once approved.
fn theme_css(kit_id: &str, tokens: &KitSemanticTokens) -> String {
    super::surface::surface_rule(kit_id, tokens)
}

fn density_spacing(density: &KitDensity) -> u8 {
    match density {
        KitDensity::Compact => 3,
        KitDensity::Comfortable => 4,
        KitDensity::Spacious => 6,
    }
}

pub(crate) fn radius_value(radius: &KitRadius) -> &'static str {
    match radius {
        KitRadius::None => "0px",
        KitRadius::Sm => "4px",
        KitRadius::Md => "8px",
        KitRadius::Lg => "12px",
        KitRadius::Xl => "18px",
    }
}

fn contrast_foreground(color: &str) -> String {
    let Some(hex) = color.strip_prefix('#').filter(|value| value.len() == 6) else {
        return "#ffffff".to_string();
    };
    let Ok(red) = u8::from_str_radix(&hex[0..2], 16) else {
        return "#ffffff".to_string();
    };
    let Ok(green) = u8::from_str_radix(&hex[2..4], 16) else {
        return "#ffffff".to_string();
    };
    let Ok(blue) = u8::from_str_radix(&hex[4..6], 16) else {
        return "#ffffff".to_string();
    };
    let luminance = (u32::from(red) * 299 + u32::from(green) * 587 + u32::from(blue) * 114) / 1000;
    if luminance > 155 {
        "#17211f".to_string()
    } else {
        "#ffffff".to_string()
    }
}

fn color_or(value: &str, fallback: &str) -> String {
    let value = value.trim();
    if is_css_color_token(value) {
        value.to_string()
    } else {
        fallback.to_string()
    }
}

pub(crate) fn css_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\n', '\r'], " ")
}

fn non_empty<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value.trim()
    }
}

fn slugify(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if (ch.is_whitespace() || ch == '-' || ch == '_') && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::spec::{KitBrand, KitColors, KitProvenance, KitSpec, KitTypography};

    fn spec() -> KitSpec {
        KitSpec {
            name: "Fintech Pro".to_string(),
            version: "0.1.0".to_string(),
            brand: KitBrand {
                vibe: "trustworthy".to_string(),
                industry: None,
            },
            colors: KitColors {
                primary: "#2F6BFF".to_string(),
                accent: None,
                surface: None,
                background: None,
                text: None,
            },
            typography: KitTypography {
                family: "Source Sans 3".to_string(),
                scale: None,
            },
            density: Default::default(),
            radius: Default::default(),
            components: Default::default(),
            assets: Vec::new(),
            provenance: KitProvenance::default(),
            templates: vec!["auth_login".to_string()],
        }
    }

    #[test]
    fn generates_a_draft_manifest_tokens_and_css_and_never_a_catalog() {
        let artifacts = KitGenerator::new().generate(&spec());
        assert_eq!(
            artifacts.manifest.id,
            "zavora.ai:adk-ui/kit/fintech-pro@0.1.0"
        );
        assert_eq!(artifacts.manifest.status, KitStatus::Draft);
        assert!(artifacts.manifest.validate().is_ok());
        assert_eq!(
            artifacts
                .manifest
                .components
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["Button", "Card", "TextField"]
        );
        assert!(artifacts.tokens["colors"]["primary"].is_string());
        assert!(
            artifacts.theme_css.contains("--accent: #2F6BFF;"),
            "{}",
            artifacts.theme_css
        );
        assert!(
            !artifacts.theme_css.contains("--adk-ui-"),
            "only the tokens a host reads"
        );
    }

    #[test]
    fn sanitizes_invalid_legacy_css_values() {
        let mut unsafe_spec = spec();
        unsafe_spec.colors.primary = "rgb(0, 0, 0); } body { display:none".to_string();
        unsafe_spec.typography.family = "Bad\"; color:red".to_string();

        let artifacts = KitGenerator::new().generate(&unsafe_spec);
        assert_eq!(artifacts.tokens["colors"]["primary"], "#0f766e");
        assert!(!artifacts.theme_css.contains("} body"));
        assert!(artifacts.theme_css.contains("Bad\\\"; color:red"));
    }
}
