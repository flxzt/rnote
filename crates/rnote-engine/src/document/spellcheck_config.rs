// Imports
use crate::engine::spellcheck;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename = "spellcheck_config_language")]
pub enum SpellcheckConfigLanguage {
    #[default]
    #[serde(rename = "automatic")]
    Automatic,
    #[serde(rename = "language")]
    Language(String),
}

impl SpellcheckConfigLanguage {
    fn resolve(&self) -> Option<&String> {
        match self {
            Self::Automatic => *spellcheck::AUTOMATIC_LANGUAGE,
            Self::Language(language) => Some(language),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "spellcheck_config")]
pub struct SpellcheckConfig {
    #[serde(rename = "enabled")]
    pub enabled: bool,
    #[serde(rename = "language")]
    pub language: SpellcheckConfigLanguage,
}

impl Default for SpellcheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            language: Default::default(),
        }
    }
}

impl SpellcheckConfig {
    pub fn get_dictionary(&self, broker: &mut enchant::Broker) -> Option<enchant::Dict> {
        if self.enabled
            && let Some(language) = self.language.resolve()
        {
            return broker.request_dict(language).ok();
        }

        None
    }
}
