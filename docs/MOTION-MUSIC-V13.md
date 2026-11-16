# Motion Music v13

This update makes the themes easier to live with and the page motion more varied. You can keep favourite themes, step between them from the keyboard, and choose which page transitions you see. Six new transitions join the existing ten, and reveals now open from the place you clicked. The **212 themes**, 16 player-bar visualisers and MilkDrop window are unchanged.

## Open this version

1. Fully quit the old Motion Music using **Quit** in the system tray or **Ctrl+Q**. Closing its window can leave it running.
2. Extract **Motion-Music-v13-Windows.zip** and open **Motion Music v13.exe**.
3. Keep **spotifast-portable.txt** beside the app. Move your existing **spotifast-data** folder beside it to keep your settings and sign-in. It is never included in the download.

## Favourite and recent themes

Open **Settings > Appearance > Motion themes**.

- Point at a theme card and click the **heart** beside its name to keep it. The heart stays filled on starred themes and pops as it changes.
- A **Favourites** tab appears at the start of the gallery once you have starred a theme, and the gallery opens on it.
- A **Recent** tab keeps the last twelve themes you applied, newest first.

Both tabs work with the gallery search. Favourites and recents are saved in your settings and survive restarts.

## Theme and visualiser shortcuts

| Keys | What they do |
| --- | --- |
| Ctrl+Shift+← / Ctrl+Shift+→ | Previous or next theme. With two or more favourites these step through your favourites; otherwise through all 212 |
| Ctrl+Shift+R | Surprise me: a random theme of the current light or dark appearance |
| Ctrl+Shift+E | Next player bar visualiser |

On a Mac use Cmd in place of Ctrl. The theme arrows are left alone while you are typing in a text field, where they select by word. All three are listed in the keyboard shortcuts dialog (**Ctrl+/** or **?**).

## Reveals that start where you clicked

- The **expanding circle** page transition opens from your click. A playlist opened from the sidebar grows out of that side of the page; a card clicked on a page grows out of the card.
- A **theme change** spreads from the theme card you clicked.
- A change made with a shortcut, a link or the system appearance opens from the centre, as before.

## Back and Forward have a direction

**Back** always swipes the previous page in from the left. **Forward** swipes the next page in from the right. This applies to the top-bar buttons, Alt+← and Alt+→, and the mouse's side buttons. They do not use up a style from the shuffle.

## Six more page transitions

The shuffle now has sixteen styles. The new ones are:

- **Blinds**: horizontal slats that open together.
- **Cascade**: columns that drop open one after another from the left.
- **Zipper**: bands that slide open in alternating directions.
- **Letterbox**: opens from the middle towards the top and bottom.
- **Chevron**: an arrowhead sweeping across the page.
- **Clock wipe**: a hand sweeping once around the centre.

Like the existing styles they take 360 ms, keep the page content in its final position, leave covered controls unclickable until they are revealed, and are skipped when the system asks for reduced motion.

## After v13: ocean waves and softer edges

The source on `main` now adds four **wave** transitions, one for each swipe direction. Their edge is a curve that swells and rolls sideways as it washes across the page, like the sea running up a beach. Every straight and angled transition, such as the diamond iris, chevron and blinds, now fades over a soft edge about 20 points wide instead of a sharp line. The circles are unchanged. These are not in the v13 download.

## Choose your transitions

**Settings > Appearance > Page transitions** lists all sixteen styles. Click a style to take it out of the shuffle or put it back. **All** and **None** set every style at once. With every style off, pages change immediately with no animation. Back and Forward keep their swipes unless every style is off.

## Your speaker is called Motion Music

If your computer still appeared in Spotify's device list as **Spotifast**, the old default name, it now appears as **Motion Music**. A name you chose yourself in **Settings > Playback on this computer > Device name** is kept.

## Visualiser colours

**Settings > Appearance > Visualizer colours** has two choices:

- **Album art** (the default) follows the playing cover while **Colour from album art** is on, as before.
- **Theme** keeps the visualiser in your theme's colours whatever is playing. A gradient theme uses its three colours; a solid theme uses its accent.

This lets pages keep following the cover while the visualiser matches a gradient theme.

## Settings file

Four optional fields are added. Existing settings files work unchanged, and the three lists are written only once they are used.

| Field | Default | Meaning |
| --- | --- | --- |
| `visualizer_colours` | `auto` | `auto` or `theme` |
| `favourite_themes` | absent | Starred themes, such as `motion-theme:crimson` |
| `recent_themes` | absent | The last twelve themes applied |
| `page_reveals_off` | absent | Transition styles left out of the shuffle |

## Licences and source

Motion Music is based on Spotifast by Carmine Paolino and contributors. Original MIT credits remain. See the README and **THIRD-PARTY-NOTICES.md** for the included components.
