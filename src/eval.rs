//! Scoring drawings: how well a model draws in a catalog, measured the way `genui` will judge
//! the drawing when it reaches a person.
//!
//! [`score`] checks one drawing (`render_screen`'s arguments, in A2UI 1.0's syntax or
//! v0.9.1's) with genui's [`Validator`] and reads what it would show: whether it is accepted
//! and, if not, the refusal the model would be told; the dark-pattern warnings it raises; the
//! components a [`Scenario`] expects that it lacks; fields a screen reader could not name; its
//! size and depth. [`Score::points`] folds them into 0 to 100, so a prompt or model change can
//! be compared run to run. [`scenarios`] are prompts across the three catalogs, and
//! `examples/drawing_eval.rs` scores the sample drawings or a JSONL file of a model's
//! drawings.
//!
//! The refusal text is what the render tool returns to the model, so a run also shows how the
//! model is told to repair a drawing.
//!
//! ```rust
//! use adk_ui::catalogs;
//! use adk_ui::eval::score;
//! use serde_json::json;
//!
//! let drawing = json!({
//!     "surfaceId": "kpis",
//!     "components": [
//!         { "id": "root", "component": "Metric", "label": "Open incidents", "value": 3 }
//!     ]
//! });
//! let scored = score(catalogs::insight(), &drawing, &["Metric"]);
//! assert!(scored.accepted);
//! assert_eq!(scored.points(), 100);
//!
//! let pushy = json!({ "surfaceId": "s", "components": [
//!     { "id": "root", "component": "Text", "text": "Approve" }
//! ] });
//! let refused = score(catalogs::insight(), &pushy, &[]);
//! assert!(!refused.accepted);
//! assert_eq!(refused.points(), 0);
//! ```

use std::collections::BTreeMap;
use std::fmt;

use adk_surface::genui::{
    Catalog, DrawnKind, DrawnNode, SurfaceDraft, Validator, components_in, drawn_tree,
};
use serde::Serialize;
use serde_json::Value;

use crate::catalogs;

/// The largest drawing that scores full marks for size, in bytes of its JSON.
pub const LEAN_BYTES: usize = 64 * 1024;
/// The deepest tree that scores full marks for depth. An extension component counts twice:
/// its node, and what the home shows for it.
pub const LEAN_DEPTH: usize = 12;

/// How one drawing fared.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    /// genui would show it.
    pub accepted: bool,
    /// Why not, as the render tool tells the model: the refusal's code and reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    /// The codes of the warnings it raises (`genui.warned.*`): pressure on the person.
    pub warnings: Vec<String>,
    /// How many of each component it uses.
    pub components: BTreeMap<String, usize>,
    /// The components the scenario expects that it does not use.
    pub missing: Vec<String>,
    /// Fields whose label is empty and that have no accessibility label: nothing a screen
    /// reader could read.
    pub unlabelled_fields: usize,
    /// The size of its JSON, in bytes.
    pub bytes: usize,
    /// The depth of what it shows.
    pub depth: usize,
    /// How many expected components it uses.
    #[serde(skip)]
    used_expected: usize,
}

impl Score {
    /// 0 to 100: nothing for a refused drawing; otherwise 50 for being shown, 15 for raising
    /// no warning, 15 for the expected components (in proportion), 10 for every field
    /// labelled, and 10 for staying within [`LEAN_BYTES`] and [`LEAN_DEPTH`].
    pub fn points(&self) -> u32 {
        if !self.accepted {
            return 0;
        }
        let mut points = 50;
        if self.warnings.is_empty() {
            points += 15;
        }
        let expected = self.missing.len() + self.used_expected;
        points += (15 * self.used_expected)
            .checked_div(expected)
            .unwrap_or(15) as u32;
        if self.unlabelled_fields == 0 {
            points += 10;
        }
        if self.bytes <= LEAN_BYTES && self.depth <= LEAN_DEPTH {
            points += 10;
        }
        points
    }
}

/// `render_screen`'s arguments as the draft genui checks, either syntax.
///
/// # Errors
///
/// Arguments that are not a screen: no components, or components 1.0 cannot read.
pub fn draft_of(args: &Value, catalog: &'static Catalog) -> Result<SurfaceDraft, String> {
    let components = args
        .get("components")
        .and_then(Value::as_array)
        .cloned()
        .ok_or("a drawing has components")?;
    let components =
        components_in(components, catalog).map_err(|error| format!("components{error}"))?;
    Ok(SurfaceDraft {
        surface_id: args
            .get("surfaceId")
            .and_then(Value::as_str)
            .unwrap_or("eval")
            .to_string(),
        kind: DrawnKind::Screen,
        title: args
            .get("title")
            .and_then(Value::as_str)
            .map(str::to_string),
        components,
        data_model: args.get("dataModel").cloned().unwrap_or(Value::Null),
        actions: args
            .get("actions")
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map_err(|error| format!("actions: {error}"))?
            .unwrap_or_default(),
    })
}

/// Scores `args`, `render_screen`'s arguments, in `catalog`, expecting `expected` components.
pub fn score(catalog: &'static Catalog, args: &Value, expected: &[&str]) -> Score {
    let bytes = args.to_string().len();
    let mut components = BTreeMap::new();
    for component in args
        .get("components")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(name) = component.get("component").and_then(Value::as_str) {
            *components.entry(name.to_string()).or_insert(0) += 1;
        }
    }
    let missing: Vec<String> = expected
        .iter()
        .filter(|name| !components.contains_key(**name))
        .map(|name| (*name).to_string())
        .collect();
    let used = expected.len() - missing.len();
    let refused = |refusal: String| Score {
        accepted: false,
        refusal: Some(refusal),
        warnings: Vec::new(),
        components: components.clone(),
        missing: missing.clone(),
        unlabelled_fields: 0,
        bytes,
        depth: 0,
        used_expected: used,
    };
    let draft = match draft_of(args, catalog) {
        Ok(draft) => draft,
        Err(reason) => return refused(reason),
    };
    let validated = match Validator::new(catalog).validate(&draft) {
        Ok(validated) => validated,
        Err(refusal) => return refused(format!("{}: {}", refusal.code(), refusal.technical())),
    };
    let tree = drawn_tree(catalog, &validated);
    Score {
        accepted: true,
        refusal: None,
        warnings: validated
            .warnings
            .iter()
            .filter_map(|warning| warning.code.clone())
            .collect(),
        components,
        missing,
        unlabelled_fields: unlabelled(&tree),
        bytes,
        depth: depth(&tree),
        used_expected: used,
    }
}

fn unlabelled(node: &DrawnNode) -> usize {
    let own = usize::from(
        node.field.is_some()
            && node.text.as_deref().is_none_or(str::is_empty)
            && node
                .accessibility
                .as_ref()
                .and_then(|a| a.label.as_deref())
                .is_none_or(str::is_empty),
    );
    own + node.children.iter().map(unlabelled).sum::<usize>()
}

fn depth(node: &DrawnNode) -> usize {
    1 + node.children.iter().map(depth).max().unwrap_or(0)
}

/// A prompt to draw from, the catalog to draw in and the components a good drawing uses.
#[derive(Debug, Clone)]
pub struct Scenario {
    /// Its name, in a report.
    pub name: String,
    /// What the model is asked to show.
    pub prompt: String,
    /// The catalog it draws in.
    pub catalog: &'static Catalog,
    /// The components a good drawing uses.
    pub expected: Vec<String>,
}

impl Scenario {
    /// A scenario named `name`: `prompt`, in `catalog`.
    pub fn new(
        name: impl Into<String>,
        prompt: impl Into<String>,
        catalog: &'static Catalog,
    ) -> Self {
        Self {
            name: name.into(),
            prompt: prompt.into(),
            catalog,
            expected: Vec::new(),
        }
    }

    /// Expects `components` in a good drawing.
    #[must_use]
    pub fn expecting<I, S>(mut self, components: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.expected = components.into_iter().map(Into::into).collect();
        self
    }

    /// Scores a drawing made for this scenario.
    pub fn score(&self, args: &Value) -> Score {
        let expected: Vec<&str> = self.expected.iter().map(String::as_str).collect();
        score(self.catalog, args, &expected)
    }
}

/// Prompts across the three catalogs, each with the components a good drawing uses.
pub fn scenarios() -> Vec<Scenario> {
    vec![
        Scenario::new(
            "weekly-revenue",
            "Show this week's revenue, orders and conversion, and revenue per day as a chart.",
            catalogs::insight(),
        )
        .expecting(["Metric", "Chart"]),
        Scenario::new(
            "incident-status",
            "Show the open incidents: how many, their severity, and the latest status of each.",
            catalogs::insight(),
        )
        .expecting(["Metric", "Badge"]),
        Scenario::new(
            "latency-vega",
            "Plot p95 latency per service over the last 7 days from these rows, as a Vega-Lite line chart.",
            catalogs::insight(),
        )
        .expecting(["VegaLite"]),
        Scenario::new(
            "release-app",
            "Lay out a release room: a page with the checklist, the build status and the owners.",
            catalogs::studio(),
        )
        .expecting(["App", "Page"]),
        Scenario::new(
            "revenue-3d",
            "Show revenue by region and quarter in 3D.",
            catalogs::spatial(),
        )
        .expecting(["Bar3D"]),
        Scenario::new(
            "service-graph",
            "Show the services and which depend on which, highlighting the failing one.",
            catalogs::spatial(),
        )
        .expecting(["Graph3D"]),
    ]
}

/// Scores by name, for a run's summary.
#[derive(Debug, Clone, Default)]
pub struct Report {
    /// Each drawing's name and score, in order.
    pub rows: Vec<(String, Score)>,
}

impl Report {
    /// Adds a scored drawing.
    pub fn add(&mut self, name: impl Into<String>, score: Score) {
        self.rows.push((name.into(), score));
    }

    /// The share of drawings genui would show, 0 to 1.
    pub fn accepted_rate(&self) -> f64 {
        if self.rows.is_empty() {
            return 0.0;
        }
        self.rows.iter().filter(|(_, score)| score.accepted).count() as f64 / self.rows.len() as f64
    }

    /// The mean of [`Score::points`].
    pub fn mean_points(&self) -> f64 {
        if self.rows.is_empty() {
            return 0.0;
        }
        self.rows
            .iter()
            .map(|(_, score)| f64::from(score.points()))
            .sum::<f64>()
            / self.rows.len() as f64
    }
}

impl fmt::Display for Report {
    /// A Markdown table, one row per drawing, then the totals.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "| Drawing | Shown | Points | Warnings | Missing | Unlabelled | KiB | Depth |"
        )?;
        writeln!(f, "|---|---|---|---|---|---|---|---|")?;
        for (name, score) in &self.rows {
            writeln!(
                f,
                "| {name} | {} | {} | {} | {} | {} | {:.1} | {} |",
                if score.accepted { "yes" } else { "no" },
                score.points(),
                score.warnings.len(),
                score.missing.join(", "),
                score.unlabelled_fields,
                score.bytes as f64 / 1024.0,
                score.depth,
            )?;
            if let Some(refusal) = &score.refusal {
                writeln!(
                    f,
                    "|  | ↳ {} |  |  |  |  |  |  |",
                    refusal.replace('|', "/")
                )?;
            }
        }
        write!(
            f,
            "\n{} drawings · {:.0}% shown · {:.1} points on average",
            self.rows.len(),
            self.accepted_rate() * 100.0,
            self.mean_points()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn points_follow_what_genui_would_show() {
        let insight = catalogs::insight();
        let plain = json!({ "surfaceId": "s", "components": [
            { "id": "root", "component": "Column", "children": ["n", "f"] },
            { "id": "n", "component": "Metric", "label": "Orders", "value": 12 },
            { "id": "f", "component": "TextField", "label": "", "value": { "path": "/note" } }
        ], "dataModel": { "note": "" } });
        let scored = score(insight, &plain, &["Metric", "Chart"]);
        assert!(scored.accepted, "{:?}", scored.refusal);
        assert_eq!(scored.missing, ["Chart"]);
        assert_eq!(scored.unlabelled_fields, 1);
        assert_eq!(scored.points(), 50 + 15 + 7 + 10);

        let urgent = json!({ "surfaceId": "s", "components": [
            { "id": "root", "component": "Text", "text": "Only 2 left — offer ends in 10 minutes!" }
        ] });
        let warned = score(insight, &urgent, &[]);
        assert!(warned.accepted && !warned.warnings.is_empty(), "{warned:?}");
        assert_eq!(warned.points(), 85);

        let linked = json!({ "surfaceId": "s", "components": [
            { "id": "root", "component": "Text", "text": "see https://example.com" }
        ] });
        let refused = score(insight, &linked, &[]);
        assert!(refused.refusal.unwrap().starts_with("genui.refused"));
    }

    #[test]
    fn a_report_totals_its_rows() {
        let mut report = Report::default();
        let scenario = &scenarios()[0];
        report.add(
            "ok",
            scenario.score(&json!({ "surfaceId": "s", "components": [
            { "id": "root", "component": "Metric", "label": "Revenue", "value": 4 }
        ] })),
        );
        report.add("bad", scenario.score(&json!({ "components": "nope" })));
        assert!((report.accepted_rate() - 0.5).abs() < f64::EPSILON);
        let table = report.to_string();
        assert!(
            table.contains("| ok | yes |") && table.contains("2 drawings · 50% shown"),
            "{table}"
        );
        assert_eq!(scenarios().len(), 6);
    }
}
