//! Mathéo's settings, in `settings.json` in his folder (changed from his menu).

use serde::{Deserialize, Serialize};

use crate::i18n::Lang;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(default)]
pub(crate) struct Settings {
    /// His advice in a bubble every half hour.
    pub tips: bool,
    pub lang: Lang,
    /// Started with the computer.
    pub autostart: bool,
    /// Never started yet (no file): he then starts with the computer.
    #[serde(skip)]
    pub first: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { tips: true, lang: Lang::from_system(), autostart: false, first: false }
    }
}

fn file() -> Option<std::path::PathBuf> {
    crate::config_dir().map(|d| d.join("settings.json"))
}

impl Settings {
    pub(crate) fn load() -> Self {
        match file().and_then(|f| std::fs::read_to_string(f).ok()) {
            Some(text) => serde_json::from_str(&text).unwrap_or_default(),
            None => Self { first: true, ..Self::default() },
        }
    }

    pub(crate) fn save(&self) {
        if let Some(f) = file() {
            let _ = std::fs::write(f, serde_json::to_string_pretty(self).unwrap_or_default());
        }
    }
}
