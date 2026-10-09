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
    fn resolve(&self) -> Option<&str> {
        match self {
            Self::Automatic => *spellcheck::AUTOMATIC_LANGUAGE,
            Self::Language(language) => Some(language.as_str()),
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
    pub fn resolved_language(&self) -> Option<&str> {
        if self.enabled {
            self.language.resolve()
        } else {
            None
        }
    }
}
