//! # adk-ui
//!
//! Extension catalogs for agent-drawn UI in [ADK-Rust](https://adk-rust.com) 3.
//!
//! ADK-Rust draws agent UI with `adk-surface`'s `genui` feature: the render tools, A2UI 1.0
//! on the wire (v0.9.1 for renderers that offer nothing newer), a version and digest record
//! for every drawing, rule 5 (a drawing never contains approval controls, secrets or links)
//! and the governed send (a form's **Send** asks for a call the person decides). adk-ui 3 adds
//! what `genui`'s `forms` catalog leaves out, as extension catalogs `genui` checks and draws:
//!
//! | Part | What |
//! |---|---|
//! | [`catalogs`] | The catalogs: [`catalogs::insight`] (charts, Vega-Lite charts, sparklines, metrics, key-value lists, progress, badges), [`catalogs::studio`] (insight, plus apps of pages, regions and primitive 3D scenes) and [`catalogs::spatial`] (studio, plus 3D charts, scenes, cards with depth and 3D layouts) |
//! | [`insight`], [`studio`], [`spatial`] | Builders for each catalog's components |
//! | [`draft`] | A screen under construction, checked as `genui` checks it |
//! | [`toolset`], [`toolset_for`] | `genui`'s render tools drawing in an adk-ui catalog |
//! | [`prompts`] | What to tell a model that draws in one |
//! | [`eval`] | Scoring drawings the way `genui` will judge them |
//! | [`kit`] | Brand kits: approved design tokens a host applies to its own pages ([`kit::ApprovedKit`]) |
//! | `apps` | With the `apps` feature: an MCP Apps server, with model-written three.js in genui-apps' sandbox and checked drawings read-only in any MCP Apps host |
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
//! How the pieces fit: `docs/ARCHITECTURE.md`. adk-ui 2.x, with its own render tools and
//! protocol adapters, stays on crates.io for ADK-Rust 2.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "apps")]
#[cfg_attr(docsrs, doc(cfg(feature = "apps")))]
pub mod apps;
mod build;
pub mod catalogs;
pub mod draft;
pub mod eval;
pub mod insight;
pub mod kit;
pub mod prompts;
mod scene;
mod schema;
pub mod spatial;
pub mod studio;

use adk_surface::genui::{Catalog, DrawToolset, Validator};

pub use insight::bind;

/// `genui`'s render tools drawing in the insight catalog: give them to an agent as its one
/// `DrawToolset`, and register the catalog with the home ([`catalogs::register`]).
pub fn toolset() -> DrawToolset {
    toolset_for(catalogs::insight())
}

/// `genui`'s render tools drawing in `catalog`.
pub fn toolset_for(catalog: &'static Catalog) -> DrawToolset {
    DrawToolset::new(Validator::new(catalog))
}
