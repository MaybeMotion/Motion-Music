#!/bin/bash
# Add the Mac licence notices to completed build images, then verify them.
set -euo pipefail
[[ "$(uname -s)" == Darwin ]] || { echo "macOS is required" >&2; exit 1; }
input="$(cd "$1" && pwd)"
here="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$2"
output="$(cd "$2" && pwd)"
notices="$3"
test -s "$notices"
work="$(mktemp -d "$output/staging.XXXXXX")"
mounted=""
cleanup() {
    if [[ -n "$mounted" ]]; then hdiutil detach "$mounted" -quiet || true; fi
    rm -rf "$work"
}
trap cleanup EXIT

for architecture in arm64 x86_64; do
    filename="Motion-Music-0.14.1-macos-$architecture.dmg"
    (cd "$input" && shasum -a 256 -c "$filename.sha256")
    stage="$work/$architecture"
    mkdir -p "$stage" "$work/mount"
    hdiutil attach -readonly -nobrowse -mountpoint "$work/mount" "$input/$filename" >/dev/null
    mounted="$work/mount"
    ditto "$mounted/Motion Music.app" "$stage/Motion Music.app"
    cp "$mounted/LICENSE" "$mounted/THIRD-PARTY-NOTICES.md" "$stage/"
    cp "$notices" "$stage/THIRD-PARTY-NOTICES-macos.txt"
    cp "$here/../../docs/MAC-INSTALL.md" "$stage/INSTALL.md"
    ln -s /Applications "$stage/Applications"
    codesign --verify --strict "$stage/Motion Music.app"
    lipo -verify_arch "$architecture" "$stage/Motion Music.app/Contents/MacOS/motion-music"
    python3 - "$stage/Motion Music.app/Contents/MacOS/motion-music" <<'PY'
import subprocess, sys
libraries = subprocess.check_output(['otool', '-L', sys.argv[1]], text=True)
unexpected = [line.strip().split(' (', 1)[0] for line in libraries.splitlines()[1:]
              if not line.strip().startswith(('/System/Library/', '/usr/lib/'))]
if unexpected:
    raise SystemExit('Non-system libraries require bundling: ' + ', '.join(unexpected))
print('Executable links only Apple system libraries')
PY
    hdiutil detach "$mounted" -quiet
    mounted=""
    hdiutil create -volname "Motion Music" -srcfolder "$stage" -format UDZO "$output/$filename"
    hdiutil verify "$output/$filename"
    (cd "$output" && shasum -a 256 "$filename" > "$filename.sha256")
done
cp "$notices" "$output/Motion-Music-v12-macos-THIRD-PARTY-NOTICES.txt"
(cd "$output" && shasum -a 256 *.dmg *NOTICES.txt > SHA256-v12-macos.txt)
