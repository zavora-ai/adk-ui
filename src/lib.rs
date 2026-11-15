//! # adk-ui
//!
//! Extension catalogs for agent-drawn UI in [ADK-Rust](https://adk-rust.com) 3.
//!
//! ADK-Rust draws agent UI with `adk-surface`'s `genui` feature: the render tools, A2UI
//! v0.9.1, a version and digest record for every drawing, rule 5 (a drawing never contains
//! approval controls, secrets or links) and the governed send (a form's **Send** asks for a
//! call the person decides). adk-ui 3 adds what `genui`'s `forms` catalog leaves out, as
//! extension catalogs `genui` checks and draws:
//!
//! | Part | What |
//! |---|---|
//! | [`catalogs`] | The catalogs: [`catalogs::insight`] (charts, sparklines, metrics, key-value lists, progress, badges) and [`catalogs::studio`] (insight, plus apps of pages, regions and primitive 3D scenes) |
//! | [`insight`] | Builders for the insight components |
//! | [`studio`] | Builders for the studio components |
//! | [`draft`] | A screen under construction, checked as `genui` checks it |
//! | [`toolset`] | `genui`'s render tools drawing in an adk-ui catalog |
//! | [`prompts`] | What to tell a model that draws in one |
//! | [`kit`] | Brand kits: approved design tokens a host applies to its own pages ([`kit::ApprovedKit`]) |
//!
//! adk-ui adds no render tool name, protocol, provenance record or approval path: an agent
//! draws with `genui`'s `render_screen`, the drawing is recorded and checked by `genui`, and a
//! home shows it once its catalog is registered ([`catalogs::register`]).
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use adk_surface::{LocalSource, SurfaceEngine};
//!
//! # fn example(source: LocalSource) {
//! // The agent's render tools draw in the insight catalog...
//! let tools = adk_ui::toolset();
//! // ...and the home shows what they draw.
//! let engine = adk_ui::catalogs::register(SurfaceEngine::builder(Arc::new(source))).build();
//! # let _ = (tools, engine);
//! # }
//! ```
//!
//! adk-ui 2.x, with its own render tools and protocol adapters, stays on crates.io for
//! ADK-Rust 2.

pub mod catalogs;
pub mod draft;
pub mod insight;
pub mod kit;
pub mod prompts;
mod schema;
pub mod studio;

use adk_surface::genui::{Catalog, DrawToolset, Validator};

pub use insight::bind;
pub use kit::{BrandKitManifest, KitArtifacts, KitGenerator, KitSpec};

/// `genui`'s render tools drawing in the insight catalog: give them to an agent as its one
/// `DrawToolset`, and register the catalog with the home ([`catalogs::register`]).
pub fn toolset() -> DrawToolset {
    toolset_for(catalogs::insight())
}

/// `genui`'s render tools drawing in `catalog`.
pub fn toolset_for(catalog: &'static Catalog) -> DrawToolset {
    DrawToolset::new(Validator::new(catalog))
}
