//! Builders for the insight components: each produces one A2UI component, as a JSON value, for
//! a [`Screen`](crate::draft::Screen) or an agent's `render_screen` call.

use serde_json::{Value, json};

use crate::build::{Props, component_builder};

/// A data binding: `{ "path": path }`, the value at a data model path. Use it wherever a
/// component takes text, a value or data.
///
/// ```rust
/// assert_eq!(adk_ui::bind("/rows"), serde_json::json!({ "path": "/rows" }));
/// ```
pub fn bind(path: impl Into<String>) -> Value {
    json!({ "path": path.into() })
}

/// How a [`Chart`] is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartKind {
    /// Bars side by side.
    Bar,
    /// Points joined by lines.
    Line,
    /// Lines with the area under them filled.
    Area,
    /// Slices of one whole, from the first series.
    Pie,
    /// Points only.
    Scatter,
}

impl ChartKind {
    fn name(self) -> &'static str {
        match self {
            Self::Bar => "bar",
            Self::Line => "line",
            Self::Area => "area",
            Self::Pie => "pie",
            Self::Scatter => "scatter",
        }
    }
}

/// What a tone a component takes means; the host draws each in its own colours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// No emphasis.
    Neutral,
    /// For information.
    Info,
    /// Went well.
    Success,
    /// Needs attention.
    Warning,
    /// Went wrong.
    Danger,
}

impl Tone {
    fn name(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Danger => "danger",
        }
    }
}

/// A chart's horizontal axis: the row field it reads and what that field holds.
#[derive(Debug, Clone, PartialEq)]
pub struct XAxis(Value);

impl XAxis {
    /// Named categories: the field's text names each place.
    pub fn category(key: impl Into<String>) -> Self {
        Self(json!({ "key": key.into(), "type": "category" }))
    }

    /// Numbers.
    pub fn number(key: impl Into<String>) -> Self {
        Self(json!({ "key": key.into(), "type": "number" }))
    }

    /// Times: epoch milliseconds, RFC 3339 times or `YYYY-MM-DD` dates.
    pub fn time(key: impl Into<String>) -> Self {
        Self(json!({ "key": key.into(), "type": "time" }))
    }

    /// The axis's name.
    #[must_use]
    pub fn label(mut self, label: impl Into<Value>) -> Self {
        self.0["label"] = label.into();
        self
    }
}

/// A chart written as a Vega-Lite spec (a subset: one mark over rows, `x`, `y`, `color`,
/// `theta`), drawn as the home's own chart.
///
/// ```rust
/// use adk_ui::insight::{VegaLite, bind};
/// use serde_json::json;
///
/// let chart = VegaLite::new("latency", json!({
///     "mark": "line",
///     "encoding": { "x": { "field": "day", "type": "temporal" },
///                   "y": { "field": "ms", "type": "quantitative" } }
/// }))
/// .data(bind("/latency"))
/// .build();
/// assert_eq!(chart["component"], "VegaLite");
/// assert_eq!(chart["data"]["path"], "/latency");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct VegaLite(Props);

impl VegaLite {
    /// A Vega-Lite chart with id `id` drawing `spec`.
    pub fn new(id: impl Into<String>, spec: Value) -> Self {
        let mut props = Props::new(id, "VegaLite");
        props.set("spec", spec);
        Self(props)
    }

    /// Its title, over the spec's.
    #[must_use]
    pub fn title(mut self, title: impl Into<Value>) -> Self {
        self.0.set("title", title);
        self
    }

    /// Its rows, a [`bind`]ing in place of the spec's `data.values`.
    #[must_use]
    pub fn data(mut self, data: impl Into<Value>) -> Self {
        self.0.set("data", data);
        self
    }
}

component_builder!(VegaLite);

/// A chart of rows of data.
///
/// ```rust
/// use adk_ui::insight::{Chart, ChartKind, XAxis};
///
/// let chart = Chart::new("sales", ChartKind::Line)
///     .title("Revenue")
///     .data(adk_ui::bind("/sales"))
///     .x(XAxis::category("month"))
///     .series("revenue", "Revenue")
///     .build();
/// assert_eq!(chart["series"][0]["key"], "revenue");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Chart(Props);

impl Chart {
    /// A chart with id `id`; give it data, an x axis and at least one series.
    pub fn new(id: impl Into<String>, kind: ChartKind) -> Self {
        let mut props = Props::new(id, "Chart");
        props.set("kind", kind.name());
        props.set("series", json!([]));
        Self(props)
    }

    /// Its title.
    #[must_use]
    pub fn title(mut self, title: impl Into<Value>) -> Self {
        self.0.set("title", title);
        self
    }

    /// Its rows: a [`bind`]ing (best) or a literal list of row objects, at most 500.
    #[must_use]
    pub fn data(mut self, data: impl Into<Value>) -> Self {
        self.0.set("data", data);
        self
    }

    /// Its horizontal axis.
    #[must_use]
    pub fn x(mut self, axis: XAxis) -> Self {
        self.0.set("x", axis.0);
        self
    }

    /// A series: the row field `key`, named `label` in the legend. At most eight.
    #[must_use]
    pub fn series(mut self, key: impl Into<String>, label: impl Into<Value>) -> Self {
        if let Some(Value::Array(series)) = self.0.0.get_mut("series") {
            series.push(json!({ "key": key.into(), "label": label.into() }));
        }
        self
    }

    /// The vertical axis's name.
    #[must_use]
    pub fn y_label(mut self, label: impl Into<Value>) -> Self {
        self.0.set("yLabel", label);
        self
    }

    /// Values as percentages.
    #[must_use]
    pub fn percent(mut self) -> Self {
        self.0.set("valueFormat", "percent");
        self
    }

    /// Values as amounts of `currency`, an ISO 4217 code such as `USD`.
    #[must_use]
    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.0.set("valueFormat", "currency");
        self.0.set("currency", currency.into());
        self
    }

    /// Only the last `rows` rows.
    #[must_use]
    pub fn window(mut self, rows: u32) -> Self {
        self.0.set("window", rows);
        self
    }

    /// Bars or areas stacked; for hosts that draw it.
    #[must_use]
    pub fn stacked(mut self) -> Self {
        self.0.set("stacked", true);
        self
    }
}

component_builder!(Chart);

/// A small line of numbers.
#[derive(Debug, Clone, PartialEq)]
pub struct Sparkline(Props);

impl Sparkline {
    /// A sparkline of `data`: a [`bind`]ing to numbers, or a literal list of them.
    pub fn new(id: impl Into<String>, data: impl Into<Value>) -> Self {
        let mut props = Props::new(id, "Sparkline");
        props.set("data", data);
        Self(props)
    }

    /// Its label.
    #[must_use]
    pub fn label(mut self, label: impl Into<Value>) -> Self {
        self.0.set("label", label);
        self
    }

    /// The field of each row its numbers are in, when its data is rows.
    #[must_use]
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.0.set("key", key.into());
        self
    }

    /// Its tone.
    #[must_use]
    pub fn tone(mut self, tone: Tone) -> Self {
        self.0.set("tone", tone.name());
        self
    }
}

component_builder!(Sparkline);

/// One number with its label.
///
/// ```rust
/// use adk_ui::insight::Metric;
///
/// let metric = Metric::new("open", "Open incidents", 3).delta(-2.0, false).build();
/// assert_eq!(metric["delta"]["goodWhen"], "down");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Metric(Props);

impl Metric {
    /// A metric labelled `label` showing `value`: a number, text or a [`bind`]ing.
    pub fn new(id: impl Into<String>, label: impl Into<Value>, value: impl Into<Value>) -> Self {
        let mut props = Props::new(id, "Metric");
        props.set("label", label);
        props.set("value", value);
        Self(props)
    }

    /// The unit after the value, such as `ms`.
    #[must_use]
    pub fn unit(mut self, unit: impl Into<Value>) -> Self {
        self.0.set("unit", unit);
        self
    }

    /// The value as a percentage.
    #[must_use]
    pub fn percent(mut self) -> Self {
        self.0.set("format", "percent");
        self
    }

    /// The value as an amount of `currency`, such as `USD`.
    #[must_use]
    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.0.set("format", "currency");
        self.0.set("currency", currency.into());
        self
    }

    /// Its change since the last reading; `up_is_good` says which way is better.
    #[must_use]
    pub fn delta(mut self, change: f64, up_is_good: bool) -> Self {
        self.0.set(
            "delta",
            json!({ "value": change, "goodWhen": if up_is_good { "up" } else { "down" } }),
        );
        self
    }

    /// A line under it.
    #[must_use]
    pub fn caption(mut self, caption: impl Into<Value>) -> Self {
        self.0.set("caption", caption);
        self
    }

    /// Its recent values: a [`bind`]ing to numbers, or a literal list of them.
    #[must_use]
    pub fn trend(mut self, trend: impl Into<Value>) -> Self {
        self.0.set("trend", trend);
        self
    }

    /// Its status: `ok`, `warning`, `critical`, `info` or `neutral`.
    #[must_use]
    pub fn status(mut self, status: &str) -> Self {
        self.0.set("status", status);
        self
    }
}

component_builder!(Metric);

/// Labelled values.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyValue(Props);

impl KeyValue {
    /// An empty list; add pairs, or bind it to rows of data.
    pub fn new(id: impl Into<String>) -> Self {
        Self(Props::new(id, "KeyValue"))
    }

    /// Its title.
    #[must_use]
    pub fn title(mut self, title: impl Into<Value>) -> Self {
        self.0.set("title", title);
        self
    }

    /// A pair. At most fifty.
    #[must_use]
    pub fn pair(mut self, key: impl Into<Value>, value: impl Into<Value>) -> Self {
        let entry = json!({ "key": key.into(), "value": value.into() });
        match self.0.0.get_mut("items") {
            Some(Value::Array(items)) => items.push(entry),
            _ => self.0.set("items", json!([entry])),
        }
        self
    }

    /// Pairs from rows of data: each row's `key_field` and `value_field`.
    #[must_use]
    pub fn rows(
        mut self,
        data: impl Into<Value>,
        key_field: impl Into<String>,
        value_field: impl Into<String>,
    ) -> Self {
        self.0.set("data", data);
        self.0.set("keyField", key_field.into());
        self.0.set("valueField", value_field.into());
        self
    }
}

component_builder!(KeyValue);

/// How far a piece of work has come.
#[derive(Debug, Clone, PartialEq)]
pub struct Progress(Props);

impl Progress {
    /// `label` at `value` out of 100: a number or a [`bind`]ing.
    pub fn new(id: impl Into<String>, label: impl Into<Value>, value: impl Into<Value>) -> Self {
        let mut props = Props::new(id, "Progress");
        props.set("label", label);
        props.set("value", value);
        Self(props)
    }

    /// Work whose end is not known yet.
    pub fn indeterminate(id: impl Into<String>, label: impl Into<Value>) -> Self {
        let mut props = Props::new(id, "Progress");
        props.set("label", label);
        props.set("indeterminate", true);
        Self(props)
    }

    /// Its tone.
    #[must_use]
    pub fn tone(mut self, tone: Tone) -> Self {
        self.0.set("tone", tone.name());
        self
    }
}

component_builder!(Progress);

/// A short status word.
#[derive(Debug, Clone, PartialEq)]
pub struct Badge(Props);

impl Badge {
    /// A badge reading `text`, never a decision's word.
    pub fn new(id: impl Into<String>, text: impl Into<Value>, tone: Tone) -> Self {
        let mut props = Props::new(id, "Badge");
        props.set("text", text);
        props.set("tone", tone.name());
        Self(props)
    }
}

component_builder!(Badge);
