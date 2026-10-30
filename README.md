<p align="center"><img src="packaging/icons/spotifast.svg" alt="Motion Music" width="88" height="88"></p>

<h1 align="center">Motion Music</h1>

<p align="center"><strong>Your music, in full colour.</strong><br>A native Spotify client with 168 curated themes and sixteen audio-reactive player-bar visualisers.</p>

Motion Music is a personal customisation of [Spotifast](https://github.com/crmne/spotifast), built with Rust, egui, fastframe, and librespot. Original copyright, licence, and upstream credits are retained. Upstream documentation and release links describe Spotifast; official downloads do not include these customisations.

## Download Motion Music v7

**[Download the Windows portable ZIP](https://github.com/MaybeMotion/Motion-Music/releases/download/motion-v7/Motion-Music-v7-Windows.zip)** or open the [Motion Music v7 release](https://github.com/MaybeMotion/Motion-Music/releases/tag/motion-v7) for the source archive and checksums. Extract the ZIP, keep `spotifast-portable.txt` beside `Motion Music v7.exe`, and fully quit an older running copy before opening it.

See the [v7 setup and visualiser guide](docs/MOTION-MUSIC-V7.md). Downloads are installed manually; the automatic updater remains disabled for this custom build. **motion-v7** identifies this customised portable edition. The underlying Cargo package and executable version remain **0.14.1**, matching the tested build.

The inherited Spotifast publishing workflows are archived and inactive. See [GitHub upload notes](docs/GITHUB-UPLOAD.md) for the source and publishing details.

## Make it yours

Open **Settings > Appearance > Motion themes**. The gallery has clearly labelled colour tabs with swatches: **Red, Orange, Yellow, Green, Teal, Cyan, Blue, Purple, Pink, Brown, Monochrome and Gradients**. Every colour collection has both dark and light choices. Click a preview to apply it. Settings search understands colour names and aliases such as **navy**, **aqua**, **grey**, **black** and **white**, as well as combinations such as **light blue** and **blue gradient**. All 168 palettes are built in and available without downloads. They also appear in the main Theme menu and matching Light mode theme or Dark mode theme picker.

| Collection | Themes |
| --- | --- |
| Red (12) | Crimson, Cherry Bomb, Ruby, Scarlet, Blood Moon, Burgundy, Cherry Blossom, Vermilion, Red Velvet, Carmine, Merlot, Poppy Red |
| Orange (10) | Ember, Inferno, Hot Lava, Peach Fuzz, Coral Flare, Tangerine, Apricot, Persimmon, Mandarin, Mango Milk |
| Yellow (8) | Liquid Gold, Lemon Sorbet, Amber Room, Citron, Marigold, Buttercream, Sunflower, Daffodil |
| Green (12) | Emerald, Toxic Mint, Jade Temple, Malachite, Mint Tea, Forest Canopy, Pine Needle, Olive Grove, Pistachio, Lime Wire, Matcha Latte, Racing Green |
| Teal (6) | Northern Lights, Seafoam, Tidal Teal, Peacock, Viridian, Aqua Marine |
| Cyan (7) | Arctic, Glacial Blue, Petrol Blue, Pacific Night, Lagoon Mist, Cyan Circuit, Ice Mint |
| Blue (8) | Deep Ocean, Electric Blue, Glacier, Sapphire, Cobalt Ink, Powder Blue, Royal Navy, Cornflower |
| Purple (20) | Amethyst, Purple Rain, Ultraviolet, Violet Hour, Nebula, Grape Soda, Cyber Orchid, Midnight Lilac, Blacklight, Lavender Milk, Lilac Cloud, Orchid Glass, Velvet Violet, Orchid Night, Electric Plum, Iris, Obsidian Violet, Wisteria, Lavender Ink, Lilac Petal |
| Pink (12) | Plum Velvet, Synthwave, Cosmic Dusk, Rose Noir, Rose Quartz, Dragonfruit, Pomegranate, Blush Silk, Bubblegum, Pink Lemonade, Fuchsia Pulse, Sakura Silk |
| Brown (8) | Bronze Age, Copper Wire, Desert Night, Espresso, Sandstone, Oat Milk, Terracotta, Toffee Cream |
| Monochrome (9) | Graphite, Paper, Slate Steel, Carbon Mono, Silver Lining, True Black, Silver Mist, Pure White, Ash Stone |
| Gradients (56) | Prism Night, Velvet Sunset, Aurora Borealis, Midnight Lagoon, Crimson Nebula, Neon Horizon, Purple Haze, Cherry Aurora, Deep Space, Molten Gold, Emerald Dream, Ocean Drive, Candy Cloud, Dawn Glow, Iridescent, Rosewater, Solar Flare, Boreal Sky, Sapphire Rose, Copper Twilight, Tropical Night, Amber Tide, Forest Ember, Pink Voltage, Blue Hour, Citrus Noir, Cobalt Ember, Opal Night, Desert Mirage, Moonlit Steel, Peacock Feather, Royal Spectrum, Sorbet Sky, Seaglass Dawn, Lemon Petal, Peach Lagoon, Pearl Prism, Sky Meadow, Coral Mist, Honey Lavender, Ruby Sunrise, Mango Wave, Lemon Lime, Jungle Current, Cyan Aurora, Indigo Spectrum, Fuchsia Lagoon, Bronze Ocean, Noir Prism, Copper Rose, Citrus Clouds, Aqua Bloom, Blue Porcelain, Rose Gold, Pistachio Peach, Arctic Pearl |

There are **112 solid palettes and 56 gradient themes**, including 51 light choices overall. The gradient collection has 38 dark and 18 light choices. Three-colour blends extend across the main background, library sidebar and player bar, with matching preview cards. These gradients are drawn natively and cached for smooth redraws; controls keep readable surfaces. Try **Cyan Aurora**, **Lemon Lime**, **Jungle Current**, or the light **Rose Gold** and **Arctic Pearl**. Solid choices include **Sunflower**, **Lime Wire**, **Cyan Circuit**, **Royal Navy**, **True Black** and **Pure White**.

Each palette colours the page, panels, surfaces, highlights, buttons, outlines, and header tint. Following the system uses Amethyst in dark mode and Lavender Milk in light mode by default. Built-in palettes can also be assigned independently to the system light and dark appearances; no palette file or cached copy is required. Saved light/dark/system selections and existing custom themes still work.

**Colour from album art** lets covers tint the page and visualisers. Turn it off in Appearance for colours driven by your selected theme.

Load your own JSON palettes through **Open themes folder**, choose separate custom themes for light and dark appearances, or use the existing shared desktop presets. See [custom theme format](docs/_reference/settings-and-files.md#custom-themes). The existing custom-file picker supports up to 128 valid palettes; the 168 built-in Motion themes are separate.

## Smooth motion

Interactive colours and press feedback ease over 160 ms. Switching pages, opening a playlist or album, visiting an artist, and using Back or Forward now gives the page a smooth **260 ms slide and fade**. Playlist-to-playlist changes have the same transition. The sidebar, top bar and player controls stay anchored, and clicks and keyboard actions remain immediate. Theme collections keep their 200 ms arrival fade; changing the theme uses a soft circular reveal over 600 ms, alongside native menu and smooth-scroll transitions.

These interface transitions stop requesting frames once they finish, so they add no idle animation loop. Loading more data, scrolling or resizing does not restart the page transition. Motion Music honours the operating system's reduced-motion preference and shows the final state immediately when motion is reduced. Existing audio visualisers keep their own frame-rate controls.

## See the sound

Open **Settings > Appearance > Player bar visualizer** and choose:

| Mode | Effect |
| --- | --- |
| Spectrum | Glowing frequency bars with peak caps |
| Waveform | A neon audio trace |
| Aurora ribbons | Flowing layers of luminous ribbons |
| Radial spectrum | Concentric frequency halos |
| Neon orbit | Comet trails around reactive ellipses |
| Ember field | Rising particles driven by frequency bands |
| Mirror wave | A mirrored waveform ribbon |
| Hyperspace tunnel | Expanding rings with a hyperspace feel |
| Neon waterfall | Cascading frequency tiles with glowing trails |
| Crystal prism | A skyline of luminous crystal facets |
| DNA helix | Twisting strands with glowing crossbars |
| Warp starfield | Stars and light trails rushing from a vanishing point |
| Plasma field | Flowing, colourful interference patterns |
| Ripple pool | Concentric ripples and bright droplets |
| Laser fan | Crossing fans of spectral light beams |
| Fireworks | Bursts of sparks with falling trails |

Visualisation starts off, preserving existing preferences. Click empty space in the bar to cycle modes when **Click player bar to change visualizer** is enabled. All sixteen effects and the Winamp mini player's bars and scope react to Motion Music's own local playback, including supported local files.

**On Windows, the effects can also follow sound from the official Spotify app and other apps on this PC.** While Motion Music's local player is idle, a visible visualiser automatically listens to the default Windows audio output. The **Windows audio** label identifies this input; hover it for its status. It includes other apps and system sounds, and app/system volume can affect the picture. Music playing only on a phone or separate speaker cannot drive these effects unless its audio reaches the PC's default Windows output.

Quiet Windows audio now gets an automatic sensitivity boost for the visualisers, so effects respond more strongly at low listening volumes. The boost eases back for louder peaks. It changes only the visualiser input, leaving music volume and Motion Music's own playback audio unchanged. Muted or silent Windows output still supplies no audio for the effects.

This Windows input uses no microphone and does not save or upload audio. It closes when Motion Music is hidden or minimised, the PC locks, the active visualiser is Off, or the mini player is shaded. It follows changes to the default Windows output and lets the effects settle when the audio is silent or unavailable.

Motion Music's own playback keeps its post-EQ, pre-volume signal: its equalizer changes the picture and its volume control does not. Frame-rate limits and reduced-motion preferences still apply.

The optional MilkDrop/projectM window continues to use Motion Music's own local playback and remains supported by the source's default feature. Builds made with **--no-default-features** include all sixteen player-bar effects and the mini player, but omit MilkDrop.

## Build and run

Spotify playback requires Spotify Premium. Sign in, then use the device menu's **Set up playback here** to approve local playback. Library browsing and playback have separate approvals.

On Windows, with Rust and Visual Studio C++ tools installed:

~~~powershell
cargo build --locked --no-default-features
.\target\debug\motion-music.exe
~~~

The repository pins Rust 1.98.0. If only a newer compatible stable toolchain is installed, use:

~~~powershell
cargo +stable build --locked --no-default-features
~~~

For a release build, add **--release**; the binary is **target/release/motion-music.exe** on Windows and **target/release/motion-music** on Linux/macOS. Native prerequisites are in the [upstream source guide](https://spotifast.rocks/getting-started/#build-from-source). Default features also need the libprojectM toolchain (CMake, libclang, GLEW).

For an isolated demo without a Spotify account:

~~~powershell
cargo run --locked --no-default-features --features demo -- --demo
~~~

Demo visual flags include **--demo-show motion-aurora**, **motion-radial**, **motion-orbit**, **motion-embers**, **motion-mirror**, and **motion-tunnel**. Use **--demo-show motion-themes** to open the filtered gallery for a native screenshot. Screenshot flags are available with **--help**.

## Compatibility and updates

The executable, windows, tray/menu entries, playback device default, and native icons identify as Motion Music. Internal crate names, Spotify authentication identifiers, credential keys, profile folders, IPC identifiers, and package IDs remain compatible with Spotifast. Existing saved settings and sign-ins are preserved; custom device names are kept.

Self-updates are disabled for this custom build. Download new Motion Music editions manually from [this repository's releases](https://github.com/MaybeMotion/Motion-Music/releases), or rebuild this source. Upstream Spotifast downloads do not include these customisations.

## Upstream and licence

Motion Music retains Spotifast's library, search, playlists, queue, Spotify Connect controls, lyrics, native media integration, and mini player. See the [Spotifast user guide](https://spotifast.rocks/using-spotifast/) and [connection details](docs/_reference/how-it-connects.md).

Original project: [crmne/spotifast](https://github.com/crmne/spotifast), by Carmine Paolino and contributors. See [LICENSE](LICENSE) and [CONTRIBUTING.md](CONTRIBUTING.md).

## Playback refusal and loading state

The native playback engine can receive **audio key 0 1** from Spotify. This means Spotify declined to supply the track's decryption key; personal Web API authorization does not resolve it. Motion Music shows the existing explanation and **Play on another device** action. A failed local request now clears its optimistic loading state immediately, while preserving remote play requests and queues. This UI fix does not bypass or resolve the server refusal.

The Motion Music v3 app icon is an original flowing violet-to-rose M inside a circular aubergine disc with a luminous rim and transparent corners. A canonical PNG supplies the window and in-app icon; a matching monochrome M supplies the tray template, with native ICO and macOS exports.

For the supplied v7 portable build, keep **spotifast-portable.txt** beside the executable. Fully quit an older running Motion Music instance before launching the new build; otherwise the existing instance handles the launch.
