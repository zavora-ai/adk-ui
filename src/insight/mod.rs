//! The insight components (catalog [`INSIGHT_ID`](crate::catalogs::INSIGHT_ID)): charts,
//! Vega-Lite charts, sparklines, metrics, key-value lists, progress and badges. Each only
//! shows; the web home draws charts (Vega-Lite ones included) natively and the rest as
//! `forms` components, and a custom host may draw them richly with `@zavora-ai/adk-ui-react`.

mod build;
mod components;
pub(crate) mod format;
mod vega;

pub use build::{
    Badge, Chart, ChartKind, KeyValue, Metric, Progress, Sparkline, Tone, VegaLite, XAxis, bind,
};
pub use components::{MAX_CHART_ROWS, MAX_CHART_SERIES, MAX_PAIRS, components};
