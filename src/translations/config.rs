use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TranslationConfig {
    pub version: String,
    pub metadata: Metadata,
    pub segments: BTreeMap<String, SegmentConfig>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Metadata {
    pub last_updated: String,
    pub description: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SegmentConfig {
    pub label: String,
    #[serde(default)]
    pub use_qualifier: bool,
    #[serde(default)]
    pub qualifiers: BTreeMap<String, SubSegmentConfig>,
    #[serde(default)]
    pub elements: BTreeMap<String, ElementConfig>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubSegmentConfig {
    pub label: String,
    #[serde(default)]
    pub elements: BTreeMap<String, ElementConfig>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum ElementConfig {
    Simple(String),
    Composite {
        label: String,
        components: BTreeMap<String, String>,
    },
}
