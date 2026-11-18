#!/usr/bin/env bash
# Publishes a new version: ./scripts/release.sh 0.2.0 (after its section in CHANGELOG.md).
# Bumps Cargo.toml, commits, tags vX.Y.Z and pushes; GitHub Actions then builds macOS, Linux and
# Windows and publishes the archives on floor, where Mathéo updates from and his page offers them.
set -euo pipefail
cd "$(dirname "$0")/.."

version=${1:?usage: scripts/release.sh X.Y.Z}
[[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "version invalide : $version (attendu X.Y.Z)" >&2; exit 1; }
[[ -z $(git status --porcelain) ]] || { echo "des modifications ne sont pas commitées" >&2; exit 1; }
[[ $(git branch --show-current) == main ]] || { echo "à lancer depuis main" >&2; exit 1; }

sed -i.bak "1,/^version = \".*\"/s/^version = \".*\"/version = \"$version\"/" Cargo.toml
rm -f Cargo.toml.bak
cargo check --quiet # refreshes Cargo.lock
git commit -am "Release v$version" || true
git tag -a "v$version" -m "Mathéo $version"
git push origin main
# The tag on its own: pushed with the branch, it once didn't start release.yml.
git push origin "v$version"
echo "v$version poussée : suis la compilation dans l'onglet Actions du dépôt."
