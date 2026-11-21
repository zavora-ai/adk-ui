//! Applying an approved brand kit to a custom host: the adk-surface tokens its `DrawnView` and
//! adk-ui's renderers read, and the A2UI theme a third-party renderer takes. A kit styles the
//! host's own pages only: the ADK-Rust web home keeps its tokens, a drawing never carries a
//! theme, and no kit restyles a decision.

use serde_json::{Value, json};

use super::generator::{css_string, radius_value};
use super::spec::{
    BrandKitManifest, KitSemanticTokens, KitStatus, KitValidationError, is_css_color_token,
};

/// A brand kit a host may apply: approved, with its reviewer and approval date, and valid.
///
/// ```rust
/// use adk_ui::kit::{ApprovedKit, KitGenerator, KitSpec};
///
/// let draft = KitGenerator::new().generate(&serde_json::from_value::<KitSpec>(serde_json::json!({
///     "name": "Fintech Pro",
///     "version": "0.1.0",
///     "brand": { "vibe": "trustworthy" },
///     "colors": { "primary": "#2F6BFF" },
///     "typography": { "family": "Inter" }
/// })).unwrap()).manifest;
/// assert!(ApprovedKit::try_from(draft).is_err(), "a draft is never applied");
/// ```
#[derive(Debug, Clone)]
pub struct ApprovedKit(BrandKitManifest);

/// Why a kit cannot be applied.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum KitNotApproved {
    /// It is a draft or deprecated.
    #[error("kit {0} is not approved")]
    Status(String),
    /// It does not validate.
    #[error("kit {id} is not valid: {errors:?}")]
    Invalid {
        /// The kit's id.
        id: String,
        /// What is wrong.
        errors: Vec<String>,
    },
}

impl TryFrom<BrandKitManifest> for ApprovedKit {
    type Error = KitNotApproved;

    fn try_from(manifest: BrandKitManifest) -> Result<Self, Self::Error> {
        if manifest.status != KitStatus::Approved {
            return Err(KitNotApproved::Status(manifest.id));
        }
        manifest
            .validate()
            .map_err(|errors: Vec<KitValidationError>| KitNotApproved::Invalid {
                id: manifest.id.clone(),
                errors: errors.iter().map(ToString::to_string).collect(),
            })?;
        Ok(Self(manifest))
    }
}

impl ApprovedKit {
    /// The kit.
    pub fn manifest(&self) -> &BrandKitManifest {
        &self.0
    }

    /// The adk-surface tokens the kit sets, as CSS declarations: the card, border and text
    /// colours, the accent and focus ring, the fonts, the radius and the eight chart colours.
    /// Decision and status colours are never set. A value that is not a CSS colour is left out.
    pub fn surface_tokens(&self) -> Vec<(String, String)> {
        surface_declarations(&self.0.tokens)
    }

    /// [`Self::surface_tokens`] as a CSS rule for `[data-adk-ui-kit="<kit id>"]`: put the
    /// attribute on the element around the host's `DrawnView`s.
    pub fn surface_css(&self) -> String {
        surface_rule(&self.0.id, &self.0.tokens)
    }

    /// The A2UI `theme` a host's transport adds to `createSurface` for a third-party A2UI
    /// renderer: the primary colour only. Never an icon or a display name, which would let a
    /// drawing pass for someone else, and never part of the stored drawing.
    pub fn a2ui_theme(&self) -> Value {
        let primary = &self.0.tokens.colors.primary;
        if is_css_color_token(primary) && !primary.trim().starts_with("var(") {
            json!({ "primaryColor": primary.trim() })
        } else {
            json!({})
        }
    }
}

/// The adk-surface tokens `tokens` set, as CSS declarations (see
/// [`ApprovedKit::surface_tokens`]).
pub(crate) fn surface_declarations(tokens: &KitSemanticTokens) -> Vec<(String, String)> {
    let colors = &tokens.colors;
    let mut out = Vec::new();
    // A literal colour only: `var(--decision)` would let an accent borrow a decision's colour.
    let mut color = |name: &str, value: &str| {
        if is_css_color_token(value) && !value.trim().starts_with("var(") {
            out.push((name.to_string(), value.trim().to_string()));
        }
    };
    color("--bg", &colors.background);
    color("--card", &colors.surface);
    color("--raised", &colors.muted);
    color("--border", &colors.border);
    color("--text", &colors.surface_foreground);
    color("--text-2", &colors.muted_foreground);
    color("--label", &colors.muted_foreground);
    color("--accent", &colors.primary);
    color("--focus", &colors.ring);
    color("--chart-grid", &colors.muted);
    color("--chart-axis", &colors.muted_foreground);
    for (index, value) in colors.chart.iter().take(8).enumerate() {
        color(&format!("--chart-{}", index + 1), value);
    }
    out.push((
        "--font".to_string(),
        font_stack(
            &tokens.typography.body_family,
            "ui-sans-serif, system-ui, sans-serif",
        ),
    ));
    out.push((
        "--mono".to_string(),
        font_stack(&tokens.typography.mono_family, "ui-monospace, monospace"),
    ));
    out.push((
        "--radius".to_string(),
        radius_value(&tokens.radius).to_string(),
    ));
    out
}

/// `tokens` as a CSS rule for `[data-adk-ui-kit="<kit_id>"]`.
pub(crate) fn surface_rule(kit_id: &str, tokens: &KitSemanticTokens) -> String {
    let mut css = format!("[data-adk-ui-kit=\"{}\"] {{\n", css_string(kit_id));
    for (name, value) in surface_declarations(tokens) {
        css.push_str(&format!("  {name}: {value};\n"));
    }
    css.push_str("}\n");
    css
}

/// The CSS generic font families, written without quotes.
const GENERIC_FAMILIES: &[&str] = &[
    "serif",
    "sans-serif",
    "monospace",
    "cursive",
    "fantasy",
    "system-ui",
    "ui-serif",
    "ui-sans-serif",
    "ui-monospace",
    "ui-rounded",
    "math",
    "emoji",
];

/// `families` (one name, or a comma-separated list) as a CSS font stack ending in
/// `fallbacks`: each name quoted on its own, generic families left bare.
fn font_stack(families: &str, fallbacks: &str) -> String {
    let mut stack: Vec<String> = families
        .split(',')
        .map(str::trim)
        .filter(|family| !family.is_empty())
        .map(|family| {
            if GENERIC_FAMILIES.contains(&family.to_ascii_lowercase().as_str()) {
                family.to_ascii_lowercase()
            } else {
                format!("\"{}\"", css_string(family))
            }
        })
        .collect();
    stack.push(fallbacks.to_string());
    stack.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_family_list_is_quoted_name_by_name() {
        assert_eq!(
            font_stack("Avenir Next, sans-serif", "ui-sans-serif"),
            "\"Avenir Next\", sans-serif, ui-sans-serif"
        );
        assert_eq!(font_stack("Inter", "monospace"), "\"Inter\", monospace");
        assert_eq!(font_stack("", "monospace"), "monospace");
    }
}
