//! adk-ui's extension catalogs, each `forms` 1.0 plus its components, and how a host
//! registers them where drawings are shown.
//!
//! A2UI gives a surface one catalog, so a catalog id always names one schema: a change of
//! schema takes a new id, and the old id stays here so earlier drawings are still checked.

use std::sync::LazyLock;

use adk_surface::SurfaceEngineBuilder;
use adk_surface::genui::{Catalog, ExtensionSpec, GenuiAgUi};

/// A catalog with its rules (`prompts::catalog_rules`) as its A2UI 1.0 `instructions`.
trait WithRules {
    fn with_rules(self) -> Catalog;
}

impl WithRules for Catalog {
    fn with_rules(self) -> Catalog {
        let rules = crate::prompts::catalog_rules(&self);
        self.with_instructions(rules)
    }
}

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
    .with_rules()
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

/// The studio catalog's id.
pub const STUDIO_ID: &str = "https://zavora.ai/a2ui/catalogs/adk-ui-studio/1.0/catalog.json";

static STUDIO: LazyLock<Catalog> = LazyLock::new(|| {
    let mut components = crate::insight::components();
    components.extend(crate::studio::components());
    Catalog::extend(ExtensionSpec {
        id: STUDIO_ID,
        name: "studio",
        version: "1.0",
        components,
    })
    .unwrap_or_else(|error| panic!("adk-ui's studio catalog does not compile: {error}"))
    .with_rules()
});

/// The studio catalog: the insight catalog plus apps of pages, regions and primitive 3D
/// scenes, for hosts that embed agent-drawn applications.
///
/// ```rust
/// let studio = adk_ui::catalogs::studio();
/// assert_eq!(studio.label(), "studio 1.0");
/// assert!(studio.has_component("App") && studio.has_component("Chart"));
/// ```
pub fn studio() -> &'static Catalog {
    &STUDIO
}

/// The spatial catalog's id.
pub const SPATIAL_ID: &str = "https://zavora.ai/a2ui/catalogs/adk-ui-spatial/1.0/catalog.json";

static SPATIAL: LazyLock<Catalog> = LazyLock::new(|| {
    let mut components = crate::insight::components();
    components.extend(crate::studio::components());
    components.extend(crate::spatial::components());
    Catalog::extend(ExtensionSpec {
        id: SPATIAL_ID,
        name: "spatial",
        version: "1.0",
        components,
    })
    .unwrap_or_else(|error| panic!("adk-ui's spatial catalog does not compile: {error}"))
    .with_rules()
});

/// The spatial catalog: the studio catalog plus 3D charts, laid-out scenes with motion, panels
/// and picking into a form, cards with depth and components arranged in 3D, which the home
/// draws natively.
///
/// ```rust
/// let spatial = adk_ui::catalogs::spatial();
/// assert_eq!(spatial.label(), "spatial 1.0");
/// assert!(spatial.has_component("Globe") && spatial.has_component("App"));
/// ```
pub fn spatial() -> &'static Catalog {
    &SPATIAL
}

/// The components whose home view the home draws natively as a 3D scene or with depth, so a
/// host needs no renderer of its own for them.
pub const HOME_DRAWN: &[&str] = &[
    "Scene3D",
    "Bar3D",
    "Scatter3D",
    "SurfacePlot",
    "Graph3D",
    "Globe",
    "Scene",
    "DepthCard",
    "Stage",
];

/// Every adk-ui catalog, for a home to register: drawings made in any of them are shown.
pub fn all() -> Vec<&'static Catalog> {
    vec![insight(), studio(), spatial()]
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
