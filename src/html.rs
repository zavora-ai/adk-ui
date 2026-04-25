//! HTML renderer for converting Component trees into clean, embeddable HTML.
//!
//! This module provides two public entry points:
//! - [`render_components_html`] — typed path for `Vec<Component>`
//! - [`render_surface_html`] — deserializes `Vec<Value>` from a `UiSurface`
//!
//! Both produce self-contained HTML with inline styles only (no external CSS/JS).

use crate::interop::surface::UiSurface;
use crate::schema::*;
use serde::{Deserialize, Serialize};

/// Bandwidth mode controlling adaptive rendering.
///
/// In `Low` mode, bandwidth-sensitive components (Chart, Image, Skeleton, Spinner)
/// are omitted and inline `style="..."` attributes are stripped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BandwidthMode {
    #[default]
    Full,
    Low,
}

/// Options for HTML rendering.
#[derive(Debug, Clone, Default)]
pub struct HtmlRenderOptions {
    /// Bandwidth mode controlling adaptive rendering.
    pub bandwidth_mode: BandwidthMode,
    /// Optional CSS class prefix to namespace generated classes (e.g. "adk-").
    pub class_prefix: Option<String>,
}

/// Escape user-provided text to prevent HTML injection.
///
/// Escapes `<`, `>`, `&`, `"`, and `'`.
pub fn escape_html(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '&' => output.push_str("&amp;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#x27;"),
            _ => output.push(ch),
        }
    }
    output
}

/// Helper to produce a prefixed CSS class name.
fn cls(prefix: &Option<String>, name: &str) -> String {
    match prefix {
        Some(p) => format!("{}{}", p, name),
        None => name.to_string(),
    }
}

/// Render a single Component to an HTML fragment.
fn render_component_html(
    component: &Component,
    options: &HtmlRenderOptions,
) -> String {
    let mode = options.bandwidth_mode;
    let prefix = &options.class_prefix;

    match component {
        // --- Atoms ---
        Component::Text(text) => {
            let content = escape_html(&text.content);
            match text.variant {
                TextVariant::H1 => format!("<h1>{}</h1>", content),
                TextVariant::H2 => format!("<h2>{}</h2>", content),
                TextVariant::H3 => format!("<h3>{}</h3>", content),
                TextVariant::H4 => format!("<h4>{}</h4>", content),
                TextVariant::Body => format!("<p>{}</p>", content),
                TextVariant::Caption => format!("<small>{}</small>", content),
                TextVariant::Code => format!("<code>{}</code>", content),
            }
        }

        Component::Button(button) => {
            let label = escape_html(&button.label);
            let action_id = escape_html(&button.action_id);
            let disabled = if button.disabled { " disabled" } else { "" };
            format!(
                "<button data-action-id=\"{}\"{}>{}</button>",
                action_id, disabled, label
            )
        }

        Component::Icon(icon) => {
            let name = escape_html(&icon.name);
            format!(
                "<span class=\"{}\" data-icon=\"{}\">{}</span>",
                cls(prefix, "icon"),
                name,
                name
            )
        }

        Component::Image(image) => {
            if mode == BandwidthMode::Low {
                return String::new();
            }
            let src = escape_html(&image.src);
            let alt = image
                .alt
                .as_deref()
                .map(escape_html)
                .unwrap_or_default();
            format!("<img src=\"{}\" alt=\"{}\">", src, alt)
        }

        Component::Badge(badge) => {
            let label = escape_html(&badge.label);
            let variant = badge_variant_str(&badge.variant);
            format!(
                "<span class=\"{} {}\">{}</span>",
                cls(prefix, "badge"),
                cls(prefix, &format!("badge-{}", variant)),
                label
            )
        }

        // --- Inputs ---
        Component::TextInput(input) => {
            let label = escape_html(&input.label);
            let name = escape_html(&input.name);
            let placeholder = input
                .placeholder
                .as_deref()
                .map(|p| format!(" placeholder=\"{}\"", escape_html(p)))
                .unwrap_or_default();
            let required = if input.required { " required" } else { "" };
            let default_val = input
                .default_value
                .as_deref()
                .map(|v| format!(" value=\"{}\"", escape_html(v)))
                .unwrap_or_default();
            format!(
                "<label>{}<input type=\"text\" name=\"{}\"{}{}{}></label>",
                label, name, placeholder, required, default_val
            )
        }

        Component::NumberInput(input) => {
            let label = escape_html(&input.label);
            let name = escape_html(&input.name);
            let min = input
                .min
                .map(|v| format!(" min=\"{}\"", v))
                .unwrap_or_default();
            let max = input
                .max
                .map(|v| format!(" max=\"{}\"", v))
                .unwrap_or_default();
            let step = input
                .step
                .map(|v| format!(" step=\"{}\"", v))
                .unwrap_or_default();
            let required = if input.required { " required" } else { "" };
            let default_val = input
                .default_value
                .map(|v| format!(" value=\"{}\"", v))
                .unwrap_or_default();
            format!(
                "<label>{}<input type=\"number\" name=\"{}\"{}{}{}{}{}></label>",
                label, name, min, max, step, required, default_val
            )
        }

        Component::Select(select) => {
            let label = escape_html(&select.label);
            let name = escape_html(&select.name);
            let required = if select.required { " required" } else { "" };
            let options_html: String = select
                .options
                .iter()
                .map(|opt| {
                    format!(
                        "<option value=\"{}\">{}</option>",
                        escape_html(&opt.value),
                        escape_html(&opt.label)
                    )
                })
                .collect();
            format!(
                "<label>{}<select name=\"{}\"{}>{}</select></label>",
                label, name, required, options_html
            )
        }

        Component::MultiSelect(multi) => {
            let label = escape_html(&multi.label);
            let name = escape_html(&multi.name);
            let required = if multi.required { " required" } else { "" };
            let options_html: String = multi
                .options
                .iter()
                .map(|opt| {
                    format!(
                        "<option value=\"{}\">{}</option>",
                        escape_html(&opt.value),
                        escape_html(&opt.label)
                    )
                })
                .collect();
            format!(
                "<label>{}<select multiple name=\"{}\"{}>{}</select></label>",
                label, name, required, options_html
            )
        }

        Component::Switch(switch) => {
            let label = escape_html(&switch.label);
            let name = escape_html(&switch.name);
            let checked = if switch.default_checked {
                " checked"
            } else {
                ""
            };
            format!(
                "<label>{}<input type=\"checkbox\" role=\"switch\" name=\"{}\"{}></label>",
                label, name, checked
            )
        }

        Component::DateInput(date) => {
            let label = escape_html(&date.label);
            let name = escape_html(&date.name);
            let required = if date.required { " required" } else { "" };
            format!(
                "<label>{}<input type=\"date\" name=\"{}\"{}></label>",
                label, name, required
            )
        }

        Component::Slider(slider) => {
            let label = escape_html(&slider.label);
            let name = escape_html(&slider.name);
            let step = slider
                .step
                .map(|v| format!(" step=\"{}\"", v))
                .unwrap_or_default();
            let default_val = slider
                .default_value
                .map(|v| format!(" value=\"{}\"", v))
                .unwrap_or_default();
            format!(
                "<label>{}<input type=\"range\" name=\"{}\" min=\"{}\" max=\"{}\"{}{}></label>",
                label, name, slider.min, slider.max, step, default_val
            )
        }

        Component::Textarea(textarea) => {
            let label = escape_html(&textarea.label);
            let name = escape_html(&textarea.name);
            let placeholder = textarea
                .placeholder
                .as_deref()
                .map(|p| format!(" placeholder=\"{}\"", escape_html(p)))
                .unwrap_or_default();
            let required = if textarea.required { " required" } else { "" };
            let default_val = textarea
                .default_value
                .as_deref()
                .map(escape_html)
                .unwrap_or_default();
            format!(
                "<label>{}<textarea name=\"{}\" rows=\"{}\"{}{}>{}</textarea></label>",
                label, name, textarea.rows, placeholder, required, default_val
            )
        }

        // --- Layouts ---
        Component::Stack(stack) => {
            let dir = match stack.direction {
                StackDirection::Horizontal => "horizontal",
                StackDirection::Vertical => "vertical",
            };
            let children_html = render_children(&stack.children, options);
            let style_attr = if mode == BandwidthMode::Low {
                String::new()
            } else if stack.gap > 0 {
                format!(" style=\"gap: {}px\"", stack.gap)
            } else {
                String::new()
            };
            format!(
                "<div class=\"{} {}\"{}>{}</div>",
                cls(prefix, "stack"),
                cls(prefix, &format!("stack-{}", dir)),
                style_attr,
                children_html
            )
        }

        Component::Grid(grid) => {
            let children_html = render_children(&grid.children, options);
            let style_attr = if mode == BandwidthMode::Low {
                String::new()
            } else {
                format!(
                    " style=\"grid-template-columns: repeat({}, 1fr)\"",
                    grid.columns
                )
            };
            format!(
                "<div class=\"{}\"{}>{}</div>",
                cls(prefix, "grid"),
                style_attr,
                children_html
            )
        }

        Component::Card(card) => {
            let mut html = format!("<div class=\"{}\">", cls(prefix, "card"));
            if let Some(title) = &card.title {
                html.push_str(&format!("<h3>{}</h3>", escape_html(title)));
            }
            if let Some(desc) = &card.description {
                html.push_str(&format!("<p>{}</p>", escape_html(desc)));
            }
            if !card.content.is_empty() {
                html.push_str(&format!(
                    "<div class=\"{}\">",
                    cls(prefix, "card-content")
                ));
                html.push_str(&render_children(&card.content, options));
                html.push_str("</div>");
            }
            if let Some(footer) = &card.footer {
                html.push_str(&format!(
                    "<div class=\"{}\">",
                    cls(prefix, "card-footer")
                ));
                html.push_str(&render_children(footer, options));
                html.push_str("</div>");
            }
            html.push_str("</div>");
            html
        }

        Component::Container(container) => {
            let children_html = render_children(&container.children, options);
            let style_attr = if mode == BandwidthMode::Low || container.padding == 0 {
                String::new()
            } else {
                format!(" style=\"padding: {}px\"", container.padding)
            };
            format!(
                "<div class=\"{}\"{}>{}</div>",
                cls(prefix, "container"),
                style_attr,
                children_html
            )
        }

        Component::Divider(_) => "<hr>".to_string(),

        Component::Tabs(tabs) => {
            let mut html = format!("<div class=\"{}\">", cls(prefix, "tabs"));
            // Tab buttons
            html.push_str(&format!(
                "<div class=\"{}\">",
                cls(prefix, "tab-buttons")
            ));
            for (i, tab) in tabs.tabs.iter().enumerate() {
                html.push_str(&format!(
                    "<button class=\"{}\" data-tab-index=\"{}\">{}</button>",
                    cls(prefix, "tab-button"),
                    i,
                    escape_html(&tab.label)
                ));
            }
            html.push_str("</div>");
            // Tab content panels
            for (i, tab) in tabs.tabs.iter().enumerate() {
                html.push_str(&format!(
                    "<div class=\"{}\" data-tab-panel=\"{}\">",
                    cls(prefix, "tab-panel"),
                    i
                ));
                html.push_str(&render_children(&tab.content, options));
                html.push_str("</div>");
            }
            html.push_str("</div>");
            html
        }

        // --- Data Display ---
        Component::Table(table) => {
            let mut html = String::from("<table>");
            // Header
            html.push_str("<thead><tr>");
            for col in &table.columns {
                html.push_str(&format!("<th>{}</th>", escape_html(&col.header)));
            }
            html.push_str("</tr></thead>");
            // Body
            html.push_str("<tbody>");
            for row in &table.data {
                html.push_str("<tr>");
                for col in &table.columns {
                    let cell_value = row
                        .get(&col.accessor_key)
                        .map(|v| match v {
                            serde_json::Value::String(s) => escape_html(s),
                            other => escape_html(&other.to_string()),
                        })
                        .unwrap_or_default();
                    html.push_str(&format!("<td>{}</td>", cell_value));
                }
                html.push_str("</tr>");
            }
            html.push_str("</tbody></table>");
            html
        }

        Component::List(list) => {
            let tag = if list.ordered { "ol" } else { "ul" };
            let items_html: String = list
                .items
                .iter()
                .map(|item| format!("<li>{}</li>", escape_html(item)))
                .collect();
            format!("<{}>{}</{}>", tag, items_html, tag)
        }

        Component::KeyValue(kv) => {
            let mut html = String::from("<dl>");
            for pair in &kv.pairs {
                html.push_str(&format!(
                    "<dt>{}</dt><dd>{}</dd>",
                    escape_html(&pair.key),
                    escape_html(&pair.value)
                ));
            }
            html.push_str("</dl>");
            html
        }

        Component::CodeBlock(code_block) => {
            let lang_attr = code_block
                .language
                .as_deref()
                .map(|l| format!(" class=\"language-{}\"", escape_html(l)))
                .unwrap_or_default();
            format!(
                "<pre><code{}>{}</code></pre>",
                lang_attr,
                escape_html(&code_block.code)
            )
        }

        // --- Visualizations ---
        Component::Chart(chart) => {
            if mode == BandwidthMode::Low {
                return String::new();
            }
            let chart_json = escape_html(
                &serde_json::to_string(chart).unwrap_or_default(),
            );
            let title_html = chart
                .title
                .as_deref()
                .map(|t| format!("<p>{}</p>", escape_html(t)))
                .unwrap_or_default();
            format!(
                "<div class=\"{}\" data-chart=\"{}\">{}</div>",
                cls(prefix, "chart-placeholder"),
                chart_json,
                title_html
            )
        }

        // --- Feedback ---
        Component::Alert(alert) => {
            let variant = alert_variant_str(&alert.variant);
            let desc = alert
                .description
                .as_deref()
                .map(|d| format!("<p>{}</p>", escape_html(d)))
                .unwrap_or_default();
            format!(
                "<div class=\"{} {}\" role=\"alert\"><strong>{}</strong>{}</div>",
                cls(prefix, "alert"),
                cls(prefix, &format!("alert-{}", variant)),
                escape_html(&alert.title),
                desc
            )
        }

        Component::Progress(progress) => {
            let label = progress
                .label
                .as_deref()
                .map(|l| format!(" aria-label=\"{}\"", escape_html(l)))
                .unwrap_or_default();
            format!(
                "<progress value=\"{}\" max=\"100\"{}></progress>",
                progress.value, label
            )
        }

        Component::Toast(toast) => {
            let variant = alert_variant_str(&toast.variant);
            format!(
                "<div class=\"{} {}\">{}</div>",
                cls(prefix, "toast"),
                cls(prefix, &format!("toast-{}", variant)),
                escape_html(&toast.message)
            )
        }

        Component::Modal(modal) => {
            let mut html = String::from("<dialog>");
            html.push_str(&format!("<h2>{}</h2>", escape_html(&modal.title)));
            html.push_str(&render_children(&modal.content, options));
            if let Some(footer) = &modal.footer {
                html.push_str(&format!(
                    "<div class=\"{}\">",
                    cls(prefix, "modal-footer")
                ));
                html.push_str(&render_children(footer, options));
                html.push_str("</div>");
            }
            html.push_str("</dialog>");
            html
        }

        Component::Spinner(spinner) => {
            if mode == BandwidthMode::Low {
                return String::new();
            }
            let label = spinner
                .label
                .as_deref()
                .map(|l| format!(" aria-label=\"{}\"", escape_html(l)))
                .unwrap_or_default();
            format!(
                "<div class=\"{}\" role=\"status\"{}></div>",
                cls(prefix, "spinner"),
                label
            )
        }

        Component::Skeleton(_) => {
            if mode == BandwidthMode::Low {
                return String::new();
            }
            format!("<div class=\"{}\"></div>", cls(prefix, "skeleton"))
        }
    }
}

/// Render a list of child components.
fn render_children(children: &[Component], options: &HtmlRenderOptions) -> String {
    children
        .iter()
        .map(|c| render_component_html(c, options))
        .collect()
}

/// Helper to convert BadgeVariant to a CSS-friendly string.
fn badge_variant_str(variant: &BadgeVariant) -> &'static str {
    match variant {
        BadgeVariant::Default => "default",
        BadgeVariant::Info => "info",
        BadgeVariant::Success => "success",
        BadgeVariant::Warning => "warning",
        BadgeVariant::Error => "error",
        BadgeVariant::Secondary => "secondary",
        BadgeVariant::Outline => "outline",
    }
}

/// Helper to convert AlertVariant to a CSS-friendly string.
fn alert_variant_str(variant: &AlertVariant) -> &'static str {
    match variant {
        AlertVariant::Info => "info",
        AlertVariant::Success => "success",
        AlertVariant::Warning => "warning",
        AlertVariant::Error => "error",
    }
}

/// Minimal inline CSS for the HTML shell.
const INLINE_CSS: &str = r#"
body { font-family: system-ui, -apple-system, sans-serif; margin: 0; padding: 16px; color: #1a1a1a; }
.stack { display: flex; }
.stack-vertical { flex-direction: column; }
.stack-horizontal { flex-direction: row; }
.grid { display: grid; }
.card { border: 1px solid #e0e0e0; border-radius: 8px; padding: 16px; margin-bottom: 12px; }
.card-content { margin-top: 8px; }
.card-footer { margin-top: 12px; border-top: 1px solid #e0e0e0; padding-top: 8px; }
.container { padding: 16px; }
.tabs { margin-bottom: 12px; }
.tab-buttons { display: flex; gap: 4px; border-bottom: 1px solid #e0e0e0; margin-bottom: 8px; }
.tab-button { background: none; border: none; padding: 8px 16px; cursor: pointer; }
.badge { display: inline-block; padding: 2px 8px; border-radius: 12px; font-size: 0.85em; }
.badge-default { background: #e0e0e0; }
.badge-info { background: #dbeafe; color: #1e40af; }
.badge-success { background: #dcfce7; color: #166534; }
.badge-warning { background: #fef3c7; color: #92400e; }
.badge-error { background: #fee2e2; color: #991b1b; }
.badge-secondary { background: #f3f4f6; color: #374151; }
.badge-outline { border: 1px solid #d1d5db; background: transparent; }
.alert { padding: 12px 16px; border-radius: 6px; margin-bottom: 12px; }
.alert-info { background: #dbeafe; color: #1e40af; }
.alert-success { background: #dcfce7; color: #166534; }
.alert-warning { background: #fef3c7; color: #92400e; }
.alert-error { background: #fee2e2; color: #991b1b; }
.toast { padding: 12px 16px; border-radius: 6px; margin-bottom: 8px; }
.toast-info { background: #dbeafe; }
.toast-success { background: #dcfce7; }
.toast-warning { background: #fef3c7; }
.toast-error { background: #fee2e2; }
.spinner { width: 24px; height: 24px; border: 3px solid #e0e0e0; border-top-color: #3b82f6; border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
.skeleton { background: linear-gradient(90deg, #e0e0e0 25%, #f0f0f0 50%, #e0e0e0 75%); background-size: 200% 100%; animation: shimmer 1.5s infinite; border-radius: 4px; min-height: 20px; }
@keyframes shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }
.chart-placeholder { border: 1px dashed #d1d5db; padding: 16px; text-align: center; color: #6b7280; }
.icon { display: inline-flex; align-items: center; }
.modal-footer { margin-top: 12px; border-top: 1px solid #e0e0e0; padding-top: 8px; }
table { width: 100%; border-collapse: collapse; }
th, td { text-align: left; padding: 8px 12px; border-bottom: 1px solid #e0e0e0; }
th { font-weight: 600; }
progress { width: 100%; }
label { display: block; margin-bottom: 12px; }
input, select, textarea { display: block; margin-top: 4px; padding: 6px 8px; border: 1px solid #d1d5db; border-radius: 4px; width: 100%; box-sizing: border-box; }
button { padding: 8px 16px; border-radius: 6px; border: 1px solid #d1d5db; cursor: pointer; background: #3b82f6; color: white; }
button:disabled { opacity: 0.5; cursor: not-allowed; }
hr { border: none; border-top: 1px solid #e0e0e0; margin: 12px 0; }
dl { margin: 0; }
dt { font-weight: 600; margin-top: 8px; }
dd { margin-left: 0; margin-bottom: 4px; }
pre { background: #f3f4f6; padding: 12px; border-radius: 6px; overflow-x: auto; }
code { font-family: ui-monospace, monospace; }
dialog { border: 1px solid #e0e0e0; border-radius: 8px; padding: 24px; max-width: 600px; }
"#;

/// Generate the prefixed inline CSS when a class prefix is set.
fn generate_prefixed_css(prefix: &str) -> String {
    INLINE_CSS.replace('.', &format!(".{}", prefix))
}

/// Wrap rendered component HTML in a minimal HTML document shell.
fn wrap_in_shell(body: &str, options: &HtmlRenderOptions) -> String {
    let css = match &options.class_prefix {
        Some(p) => {
            // Include both unprefixed (for elements like table, button, etc.) and prefixed CSS
            let prefixed = generate_prefixed_css(p);
            format!("{}\n{}", INLINE_CSS.trim(), prefixed.trim())
        }
        None => INLINE_CSS.trim().to_string(),
    };

    if options.bandwidth_mode == BandwidthMode::Low {
        // In low bandwidth mode, omit the style block entirely
        format!(
            "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"></head><body>{}</body></html>",
            body
        )
    } else {
        format!(
            "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><style>{}</style></head><body>{}</body></html>",
            css, body
        )
    }
}

/// Render typed components directly to HTML.
///
/// Preferred when you have a `UiResponse` with `Vec<Component>`.
/// Wraps output in a minimal self-contained HTML shell with inline styles only.
pub fn render_components_html(components: &[Component], options: &HtmlRenderOptions) -> String {
    let body: String = components
        .iter()
        .map(|c| render_component_html(c, options))
        .collect();
    wrap_in_shell(&body, options)
}

/// Render a UiSurface as embeddable HTML.
///
/// Deserializes each `Value` in `surface.components` into a `Component`.
/// Values that fail deserialization are rendered as `<!-- unknown component -->`.
/// Wraps output in a minimal self-contained HTML shell with inline styles only.
pub fn render_surface_html(surface: &UiSurface, options: &HtmlRenderOptions) -> String {
    let body: String = surface
        .components
        .iter()
        .map(|value| {
            match serde_json::from_value::<Component>(value.clone()) {
                Ok(component) => render_component_html(&component, options),
                Err(_) => "<!-- unknown component -->".to_string(),
            }
        })
        .collect();
    wrap_in_shell(&body, options)
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn default_opts() -> HtmlRenderOptions {
        HtmlRenderOptions::default()
    }

    fn low_bw_opts() -> HtmlRenderOptions {
        HtmlRenderOptions {
            bandwidth_mode: BandwidthMode::Low,
            ..Default::default()
        }
    }

    fn prefixed_opts(prefix: &str) -> HtmlRenderOptions {
        HtmlRenderOptions {
            class_prefix: Some(prefix.to_string()),
            ..Default::default()
        }
    }

    // --- escape_html ---

    #[test]
    fn escape_html_escapes_all_special_chars() {
        assert_eq!(
            escape_html("<script>alert('xss')&\"</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&amp;&quot;&lt;/script&gt;"
        );
    }

    #[test]
    fn escape_html_passes_through_normal_text() {
        assert_eq!(escape_html("Hello, world!"), "Hello, world!");
    }

    // --- BandwidthMode ---

    #[test]
    fn bandwidth_mode_default_is_full() {
        assert_eq!(BandwidthMode::default(), BandwidthMode::Full);
    }

    // --- Text variants ---

    #[test]
    fn text_body_renders_as_p() {
        let c = Component::Text(Text {
            id: None,
            content: "Hello".to_string(),
            variant: TextVariant::Body,
        });
        let html = render_component_html(&c, &default_opts());
        assert_eq!(html, "<p>Hello</p>");
    }

    #[test]
    fn text_h1_renders_as_h1() {
        let c = Component::Text(Text {
            id: None,
            content: "Title".to_string(),
            variant: TextVariant::H1,
        });
        let html = render_component_html(&c, &default_opts());
        assert_eq!(html, "<h1>Title</h1>");
    }

    #[test]
    fn text_caption_renders_as_small() {
        let c = Component::Text(Text {
            id: None,
            content: "Note".to_string(),
            variant: TextVariant::Caption,
        });
        let html = render_component_html(&c, &default_opts());
        assert_eq!(html, "<small>Note</small>");
    }

    #[test]
    fn text_code_renders_as_code() {
        let c = Component::Text(Text {
            id: None,
            content: "let x = 1;".to_string(),
            variant: TextVariant::Code,
        });
        let html = render_component_html(&c, &default_opts());
        assert_eq!(html, "<code>let x = 1;</code>");
    }

    // --- Button ---

    #[test]
    fn button_renders_with_action_id() {
        let c = Component::Button(Button {
            id: None,
            label: "Click me".to_string(),
            action_id: "btn-1".to_string(),
            variant: ButtonVariant::Primary,
            disabled: false,
            icon: None,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("data-action-id=\"btn-1\""));
        assert!(html.contains("Click me"));
    }

    #[test]
    fn button_disabled_renders_disabled_attr() {
        let c = Component::Button(Button {
            id: None,
            label: "Disabled".to_string(),
            action_id: "btn-2".to_string(),
            variant: ButtonVariant::Primary,
            disabled: true,
            icon: None,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains(" disabled"));
    }

    // --- Image ---

    #[test]
    fn image_renders_in_full_mode() {
        let c = Component::Image(Image {
            id: None,
            src: "https://example.com/img.png".to_string(),
            alt: Some("A photo".to_string()),
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("<img"));
        assert!(html.contains("src=\"https://example.com/img.png\""));
        assert!(html.contains("alt=\"A photo\""));
    }

    #[test]
    fn image_omitted_in_low_bandwidth() {
        let c = Component::Image(Image {
            id: None,
            src: "https://example.com/img.png".to_string(),
            alt: Some("A photo".to_string()),
        });
        let html = render_component_html(&c, &low_bw_opts());
        assert!(html.is_empty());
    }

    // --- Badge ---

    #[test]
    fn badge_renders_with_variant() {
        let c = Component::Badge(Badge {
            id: None,
            label: "New".to_string(),
            variant: BadgeVariant::Success,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("badge"));
        assert!(html.contains("badge-success"));
        assert!(html.contains("New"));
    }

    // --- Stack ---

    #[test]
    fn stack_vertical_renders_correctly() {
        let c = Component::Stack(Stack {
            id: None,
            direction: StackDirection::Vertical,
            children: vec![Component::Text(Text {
                id: None,
                content: "Child".to_string(),
                variant: TextVariant::Body,
            })],
            gap: 0,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("stack-vertical"));
        assert!(html.contains("<p>Child</p>"));
    }

    // --- Grid ---

    #[test]
    fn grid_renders_with_columns() {
        let c = Component::Grid(Grid {
            id: None,
            columns: 3,
            children: vec![],
            gap: 0,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("grid"));
        assert!(html.contains("grid-template-columns: repeat(3, 1fr)"));
    }

    #[test]
    fn grid_low_bandwidth_strips_style() {
        let c = Component::Grid(Grid {
            id: None,
            columns: 3,
            children: vec![],
            gap: 0,
        });
        let html = render_component_html(&c, &low_bw_opts());
        assert!(html.contains("grid"));
        assert!(!html.contains("style="));
    }

    // --- Card ---

    #[test]
    fn card_renders_with_title_and_content() {
        let c = Component::Card(Card {
            id: None,
            title: Some("My Card".to_string()),
            description: None,
            content: vec![Component::Text(Text {
                id: None,
                content: "Body text".to_string(),
                variant: TextVariant::Body,
            })],
            footer: None,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("card"));
        assert!(html.contains("<h3>My Card</h3>"));
        assert!(html.contains("<p>Body text</p>"));
    }

    // --- Table ---

    #[test]
    fn table_renders_with_headers_and_rows() {
        let c = Component::Table(Table {
            id: None,
            columns: vec![TableColumn {
                header: "Name".to_string(),
                accessor_key: "name".to_string(),
                sortable: true,
            }],
            data: vec![{
                let mut row = std::collections::HashMap::new();
                row.insert("name".to_string(), json!("Alice"));
                row
            }],
            sortable: false,
            page_size: None,
            striped: false,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>Name</th>"));
        assert!(html.contains("<td>Alice</td>"));
    }

    // --- Alert ---

    #[test]
    fn alert_renders_with_role() {
        let c = Component::Alert(Alert {
            id: None,
            title: "Warning!".to_string(),
            description: Some("Be careful".to_string()),
            variant: AlertVariant::Warning,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("alert-warning"));
        assert!(html.contains("Warning!"));
    }

    // --- Progress ---

    #[test]
    fn progress_renders_with_value() {
        let c = Component::Progress(Progress {
            id: None,
            value: 75,
            label: Some("Loading".to_string()),
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("<progress"));
        assert!(html.contains("value=\"75\""));
        assert!(html.contains("max=\"100\""));
    }

    // --- Chart ---

    #[test]
    fn chart_omitted_in_low_bandwidth() {
        let c = Component::Chart(Chart {
            id: None,
            title: Some("Sales".to_string()),
            kind: ChartKind::Bar,
            data: vec![],
            x_key: "month".to_string(),
            y_keys: vec!["revenue".to_string()],
            x_label: None,
            y_label: None,
            show_legend: true,
            colors: None,
        });
        let html = render_component_html(&c, &low_bw_opts());
        assert!(html.is_empty());
    }

    #[test]
    fn chart_renders_placeholder_in_full_mode() {
        let c = Component::Chart(Chart {
            id: None,
            title: Some("Sales".to_string()),
            kind: ChartKind::Bar,
            data: vec![],
            x_key: "month".to_string(),
            y_keys: vec!["revenue".to_string()],
            x_label: None,
            y_label: None,
            show_legend: true,
            colors: None,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("chart-placeholder"));
        assert!(html.contains("data-chart="));
    }

    // --- Spinner ---

    #[test]
    fn spinner_omitted_in_low_bandwidth() {
        let c = Component::Spinner(Spinner {
            id: None,
            size: SpinnerSize::Medium,
            label: None,
        });
        let html = render_component_html(&c, &low_bw_opts());
        assert!(html.is_empty());
    }

    // --- Skeleton ---

    #[test]
    fn skeleton_omitted_in_low_bandwidth() {
        let c = Component::Skeleton(Skeleton {
            id: None,
            variant: SkeletonVariant::Text,
            width: None,
            height: None,
        });
        let html = render_component_html(&c, &low_bw_opts());
        assert!(html.is_empty());
    }

    // --- Modal ---

    #[test]
    fn modal_renders_as_dialog() {
        let c = Component::Modal(Modal {
            id: None,
            title: "Confirm".to_string(),
            content: vec![Component::Text(Text {
                id: None,
                content: "Are you sure?".to_string(),
                variant: TextVariant::Body,
            })],
            footer: None,
            size: ModalSize::Medium,
            closable: true,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("<dialog>"));
        assert!(html.contains("<h2>Confirm</h2>"));
        assert!(html.contains("<p>Are you sure?</p>"));
    }

    // --- Class prefix ---

    #[test]
    fn class_prefix_applied_to_stack() {
        let c = Component::Stack(Stack {
            id: None,
            direction: StackDirection::Vertical,
            children: vec![],
            gap: 0,
        });
        let html = render_component_html(&c, &prefixed_opts("adk-"));
        assert!(html.contains("adk-stack"));
        assert!(html.contains("adk-stack-vertical"));
    }

    #[test]
    fn class_prefix_applied_to_badge() {
        let c = Component::Badge(Badge {
            id: None,
            label: "Test".to_string(),
            variant: BadgeVariant::Info,
        });
        let html = render_component_html(&c, &prefixed_opts("adk-"));
        assert!(html.contains("adk-badge"));
        assert!(html.contains("adk-badge-info"));
    }

    // --- HTML escaping in content ---

    #[test]
    fn text_content_is_escaped() {
        let c = Component::Text(Text {
            id: None,
            content: "<script>alert('xss')</script>".to_string(),
            variant: TextVariant::Body,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    // --- render_components_html ---

    #[test]
    fn render_components_html_wraps_in_shell() {
        let components = vec![Component::Text(Text {
            id: None,
            content: "Hello".to_string(),
            variant: TextVariant::Body,
        })];
        let html = render_components_html(&components, &default_opts());
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html>"));
        assert!(html.contains("<body>"));
        assert!(html.contains("<p>Hello</p>"));
    }

    #[test]
    fn render_components_html_no_external_resources() {
        let components = vec![Component::Text(Text {
            id: None,
            content: "Test".to_string(),
            variant: TextVariant::Body,
        })];
        let html = render_components_html(&components, &default_opts());
        assert!(!html.contains("<link rel=\"stylesheet\""));
        assert!(!html.contains("<script src="));
        assert!(!html.contains("@import"));
    }

    // --- render_surface_html ---

    #[test]
    fn render_surface_html_handles_valid_components() {
        let surface = UiSurface::new(
            "main",
            "catalog",
            vec![json!({"type": "text", "content": "Hello", "variant": "body"})],
        );
        let html = render_surface_html(&surface, &default_opts());
        assert!(html.contains("<p>Hello</p>"));
    }

    #[test]
    fn render_surface_html_handles_unknown_components() {
        let surface = UiSurface::new(
            "main",
            "catalog",
            vec![json!({"type": "unknown_widget", "data": 42})],
        );
        let html = render_surface_html(&surface, &default_opts());
        assert!(html.contains("<!-- unknown component -->"));
    }

    #[test]
    fn render_surface_html_mixes_valid_and_unknown() {
        let surface = UiSurface::new(
            "main",
            "catalog",
            vec![
                json!({"type": "text", "content": "Valid", "variant": "body"}),
                json!({"type": "bogus"}),
                json!({"type": "divider"}),
            ],
        );
        let html = render_surface_html(&surface, &default_opts());
        assert!(html.contains("<p>Valid</p>"));
        assert!(html.contains("<!-- unknown component -->"));
        assert!(html.contains("<hr>"));
    }

    // --- Low bandwidth strips style from shell ---

    #[test]
    fn low_bandwidth_shell_has_no_style_tag() {
        let components = vec![Component::Text(Text {
            id: None,
            content: "Test".to_string(),
            variant: TextVariant::Body,
        })];
        let html = render_components_html(&components, &low_bw_opts());
        assert!(!html.contains("<style>"));
        assert!(!html.contains("style="));
    }

    // --- Select ---

    #[test]
    fn select_renders_with_options() {
        let c = Component::Select(Select {
            id: None,
            name: "color".to_string(),
            label: "Color".to_string(),
            options: vec![
                SelectOption {
                    label: "Red".to_string(),
                    value: "red".to_string(),
                },
                SelectOption {
                    label: "Blue".to_string(),
                    value: "blue".to_string(),
                },
            ],
            required: false,
            error: None,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("<select"));
        assert!(html.contains("<option value=\"red\">Red</option>"));
        assert!(html.contains("<option value=\"blue\">Blue</option>"));
    }

    // --- MultiSelect ---

    #[test]
    fn multiselect_renders_with_multiple_attr() {
        let c = Component::MultiSelect(MultiSelect {
            id: None,
            name: "tags".to_string(),
            label: "Tags".to_string(),
            options: vec![SelectOption {
                label: "A".to_string(),
                value: "a".to_string(),
            }],
            required: false,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("<select multiple"));
    }

    // --- List ---

    #[test]
    fn ordered_list_renders_as_ol() {
        let c = Component::List(List {
            id: None,
            items: vec!["First".to_string(), "Second".to_string()],
            ordered: true,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("<ol>"));
        assert!(html.contains("<li>First</li>"));
    }

    #[test]
    fn unordered_list_renders_as_ul() {
        let c = Component::List(List {
            id: None,
            items: vec!["Item".to_string()],
            ordered: false,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("<ul>"));
    }

    // --- KeyValue ---

    #[test]
    fn keyvalue_renders_as_dl() {
        let c = Component::KeyValue(KeyValue {
            id: None,
            pairs: vec![KeyValuePair {
                key: "Name".to_string(),
                value: "Alice".to_string(),
            }],
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("<dl>"));
        assert!(html.contains("<dt>Name</dt>"));
        assert!(html.contains("<dd>Alice</dd>"));
    }

    // --- CodeBlock ---

    #[test]
    fn codeblock_renders_as_pre_code() {
        let c = Component::CodeBlock(CodeBlock {
            id: None,
            code: "fn main() {}".to_string(),
            language: Some("rust".to_string()),
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("<pre><code"));
        assert!(html.contains("language-rust"));
        assert!(html.contains("fn main() {}"));
    }

    // --- Toast ---

    #[test]
    fn toast_renders_with_variant() {
        let c = Component::Toast(Toast {
            id: None,
            message: "Saved!".to_string(),
            variant: AlertVariant::Success,
            duration: 5000,
            dismissible: true,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("toast"));
        assert!(html.contains("toast-success"));
        assert!(html.contains("Saved!"));
    }

    // --- Tabs ---

    #[test]
    fn tabs_renders_buttons_and_panels() {
        let c = Component::Tabs(Tabs {
            id: None,
            tabs: vec![
                Tab {
                    label: "Tab 1".to_string(),
                    content: vec![Component::Text(Text {
                        id: None,
                        content: "Content 1".to_string(),
                        variant: TextVariant::Body,
                    })],
                },
                Tab {
                    label: "Tab 2".to_string(),
                    content: vec![],
                },
            ],
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("tab-button"));
        assert!(html.contains("Tab 1"));
        assert!(html.contains("Tab 2"));
        assert!(html.contains("tab-panel"));
        assert!(html.contains("<p>Content 1</p>"));
    }

    // --- Divider ---

    #[test]
    fn divider_renders_as_hr() {
        let c = Component::Divider(Divider { id: None });
        let html = render_component_html(&c, &default_opts());
        assert_eq!(html, "<hr>");
    }

    // --- Switch ---

    #[test]
    fn switch_renders_as_checkbox_with_role() {
        let c = Component::Switch(Switch {
            id: None,
            name: "toggle".to_string(),
            label: "Enable".to_string(),
            default_checked: true,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("role=\"switch\""));
        assert!(html.contains("type=\"checkbox\""));
        assert!(html.contains(" checked"));
    }

    // --- DateInput ---

    #[test]
    fn date_input_renders_correctly() {
        let c = Component::DateInput(DateInput {
            id: None,
            name: "dob".to_string(),
            label: "Date of Birth".to_string(),
            required: true,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("type=\"date\""));
        assert!(html.contains("Date of Birth"));
        assert!(html.contains(" required"));
    }

    // --- Slider ---

    #[test]
    fn slider_renders_as_range() {
        let c = Component::Slider(Slider {
            id: None,
            name: "volume".to_string(),
            label: "Volume".to_string(),
            min: 0.0,
            max: 100.0,
            step: Some(1.0),
            default_value: Some(50.0),
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("type=\"range\""));
        assert!(html.contains("min=\"0\""));
        assert!(html.contains("max=\"100\""));
    }

    // --- Textarea ---

    #[test]
    fn textarea_renders_correctly() {
        let c = Component::Textarea(Textarea {
            id: None,
            name: "bio".to_string(),
            label: "Bio".to_string(),
            placeholder: Some("Tell us about yourself".to_string()),
            rows: 4,
            required: false,
            default_value: None,
            error: None,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("<textarea"));
        assert!(html.contains("name=\"bio\""));
        assert!(html.contains("Bio"));
    }

    // --- Icon ---

    #[test]
    fn icon_renders_with_data_icon() {
        let c = Component::Icon(Icon {
            id: None,
            name: "heart".to_string(),
            size: 24,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("data-icon=\"heart\""));
        assert!(html.contains("icon"));
    }

    // --- Container ---

    #[test]
    fn container_renders_children() {
        let c = Component::Container(Container {
            id: None,
            children: vec![Component::Divider(Divider { id: None })],
            padding: 16,
        });
        let html = render_component_html(&c, &default_opts());
        assert!(html.contains("container"));
        assert!(html.contains("<hr>"));
        assert!(html.contains("style=\"padding: 16px\""));
    }

    #[test]
    fn container_low_bandwidth_strips_style() {
        let c = Component::Container(Container {
            id: None,
            children: vec![],
            padding: 16,
        });
        let html = render_component_html(&c, &low_bw_opts());
        assert!(!html.contains("style="));
    }
}
