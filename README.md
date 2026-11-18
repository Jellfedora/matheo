# Mathéo

La plante verte qui veille sur toi en bas de l’écran : boire, bouger, dormir. Une appli à part, tirée de
Ronnie (où Mathéo est une récompense de Riff Hero), pour macOS, Windows et Linux (X11).

## En bref

| | |
|---|---|
| **Code** | ce dépôt, GitHub `Jellfedora/matheo` (**privé**), branche `main` |
| **Téléchargement** | https://185.245.143.130/floor/matheo — la page de Mathéo sur floor, à côté de celle de Ronnie |
| **Versions hébergées** | sur floor (VM ronnie-bot), API `https://185.245.143.130/floor/v1/matheo` : `GET /release`, `GET /files/:version/:name`, publication `PUT /releases/...` avec le jeton |
| **Archives** | `matheo-universal-apple-darwin.tar.gz` (Mathéo.app, Apple Silicon), `matheo-x86_64-pc-windows-msvc.zip` (matheo.exe), `matheo-x86_64-unknown-linux-gnu.tar.gz` (matheo) |
| **Signature** | chaque archive signée Ed25519 avec la clé de release de Ronnie (secret `RONNIE_UPDATE_KEY`, aussi dans `~/.ronnie-signing/update-ed25519.pem`) ; Mathéo vérifie empreinte et signature avant d’installer |
| **Signature macOS** | certificat auto-signé de Ronnie (« Ronnie Code Signing », secrets `MACOS_SIGN_P12` / `MACOS_SIGN_P12_PASSWORD`), sinon ad hoc |
| **Réglages sur le poste** | `settings.json`, `matheo.json` (sa place), `matheo-thirst.json`, `matheo-growth.json`, `matheo-quiet.json` dans le dossier de config (`~/Library/Application Support/Matheo`, `%APPDATA%\Matheo\config`, `~/.config/matheo`) |

## Ce qu’il fait

- Une bulle toutes les 30 min (souris active) : eau et mouvement en alternance ; un arrosoir avec l’eau.
- Arrosé : content, il grandit (une feuille tous les 5 bons jours, jusqu’à 3, une fleur à 20). Oublié : il dépérit.
- Vendredi après 14 h : lunettes noires. Après 23 h : fatigué, « va te coucher ». 2 h sans pause : il insiste.
- Attrapé, lancé (physique, rebonds), secoué (6 allers-retours vifs) : muet 6 h.
- macOS : posé sur le Dock grâce à l’Accessibilité (demandée au premier lancement, et dans son menu tant qu’elle manque) ; sans elle, juste au-dessus.
- Clic droit sur lui : son menu (conseils, se taire, lancer au démarrage, langue, sa page, installer la mise à jour, quitter).
- Une seule instance à la fois (port local 47391). Lancé au démarrage dès la première fois (LaunchAgent, clé Run, ~/.config/autostart).
- Pas de Gérard ni de horde ici : ce sont des compagnons de Ronnie.

## Mises à jour

Vérifiées au lancement puis toutes les 6 h sur floor. Mathéo l’annonce dans une bulle ; un clic dessus (ou
« Installer la version… » dans son menu) télécharge l’archive, vérifie SHA-256 et signature, remplace
l’appli et la relance.

## Développer

- `cargo run` : le lance (attention, à côté du Mathéo de Ronnie s’il tourne).
- `MATHEO_GALLERY=1 cargo run` : tous ses états côte à côte ; `MATHEO_CLOCK=late|friday` force l’heure (builds de debug seulement).
- `cargo test`, `cargo clippy`.
- L’icône : `scripts/make-icon.py` (Pillow) écrit `assets/icon.png`, `Matheo.icns`, `matheo.ico`.
- Le code de Mathéo vient de Ronnie (`src/app/desktop.rs`) : un changement de ce côté se reporte ici à la main.

## Publier

1. Une section `## [X.Y.Z] - date` dans `CHANGELOG.md`.
2. `scripts/release.sh X.Y.Z` : version, commit, tag, push. La CI (`.github/workflows/release.yml`) compile les trois
   systèmes et publie sur floor (secrets du dépôt : `RONNIE_UPDATE_KEY`, `FLOOR_PUBLISH_TOKEN`,
   `MACOS_SIGN_P12`, `MACOS_SIGN_P12_PASSWORD`).
3. Dépôt privé : les minutes GitHub Actions comptent (macOS ×10, Windows ×2). Le quota avait été épuisé
   le 2026-11-10 pour Ronnie.
