# Mathéo

La plante verte qui veille sur toi en bas de l’écran : boire, bouger, dormir. Une appli à part, tirée de
Ronnie (où Mathéo est une récompense de Riff Hero), pour macOS, Windows et Linux (X11).

## En bref

| | |
|---|---|
| **Code** | ce dépôt, GitHub `Jellfedora/matheo` (public), branche `main` |
| **Téléchargement** | https://185.245.143.130/floor/matheo — la page de Mathéo sur floor, à côté de celle de Ronnie |
| **Versions hébergées** | sur floor (VM ronnie-bot), API `https://185.245.143.130/floor/v1/matheo` : `GET /release`, `GET /files/:version/:name`, publication `PUT /releases/...` avec le jeton |
| **Archives** | `matheo-universal-apple-darwin.tar.gz` (Mathéo.app, Apple Silicon), `matheo-x86_64-pc-windows-msvc.zip` (matheo.exe), `matheo-x86_64-unknown-linux-gnu.tar.gz` (matheo) |
| **Signature** | chaque archive signée Ed25519 avec la clé de release de Ronnie (secret `RONNIE_UPDATE_KEY`, aussi dans `~/.ronnie-signing/update-ed25519.pem`) ; Mathéo vérifie empreinte et signature avant d’installer |
| **Signature macOS** | certificat auto-signé de Ronnie (« Ronnie Code Signing », secrets `MACOS_SIGN_P12` / `MACOS_SIGN_P12_PASSWORD`), sinon ad hoc |
| **Réglages sur le poste** | `settings.json`, `matheo.json` (sa place), `matheo-thirst.json`, `matheo-growth.json`, `matheo-quiet.json` dans le dossier de config (`~/Library/Application Support/Matheo`, `%APPDATA%\Matheo\config`, `~/.config/matheo`) |

## Ce qu’il fait

- Une bulle toutes les 30 min (souris active) : eau et mouvement en alternance ; un arrosoir avec l’eau.
- Arrosé : content, il grandit (une fleur du jour dès le 3ᵉ arrosage de la journée, partie le lendemain ; une feuille tous les 5 bons jours, jusqu’à 3). Oublié : il dépérit.
- Vendredi après 14 h : lunettes noires. Après 23 h : fatigué, « va te coucher ». 2 h sans pause : il insiste.
- En saison : bonnet et neige à Noël (1er–25 déc.), chapeau de fête et confettis au Nouvel an (31 déc.–2 janv.), nœud et cœurs à la Saint-Valentin, oreilles de lapin et œuf la semaine de Pâques, chapeau de sorcière et chauve-souris à Halloween (25–31 oct.). Propre à l’appli (`scripts/sync/season.rs`), pas dans Ronnie.
- Caché sur un écran qu’une autre appli remplit (un film, un jeu), sauf Ronnie.
- Attrapé, lancé (physique, rebonds), secoué (6 allers-retours vifs) : muet 6 h.
- Lancé : yeux ronds en vol, écrasé et rebondissant à l’atterrissage, poussière quand il tombe fort ou glisse au sol ; lâché de haut (plus de 260 pt au-dessus du sol), un parachute une fois sur trois. Propre à l’appli (`scripts/sync/antics.rs`), pas dans Ronnie.
- En semaine : la pause à 10 h et 16 h, manger à 12 h, rentrer à 18 h (dans la demi-heure, une fois chacune, ses conseils activés et pas muet), réglables dans son menu. Entre ses conseils, toutes les 40 min : yeux, posture, respiration (et l’eau l’été). Lundi matin, la lampe les soirs d’hiver (17 h, novembre à janvier), le week-end, ton retour après 1 h d’absence. Ton prénom, demandé une fois (bulle, puis une petite fenêtre), glissé dans certaines phrases. Propre à l’appli (`scripts/sync/care.rs`).
- macOS : posé sur le Dock grâce à l’Accessibilité (demandée au premier lancement, et dans son menu tant qu’elle manque) ; sans elle, juste au-dessus.
- Trois plantes au choix (son menu, « Mon apparence… ») : Mathéo la plante verte, Paloma le cactus, Hélio le tournesol (son cœur tourné vers le pointeur) ; même pot, même visage, et il reste Mathéo (ce ne sont que des noms de skins). Propre à l’appli (`scripts/sync/skins.rs`).
- Clic droit sur lui : son menu, à la première personne (sa version, conseils, se taire, lancer au démarrage, langue, rechercher ou installer une mise à jour, son site, quitter).
- Une seule instance à la fois (port local 47391). Lancé au démarrage dès la première fois (LaunchAgent, clé Run, ~/.config/autostart).
- Pas de Gérard ni de horde ici : ce sont des compagnons de Ronnie.

## Mises à jour

Vérifiées au lancement, puis toutes les heures et à la demande (son menu) sur floor. Mathéo l’annonce dans une bulle ; un clic dessus (ou
« Installer la version… » dans son menu) télécharge l’archive, vérifie SHA-256 et signature, remplace
l’appli et la relance.

## Développer

- `cargo run` : le lance, à côté du Mathéo installé (un build de debug a son port, 47392, et ses réglages, `Matheo-dev`).
- `MATHEO_GALLERY=1 cargo run` : tous ses états et ses tenues côte à côte ; `MATHEO_CLOCK=late|friday|10|12|16|17|18|saturday` force l’heure, `MATHEO_CHUTE=1` le parachute à chaque chute, `MATHEO_CARE=1` ses soins toutes les 20 s ; son menu a une page « Démo… » qui joue chaque animation et chaque phrase, `MATHEO_SEASON=christmas|newyear|valentine|easter|halloween` la tenue (builds de debug seulement).
- `cargo test`, `cargo clippy`.
- Le test sur un bureau Windows et Linux : `gh workflow run smoke.yml` (`scripts/smoke/`) ; il démarre Mathéo, le survole, le lance, ouvre son menu, et garde des captures et son journal dans les artefacts du run (un seul écran, pas de vraie carte graphique).
- Son journal : `matheo.log` dans le dossier de config (aussi dans son menu, « Mon journal »), refait à chaque lancement.
- L’icône : `scripts/make-icon.py` (Pillow) écrit `assets/icon.png`, `Matheo.icns`, `matheo.ico`.
- Le code de Mathéo vient de Ronnie (`src/app/desktop.rs`, `src/i18n.rs`) : `python3 scripts/sync/gen.py` regénère `src/overlay.rs` et `src/i18n.rs` depuis `../bipbip` (ou `RONNIE_DIR`). Ce qui est propre à l’appli (fenêtre, menu, mise à jour) se change dans `scripts/sync/overlay_head.rs`, ses tenues dans `scripts/sync/season.rs`, ses autres plantes dans `scripts/sync/skins.rs`, ses animations dans `scripts/sync/antics.rs`, ses phrases à lui et ses horaires dans `scripts/sync/care.rs`, pas dans `src/overlay.rs`.

## Publier

1. Une section `## [X.Y.Z] - date` dans `CHANGELOG.md`.
2. `scripts/release.sh X.Y.Z` : version, commit, tag, push. La CI (`.github/workflows/release.yml`) compile les trois
   systèmes et publie sur floor (secrets du dépôt : `RONNIE_UPDATE_KEY`, `FLOOR_PUBLISH_TOKEN`,
   `MACOS_SIGN_P12`, `MACOS_SIGN_P12_PASSWORD`).
3. Dépôt public : les minutes GitHub Actions sont gratuites (en privé, le quota avait été épuisé
   le 2026-11-10 pour Ronnie).
