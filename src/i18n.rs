//! Mathéo's words, in French and in English (taken from Ronnie's).

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[default]
    Fr,
    En,
}

impl Lang {
    /// Every language (the tests go through them).
    #[cfg(test)]
    pub const ALL: [Lang; 2] = [Lang::Fr, Lang::En];

    pub fn strings(self) -> &'static Strings {
        match self {
            Lang::Fr => &FR,
            Lang::En => &EN,
        }
    }

    /// The system's language: English when it says so, French otherwise.
    pub fn from_system() -> Self {
        let lang = ["LC_ALL", "LC_MESSAGES", "LANG"].iter().find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty())).unwrap_or_default();
        if lang.starts_with("en") { Lang::En } else { Lang::Fr }
    }

    pub fn other(self) -> Self {
        match self {
            Lang::Fr => Lang::En,
            Lang::En => Lang::Fr,
        }
    }
}

pub struct Strings {
    pub matheo_water: &'static [&'static str],
    pub matheo_moves: &'static [&'static str],
    pub matheo_shaking: &'static str,
    pub matheo_quiet: &'static str,
    pub matheo_ouch: &'static str,
    pub matheo_thanks: &'static str,
    pub matheo_thirsty: &'static str,
    pub matheo_bed: &'static [&'static str],
    pub matheo_weekend: &'static [&'static str],
    pub matheo_break: &'static [&'static str],
    pub matheo_hello: &'static str,
    pub matheo_hiding: &'static str,
    pub matheo_gone: &'static str,
    pub menu_tips_on: &'static str,
    pub menu_quiet: &'static str,
    pub menu_speak: &'static str,
    pub menu_autostart: &'static str,
    pub menu_lang: &'static str,
    pub menu_page: &'static str,
    pub menu_version: &'static str,
    pub menu_quit: &'static str,
    pub matheo_update: &'static str,
    pub matheo_installing: &'static str,
    pub matheo_updated: &'static str,
    pub matheo_update_failed: &'static str,
    pub matheo_dock: &'static str,
    pub menu_install: &'static str,
    pub menu_check: &'static str,
    pub matheo_checking: &'static str,
    pub matheo_latest: &'static str,
    pub matheo_unreachable: &'static str,
    pub menu_installing: &'static str,
    pub menu_dock: &'static str,
    pub matheo_hi: &'static str,
}

pub static FR: Strings = Strings {
    matheo_water: &[
        "Une gorgée d’eau ? Moi j’ai été arrosé ce matin, à ton tour.",
        "Pense à boire un verre d’eau. Les plantes et les humains, même combat.",
        "Ta gourde t’attend. Un peu d’eau, et ça repart.",
        "Un verre d’eau maintenant : ta tête te dira merci cet après-midi.",
        "Ça fait un moment que tu n’as pas bu. Va remplir ton verre, je surveille.",
        "Le café, ça ne compte pas. Un vrai verre d’eau, s’il te plaît.",
        "Et les autres plantes du bureau, quelqu’un a pensé à les arroser ? Elles ont soif, elles aussi.",
    ],
    matheo_moves: &[
        "Lève-toi et fais quelques pas, je garde ton poste.",
        "Étire ton dos : les bras au ciel, et on pousse vers le plafond.",
        "Fais rouler tes épaules : cinq fois en avant, cinq fois en arrière.",
        "Regarde au loin pendant vingt secondes, tes yeux en ont besoin.",
        "Penche doucement la tête à droite, puis à gauche. Ta nuque respire.",
        "Redresse-toi : les pieds à plat, les épaules relâchées. Voilà.",
        "Va voir un collègue à pied plutôt que sur le tchat.",
        "Trois grandes respirations, lentement. Moi, je fais de la photosynthèse.",
        "Étire tes poignets : la main tendue, tire doucement les doigts vers toi.",
        "Debout ! Hausse-toi sur la pointe des pieds dix fois.",
    ],
    matheo_shaking: "Waaaah ! Arrête !",
    matheo_quiet: "D’accord, d’accord… Je me tais pendant 6 h. Ça tourne…",
    matheo_ouch: "Aïe !",
    matheo_thanks: "Merci ! Santé ! Ça fait du bien.",
    matheo_thirsty: "J’ai soif… et toi aussi, sûrement. Un grand verre d’eau, puis un coup d’arrosoir ?",
    matheo_bed: &[
        "Il est tard… Va te coucher, le code sera encore là demain.",
        "Bâille avec moi : aaaah. Allez, au lit.",
        "Même les plantes dorment la nuit. Éteins tout, à demain.",
        "Le bug de ce soir, tu le trouveras en deux minutes demain matin. Au lit !",
        "Je n’ai plus de lumière pour ma photosynthèse. Toi non plus, tu devrais dormir.",
    ],
    matheo_weekend: &[
        "C’est vendredi ! Bientôt le week-end, tiens bon.",
        "Lunettes de soleil sorties : le week-end approche.",
        "Plus que quelques heures… Je sens déjà le barbecue d’ici.",
        "Vendredi après-midi : on ne met rien en prod, d’accord ?",
        "Le week-end est en vue. Range un peu ton bureau, lundi te dira merci.",
        "Moi ce week-end, je fais de la photosynthèse au soleil. Et toi ?",
    ],
    matheo_break: &[
        "Ça fait 2 h que tu n’as pas fait de pause. Lève-toi, vraiment, je te regarde.",
        "Deux heures d’affilée ! Même moi je me dégourdirais les racines. Pause !",
        "Stop ! Cinq minutes loin de l’écran, un verre d’eau, et tu reviens.",
        "Ton dos m’a écrit : il demande une pause. Tout de suite.",
    ],
    matheo_hello: "Salut Gérard !",
    matheo_hiding: "Chut… Je ne suis pas là.",
    matheo_gone: "Ils sont partis ?",
    menu_tips_on: "Ses conseils",
    menu_quiet: "Me taire pendant 6 h",
    menu_speak: "Reparler",
    menu_autostart: "Lancer au démarrage",
    menu_lang: "Language: English",
    menu_page: "Le site de Mathéo",
    menu_version: "Mathéo, version {v}",
    menu_quit: "Quitter Mathéo",
    matheo_update: "Une nouvelle version de moi est sortie ! Clique sur cette bulle pour l’installer, je me relance tout seul.",
    matheo_installing: "Je me mets à jour… ne bouge pas.",
    matheo_updated: "C’est installé, je reviens tout de suite !",
    matheo_update_failed: "La mise à jour n’a pas marché. Je réessaierai plus tard.",
    matheo_dock: "Salut ! Moi c’est Mathéo. Pour que je me pose sur le Dock, autorise-moi dans Accessibilité (la fenêtre de macOS). Sinon je reste juste au-dessus. Un clic droit sur moi pour mon menu.",
    menu_install: "Installer la version {v}",
    menu_check: "Rechercher une mise à jour",
    matheo_checking: "Je regarde s’il y a une nouvelle version de moi…",
    matheo_latest: "Je suis à jour, c’est ma dernière version.",
    matheo_unreachable: "Je n’arrive pas à savoir s’il y a une nouvelle version. Réessaie plus tard.",
    menu_installing: "Mise à jour en cours…",
    menu_dock: "Me poser sur le Dock…",
    matheo_hi: "Salut ! Moi c’est Mathéo, je veille sur toi. Un clic droit sur moi pour mon menu.",
};

pub static EN: Strings = Strings {
    matheo_water: &[
        "A sip of water? I got watered this morning, your turn.",
        "Remember to drink a glass of water. Plants and humans, same fight.",
        "Your bottle is waiting. A little water and off you go again.",
        "A glass of water now: your head will thank you this afternoon.",
        "It's been a while since you drank. Go fill your glass, I'm watching.",
        "Coffee doesn't count. A real glass of water, please.",
        "And the other plants in the office, has anyone thought of watering them? They're thirsty too.",
    ],
    matheo_moves: &[
        "Get up and take a few steps, I'll mind your desk.",
        "Stretch your back: arms up, and push towards the ceiling.",
        "Roll your shoulders: five times forward, five times back.",
        "Look far away for twenty seconds, your eyes need it.",
        "Gently tilt your head to the right, then to the left. Your neck breathes.",
        "Sit up: feet flat, shoulders relaxed. There.",
        "Walk over to a colleague rather than using the chat.",
        "Three deep breaths, slowly. Me, I'm doing photosynthesis.",
        "Stretch your wrists: hand out, gently pull your fingers towards you.",
        "Stand up! Rise on your toes ten times.",
    ],
    matheo_shaking: "Waaaah! Stop it!",
    matheo_quiet: "All right, all right… I'll keep quiet for 6 hours. Everything's spinning…",
    matheo_ouch: "Ouch!",
    matheo_thanks: "Thanks! Cheers! That feels good.",
    matheo_thirsty: "I'm thirsty… and so are you, surely. A big glass of water, then the watering can?",
    matheo_bed: &[
        "It's late… Go to bed, the code will still be there tomorrow.",
        "Yawn with me: aaaah. Come on, bedtime.",
        "Even plants sleep at night. Switch everything off, see you tomorrow.",
        "Tonight's bug, you'll find it in two minutes tomorrow morning. Bedtime!",
        "No more light for my photosynthesis. You should sleep too.",
    ],
    matheo_weekend: &[
        "It's Friday! The weekend is near, hang in there.",
        "Sunglasses on: the weekend is coming.",
        "Only a few hours left… I can already smell the barbecue.",
        "Friday afternoon: nothing goes to production, all right?",
        "The weekend is in sight. Tidy your desk a little, Monday will thank you.",
        "This weekend I'm doing photosynthesis in the sun. And you?",
    ],
    matheo_break: &[
        "Two hours without a break. Get up, really, I'm watching you.",
        "Two hours on end! Even I would stretch my roots. Break time!",
        "Stop! Five minutes away from the screen, a glass of water, then come back.",
        "Your back wrote to me: it wants a break. Right now.",
    ],
    matheo_hello: "Hi Gérard!",
    matheo_hiding: "Shh… I'm not here.",
    matheo_gone: "Are they gone?",
    menu_tips_on: "His advice",
    menu_quiet: "Keep quiet for 6 hours",
    menu_speak: "Speak again",
    menu_autostart: "Start with the computer",
    menu_lang: "Langue : français",
    menu_page: "Mathéo's website",
    menu_version: "Mathéo, version {v}",
    menu_quit: "Quit Mathéo",
    matheo_update: "A new version of me is out! Click this bubble to install it, I’ll restart by myself.",
    matheo_installing: "Updating myself… don’t move.",
    matheo_updated: "Installed, I’ll be right back!",
    matheo_update_failed: "The update didn’t work. I’ll try again later.",
    matheo_dock: "Hi! I’m Mathéo. To stand on the Dock, allow me in Accessibility (macOS’ window). Else I stay just above it. Right-click me for my menu.",
    menu_install: "Install version {v}",
    menu_check: "Check for updates",
    matheo_checking: "Looking for a new version of me…",
    matheo_latest: "I’m up to date, this is my latest version.",
    matheo_unreachable: "I can’t find out whether there’s a new version. Try again later.",
    menu_installing: "Updating…",
    menu_dock: "Stand on the Dock…",
    matheo_hi: "Hi! I’m Mathéo, I look after you. Right-click me for my menu.",
};
