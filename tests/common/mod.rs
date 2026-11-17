//! The sample drawings under `fixtures/drawings/`, shared by the tests.

#![allow(dead_code)]

use std::path::PathBuf;

use adk_surface::genui::{Catalog, DrawnKind, SurfaceDraft};
use serde_json::Value;

/// One fixture: its catalog's name, `render_screen`'s arguments and `patch_surface`'s data.
pub struct Fixture {
    pub name: String,
    pub catalog: &'static Catalog,
    pub screen: Value,
    pub patches: Vec<Value>,
}

pub fn fixtures() -> Vec<Fixture> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/drawings");
    let mut out: Vec<Fixture> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .map(|path| {
            let value: Value =
                serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            let catalog = match value["catalog"].as_str() {
                Some("insight") => adk_ui::catalogs::insight(),
                Some("studio") => adk_ui::catalogs::studio(),
                other => panic!("{}: unknown catalog {other:?}", path.display()),
            };
            Fixture {
                name: path.file_stem().unwrap().to_string_lossy().into_owned(),
                catalog,
                screen: value["screen"].clone(),
                patches: value["patches"].as_array().cloned().unwrap_or_default(),
            }
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// `render_screen`'s arguments as the draft `genui` checks.
pub fn draft(screen: &Value) -> SurfaceDraft {
    SurfaceDraft {
        surface_id: screen["surfaceId"].as_str().unwrap().to_string(),
        kind: DrawnKind::Screen,
        title: screen["title"].as_str().map(str::to_string),
        components: screen["components"].as_array().cloned().unwrap(),
        data_model: screen["dataModel"].clone(),
        actions: serde_json::from_value(screen["actions"].clone()).unwrap_or_default(),
    }
}
