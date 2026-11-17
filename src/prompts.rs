//! What to tell a model that draws in an adk-ui catalog, beside `genui`'s own tool
//! descriptions.

use adk_surface::genui::{Catalog, RESERVED_LABELS, RESERVED_PHRASES};

/// Instructions for an agent that draws in `catalog` with `genui`'s render tools: which
/// components only show, how to bind data, the limits, and what a drawing never contains.
///
/// ```rust
/// let instructions = adk_ui::prompts::instructions(adk_ui::catalogs::insight());
/// assert!(instructions.contains("render_screen"));
/// assert!(instructions.contains("Allow once"));
/// assert!(!instructions.contains("pickInto"));
/// assert!(adk_ui::prompts::instructions(adk_ui::catalogs::spatial()).contains("pickInto"));
/// ```
pub fn instructions(catalog: &Catalog) -> String {
    let added: Vec<&str> = catalog.extensions().collect();
    let reserved: Vec<&str> = RESERVED_PHRASES
        .iter()
        .chain(RESERVED_LABELS.iter())
        .copied()
        .collect();
    format!(
        "You can show the person charts and figures in their thread with render_screen, in the \
         {label} catalog: the forms components and {added}. These components only show; only a \
         Button asks for an action, and pressing it never runs a tool: it asks the person first, \
         on the home's own card.\n\
         - Put the numbers in dataModel and bind them, as {{\"path\": \"/sales\"}}; never paste \
         data into labels.\n\
         - A chart draws at most 500 rows and 8 series: aggregate or downsample first, and say \
         so in its title.\n\
         - When numbers change, call patch_surface with the new data instead of drawing again.\n\
         - Label things with plain words. A drawing never shows a decision's words ({reserved}), \
         links, images or password fields, and is refused if it does: say it in your reply or \
         draw it another way.\n\
         - Every component needs a unique id, and one has the id root.{spatial}",
        label = catalog.label(),
        added = added.join(", "),
        reserved = reserved.join(", "),
        spatial = if catalog.has_component("Scene") {
            SPATIAL
        } else {
            ""
        },
    )
}

/// What a model drawing in the spatial catalog needs to know about 3D.
const SPATIAL: &str = "\n\
     3D, drawn natively by the home in its own colours:\n\
     - Use 3D only when depth carries information: a value over two categories (Bar3D), three \
     measures (Scatter3D), a value over a grid (SurfacePlot), a network (Graph3D), places and \
     routes (Globe), a physical layout (Scene). Otherwise use Chart.\n\
     - Limits: Bar3D 50 rows by 8 series; Scatter3D 2000 points; SurfacePlot 64 by 64; Graph3D \
     200 nodes and 400 edges; Globe 500 places and 200 routes; Scene 128 objects, 3 lights, 4 \
     panels, 32 moving. Aggregate or sample first and say so in the title.\n\
     - Colours are names, never codes: chart1 to chart8, ink, neutral, muted; a state (ok, \
     warning, critical, info) needs a label too.\n\
     - Bind data and change it with patch_surface; keep object ids so things move smoothly. \
     Give motion only where it means something.\n\
     - To let the person pick by clicking, show a ChoicePicker, set the Scene's pickInto to its \
     path, and give each pickable object a label and a value from its options. A Button still \
     sends it, and the person decides.\n\
     - Panels beside objects, and a Stage's wall or stage, hold only Text, Metric, KeyValue or \
     charts. DepthCard and Stage present components you drew; never hide what the person must \
     read.";

/// Instructions for an agent that also has `show_3d_app` (feature `apps`): when to write
/// three.js and when to draw instead.
///
/// ```rust
/// assert!(adk_ui::prompts::sandboxed_3d_apps().contains("show_3d_app"));
/// ```
pub fn sandboxed_3d_apps() -> String {
    "You can also show a 3D scene you write in three.js with show_3d_app. It is framed in the \
     thread as an app that can't approve anything and reaches no sites.\n\
     - Draw with render_screen first: charts, figures, forms and anything the catalog declares. \
     Use show_3d_app only for 3D the catalog can't declare, such as motion, particles or \
     procedural shapes.\n\
     - Keep the code small and readable, use theme.palette for colours, and label what matters \
     with label(text, position).\n\
     - If it is refused, read why, change the code and try again; never work around the check."
        .to_string()
}
