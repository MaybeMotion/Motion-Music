<p align="center"><img src="packaging/icons/spotifast.svg" alt="Motion Music" width="88" height="88"></p>

<h1 align="center">Motion Music</h1>

<p align="center"><strong>Your music, in full colour.</strong><br>A native Spotify client with 212 curated themes and sixteen audio-reactive player-bar visualisers.</p>

Motion Music is a personal customisation of [Spotifast](https://github.com/crmne/spotifast), built with Rust, egui, fastframe, and librespot. Original copyright, licence, and upstream credits are retained. Upstream documentation and release links describe Spotifast; official downloads do not include these customisations.

## Download Motion Music v10

**[Download the Windows portable ZIP](https://github.com/MaybeMotion/Motion-Music/releases/download/motion-v10/Motion-Music-v10-Windows.zip)** or open the [Motion Music v10 release](https://github.com/MaybeMotion/Motion-Music/releases/tag/motion-v10) for the source archive and checksums. Extract the ZIP, keep `spotifast-portable.txt` beside `Motion Music v10.exe`, and fully quit an older running copy before opening it.

See the [v10 setup and visualiser guide](docs/MOTION-MUSIC-V10.md). Downloads are installed manually; the automatic updater remains disabled for this custom build. **motion-v10** identifies this customised portable edition. The underlying Cargo package and executable version remain **0.14.1**, matching the tested build.

The inherited Spotifast publishing workflows are archived and inactive. See [GitHub upload notes](docs/GITHUB-UPLOAD.md) for the source and publishing details.

## Make it yours

Open **Settings > Appearance > Motion themes**. Start with **Main colours (16)** or **Main gradients (16)** for red, orange, yellow, lime, green, teal, cyan, blue, indigo, purple, pink, brown, gold, black, white and grey. Each has a solid Motion palette and a matching Horizon gradient. Search **main colours** (or **main colors**) or **main gradients** to open these collections directly. The gallery has clearly labelled colour tabs with swatches: **Red, Orange, Yellow, Green, Teal, Cyan, Blue, Purple, Pink, Brown, Monochrome and Gradients**. Every colour collection has both dark and light choices. Click a preview to apply it. Settings search understands colour names and aliases such as **navy**, **aqua**, **grey**, **black** and **white**, as well as combinations such as **light blue** and **blue gradient**. All 212 palettes are built in and available without downloads. They also appear in the main Theme menu and matching Light mode theme or Dark mode theme picker.

| Collection | Themes |
| --- | --- |
| Red (13) | Crimson, Cherry Bomb, Ruby, Scarlet, Blood Moon, Burgundy, Cherry Blossom, Vermilion, Red Velvet, Carmine, Merlot, Poppy Red, Motion Red |
| Orange (11) | Ember, Inferno, Hot Lava, Peach Fuzz, Coral Flare, Tangerine, Apricot, Persimmon, Mandarin, Mango Milk, Motion Orange |
| Yellow (10) | Liquid Gold, Lemon Sorbet, Amber Room, Citron, Marigold, Buttercream, Sunflower, Daffodil, Motion Yellow, Motion Gold |
| Green (14) | Emerald, Toxic Mint, Jade Temple, Malachite, Mint Tea, Forest Canopy, Pine Needle, Olive Grove, Pistachio, Lime Wire, Matcha Latte, Racing Green, Motion Lime, Motion Green |
| Teal (7) | Northern Lights, Seafoam, Tidal Teal, Peacock, Viridian, Aqua Marine, Motion Teal |
| Cyan (8) | Arctic, Glacial Blue, Petrol Blue, Pacific Night, Lagoon Mist, Cyan Circuit, Ice Mint, Motion Cyan |
| Blue (10) | Deep Ocean, Electric Blue, Glacier, Sapphire, Cobalt Ink, Powder Blue, Royal Navy, Cornflower, Motion Blue, Motion Indigo |
| Purple (21) | Amethyst, Purple Rain, Ultraviolet, Violet Hour, Nebula, Grape Soda, Cyber Orchid, Midnight Lilac, Blacklight, Lavender Milk, Lilac Cloud, Orchid Glass, Velvet Violet, Orchid Night, Electric Plum, Iris, Obsidian Violet, Wisteria, Lavender Ink, Lilac Petal, Motion Purple |
| Pink (13) | Plum Velvet, Synthwave, Cosmic Dusk, Rose Noir, Rose Quartz, Dragonfruit, Pomegranate, Blush Silk, Bubblegum, Pink Lemonade, Fuchsia Pulse, Sakura Silk, Motion Pink |
| Brown (9) | Bronze Age, Copper Wire, Desert Night, Espresso, Sandstone, Oat Milk, Terracotta, Toffee Cream, Motion Brown |
| Monochrome (12) | Graphite, Paper, Slate Steel, Carbon Mono, Silver Lining, True Black, Silver Mist, Pure White, Ash Stone, Motion Black, Motion White, Motion Grey |
| Gradients (84) | Prism Night, Velvet Sunset, Aurora Borealis, Midnight Lagoon, Crimson Nebula, Neon Horizon, Purple Haze, Cherry Aurora, Deep Space, Molten Gold, Emerald Dream, Ocean Drive, Candy Cloud, Dawn Glow, Iridescent, Rosewater, Solar Flare, Boreal Sky, Sapphire Rose, Copper Twilight, Tropical Night, Amber Tide, Forest Ember, Pink Voltage, Blue Hour, Citrus Noir, Cobalt Ember, Opal Night, Desert Mirage, Moonlit Steel, Peacock Feather, Royal Spectrum, Sorbet Sky, Seaglass Dawn, Lemon Petal, Peach Lagoon, Pearl Prism, Sky Meadow, Coral Mist, Honey Lavender, Ruby Sunrise, Mango Wave, Lemon Lime, Jungle Current, Cyan Aurora, Indigo Spectrum, Fuchsia Lagoon, Bronze Ocean, Noir Prism, Copper Rose, Citrus Clouds, Aqua Bloom, Blue Porcelain, Rose Gold, Pistachio Peach, Arctic Pearl, Ruby Silk, Scarlet Ember, Garnet Glow, Cherry Lacquer, Rosefire, Crimson Tide, Velvet Ruby, Blood Orange, Strawberry Cream, Cherry Sorbet, Rose Satin, Coral Velvet, Red Horizon, Orange Horizon, Yellow Horizon, Lime Horizon, Green Horizon, Teal Horizon, Cyan Horizon, Blue Horizon, Indigo Horizon, Purple Horizon, Pink Horizon, Brown Horizon, Gold Horizon, Black Horizon, White Horizon, Grey Horizon |

There are **128 solid palettes and 84 gradient themes**, including 63 light choices overall. The gradient collection has 58 dark and 26 light choices. Three-colour blends extend across the main background, library sidebar and player bar, with matching preview cards. These gradients are drawn natively and cached for smooth redraws; controls keep readable surfaces. Twelve more red-led gradients range from deep **Ruby Silk**, **Garnet Glow** and **Cherry Lacquer** to light **Strawberry Cream**, **Rose Satin** and **Coral Velvet**. Search **red gradient**, **dark red gradient** or **light red gradient** to find blends by colour and appearance. Existing favourites such as **Cyan Aurora**, **Lemon Lime**, **Rose Gold** and **Arctic Pearl** remain available. Solid choices include **Sunflower**, **Lime Wire**, **Cyan Circuit**, **Royal Navy**, **True Black** and **Pure White**.

Each palette colours the page, panels, surfaces, highlights, buttons, outlines, and header tint. Following the system uses Amethyst in dark mode and Lavender Milk in light mode by default. Built-in palettes can also be assigned independently to the system light and dark appearances; no palette file or cached copy is required. Saved light/dark/system selections and existing custom themes still work.

**Colour from album art** lets covers tint the page and visualisers. Turn it off in Appearance for colours driven by your selected theme.

Load your own JSON palettes through **Open themes folder**, choose separate custom themes for light and dark appearances, or use the existing shared desktop presets. See [custom theme format](docs/_reference/settings-and-files.md#custom-themes). The existing custom-file picker supports up to 128 valid palettes; the 212 built-in Motion themes are separate.

## Smooth motion

Interactive colours and press feedback keep their 160 ms easing. Pages, playlists, albums, artists, and Back or Forward now use a shuffled set of ten **fixed-page reveals**: **expanding circle**, **inward circle**, **corner circle**, **swipe left**, **swipe right**, **swipe up**, **swipe down**, **diagonal sweep**, **horizontal curtain** and **diamond iris**. The page content stays in its final position throughout; the old slide-ins, drops, lifts and page fades have been removed.

Each reveal takes 560 ms with smooth easing. Each shuffled set uses every style once, with no consecutive repeat across sets. The sidebar, header and player stay anchored. Clicking an exposed control or using the keyboard finishes the reveal immediately; a control covered by the old page cannot receive an accidental click. Rapid navigation, scrolling, resizing or a theme change settles the current reveal safely.

Theme collections keep their 200 ms arrival fade, and theme changes retain the soft circular reveal over 600 ms. Menu and smooth-scroll animations remain. Interface effects stop requesting frames when settled and honour the operating system's reduced-motion preference. Existing audio visualisers keep their separate frame-rate controls.

## Discord listening activity

Open **Settings > Discord** and enable **Show listening activity**. Sharing starts off. Keep the Discord desktop app open; the built-in Motion Music application is ready to use without entering an ID, bot token, client secret or Discord OAuth grant.

Your activity can show **Listening to Motion Music**, the track and artist, album artwork, playback progress and an **Open in Spotify** button. It follows confirmed Spotify catalogue playback in Motion Music, including its selected Spotify Connect device. Pause, private sessions, local-file playback, signing out, disabling the switch or quitting clears the activity. Loading and resume previews are not shared.

This is Motion Music's custom Listening activity. Discord's native **Listening to Spotify** card and **Listen Along** remain part of Discord's separate Spotify account connection. Use that connection if you want those Spotify-specific features.

Motion Music sends catalogue metadata to the local Discord desktop client, which publishes it through Discord. No Spotify grant, Discord account token, local-file path or audio is sent by this integration. It reconnects when Discord opens. The Settings status shows whether it is waiting, connected or unable to publish. The optional **Application ID** field is for a personal Discord application; leave its pre-filled public value unchanged for normal use.

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

This Windows input uses no microphone and makes no recording or audio upload. Bar and mini-player input stops when no visible effect needs it. It can continue while the main window is hidden if the separate MilkDrop window is visible, and stops when the PC locks or the display is off. It follows changes to the default Windows output and lets the effects settle when the audio is silent or unavailable.

Motion Music's own playback keeps its post-EQ, pre-volume signal: its equalizer changes the picture and its volume control does not. Frame-rate limits and reduced-motion preferences still apply.

The supplied Windows build now includes the real **MilkDrop/projectM window**. Open it in **Settings > MilkDrop**, from the top bar, or with **Ctrl+Shift+K**. An original built-in preset works offline immediately; existing optional community preset downloads still run in the background. MilkDrop uses native post-EQ, pre-volume sound while local playback is active, or default Windows output while it is idle. It needs an OpenGL 3.3 capable driver. Errors opening the window are shown instead of leaving its switch on. A short-lived local, file-backed shared-memory ring passes up to half a second of audio to the child window and is removed on close. Builds made with **--no-default-features** omit MilkDrop and show that it is unavailable. See **THIRD-PARTY-NOTICES.md** and **packaging/windows/MILKDROP-BUILD.md** for the included engine's licences and rebuild instructions.

## Build and run

Spotify playback requires Spotify Premium. Sign in, then use the device menu's **Set up playback here** to approve local playback. Library browsing and playback have separate approvals.

For the full Windows build, install Rust, Visual Studio C++ tools, CMake, libclang and vcpkg with static GLEW; follow [MilkDrop build instructions](packaging/windows/MILKDROP-BUILD.md). Then:

~~~powershell
cargo build --locked
.\target\debug\motion-music.exe
~~~

The repository pins Rust 1.98.0. If only a newer compatible stable toolchain is installed, use:

~~~powershell
cargo +stable build --locked
~~~

For a release build, add **--release**; the binary is **target/release/motion-music.exe** on Windows and **target/release/motion-music** on Linux/macOS. Native prerequisites are in the [upstream source guide](https://spotifast.rocks/getting-started/#build-from-source). Default features also need the libprojectM toolchain (CMake, libclang, GLEW).

For an isolated demo without a Spotify account:

~~~powershell
cargo run --locked --no-default-features --features demo -- --demo
~~~

Demo visual flags include **--demo-show motion-aurora**, **motion-radial**, **motion-orbit**, **motion-embers**, **motion-mirror**, and **motion-tunnel**. Use **--demo-show motion-themes** to open the filtered gallery for a native screenshot. Screenshot flags are available with **--help**.

## Compatibility and updates

The executable, windows, tray/menu entries, playback device default, and native icons identify as Motion Music. Internal crate names, Spotify authentication identifiers, credential keys, profile folders, IPC identifiers, and package IDs remain compatible with Spotifast. Existing saved settings and sign-ins are preserved; custom device names are kept.

Self-updates remain disabled for this custom build, preventing an upstream Spotifast download from replacing Motion Music. Update manually from [Motion Music releases](https://github.com/MaybeMotion/Motion-Music/releases), or rebuild this source. The custom v10 label identifies this portable update; the underlying package version remains 0.14.1.

## Upstream and licence

Motion Music retains Spotifast's library, search, playlists, queue, Spotify Connect controls, lyrics, native media integration, and mini player. See the [Spotifast user guide](https://spotifast.rocks/using-spotifast/) and [connection details](docs/_reference/how-it-connects.md).

Original project: [crmne/spotifast](https://github.com/crmne/spotifast), by Carmine Paolino and contributors. See [LICENSE](LICENSE) and [CONTRIBUTING.md](CONTRIBUTING.md).

## Playback refusal and loading state

The native playback engine can receive **audio key 0 1** from Spotify. This means Spotify declined to supply the track's decryption key; personal Web API authorization does not resolve it. Motion Music shows the existing explanation and **Play on another device** action. A failed local request now clears its optimistic loading state immediately, while preserving remote play requests and queues. This UI fix does not bypass or resolve the server refusal.

The Motion Music v3 app icon is an original flowing violet-to-rose M inside a circular aubergine disc with a luminous rim and transparent corners. A canonical PNG supplies the window and in-app icon; a matching monochrome M supplies the tray template, with native ICO and macOS exports.

For the supplied v10 portable build, keep **spotifast-portable.txt** beside the executable. Fully quit an older running Motion Music instance before launching the new build; otherwise the existing instance handles the launch.
