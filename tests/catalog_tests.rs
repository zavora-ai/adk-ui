use serde_json::Value;

#[test]
fn default_catalog_is_valid_json() {
    let raw = include_str!("../catalog/default_catalog.json");
    let value: Value = serde_json::from_str(raw).expect("default catalog should be valid JSON");
    let catalog_id = value.get("catalogId").and_then(Value::as_str).unwrap_or("");
    assert_eq!(catalog_id, "zavora.ai:adk-ui/default@0.1.0");
    assert!(
        value.get("components").is_some(),
        "catalog should define components"
    );
}

#[test]
fn catalog_metadata_is_valid_json() {
    let raw = include_str!("../catalog/metadata.json");
    let value: Value = serde_json::from_str(raw).expect("catalog metadata should be valid JSON");
    let catalog_id = value.get("catalogId").and_then(Value::as_str).unwrap_or("");
    assert_eq!(catalog_id, "zavora.ai:adk-ui/default@0.1.0");
    assert_eq!(
        value.get("license").and_then(Value::as_str),
        Some("Apache-2.0")
    );
}

#[test]
fn registry_resolves_default_catalog() {
    let registry = adk_ui::CatalogRegistry::default();
    let default_catalog_id = registry.default_catalog_id().to_string();
    let artifact = registry
        .load_local_catalog(&default_catalog_id)
        .expect("default catalog should resolve");
    assert_eq!(artifact.catalog_id, default_catalog_id);
    assert!(artifact.catalog.get("components").is_some());
    assert!(artifact.metadata.is_some());
}

#[tokio::test]
async fn registry_blocks_absolute_remote_catalog_urls_by_default() {
    let registry = adk_ui::CatalogRegistry::default();
    let error = registry
        .resolve_catalog("https://example.com/catalog.json")
        .await
        .expect_err("absolute URLs should require an explicit opt-in");

    match error {
        adk_ui::CatalogError::Remote(message) => {
            assert!(message.contains("absolute remote catalog URLs are disabled"));
        }
        other => panic!("expected remote URL guard, got {other:?}"),
    }
}

#[cfg(not(feature = "remote-catalogs"))]
#[tokio::test]
async fn registry_allows_opted_in_absolute_remote_catalog_urls() {
    let registry = adk_ui::CatalogRegistry::default().with_allow_absolute_remote_urls(true);
    let error = registry
        .resolve_catalog("https://example.com/catalog.json")
        .await
        .expect_err("remote fetching is disabled in the default test build");

    assert!(matches!(error, adk_ui::CatalogError::RemoteDisabled));
}
