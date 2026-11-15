//! adk-ui's extension catalogs, each `forms` 1.0 plus its components, and how a host
//! registers them where drawings are shown.
//!
//! A2UI gives a surface one catalog, so a catalog id always names one schema: a change of
//! schema takes a new id, and the old id stays here so earlier drawings are still checked.

use std::sync::LazyLock;

use adk_surface::SurfaceEngineBuilder;
use adk_surface::genui::{Catalog, ExtensionSpec, GenuiAgUi};

/// The insight catalog's id.
pub const INSIGHT_ID: &str = "https://zavora.ai/a2ui/catalogs/adk-ui-insight/1.0/catalog.json";

static INSIGHT: LazyLock<Catalog> = LazyLock::new(|| {
    Catalog::extend(ExtensionSpec {
        id: INSIGHT_ID,
        name: "insight",
        version: "1.0",
        components: crate::insight::components(),
    })
    .unwrap_or_else(|error| panic!("adk-ui's insight catalog does not compile: {error}"))
});

/// The insight catalog: `forms` 1.0 plus charts, sparklines, metrics, key-value lists,
/// progress and badges.
///
/// ```rust
/// let insight = adk_ui::catalogs::insight();
/// assert_eq!(insight.label(), "insight 1.0");
/// assert!(insight.has_component("Chart") && insight.has_component("TextField"));
/// ```
pub fn insight() -> &'static Catalog {
    &INSIGHT
}

/// Every adk-ui catalog, for a home to register.
pub fn all() -> Vec<&'static Catalog> {
    vec![insight()]
}

/// `builder` showing drawings in every adk-ui catalog.
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use adk_surface::{LocalSource, SurfaceEngine};
///
/// # fn example(source: LocalSource) {
/// let engine = adk_ui::catalogs::register(SurfaceEngine::builder(Arc::new(source))).build();
/// # }
/// ```
pub fn register(builder: SurfaceEngineBuilder) -> SurfaceEngineBuilder {
    all()
        .into_iter()
        .fold(builder, SurfaceEngineBuilder::with_catalog)
}

/// AG-UI's drawing activities, carrying drawings in every adk-ui catalog.
pub fn ag_ui() -> GenuiAgUi {
    all()
        .into_iter()
        .fold(GenuiAgUi::new(), GenuiAgUi::with_catalog)
}
