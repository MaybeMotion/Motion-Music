//! User preferences, stored as one readable JSON file.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Local Library identity only. Never sent to Spotify as a context URI.
pub const LIKED_SONGS_KEY: &str = "spotifast:liked-songs";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LibraryShelf {
    #[default]
    Playlists,
    Albums,
    Artists,
    Podcasts,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibrarySort {
    Library,
    RecentlyPlayed,
    Name,
    RecentlyAdded,
    Local,
    Spotify,
}

impl LibrarySort {
    pub fn supports(self, shelf: LibraryShelf) -> bool {
        match self {
            Self::RecentlyPlayed | Self::Name | Self::Library => true,
            Self::RecentlyAdded => matches!(shelf, LibraryShelf::Albums | LibraryShelf::Podcasts),
            Self::Local | Self::Spotify => shelf == LibraryShelf::Playlists,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeChoice {
    /// A bundled Motion Music palette. Existing saved choices remain readable.
    Motion(crate::theme::MotionTheme),
    Dark,
    Light,
    #[default]
    System,
}

/// How the tray icon is drawn on Linux and Windows: in colour, or white or
/// black to match the panel. The macOS menu bar always gets the template
/// image, which it draws black or white to match itself.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrayIcon {
    #[default]
    Colour,
    White,
    Black,
}

/// Whether a Home shelf is drawn. Hidden shelves still refresh normally.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HomeShelfSettings {
    pub visible: bool,
}

impl Default for HomeShelfSettings {
    fn default() -> Self {
        Self { visible: true }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HomeSettings {
    pub made_for_you: HomeShelfSettings,
    pub recommendations: HomeShelfSettings,
}

/// What moves behind the player bar's controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlayerBarVis {
    #[default]
    Off,
    Spectrum,
    Waveform,
    Mirror,
    Waterfall,
    Prism,
    Helix,
    Synthwave,
    Terrain,
    Lava,
    #[serde(alias = "starfield", alias = "tunnel")]
    Galaxy,
    Matrix,
    Spectrogram,
    // Effects removed in v17 load as the nearest one still offered.
    #[serde(
        alias = "orbit",
        alias = "radial",
        alias = "kaleidoscope",
        alias = "rose"
    )]
    Orb,
    Bubbles,
    Skyline,
    #[serde(alias = "aurora", alias = "plasma", alias = "ripples")]
    Ocean,
    #[serde(alias = "lasers")]
    Led,
    #[serde(alias = "embers", alias = "fireworks")]
    Fireflies,
    Heartbeat,
    Circuit,
}

impl PlayerBarVis {
    /// All modes in the same order as the settings picker and click cycle.
    pub const ALL: [Self; 21] = [
        Self::Off,
        Self::Spectrum,
        Self::Waveform,
        Self::Mirror,
        Self::Waterfall,
        Self::Prism,
        Self::Helix,
        Self::Synthwave,
        Self::Terrain,
        Self::Lava,
        Self::Galaxy,
        Self::Matrix,
        Self::Spectrogram,
        Self::Orb,
        Self::Bubbles,
        Self::Skyline,
        Self::Ocean,
        Self::Led,
        Self::Fireflies,
        Self::Heartbeat,
        Self::Circuit,
    ];

    pub fn label(self, locale: crate::i18n::Locale) -> std::borrow::Cow<'static, str> {
        use crate::i18n::gettext;
        match self {
            Self::Off => gettext(locale, "Off"),
            Self::Spectrum => gettext(locale, "Spectrum"),
            Self::Waveform => gettext(locale, "Waveform"),
            Self::Mirror => gettext(locale, "Mirror wave"),
            Self::Waterfall => gettext(locale, "Neon waterfall"),
            Self::Prism => gettext(locale, "Crystal prism"),
            Self::Helix => gettext(locale, "DNA helix"),
            Self::Synthwave => gettext(locale, "Synthwave sunset"),
            Self::Terrain => gettext(locale, "Sound terrain"),
            Self::Lava => gettext(locale, "Lava lamp"),
            Self::Galaxy => gettext(locale, "Galaxy"),
            Self::Matrix => gettext(locale, "Matrix rain"),
            Self::Spectrogram => gettext(locale, "Spectrogram"),
            Self::Orb => gettext(locale, "Neon orb"),
            Self::Bubbles => gettext(locale, "Bubbles"),
            Self::Skyline => gettext(locale, "City skyline"),
            Self::Ocean => gettext(locale, "Ocean waves"),
            Self::Led => gettext(locale, "Retro LED meter"),
            Self::Fireflies => gettext(locale, "Fireflies"),
            Self::Heartbeat => gettext(locale, "Heart monitor"),
            Self::Circuit => gettext(locale, "Circuit board"),
        }
    }

    /// The next mode when empty player bar space is clicked.
    pub fn next(self) -> Self {
        let place = Self::ALL.iter().position(|mode| *mode == self).unwrap_or(0);
        Self::ALL[(place + 1) % Self::ALL.len()]
    }

    /// The next mode a click moves to: through the favourites, in their
    /// list order, once there are two or more; otherwise through them all.
    pub fn next_among(self, favourites: &[Self]) -> Self {
        let favourites: Vec<Self> = Self::ALL
            .into_iter()
            .filter(|mode| *mode != Self::Off && favourites.contains(mode))
            .collect();
        if favourites.len() < 2 {
            return self.next();
        }
        match favourites.iter().position(|mode| *mode == self) {
            Some(place) => favourites[(place + 1) % favourites.len()],
            None => favourites[0],
        }
    }

    /// The modes visualizer shuffle picks among other than `showing`: the
    /// favourites when there are at least two others, otherwise every mode.
    pub fn shuffle_pool(showing: Self, favourites: &[Self]) -> Vec<Self> {
        let others = |all: bool| -> Vec<Self> {
            Self::ALL
                .into_iter()
                .filter(|mode| {
                    *mode != Self::Off && *mode != showing && (all || favourites.contains(mode))
                })
                .collect()
        };
        let chosen = others(false);
        if chosen.len() >= 2 {
            chosen
        } else {
            others(true)
        }
    }
}

/// How strongly the beat pulse shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PulseStrength {
    /// A soft glow along the top of the player bar.
    #[default]
    Subtle,
    Medium,
    /// As bright as v16's pulse.
    Strong,
}

impl PulseStrength {
    pub const ALL: [Self; 3] = [Self::Subtle, Self::Medium, Self::Strong];

    pub fn label(self, locale: crate::i18n::Locale) -> std::borrow::Cow<'static, str> {
        use crate::i18n::gettext;
        match self {
            Self::Subtle => gettext(locale, "Subtle"),
            Self::Medium => gettext(locale, "Medium"),
            Self::Strong => gettext(locale, "Strong"),
        }
    }

    /// How bright the pulse is drawn, from 0 to 1.
    pub fn amount(self) -> f32 {
        match self {
            Self::Subtle => 0.3,
            Self::Medium => 0.6,
            Self::Strong => 1.0,
        }
    }
}

/// How fast a living gradient drifts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GradientDrift {
    #[default]
    Slow,
    Medium,
    Fast,
}

impl GradientDrift {
    pub const ALL: [Self; 3] = [Self::Slow, Self::Medium, Self::Fast];

    pub fn label(self, locale: crate::i18n::Locale) -> std::borrow::Cow<'static, str> {
        use crate::i18n::gettext;
        match self {
            Self::Slow => gettext(locale, "Slow"),
            Self::Medium => gettext(locale, "Medium"),
            Self::Fast => gettext(locale, "Fast"),
        }
    }

    /// One full drift's share each second.
    pub fn speed(self) -> f32 {
        match self {
            Self::Slow => 1.0 / 60.0,
            Self::Medium => 1.0 / 30.0,
            Self::Fast => 1.0 / 15.0,
        }
    }
}

/// What a click on a page link, playlist, card or song shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClickEffect {
    Off,
    /// A soft ring of light spreading out from the pointer.
    #[default]
    Ripple,
    /// The clicked item glows and fades.
    Glow,
    /// An outline springs out from the item and fades.
    Pop,
    /// A few sparks float out from the pointer.
    Sparkle,
    /// Rings of light ripple outwards from the pointer, one after another.
    Rings,
    /// A soft glow blooms from the pointer like a flower opening.
    Bloom,
    /// A little burst of confetti.
    Confetti,
    /// A wave of light washes across the item, like the sea on a beach.
    Wave,
}

impl ClickEffect {
    pub const ALL: [Self; 9] = [
        Self::Off,
        Self::Ripple,
        Self::Glow,
        Self::Pop,
        Self::Sparkle,
        Self::Rings,
        Self::Bloom,
        Self::Confetti,
        Self::Wave,
    ];

    pub fn label(self, locale: crate::i18n::Locale) -> std::borrow::Cow<'static, str> {
        use crate::i18n::gettext;
        match self {
            Self::Off => gettext(locale, "Off"),
            Self::Ripple => gettext(locale, "Ripple"),
            Self::Glow => gettext(locale, "Glow"),
            Self::Pop => gettext(locale, "Pop"),
            Self::Sparkle => gettext(locale, "Sparkle"),
            Self::Rings => gettext(locale, "Rings"),
            Self::Bloom => gettext(locale, "Bloom"),
            Self::Confetti => gettext(locale, "Confetti"),
            Self::Wave => gettext(locale, "Wave"),
        }
    }
}

/// The corners of cards, covers and highlighted rows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BoxShape {
    /// The corners Motion Music has always had.
    #[default]
    Rounded,
    Square,
    Soft,
    Bubbly,
    /// As round as each box allows: rows become pills and small covers
    /// circles.
    Pill,
    /// Two opposite corners round and two nearly square, like a leaf.
    Leaf,
}

impl BoxShape {
    pub const ALL: [Self; 6] = [
        Self::Rounded,
        Self::Square,
        Self::Soft,
        Self::Bubbly,
        Self::Pill,
        Self::Leaf,
    ];

    pub fn label(self, locale: crate::i18n::Locale) -> std::borrow::Cow<'static, str> {
        use crate::i18n::gettext;
        match self {
            Self::Rounded => gettext(locale, "Rounded"),
            Self::Square => gettext(locale, "Square"),
            Self::Soft => gettext(locale, "Soft"),
            Self::Bubbly => gettext(locale, "Bubbly"),
            Self::Pill => gettext(locale, "Pill"),
            Self::Leaf => gettext(locale, "Leaf"),
        }
    }

    /// The size of a corner drawn `base` points round in the usual shape,
    /// in this one. Leaf's round corners are this size.
    pub fn radius(self, base: f32) -> f32 {
        match self {
            Self::Rounded => base,
            Self::Square => 0.0,
            Self::Soft => base * 2.0,
            Self::Bubbly => base * 3.5,
            // egui keeps every corner to half the box's shorter side.
            Self::Pill => 127.0,
            Self::Leaf => base * 2.5,
        }
    }

    /// The four corners, top left, top right, bottom left and bottom right,
    /// of a box whose corners are `radius` (from [`Self::radius`]) round.
    pub fn corners(self, radius: f32) -> [f32; 4] {
        match self {
            Self::Leaf => [radius, radius * 0.15, radius * 0.15, radius],
            _ => [radius; 4],
        }
    }
}

/// When the player bar visualizer changes to another by itself.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VisShuffle {
    /// It stays on the one chosen.
    #[default]
    Off,
    /// A different one for every song.
    Song,
    /// A different one every minute.
    Minute,
}

impl VisShuffle {
    pub const ALL: [Self; 3] = [Self::Off, Self::Song, Self::Minute];

    pub fn label(self, locale: crate::i18n::Locale) -> std::borrow::Cow<'static, str> {
        use crate::i18n::gettext;
        match self {
            Self::Off => gettext(locale, "Off"),
            Self::Song => gettext(locale, "Each song"),
            Self::Minute => gettext(locale, "Every minute"),
        }
    }
}
/// A setting with a few named choices: saved by its lower-case name, with
/// every choice in order and a label for each.
macro_rules! choices {
    ($(#[$meta:meta])* $name:ident default $default:ident { $($(#[$doc:meta])* $variant:ident => $label:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(rename_all = "lowercase")]
        pub enum $name {
            $($(#[$doc])* $variant),+
        }

        impl Default for $name {
            fn default() -> Self {
                Self::$default
            }
        }

        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            pub fn label(self, locale: crate::i18n::Locale) -> std::borrow::Cow<'static, str> {
                match self {
                    $(Self::$variant => crate::i18n::gettext(locale, $label)),+
                }
            }
        }
    };
}

choices! {
    /// How the interface's surfaces are drawn.
    InterfaceStyle default Solid {
        /// The theme's colours, as always.
        Solid => "Solid",
        /// See-through frosted glass over a blurred backdrop, like iOS.
        Glass => "Glass",
    }
}

choices! {
    /// How frosted the glass is: how much of the backdrop shows through.
    GlassFrost default Medium {
        Light => "Light",
        Medium => "Medium",
        Heavy => "Heavy",
    }
}

impl GlassFrost {
    /// How opaque the pages are, and the panels around them.
    pub fn opacity(self) -> (f32, f32) {
        match self {
            Self::Light => (0.5, 0.6),
            Self::Medium => (0.66, 0.74),
            Self::Heavy => (0.8, 0.86),
        }
    }
}

choices! {
    /// What shows through the glass.
    GlassBackdrop default AlbumArt {
        /// The playing song's cover, blurred; the theme's colours without one.
        AlbumArt => "Album art",
        /// Soft clouds of the theme's colours.
        Theme => "Theme colours",
    }
}

choices! {
    /// How the seek bar under the player's buttons is drawn.
    SeekStyle default Classic {
        Classic => "Classic",
        /// Twice as thick, with rounded ends.
        Thick => "Thick",
        /// A glowing line in the visualizer's colours.
        Glow => "Glow",
        /// The played part ripples like a wave while music plays.
        Wave => "Wave",
        /// A row of dots that light up as the song goes.
        Dots => "Dots",
        /// A hairline.
        Line => "Line",
        /// The played part as a gradient of the visualizer's colours.
        Gradient => "Gradient",
        /// Small blocks, like a progress meter.
        Segments => "Segments",
        /// A bright line in the accent with a strong glow.
        Neon => "Neon",
        /// A tall rounded capsule with its knob always showing.
        Capsule => "Capsule",
    }
}

choices! {
    /// The play button's shape.
    PlayButtonStyle default Circle {
        Circle => "Circle",
        /// A rounded square.
        Rounded => "Rounded",
        /// An outline ring with the symbol inside.
        Ring => "Ring",
        /// A circle in the visualizer's colours.
        Gradient => "Gradient",
    }
}

choices! {
    /// How strongly the player bar's visualizer shows.
    VisIntensity default Full {
        Faint => "Faint",
        Soft => "Soft",
        Full => "Full",
    }
}

impl VisIntensity {
    pub fn opacity(self) -> f32 {
        match self {
            Self::Faint => 0.35,
            Self::Soft => 0.65,
            Self::Full => 1.0,
        }
    }
}

choices! {
    /// A trail that follows the pointer.
    CursorTrail default Off {
        Off => "Off",
        /// Sparks in the visualizer's colours that drift and fade.
        Sparkle => "Sparkle",
        /// A smooth glowing tail.
        Comet => "Comet",
    }
}

choices! {
    /// A faint pattern on the page behind everything.
    BackgroundPattern default None {
        None => "None",
        Dots => "Dots",
        Grid => "Grid",
        Diagonal => "Diagonal",
        Waves => "Waves",
        Stars => "Stars",
    }
}

choices! {
    /// How a song row shows the pointer over it.
    RowHover default Fill {
        /// A soft fill, as always.
        Fill => "Fill",
        /// An outline in the accent colour.
        Outline => "Outline",
        /// A bar of the accent at the row's left edge.
        Bar => "Accent bar",
        /// A soft glow of the accent.
        Glow => "Glow",
    }
}

/// The accent colours offered in Settings > Customise, besides the theme's.
pub const ACCENTS: [[u8; 3]; 14] = [
    [239, 68, 68],
    [249, 115, 22],
    [245, 158, 11],
    [234, 179, 8],
    [132, 204, 22],
    [34, 197, 94],
    [20, 184, 166],
    [6, 182, 212],
    [59, 130, 246],
    [99, 102, 241],
    [139, 92, 246],
    [217, 70, 239],
    [236, 72, 153],
    [244, 244, 245],
];

/// How the logo reveal looks when the app opens.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StartupStyle {
    /// The logo grows in as rings spread out from it.
    #[default]
    Bloom,
    /// A circle opens from the middle onto the app.
    Iris,
    /// The logo spins into place.
    Spin,
    /// A curtain parts down the middle.
    Curtain,
    /// The logo fades up in a soft glow.
    Glow,
    /// A band of the theme's colours sweeps across.
    Sweep,
    /// The logo beats twice like a heart.
    Pulse,
    /// The logo flies towards you and past.
    Zoom,
    /// Tiles tip away from the middle outward.
    Mosaic,
    /// A different one each launch.
    Random,
}

impl StartupStyle {
    pub const ALL: [Self; 10] = [
        Self::Bloom,
        Self::Iris,
        Self::Spin,
        Self::Curtain,
        Self::Glow,
        Self::Sweep,
        Self::Pulse,
        Self::Zoom,
        Self::Mosaic,
        Self::Random,
    ];
    /// The styles Random picks among.
    pub const PLAYED: [Self; 9] = [
        Self::Bloom,
        Self::Iris,
        Self::Spin,
        Self::Curtain,
        Self::Glow,
        Self::Sweep,
        Self::Pulse,
        Self::Zoom,
        Self::Mosaic,
    ];

    pub fn label(self, locale: crate::i18n::Locale) -> std::borrow::Cow<'static, str> {
        use crate::i18n::gettext;
        match self {
            Self::Bloom => gettext(locale, "Bloom"),
            Self::Iris => gettext(locale, "Iris"),
            Self::Spin => gettext(locale, "Spin"),
            Self::Curtain => gettext(locale, "Curtain"),
            Self::Glow => gettext(locale, "Glow"),
            Self::Sweep => gettext(locale, "Sweep"),
            Self::Pulse => gettext(locale, "Pulse"),
            Self::Zoom => gettext(locale, "Zoom"),
            Self::Mosaic => gettext(locale, "Mosaic"),
            Self::Random => gettext(locale, "Random"),
        }
    }
}

/// The lengths the sleep timer offers, in minutes.
pub const SLEEP_TIMER_LENGTHS: [u32; 6] = [15, 30, 45, 60, 90, 120];

/// What the visualizers listen to on Windows while the music plays in
/// another app on this PC, such as Soloist, rather than in Motion Music.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VisAudio {
    /// Only the Spotify player app's own sound (Soloist, else the Spotify
    /// app); everything on the PC while neither is open.
    #[default]
    Player,
    /// Everything audible on the PC, as before this setting existed.
    Everything,
}

impl VisAudio {
    pub const ALL: [Self; 2] = [Self::Player, Self::Everything];

    pub fn label(self, locale: crate::i18n::Locale) -> std::borrow::Cow<'static, str> {
        use crate::i18n::gettext;
        match self {
            Self::Player => gettext(locale, "Spotify player only"),
            Self::Everything => gettext(locale, "All PC audio"),
        }
    }
}

/// Where the player bar visualizer takes its colours from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VisColours {
    /// The playing cover while colour from album art is on, as before this
    /// setting existed; otherwise the theme.
    #[default]
    Auto,
    /// Always the theme: a gradient theme's three colours, or the accent.
    Theme,
    /// A colour scheme of its own, whatever the theme or cover.
    Rainbow,
    Fire,
    Ice,
    Neon,
    Sunset,
    Forest,
}

impl VisColours {
    pub const ALL: [Self; 8] = [
        Self::Auto,
        Self::Theme,
        Self::Rainbow,
        Self::Fire,
        Self::Ice,
        Self::Neon,
        Self::Sunset,
        Self::Forest,
    ];

    pub fn label(self, locale: crate::i18n::Locale) -> std::borrow::Cow<'static, str> {
        use crate::i18n::gettext;
        match self {
            Self::Auto => gettext(locale, "Album art"),
            Self::Theme => gettext(locale, "Theme"),
            Self::Rainbow => gettext(locale, "Rainbow"),
            Self::Fire => gettext(locale, "Fire"),
            Self::Ice => gettext(locale, "Ice"),
            Self::Neon => gettext(locale, "Neon"),
            Self::Sunset => gettext(locale, "Sunset"),
            Self::Forest => gettext(locale, "Forest"),
        }
    }

    /// A scheme's own colours, from the bass to the treble; `None` for the
    /// cover's and the theme's.
    pub fn scheme(self) -> Option<&'static [[u8; 3]]> {
        Some(match self {
            Self::Auto | Self::Theme => return None,
            Self::Rainbow => &[
                [255, 59, 48],
                [255, 149, 0],
                [255, 214, 10],
                [52, 199, 89],
                [0, 199, 190],
                [10, 132, 255],
                [191, 90, 242],
            ],
            Self::Fire => &[[220, 30, 10], [255, 100, 0], [255, 170, 20], [255, 220, 80]],
            Self::Ice => &[
                [40, 90, 255],
                [40, 170, 255],
                [60, 220, 255],
                [150, 240, 255],
            ],
            Self::Neon => &[[255, 0, 200], [140, 50, 255], [0, 230, 255], [0, 255, 140]],
            Self::Sunset => &[
                [150, 50, 220],
                [255, 70, 130],
                [255, 130, 60],
                [255, 200, 80],
            ],
            Self::Forest => &[
                [20, 140, 110],
                [40, 190, 90],
                [150, 220, 60],
                [230, 220, 90],
            ],
        })
    }
}

/// How many recently applied Motion themes the gallery keeps.
pub const RECENT_THEMES: usize = 12;

/// The saved theme names this version still carries, once each, in order.
fn known_themes(names: &[String]) -> Vec<crate::theme::MotionTheme> {
    let mut themes = Vec::with_capacity(names.len());
    for name in names {
        if let Some(theme) = crate::theme::MotionTheme::from_filename(name)
            && !themes.contains(&theme)
        {
            themes.push(theme);
        }
    }
    themes
}

/// Mini-player visualizer mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VisMode {
    #[default]
    Bars,
    Scope,
    Off,
}

impl VisMode {
    /// Next mode in the display's click cycle.
    pub fn next(self) -> Self {
        match self {
            Self::Bars => Self::Scope,
            Self::Scope => Self::Off,
            Self::Off => Self::Bars,
        }
    }
}

/// The interface language: the operating system's, or one chosen in Settings.
///
/// Stored as `"system"` or a locale tag such as `"es"` or `"pt-BR"`. A file
/// without the field follows the system; a tag this version does not carry
/// also follows the system, rather than making the whole file unreadable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LanguageChoice {
    #[default]
    System,
    Locale(crate::i18n::Locale),
}

impl LanguageChoice {
    /// The locale the interface is drawn in.
    pub fn resolve(self) -> crate::i18n::Locale {
        match self {
            Self::System => crate::i18n::Locale::from_system(),
            Self::Locale(locale) => locale,
        }
    }
}

impl Serialize for LanguageChoice {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(match self {
            Self::System => "system",
            Self::Locale(locale) => locale.tag(),
        })
    }
}

impl<'de> Deserialize<'de> for LanguageChoice {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(value
            .as_str()
            .and_then(crate::i18n::Locale::from_tag)
            .map_or(Self::System, Self::Locale))
    }
}

impl ThemeChoice {
    pub const ALL: [ThemeChoice; 3] = [Self::System, Self::Light, Self::Dark];

    pub fn label(self, locale: crate::i18n::Locale) -> std::borrow::Cow<'static, str> {
        use crate::i18n::{gettext, pgettext};
        match self {
            Self::Motion(theme) => theme.name().into(),
            Self::Dark => pgettext(locale, "theme", "Dark"),
            Self::Light => pgettext(locale, "theme", "Light"),
            Self::System => gettext(locale, "Follow system"),
        }
    }
}

/// How outbound HTTP traffic reaches the network.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyMode {
    /// No proxy, ignoring environment and OS proxy settings.
    Off,
    /// Environment variables and, on macOS and Windows, the OS proxy.
    #[default]
    System,
    /// A configured HTTP proxy.
    Http,
    /// A configured SOCKS5 proxy.
    Socks,
}

impl ProxyMode {
    pub const ALL: [ProxyMode; 4] = [Self::Off, Self::System, Self::Http, Self::Socks];

    pub fn label(self, locale: crate::i18n::Locale) -> std::borrow::Cow<'static, str> {
        use crate::i18n::pgettext;
        match self {
            Self::Off => pgettext(locale, "proxy", "Off"),
            Self::System => pgettext(locale, "proxy", "System"),
            Self::Http => "HTTP".into(),
            Self::Socks => "SOCKS5".into(),
        }
    }

    pub fn is_manual(self) -> bool {
        matches!(self, Self::Http | Self::Socks)
    }
}

/// Only confirmed proxy preferences are written with other settings. The
/// password stays in memory and the protected store, never in this snapshot.
#[derive(Clone, PartialEq, Eq)]
pub struct ProxyPreferences {
    mode: ProxyMode,
    host: String,
    port: String,
    username: String,
}

impl ProxyPreferences {
    pub fn apply_to(&self, settings: &mut Settings) {
        settings.proxy_mode = self.mode;
        settings.proxy_host.clone_from(&self.host);
        settings.proxy_port.clone_from(&self.port);
        settings.proxy_username.clone_from(&self.username);
    }
}

/// An equalizer preset saved by name: the preamp and the ten bands, so a
/// headphone correction curve keeps the headroom it needs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserEqPreset {
    pub name: String,
    #[serde(default)]
    pub preamp_db: f32,
    pub bands_db: [f32; 10],
}

fn proxy_mode_is_system(mode: &ProxyMode) -> bool {
    *mode == ProxyMode::System
}

/// How often the banner suggesting a personal app has appeared. It shows at
/// most once a day, and never again after the listener closed it
/// [`SlowAppBanner::MAX_DISMISSALS`] times.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SlowAppBanner {
    /// When it last appeared, as an RFC 3339 timestamp.
    pub shown_at: Option<String>,
    /// How many times the listener closed it.
    pub dismissals: u8,
}

impl SlowAppBanner {
    pub const MAX_DISMISSALS: u8 = 3;
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// The Spotify Connect name other devices see.
    pub device_name: String,
    /// The Spotify Connect speaker, by its name, that Motion Music controls
    /// when it starts, in place of this computer. `None` starts on this
    /// computer, as before this setting existed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_speaker: Option<String>,
    /// 96, 160, or 320 kbps.
    pub bitrate: u16,
    pub normalisation: bool,
    pub autoplay: bool,
    pub gapless: bool,
    /// The output local playback uses: the id of a device
    /// `fastframe_audio::output_devices` lists, or `None` for the system
    /// default. Files from 0.13.3 and earlier may hold a device's name instead; see
    /// `crate::sink::Saved`. The `audio_backend` they may also hold is
    /// ignored: fastframe-audio is the only output.
    pub audio_device: Option<String>,
    /// Windows output buffer in milliseconds. Smaller values may click under
    /// load; larger values delay playback controls.
    /// See [`crate::sink::DEFAULT_BUFFER_MS`].
    #[serde(default = "default_buffer_ms")]
    pub audio_buffer_ms: u32,
    pub audio_cache: bool,
    pub audio_cache_mb: u64,
    /// The folder, searched with its subfolders, that a playlist's
    /// `spotify:local:` songs play from; `None` plays none.
    pub local_files_folder: Option<String>,
    pub theme: ThemeChoice,
    /// The interface language; older files without it follow the system.
    pub language: LanguageChoice,
    /// Filename selected from the local themes directory.
    pub custom_theme: Option<String>,
    /// Last accepted appearance, retained if its source file becomes unavailable.
    #[serde(
        default,
        deserialize_with = "fastframe_theme::read_cached_theme",
        skip_serializing_if = "Option::is_none"
    )]
    pub custom_theme_cache: Option<crate::theme::CustomTheme>,
    /// Last detected system palette, so following Omarchy survives a restart.
    #[serde(
        default,
        deserialize_with = "fastframe_theme::read_cached_theme",
        skip_serializing_if = "Option::is_none"
    )]
    pub system_theme_cache: Option<crate::theme::CustomTheme>,
    /// The palettes Follow system shows in light and in dark mode, each with
    /// its last accepted colours; empty slots are Motion Music's own Light and
    /// Dark, as before this setting existed.
    #[serde(skip_serializing_if = "fastframe_theme::Pair::is_default")]
    pub theme_pair: crate::theme::ThemePair,
    pub home: HomeSettings,
    /// Tint the interface with the colour of the playing album's art.
    pub accent_from_art: bool,
    /// An audio-reactive scene of the playing song behind the player bar.
    pub player_bar_vis: PlayerBarVis,
    /// Allow clicks on the player bar background to cycle visualizer modes.
    pub player_bar_vis_click: bool,
    /// Whether the visualizer follows the playing cover or keeps the theme's
    /// colours. Older files without it follow the cover as before.
    pub visualizer_colours: VisColours,
    /// Motion themes starred in the gallery, by their built-in filename, in
    /// the order they were starred. Unknown names are kept but not shown.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub favourite_themes: Vec<String>,
    /// The Motion themes applied most recently, newest first.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub recent_themes: Vec<String>,
    /// Flash the play button and the top of the player bar on the beat.
    pub beat_pulse: bool,
    /// How brightly the beat pulse shows.
    pub beat_pulse_strength: PulseStrength,
    /// Let a gradient theme's colours drift slowly while music plays.
    pub living_gradients: bool,
    /// How fast a living gradient drifts.
    pub gradient_drift: GradientDrift,
    /// What a click on a link, playlist, card or song shows.
    pub click_effect: ClickEffect,
    /// The corners of cards, covers and highlighted rows.
    pub box_shape: BoxShape,
    /// When the player bar visualizer changes to another by itself.
    pub visualiser_shuffle: VisShuffle,
    /// What the visualizers listen to while another app plays the music.
    pub visualiser_audio: VisAudio,
    /// Visualizers marked with a heart: listed first, and with two or more
    /// the only ones a click or shuffle moves between.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub favourite_visualisers: Vec<PlayerBarVis>,
    /// The visualizers and beat pulse fade across a change of speaker.
    pub speaker_fade: bool,
    /// The playing cover drifts with the pointer and breathes with the beat.
    pub moving_art: bool,
    /// The sung line fills with the visualizer's colours as it is sung.
    pub karaoke_lyrics: bool,
    /// Show a recap of the week's listening once each new week starts.
    pub weekly_recap: bool,
    /// The week whose recap was last shown, as `2026-W47`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weekly_recap_seen: Option<String>,
    /// Windows: Ctrl+Alt shortcuts that control playback from any app.
    pub global_shortcuts: bool,
    /// How long the sleep timer runs when it is turned on in Settings.
    pub sleep_timer_minutes: u32,
    /// A short logo reveal in the theme's colours when the app opens.
    pub startup_animation: bool,
    /// How the logo reveal looks.
    pub startup_style: StartupStyle,
    /// The sleep timer's clock button shows on the player bar.
    pub sleep_timer_button: bool,
    /// Depth as pages move: a blurred cover behind album and playlist
    /// headers, and soft clouds of colour behind Home.
    pub parallax: bool,
    /// A band of light glides across a cover as the pointer moves over it.
    pub cover_sheen: bool,
    /// Solid colours, or frosted glass over a blurred backdrop.
    pub interface_style: InterfaceStyle,
    pub glass_frost: GlassFrost,
    pub glass_backdrop: GlassBackdrop,
    /// An accent colour of the listener's own in place of the theme's.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_accent: Option<[u8; 3]>,
    pub seek_style: SeekStyle,
    pub play_button: PlayButtonStyle,
    pub vis_intensity: VisIntensity,
    pub cursor_trail: CursorTrail,
    pub background_pattern: BackgroundPattern,
    /// The background pattern drifts slowly.
    pub pattern_motion: bool,
    pub row_hover: RowHover,
    /// A name Home's greeting adds, as in "Good evening, Louis".
    #[serde(skip_serializing_if = "String::is_empty")]
    pub greeting_name: String,
    /// The player bar shows the time left instead of the song's length.
    pub time_remaining: bool,
    /// Page reveal styles left out of the shuffle, by their saved name.
    /// Empty uses every style, as before this setting existed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub page_reveals_off: Vec<String>,
    /// How many frames a second the player bar's visualizer and the mini
    /// player's analyser draw; 0 is uncapped. MilkDrop has its own.
    pub visualizer_fps: u32,
    /// Opt-in publishing of catalog listening metadata to the desktop Discord client.
    pub discord_presence: bool,
    /// Public Discord Application ID. Never a bot token or client secret.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub discord_application_id: String,
    /// Last local volume, 0..=65535.
    pub volume: u16,
    /// Whether the library sidebar is visible.
    pub sidebar_visible: bool,
    /// The playing album's art docked large at the sidebar's bottom.
    pub art_expanded: bool,
    /// Use compact single-line rows without cover art in the sidebar.
    pub sidebar_compact: bool,
    /// Show the Library as responsive cover cards instead of rows.
    pub sidebar_grid: bool,
    pub sidebar_width: f32,
    pub lyrics_width: f32,
    pub queue_width: f32,
    /// Use compact single-line rows without cover art in track lists.
    pub tracklist_compact: bool,
    /// Linux: middle-click a list to autoscroll it. Off by default, because
    /// Linux desktops usually paste the primary selection on middle click.
    /// Windows always autoscrolls and macOS never does.
    pub middle_click_autoscroll: bool,
    pub search_history: Vec<String>,
    pub show_shortcut_hints: bool,
    /// An optional personal Spotify Web API application id. The shared
    /// application remains active for coverage when this is present.
    pub web_client_id: Option<String>,
    /// Legacy reminder time, retained for older Motion Music versions.
    pub personal_app_nudge_at: Option<String>,
    /// The listener has dismissed or followed the personal-app introduction.
    pub personal_app_intro_seen: bool,
    /// When the "Spotify is being slow" banner last appeared and how often
    /// the listener closed it. Kept by the app, not chosen in Settings.
    pub slow_app_banner: SlowAppBanner,
    /// Local playback has been authorized at least once on this machine, so
    /// the app can resume it silently instead of prompting.
    pub playback_authorized: bool,
    /// Closing the window hides to the tray and keeps the music playing.
    pub keep_playing_in_background: bool,
    /// Linux and Windows: the tray icon in colour, white or black (#128).
    pub tray_icon: TrayIcon,
    /// Show the interactive Now Playing widget when hovering over the MacBook notch.
    pub mac_notch_widget: bool,
    /// macOS: keep Motion Music in the Dock and Cmd-Tab. Off makes it a
    /// menu-bar app, whose menu-bar icon is the way back (#837).
    pub mac_show_in_dock: bool,
    /// Ask GitHub once a day whether a newer release exists.
    pub check_for_updates: bool,
    pub download_updates_automatically: bool,
    /// Context URIs and the local Liked Songs key, in pin order.
    pub pinned_contexts: Vec<String>,
    /// Older settings keep Liked Songs first until it is moved or unpinned.
    pub liked_songs_pinned: bool,
    /// The sidebar's own playlist order, set by dragging rows. Kept while
    /// another sort is selected; empty means no saved local arrangement.
    pub sidebar_order: Vec<String>,
    /// Explicit order per Library shelf. Missing shelves keep their previous
    /// behaviour; selecting another order never deletes the local arrangement.
    pub library_sort: std::collections::BTreeMap<LibraryShelf, LibrarySort>,
    /// Show only playlists owned by the signed-in account in the playlist shelf.
    pub library_by_you: bool,
    /// Interface zoom, egui's zoom factor; Ctrl+plus/minus changes it.
    pub zoom: f32,
    /// The Winamp window is open.
    pub winamp_window: bool,
    /// Windows and X11: keep a taskbar button while the Winamp window is visible.
    pub winamp_show_taskbar: bool,
    /// Windows and Linux: draw Motion Music's own title bar and window buttons
    /// instead of the desktop frame.
    pub custom_titlebar: bool,
    /// Skin file or folder name. `None` selects the built-in skin.
    pub skin: Option<String>,
    /// Pick a different skin each time the mini player opens; `skin` holds
    /// the one picked.
    pub random_skin: bool,
    /// Screen pixels per skin pixel; `None` picks double size for the
    /// display.
    pub skin_scale: Option<u8>,
    /// The Winamp window stays above other windows.
    pub winamp_on_top: bool,
    /// The mini player's visualiser: bars, scope, or off.
    pub vis: VisMode,
    /// The playlist window is open under the mini player.
    pub playlist_open: bool,
    /// How tall the playlist window is, in skin pixels.
    pub playlist_height: u32,
    /// The equalizer window is open under the mini player.
    pub eq_open: bool,
    /// The equalizer shapes local playback.
    pub eq_on: bool,
    /// The preamp, in decibels, never above zero.
    pub eq_preamp_db: f32,
    /// The ten bands, in decibels, 60 Hz to 16 kHz.
    pub eq_bands_db: [f32; 10],
    /// Equalizer presets saved here, listed after the built-in ones (#702).
    pub eq_user_presets: Vec<UserEqPreset>,
    /// The balance, -1 all left to 1 all right.
    pub balance: f32,
    /// Play both channels the same.
    pub mono: bool,
    /// The playlist window is rolled up to its title bar.
    pub playlist_shaded: bool,
    /// The equalizer window is rolled up to its title bar.
    pub eq_shaded: bool,
    /// The main window is rolled up to its title bar.
    pub winamp_shaded: bool,
    /// The MilkDrop window is open (its own window, not part of the skin).
    pub milkdrop_open: bool,
    /// How long each preset plays before the next, in seconds.
    pub milkdrop_seconds: u32,
    /// How many frames a second the MilkDrop window draws; 0 is uncapped.
    pub milkdrop_fps: u32,
    /// Last reported MilkDrop screen refresh rate. The first value sets the
    /// default frame rate; this field is not directly configurable.
    pub milkdrop_screen_hz: u32,
    /// The picture's inner resolution: 1 full, 2 half, 4 quarter.
    pub milkdrop_scale: u32,
    /// The MilkDrop window fills the screen.
    pub milkdrop_fullscreen: bool,
    /// The MilkDrop window's size in logical points, when not full-screen.
    pub milkdrop_size: [f32; 2],
    /// Which proxy to use. Older files without this field stay on `system`.
    #[serde(default, skip_serializing_if = "proxy_mode_is_system")]
    pub proxy_mode: ProxyMode,
    /// Combined address from older settings files. Split into host and port
    /// on load; not written again.
    #[serde(default, skip_serializing)]
    pub proxy: String,
    /// Host of a manual HTTP or SOCKS5 proxy. Ignored when the mode is Off
    /// or System.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub proxy_host: String,
    /// Port of a manual HTTP or SOCKS5 proxy.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub proxy_port: String,
    /// Optional proxy login for Web requests.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub proxy_username: String,
    /// Optional proxy password, in memory and the platform credential store.
    /// Legacy JSON copies remain until protected migration succeeds.
    #[serde(default, skip_serializing)]
    pub proxy_password: String,
    /// Legacy settings and their password must survive until protected migration
    /// succeeds. This flag is only in memory and is never persisted.
    #[serde(skip)]
    pub proxy_password_legacy: bool,
}

impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Serialization already omits the password and old combined URL.
        // Keep the proxy username out of diagnostics as well.
        let mut preferences = serde_json::to_value(self).map_err(|_| std::fmt::Error)?;
        if let Some(fields) = preferences.as_object_mut() {
            fields.remove("proxy_username");
        }
        f.debug_struct("Settings")
            .field("preferences", &preferences)
            .finish()
    }
}

impl Default for Settings {
    /// Supply the initial preferences and defaults for fields absent from older
    /// settings files, including an inactive playlist ownership filter.
    fn default() -> Self {
        Self {
            device_name: "Motion Music".to_string(),
            default_speaker: None,
            bitrate: 320,
            normalisation: false,
            autoplay: true,
            gapless: true,
            audio_device: None,
            local_files_folder: None,
            audio_buffer_ms: default_buffer_ms(),
            audio_cache: true,
            audio_cache_mb: 1024,
            theme: ThemeChoice::System,
            language: LanguageChoice::System,
            custom_theme: None,
            custom_theme_cache: None,
            system_theme_cache: None,
            theme_pair: crate::theme::ThemePair::default(),
            home: HomeSettings::default(),
            accent_from_art: true,
            player_bar_vis: PlayerBarVis::Off,
            player_bar_vis_click: true,
            visualizer_colours: VisColours::Auto,
            favourite_themes: Vec::new(),
            recent_themes: Vec::new(),
            page_reveals_off: Vec::new(),
            beat_pulse: false,
            beat_pulse_strength: PulseStrength::Subtle,
            living_gradients: false,
            gradient_drift: GradientDrift::Slow,
            click_effect: ClickEffect::Ripple,
            box_shape: BoxShape::Rounded,
            visualiser_shuffle: VisShuffle::Off,
            visualiser_audio: VisAudio::Player,
            favourite_visualisers: Vec::new(),
            speaker_fade: true,
            moving_art: true,
            karaoke_lyrics: true,
            weekly_recap: true,
            weekly_recap_seen: None,
            global_shortcuts: true,
            sleep_timer_minutes: 30,
            startup_animation: true,
            startup_style: StartupStyle::Bloom,
            sleep_timer_button: true,
            parallax: true,
            cover_sheen: true,
            interface_style: InterfaceStyle::Solid,
            glass_frost: GlassFrost::Medium,
            glass_backdrop: GlassBackdrop::AlbumArt,
            custom_accent: None,
            seek_style: SeekStyle::Classic,
            play_button: PlayButtonStyle::Circle,
            vis_intensity: VisIntensity::Full,
            cursor_trail: CursorTrail::Off,
            background_pattern: BackgroundPattern::None,
            pattern_motion: true,
            row_hover: RowHover::Fill,
            greeting_name: String::new(),
            time_remaining: false,
            visualizer_fps: crate::milkdrop::DEFAULT_FPS,
            discord_presence: false,
            discord_application_id: crate::discord_activity::DEFAULT_APPLICATION_ID.to_owned(),
            volume: (u16::MAX as u32 * 70 / 100) as u16,
            sidebar_visible: true,
            art_expanded: false,
            sidebar_compact: false,
            sidebar_grid: false,
            sidebar_width: 250.0,
            lyrics_width: 360.0,
            queue_width: 360.0,
            tracklist_compact: false,
            middle_click_autoscroll: false,
            search_history: Vec::new(),
            show_shortcut_hints: true,
            web_client_id: None,
            personal_app_nudge_at: None,
            personal_app_intro_seen: false,
            slow_app_banner: SlowAppBanner::default(),
            playback_authorized: false,
            keep_playing_in_background: true,
            tray_icon: TrayIcon::Colour,
            mac_notch_widget: false,
            mac_show_in_dock: true,
            check_for_updates: false,
            download_updates_automatically: false,
            pinned_contexts: Vec::new(),
            liked_songs_pinned: true,
            sidebar_order: Vec::new(),
            library_sort: std::collections::BTreeMap::new(),
            library_by_you: false,
            zoom: 1.0,
            winamp_window: false,
            winamp_show_taskbar: true,
            custom_titlebar: false,
            skin: None,
            random_skin: false,
            skin_scale: None,
            winamp_on_top: false,
            vis: VisMode::default(),
            playlist_open: false,
            playlist_height: 174,
            eq_open: false,
            eq_on: false,
            eq_preamp_db: 0.0,
            eq_bands_db: [0.0; 10],
            eq_user_presets: Vec::new(),
            balance: 0.0,
            mono: false,
            playlist_shaded: false,
            eq_shaded: false,
            winamp_shaded: false,
            milkdrop_open: false,
            milkdrop_seconds: crate::milkdrop::DEFAULT_SECONDS,
            milkdrop_fps: crate::milkdrop::DEFAULT_FPS,
            milkdrop_screen_hz: 0,
            milkdrop_scale: 1,
            milkdrop_fullscreen: false,
            milkdrop_size: crate::milkdrop::DEFAULT_SIZE,
            proxy_mode: ProxyMode::System,
            proxy: String::new(),
            proxy_host: String::new(),
            proxy_port: String::new(),
            proxy_username: String::new(),
            proxy_password: String::new(),
            proxy_password_legacy: false,
        }
    }
}

fn default_buffer_ms() -> u32 {
    crate::sink::DEFAULT_BUFFER_MS
}

impl Settings {
    /// The starred Motion themes this version carries, in starred order.
    pub fn favourite_theme_list(&self) -> Vec<crate::theme::MotionTheme> {
        known_themes(&self.favourite_themes)
    }

    /// The recently applied Motion themes this version carries, newest first.
    pub fn recent_theme_list(&self) -> Vec<crate::theme::MotionTheme> {
        known_themes(&self.recent_themes)
    }

    pub fn is_favourite_theme(&self, theme: crate::theme::MotionTheme) -> bool {
        self.favourite_themes
            .iter()
            .any(|name| name == theme.filename())
    }

    /// Star or unstar a Motion theme; answers whether it is now starred.
    pub fn toggle_favourite_theme(&mut self, theme: crate::theme::MotionTheme) -> bool {
        let before = self.favourite_themes.len();
        self.favourite_themes
            .retain(|name| name != theme.filename());
        if self.favourite_themes.len() == before {
            self.favourite_themes.push(theme.filename().to_owned());
            true
        } else {
            false
        }
    }

    /// Move a just-applied Motion theme to the front of the recent list.
    pub fn note_recent_theme(&mut self, theme: crate::theme::MotionTheme) {
        if self.recent_themes.first().map(String::as_str) == Some(theme.filename()) {
            return;
        }
        self.recent_themes.retain(|name| name != theme.filename());
        self.recent_themes.insert(0, theme.filename().to_owned());
        self.recent_themes.truncate(RECENT_THEMES);
    }

    /// The speaker to start on, when one is set to something other than
    /// blank space.
    pub fn default_speaker(&self) -> Option<&str> {
        self.default_speaker
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
    }

    /// The local files folder, when one is set.
    pub fn local_files_folder(&self) -> Option<std::path::PathBuf> {
        self.local_files_folder
            .as_deref()
            .filter(|folder| !folder.trim().is_empty())
            .map(std::path::PathBuf::from)
    }

    /// The one palette shown whatever the system's appearance: a chosen
    /// palette file, or Omarchy's while Follow system follows it.
    pub(crate) fn fixed_palette(&self) -> Option<crate::theme::Palette> {
        let theme = if self.custom_theme.is_some() {
            self.custom_theme_cache.as_ref()
        } else if self.theme == ThemeChoice::System {
            self.system_theme_cache.as_ref()
        } else {
            None
        };
        theme
            .map(|theme| theme.palette)
            .or_else(|| match self.theme {
                ThemeChoice::Motion(theme) => Some(theme.palette()),
                _ => None,
            })
    }

    /// The palette to show while the system is `dark` or light: the fixed
    /// one, else Follow system's choice for that appearance. `None` leaves
    /// Motion Music's own Light or Dark.
    pub(crate) fn cached_palette(&self, dark: bool) -> Option<crate::theme::Palette> {
        if let Some(palette) = self.fixed_palette() {
            return Some(palette);
        }
        let base = fastframe_theme::Base::from_dark(dark);
        if self.theme != ThemeChoice::System {
            return None;
        }
        let slot = self.theme_pair.slot(base)?;
        // Bundled palettes do not require a cached file, and always use
        // the current palette definition after an app update.
        Some(
            crate::theme::MotionTheme::from_filename(&slot.filename)
                .filter(|preset| preset.palette().dark == dark)
                .map_or_else(|| self.theme_pair.palette(base), |preset| preset.palette()),
        )
    }

    pub(crate) fn proxy_password_record(
        &self,
    ) -> Result<Option<crate::credentials::ProxyPassword>, String> {
        if self.proxy_password.is_empty() {
            return Ok(None);
        }
        let manual = ManualProxy::parse(
            ManualKind::Http,
            &self.proxy_host,
            &self.proxy_port,
            &self.proxy_username,
            &self.proxy_password,
        )?;
        Ok(manual.password_record())
    }

    pub(crate) fn restore_proxy_password(
        &mut self,
        saved: &crate::credentials::ProxyPassword,
    ) -> bool {
        let Ok(mut manual) = ManualProxy::parse(
            ManualKind::Http,
            &self.proxy_host,
            &self.proxy_port,
            &self.proxy_username,
            "",
        ) else {
            return false;
        };
        if !manual.restore_password(saved) {
            return false;
        }
        self.proxy_password = manual.password;
        true
    }

    pub fn proxy_preferences(&self) -> ProxyPreferences {
        ProxyPreferences {
            mode: self.proxy_mode,
            host: self.proxy_host.clone(),
            port: self.proxy_port.clone(),
            username: self.proxy_username.clone(),
        }
    }

    pub fn library_pins(&self) -> Vec<String> {
        let mut pins = self.pinned_contexts.clone();
        if !self.liked_songs_pinned {
            pins.retain(|key| key != LIKED_SONGS_KEY);
        } else if !pins.iter().any(|key| key == LIKED_SONGS_KEY) {
            pins.insert(0, LIKED_SONGS_KEY.into());
        }
        pins
    }

    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(text) => {
                // A file edited in Notepad or Windows PowerShell can start with
                // a byte order mark, which JSON does not allow.
                let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
                let mut settings = serde_json::from_str(text).unwrap_or_else(|error| {
                    log::warn!("settings at {} are unreadable: {error}", path.display());
                    Self::default()
                });
                settings.migrate_proxy(text);
                settings.forget_unusable_screen_rate();
                settings.proxy_password_legacy = !settings.proxy_password.is_empty();
                settings
            }
            Err(_) => Self::default(),
        }
    }

    /// The settings as their file holds them.
    pub fn encode(&self) -> Option<String> {
        serde_json::to_string_pretty(self)
            .map_err(|error| log::warn!("unable to encode settings: {error}"))
            .ok()
    }

    /// Writes the settings and waits for the disk. The app saves through
    /// its [`fastframe_file::Writer`] instead.
    pub fn save(&self, path: &Path) {
        let Some(text) = self.encode() else { return };
        if let Err(error) = fastframe_file::write(path, text) {
            log::warn!("unable to save settings to {}: {error}", path.display());
        }
    }

    pub fn remember_search(&mut self, query: &str) {
        let query = query.trim();
        if query.is_empty() {
            return;
        }
        self.search_history.retain(|entry| entry != query);
        self.search_history.insert(0, query.to_string());
        self.search_history.truncate(12);
    }

    /// Forgets the frame rate an earlier version set from Windows's 1 Hz
    /// "hardware default", taken then as MilkDrop's first screen rate, and
    /// that screen rate with it, so MilkDrop's next real report is learned
    /// as the first. A frame rate chosen since, or learned from a real
    /// screen before, stays, and so does the 1 Hz beside it, which no dial
    /// shows and the next real report replaces.
    pub(crate) fn forget_unusable_screen_rate(&mut self) {
        let hz = self.milkdrop_screen_hz;
        if hz == 0 || crate::milkdrop::is_screen_rate(hz) || self.milkdrop_fps != hz {
            return;
        }
        self.milkdrop_fps = crate::milkdrop::DEFAULT_FPS;
        self.milkdrop_screen_hz = 0;
    }

    pub(crate) fn migrate_proxy(&mut self, text: &str) {
        if !text.contains("\"proxy_mode\"") {
            self.proxy_mode = infer_legacy_proxy_mode(&self.proxy);
        }
        if self.proxy_host.is_empty() && !self.proxy.trim().is_empty() {
            let (host, port) = split_legacy_address(&self.proxy);
            self.proxy_host = host;
            if self.proxy_port.is_empty() {
                self.proxy_port = port;
            }
        }
    }

    /// The proxy the HTTP client should use. Off and System always succeed.
    /// HTTP and SOCKS5 need a host and port.
    pub fn proxy_config(&self) -> Result<ProxyConfig, String> {
        match self.proxy_mode {
            ProxyMode::Off => Ok(ProxyConfig::Off),
            ProxyMode::System => Ok(ProxyConfig::System),
            ProxyMode::Http => Ok(ProxyConfig::Http(ManualProxy::parse(
                ManualKind::Http,
                &self.proxy_host,
                &self.proxy_port,
                &self.proxy_username,
                &self.proxy_password,
            )?)),
            ProxyMode::Socks => Ok(ProxyConfig::Socks(ManualProxy::parse(
                ManualKind::Socks,
                &self.proxy_host,
                &self.proxy_port,
                &self.proxy_username,
                &self.proxy_password,
            )?)),
        }
    }
}

fn infer_legacy_proxy_mode(proxy: &str) -> ProxyMode {
    let raw = proxy.trim();
    if raw.is_empty() {
        return ProxyMode::System;
    }
    let scheme = raw.split("://").next().unwrap_or("").to_ascii_lowercase();
    match scheme.as_str() {
        "socks" | "socks5" | "socks5h" => ProxyMode::Socks,
        _ => ProxyMode::Http,
    }
}

fn validate_host(host: &str) -> Result<String, String> {
    let host = host.trim();
    if host.is_empty() {
        return Err("enter a host".into());
    }
    let host = host
        .split_once("://")
        .map_or(host, |(_, rest)| rest)
        .trim_start_matches('[')
        .trim_end_matches(']');
    if host.is_empty() {
        return Err("enter a host".into());
    }
    if host.parse::<std::net::IpAddr>().is_ok() {
        return Ok(host.to_string());
    }
    if is_hostname(host) {
        return Ok(host.to_string());
    }
    Err("the host must be a hostname or IP address".into())
}

fn is_hostname(host: &str) -> bool {
    if host.len() > 253 || host.starts_with('.') || host.ends_with('.') {
        return false;
    }
    host.split('.').all(|label| {
        (1..=63).contains(&label.len())
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    })
}

fn validate_port(port: &str) -> Result<u16, String> {
    let port = port.trim();
    if port.is_empty() {
        return Err("enter a port".into());
    }
    if !port.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("the port must be a number between 1 and 65535".into());
    }
    match port.parse::<u16>() {
        Ok(port) if port > 0 => Ok(port),
        _ => Err("the port must be a number between 1 and 65535".into()),
    }
}

fn split_legacy_address(raw: &str) -> (String, String) {
    let raw = raw.trim();
    let rest = raw.split_once("://").map_or(raw, |(_, rest)| rest);
    let rest = rest.rsplit_once('@').map_or(rest, |(_, rest)| rest);
    if let Some(inner) = rest.strip_prefix('[')
        && let Some((host, port)) = inner.split_once("]:")
    {
        return (host.to_string(), port.trim().to_string());
    }
    if let Some((host, port)) = rest.rsplit_once(':')
        && !host.contains(':')
    {
        return (host.to_string(), port.trim().to_string());
    }
    (rest.to_string(), String::new())
}

/// The resolved proxy policy used by the HTTP client and local playback.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ProxyConfig {
    /// Invalid saved settings block requests until corrected in the interface.
    Invalid(String),
    Off,
    #[default]
    System,
    Http(ManualProxy),
    Socks(ManualProxy),
}

impl ProxyConfig {
    pub(crate) fn password_record(&self) -> Option<crate::credentials::ProxyPassword> {
        match self {
            Self::Http(manual) | Self::Socks(manual) => manual.password_record(),
            _ => None,
        }
    }

    pub(crate) fn restore_password(&mut self, saved: &crate::credentials::ProxyPassword) -> bool {
        match self {
            Self::Http(manual) | Self::Socks(manual) => manual.restore_password(saved),
            _ => false,
        }
    }

    /// Librespot's CONNECT client only understands unauthenticated,
    /// plaintext HTTP proxies. Never give it a credential-bearing URL: the
    /// upstream client logs that URL at info level and does not send proxy
    /// authentication in either of its HTTP paths.
    pub fn librespot_url(&self) -> Option<reqwest::Url> {
        match self {
            Self::Http(manual) => manual.librespot_url(),
            Self::System => system_http_proxy(),
            Self::Invalid(_) | Self::Off | Self::Socks(_) => None,
        }
    }
}

/// The HTTP proxy System mode would hand librespot, if it can resolve one.
/// SOCKS system proxies are ignored: the engine cannot use them.
fn system_http_proxy() -> Option<reqwest::Url> {
    let matcher = hyper_util::client::proxy::matcher::Matcher::from_system();
    let dest = http::Uri::from_static("https://apresolve.spotify.com");
    let intercept = matcher.intercept(&dest)?;
    librespot_system_proxy(&intercept)
}

fn librespot_system_proxy(
    intercept: &hyper_util::client::proxy::matcher::Intercept,
) -> Option<reqwest::Url> {
    if intercept.basic_auth().is_some() || intercept.raw_auth().is_some() {
        return None;
    }
    http_proxy_from_uri(intercept.uri())
}

fn http_proxy_from_uri(uri: &http::Uri) -> Option<reqwest::Url> {
    match uri.scheme_str() {
        Some("http") => reqwest::Url::parse(&uri.to_string()).ok(),
        _ => None,
    }
}

#[derive(Clone, Copy)]
enum ManualKind {
    Http,
    Socks,
}

/// A configured HTTP or SOCKS5 endpoint.
#[derive(Clone, PartialEq, Eq)]
pub struct ManualProxy {
    endpoint: reqwest::Url,
    username: String,
    password: String,
}

impl std::fmt::Debug for ManualProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManualProxy")
            .field("url", &self.redacted())
            .finish()
    }
}

impl ManualProxy {
    fn password_record(&self) -> Option<crate::credentials::ProxyPassword> {
        if self.password.is_empty() {
            return None;
        }
        Some(crate::credentials::ProxyPassword {
            host: self.endpoint.host_str()?.to_string(),
            port: self.endpoint.port()?,
            username: self.username.clone(),
            password: self.password.clone(),
        })
    }

    fn restore_password(&mut self, saved: &crate::credentials::ProxyPassword) -> bool {
        if self.endpoint.host_str() != Some(saved.host.as_str())
            || self.endpoint.port() != Some(saved.port)
            || self.username != saved.username
        {
            return false;
        }
        self.password.clone_from(&saved.password);
        true
    }

    fn parse(
        kind: ManualKind,
        host: &str,
        port: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, String> {
        let host = validate_host(host)?;
        let port = validate_port(port)?;
        let host = if host.contains(':') {
            format!("[{host}]")
        } else {
            host
        };
        let scheme = match kind {
            ManualKind::Http => "http",
            // Resolve Spotify hostnames at the proxy, so SOCKS mode does not
            // leak DNS queries or fail behind a proxy-only network.
            ManualKind::Socks => "socks5h",
        };
        let endpoint = reqwest::Url::parse(&format!("{scheme}://{host}:{port}"))
            .map_err(|error| format!("not a proxy address: {error}"))?;
        Ok(Self {
            endpoint,
            username: username.to_string(),
            password: password.to_string(),
        })
    }

    /// A reqwest proxy with credentials applied separately so build errors
    /// cannot echo a password.
    pub fn reqwest_proxy(&self) -> Result<reqwest::Proxy, reqwest::Error> {
        let mut proxy = reqwest::Proxy::all(self.endpoint.clone())?;
        if !self.username.is_empty() || !self.password.is_empty() {
            proxy = proxy.basic_auth(&self.username, &self.password);
        }
        Ok(proxy)
    }

    fn librespot_url(&self) -> Option<reqwest::Url> {
        (self.username.is_empty() && self.password.is_empty()).then(|| self.endpoint.clone())
    }

    /// The credential-free endpoint, for logs and the debugger.
    pub fn redacted(&self) -> reqwest::Url {
        self.endpoint.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::Settings;

    #[test]
    fn discord_presence_is_opt_in_and_old_settings_remain_readable() {
        let old: super::Settings = serde_json::from_str(r#"{"volume":1000}"#).unwrap();
        assert!(!old.discord_presence);
        assert_eq!(
            old.discord_application_id,
            crate::discord_activity::DEFAULT_APPLICATION_ID
        );
        let mut configured = old;
        configured.discord_presence = true;
        configured.discord_application_id = "123456789012345678".into();
        let restored: super::Settings =
            serde_json::from_str(&serde_json::to_string(&configured).unwrap()).unwrap();
        assert!(restored.discord_presence);
        assert_eq!(
            restored.discord_application_id,
            configured.discord_application_id
        );
        assert_eq!(restored.volume, 1000);
    }

    #[test]
    fn bundled_system_pairs_survive_missing_or_corrupt_caches_without_files() {
        use crate::theme::MotionTheme;
        use fastframe_theme::Base;
        let mut settings = Settings::default();
        assert!(
            settings
                .theme_pair
                .set(Base::Dark, Some(&MotionTheme::Crimson.as_custom_theme()))
        );
        assert!(settings.theme_pair.set(
            Base::Light,
            Some(&MotionTheme::LavenderMilk.as_custom_theme())
        ));
        let mut json = serde_json::to_value(&settings).unwrap();
        json["theme_pair"]["dark"]["palette"] = serde_json::json!({"broken": true});
        json["theme_pair"]["light"]
            .as_object_mut()
            .unwrap()
            .remove("palette");
        let restored: Settings = serde_json::from_value(json).unwrap();
        assert_eq!(
            restored.cached_palette(true),
            Some(MotionTheme::Crimson.palette())
        );
        assert_eq!(
            restored.cached_palette(false),
            Some(MotionTheme::LavenderMilk.palette())
        );
        assert!(
            restored
                .theme_pair
                .slot(Base::Dark)
                .unwrap()
                .palette
                .is_none()
        );
        assert!(
            restored
                .theme_pair
                .slot(Base::Light)
                .unwrap()
                .palette
                .is_none()
        );
    }

    #[test]
    fn bundled_themes_persist_without_files_and_override_system_appearance() {
        use super::ThemeChoice;
        for theme in crate::theme::MotionTheme::ALL {
            let settings = Settings {
                theme: ThemeChoice::Motion(theme),
                volume: 37,
                ..Default::default()
            };
            let json = serde_json::to_string(&settings).unwrap();
            let restored: Settings = serde_json::from_str(&json).unwrap();
            assert_eq!(restored.theme, settings.theme);
            assert_eq!(restored.volume, 37);
            assert_eq!(restored.cached_palette(true), Some(theme.palette()));
            assert_eq!(restored.cached_palette(false), Some(theme.palette()));
        }
    }

    #[test]
    fn unavailable_custom_theme_falls_back_to_motion_and_valid_cache_takes_priority() {
        use super::ThemeChoice;
        use crate::theme::{CustomTheme, MotionTheme, Palette};
        let mut settings = Settings {
            theme: ThemeChoice::Motion(MotionTheme::Crimson),
            custom_theme: Some("missing.json".into()),
            ..Default::default()
        };
        assert_eq!(
            settings.fixed_palette(),
            Some(MotionTheme::Crimson.palette())
        );
        settings.custom_theme_cache = Some(CustomTheme {
            filename: "missing.json".into(),
            palette: Palette::light(),
        });
        assert_eq!(settings.fixed_palette(), Some(Palette::light()));
    }

    #[test]
    fn new_profiles_follow_the_system_and_saved_choices_are_preserved() {
        use super::ThemeChoice;
        assert_eq!(Settings::default().theme, ThemeChoice::System);
        assert_eq!(ThemeChoice::default(), ThemeChoice::System);
        let empty: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(empty.theme, ThemeChoice::System);
        for (json, choice) in [("dark", ThemeChoice::Dark), ("light", ThemeChoice::Light)] {
            let settings: Settings =
                serde_json::from_value(serde_json::json!({"theme": json, "volume": 37})).unwrap();
            assert_eq!(settings.theme, choice);
            assert_eq!(settings.volume, 37);
        }
        let settings: Settings = serde_json::from_value(serde_json::json!({
            "theme": "dark", "system_theme_cache": {"broken": true}, "volume": 37
        }))
        .unwrap();
        assert!(settings.system_theme_cache.is_none());
        assert_eq!(settings.theme, ThemeChoice::Dark);
        assert_eq!(settings.volume, 37);
    }

    /// Files from before fastframe-audio was the only output name a
    /// librespot backend. It is read and ignored, the rest of the file still
    /// loads, and the key is not written back, which older builds read as
    /// their own default.
    #[test]
    fn an_old_audio_backend_is_read_and_ignored() {
        for backend in ["pulseaudio", "rodio", "alsa", "no-such-backend"] {
            let settings: Settings = serde_json::from_value(serde_json::json!({
                "audio_backend": backend,
                "audio_device": "PipeWire Sound Server",
                "volume": 37,
            }))
            .unwrap();
            assert_eq!(settings.volume, 37);
            assert_eq!(
                settings.audio_device.as_deref(),
                Some("PipeWire Sound Server")
            );
            let saved: serde_json::Value =
                serde_json::from_str(&settings.encode().unwrap()).unwrap();
            assert!(saved.get("audio_backend").is_none());
            assert_eq!(saved["audio_device"], "PipeWire Sound Server");
        }
    }

    #[test]
    fn language_follows_the_system_unless_a_known_locale_was_chosen() {
        use super::LanguageChoice;
        use crate::i18n::Locale;
        assert_eq!(Settings::default().language, LanguageChoice::System);
        let older: Settings = serde_json::from_value(serde_json::json!({"volume": 37})).unwrap();
        assert_eq!(older.language, LanguageChoice::System);
        assert_eq!(older.volume, 37);
        for (json, choice) in [
            (serde_json::json!("system"), LanguageChoice::System),
            (
                serde_json::json!("es"),
                LanguageChoice::Locale(Locale::Spanish),
            ),
            (
                serde_json::json!("pt-BR"),
                LanguageChoice::Locale(Locale::PortugueseBrazil),
            ),
            (
                serde_json::json!("zh-Hant"),
                LanguageChoice::Locale(Locale::ChineseTraditional),
            ),
            (
                serde_json::json!("de"),
                LanguageChoice::Locale(Locale::German),
            ),
            // A language a later version added, or a hand-edited typo, keeps
            // the rest of the file and follows the system.
            (serde_json::json!("tlh"), LanguageChoice::System),
            (serde_json::json!(7), LanguageChoice::System),
            (serde_json::Value::Null, LanguageChoice::System),
        ] {
            let settings: Settings =
                serde_json::from_value(serde_json::json!({"language": json, "volume": 37}))
                    .unwrap();
            assert_eq!(settings.language, choice, "{json}");
            assert_eq!(settings.volume, 37);
        }
        for &locale in crate::i18n::LOCALES {
            let settings = Settings {
                language: LanguageChoice::Locale(locale),
                ..Default::default()
            };
            let text = serde_json::to_string(&settings).unwrap();
            assert!(text.contains(&format!("\"language\":\"{}\"", locale.tag())));
            let saved: Settings = serde_json::from_str(&text).unwrap();
            assert_eq!(saved.language, settings.language);
        }
        let text = serde_json::to_string(&Settings::default()).unwrap();
        assert!(text.contains("\"language\":\"system\""));
        assert_eq!(
            LanguageChoice::System.resolve(),
            Locale::English,
            "tests pin English"
        );
    }

    #[test]
    fn queue_width_round_trips_through_saved_settings() {
        let settings = Settings {
            queue_width: 300.0,
            ..Default::default()
        };
        let text = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&text).unwrap();
        assert_eq!(restored.queue_width, 300.0);
    }

    /// Older files keep the Dock icon; hiding it survives a save (#837).
    #[test]
    fn the_dock_icon_stays_unless_hidden_and_round_trips() {
        let empty: Settings = serde_json::from_str("{}").unwrap();
        assert!(empty.mac_show_in_dock);
        let hidden: Settings = serde_json::from_str(r#"{"mac_show_in_dock": false}"#).unwrap();
        assert!(!hidden.mac_show_in_dock);
        let encoded = serde_json::to_string(&hidden).unwrap();
        let decoded: Settings = serde_json::from_str(&encoded).unwrap();
        assert!(!decoded.mac_show_in_dock);
    }

    #[test]
    fn mac_notch_widget_defaults_to_false_when_absent_and_round_trips_true() {
        let empty: Settings = serde_json::from_str("{}").unwrap();
        assert!(!empty.mac_notch_widget);
        let enabled: Settings = serde_json::from_str(r#"{"mac_notch_widget": true}"#).unwrap();
        assert!(enabled.mac_notch_widget);
        let encoded = serde_json::to_string(&enabled).unwrap();
        let decoded: Settings = serde_json::from_str(&encoded).unwrap();
        assert!(decoded.mac_notch_widget);
    }

    #[test]
    fn custom_theme_cache_round_trips_and_a_bad_cache_keeps_other_settings() {
        let mut settings: Settings = serde_json::from_str("{}").unwrap();
        assert!(settings.custom_theme.is_none() && settings.custom_theme_cache.is_none());
        settings.custom_theme = Some("gruvbox.json".into());
        let mut palette = crate::theme::Palette::light();
        palette.shadow = egui::Color32::from_rgba_unmultiplied(37, 128, 249, 117);
        settings.custom_theme_cache = Some(crate::theme::CustomTheme {
            filename: "gruvbox.json".into(),
            palette,
        });
        let encoded = serde_json::to_string(&settings).unwrap();
        assert_eq!(
            serde_json::from_str::<Settings>(&encoded).unwrap(),
            settings
        );
        let mut damaged = serde_json::to_value(&settings).unwrap();
        damaged["custom_theme_cache"] = serde_json::json!({"palette": "broken"});
        damaged["audio_cache_mb"] = 777.into();
        let recovered: Settings = serde_json::from_value(damaged).unwrap();
        assert_eq!(recovered.custom_theme.as_deref(), Some("gruvbox.json"));
        assert_eq!(recovered.audio_cache_mb, 777);
        assert!(recovered.custom_theme_cache.is_none());
    }

    /// Settings from before the light and dark pair keep their look, and
    /// write no pair until one is chosen (#619).
    #[test]
    fn follow_system_pairs_load_old_settings_unchanged_and_round_trip() {
        use super::ThemeChoice;
        use crate::theme::{CustomTheme, Palette};
        use fastframe_theme::Base;
        let mut nord = Palette::dark();
        nord.accent = egui::Color32::from_rgb(0x88, 0xc0, 0xd0);
        let cache = serde_json::to_value(CustomTheme {
            filename: "Nord.json".into(),
            palette: nord,
        })
        .unwrap();
        // One custom palette, with Follow system underneath: still Nord in
        // light and dark mode alike.
        let single: Settings = serde_json::from_value(serde_json::json!({
            "theme": "system", "custom_theme": "Nord.json", "custom_theme_cache": cache,
        }))
        .unwrap();
        assert!(single.theme_pair.is_default());
        assert_eq!(single.cached_palette(false), Some(nord));
        assert_eq!(single.cached_palette(true), Some(nord));
        // Plain Follow system: Motion Music's own Light and Dark.
        let follow: Settings = serde_json::from_str(r#"{"theme":"system"}"#).unwrap();
        assert_eq!(follow.cached_palette(false), None);
        assert_eq!(follow.cached_palette(true), None);
        let encoded = serde_json::to_value(&follow).unwrap();
        assert!(encoded.get("theme_pair").is_none(), "no pair, no field");

        let mut latte = Palette::light();
        latte.window = egui::Color32::from_rgb(0xef, 0xf1, 0xf5);
        let mut paired = follow.clone();
        assert!(paired.theme_pair.set(
            Base::Light,
            Some(&CustomTheme {
                filename: "Catppuccin Latte.json".into(),
                palette: latte,
            })
        ));
        assert_eq!(paired.cached_palette(false), Some(latte));
        assert_eq!(
            paired.cached_palette(true),
            None,
            "dark stays Motion Music's"
        );
        let encoded = serde_json::to_string(&paired).unwrap();
        let decoded: Settings = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, paired);

        // A pair waits behind a fixed choice and comes back with Follow system.
        let mut fixed = paired.clone();
        fixed.theme = ThemeChoice::Dark;
        assert_eq!(fixed.cached_palette(false), None);
        fixed.theme = ThemeChoice::System;
        assert_eq!(fixed.cached_palette(false), Some(latte));

        // A damaged cache keeps the choice and the other settings.
        let mut damaged = serde_json::to_value(&paired).unwrap();
        damaged["theme_pair"]["light"]["palette"] = "broken".into();
        damaged["audio_cache_mb"] = 777.into();
        let recovered: Settings = serde_json::from_value(damaged).unwrap();
        assert_eq!(
            recovered.theme_pair.filename(Base::Light),
            Some("Catppuccin Latte.json")
        );
        assert_eq!(recovered.cached_palette(false), Some(Palette::light()));
        assert_eq!(recovered.audio_cache_mb, 777);
    }

    #[test]
    fn partial_home_preferences_keep_defaults_and_survive_a_settings_round_trip() {
        for (home, made_for_you, recommendations) in [
            ("{}", true, true),
            (r#"{"made_for_you":{"visible":false}}"#, false, true),
            (r#"{"recommendations":{"visible":false}}"#, true, false),
            (
                r#"{"made_for_you":{"visible":false},"recommendations":{"visible":false}}"#,
                false,
                false,
            ),
        ] {
            let settings: Settings =
                serde_json::from_str(&format!(r#"{{"volume":12345,"home":{home}}}"#)).unwrap();
            assert_eq!(settings.home.made_for_you.visible, made_for_you);
            assert_eq!(settings.home.recommendations.visible, recommendations);
            assert_eq!(settings.volume, 12345);
            let restored: Settings =
                serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
            assert_eq!(restored, settings);
        }
        let old: Settings = serde_json::from_str(r#"{"volume":12345}"#).unwrap();
        assert!(old.home.made_for_you.visible);
        assert!(old.home.recommendations.visible);
        assert_eq!(old.volume, 12345);
    }

    /// Notepad and Windows PowerShell can save UTF-8 with a byte order mark.
    /// A file edited that way, as the Home shelves guide asks, is still read
    /// rather than dropped as unreadable and replaced with the defaults.
    #[test]
    fn a_settings_file_saved_with_a_byte_order_mark_keeps_its_preferences() {
        let dir = std::env::temp_dir().join(format!("spotifast-bom-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("settings.json");
        std::fs::write(
            &path,
            "\u{feff}{\"volume\":12345,\"home\":{\"made_for_you\":{\"visible\":false}}}",
        )
        .unwrap();
        let settings = Settings::load(&path);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(settings.volume, 12345);
        assert!(!settings.home.made_for_you.visible);
    }

    /// Earlier versions kept Windows's 1 Hz "hardware default" as MilkDrop's
    /// screen rate and, the first time, as its frame rate. Loading forgets
    /// both, and leaves a frame rate chosen since, and a real rate, alone.
    #[test]
    fn a_saved_hardware_default_screen_rate_is_forgotten() {
        let dir =
            std::env::temp_dir().join(format!("spotifast-screen-default-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("settings.json");
        let load = |text: &str| {
            std::fs::write(&path, text).unwrap();
            let settings = Settings::load(&path);
            (settings.milkdrop_screen_hz, settings.milkdrop_fps)
        };
        let learned = load(r#"{"milkdrop_screen_hz":1,"milkdrop_fps":1}"#);
        let chosen = load(r#"{"milkdrop_screen_hz":1,"milkdrop_fps":30}"#);
        let sixty = load(r#"{"milkdrop_screen_hz":1,"milkdrop_fps":60}"#);
        let real = load(r#"{"milkdrop_screen_hz":144,"milkdrop_fps":144}"#);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(learned, (0, crate::milkdrop::DEFAULT_FPS));
        assert_eq!(chosen, (1, 30), "their rate stands");
        assert_eq!(
            sixty,
            (1, 60),
            "a chosen 60 is not taken for the default, which a real report would replace"
        );
        assert_eq!(real, (144, 144), "a real rate stays");
    }

    #[test]
    fn older_settings_keep_the_sidebar_visible() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert!(settings.sidebar_visible);
    }

    #[test]
    fn older_library_settings_keep_liked_songs_ahead_of_existing_pins() {
        let settings: Settings = serde_json::from_str(
            r#"{"pinned_contexts":["spotify:playlist:one"],"sidebar_order":["spotify:playlist:two"]}"#,
        ).unwrap();
        assert!(settings.liked_songs_pinned);
        assert_eq!(
            settings.library_pins(),
            [super::LIKED_SONGS_KEY, "spotify:playlist:one"]
        );
        assert_eq!(settings.sidebar_order, ["spotify:playlist:two"]);
    }

    /// Saved equalizer presets read back as written; files from before them
    /// have none, and a preset without a preamp keeps it at zero (#702).
    #[test]
    fn saved_equalizer_presets_round_trip_and_default_to_none() {
        let settings: Settings = serde_json::from_str(r#"{"zoom": 1.2}"#).unwrap();
        assert!(settings.eq_user_presets.is_empty());
        let settings: Settings = serde_json::from_str(
            r#"{"eq_user_presets":[{"name":"AirPods","preamp_db":-3.0,"bands_db":[1.2,0,-1.9,-2,0,1.6,1.6,2.6,0,-4.4]},{"name":"Old","bands_db":[0,0,0,0,0,0,0,0,0,1]}]}"#,
        )
        .unwrap();
        assert_eq!(settings.eq_user_presets[0].preamp_db, -3.0);
        assert_eq!(settings.eq_user_presets[1].preamp_db, 0.0);
        let again: Settings =
            serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
        assert_eq!(again.eq_user_presets, settings.eq_user_presets);
    }

    #[test]
    fn older_settings_keep_the_winamp_window_closed_and_the_built_in_skin() {
        let settings: Settings = serde_json::from_str(r#"{"zoom": 1.2}"#).unwrap();
        assert!(!settings.winamp_window);
        assert!(settings.winamp_show_taskbar);
        assert_eq!(settings.skin, None);
        assert_eq!(settings.skin_scale, None);
        assert!(!settings.winamp_on_top);
        assert_eq!(settings.vis, super::VisMode::Bars);
        assert!(!settings.playlist_open);
        assert_eq!(settings.playlist_height, 174);
        assert!(!settings.eq_on);
        assert_eq!(settings.eq_bands_db, [0.0; 10]);
        assert_eq!(settings.balance, 0.0);
        assert!(!settings.mono);
        assert!(!settings.playlist_shaded);
        assert!(!settings.eq_shaded);
        assert!(!settings.winamp_shaded);
    }

    #[test]
    fn the_visualiser_cycles_bars_scope_off() {
        use super::VisMode;
        assert_eq!(VisMode::Bars.next(), VisMode::Scope);
        assert_eq!(VisMode::Scope.next(), VisMode::Off);
        assert_eq!(VisMode::Off.next(), VisMode::Bars);
        let settings: Settings = serde_json::from_str(r#"{"vis": "scope"}"#).unwrap();
        assert_eq!(settings.vis, VisMode::Scope);
    }

    #[test]
    fn a_chosen_skin_round_trips() {
        let settings = Settings {
            winamp_window: true,
            skin: Some("Zaxon.wsz".into()),
            skin_scale: Some(3),
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, settings);
    }

    #[test]
    fn hidden_sidebar_round_trips() {
        let settings = Settings {
            sidebar_visible: false,
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert!(!restored.sidebar_visible);
    }

    #[test]
    fn older_settings_default_to_standard_sidebar() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert!(!settings.sidebar_compact);
    }

    #[test]
    fn compact_sidebar_round_trips() {
        let settings = Settings {
            sidebar_compact: true,
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert!(restored.sidebar_compact);
    }

    #[test]
    fn library_grid_round_trips_and_older_settings_keep_the_list() {
        let old: Settings = serde_json::from_str("{}").unwrap();
        assert!(!old.sidebar_grid);

        let settings = Settings {
            sidebar_grid: true,
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert!(restored.sidebar_grid);
    }

    #[test]
    fn older_settings_default_to_standard_tracklist() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert!(!settings.tracklist_compact);
    }

    #[test]
    fn compact_tracklist_round_trips() {
        let settings = Settings {
            tracklist_compact: true,
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert!(restored.tracklist_compact);
    }

    /// The main window's visualizers draw 60 frames a second until the user
    /// picks another rate, and the rate they pick, uncapped included, is
    /// the rate they get back, without touching MilkDrop's.
    #[test]
    fn the_visualizer_frame_rate_defaults_to_60_and_round_trips() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings.visualizer_fps, 60);
        assert_eq!(Settings::default().visualizer_fps, 60);
        for rate in [30, 144, 0] {
            let chosen = Settings {
                visualizer_fps: rate,
                ..Settings::default()
            };
            let json = serde_json::to_string(&chosen).unwrap();
            let restored: Settings = serde_json::from_str(&json).unwrap();
            assert_eq!(restored.visualizer_fps, rate);
            assert_eq!(restored.milkdrop_fps, Settings::default().milkdrop_fps);
        }
        let milkdrop_only: Settings =
            serde_json::from_value(serde_json::json!({"milkdrop_fps": 30})).unwrap();
        assert_eq!(
            milkdrop_only.visualizer_fps, 60,
            "MilkDrop's rate is its own"
        );
    }

    /// Older files keep today's colour icon, and the choice round-trips as
    /// a lowercase word (#128).
    #[test]
    fn the_tray_icon_stays_in_colour_unless_chosen_and_round_trips() {
        use super::TrayIcon;
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings.tray_icon, TrayIcon::Colour);
        for (choice, word) in [
            (TrayIcon::Colour, "colour"),
            (TrayIcon::White, "white"),
            (TrayIcon::Black, "black"),
        ] {
            let json = serde_json::to_value(Settings {
                tray_icon: choice,
                ..Settings::default()
            })
            .unwrap();
            assert_eq!(json["tray_icon"], word);
            let restored: Settings = serde_json::from_value(json).unwrap();
            assert_eq!(restored.tray_icon, choice);
        }
    }

    #[test]
    fn the_default_speaker_is_optional_and_round_trips() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings.default_speaker, None);
        assert_eq!(settings.default_speaker(), None);
        assert!(
            serde_json::to_value(&settings)
                .unwrap()
                .get("default_speaker")
                .is_none(),
            "not written until chosen"
        );
        let chosen = Settings {
            default_speaker: Some("MotionMusic".into()),
            ..Settings::default()
        };
        let restored: Settings =
            serde_json::from_str(&serde_json::to_string(&chosen).unwrap()).unwrap();
        assert_eq!(restored.default_speaker(), Some("MotionMusic"));
        let blank = Settings {
            default_speaker: Some("   ".into()),
            ..Settings::default()
        };
        assert_eq!(blank.default_speaker(), None);
    }

    #[test]
    fn favourites_recents_and_motion_choices_default_empty_and_round_trip() {
        use super::{
            BoxShape, ClickEffect, GradientDrift, PlayerBarVis, PulseStrength, RECENT_THEMES,
            StartupStyle, VisAudio, VisColours, VisShuffle,
        };
        use crate::theme::MotionTheme;
        let mut settings: Settings = serde_json::from_str("{}").unwrap();
        assert!(settings.favourite_themes.is_empty());
        assert!(settings.recent_themes.is_empty());
        assert!(settings.page_reveals_off.is_empty());
        assert!(!settings.beat_pulse, "the beat pulse starts off");
        assert!(!settings.living_gradients, "gradients start still");
        assert_eq!(settings.beat_pulse_strength, PulseStrength::Subtle);
        assert_eq!(settings.gradient_drift, GradientDrift::Slow);
        assert_eq!(settings.click_effect, ClickEffect::Ripple);
        assert_eq!(settings.box_shape, BoxShape::Rounded);
        assert_eq!(BoxShape::Rounded.radius(6.0), 6.0, "the usual corners");
        assert_eq!(BoxShape::Square.radius(6.0), 0.0);
        assert!(PulseStrength::Subtle.amount() < PulseStrength::Strong.amount());
        let chosen: Settings = serde_json::from_str(
            r#"{"beat_pulse_strength":"strong","gradient_drift":"fast","click_effect":"sparkle","box_shape":"bubbly"}"#,
        )
        .unwrap();
        assert_eq!(chosen.beat_pulse_strength, PulseStrength::Strong);
        assert_eq!(chosen.gradient_drift, GradientDrift::Fast);
        assert_eq!(chosen.click_effect, ClickEffect::Sparkle);
        assert_eq!(chosen.box_shape, BoxShape::Bubbly);
        assert_eq!(settings.visualiser_shuffle, VisShuffle::Off);
        let more: Settings = serde_json::from_str(
            r#"{"click_effect":"confetti","box_shape":"leaf","visualiser_shuffle":"song"}"#,
        )
        .unwrap();
        assert_eq!(more.click_effect, ClickEffect::Confetti);
        assert_eq!(more.box_shape, BoxShape::Leaf);
        assert_eq!(more.visualiser_shuffle, VisShuffle::Song);
        let json = serde_json::to_value(&more).unwrap();
        assert_eq!(json["visualiser_shuffle"], "song");
        assert_eq!(json["box_shape"], "leaf");
        assert_eq!(settings.visualiser_audio, VisAudio::Player);
        // v20's extras all start on, with a 30 minute sleep timer.
        assert!(settings.speaker_fade && settings.moving_art);
        assert!(settings.karaoke_lyrics && settings.weekly_recap);
        assert!(settings.global_shortcuts && settings.startup_animation);
        assert!(settings.sleep_timer_button && settings.parallax && settings.cover_sheen);
        assert_eq!(settings.startup_style, StartupStyle::Bloom);
        assert_eq!(settings.sleep_timer_minutes, 30);
        assert_eq!(settings.weekly_recap_seen, None);
        // A file from before the song card was taken out still loads.
        let quiet: Settings = serde_json::from_str(
            r#"{"song_card":false,"sleep_timer_minutes":60,"weekly_recap_seen":"2026-W47","startup_style":"curtain","sleep_timer_button":false}"#,
        )
        .unwrap();
        assert_eq!(quiet.startup_style, StartupStyle::Curtain);
        assert!(!quiet.sleep_timer_button);
        assert_eq!(quiet.sleep_timer_minutes, 60);
        for style in StartupStyle::ALL {
            let saved = serde_json::to_value(style).unwrap();
            assert_eq!(
                serde_json::from_value::<StartupStyle>(saved).unwrap(),
                style
            );
        }
        assert!(!StartupStyle::PLAYED.contains(&StartupStyle::Random));
        assert!(settings.favourite_visualisers.is_empty());
        // Customise starts exactly as the app looked before it.
        assert_eq!(settings.interface_style, super::InterfaceStyle::Solid);
        assert_eq!(settings.custom_accent, None);
        assert_eq!(settings.seek_style, super::SeekStyle::Classic);
        assert_eq!(settings.play_button, super::PlayButtonStyle::Circle);
        assert_eq!(settings.vis_intensity, super::VisIntensity::Full);
        assert_eq!(settings.cursor_trail, super::CursorTrail::Off);
        assert_eq!(settings.background_pattern, super::BackgroundPattern::None);
        assert_eq!(settings.row_hover, super::RowHover::Fill);
        assert!(settings.greeting_name.is_empty() && !settings.time_remaining);
        let more: Settings = serde_json::from_str(
            r#"{"background_pattern":"stars","row_hover":"bar","greeting_name":"Louis","time_remaining":true,"visualizer_colours":"neon"}"#,
        )
        .unwrap();
        assert_eq!(more.background_pattern, super::BackgroundPattern::Stars);
        assert_eq!(more.row_hover, super::RowHover::Bar);
        assert_eq!(more.greeting_name, "Louis");
        assert!(more.time_remaining);
        assert_eq!(more.visualizer_colours, VisColours::Neon);
        assert!(VisColours::Neon.scheme().is_some() && VisColours::Auto.scheme().is_none());
        let custom: Settings = serde_json::from_str(
            r#"{"interface_style":"glass","glass_frost":"heavy","glass_backdrop":"albumart","custom_accent":[236,72,153],"seek_style":"wave","play_button":"ring","vis_intensity":"soft","cursor_trail":"comet"}"#,
        )
        .unwrap();
        assert_eq!(custom.interface_style, super::InterfaceStyle::Glass);
        assert_eq!(custom.glass_frost, super::GlassFrost::Heavy);
        assert_eq!(custom.glass_backdrop, super::GlassBackdrop::AlbumArt);
        assert_eq!(custom.custom_accent, Some([236, 72, 153]));
        assert_eq!(custom.seek_style, super::SeekStyle::Wave);
        assert_eq!(custom.play_button, super::PlayButtonStyle::Ring);
        assert_eq!(custom.cursor_trail, super::CursorTrail::Comet);
        assert_eq!(
            serde_json::from_value::<Settings>(serde_json::to_value(&custom).unwrap())
                .unwrap()
                .seek_style,
            super::SeekStyle::Wave
        );
        let hearts: Settings =
            serde_json::from_str(r#"{"favourite_visualisers":["galaxy","orb"]}"#).unwrap();
        assert_eq!(
            hearts.favourite_visualisers,
            [PlayerBarVis::Galaxy, PlayerBarVis::Orb]
        );
        assert_eq!(quiet.weekly_recap_seen.as_deref(), Some("2026-W47"));
        let everything: Settings =
            serde_json::from_str(r#"{"visualiser_audio":"everything"}"#).unwrap();
        assert_eq!(everything.visualiser_audio, VisAudio::Everything);
        assert_eq!(
            serde_json::to_value(&everything).unwrap()["visualiser_audio"],
            "everything"
        );
        // Each choice has its own name in the picker.
        for (index, effect) in ClickEffect::ALL.into_iter().enumerate() {
            assert!(
                ClickEffect::ALL[..index]
                    .iter()
                    .all(|earlier| earlier.label(crate::i18n::Locale::English)
                        != effect.label(crate::i18n::Locale::English))
            );
        }
        for (index, shape) in BoxShape::ALL.into_iter().enumerate() {
            assert!(
                BoxShape::ALL[..index]
                    .iter()
                    .all(|earlier| earlier.label(crate::i18n::Locale::English)
                        != shape.label(crate::i18n::Locale::English))
            );
        }
        assert_eq!(settings.visualizer_colours, VisColours::Auto);
        // Untouched, the new lists add nothing to an existing file.
        let json = serde_json::to_value(&settings).unwrap();
        for field in ["favourite_themes", "recent_themes", "page_reveals_off"] {
            assert!(
                json.get(field).is_none(),
                "{field} is written only when used"
            );
        }
        assert_eq!(json["visualizer_colours"], "auto");

        assert!(settings.toggle_favourite_theme(MotionTheme::Crimson));
        assert!(settings.toggle_favourite_theme(MotionTheme::Amethyst));
        assert!(settings.is_favourite_theme(MotionTheme::Crimson));
        assert!(!settings.toggle_favourite_theme(MotionTheme::Crimson));
        assert_eq!(settings.favourite_theme_list(), [MotionTheme::Amethyst]);

        for theme in MotionTheme::ALL.into_iter().take(RECENT_THEMES + 5) {
            settings.note_recent_theme(theme);
        }
        settings.note_recent_theme(MotionTheme::ALL[RECENT_THEMES]);
        let recent = settings.recent_theme_list();
        assert_eq!(recent.len(), RECENT_THEMES, "the list stays short");
        assert_eq!(recent[0], MotionTheme::ALL[RECENT_THEMES], "newest first");
        assert_eq!(
            recent.iter().filter(|&&theme| theme == recent[0]).count(),
            1,
            "a theme used again moves up rather than repeating"
        );

        settings.page_reveals_off = vec!["blinds".into(), "clock".into()];
        settings.visualizer_colours = VisColours::Theme;
        let restored: Settings =
            serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
        assert!(restored == settings);

        // A name from a newer or older version is kept in the file but not
        // shown, and never makes the settings unreadable.
        let odd: Settings = serde_json::from_str(
            r#"{"favourite_themes":["motion-theme:not-real","motion-theme:crimson"],"visualizer_colours":"theme"}"#,
        )
        .unwrap();
        assert_eq!(odd.favourite_theme_list(), [MotionTheme::Crimson]);
        assert_eq!(odd.favourite_themes.len(), 2);
        assert_eq!(odd.visualizer_colours, VisColours::Theme);
    }

    #[test]
    fn the_player_bar_visualizer_is_opt_in_and_round_trips() {
        use super::PlayerBarVis;
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings.player_bar_vis, PlayerBarVis::Off);
        assert!(settings.player_bar_vis_click);
        let disabled = Settings {
            player_bar_vis_click: false,
            ..Settings::default()
        };
        let json = serde_json::to_string(&disabled).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert!(!restored.player_bar_vis_click);
        let stored_names = [
            "off",
            "spectrum",
            "waveform",
            "mirror",
            "waterfall",
            "prism",
            "helix",
            "synthwave",
            "terrain",
            "lava",
            "galaxy",
            "matrix",
            "spectrogram",
            "orb",
            "bubbles",
            "skyline",
            "ocean",
            "led",
            "fireflies",
            "heartbeat",
            "circuit",
        ];
        assert_eq!(PlayerBarVis::ALL.len(), 21);
        // A click moves through the favourites once there are two, in the
        // list's order, from wherever it is; shuffle picks among them too.
        let favourites = [PlayerBarVis::Orb, PlayerBarVis::Galaxy, PlayerBarVis::Off];
        assert_eq!(
            PlayerBarVis::Galaxy.next_among(&favourites),
            PlayerBarVis::Orb
        );
        assert_eq!(
            PlayerBarVis::Orb.next_among(&favourites),
            PlayerBarVis::Galaxy
        );
        assert_eq!(
            PlayerBarVis::Lava.next_among(&favourites),
            PlayerBarVis::Galaxy
        );
        assert_eq!(
            PlayerBarVis::Lava.next_among(&[PlayerBarVis::Orb]),
            PlayerBarVis::Lava.next(),
            "one favourite is not a list to move through"
        );
        assert_eq!(
            PlayerBarVis::shuffle_pool(PlayerBarVis::Lava, &favourites),
            [PlayerBarVis::Galaxy, PlayerBarVis::Orb]
        );
        assert_eq!(
            PlayerBarVis::shuffle_pool(PlayerBarVis::Orb, &favourites).len(),
            19,
            "one other favourite falls back to every mode but Off and Orb"
        );
        // Effects taken out in v17 load as a close one still offered, so an
        // older settings file stays readable.
        for (removed, now) in [
            ("aurora", PlayerBarVis::Ocean),
            ("radial", PlayerBarVis::Orb),
            ("orbit", PlayerBarVis::Orb),
            ("embers", PlayerBarVis::Fireflies),
            ("tunnel", PlayerBarVis::Galaxy),
            ("starfield", PlayerBarVis::Galaxy),
            ("plasma", PlayerBarVis::Ocean),
            ("ripples", PlayerBarVis::Ocean),
            ("lasers", PlayerBarVis::Led),
            ("fireworks", PlayerBarVis::Fireflies),
            ("kaleidoscope", PlayerBarVis::Orb),
            ("rose", PlayerBarVis::Orb),
        ] {
            let old: Settings =
                serde_json::from_str(&format!(r#"{{"player_bar_vis":"{removed}"}}"#)).unwrap();
            assert_eq!(old.player_bar_vis, now, "{removed}");
        }
        assert_eq!(PlayerBarVis::ALL.len(), stored_names.len());
        for (mode, stored) in PlayerBarVis::ALL.into_iter().zip(stored_names) {
            let settings = Settings {
                player_bar_vis: mode,
                ..Settings::default()
            };
            let json = serde_json::to_value(&settings).unwrap();
            assert_eq!(json["player_bar_vis"], stored);
            let restored: Settings = serde_json::from_value(json).unwrap();
            assert_eq!(restored.player_bar_vis, mode);
        }
        let spectrum: Settings = serde_json::from_str(r#"{"player_bar_vis":"spectrum"}"#).unwrap();
        assert_eq!(spectrum.player_bar_vis, PlayerBarVis::Spectrum);
        let waveform: Settings = serde_json::from_str(r#"{"player_bar_vis":"waveform"}"#).unwrap();
        assert_eq!(waveform.player_bar_vis, PlayerBarVis::Waveform);
    }

    #[test]
    fn the_player_bar_click_cycle_visits_every_mode_once() {
        use super::PlayerBarVis;
        let mut next = PlayerBarVis::Off;
        for (index, mode) in PlayerBarVis::ALL.into_iter().enumerate() {
            assert_eq!(next, mode);
            assert!(!PlayerBarVis::ALL[..index].contains(&mode));
            assert!(
                PlayerBarVis::ALL[..index]
                    .iter()
                    .all(|earlier| earlier.label(crate::i18n::Locale::English)
                        != mode.label(crate::i18n::Locale::English))
            );
            assert!(!mode.label(crate::i18n::Locale::English).is_empty());
            next = next.next();
        }
        assert_eq!(next, PlayerBarVis::Off);
    }

    #[test]
    fn older_settings_keep_the_chosen_skin() {
        let settings: Settings = serde_json::from_str(r#"{"skin":"A.wsz"}"#).unwrap();
        assert!(!settings.random_skin);
        assert_eq!(settings.skin.as_deref(), Some("A.wsz"));
    }

    #[test]
    fn middle_click_autoscroll_is_opt_in_and_round_trips() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert!(!settings.middle_click_autoscroll);
        let settings = Settings {
            middle_click_autoscroll: true,
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert!(restored.middle_click_autoscroll);
    }

    #[test]
    fn older_settings_use_the_system_proxy() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings.proxy_mode, super::ProxyMode::System);
        assert!(settings.proxy_host.is_empty());
        assert!(settings.proxy_port.is_empty());
        assert!(settings.proxy_username.is_empty());
        assert!(settings.proxy_password.is_empty());
        assert_eq!(settings.proxy_config().unwrap(), super::ProxyConfig::System);
    }

    #[test]
    fn a_proxy_round_trips_through_settings() {
        let settings = Settings {
            proxy_mode: super::ProxyMode::Socks,
            proxy_host: "127.0.0.1".into(),
            proxy_port: "1080".into(),
            proxy_username: "alice".into(),
            proxy_password: "secret".into(),
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.proxy_mode, settings.proxy_mode);
        assert_eq!(restored.proxy_host, settings.proxy_host);
        assert_eq!(restored.proxy_port, settings.proxy_port);
        assert_eq!(restored.proxy_username, settings.proxy_username);
        assert!(restored.proxy_password.is_empty());
        assert!(json.contains("socks"));
        assert!(json.contains("127.0.0.1"));
        assert!(json.contains("1080"));
        assert!(json.contains("alice"));
        assert!(!json.contains("secret"));
        assert!(!json.contains("proxy_password"));
        assert!(!json.contains("\"proxy\":"));
    }

    #[test]
    fn settings_debug_keeps_proxy_credentials_private() {
        let settings = Settings {
            proxy_username: "private-proxy-user".into(),
            proxy_password: "private-proxy-password".into(),
            ..Default::default()
        };
        let debug = format!("{settings:?}");
        assert!(!debug.contains("private-proxy-user"));
        assert!(!debug.contains("private-proxy-password"));
    }

    #[test]
    fn a_saved_proxy_password_is_bound_to_its_endpoint_and_username() {
        let original = Settings {
            proxy_mode: super::ProxyMode::Http,
            proxy_host: "127.0.0.1".into(),
            proxy_port: "8080".into(),
            proxy_username: "dummy-user".into(),
            proxy_password: " dummy-private-password\n".into(),
            ..Default::default()
        };
        let password = original.proxy_password_record().unwrap().unwrap();
        let mut restored = original.clone();
        restored.proxy_password.clear();
        assert!(restored.restore_proxy_password(&password));
        assert_eq!(restored.proxy_password, original.proxy_password);
        for (host, port, username) in [
            ("other.invalid", "8080", "dummy-user"),
            ("127.0.0.1", "8081", "dummy-user"),
            ("127.0.0.1", "8080", "other-user"),
        ] {
            let mut changed = Settings {
                proxy_host: host.into(),
                proxy_port: port.into(),
                proxy_username: username.into(),
                ..restored.clone()
            };
            changed.proxy_password.clear();
            assert!(!changed.restore_proxy_password(&password));
            assert!(changed.proxy_password.is_empty());
        }
        assert!(
            !serde_json::to_string(&restored)
                .unwrap()
                .contains("dummy-private-password")
        );
    }

    #[test]
    fn an_empty_proxy_is_omitted_from_the_file() {
        let json = serde_json::to_string(&Settings::default()).unwrap();
        assert!(!json.contains("proxy"));
    }

    #[test]
    fn off_is_written_and_round_trips() {
        let settings = Settings {
            proxy_mode: super::ProxyMode::Off,
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"off\""));
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.proxy_mode, super::ProxyMode::Off);
        assert_eq!(restored.proxy_config().unwrap(), super::ProxyConfig::Off);
    }

    #[test]
    fn a_legacy_socks_url_becomes_socks_mode() {
        let dir = std::env::temp_dir().join(format!("spotifast-proxy-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("settings.json");
        std::fs::write(
            &path,
            r#"{"proxy":"socks5://127.0.0.1:1080","proxy_username":"alice"}"#,
        )
        .unwrap();
        let settings = Settings::load(&path);
        assert_eq!(settings.proxy_mode, super::ProxyMode::Socks);
        assert_eq!(settings.proxy_host, "127.0.0.1");
        assert_eq!(settings.proxy_port, "1080");
        let super::ProxyConfig::Socks(manual) = settings.proxy_config().unwrap() else {
            panic!("expected a SOCKS proxy");
        };
        assert_eq!(manual.redacted().scheme(), "socks5h");
        assert_eq!(manual.redacted().host_str(), Some("127.0.0.1"));
        assert_eq!(manual.redacted().port(), Some(1080));
        assert!(manual.redacted().username().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn http_and_socks_parse_a_host_and_port() {
        let http = super::ManualProxy::parse(super::ManualKind::Http, "127.0.0.1", "7890", "", "")
            .unwrap();
        assert_eq!(http.redacted().as_str(), "http://127.0.0.1:7890/");
        assert!(
            super::ProxyConfig::Http(http.clone())
                .librespot_url()
                .is_some()
        );

        let socks =
            super::ManualProxy::parse(super::ManualKind::Socks, "127.0.0.1", "1080", "", "")
                .unwrap();
        assert_eq!(socks.redacted().scheme(), "socks5h");
        assert_eq!(socks.redacted().port(), Some(1080));
        assert_eq!(super::ProxyConfig::Socks(socks).librespot_url(), None);
    }

    #[test]
    fn engine_only_accepts_plain_http_proxy_urls() {
        assert_eq!(super::ProxyConfig::Off.librespot_url(), None);
        let http_uri: http::Uri = "http://127.0.0.1:8080".parse().unwrap();
        let https_uri: http::Uri = "https://127.0.0.1:8080".parse().unwrap();
        let socks_uri: http::Uri = "socks5://127.0.0.1:1080".parse().unwrap();
        assert!(super::http_proxy_from_uri(&http_uri).is_some());
        assert!(super::http_proxy_from_uri(&https_uri).is_none());
        assert!(super::http_proxy_from_uri(&socks_uri).is_none());
    }

    #[test]
    fn proxy_credentials_stay_opaque_and_out_of_urls() {
        let proxy = super::ManualProxy::parse(
            super::ManualKind::Http,
            "127.0.0.1",
            "7890",
            "alice/name",
            " secret/with spaces ",
        )
        .unwrap();
        let redacted = proxy.redacted();
        assert!(redacted.username().is_empty());
        assert_eq!(redacted.password(), None);
        assert_eq!(proxy.username, "alice/name");
        assert_eq!(proxy.password, " secret/with spaces ");
        assert_eq!(
            super::ProxyConfig::Http(proxy.clone()).librespot_url(),
            None
        );
        let debug = format!("{proxy:?}");
        assert!(!debug.contains("alice"));
        assert!(!debug.contains("secret"));
    }

    #[test]
    fn system_proxy_authentication_and_tls_endpoints_stay_out_of_librespot() {
        let authenticated = hyper_util::client::proxy::matcher::Matcher::builder()
            .all("http://alice:secret@127.0.0.1:8080")
            .build();
        let destination = http::Uri::from_static("https://apresolve.spotify.com");
        let intercept = authenticated.intercept(&destination).unwrap();
        assert!(super::librespot_system_proxy(&intercept).is_none());

        let tls = hyper_util::client::proxy::matcher::Matcher::builder()
            .all("https://127.0.0.1:8080")
            .build();
        let intercept = tls.intercept(&destination).unwrap();
        assert!(super::librespot_system_proxy(&intercept).is_none());
    }

    #[test]
    fn proxy_parse_rejects_a_missing_host_or_port() {
        assert!(
            super::ManualProxy::parse(super::ManualKind::Http, "", "8080", "", "")
                .unwrap_err()
                .contains("host")
        );
        assert!(
            super::ManualProxy::parse(super::ManualKind::Http, "127.0.0.1", "", "", "")
                .unwrap_err()
                .contains("port")
        );
        assert!(
            super::ManualProxy::parse(super::ManualKind::Http, "127.0.0.1", "abc", "", "")
                .unwrap_err()
                .contains("number")
        );
        assert!(
            super::ManualProxy::parse(super::ManualKind::Http, "not a host", "1080", "", "")
                .unwrap_err()
                .contains("hostname")
        );
        assert!(
            super::ManualProxy::parse(super::ManualKind::Http, "127.0.0.1", "0", "", "")
                .unwrap_err()
                .contains("65535")
        );
        assert!(
            super::ManualProxy::parse(super::ManualKind::Http, "127.0.0.1", "65536", "", "")
                .unwrap_err()
                .contains("65535")
        );
        super::ManualProxy::parse(super::ManualKind::Http, "localhost", "8080", "", "").unwrap();
        super::ManualProxy::parse(super::ManualKind::Http, "::1", "8080", "", "").unwrap();
    }

    #[test]
    fn personal_app_nudge_time_is_backward_compatible_and_round_trips() {
        let older: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(older.personal_app_nudge_at, None);
        assert!(!older.personal_app_intro_seen);

        let settings = Settings {
            personal_app_nudge_at: Some("2026-09-03T15:00:00Z".into()),
            personal_app_intro_seen: true,
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(
            restored.personal_app_nudge_at,
            settings.personal_app_nudge_at
        );
        assert!(restored.personal_app_intro_seen);
    }

    #[test]
    fn slow_app_banner_state_is_backward_compatible_and_round_trips() {
        let older: Settings = serde_json::from_str(r#"{"personal_app_intro_seen":true}"#).unwrap();
        assert_eq!(older.slow_app_banner, super::SlowAppBanner::default());
        assert_eq!(older.slow_app_banner.dismissals, 0);
        let partial: Settings =
            serde_json::from_str(r#"{"slow_app_banner":{"dismissals":2}}"#).unwrap();
        assert_eq!(partial.slow_app_banner.dismissals, 2);
        assert_eq!(partial.slow_app_banner.shown_at, None);

        let settings = Settings {
            slow_app_banner: super::SlowAppBanner {
                shown_at: Some("2026-10-21T09:00:00Z".into()),
                dismissals: 1,
            },
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.slow_app_banner, settings.slow_app_banner);
    }
}

/// The last playlist tree received for one account.
///
/// Only folder order is cached. Edit grants must always come from the live
/// session because they can be revoked.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CachedRootlist {
    pub account_id: String,
    pub entries: Vec<crate::player::RootlistEntry>,
}

/// Restorable UI session: what was open when the app last closed.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionState {
    /// Recent explicit output choices, scoped to the verified Spotify account.
    pub device_history: crate::device_history::DeviceHistory,
    pub last_page: Option<String>,
    /// Context URIs most recently played, newest first.
    pub recent_contexts: Vec<String>,
    /// What was playing when the app closed, to resume from a cold start.
    pub last_context: Option<String>,
    pub last_track: Option<String>,
    pub last_position_ms: u32,
    /// Manually queued songs to restore with the remembered track.
    ///
    /// Context rows are excluded to prevent duplicates. This replaced the old
    /// `last_queue` field, so sessions using that field restore no added rows.
    pub last_added_queue: Vec<String>,
    /// Queue rows displayed on the next start. Playback restores manual rows
    /// from `last_added_queue`; it does not enqueue this list.
    pub last_queue_rows: Vec<crate::api::models::PlayableItem>,
    /// Sidebar folders rolled up, by their rootlist ids.
    pub collapsed_folders: Vec<String>,
    /// Last good playlist tree, scoped to the account that supplied it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rootlist: Option<CachedRootlist>,
    /// Shuffle mode saved across contexts and restarts.
    pub shuffle_on: bool,
    /// Local repeat mode ("off", "context" or "track") saved across
    /// restarts. Sessions saved before it existed read as off.
    pub repeat: String,
    /// Each table's chosen sort, by encoded page, restored at start.
    pub sorts: Vec<(String, crate::model::TableSort)>,
    /// Last window inner size, to restore on next launch. Logical pixels
    /// (points at zoom 1) since fastframe-shell's `Geometry`; earlier
    /// versions saved points at the interface zoom, the same at zoom 1.
    pub window_size: Option<[f32; 2]>,
    /// Last window position, to restore on next launch: the outer frame's
    /// corner, the content's on macOS.
    pub window_pos: Option<[f32; 2]>,
    /// Whether the window was maximized; `window_size` and `window_pos`
    /// stay those of the normal window behind it.
    pub window_maximized: bool,
    /// Whether the queue panel was open.
    pub queue_open: Option<bool>,
    /// Which tab the queue panel showed: `queue` or `recents`.
    pub queue_tab: Option<String>,
    /// Last outer position of the Winamp window.
    pub winamp_pos: Option<[f32; 2]>,
    /// Last outer position of the MilkDrop window.
    pub milkdrop_pos: Option<[f32; 2]>,
    /// The window mode fullscreen lyrics left, when the app closed while
    /// showing them. eframe restores the window full screen, so the next
    /// start returns it to this mode instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyrics_fullscreen_from: Option<WindowMode>,
}

/// Whether a window was full screen, and whether it was maximized.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowMode {
    pub fullscreen: bool,
    pub maximized: bool,
}

impl SessionState {
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    /// The session as its file holds it.
    pub fn encode(&self) -> Option<String> {
        serde_json::to_string(self)
            .map_err(|error| log::warn!("unable to encode session: {error}"))
            .ok()
    }

    /// Writes the session and waits for the disk. The app saves through
    /// its [`fastframe_file::Writer`] instead.
    pub fn save(&self, path: &Path) {
        let Some(text) = self.encode() else { return };
        if let Err(error) = fastframe_file::write(path, text) {
            log::warn!("unable to save session to {}: {error}", path.display());
        }
    }
}

#[cfg(test)]
mod session_tests {
    use super::{CachedRootlist, SessionState};
    use crate::player::RootlistEntry;

    /// A session saved before the maximized state was remembered keeps its
    /// window size and positions, and reads as an ordinary window.
    #[test]
    fn old_sessions_keep_their_window_geometry() {
        let state: SessionState = serde_json::from_str(
            r#"{"window_size":[1100.0,700.0],"window_pos":[100.0,150.0],"winamp_pos":[300.0,200.0]}"#,
        )
        .unwrap();
        assert_eq!(state.window_size, Some([1100.0, 700.0]));
        assert_eq!(state.window_pos, Some([100.0, 150.0]));
        assert!(!state.window_maximized);
        assert_eq!(state.winamp_pos, Some([300.0, 200.0]));
    }

    #[test]
    fn old_sessions_without_a_playlist_tree_remain_readable() {
        let state: SessionState = serde_json::from_str(r#"{"last_page":"home"}"#).unwrap();
        assert_eq!(state.last_page.as_deref(), Some("home"));
        assert_eq!(state.rootlist, None);
        assert_eq!(
            crate::player::RepeatMode::from_api(&state.repeat),
            crate::player::RepeatMode::Off,
            "no saved repeat mode is off"
        );
    }

    #[test]
    fn the_repeat_mode_round_trips() {
        let state = SessionState {
            repeat: crate::player::RepeatMode::Track.api_name().into(),
            ..SessionState::default()
        };
        let json = serde_json::to_string(&state).unwrap();
        let read = serde_json::from_str::<SessionState>(&json).unwrap();
        assert_eq!(
            crate::player::RepeatMode::from_api(&read.repeat),
            crate::player::RepeatMode::Track
        );
    }

    #[test]
    fn the_playlist_tree_round_trips_with_its_account() {
        let state = SessionState {
            rootlist: Some(CachedRootlist {
                account_id: "listener".into(),
                entries: vec![
                    RootlistEntry::FolderStart {
                        id: "folder".into(),
                        name: "Favorites".into(),
                    },
                    RootlistEntry::Playlist("spotify:playlist:one".into()),
                    RootlistEntry::FolderEnd,
                ],
            }),
            ..SessionState::default()
        };

        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(serde_json::from_str::<SessionState>(&json).unwrap(), state);
    }

    #[test]
    fn a_new_session_atomically_replaces_the_previous_one() {
        let root = std::env::temp_dir().join(format!(
            "spotifast-session-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let path = root.join("session.json");
        let state = |page: &str| SessionState {
            last_page: Some(page.into()),
            ..SessionState::default()
        };

        state("home").save(&path);
        state("liked").save(&path);

        assert_eq!(
            SessionState::load(&path).last_page.as_deref(),
            Some("liked")
        );
        assert!(crate::util::temporary_files(path.parent().unwrap()).is_empty());
        let _ = std::fs::remove_dir_all(root);
    }
}
