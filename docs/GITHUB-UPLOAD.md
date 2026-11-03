# Motion Music GitHub upload

This repository contains the tested Motion Music v12 customisation of
[Spotifast](https://github.com/crmne/spotifast). The original MIT licence and
upstream credits are retained. Native product source files match the supplied
Motion Music v12 source archive.

## Downloads and version

The [motion-v12 release](https://github.com/MaybeMotion/Motion-Music/releases/tag/motion-v12)
provides the Windows portable ZIP, source archive and checksums. `motion-v12` is
the custom edition label. The underlying Cargo package and executable version
remain `0.14.1`; the automatic updater remains disabled. Updates are downloaded
manually from this repository.

The interface has a faster speed ramp with an early acceleration and a gentle
landing. Ten shuffled fixed-page reveals now take 360 ms, and theme changes keep
their soft circular reveal at 400 ms. Hover and press feedback, theme collection
fades, row movement, lyric highlights, artwork tint and toasts use the same ramp
with shorter durations. Native menus and smooth scrolling keep their own easing
with shorter timing. Repeated clicks preserve the reveal clock, and a new page
starts from the visible composite. Reduced motion still settles effects at once.

The approved transparent Vortex Bloom icon, all 212 themes, sixteen player-bar
visualisers, the real MilkDrop/projectM window and optional Discord Listening
activity are retained. Audio, visualiser timing, settings and authentication are
unchanged by this update. Discord sharing starts off and uses the included public
application ID without OAuth or a secret.

This update was validated on Windows x86_64 with normal, all-feature and
no-MilkDrop configurations. Test and lint results are recorded in the release
notes. Native light and dark comparisons at wide and narrow sizes use a matched
v11 baseline and isolated offline fixtures, including rapid-navigation and
circular theme-change sequences. The full release executable also passes a
native MilkDrop child diagnostic without opening its window. This update does
not claim Linux or macOS validation and does not change Spotify's native
audio-key behaviour. Third-party licences and matching application/library
source archives accompany the full binary.

## Inactive upstream publishing configuration

All eight inherited workflows are archived in
[`packaging/upstream-workflows`](../packaging/upstream-workflows/) with `.yml.txt`
extensions. Their contents are retained byte for byte for reference, and GitHub
Actions will not run them at these paths.

The upstream `docs/CNAME` file naming `spotifast.rocks` is omitted from this
upload. No GitHub Pages site or custom domain is configured by this source.
Other inherited documentation and `native-packages.yaml` retain upstream
Spotifast links and publishing destinations as reference material. Review and
adapt them before enabling any publishing or packaging automation.

## Local data

Portable profile folders, environment files, root-level credential/token files,
build output and upload work are ignored. No Spotify sign-in, real user profile,
audio capture or personal listening data is included in the source or Windows
ZIP. Public project assets and synthetic test fixtures remain part of the source.
