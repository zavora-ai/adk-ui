use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitSpec {
    pub name: String,
    pub version: String,
    pub brand: KitBrand,
    pub colors: KitColors,
    pub typography: KitTypography,
    #[serde(default)]
    pub density: KitDensity,
    #[serde(default)]
    pub radius: KitRadius,
    #[serde(default)]
    pub components: KitComponents,
    #[serde(default)]
    pub templates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitBrand {
    pub vibe: String,
    #[serde(default)]
    pub industry: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitColors {
    pub primary: String,
    #[serde(default)]
    pub accent: Option<String>,
    #[serde(default)]
    pub surface: Option<String>,
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitTypography {
    pub family: String,
    #[serde(default)]
    pub scale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum KitDensity {
    Compact,
    #[default]
    Comfortable,
    Spacious,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum KitRadius {
    None,
    Sm,
    #[default]
    Md,
    Lg,
    Xl,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct KitComponents {
    #[serde(default)]
    pub button: Option<KitComponentButton>,
    #[serde(default)]
    pub card: Option<KitComponentCard>,
    #[serde(default)]
    pub input: Option<KitComponentInput>,
    #[serde(default)]
    pub table: Option<KitComponentTable>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitComponentButton {
    #[serde(default)]
    pub variants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitComponentCard {
    #[serde(default)]
    pub elevation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitComponentInput {
    #[serde(default)]
    pub style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KitComponentTable {
    #[serde(default)]
    pub striped: Option<bool>,
}
