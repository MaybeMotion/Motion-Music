# Motion Music v7

Motion Music now includes **168 themes: 112 solid palettes and 56 gradients**, with gentle, smooth animation throughout the interface. All 16 bar visualisers, the circular purple-and-rose **M** icon and automatic sensitivity for quiet Windows audio are retained. This update focuses on appearance and interaction; it does not include a new Spotify playback engine.

## Open this version

1. Fully quit the old Motion Music. Choose **Quit** from its system-tray menu, or press **Ctrl+Q** in its window. Closing the window can leave it running in the tray.
2. Open **Motion Music v7.exe**.
3. Keep **spotifast-portable.txt** beside the executable. Keep any existing **spotifast-data** folder there too, so your profile and sign-in stay available. The app creates this folder on first launch if needed.

The old and new versions use the same portable profile. Fully quitting the old copy ensures the new version opens.

## Choose a theme

Open **Settings → Appearance → Motion themes**. Click a preview card to apply it immediately. The gallery now has colour-labelled tabs with swatches:

- **Red (12), Orange (10), Yellow (8)** for warm colours.
- **Green (12), Teal (6), Cyan (7), Blue (8)** for greens and cool colours.
- **Purple (20), Pink (12), Brown (8)** for violets, roses and earth tones.
- **Monochrome (9)** for black, white and grey.
- **Gradients (56):** 38 dark and 18 light blends.

Every colour collection includes both dark and light choices, with 51 light themes overall. Try **Sunflower**, **Lime Wire**, **Cyan Circuit**, **Royal Navy**, **Fuchsia Pulse** or **Toffee Cream**. **True Black** and **Pure White** provide neutral choices.

Use the Settings search box to find a palette by name or colour. It understands aliases such as **navy**, **aqua**, **grey**, **black** and **white**, and combinations such as **light blue** or **blue gradient**. The main **Theme** dropdown also includes all 168 palettes. With **Follow system** selected, the **Light mode theme** and **Dark mode theme** pickers include matching Motion palettes.

For blends, open the **Gradients** tab and try **Cyan Aurora**, **Lemon Lime**, **Jungle Current**, **Rose Gold** or **Arctic Pearl**. Existing favourites remain available. The three-colour gradients cover the main background, sidebar and player bar, and previews show the same blend.

Turn **Colour from album art** off if you want the selected theme's colours throughout the app.

## Smooth page transitions

Switching pages, opening playlists, albums and artists, and using Back or Forward now gives the page a **260 ms slide and fade**. Switching directly from one playlist to another has the same effect. The sidebar, top bar and player controls stay anchored, and clicks and keyboard actions still work immediately.

Buttons, navigation, song rows, cover cards and theme previews retain their 160 ms interaction easing. Theme-collection changes keep a 200 ms arrival fade, and theme changes use a soft circular reveal over 600 ms. Toasts ease in and out.

Transitions finish without adding an idle animation loop. Scrolling, resizing, ordinary redraws and newly loaded data do not restart page motion. Motion Music follows your operating system's reduced-motion preference and shows the final state immediately when motion is reduced.

## Play music through Spotify

If Motion Music's local player reports that Spotify refused an audio key, use the official Spotify desktop app for playback. Authorising a personal API Client ID does not remove that server refusal. The playback engine's [upstream audio-key issue](https://github.com/librespot-org/librespot/issues/1649) tracks the same failure.

To listen with reactive visuals on this PC:

1. Open the official Spotify desktop app and start a song.
2. Make sure it is audible through your default Windows sound output.
3. In Motion Music, open the **device menu beside the volume control** and select the device running the official Spotify player.
4. Keep Motion Music visible and turn on a visualiser below.

Audio plays through the official Spotify app. Motion Music can also control a phone or separate speaker, but those devices cannot drive the visuals unless their audio reaches this PC's default Windows output. The appearance changes do not address that native playback error.

## Choose a visualiser

Open **Settings → Appearance → Player bar visualizer** and choose one of 16 effects.

All existing effects remain available: **Spectrum**, **Waveform**, **Aurora ribbons**, **Radial spectrum**, **Neon orbit**, **Ember field**, **Mirror wave**, **Hyperspace tunnel**, **Neon waterfall**, **Crystal prism**, **DNA helix**, **Warp starfield**, **Plasma field**, **Ripple pool**, **Laser fan** and **Fireworks**.

Try Crystal prism for glowing facets, DNA helix for twisting strands, or Warp starfield for rushing light trails.

Enable **Click player bar to change visualizer** to cycle effects by clicking empty player-bar space.

When Motion Music's own player is idle, the **Windows audio** label shows that the effects are following your default Windows output. Hover the label for status and details. All sixteen bar effects and the Winamp mini player's bars and scope can use this sound, including the official Spotify app, other apps and system sounds. App/system volume can affect the picture.

Quiet Windows audio now gets an automatic sensitivity boost for the visualisers, so effects respond more strongly at low listening volumes. The boost eases back for louder peaks. It changes only the visualiser input, leaving music volume and Motion Music's own playback audio unchanged. Muted or silent Windows output still supplies no audio for the effects.

No microphone is used, and this audio is never saved or uploaded. The Windows input stops when Motion Music is hidden or minimised, the PC locks, visualisers are Off, or the mini player is shaded. It follows your default-output changes and settles when the sound is silent or unavailable.

If nothing moves, check that Spotify is audible on the default Windows output, keep Motion Music visible, select an effect, and hover **Windows audio** for its status.

Motion Music's own local playback still uses its post-EQ, pre-volume sound. The supplied executable includes all sixteen bar visualisers; optional MilkDrop is omitted. MilkDrop in source builds continues to use Motion Music's own local playback.

Based on Spotifast by Carmine Paolino and contributors. Original MIT licence and upstream credits are retained.
