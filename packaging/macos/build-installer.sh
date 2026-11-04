#!/bin/bash
# Run on the Mac architecture for which the installer is intended.
set -euo pipefail

if [[ "$(uname -s)" != Darwin ]]; then
    echo "This installer must be built on macOS (Xcode command line tools required)." >&2
    exit 1
fi
cd "$(dirname "$0")/../.."
version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)"
architecture="$(uname -m)"
case "$architecture" in
    arm64) target=aarch64-apple-darwin ;;
    x86_64) target=x86_64-apple-darwin ;;
    *) echo "Unsupported Mac architecture: $architecture" >&2; exit 1 ;;
esac
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-11.0}"

cargo build --locked --release --target "$target"
"target/$target/release/motion-music" --version
mkdir -p dist
staging="$(mktemp -d "$PWD/dist/macos-stage.XXXXXX")"
trap 'rm -rf "$staging"' EXIT
bash packaging/macos/bundle.sh "target/$target/release/motion-music" \
    "$staging/Motion Music.app" "$version"
cp LICENSE THIRD-PARTY-NOTICES.md "$staging/"
ln -s /Applications "$staging/Applications"
filename="Motion-Music-$version-macos-$architecture.dmg"
output="$PWD/dist/$filename"
# hdiutil refuses to overwrite an existing installer.
hdiutil create -volname "Motion Music" -srcfolder "$staging" -format UDZO "$output"
hdiutil verify "$output"
(cd dist && shasum -a 256 "$filename" > "$filename.sha256")
echo "Installer: $output"
