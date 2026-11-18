//! The studio components (catalog [`STUDIO_ID`](crate::catalogs::STUDIO_ID)): the insight
//! components, and pages, regions and primitive 3D scenes for hosts that embed agent-drawn
//! applications. The web home shows an app as tabs and a scene as the list of what is in it;
//! a custom host draws them richly with `@zavora-ai/adk-ui-react`.

mod components;

pub use components::{
    App, MAX_PAGES, MAX_SCENE_OBJECTS, Page, Region, Scene3D, SceneObject, Shape, components,
};
