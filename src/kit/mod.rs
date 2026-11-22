//! Brand kits: a host's approved design tokens, applied to its own pages around `DrawnView`.
//! The model never picks a kit and a drawing never carries one. [`KitGenerator`] drafts a kit
//! from a [`KitSpec`]; a person reviews and approves it, and [`ApprovedKit`] applies it only
//! once it is approved and valid.

mod generator;
mod spec;
mod surface;

pub use generator::{KitArtifacts, KitGenerator};
pub use spec::{
    BRAND_KIT_SCHEMA_VERSION, BrandKitManifest, KitAsset, KitAssetKind, KitBrand, KitColors,
    KitComponentButton, KitComponentCard, KitComponentInput, KitComponentRecipe, KitComponentTable,
    KitComponents, KitDensity, KitProvenance, KitRadius, KitSemanticColors, KitSemanticTokens,
    KitSemanticTypography, KitSource, KitSpec, KitStatus, KitTemplate, KitTypography,
    KitValidationError,
};
pub use surface::{ApprovedKit, KitNotApproved};
