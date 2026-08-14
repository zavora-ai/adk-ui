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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kit_id: Option<String>,
    pub initial_route: String,
    #[serde(default)]
    pub navigation: Vec<UiNavigationItem>,
    pub pages: Vec<UiApplicationPage>,
    #[serde(default)]
    pub state: HashMap<String, Value>,
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
                application.validate()?;
                *self = application;
                Ok(Some(self.initial_route.clone()))
            }
            UiApplicationUpdate::UpsertPage { page } => {
                let mut candidate = self.clone();
                if let Some(existing) = candidate
                    .pages
                    .iter_mut()
                    .find(|candidate| candidate.id == page.id)
                {
                    *existing = page;
                } else {
                    candidate.pages.push(page);
                }
                candidate.validate()?;
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
                *self = candidate;
                Ok(None)
            }
            UiApplicationUpdate::PatchState { values } => {
                self.state.extend(values);
                Ok(None)
            }
        }
    }
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
}
