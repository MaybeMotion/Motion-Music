# Motion Music v18

This update colours the beat pulse like your visualiser, adds four click effects, two box shapes and visualiser shuffle, makes every change of visualiser fade softly, and polishes the details.

## Open this version

1. Fully quit the old Motion Music using **Quit** in the system tray or **Ctrl+Q**.
2. Extract the Windows ZIP and open **Motion Music v18.exe**.
3. Keep **spotifast-portable.txt** beside the app, and move your existing **spotifast-data** folder beside it to keep your settings, sign-in and personal Spotify app.

## The beat pulse in your visualiser's colours

The glow at the top of the player bar now sweeps across the same colours as the visualiser, from the bass colour on the left to the treble colour on the right. The halo around the play button takes the colour beneath it. On the visualiser stage, each beat rises softly from the foot of the screen in those colours too. **Visualizer colours** chooses whether they come from the album art or your theme.

## More click effects

**Settings > Appearance > Click effect** now has nine choices, all drawn in the visualiser's colours:

| Effect | What you see |
| --- | --- |
| Ripple | A soft disc of light spreading out from where you clicked, kept inside the item's corners |
| Glow | The item glows and fades |
| Pop | An outline springs out from the item |
| Sparkle | A few sparks float out from the pointer |
| Rings | Three rings of light ripple outwards, one after another |
| Bloom | A soft glow opens from the pointer like a flower |
| Confetti | A little burst of confetti, thrown up and falling |
| Wave | A wave of light washes across the item, like the sea on a beach |
| Off | No effect |

## More box shapes

**Settings > Appearance > Box shape** adds **Pill**, where rows and Home's tiles get fully round ends and covers become soft rounded squares, and **Leaf**, where two opposite corners are round and two nearly square. Covers, their shadows and Liked Songs' tile follow the shape too.

## Visualizer shuffle

**Settings > Appearance > Visualizer shuffle** changes to a different visualiser for **each song** or **every minute**. Choosing one yourself, or clicking the player bar, shows yours straight away until shuffle next moves on. It starts **Off**.

Every change of visualiser now fades softly: the old one fades out and the new one fades in, whether shuffle or you made the change.

## Polish

- Ripples and waves stay inside each item's own corners, whatever the box shape.
- My Wrapped's stat cards keep their colour wash inside rounded corners.
- Card shadows follow each cover's corners.
- Settings with many choices, such as Click effect and Box shape, are menus, so their rows stay tidy.

## Settings file

New optional field: `visualiser_shuffle` (`off`, `song` or `minute`). `click_effect` also takes `rings`, `bloom`, `confetti` and `wave`, and `box_shape` takes `pill` and `leaf`. Existing settings files load unchanged.

## Licences and source

Motion Music is based on Spotifast by Carmine Paolino and contributors. Original MIT credits remain. See the README and **THIRD-PARTY-NOTICES.md** for the included components.
