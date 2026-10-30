# Motion Music GitHub upload

This repository contains the tested Motion Music v7 customisation of
[Spotifast](https://github.com/crmne/spotifast). The original MIT licence and
upstream credits are retained. Native product source files are unchanged from
the supplied Motion Music v7 source archive.

## Downloads and version

The [motion-v7 release](https://github.com/MaybeMotion/Motion-Music/releases/tag/motion-v7)
provides the Windows portable ZIP, source archive and checksums. `motion-v7` is
the custom edition label. The underlying Cargo package and executable version
remain `0.14.1`; the automatic updater remains disabled. Updates are downloaded
manually from this repository.

The supplied Windows build includes all sixteen player-bar visualisers and omits
optional MilkDrop. It was validated on Windows x86_64 with 1,434 unit tests
passing, one existing credential-store test ignored, and strict Clippy checks
for the normal and demo configurations. This upload does not claim Linux, macOS
or all-features validation, and does not fix Spotify's native audio-key refusal.

## Inactive upstream publishing configuration

All eight inherited workflows were moved from `.github/workflows` to
[`packaging/upstream-workflows`](../packaging/upstream-workflows/) with `.yml.txt`
extensions. Their contents are retained byte for byte for reference, and GitHub
Actions will not run them at these paths.

The upstream `docs/CNAME` file naming `spotifast.rocks` was omitted from this
upload. No GitHub Pages site or custom domain is configured by this source.
Other inherited documentation and `native-packages.yaml` retain upstream
Spotifast links and publishing destinations as reference material. Review and
adapt them before enabling any publishing or packaging automation.

## Local data

Portable profile folders, environment files, root-level credential/token files,
build output and upload work are ignored. No Spotify sign-in, real user profile,
audio capture or personal listening data is included in the source or Windows
ZIP. Public project assets and synthetic test fixtures remain part of the source.
