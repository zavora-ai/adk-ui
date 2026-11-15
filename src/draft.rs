//! A screen under construction: its components, data model and actions, as `genui`'s
//! `render_screen` takes them, and checked as `genui` checks them.

use adk_surface::genui::{
    DrawRefusal, DrawnAction, DrawnKind, SurfaceDraft, ValidatedSurface, Validator,
};
use serde_json::{Value, json};

/// A screen: components (one with id `root`), a data model and the actions its buttons ask
/// for.
///
/// ```rust
/// use adk_ui::draft::{Screen, column, text};
/// use adk_ui::insight::Metric;
/// use adk_surface::genui::Validator;
///
/// let screen = Screen::new("incidents")
///     .title("Incidents")
///     .data(serde_json::json!({ "open": 3 }))
///     .with(column("root", ["heading", "open"]))
///     .with(text("heading", "Right now"))
///     .with(Metric::new("open", "Open incidents", adk_ui::bind("/open")));
/// assert!(screen.validate(&Validator::new(adk_ui::catalogs::insight())).is_ok());
/// assert_eq!(screen.args()["surfaceId"], "incidents");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Screen {
    draft: SurfaceDraft,
}

impl Screen {
    /// An empty screen for the surface `surface_id`.
    pub fn new(surface_id: impl Into<String>) -> Self {
        Self {
            draft: SurfaceDraft {
                surface_id: surface_id.into(),
                kind: DrawnKind::Screen,
                title: None,
                components: Vec::new(),
                data_model: json!({}),
                actions: Vec::new(),
            },
        }
    }

    /// Its title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.draft.title = Some(title.into());
        self
    }

    /// Its data model.
    #[must_use]
    pub fn data(mut self, model: Value) -> Self {
        self.draft.data_model = model;
        self
    }

    /// The screen with `component` added.
    #[must_use]
    pub fn with(mut self, component: impl Into<Value>) -> Self {
        self.draft.components.push(component.into());
        self
    }

    /// An action a button asks for: one of the drawing agent's own tools, which the person
    /// decides before it runs.
    #[must_use]
    pub fn action(mut self, action: DrawnAction) -> Self {
        self.draft.actions.push(action);
        self
    }

    /// The screen as `genui` checks it.
    pub fn draft(&self) -> &SurfaceDraft {
        &self.draft
    }

    /// The screen as `genui` checks it.
    pub fn into_draft(self) -> SurfaceDraft {
        self.draft
    }

    /// The screen checked by `validator`, as `genui` checks it before anything is drawn.
    ///
    /// # Errors
    ///
    /// Returns why `genui` would refuse it.
    pub fn validate(&self, validator: &Validator) -> Result<ValidatedSurface, DrawRefusal> {
        validator.validate(&self.draft)
    }

    /// The arguments of the `render_screen` call that draws it.
    pub fn args(&self) -> Value {
        let mut args = json!({
            "surfaceId": self.draft.surface_id,
            "components": self.draft.components,
            "dataModel": self.draft.data_model,
            "actions": self.draft.actions,
        });
        if let Some(title) = &self.draft.title {
            args["title"] = json!(title);
        }
        args
    }
}

/// A `forms` `Text`.
pub fn text(id: impl Into<String>, text: impl Into<Value>) -> Value {
    json!({ "id": id.into(), "component": "Text", "text": text.into() })
}

/// A `forms` `Row` of `children`, by id.
pub fn row<I, S>(id: impl Into<String>, children: I) -> Value
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let children: Vec<String> = children.into_iter().map(Into::into).collect();
    json!({ "id": id.into(), "component": "Row", "children": children })
}

/// A `forms` `Column` of `children`, by id.
pub fn column<I, S>(id: impl Into<String>, children: I) -> Value
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let children: Vec<String> = children.into_iter().map(Into::into).collect();
    json!({ "id": id.into(), "component": "Column", "children": children })
}

/// A `forms` `Card` around `child`, by id.
pub fn card(id: impl Into<String>, child: impl Into<String>) -> Value {
    json!({ "id": id.into(), "component": "Card", "child": child.into() })
}
