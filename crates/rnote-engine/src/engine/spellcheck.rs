// Imports
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::fmt::Debug;
use tracing::{debug, warn};

thread_local! {
    static BROKER: RefCell<enchant::Broker> = RefCell::new(enchant::Broker::new());
}

pub static AVAILABLE_LANGUAGES: Lazy<Vec<String>> = Lazy::new(|| {
    BROKER.with_borrow_mut(|broker| {
        broker
            .list_dicts()
            .iter()
            .map(|dict| dict.lang.to_owned())
            .collect()
    })
});

pub static AUTOMATIC_LANGUAGE: Lazy<Option<&str>> = Lazy::new(|| {
    // try each system language
    for system_language in glib::language_names() {
        // first pass: try exact match (e.g. "en_US.UTF-8" starts with "en_US")
        for available_language in AVAILABLE_LANGUAGES.iter() {
            if system_language.starts_with(available_language) {
                debug!(
                    "found exact spellcheck language match: {:?} (system: {:?})",
                    available_language, system_language
                );
                return Some(available_language.as_str());
            }
        }

        // second pass: try language-only match (e.g. "en_GB" starts with "en" derived from "en_US.UTF-8")
        if let Some((system_language_code, _)) = system_language.split_once('_') {
            for available_language in AVAILABLE_LANGUAGES.iter() {
                if available_language.starts_with(system_language_code) {
                    debug!(
                        "found language-only spellcheck match: {:?} (system: {:?})",
                        available_language, system_language
                    );
                    return Some(available_language.as_str());
                }
            }
        }
    }

    // fallback: use the first available language
    if let Some(language) = AVAILABLE_LANGUAGES.first() {
        debug!("using fallback spellcheck language: {:?}", language);
        Some(language.as_str())
    } else {
        None
    }
});

#[derive(Default)]
pub struct Spellcheck {
    pub dict: Option<enchant::Dict>,
}

impl Debug for Spellcheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Spellcheck")
            .field(
                "dict",
                &self
                    .dict
                    .as_ref()
                    .map(|dict| format!("Some({})", dict.get_lang()))
                    .unwrap_or(String::from("None")),
            )
            .finish()
    }
}

impl Spellcheck {
    pub fn set_language(&mut self, language: Option<&str>) {
        if self.dict.as_ref().map(|dict| dict.get_lang()) == language {
            return;
        }

        self.dict = None;
        let Some(language) = language else {
            return;
        };

        match BROKER.with_borrow_mut(|broker| broker.request_dict(language)) {
            Ok(dict) => self.dict = Some(dict),
            Err(err) => warn!("requesting spellcheck dictionary `{language}` failed, Err: {err}"),
        }
    }
}
