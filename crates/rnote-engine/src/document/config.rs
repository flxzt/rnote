// Imports
use super::{Background, Format, Layout};
use crate::engine::SPELLCHECK_DEFAULT_LANGUAGE;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "spellcheck_config")]
pub struct SpellcheckConfig {
    #[serde(rename = "enabled")]
    pub enabled: bool,
    #[serde(rename = "language")]
    pub language: Option<String>,
}

impl Default for SpellcheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            language: SPELLCHECK_DEFAULT_LANGUAGE.clone(),
        }
    }
}

impl SpellcheckConfig {
    pub fn dictionary(&self, broker: &mut enchant::Broker) -> Option<enchant::Dict> {
        if self.enabled {
            if let Some(language) = &self.language {
                broker.request_dict(language).ok()
            } else {
                None
            }
        } else {
            None
        }
    }
}

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
