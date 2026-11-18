#!/usr/bin/env python3
"""Builds src/i18n.rs and src/overlay.rs from Ronnie's Mathéo (its src/app/desktop.rs and src/i18n.rs).

Ronnie's repository next to this one (../bipbip), or its path in RONNIE_DIR. Mathéo's own parts (his
window, his menu, the update) are in overlay_head.rs, next to this script: change them there, then run
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
    "matheo_dock": ("Salut ! Moi c’est Mathéo. Pour que je me pose sur le Dock, autorise-moi dans Accessibilité (la fenêtre de macOS). Sinon je reste juste au-dessus. Un clic droit sur moi pour mon menu.",
                    "Hi! I’m Mathéo. To stand on the Dock, allow me in Accessibility (macOS’ window). Else I stay just above it. Right-click me for my menu."),
    "menu_install": ("Installer la version {v}", "Install version {v}"),
    "menu_installing": ("Mise à jour en cours…", "Updating…"),
    "menu_dock": ("Me poser sur le Dock…", "Stand on the Dock…"),
    "matheo_hi": ("Salut ! Moi c’est Mathéo, je veille sur toi. Un clic droit sur moi pour mon menu.",
                  "Hi! I’m Mathéo, I look after you. Right-click me for my menu."),
}
decl = "\n".join(
    [f"    pub {f}: &'static [&'static str]," if f in ("matheo_water", "matheo_moves", "matheo_bed", "matheo_weekend", "matheo_break")
     else f"    pub {f}: &'static str," for f in FIELDS]
    + [f"    pub {k}: &'static str," for k in MENU]
)


def body(sec, idx):
    return "\n".join([value(sec, f) for f in FIELDS] + [f'    {k}: "{v[idx]}",' for k, v in MENU.items()])


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
}""")
plant = plant.replace("""            Said::Hiding => Duration::from_millis(1500),""", """            Said::Hiding => Duration::from_millis(1500),
            Said::Update | Said::Hi | Said::Dock => Duration::from_secs(30),
            Said::Installing => Duration::from_secs(120),
            Said::Updated => Duration::from_secs(5),
            Said::UpdateFailed => Duration::from_secs(8),""")
plant = plant.replace("""            Said::Gone => t.matheo_gone,""", """            Said::Gone => t.matheo_gone,
            Said::Update => t.matheo_update,
            Said::Hi => t.matheo_hi,
            Said::Dock => t.matheo_dock,
            Said::Installing => t.matheo_installing,
            Said::Updated => t.matheo_updated,
            Said::UpdateFailed => t.matheo_update_failed,""")
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
]
for a, b in reps:
    assert a in oses, a
    oses = oses.replace(a, b)

tests_from = tests_at
tests = lines(tests_from, len(desk))
keep = ["matheo_lands_at_the_foot_of_his_screen", "mathéo_says_water_and_moving_in_turns", "the_can_stands_on_the_free_side_and_comes_back_down", "mathéo_wilts_with_thirst"]
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
overlay = overlay.replace("//@PLANT@", plant).replace("//@BLOCK@", block).replace("//@OS@", oses).replace("//@TESTS@", tests)
(OUT / "overlay.rs").write_text(overlay)
print("ok", len(overlay.split(chr(10))), "lines")
