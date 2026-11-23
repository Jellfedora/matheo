#!/usr/bin/env python3
"""Builds src/i18n.rs and src/overlay.rs from Ronnie's Mathéo (its src/app/desktop.rs and src/i18n.rs).

Ronnie's repository next to this one (../bipbip), or its path in RONNIE_DIR. Mathéo's own parts (his
window, his menu, the update) are in overlay_head.rs, his season outfits in season.rs, his antics in antics.rs, his own words and hours in care.rs, next to this script: change them there, then run
`python3 scripts/sync/gen.py` and check with `cargo test` and `cargo clippy`.
"""
import re
from pathlib import Path

import os

HERE = Path(__file__).resolve().parent
OUT = HERE.parent.parent / "src"
RONNIE = Path(os.environ.get("RONNIE_DIR", HERE.parent.parent.parent / "bipbip"))

desk = (RONNIE / "src/app/desktop.rs").read_text().split("\n")
i18n = (RONNIE / "src/i18n.rs").read_text().split("\n")


def lines(a, b):
    """Lines a..b (1-based, inclusive)."""
    return "\n".join(desk[a - 1 : b])


def find(text, start=1):
    for i in range(start - 1, len(desk)):
        if desk[i].startswith(text):
            return i + 1
    raise SystemExit(f"not found: {text}")


# ---------- i18n
FIELDS = ["matheo_water", "matheo_moves", "matheo_shaking", "matheo_quiet", "matheo_ouch", "matheo_thanks",
          "matheo_thirsty", "matheo_bed", "matheo_weekend", "matheo_break", "matheo_hello", "matheo_hiding", "matheo_gone"]


def section(start_marker):
    i = next(k for k, l in enumerate(i18n) if l.startswith(start_marker))
    j = next(k for k in range(i + 1, len(i18n)) if i18n[k] == "};")
    return i18n[i + 1 : j]


def value(sec, name):
    for k, l in enumerate(sec):
        if l.startswith(f"    {name}: "):
            out = [l]
            m = k + 1
            while m < len(sec) and not re.match(r"^    [a-z_0-9]+: ", sec[m]):
                out.append(sec[m])
                m += 1
            return "\n".join(out)
    raise SystemExit(f"no {name}")


fr, en = section("pub static FR"), section("pub static EN")
MENU = {
    "menu_tips_on": ("Ses conseils", "His advice"),
    "menu_quiet": ("Me taire pendant 6 h", "Keep quiet for 6 hours"),
    "menu_speak": ("Reparler", "Speak again"),
    "menu_autostart": ("Lancer au démarrage", "Start with the computer"),
    "menu_lang": ("Language: English", "Langue : français"),
    "menu_page": ("Le site de Mathéo", "Mathéo's website"),
    "menu_version": ("Mathéo, version {v}", "Mathéo, version {v}"),
    "menu_quit": ("Quitter Mathéo", "Quit Mathéo"),
    "matheo_update": ("Une nouvelle version de moi est sortie ! Clique sur cette bulle pour l’installer, je me relance tout seul.",
                      "A new version of me is out! Click this bubble to install it, I’ll restart by myself."),
    "matheo_installing": ("Je me mets à jour… ne bouge pas.", "Updating myself… don’t move."),
    "matheo_updated": ("C’est installé, je reviens tout de suite !", "Installed, I’ll be right back!"),
    "matheo_update_failed": ("La mise à jour n’a pas marché. Je réessaierai plus tard.", "The update didn’t work. I’ll try again later."),
    "matheo_dock": ("Coucou ! Moi c’est Mathéo, ta nouvelle plante de bureau. Je veille sur toi, et toi tu m’arroses : marché conclu ? Pour que je me pose sur le Dock, autorise-moi dans Accessibilité (la fenêtre de macOS) ; sinon je reste juste au-dessus. Un clic droit sur moi pour mon menu !",
                    "Hi there! I’m Mathéo, your new desk plant. I look after you, you water me: deal? To stand on the Dock, allow me in Accessibility (macOS’ window); else I stay just above it. Right-click me for my menu!"),
    "menu_install": ("Installer la version {v}", "Install version {v}"),
    "menu_check": ("Rechercher une mise à jour", "Check for updates"),
    "menu_journal": ("Son journal (en cas de souci)", "His log (if something's wrong)"),
    "matheo_checking": ("Je regarde s’il y a une nouvelle version de moi…", "Looking for a new version of me…"),
    "matheo_latest": ("Je suis à jour, c’est ma dernière version.", "I’m up to date, this is my latest version."),
    "matheo_unreachable": ("Je n’arrive pas à savoir s’il y a une nouvelle version. Réessaie plus tard.", "I can’t find out whether there’s a new version. Try again later."),
    "menu_installing": ("Mise à jour en cours…", "Updating…"),
    "menu_dock": ("Me poser sur le Dock…", "Stand on the Dock…"),
    "breathe_in": ("Inspire par le nez… {n}", "Breathe in through your nose… {n}"),
    "breathe_out": ("Souffle tout doucement par la bouche… {n}", "Breathe out slowly through your mouth… {n}"),
    "breathe_count": ("Respiration {k} sur {of}", "Breath {k} of {of}"),
    "matheo_ask_name": ("Au fait…\nComment je peux t’appeler ?\nClique dans la case et écris ton prénom :",
                        "By the way…\nWhat should I call you?\nClick in the box and type your name:"),
    "matheo_psst": ("Psst, {name}… ", "Psst, {name}… "),
    "menu_name": ("Mon prénom…", "My name…"),
    "name_hint": ("Ton prénom", "Your name"),
    "name_ok": ("C’est moi !", "That’s me!"),
    "name_later": ("Passer", "Skip"),
    "menu_hours": ("Mes horaires…", "My hours…"),
    "menu_back": ("‹ Retour", "‹ Back"),
    "menu_hours_on": ("Ses rappels d’horaires", "His time reminders"),
    "menu_hour_morning": ("Pause du matin", "Morning break"),
    "menu_hour_lunch": ("Repas", "Lunch"),
    "menu_hour_afternoon": ("Pause de l’aprèm", "Afternoon break"),
    "menu_hour_home": ("Fin de journée", "End of the day"),
    "menu_hours_reset": ("Remettre par défaut", "Back to default"),
    "matheo_hi": ("Coucou ! Moi c’est Mathéo, ta nouvelle plante de bureau. Je veille sur toi : de l’eau, un peu de mouvement, des pauses. En échange, juste un petit coup d’arrosoir de temps en temps. Un clic droit sur moi pour mon menu !",
                  "Hi there! I’m Mathéo, your new desk plant. I look after you: water, a bit of moving, breaks. In return, just a little watering now and then. Right-click me for my menu!"),
}
V = {
    'matheo_pause': (['C’est l’heure de la pause, {name} ! Lève-toi, étire-toi, va prendre l’air : la pause, c’est important. Je garde l’écran, promis.', 'Pause ! Tes jambes ont envie de faire un tour, et ton cerveau d’un petit café. Vas-y, je ne touche à rien.', 'Hop hop hop, {name}, c’est la pause ! Même moi je fais la sieste entre deux arrosages. La pause, c’est sacré.', 'Ding ding ! C’est l’heure de souffler un peu. Cinq minutes loin de l’écran, et tu reviens tout neuf.'], ['Break time, {name}! Get up, stretch, get some fresh air: breaks matter. I’ll guard the screen, promise.', 'Break! Your legs want a little walk, and your brain a little coffee. Go on, I won’t touch a thing.', 'Hop hop hop, {name}, break time! Even I nap between waterings. Breaks are sacred.', 'Ding ding! Time to catch your breath. Five minutes away from the screen and you’ll be good as new.']),
    'matheo_lunch': (['C’est l’heure de manger ! Ton assiette t’attend (moi je me contente d’un verre d’eau, mais toi tu mérites mieux).', 'Miam, {name}, c’est l’heure du repas ! Et pas devant l’écran, hein : je surveille.', 'Ton ventre gargouille, je l’entends d’ici ! File manger, le travail ne s’envolera pas.', 'À table ! Un vrai repas, assis, sans écran. Moi je fais un peu de photosynthèse en t’attendant.'], ['Time to eat! Your plate is waiting (I’ll make do with a glass of water, but you deserve better).', 'Yum, {name}, lunchtime! And not in front of the screen, okay? I’m watching.', 'I can hear your tummy rumbling from here! Go eat, the work won’t fly away.', 'Lunch is served! A real meal, sitting down, no screen. I’ll do some photosynthesis while I wait.']),
    'matheo_home': (['C’est l’heure ! On ferme tout et on rentre à la maison. Tu as bien bossé, {name}, je suis fier de toi.', 'Fin de journée ! Range tes affaires, éteins tout, et va faire un truc qui te fait plaisir.', 'Et c’est fini pour aujourd’hui ! Le reste attendra demain, promis. Rentre bien, {name}.', 'Clap de fin ! Tu as fait de ton mieux, et c’est déjà beaucoup. Allez, ouste, à la maison !'], ['It’s time! Close everything and head home. You worked well, {name}, I’m proud of you.', 'End of the day! Pack up, switch off, and go do something you enjoy.', 'That’s it for today! The rest can wait until tomorrow, promise. Get home safe, {name}.', 'That’s a wrap! You did your best, and that’s already a lot. Off you go, home!']),
    'matheo_monday': (['Bon lundi, {name} ! On y va tout doux, une chose après l’autre. Moi je pousse à mon rythme, et ça marche.', 'Lundi matin… courage ! Un café, une petite liste, et on attaque la semaine tranquillement.', 'Coucou, nouvelle semaine ! J’ai un bon pressentiment pour celle-là, {name}.'], ['Happy Monday, {name}! Easy does it, one thing at a time. I grow at my own pace, and it works.', 'Monday morning… hang in there! A coffee, a little to-do list, and we start the week gently.', 'Hello, new week! I have a good feeling about this one, {name}.']),
    'matheo_lamp': (['Il fait déjà nuit… Allume une lampe ! Tes yeux te diront merci (et moi j’aime bien la lumière aussi).', 'Brr, il fait sombre ! Une petite lampe, et c’est tout de suite plus doux pour tes yeux.', 'Le soleil est parti se coucher. Allume la lumière, on n’est pas des chauves-souris !'], ['It’s already dark… Switch a lamp on! Your eyes will thank you (and I like light too).', 'Brr, it’s getting dark! A little lamp and it’s much gentler on your eyes.', 'The sun has gone to bed. Switch the light on, we’re not bats!']),
    'matheo_weekend_work': (['Psst, {name}… c’est le week-end. Tu es sûr de vouloir travailler ? Va jouer dehors, je garde la maison !', 'Euh… on est le week-end, tu sais ? Même le travail a besoin de se reposer.', 'Week-end détecté ! Si c’est pour le plaisir, d’accord. Sinon, file te reposer, {name}.'], ['Psst, {name}… it’s the weekend. Are you sure you want to work? Go play outside, I’ll mind the house!', 'Um… it’s the weekend, you know? Even work needs a rest.', 'Weekend detected! If it’s for fun, fine. Otherwise, go and rest, {name}.']),
    'matheo_eyes': (['Petite pause pour tes yeux : regarde au loin pendant 20 secondes. Par la fenêtre, c’est encore mieux !', 'Tes yeux fixent l’écran depuis un moment… Regarde un truc loin, très loin, pendant 20 secondes.', 'Jeu du jour : trouve l’objet le plus éloigné de toi et regarde-le 20 secondes. Tes yeux adorent ce jeu.', 'Cligne des yeux, regarde au loin, et souris. Voilà, tes yeux sont tout reposés (et moi je suis content).'], ['A little break for your eyes: look far away for 20 seconds. Out of the window is even better!', 'Your eyes have been staring at the screen for a while… Look at something far, far away for 20 seconds.', 'Game of the day: find the furthest thing from you and look at it for 20 seconds. Your eyes love this game.', 'Blink, look far away, and smile. There, your eyes are all rested (and I’m happy).']),
    'matheo_posture': (['Hop, on se redresse ! Le dos droit, les épaules en arrière… comme moi. Bon, moi j’ai un pot, c’est plus facile.', 'Tu ne serais pas un peu en train de te ratatiner, {name} ? Redresse-toi, comme une belle plante !', 'Check posture : pieds au sol, dos contre le dossier, épaules détendues. Parfait, on dirait moi.', 'Étire-toi vers le plafond, comme pour attraper le soleil. Ahhh, ça fait du bien, non ?'], ['Up we sit! Back straight, shoulders back… like me. Well, I have a pot, it’s easier.', 'Are you shrivelling up a little, {name}? Sit up straight, like a proud plant!', 'Posture check: feet on the floor, back against the chair, shoulders relaxed. Perfect, you look like me.', 'Stretch up to the ceiling, as if to catch the sun. Ahhh, feels good, doesn’t it?']),
    'matheo_breathe': (['On respire ensemble ? Quatre grandes respirations, suis-moi : je gonfle, tu inspires ; je dégonfle, tu souffles.', 'Petite pause respiration, {name} ? Regarde-moi : quand je gonfle, tu inspires ; quand je dégonfle, tu souffles.', 'Allez, quatre grandes bouffées d’air ensemble. Je montre l’exemple, tu me suis !'], ['Shall we breathe together? Four deep breaths, follow me: I swell, you breathe in; I shrink, you breathe out.', 'A little breathing break, {name}? Watch me: when I swell, breathe in; when I shrink, breathe out.', 'Come on, four big breaths of air together. I’ll lead, you follow!']),
    'breathe_done': (['Et voilà ! Tu te sens plus léger, non ? Moi j’ai l’impression d’avoir pris une feuille.', 'Bravo ! Quatre respirations, zéro stress. On refait ça quand tu veux.', 'Ahhh… tout calme. Tu vois, c’est facile d’être une plante.'], ['There you go! Feeling lighter? I feel like I grew a leaf.', 'Well done! Four breaths, zero stress. Let’s do it again whenever you like.', 'Ahhh… all calm. See, being a plant is easy.']),
    'matheo_hot': (['Il fait chaud ! Un grand verre d’eau pour toi, un petit pour moi ?', 'Ouf, quelle chaleur ! Bois un coup, {name}, avant de fondre comme une glace.', 'Mode canicule activé : de l’eau, de l’eau, et encore de l’eau !'], ['It’s hot! A big glass of water for you, a little one for me?', 'Phew, what a heat! Have a drink, {name}, before you melt like an ice cream.', 'Heatwave mode on: water, water, and more water!']),
    'matheo_back': (['Te revoilà, {name} ! Tu m’as manqué.', 'Ah, tu es là ! Je t’ai gardé ta place bien au chaud.', 'Coucou ! Je commençais à parler tout seul…', 'Re-bonjour, {name} ! On reprend où on en était ?'], ['There you are, {name}! I missed you.', 'Ah, you’re here! I kept your seat nice and warm.', 'Hi! I was starting to talk to myself…', 'Hello again, {name}! Shall we pick up where we left off?']),
    'matheo_named': (['Enchanté, {name} ! Je retiens, promis. Moi c’est toujours Mathéo, hein.', '{name} ! Quel joli prénom. Je vais le dire souvent, tu verras.'], ['Nice to meet you, {name}! I’ll remember, promise. I’m still Mathéo, by the way.', '{name}! What a lovely name. I’ll say it often, you’ll see.']),
    'matheo_again': (['Coucou, c’est encore moi ! Je me suis fait une petite beauté (nouvelle version), et j’ai appris plein de trucs.', 'Tadaa ! Une nouvelle version de moi, toute fraîche. Et j’ai des surprises pour toi.'], ['Hi, it’s me again! I got a little makeover (new version) and learned lots of new tricks.', 'Ta-da! A brand new version of me. And I have surprises for you.']),
    'more_ouch': (['Aïe !', 'Ouille, ma tête !', 'Aïeuh ! Doucement…', 'Bonk ! Je vois des étoiles…'], ['Ouch!', 'Ow, my head!', 'Ouchie! Gently…', 'Bonk! I see stars…']),
    'more_shaking': (['Waaaah ! Arrête !', 'Je vais avoir mal au cœur !', 'Pas si fort, j’ai la tête qui tourne !'], ['Waaaah! Stop it!', 'I’m going to be sick!', 'Not so hard, my head’s spinning!']),
    'more_thanks': (['Merci ! Santé ! Ça fait du bien.', 'Ahhh, délicieux ! Merci, {name}.', 'Glou glou glou… Je revis !', 'Merci ! Je sens que je vais pousser d’un coup.'], ['Thanks! Cheers! That feels good.', 'Ahhh, delicious! Thank you, {name}.', 'Glug glug glug… I’m alive again!', 'Thanks! I can feel myself growing.']),
    'more_thirsty': (['J’ai soif… et toi aussi, sûrement. Un grand verre d’eau, puis un coup d’arrosoir ?', 'Mes feuilles font la tête… Un peu d’eau pour moi, et un verre pour toi ?', 'Je suis tout sec, {name}… On boit un coup ensemble ?'], ["I'm thirsty… and so are you, surely. A big glass of water, then the watering can?", 'My leaves are sulking… A little water for me, and a glass for you?', 'I’m all dry, {name}… Shall we have a drink together?']),
}

decl = "\n".join(
    [f"    pub {f}: &'static [&'static str]," if f in ("matheo_water", "matheo_moves", "matheo_bed", "matheo_weekend", "matheo_break")
     else f"    pub {f}: &'static str," for f in FIELDS]
    + [f"    pub {k}: &'static str," for k in MENU]
    + [f"    pub {k}: &'static [&'static str]," for k in V]
)


def body(sec, idx):
    lists = [f"    {k}: &[\n" + "".join(f'        "{x}",\n' for x in v[idx]) + "    ]," for k, v in V.items()]
    return "\n".join([value(sec, f) for f in FIELDS] + [f'    {k}: "{v[idx]}",' for k, v in MENU.items()] + lists)


(OUT / "i18n.rs").write_text(f'''//! Mathéo's words, in French and in English (taken from Ronnie's).

use serde::{{Deserialize, Serialize}};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum Lang {{
    #[default]
    Fr,
    En,
}}

impl Lang {{
    /// Every language (the tests go through them).
    #[cfg(test)]
    pub const ALL: [Lang; 2] = [Lang::Fr, Lang::En];

    pub fn strings(self) -> &'static Strings {{
        match self {{
            Lang::Fr => &FR,
            Lang::En => &EN,
        }}
    }}

    /// The system's language: English when it says so, French otherwise.
    pub fn from_system() -> Self {{
        let lang = ["LC_ALL", "LC_MESSAGES", "LANG"].iter().find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty())).unwrap_or_default();
        if lang.starts_with("en") {{ Lang::En }} else {{ Lang::Fr }}
    }}

    pub fn other(self) -> Self {{
        match self {{
            Lang::Fr => Lang::En,
            Lang::En => Lang::Fr,
        }}
    }}
}}

pub struct Strings {{
{decl}
}}

pub static FR: Strings = Strings {{
{body(fr, 0)}
}};

pub static EN: Strings = Strings {{
{body(en, 1)}
}};
''')

# ---------- overlay
other_from = find("/// A window over another screen than the pointer's.")
plant_to = find("pub(crate) fn run() {") - 1
plant = lines(other_from, plant_to)
plant = plant.replace("crate::config::config_dir()", "crate::config_dir()")
plant = "\n".join(l for l in plant.split("\n") if "`ronnie desktop-fx`" not in l)
plant = plant.replace('std::env::var("RONNIE_MATHEO_CLOCK")', 'std::env::var("MATHEO_CLOCK")')
plant = plant.replace("/// Debug builds: `RONNIE_MATHEO_CLOCK=late|friday`", "/// Debug builds: `MATHEO_CLOCK=late|friday`")
plant = plant.replace("/// Debug builds: `RONNIE_MATHEO_GALLERY=1`", "/// Debug builds: `MATHEO_GALLERY=1`")
for gone in ['        ("Salut Gérard", Look { wave: 1.0, ..base }, None),\n', '        ("La horde, 10 h", Look { hide: 1.0, ..base }, None),\n']:
    assert gone in plant, gone
    plant = plant.replace(gone, "")
plant = plant.replace("pub(super) fn paint_matheo(", "#[allow(dead_code)]\nfn paint_matheo(")
# His eyes following the pointer near him (`gaze`, set by the window: see overlay_head.rs).
for a, b in [
    ("    wilt: f32,\n}\n", "    wilt: f32,\n    /// Where his eyes look, the pointer near him (his units, 1 at most): towards it.\n    gaze: Vec2,\n}\n"),
    ("            flower: self.growth.2 >= DAYS_A_FLOWER,\n", "            flower: self.growth.2 >= DAYS_A_FLOWER,\n            gaze: Vec2::ZERO,\n"),
    ("        } else {\n            painter.circle_filled(at(x, y), 2.0 * u, ink);\n",
     "        } else {\n            // Following the pointer near him.\n            painter.circle_filled(at(x + look.gaze.x * 1.4, y + look.gaze.y * 1.1), 2.0 * u, ink);\n"),
]:
    assert plant.count(a) == 1, a
    plant = plant.replace(a, b)
# A word when a newer version is out, and hello at the very first start.
plant = plant.replace("""    /// The horde gone.
    Gone,
}""", """    /// The horde gone.
    Gone,
    /// A newer version of him published.
    Update,
    /// Hello, the very first time.
    Hi,
    /// Hello, the very first time on macOS, asking for Accessibility (to stand on the Dock).
    Dock,
    Installing,
    Updated,
    UpdateFailed,
    /// Looking for a newer version, asked from his menu.
    Checking,
    /// Asked: he's the latest version.
    Latest,
    /// Asked: floor couldn't be reached.
    Unreachable,
    /// What he says of his own (see `care.rs`).
    Own(Own),
}""")
plant = plant.replace("""            Said::Hiding => Duration::from_millis(1500),""", """            Said::Hiding => Duration::from_millis(1500),
            Said::Update | Said::Hi | Said::Dock => Duration::from_secs(30),
            Said::Installing => Duration::from_secs(120),
            Said::Updated => Duration::from_secs(5),
            Said::UpdateFailed | Said::Latest | Said::Unreachable => Duration::from_secs(8),
            Said::Checking => Duration::from_secs(40),
            Said::Own(o) => o.lasts(),""")
plant = plant.replace("""            Said::Gone => t.matheo_gone,""", """            Said::Gone => t.matheo_gone,
            Said::Update => t.matheo_update,
            Said::Hi => t.matheo_hi,
            Said::Dock => t.matheo_dock,
            Said::Installing => t.matheo_installing,
            Said::Updated => t.matheo_updated,
            Said::UpdateFailed => t.matheo_update_failed,
            Said::Checking => t.matheo_checking,
            Said::Latest => t.matheo_latest,
            Said::Unreachable => t.matheo_unreachable,
            Said::Own(o) => o.text(t, 0),""")
assert "Said::Update" in plant and "t.matheo_hi" in plant

ui_from = find("        // Mathéo: grabbed with the mouse")
gerard = find("            // Gérard passing near him", ui_from)
pass_line = find("            let pass = !(over || plant.grab.is_some());", ui_from)
block = lines(ui_from, gerard - 1)
block = block.replace("        if self.shown.matheo {\n", "        {\n", 1)
block = block.replace("self.shown.matheo_tips", "self.settings.tips")
# A right click on him (or his bubble): his menu.
anchor = "            let over = mouse.at.is_some_and(|p| body.contains(p)) || on_bubble || on_can;\n"
assert anchor in block
block = block.replace(anchor, anchor + """            if right_click && (mouse.at.is_some_and(|p| body.contains(p)) || on_bubble) {
                open_menu = mouse.at;
            }
""")
assert block.count("click") > 3

os_from = find('#[cfg(target_os = "macos")]', pass_line)
tests_at = find("#[cfg(test)]", os_from)
oses = lines(os_from, tests_at - 1)
# The right button too.
reps = [
    ("Mouse { at: Some(Pos2::new(p.x as f32, p.y as f32)), pressed: CGEventSourceButtonState(0, 0) }",
     "Mouse { at: Some(Pos2::new(p.x as f32, p.y as f32)), pressed: CGEventSourceButtonState(0, 0), right: CGEventSourceButtonState(0, 1) }"),
    ("let (ok, pressed) = unsafe { (GetCursorPos(&mut p) != 0, (GetAsyncKeyState(VK_LBUTTON as i32) as u16 & 0x8000) != 0) };",
     "let (ok, pressed, right) = unsafe { (GetCursorPos(&mut p) != 0, (GetAsyncKeyState(VK_LBUTTON as i32) as u16 & 0x8000) != 0, (GetAsyncKeyState(VK_RBUTTON as i32) as u16 & 0x8000) != 0) };"),
    ("Mouse { at: Some(Pos2::new(p.x as f32 / scale, p.y as f32 / scale)), pressed }",
     "Mouse { at: Some(Pos2::new(p.x as f32 / scale, p.y as f32 / scale)), pressed, right }"),
    ("use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};",
     "use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON};"),
    ("Some(p) => Mouse { at: Some(Pos2::new(f32::from(p.root_x), f32::from(p.root_y))), pressed: p.mask.contains(KeyButMask::BUTTON1) },",
     "Some(p) => Mouse { at: Some(Pos2::new(f32::from(p.root_x), f32::from(p.root_y))), pressed: p.mask.contains(KeyButMask::BUTTON1), right: p.mask.contains(KeyButMask::BUTTON3) },"),
    ('"Ronnie\'s rewards over the screen"', '"Mathéo over the screen"'),
    # Windows: the frame left to DWM (see the comment put in its place).
    ("use windows_sys::Win32::Graphics::Dwm::{DWMNCRP_DISABLED, DWMWA_BORDER_COLOR, DWMWA_NCRENDERING_POLICY, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND, DwmSetWindowAttribute};",
     "use windows_sys::Win32::Graphics::Dwm::{DWMWA_BORDER_COLOR, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND, DwmSetWindowAttribute};"),
    ("            set(DWMWA_NCRENDERING_POLICY, DWMNCRP_DISABLED as u32);\n",
     "            // Not DWMNCRP_DISABLED: winit's windows keep WS_CAPTION | WS_BORDER (their frame only\n"
     "            // trimmed through WM_NCCALCSIZE), and without DWM drawing the frame, DefWindowProc painted\n"
     "            // a classic one over the whole window at each WM_NCACTIVATE (him taking or losing the\n"
     "            // focus): a screen-sized frame around nothing.\n"),
]
for a, b in reps:
    assert a in oses, a
    oses = oses.replace(a, b)

tests_from = tests_at
tests = lines(tests_from, len(desk))
keep = ["matheo_lands_at_the_foot_of_his_screen", "matheo_lands_on_the_screen_under_him_when_screens_are_stacked", "mathéo_says_water_and_moving_in_turns", "the_can_stands_on_the_free_side_and_comes_back_down", "mathéo_wilts_with_thirst"]
fns = re.split(r"\n(?=    #\[test\]\n)", tests)
head, rest = fns[0], fns[1:]
kept = [f for f in rest if any(f"fn {k}(" in f for k in keep)]
assert len(kept) == len(keep), [k for k in keep if not any(f"fn {k}(" in f for f in kept)]
tests = "\n".join([head] + kept).rstrip()
if not tests.endswith("}"):
    tests += "\n}"
tests = re.sub(r"\n    use [^\n]*Pointer[^\n]*", "", tests)
tests = tests.replace("crate::i18n::Lang::ALL", "crate::i18n::Lang::ALL")

overlay = (HERE / "overlay_head.rs").read_text()
overlay = overlay.replace("//@SEASON@", (HERE / "season.rs").read_text().rstrip())
overlay = overlay.replace("//@ANTICS@", (HERE / "antics.rs").read_text().rstrip())
overlay = overlay.replace("//@CARE@", (HERE / "care.rs").read_text().rstrip())
overlay = overlay.replace("//@PLANT@", plant).replace("//@BLOCK@", block).replace("//@OS@", oses).replace("//@TESTS@", tests)
(OUT / "overlay.rs").write_text(overlay)
print("ok", len(overlay.split(chr(10))), "lines")
