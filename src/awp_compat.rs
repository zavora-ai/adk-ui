//! AWP compatibility re-exports.
//!
//! When the `awp` feature is enabled, this module re-exports the core types
//! from the `awp-types` crate for convenient access.

#[cfg(feature = "awp")]
pub use awp_types::{
    AwpResponse, AwpVersion, CURRENT_VERSION, CapabilityEntry, CapabilityManifest,
};
