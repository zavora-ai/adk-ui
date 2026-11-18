//! The catalog documents under `catalogs/`: what a third-party A2UI renderer reads, and what
//! the React package's renderers are checked against. `catalogs/*.json` are the A2UI v0.9.1
//! documents (each catalog whole, `forms` included); `catalogs/a2ui-1.0/*.json` are A2UI 1.0
//! documents (only what each catalog adds, which a surface mixes with `forms`). They are written from the Rust catalogs
//! with `ADK_UI_WRITE_CATALOGS=1 cargo test --test catalog_files`, and this test fails when
//! they drift.

use std::path::PathBuf;

use adk_surface::genui::{A2uiVersion, Catalog};
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
    for catalog in &catalogs {
        files.push((
            root().join("a2ui-1.0").join(file_name(catalog)),
            pretty(&catalog.document_for(A2uiVersion::V1_0)),
        ));
    }
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
                "a2ui1_0File": format!("a2ui-1.0/{}", file_name(catalog)),
                "extensions": extensions,
                "homeDrawn": home_drawn,
            })
        }).collect::<Vec<_>>()
    });
    files.push((root().join("index.json"), pretty(&index)));

    if std::env::var_os("ADK_UI_WRITE_CATALOGS").is_some() {
        std::fs::create_dir_all(root().join("a2ui-1.0")).unwrap();
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

/// Every A2UI 1.0 catalog document passes the A2UI 1.0 catalog meta-schema (pinned in
/// `tests/a2ui-1.0/`, from a2ui-project/a2ui@de4a91688524).
#[test]
fn the_a2ui_1_0_documents_pass_the_a2ui_1_0_meta_schema() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/a2ui-1.0");
    let read = |name: &str| -> Value {
        serde_json::from_str(&std::fs::read_to_string(fixtures.join(name)).unwrap()).unwrap()
    };
    let registry = jsonschema::Registry::new()
        .add(
            "https://a2ui.org/specification/v1_0/common_types.json",
            read("common_types.json"),
        )
        .and_then(|registry| {
            registry.add(
                "https://a2ui.org/specification/v1_0/catalog.json",
                json!({ "$defs": { "anyComponent": {}, "anyFunction": {} } }),
            )
        })
        .and_then(jsonschema::RegistryBuilder::prepare)
        .unwrap();
    let meta = jsonschema::options()
        .with_registry(&registry)
        .build(&read("catalog_definition.json"))
        .unwrap();
    for catalog in adk_ui::catalogs::all() {
        let document = catalog.document_for(A2uiVersion::V1_0);
        if let Some(error) = meta.iter_errors(&document).next() {
            panic!("{}: {error} at {}", catalog.label(), error.instance_path());
        }
        assert_eq!(document["protocolVersion"], "1.0");
        assert!(
            document["components"].get("TextField").is_none(),
            "only what it adds"
        );
        assert!(document["instructions"].as_str().unwrap().contains("@path"));
    }
}
