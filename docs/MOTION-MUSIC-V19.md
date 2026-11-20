# Motion Music v19

This update lets the visualisers listen only to the app playing your music, such as Soloist, instead of everything on your PC.

## Open this version

1. Fully quit the old Motion Music using **Quit** in the system tray or **Ctrl+Q**.
2. Extract the Windows ZIP and open **Motion Music v19.exe**.
3. Keep **spotifast-portable.txt** beside the app, and move your existing **spotifast-data** folder beside it to keep your settings, sign-in and personal Spotify app.

## Visualisers hear only your Spotify player

When another app on this PC plays the music, such as Soloist acting as a Spotify Connect speaker, the visualisers used to listen to everything Windows played. YouTube, games and notification sounds moved them too.

Now they listen to that player alone: Soloist if it is running, otherwise the Spotify app. Other apps no longer move them, and when the music pauses they rest. Hover the audio badge on the player bar to see which app they are hearing.

**Settings > Appearance > Visualizer listens to** chooses:

| Choice | What the visualisers hear |
| --- | --- |
| Spotify player only | Soloist, else the Spotify app. Everything on the PC while neither is open. This is the default. |
| All PC audio | Everything audible on this PC, as before |

When Motion Music plays the music itself, the visualisers always use its own sound, as before.

This needs Windows 10 version 2004 or later. On an older Windows, the visualisers listen to everything instead.

## Settings file

New optional field: `visualiser_audio` (`player` or `everything`). Existing settings files load unchanged.

## Licences and source

Motion Music is based on Spotifast by Carmine Paolino and contributors. Original MIT credits remain. See the README and **THIRD-PARTY-NOTICES.md** for the included components.
