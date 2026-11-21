# Motion Music v20

This update adds karaoke lyrics, a weekly recap, a listening clock and heatmap, album art that moves, a fade between speakers, a startup animation, global shortcuts, a fuller tray menu, eight mini player themes and a sleep timer you can switch on in Settings. My Wrapped also opens and scrolls much faster.

## Open this version

1. Fully quit the old Motion Music using **Quit** in the system tray or **Ctrl+Q**.
2. Extract the Windows ZIP and open **Motion Music v20.exe**.
3. Keep **spotifast-portable.txt** beside the app, and move your existing **spotifast-data** folder beside it to keep your settings, sign-in and personal Spotify app.

## Settings > Extras

A new section with a switch for each extra. They all start on.

| Extra | What it does |
| --- | --- |
| Fade between speakers | When the music moves to another speaker, the visualisers and beat pulse fade out and back in |
| Moving album art | The playing cover drifts towards the pointer and breathes slowly, without jumping on the beat |
| Karaoke lyrics | In full screen lyrics, the line being sung fills from the left in the visualiser's colours |
| Weekly recap | When a new week starts, a card shows last week's listening |
| Global shortcuts | Windows: control playback from any app (below) |
| Startup animation | A quick logo reveal in your theme's colours as Motion Music opens, about half a second, once per launch; a click or key skips it |
| Startup animation style | Bloom, Iris, Spin, Curtain, Glow, Sweep, Pulse, Zoom, Mosaic, or Random for a different one each launch. **Preview** plays it now |
| Sleep timer | Turns the sleep timer on or off |
| Sleep timer length | 15, 30, 45, 60, 90 or 120 minutes |
| Sleep timer on the player bar | Shows or hides the sleep timer's clock button on the player bar |
| Parallax | Album and playlist headers shrink and fade as they scroll away, and soft clouds of colour behind Home follow the pointer |
| Cover sheen | A band of light glides across a cover as the pointer moves over it |

## Settings > Customise

| Setting | Choices |
| --- | --- |
| Interface style | Solid, or **Glass**: frosted, see-through panels over a blurred backdrop, like iOS |
| Glass frost | Light, Medium or Heavy: how much of the backdrop shows through |
| Glass backdrop | The playing song's cover, blurred, or soft clouds of your theme's colours |
| Accent colour | The theme's own, or any of fourteen colours, over any theme |
| Play button | Circle, Rounded, Ring or Gradient |
| Seek bar | Classic, Thick, Line, Capsule, Segments, Dots, Glow, Gradient, Neon or Wave |
| Visualizer intensity | Faint, Soft or Full |
| Cursor trail | Off, Sparkle or Comet |
| Background pattern | None, Dots, Grid, Diagonal, Waves or Stars |
| Pattern motion | The pattern drifts slowly across the page |
| Song row highlight | Fill, Outline, Accent bar or Glow |
| Your name | Home greets you by it |
| Show time left | The player bar counts down; clicking the time swaps too |

**Visualizer colours** in Appearance also has six schemes of its own: Rainbow, Fire, Ice, Neon, Sunset and Forest.

## Favourite visualisers

In **Settings > Appearance > Player bar visualizer**, each visualiser has a heart. Hearted ones come first in the list. With two or more favourites, clicking the player bar and visualizer shuffle move between your favourites alone.

## Karaoke lyrics

Open full screen lyrics on a song with timed words. The line being sung fills from left to right as it is sung, in the visualiser's colours, with a soft glow behind the words already sung. A line that wraps fills row by row.

## Your week in music

My Wrapped now starts with **Your week in music**: last week's minutes, plays, songs and longest streak, counting up, with the song and artist of the week. When a new week starts, the same recap appears once as a card, with **See My Wrapped** to open the page. A week with fewer than three plays shows no card.

## Listening clock and heatmap

**When you listen** is now a round clock, midnight at the top, with a petal for each hour reaching out as far as that hour's music, in the visualiser's colours. **When you listen most** is a heatmap of every hour of every weekday. The twelve-week calendar stays.

## Global shortcuts and the tray

On Windows, these work from any app, even with Motion Music hidden in the tray:

| Shortcut | Action |
| --- | --- |
| Ctrl+Alt+Space | Play or pause |
| Ctrl+Alt+Right | Next |
| Ctrl+Alt+Left | Previous |
| Ctrl+Alt+Up | Volume up |
| Ctrl+Alt+Down | Volume down |

A shortcut that another app already uses is left to that app. The **tray menu** now names the playing song at the top, opens the **mini player**, and turns the **sleep timer** on or off. Its tooltip names the song too.

## Mini player themes

**Settings > Winamp skins > Skin** lists eight themes of Motion Music's own before your installed skins: **Neon Night**, **Sunset**, **Ocean**, **Forest**, **Mint**, **Ember**, **Candy** and **Mono**. Each recolours the built-in skin, its playlist and its visualiser. **Random** picks among them too.

## Sleep timer

The fade now lasts the whole last minute. Turn the timer on in **Settings > Extras**, from the tray menu, or from the clock button in the player bar.

## Faster

- **My Wrapped** works out its stats once when a play is added, instead of on every frame. Measured on a test build with 2,000 plays, a frame of the page took about a fifth of the time it did before.
- Moving album art only moves on frames the app is already drawing, so it never keeps a still window busy.

## Settings file

New optional fields: `speaker_fade`, `moving_art`, `karaoke_lyrics`, `weekly_recap`, `global_shortcuts`, `startup_animation`, `sleep_timer_button`, `parallax` and `cover_sheen` (all `true` or `false`), `startup_style` (`bloom`, `iris`, `spin`, `curtain`, `glow`, `sweep`, `pulse`, `zoom`, `mosaic` or `random`), `favourite_visualisers` (a list such as `["galaxy", "orb"]`), `interface_style` (`solid` or `glass`), `glass_frost` (`light`, `medium` or `heavy`), `glass_backdrop` (`albumart` or `theme`), `custom_accent` (red, green and blue such as `[236, 72, 153]`), `play_button` (`circle`, `rounded`, `ring` or `gradient`), `seek_style` (`classic`, `thick`, `line`, `capsule`, `segments`, `dots`, `glow`, `gradient`, `neon` or `wave`), `pattern_motion` (`true` or `false`), `vis_intensity` (`faint`, `soft` or `full`), `cursor_trail` (`off`, `sparkle` or `comet`), `background_pattern` (`none`, `dots`, `grid`, `diagonal`, `waves` or `stars`), `row_hover` (`fill`, `outline`, `bar` or `glow`), `greeting_name` and `time_remaining`. `visualizer_colours` also takes `rainbow`, `fire`, `ice`, `neon`, `sunset` and `forest`, `weekly_recap_seen` (a week such as `2026-W47`) and `sleep_timer_minutes` (15 to 120). `skin` also takes the mini player themes, such as `motion:neon-night`. Existing settings files load unchanged.

## Licences and source

Motion Music is based on Spotifast by Carmine Paolino and contributors. Original MIT credits remain. See the README and **THIRD-PARTY-NOTICES.md** for the included components.
