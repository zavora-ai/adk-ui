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
//! | [`kit`] | Brand kits: approved design tokens a host applies to its own pages |
//!
//! The catalogs (data visualisation, pages and scenes), their builders and the toolset arrive
//! in the following changes of the 3.0 line. adk-ui adds no render tool name, protocol,
//! provenance record or approval path: those stay in `genui`.
//!
//! adk-ui 2.x, with its own render tools and protocol adapters, stays on crates.io for
//! ADK-Rust 2.

pub mod kit;

pub use kit::{BrandKitManifest, KitArtifacts, KitGenerator, KitSpec};
