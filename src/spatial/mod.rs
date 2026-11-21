//! The spatial components (catalog [`SPATIAL_ID`](crate::catalogs::SPATIAL_ID)): 3D charts
//! (`Bar3D`, `Scatter3D`, `SurfacePlot`, `Graph3D`, `Globe`), laid-out scenes (`Scene`) with
//! motion, panels beside objects and objects the person picks into a form, cards with depth
//! (`DepthCard`) and components arranged in 3D (`Stage`).
//!
//! Each one only shows. The web home, and any host drawing with `DrawnView`, draws them
//! natively: scenes with WebGL from `genui`'s checked `DrawnScene`, cards and layouts with CSS
//! 3D. The terminal shows their words. Colours come from the home's palette and state tokens,
//! never from the agent.

mod build;
mod components;

pub use build::{
    Bar3D, DepthCard, Globe, Graph3D, Motion, Object, Scatter3D, Scene, Stage, SurfacePlot,
};
pub use components::{
    MAX_BAR_ROWS, MAX_GLOBE_ARCS, MAX_GLOBE_POINTS, MAX_GRAPH_EDGES, MAX_GRAPH_NODES, MAX_LAYERS,
    MAX_LIGHTS, MAX_MOVING, MAX_OBJECTS, MAX_PANELS, MAX_SCATTER_POINTS, MAX_SERIES, MAX_STAGED,
    MAX_SURFACE_SIDE, components,
};
