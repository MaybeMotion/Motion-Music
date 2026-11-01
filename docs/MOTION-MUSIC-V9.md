# Motion Music v9

This update adds circular wallpaper-style page reveals, main-colour collections, and the real MilkDrop window. There are **212 themes: 128 solids and 84 gradients**, including 63 light themes overall. All 180 previous themes and the 16 bar visualisers remain.

## Open this version

1. Fully quit the old Motion Music using **Quit** in the system tray or **Ctrl+Q**. Closing its window can leave it running.
2. Extract the Windows ZIP. Open **Motion Music v9.exe**.
3. Keep **spotifast-portable.txt** beside the app. Keep your existing **spotifast-data** folder there to retain your settings and sign-in. It is never included in the download.

## Every main colour

Open **Settings > Appearance > Motion themes**. The **Main colours (16)** and **Main gradients (16)** collections cover **red, orange, yellow, lime, green, teal, cyan, blue, indigo, purple, pink, brown, gold, black, white and grey**.

Solids are named **Motion Red**, **Motion Blue**, and so on. Matching blends are named **Red Horizon**, **Blue Horizon**, and so on. Black, white and grey choices keep neutral colours throughout. Each card applies the theme immediately. All existing colour families and red gradients remain available.

Search **main colours**, **main colors** or **main gradients** to jump to these collections. Ordinary searches such as **red gradient**, **light blue**, **navy** and **white** search the full theme selection. All 212 themes appear in the Theme dropdown and matching system light/dark pickers. Turn **Colour from album art** off to keep the chosen palette throughout.

## Wallpaper-style transitions

Page and playlist changes now randomly choose from **ten** effects. The original six stay: **rise and fade**, **glide left**, **glide right**, **soft drop**, **diagonal lift**, and **dissolve**. Four new effects reveal the new page through the old one: **expanding circle**, **inward circle**, **diagonal sweep**, and **horizontal curtain**. These take inspiration from circular and wiping Linux wallpaper animations.

The original arrivals take 260 ms; the new reveals take 560 ms. Each shuffled set uses every animation once, with no consecutive repeat across sets. The header, sidebar and player stay in place. Clicking an exposed control or using the keyboard finishes the reveal immediately. Covered controls cannot receive an accidental click. Rapid navigation, scrolling and resizing settle the current reveal safely.

Theme changes retain their soft 600 ms circular reveal. Hover/press easing, theme collection fades and menu transitions remain. Interface animations finish without an idle animation loop and honour the operating system's reduced-motion preference.

## MilkDrop now opens

This download includes the actual **projectM MilkDrop engine**. Open **Settings > MilkDrop > MilkDrop window**, use the top-bar control, or press **Ctrl+Shift+K**.

An original built-in preset works immediately, even offline. Optional community preset packs still download in the background when the preset folder is empty. You can also add your own .milk files in the presets folder. Press **?** or **F1** in MilkDrop for shortcuts.

On Windows, MilkDrop can follow sound from the official Spotify app and other apps on your default Windows output whenever Motion Music's own local player is idle. Quiet audio gets the existing automatic sensitivity boost. If native playback is active, it follows the post-EQ, pre-volume signal instead.

MilkDrop has its own frame-rate and resolution controls. It needs an **OpenGL 3.3** capable graphics driver. A startup failure now shows an error and clears the open switch. Source builds made with **--no-default-features** omit the engine and show that MilkDrop is unavailable.

A visible MilkDrop window can keep listening while the main window is hidden. Capture stops when no visible effect needs it, the PC locks, or the display turns off. No microphone, recording or audio upload is used. A short-lived local file-backed shared-memory buffer passes up to half a second of stereo audio to the child window and is removed on close.

## Spotify playback

The MilkDrop fix does not change Spotify's playback service. If Spotify refuses Motion Music's audio key, play music through the official Spotify desktop app and select its device in Motion Music's device menu. Keep Spotify audible through the default Windows output to drive the visuals. A phone or separate speaker cannot drive them unless its sound reaches this PC.

All 16 player-bar effects remain in **Settings > Appearance > Player bar visualizer**, including Aurora ribbons, Crystal prism, DNA helix, Warp starfield, Plasma field and Fireworks.

## Licences and source

Motion Music is based on Spotifast by Carmine Paolino and contributors. Original MIT credits remain. The full build also includes projectM/projectm-sys and other third-party components. Keep **THIRD-PARTY-NOTICES.md** and **LICENSES** from the ZIP; they include the LGPL/GPL licence texts and source/rebuild links. The release provides separate application and third-party source archives. See **packaging/windows/MILKDROP-BUILD.md** in the source for rebuilding or relinking with a modified engine. No restriction on debugging modifications to LGPL components is added.
