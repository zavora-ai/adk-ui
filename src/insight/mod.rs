//! The insight components (catalog [`INSIGHT_ID`](crate::catalogs::INSIGHT_ID)): charts,
//! sparklines, metrics, key-value lists, progress and badges. Each only shows; the web home
//! draws charts natively and the rest as `forms` components, and a custom host may draw
//! them richly with `@zavora-ai/adk-ui-react`.

mod build;
mod components;
mod format;

pub use build::{
    Badge, Chart, ChartKind, KeyValue, Metric, Progress, Sparkline, Tone, XAxis, bind,
};
pub use components::{MAX_CHART_ROWS, MAX_CHART_SERIES, MAX_PAIRS, components};
