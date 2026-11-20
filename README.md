<p align="center"><img src="packaging/icons/spotifast.svg" alt="Motion Music" width="88" height="88"></p>

<h1 align="center">Motion Music</h1>

<p align="center"><strong>Your music, in full colour.</strong><br>A native Spotify client with 212 curated themes and twenty-six audio-reactive visualisers.</p>

Motion Music is a personal customisation of [Spotifast](https://github.com/crmne/spotifast), built with Rust, egui, fastframe, and librespot. Original copyright, licence, and upstream credits are retained. Upstream documentation and release links describe Spotifast; official downloads do not include these customisations.

## Download Motion Music v18

**[Download the Windows portable ZIP](https://github.com/MaybeMotion/Motion-Music/releases/download/motion-v18/Motion-Music-v18-Windows.zip)** or open the [Motion Music v18 release](https://github.com/MaybeMotion/Motion-Music/releases/tag/motion-v18) for the source archive and checksums. Extract the ZIP, keep `spotifast-portable.txt` beside `Motion Music v18.exe`, move your existing `spotifast-data` folder beside it to keep your settings, sign-in and personal Spotify app, and fully quit an older running copy before opening it.

**Mac:** v18 has not been built for Mac yet. The [v12 Apple Silicon installer](https://github.com/MaybeMotion/Motion-Music/releases/download/motion-v12/Motion-Music-v12-macos-arm64.zip) and [v12 Intel installer](https://github.com/MaybeMotion/Motion-Music/releases/download/motion-v12/Motion-Music-v12-macos-x86_64.zip) remain available. See the [Mac installation guide](docs/MAC-INSTALL.md).

See the [v18 guide](docs/MOTION-MUSIC-V18.md) for the beat pulse in your visualiser's colours, more click effects and box shapes, and visualiser shuffle. Earlier guides: [v17](docs/MOTION-MUSIC-V17.md), [v16](docs/MOTION-MUSIC-V16.md), [v15](docs/MOTION-MUSIC-V15.md), [v14](docs/MOTION-MUSIC-V14.md) and [v13](docs/MOTION-MUSIC-V13.md). Downloads are installed manually; the automatic updater remains disabled for this custom build. **motion-v18** identifies this customised portable edition. The underlying Cargo package and executable version remain **0.14.1**.

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

Point at a theme card and click its **heart** to keep it in **Favourites**, a tab that appears at the start of the gallery and opens first once you have some. The **Recent** tab keeps the last twelve themes you applied. **Ctrl+Shift+←** and **Ctrl+Shift+→** step to the previous or next theme, through your favourites when you have at least two and through all 212 otherwise. **Ctrl+Shift+R** picks a random theme of the current light or dark appearance.

**Colour from album art** lets covers tint the page and visualisers. Turn it off in Appearance for colours driven by your selected theme. **Visualizer colours** set to **Theme** keeps the visualiser in your theme's colours, including a gradient theme's three, while pages still follow the cover.

Load your own JSON palettes through **Open themes folder**, choose separate custom themes for light and dark appearances, or use the existing shared desktop presets. See [custom theme format](docs/_reference/settings-and-files.md#custom-themes). The existing custom-file picker supports up to 128 valid palettes; the 212 built-in Motion themes are separate.

## Fun extras

- **Beat pulse** (Settings > Appearance): a soft glow eases in at the top of the player bar and around the play button on each beat, in the visualiser's own colours, found in the bass of the same audio the visualisers use. **Beat pulse strength** chooses Subtle (the default), Medium or Strong.
- **Living gradients** (Settings > Appearance): a gradient theme sways gently between its own colours while music plays, and holds still while it is paused. It keeps exactly the colours it has with the setting off. **Gradient drift speed** chooses Slow (once a minute), Medium or Fast. Solid themes stay solid.
- **Click effect** (Settings > Appearance): clicking Home, Search, My Wrapped, a playlist, a card or a song shows a soft **Ripple** (the default), **Glow**, **Pop**, **Sparkle**, **Rings**, **Bloom**, **Confetti** or a **Wave** washing across it, in the visualiser's colours. Choose **Off** to turn it off.
- **Box shape** (Settings > Appearance): the corners of cards, covers and highlighted rows. **Rounded** is the usual look; **Square**, **Soft**, **Bubbly**, **Pill** (rows and tiles with fully round ends) and **Leaf** (two opposite corners round) change it everywhere at once.
- **Visualizer shuffle** (Settings > Appearance): a different visualiser for each song or every minute. Every change of visualiser, by shuffle or by hand, fades softly from one to the next.
- **Visualizer listens to** (Settings > Appearance, Windows): while another app on this PC plays the music, such as Soloist, the visualisers hear that player alone (Soloist, else the Spotify app), so YouTube, games and notification sounds no longer move them. Choose **All PC audio** for the old behaviour. With neither player open, they hear everything.
- **Theme from this cover** (Settings > Appearance, or **Ctrl+Shift+T**): makes a dark theme from the playing song's cover, saves it in your themes folder as "Cover - album name" and uses it at once.
- **My Wrapped** (in the sidebar under Search): minutes listened, plays, songs, artists and your longest daily streak; badges such as Night owl, Superfan and On a streak; a chart of when you listen and a twelve-week listening calendar; a throwback to your song of the day a week ago; your most played songs, albums and artists from this computer's last 2,000 plays; and Spotify's own top artists, a genre mix and top songs. **Save as image** puts the summary in Pictures\Motion Music, ready to share. Nothing is sent anywhere.
- **Sleep timer** (the clock button in the player bar): fades the music out over its last 45 seconds and pauses it after 5 to 90 minutes, then puts the volume back.

## Smooth motion

Controls now respond with a 120 ms speed ramp. Pages, playlists, albums and artists use a shuffled set of twenty **fixed-page reveals**: **expanding circle**, **inward circle**, **corner circle**, **swipe left**, **swipe right**, **swipe up**, **swipe down**, **diagonal sweep**, **horizontal curtain**, **diamond iris**, **blinds**, **cascade**, **zipper**, **letterbox**, **chevron**, **clock wipe**, and four ocean **waves** (left, right, up and down) whose curved edge, up to 80 points high, swells and rolls sideways as it washes across the page like surf on a beach. Waves take 560 ms; the other styles take 360 ms. Every straight, angled and wave edge fades softly over about 20 points rather than cutting a sharp line; the circles keep their fine edge. The page content stays in its final position throughout.

The expanding circle opens from the place you clicked, so a playlist opened from the sidebar grows out of that side of the page. A theme chosen in the gallery spreads from its card in the same way. Changes made with a shortcut open from the centre. **Back** always swipes the previous page in from the left and **Forward** swipes the next one in from the right, without using up a shuffled style. Choose which styles the shuffle uses in **Settings > Appearance > Page transitions**; turning every style off changes pages at once.

Page reveals finish in 360 ms, down from 560 ms. Their speed builds early and settles gently, completing about 84% of the movement halfway through. Each shuffled set uses every style once, with no consecutive repeat across sets. The sidebar, header and player stay anchored. Repeated clicks on the current destination or empty page space do not stop or restart the animation. Selecting another page mid-animation starts from the visible page composite so rapid navigation stays continuous. Covered controls cannot receive accidental clicks. Scrolling, keyboard editing, resizing or a theme change can settle the reveal safely.

Theme changes use the same ramp for their soft circular reveal over 400 ms, down from 600 ms. Theme collections arrive in 140 ms. Hover and press feedback, switches, row movement, artwork colour changes and lyric highlights also use the shared ramp and shorter timings. Reversing a control or changing its target continues from the value currently visible. Menus and smooth scrolling use shorter native easing. Toasts arrive and leave more quickly while keeping their reading time. Interface effects stop requesting frames when settled and honour the operating system's reduced-motion preference. Music playback, audio-reactive visualiser timing and their separate frame-rate controls stay unchanged.

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
| Mirror wave | A mirrored waveform ribbon |
| Neon waterfall | Cascading frequency tiles with glowing trails |
| Crystal prism | A skyline of luminous crystal facets |
| DNA helix | Twisting strands with glowing crossbars |
| Fireworks | Bursts of sparks with falling trails |
| Synthwave sunset | A striped retro sun over spectrum mountains and a neon grid rushing towards you |
| Sound terrain | The last second of music as glowing ridges receding into the distance |
| Kaleidoscope | Three mirrored mandalas whose petals open with the spectrum |
| Lava lamp | Soft glowing blobs drifting and merging like wax |
| Galaxy | A spiral galaxy of stars around a core that flares with the bass |
| Matrix rain | Columns of glowing marks falling faster with their part of the spectrum |
| Spectrogram | A scrolling heat map of the music, low notes at the bottom |
| Neon orb | A spinning wireframe globe inside a ring of light, swelling with the bass |
| Rose curves | Three layered flowers whose petals reach out with the spectrum |
| Bubbles | Bubbles rising and swelling with their part of the spectrum |
| City skyline | A city at night whose buildings rise with the spectrum and whose windows light up and twinkle as it gets louder |
| Ocean waves | Four layers of rolling waves with foam on every crest, each swelling with the music beneath it |
| Retro LED meter | A hi-fi meter of LED segments lighting up from the bottom |
| Fireflies | Fireflies drifting in the dark, blinking softly and glowing with their part of the spectrum |
| Heart monitor | A glowing heartbeat trace sweeping across a grid, its spikes leaping with the bass |
| Circuit board | Pulses of light racing along circuit traces, with pads glowing beneath the music |

Press **Ctrl+Shift+S**, or the expand button in the player bar, for the **visualiser stage**: the chosen effect fills the whole window in full colour, with the playing song along the bottom. Click or press **Esc** to close it. With the player-bar visualiser off, the stage opens on the galaxy.

Visualisation starts off, preserving existing preferences. Click empty space in the bar to cycle modes when **Click player bar to change visualizer** is enabled, or press **Ctrl+Shift+E** anywhere. The scenes glide smoothly between spectra instead of jumping, and an uncapped frame rate stays at 60 frames a second while the window is behind another. All the effects and the Winamp mini player's bars and scope react to Motion Music's own local playback, including supported local files.

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

Demo visual flags include **--demo-show motion-aurora**, **motion-radial**, **motion-orbit**, **motion-embers**, **motion-mirror**, and **motion-tunnel**. Use **--demo-show motion-themes** to open the filtered gallery for a native screenshot. Screenshot flags are available with **--help**. Use **--demo-show motion-stress** for an isolated repeat-click and rapid-navigation capture.

## Compatibility and updates

The executable, windows, tray/menu entries, playback device default, and native icons identify as Motion Music. Internal crate names, Spotify authentication identifiers, credential keys, profile folders, IPC identifiers, and package IDs remain compatible with Spotifast. Existing saved settings, sign-ins and custom device names are preserved. To have Motion Music control another Spotify Connect speaker as soon as it starts, choose it in **Settings > Playback on this computer > Default speaker**. It is picked as soon as Spotify lists it, unless music is already playing on another device.

Self-updates remain disabled for this custom build, preventing an upstream Spotifast download from replacing Motion Music. Update manually from [Motion Music releases](https://github.com/MaybeMotion/Motion-Music/releases), or rebuild this source. The custom v18 label identifies this portable update; the underlying package version remains 0.14.1.

## Upstream and licence

Motion Music retains Spotifast's library, search, playlists, queue, Spotify Connect controls, lyrics, native media integration, and mini player. See the [Spotifast user guide](https://spotifast.rocks/using-spotifast/) and [connection details](docs/_reference/how-it-connects.md).

Original project: [crmne/spotifast](https://github.com/crmne/spotifast), by Carmine Paolino and contributors. See [LICENSE](LICENSE) and [CONTRIBUTING.md](CONTRIBUTING.md).

## Playback refusal and loading state

The native playback engine can receive **audio key 0 1** from Spotify. This means Spotify declined to supply the track's decryption key; personal Web API authorization does not resolve it. Motion Music shows the existing explanation and **Play on another device** action. A failed local request now clears its optimistic loading state immediately, while preserving remote play requests and queues. This UI fix does not bypass or resolve the server refusal.

The Motion Music v12 app icon is **Vortex Bloom**, an original four-blade symbol in violet, pink, coral and cyan with a transparent background and open centre. A canonical PNG supplies the window and in-app icon; a matching monochrome bloom supplies the tray template, with native ICO and macOS exports.

For the supplied v18 portable build, keep **spotifast-portable.txt** beside the executable. Fully quit an older running Motion Music instance before launching the new build; otherwise the existing instance handles the launch.
