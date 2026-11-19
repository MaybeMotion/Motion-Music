# Motion Music v15

This update adds five new visualisers and a full-window stage to show them off, a beat pulse, slowly drifting gradients, themes made from album covers, your own Wrapped and a sleep timer. It also lets you choose a default speaker.

## Open this version

1. Fully quit the old Motion Music using **Quit** in the system tray or **Ctrl+Q**.
2. Extract the Windows ZIP and open **Motion Music v15.exe**.
3. Keep **spotifast-portable.txt** beside the app, and move your existing **spotifast-data** folder beside it to keep your settings, sign-in and personal Spotify app.

## Five new visualisers

Choose them in **Settings > Appearance > Player bar visualizer**, click empty player-bar space, or press **Ctrl+Shift+E**.

| Visualiser | What you see |
| --- | --- |
| Synthwave sunset | A striped retro sun that swells with the bass, over spectrum mountains and a neon grid rushing towards you |
| Sound terrain | The last second of music as glowing ridges receding into the distance |
| Kaleidoscope | Three mirrored mandalas whose petals open with the spectrum |
| Lava lamp | Soft glowing blobs drifting and merging like wax |
| Galaxy | A spiral galaxy of stars around a core that flares with the bass |

## The visualiser stage

Press **Ctrl+Shift+V** to fill the whole window with the chosen visualiser in full colour, with the playing song along the bottom. Click anywhere or press **Esc** to close it. If the player-bar visualiser is off, the stage opens on the galaxy. All twenty-one effects work on the stage.

## Beat pulse

Turn on **Settings > Appearance > Beat pulse**. On each beat a ring springs out from the play button and the top of the player bar glows softly in your accent colour. The beat is found in the bass of the same audio the visualisers use: Motion Music's own playback, or on Windows the sound playing on this PC. Reduced-motion settings turn it off.

## Living gradients

Turn on **Settings > Appearance > Living gradients** with a gradient theme. Its colours drift slowly across the page, sidebar and player bar while music plays, and hold still while it is paused, so nothing jumps when it starts again.

## Theme from this cover

While a song is playing, press **Ctrl+Shift+T** or use **Settings > Appearance > Theme from this cover**. Motion Music builds a dark theme from the cover's colours, saves it in your themes folder as "Cover - album name" and switches to it. Choose it again any time from the Theme list. A cover with no colour, such as a black-and-white one, cannot make a theme.

## My Wrapped

Open **My Wrapped** in the sidebar under Search.

- Minutes listened, plays, different songs and artists, and your longest run of days in a row.
- **When you listen**: a chart of the hours you play music, your favourite hour and your busiest day.
- **On repeat** and **Your most played artists here**, from the plays this computer has recorded (the last 500). Click a song to play it or an artist to open them.
- **Spotify's top artists** for the last six months with the genres they share, and **Spotify's top songs** for the last four weeks.

Nothing is uploaded: the stats are worked out on this computer, and the top lists are your own from Spotify.

## Sleep timer

Click the clock button in the player bar and choose 5 to 90 minutes. In the last 45 seconds the music fades out, then it pauses and the volume is put back for next time. The button is lit while a timer is set; its tooltip shows the minutes left, and **Turn off** cancels it.

## Default speaker

**Settings > Playback on this computer > Default speaker** chooses a Spotify Connect speaker for Motion Music to control as soon as it starts, instead of this computer. It is picked once Spotify lists it, unless music is already playing on another device or you choose one yourself first. A device name you saved is now always kept as it is.

## Shortcuts

| Keys | What they do |
| --- | --- |
| Ctrl+Shift+V | Visualiser stage |
| Ctrl+Shift+T | Make a theme from this cover |
| Ctrl+Shift+E | Next player bar visualiser |
| Ctrl+Shift+← / → | Previous or next theme |
| Ctrl+Shift+R | Random theme |

## Settings file

New optional fields: `beat_pulse` and `living_gradients` (both `false`), `default_speaker` (absent) and five more `player_bar_vis` values: `synthwave`, `terrain`, `kaleidoscope`, `lava` and `galaxy`. Existing settings files load unchanged.

## Licences and source

Motion Music is based on Spotifast by Carmine Paolino and contributors. Original MIT credits remain. See the README and **THIRD-PARTY-NOTICES.md** for the included components.
