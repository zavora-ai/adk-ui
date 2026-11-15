pub mod generator;
pub mod spec;
mod surface;

pub use generator::{KitArtifacts, KitGenerator};
pub use spec::{
    BRAND_KIT_SCHEMA_VERSION, BrandKitManifest, KitAsset, KitAssetKind, KitBrand, KitColors,
    KitComponentRecipe, KitComponents, KitDensity, KitProvenance, KitRadius, KitSemanticColors,
    KitSemanticTokens, KitSemanticTypography, KitSource, KitSpec, KitStatus, KitTemplate,
    KitTypography, KitValidationError,
};
pub use surface::{ApprovedKit, KitNotApproved};
