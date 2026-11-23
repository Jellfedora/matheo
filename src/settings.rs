//! Mathéo's settings, in `settings.json` in his folder (changed from his menu).

use serde::{Deserialize, Serialize};

use crate::i18n::Lang;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub(crate) struct Settings {
    /// His advice in a bubble every half hour.
    pub tips: bool,
    pub lang: Lang,
    /// Started with the computer.
    pub autostart: bool,
    /// His time reminders (the breaks, lunch, the end of the day), as set in his menu.
    pub hours: Hours,
    /// What he calls you (asked once, changed from his menu).
    pub name: Option<String>,
    /// His skin (chosen from his menu): the green plant, Paloma the cactus, Hélio the sunflower. He's
    /// still Mathéo: their names are only the skins'.
    pub skin: Skin,
    /// The version he last started as: another one, he tells what's new in it.
    pub seen: Option<String>,
    /// Never started yet (no file): he then starts with the computer.
    #[serde(skip)]
    pub first: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { tips: true, lang: Lang::from_system(), autostart: false, hours: Hours::default(), name: None, skin: Skin::default(), seen: None, first: false }
    }
}

/// His skin: each has its name (only the skin's: he's still Mathéo).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Skin {
    /// Mathéo, the green plant with his big leaves.
    #[default]
    Classic,
    /// Paloma, a cactus with two arms, a pink flower on top on a day well watered.
    Cactus,
    /// Hélio, a sunflower turning his head to the pointer.
    Sunflower,
}

impl Skin {
    pub const ALL: [Skin; 3] = [Skin::Classic, Skin::Cactus, Skin::Sunflower];

    /// The skin's name, in his menu.
    pub fn name(self) -> &'static str {
        match self {
            Skin::Classic => "Mathéo",
            Skin::Cactus => "Paloma",
            Skin::Sunflower => "Hélio",
        }
    }

    /// What he is, in his menu.
    pub fn label(self, lang: Lang) -> String {
        let what = match (self, lang) {
            (Skin::Classic, Lang::Fr) => "la plante verte",
            (Skin::Classic, Lang::En) => "the green plant",
            (Skin::Cactus, Lang::Fr) => "le cactus",
            (Skin::Cactus, Lang::En) => "the cactus",
            (Skin::Sunflower, Lang::Fr) => "le tournesol",
            (Skin::Sunflower, Lang::En) => "the sunflower",
        };
        format!("{}, {what}", self.name())
    }
}

/// His time reminders: on or off, and when (minutes from midnight): the morning break, lunch, the
/// afternoon break, the end of the day.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(default)]
pub(crate) struct Hours {
    pub on: bool,
    pub at: [u32; 4],
}

impl Hours {
    pub const DEFAULT: [u32; 4] = [10 * 60, 12 * 60, 16 * 60, 18 * 60];
    /// Set from his menu a quarter of an hour at a time.
    pub const STEP: i32 = 15;

    /// Reminder `k` moved by `minutes`, within the day.
    pub fn shift(&mut self, k: usize, minutes: i32) {
        if let Some(at) = self.at.get_mut(k) {
            *at = (*at as i32 + minutes).clamp(0, 24 * 60 - Self::STEP) as u32;
        }
    }
}

impl Default for Hours {
    fn default() -> Self {
        Self { on: true, at: Self::DEFAULT }
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
