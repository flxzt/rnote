// Imports
use super::{Background, Format, Layout, SpellcheckConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename = "document_config")]
pub struct DocumentConfig {
    #[serde(rename = "format")]
    pub format: Format,
    #[serde(rename = "background")]
    pub background: Background,
    #[serde(rename = "layout", alias = "expand_mode")]
    pub layout: Layout,
    #[serde(rename = "spellcheck")]
    pub spellcheck: SpellcheckConfig,
}
