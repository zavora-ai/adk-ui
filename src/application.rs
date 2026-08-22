//! Multi-page application schema and deterministic runtime updates.
//!
//! `UiApplication` is additive to `UiResponse`: hosts that only understand a single
//! surface can continue rendering page components, while application-aware hosts
//! gain routing, page composition, and state-preserving navigation.

use crate::schema::Component;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub const MAX_APPLICATION_PAGES: usize = 24;
pub const MAX_PAGE_REGIONS: usize = 16;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UiApplication {
    pub id: String,
    pub name: String,
    #[serde(default = "default_application_version")]
    pub version: u64,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kit_id: Option<String>,
    pub initial_route: String,
    #[serde(default)]
    pub navigation: Vec<UiNavigationItem>,
    pub pages: Vec<UiApplicationPage>,
    #[serde(default)]
    pub state: HashMap<String, Value>,
}

fn default_application_version() -> u64 {
    1
}

fn timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UiNavigationItem {
    pub label: String,
    pub route: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UiApplicationPage {
    pub id: String,
    pub route: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub template: UiPageTemplate,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eyebrow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hero: Option<UiPageHero>,
    pub regions: Vec<UiPageRegion>,
    #[serde(default)]
    pub aside: Vec<Component>,
    #[serde(default)]
    pub footer: Vec<Component>,
    #[serde(default)]
    pub atmosphere: UiPageAtmosphere,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiPageTemplate {
    Marketing,
    #[default]
    Dashboard,
    Workspace,
    Detail,
    Form,
    Immersive,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UiPageHero {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub content: Vec<Component>,
    #[serde(default)]
    pub actions: Vec<Component>,
    #[serde(default)]
    pub visual: Vec<Component>,
    #[serde(default)]
    pub alignment: UiHeroAlignment,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiHeroAlignment {
    #[default]
    Split,
    Center,
    Editorial,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UiPageRegion {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub layout: UiRegionLayout,
    #[serde(default = "default_region_columns")]
    pub columns: u8,
    #[serde(default)]
    pub tone: UiRegionTone,
    pub components: Vec<Component>,
}

fn default_region_columns() -> u8 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiRegionLayout {
    #[default]
    Stack,
    Grid,
    Bento,
    Split,
    Editorial,
    Stats,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiRegionTone {
    #[default]
    Default,
    Muted,
    Accent,
    Contrast,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiPageAtmosphere {
    #[default]
    Clean,
    Grid,
    Glow,
    Gradient,
    Spatial,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum UiApplicationUpdate {
    Navigate { route: String },
    Replace { application: UiApplication },
    UpsertPage { page: UiApplicationPage },
    RemovePage { page_id: String },
    PatchState { values: HashMap<String, Value> },
}

impl UiApplication {
    pub fn normalize_metadata(&mut self, owner: impl Into<String>) {
        if self.version == 0 {
            self.version = 1;
        }
        if self.owner.trim().is_empty() {
            self.owner = owner.into();
        }
        if self.updated_at.trim().is_empty() {
            self.updated_at = timestamp();
        }
    }

    fn touch(&mut self) {
        self.version = self.version.saturating_add(1).max(1);
        self.updated_at = timestamp();
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.id.trim().is_empty() {
            errors.push("id must not be empty".to_string());
        }
        if self.name.trim().is_empty() {
            errors.push("name must not be empty".to_string());
        }
        if self.pages.is_empty() {
            errors.push("pages must contain at least one page".to_string());
        }
        if self.pages.len() > MAX_APPLICATION_PAGES {
            errors.push(format!("pages must not exceed {MAX_APPLICATION_PAGES}"));
        }

        let mut ids = HashSet::new();
        let mut routes = HashSet::new();
        for (page_index, page) in self.pages.iter().enumerate() {
            if page.id.trim().is_empty() || !ids.insert(page.id.as_str()) {
                errors.push(format!(
                    "pages[{page_index}].id must be non-empty and unique"
                ));
            }
            if !is_route(&page.route) || !routes.insert(page.route.as_str()) {
                errors.push(format!(
                    "pages[{page_index}].route must be a unique absolute application route"
                ));
            }
            if page.title.trim().is_empty() {
                errors.push(format!("pages[{page_index}].title must not be empty"));
            }
            if page.regions.len() > MAX_PAGE_REGIONS {
                errors.push(format!(
                    "pages[{page_index}].regions must not exceed {MAX_PAGE_REGIONS}"
                ));
            }
            let mut region_ids = HashSet::new();
            for (region_index, region) in page.regions.iter().enumerate() {
                if region.id.trim().is_empty() || !region_ids.insert(region.id.as_str()) {
                    errors.push(format!("pages[{page_index}].regions[{region_index}].id must be non-empty and unique"));
                }
                if !(1..=6).contains(&region.columns) {
                    errors.push(format!("pages[{page_index}].regions[{region_index}].columns must be between 1 and 6"));
                }
            }
        }

        if !routes.contains(self.initial_route.as_str()) {
            errors.push("initial_route must reference an application page".to_string());
        }
        for (index, item) in self.navigation.iter().enumerate() {
            if item.label.trim().is_empty() {
                errors.push(format!("navigation[{index}].label must not be empty"));
            }
            if !routes.contains(item.route.as_str()) {
                errors.push(format!(
                    "navigation[{index}].route must reference an application page"
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn apply_update(
        &mut self,
        update: UiApplicationUpdate,
    ) -> Result<Option<String>, Vec<String>> {
        match update {
            UiApplicationUpdate::Navigate { route } => {
                if self.pages.iter().any(|page| page.route == route) {
                    Ok(Some(route))
                } else {
                    Err(vec![
                        "navigate route must reference an application page".to_string(),
                    ])
                }
            }
            UiApplicationUpdate::Replace { application } => {
                let mut application = application;
                application.validate()?;
                application.version = self.version.saturating_add(1).max(1);
                if application.owner.trim().is_empty() {
                    application.owner = self.owner.clone();
                }
                application.updated_at = timestamp();
                *self = application;
                Ok(Some(self.initial_route.clone()))
            }
            UiApplicationUpdate::UpsertPage { page } => {
                let mut candidate = self.clone();
                if let Some(index) = candidate
                    .pages
                    .iter()
                    .position(|candidate| candidate.id == page.id)
                {
                    let previous_route = candidate.pages[index].route.clone();
                    candidate.pages[index] = page;
                    let next_route = candidate.pages[index].route.clone();
                    if previous_route != next_route {
                        if candidate.initial_route == previous_route {
                            candidate.initial_route = next_route.clone();
                        }
                        for item in &mut candidate.navigation {
                            if item.route == previous_route {
                                item.route = next_route.clone();
                            }
                        }
                    }
                } else {
                    candidate.pages.push(page);
                }
                candidate.validate()?;
                candidate.touch();
                *self = candidate;
                Ok(None)
            }
            UiApplicationUpdate::RemovePage { page_id } => {
                let mut candidate = self.clone();
                candidate.pages.retain(|page| page.id != page_id);
                candidate
                    .navigation
                    .retain(|item| candidate.pages.iter().any(|page| page.route == item.route));
                candidate.validate()?;
                candidate.touch();
                *self = candidate;
                Ok(None)
            }
            UiApplicationUpdate::PatchState { values } => {
                self.state.extend(values);
                self.touch();
                Ok(None)
            }
        }
    }
}

/// Atomically upsert a page in a persisted application surface.
pub async fn upsert_persisted_application_page(
    store: &dyn crate::persistence::SurfaceStore,
    owner: &str,
    application_id: &str,
    expected_version: u64,
    page: UiApplicationPage,
) -> Result<UiApplication, crate::persistence::SurfaceStoreError> {
    let saved = store.load(owner, application_id).await?;
    if saved.version != expected_version {
        return Err(crate::persistence::SurfaceStoreError::VersionConflict {
            expected: expected_version,
            actual: saved.version,
        });
    }
    let mut application: UiApplication = serde_json::from_value(saved.payload)
        .map_err(|error| crate::persistence::SurfaceStoreError::Json(error.to_string()))?;
    application
        .apply_update(UiApplicationUpdate::UpsertPage { page })
        .map_err(|errors| crate::persistence::SurfaceStoreError::Json(errors.join("; ")))?;
    application.owner = owner.to_string();
    let payload = serde_json::to_value(&application)
        .map_err(|error| crate::persistence::SurfaceStoreError::Json(error.to_string()))?;
    let persisted = store
        .save(
            owner,
            application_id,
            &saved.name,
            payload,
            Some(expected_version),
        )
        .await?;
    application.version = persisted.version;
    application.updated_at = persisted.updated_at;
    Ok(application)
}

fn is_route(route: &str) -> bool {
    route.starts_with('/')
        && !route.starts_with("//")
        && !route.contains("..")
        && !route.contains(['?', '#', '\\'])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(id: &str, route: &str) -> UiApplicationPage {
        UiApplicationPage {
            id: id.to_string(),
            route: route.to_string(),
            title: id.to_string(),
            description: None,
            template: UiPageTemplate::Dashboard,
            eyebrow: None,
            hero: None,
            regions: vec![],
            aside: vec![],
            footer: vec![],
            atmosphere: UiPageAtmosphere::Clean,
        }
    }

    #[test]
    fn validates_multi_page_application() {
        let app = UiApplication {
            id: "ops".into(),
            name: "Operations".into(),
            version: 1,
            owner: "test".into(),
            updated_at: timestamp(),
            kit_id: None,
            initial_route: "/".into(),
            navigation: vec![UiNavigationItem {
                label: "Home".into(),
                route: "/".into(),
                icon: None,
            }],
            pages: vec![page("home", "/"), page("detail", "/detail")],
            state: HashMap::new(),
        };
        assert!(app.validate().is_ok());
    }

    #[test]
    fn rejects_unknown_navigation_route() {
        let mut app = UiApplication {
            id: "ops".into(),
            name: "Operations".into(),
            version: 1,
            owner: "test".into(),
            updated_at: timestamp(),
            kit_id: None,
            initial_route: "/".into(),
            navigation: vec![UiNavigationItem {
                label: "Missing".into(),
                route: "/missing".into(),
                icon: None,
            }],
            pages: vec![page("home", "/")],
            state: HashMap::new(),
        };
        assert!(app.validate().is_err());
        assert!(
            app.apply_update(UiApplicationUpdate::Navigate {
                route: "/missing".into()
            })
            .is_err()
        );
    }

    #[test]
    fn upsert_page_rewrites_routes_and_increments_version() {
        let mut app = UiApplication {
            id: "ops".into(),
            name: "Operations".into(),
            version: 3,
            owner: "agent".into(),
            updated_at: timestamp(),
            kit_id: None,
            initial_route: "/".into(),
            navigation: vec![UiNavigationItem {
                label: "Home".into(),
                route: "/".into(),
                icon: None,
            }],
            pages: vec![page("home", "/")],
            state: HashMap::new(),
        };
        app.apply_update(UiApplicationUpdate::UpsertPage {
            page: page("home", "/home"),
        })
        .unwrap();
        assert_eq!(app.version, 4);
        assert_eq!(app.initial_route, "/home");
        assert_eq!(app.navigation[0].route, "/home");
    }

    #[tokio::test]
    async fn persisted_upsert_uses_optimistic_versioning() {
        let store = crate::persistence::InMemorySurfaceStore::new();
        let app = UiApplication {
            id: "ops".into(),
            name: "Operations".into(),
            version: 1,
            owner: "agent".into(),
            updated_at: timestamp(),
            kit_id: None,
            initial_route: "/".into(),
            navigation: vec![],
            pages: vec![page("home", "/")],
            state: HashMap::new(),
        };
        let payload = serde_json::to_value(&app).unwrap();
        crate::persistence::SurfaceStore::save(
            &store,
            "agent",
            "ops",
            "Operations",
            payload,
            Some(0),
        )
        .await
        .unwrap();
        let updated =
            upsert_persisted_application_page(&store, "agent", "ops", 1, page("detail", "/detail"))
                .await
                .unwrap();
        assert_eq!(updated.version, 2);
        assert_eq!(updated.pages.len(), 2);
    }
}
