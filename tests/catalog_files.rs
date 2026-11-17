//! The catalog documents under `catalogs/`: what a third-party A2UI renderer reads, and what
//! the React package's renderers are checked against. They are written from the Rust catalogs
//! with `ADK_UI_WRITE_CATALOGS=1 cargo test --test catalog_files`, and this test fails when
//! they drift.

use std::path::PathBuf;

use adk_surface::genui::Catalog;
use serde_json::{Value, json};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("catalogs")
}

fn file_name(catalog: &Catalog) -> String {
    let (name, version) = catalog
        .label()
        .split_once(' ')
        .map(|(n, v)| (n.to_string(), v.to_string()))
        .unwrap();
    format!("adk-ui-{name}-{version}.json")
}

fn pretty(value: &Value) -> String {
    format!("{}\n", serde_json::to_string_pretty(value).unwrap())
}

#[test]
fn the_catalog_documents_match_the_rust_catalogs() {
    let catalogs = adk_ui::catalogs::all();
    let mut files: Vec<(PathBuf, String)> = catalogs
        .iter()
        .map(|catalog| (root().join(file_name(catalog)), pretty(catalog.document())))
        .collect();
    let index = json!({
        "catalogs": catalogs.iter().map(|catalog| {
            let mut extensions: Vec<&str> = catalog.extensions().collect();
            extensions.sort_unstable();
            // What the home draws natively (3D scenes, depth): a host renderer is never needed.
            let home_drawn: Vec<&str> = extensions
                .iter()
                .copied()
                .filter(|name| adk_ui::catalogs::HOME_DRAWN.contains(name))
                .collect();
            json!({
                "id": catalog.id(),
                "label": catalog.label(),
                "file": file_name(catalog),
                "extensions": extensions,
                "homeDrawn": home_drawn,
            })
        }).collect::<Vec<_>>()
    });
    files.push((root().join("index.json"), pretty(&index)));

    if std::env::var_os("ADK_UI_WRITE_CATALOGS").is_some() {
        std::fs::create_dir_all(root()).unwrap();
        for (path, text) in &files {
            std::fs::write(path, text).unwrap();
        }
        return;
    }
    for (path, text) in files {
        let written = std::fs::read_to_string(&path).unwrap_or_default();
        assert_eq!(
            written,
            text,
            "{} is stale: run ADK_UI_WRITE_CATALOGS=1 cargo test --test catalog_files",
            path.display()
        );
    }
}
