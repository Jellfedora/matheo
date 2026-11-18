#!/usr/bin/env bash
# Builds the release archive for the current OS into dist/ (the names src/update.rs and floor's page
# expect): Mathéo.app on macOS, matheo on Linux, matheo.exe on Windows.
set -euo pipefail
cd "$(dirname "$0")/.."

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
rm -rf dist
mkdir -p dist/stage

case "$(uname -s)" in
Darwin)
    # Apple Silicon.
    rustup target add aarch64-apple-darwin
    cargo build --release --locked --target aarch64-apple-darwin
    app="dist/stage/Mathéo.app"
    mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
    cp target/aarch64-apple-darwin/release/matheo "$app/Contents/MacOS/matheo"
    cp assets/Matheo.icns "$app/Contents/Resources/"
    sed "s/{{VERSION}}/$version/g" packaging/macos/Info.plist >"$app/Contents/Info.plist"
    # Ronnie's self-signed certificate when SIGN_IDENTITY names it (the Accessibility permission then
    # survives updates); else ad hoc.
    codesign --force --deep --sign "${SIGN_IDENTITY:--}" "$app"
    tar -C dist/stage -czf dist/matheo-universal-apple-darwin.tar.gz "Mathéo.app"
    ;;
Linux)
    cargo build --release --locked
    cp target/release/matheo dist/stage/
    tar -C dist/stage -czf dist/matheo-x86_64-unknown-linux-gnu.tar.gz matheo
    ;;
MINGW* | MSYS* | CYGWIN*)
    cargo build --release --locked
    cp target/release/matheo.exe dist/stage/
    (cd dist/stage && 7z a -tzip ../matheo-x86_64-pc-windows-msvc.zip matheo.exe >/dev/null)
    ;;
*)
    echo "OS non géré : $(uname -s)" >&2
    exit 1
    ;;
esac
rm -rf dist/stage
ls -l dist
