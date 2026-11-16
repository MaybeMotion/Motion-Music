# Motion Music v14

This update makes page motion gentler. Four ocean-wave transitions join the shuffle, and every straight or angled transition now fades over a soft edge instead of cutting a sharp line. Everything from v13 (favourite themes, theme shortcuts, click-origin reveals, the transition picker and visualiser colours) is included. The **212 themes**, 16 player-bar visualisers and MilkDrop window are unchanged.

## Open this version

1. Fully quit the old Motion Music using **Quit** in the system tray or **Ctrl+Q**. Closing its window can leave it running.
2. Extract **Motion-Music-v14-Windows.zip** and open **Motion Music v14.exe**.
3. Keep **spotifast-portable.txt** beside the app, and move your existing **spotifast-data** folder beside it. That folder holds your settings, sign-in and personal Spotify app (Client ID), so they carry over. It is never included in the download.

Without **spotifast-portable.txt** beside it, the app uses a separate profile in your Windows user folder instead of **spotifast-data**, and asks you to sign in and set up your personal Spotify app again.

## Ocean waves

Four new transitions, one for each swipe direction: **Wave left**, **Wave right**, **Wave up** and **Wave down**. Their edge is a soft curve that swells and rolls sideways as it washes across the page, like the sea running up a beach. Like every page transition they take 360 ms, keep the page content in its final position, and only ever move forward over the old page.

## Softer edges everywhere

The swipes, diagonal sweep, curtain, **diamond iris**, chevron, blinds, cascade, zipper and letterbox now fade over an edge about 20 points wide. The clock hand has a slightly softer edge than before. The expanding, inward, corner and click-origin circles keep their fine edge.

Blinds, cascade and zipper still show a line where one slat or column meets the next, because each part moves on its own. Turn any style off in **Settings > Appearance > Page transitions**.

## Choose your transitions

**Settings > Appearance > Page transitions** now lists twenty styles. Click a style to take it out of the shuffle or put it back. **All** and **None** set every style at once. With every style off, pages change immediately. Back and Forward keep their own swipes.

The settings file names the wave styles `wave-left`, `wave-right`, `wave-up` and `wave-down` in `page_reveals_off`. No other settings change, and existing files load as before.

## Licences and source

Motion Music is based on Spotifast by Carmine Paolino and contributors. Original MIT credits remain. See the README and **THIRD-PARTY-NOTICES.md** for the included components.
