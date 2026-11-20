# Motion Music v17

This update swaps twelve visualisers for six new ones, makes the beat pulse soft and smooth, brings living gradients back to the theme's own colours, and adds click effects and box shapes. Each new look has a setting, so you can tone it down or turn it off.

## Open this version

1. Fully quit the old Motion Music using **Quit** in the system tray or **Ctrl+Q**.
2. Extract the Windows ZIP and open **Motion Music v17.exe**.
3. Keep **spotifast-portable.txt** beside the app, and move your existing **spotifast-data** folder beside it to keep your settings, sign-in and personal Spotify app.

## Visualisers

Aurora ribbons, Radial spectrum, Neon orbit, Ember field, Hyperspace tunnel, Warp starfield, Plasma field, Ripple pool, Laser fan, Fireworks, Kaleidoscope and Rose curves are gone. If one of them was your choice, Motion Music switches to the closest new or remaining one.

Six new ones join the rest:

| Visualiser | What you see |
| --- | --- |
| City skyline | A city at night whose buildings rise with the spectrum, their windows lighting up and twinkling as it gets louder |
| Ocean waves | Four layers of rolling waves with foam on every crest |
| Retro LED meter | A hi-fi meter of LED segments lighting up from the bottom |
| Fireflies | Fireflies drifting in the dark, blinking softly with the music |
| Heart monitor | A glowing heartbeat trace sweeping across a grid, its spikes leaping with the bass |
| Circuit board | Pulses of light racing along circuit traces, the pads glowing with the music |

That makes twenty. They all work on the stage too (**Ctrl+Shift+S**).

## A softer beat pulse

The pulse now eases in over a moment and fades out more slowly, as a soft glow at the top of the player bar and a halo around the play button. It no longer washes the whole bar. Choose how bright it is in **Settings > Appearance > Beat pulse strength**: **Subtle** (the default), **Medium**, or **Strong**, which is close to v16's.

## Living gradients

Living gradients now show exactly the colours your theme has with the setting off, swaying gently between them, so there are no stronger colours and no bands. Solid themes stay solid. **Gradient drift speed** chooses **Slow** (about once a minute, the default), **Medium** or **Fast**.

## Click effects

Clicking Home, Search, My Wrapped, a playlist, a card or a song shows a short effect. Choose it in **Settings > Appearance > Click effect**:

- **Ripple** (the default): a soft ring of light spreads out from where you clicked.
- **Glow**: the item glows and fades.
- **Pop**: an outline springs out from the item.
- **Sparkle**: a few sparks float out from the pointer.
- **Off**: no effect.

Reduced-motion settings turn it off.

## Box shapes

**Settings > Appearance > Box shape** changes the corners of cards, covers, Home's tiles, sidebar rows, songs and My Wrapped's cards: **Rounded** (the usual look), **Square**, **Soft** or **Bubbly**.

## Settings file

New optional fields: `beat_pulse_strength` (`subtle`), `gradient_drift` (`slow`), `click_effect` (`ripple`) and `box_shape` (`rounded`), and the `player_bar_vis` values `skyline`, `ocean`, `led`, `fireflies`, `heartbeat` and `circuit`. Existing settings files load unchanged.

## Licences and source

Motion Music is based on Spotifast by Carmine Paolino and contributors. Original MIT credits remain. See the README and **THIRD-PARTY-NOTICES.md** for the included components.
