# Motion Music v10

This update adds optional Discord listening activity and replaces page slide-ins with circular, swiping and shaped reveals. The existing **212 themes: 128 solids and 84 gradients**, 16 player-bar visualisers and working MilkDrop window remain.

## Open this version

1. Fully quit the old Motion Music using **Quit** in the system tray or **Ctrl+Q**. Closing its window can leave it running.
2. Extract the Windows ZIP. Open **Motion Music v10.exe**.
3. Keep **spotifast-portable.txt** beside the app. Keep your existing **spotifast-data** folder there to retain your settings and sign-in. It is never included in the download.

## Every main colour

Open **Settings > Appearance > Motion themes**. The **Main colours (16)** and **Main gradients (16)** collections cover **red, orange, yellow, lime, green, teal, cyan, blue, indigo, purple, pink, brown, gold, black, white and grey**.

Solids are named **Motion Red**, **Motion Blue**, and so on. Matching blends are named **Red Horizon**, **Blue Horizon**, and so on. Black, white and grey choices keep neutral colours throughout. Each card applies the theme immediately. All existing colour families and red gradients remain available.

Search **main colours**, **main colors** or **main gradients** to jump to these collections. Ordinary searches such as **red gradient**, **light blue**, **navy** and **white** search the full theme selection. All 212 themes appear in the Theme dropdown and matching system light/dark pickers. Turn **Colour from album art** off to keep the chosen palette throughout.

## Circular and swiping transitions

Pages and playlists now reveal in place through a shuffled mix of ten effects:

- Expanding circle, inward circle and corner circle.
- Swipes to the left, right, top and bottom.
- Diagonal sweep, horizontal curtain and diamond iris.

Page content stays at its final position throughout. The slide-ins, drops, lifts and page fades have been removed. Each reveal takes 560 ms with smooth easing. Every shuffled set uses all ten effects once, with no consecutive repeat across sets.

The header, sidebar and player stay anchored. Click an exposed control or use the keyboard to complete a reveal immediately; covered controls cannot receive accidental clicks. Rapid navigation, scrolling, resizing and theme changes settle the current reveal safely. Theme changes retain their circular reveal, and the system's reduced-motion preference is respected. Interface effects finish without an idle animation loop.

## Show your music on Discord

1. Open the **Discord desktop app** and sign in there as usual.
2. In Motion Music, open **Settings > Discord**.
3. Enable **Show listening activity**. Leave the pre-filled Application ID unchanged.
4. Play a Spotify catalogue song. The status row shows when Discord connects and begins sharing.

The activity can show **Listening to Motion Music**, the song, artist, artwork, playback progress and an **Open in Spotify** button. It follows the confirmed Spotify playback state already visible in Motion Music, including its selected Spotify Connect device. No bot token, client secret or new account authorization is needed.

Sharing starts off. Pausing, private sessions, local-file playback, signing out, turning the switch off or quitting clears the activity. Loading and resume previews are not shared. The client reconnects when Discord opens; check the status row if nothing appears and make sure Discord allows activity sharing.

This is Motion Music's custom Listening activity. Discord's **Listening to Spotify** card and **Listen Along** remain part of Discord's separate Spotify account connection. Use that connection for those Spotify-specific features.

The Application ID field is an advanced option for a personal Discord application. Its default is the public Motion Music ID `1566813127557783714`; it is not a secret. Motion Music sends catalogue metadata to the local Discord client, which publishes the activity through Discord. It sends no audio, Spotify grant, Discord account token or local-file path.

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
