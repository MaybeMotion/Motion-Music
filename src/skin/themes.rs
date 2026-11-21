//! Motion Music's own mini player themes: the built-in skin recoloured in
//! three colours each, drawn on the fly rather than shipped as files. A
//! theme is chosen like any skin, under a name that no file can have.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use super::{Rgb, Skin, VisColors};

/// What sets a theme's name apart from a skin file's.
pub const PREFIX: &str = "motion:";

/// A mini player theme: its saved name, its name in the picker, and its
/// colours from darkest to brightest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MiniTheme {
    pub id: &'static str,
    pub name: &'static str,
    pub stops: [Rgb; 3],
}

pub const THEMES: [MiniTheme; 8] = [
    MiniTheme {
        id: "motion:neon-night",
        name: "Neon Night",
        stops: [[38, 12, 84], [214, 40, 220], [80, 240, 255]],
    },
    MiniTheme {
        id: "motion:sunset",
        name: "Sunset",
        stops: [[70, 14, 40], [240, 90, 60], [255, 210, 120]],
    },
    MiniTheme {
        id: "motion:ocean",
        name: "Ocean",
        stops: [[6, 30, 66], [20, 140, 200], [150, 240, 230]],
    },
    MiniTheme {
        id: "motion:forest",
        name: "Forest",
        stops: [[10, 40, 24], [60, 160, 80], [200, 240, 150]],
    },
    MiniTheme {
        id: "motion:mint",
        name: "Mint",
        stops: [[14, 40, 38], [50, 196, 160], [210, 255, 234]],
    },
    MiniTheme {
        id: "motion:ember",
        name: "Ember",
        stops: [[40, 8, 4], [210, 60, 20], [255, 190, 60]],
    },
    MiniTheme {
        id: "motion:candy",
        name: "Candy",
        stops: [[120, 50, 110], [255, 140, 190], [255, 235, 245]],
    },
    MiniTheme {
        id: "motion:mono",
        name: "Mono",
        stops: [[24, 24, 28], [130, 132, 140], [240, 240, 244]],
    },
];

/// The theme saved under `name`, if it is one.
pub fn find(name: &str) -> Option<&'static MiniTheme> {
    THEMES.iter().find(|theme| theme.id == name)
}

/// The theme's skin, made once and kept.
pub fn skin(theme: &MiniTheme) -> Arc<Skin> {
    static MADE: LazyLock<Mutex<HashMap<&'static str, Arc<Skin>>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));
    let mut made = MADE.lock().unwrap_or_else(|p| p.into_inner());
    made.entry(theme.id)
        .or_insert_with(|| Arc::new(Skin::builtin().recoloured(theme.name, &theme.ramp())))
        .clone()
}

impl MiniTheme {
    /// The colours a pixel's brightness is mapped along: near black, the
    /// three stops, and near white, evenly spaced.
    fn ramp(&self) -> Ramp {
        let [dark, middle, bright] = self.stops;
        let scale = |colour: Rgb, by: f32| colour.map(|channel| (f32::from(channel) * by) as u8);
        let lift = |colour: Rgb, by: f32| {
            colour.map(|channel| (f32::from(channel) + (255.0 - f32::from(channel)) * by) as u8)
        };
        Ramp([scale(dark, 0.25), dark, middle, bright, lift(bright, 0.7)])
    }
}

/// Five colours, darkest first, that brightness 0 to 1 runs along.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ramp(pub [Rgb; 5]);

impl Ramp {
    /// The colour at `at`, 0 (darkest) to 1 (brightest).
    pub fn at(&self, at: f32) -> Rgb {
        let scaled = at.clamp(0.0, 1.0) * 4.0;
        let index = (scaled.floor() as usize).min(3);
        let t = scaled - index as f32;
        let (from, to) = (self.0[index], self.0[index + 1]);
        std::array::from_fn(|channel| {
            (f32::from(from[channel]) + (f32::from(to[channel]) - f32::from(from[channel])) * t)
                .round() as u8
        })
    }

    /// A colour of the original skin, recoloured by its brightness. Strong
    /// colours lean a little brighter, so the play button's and the
    /// visualizer's highlights still stand out.
    pub fn map(&self, colour: Rgb) -> Rgb {
        let [r, g, b] = colour.map(f32::from);
        let brightness = (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255.0;
        let saturation = (r.max(g).max(b) - r.min(g).min(b)) / 255.0;
        self.at(brightness + saturation * 0.2)
    }

    /// A visualizer palette in these colours: a dark background, the
    /// spectrum from brightest at the top to darkest at the foot, and the
    /// oscilloscope in the middle colours.
    pub fn vis_colors(&self) -> VisColors {
        let mut colours = [[0; 3]; 24];
        colours[0] = self.at(0.02);
        colours[1] = self.at(0.3);
        for (index, colour) in colours[2..18].iter_mut().enumerate() {
            *colour = self.at(1.0 - index as f32 / 15.0 * 0.75);
        }
        for (index, colour) in colours[18..23].iter_mut().enumerate() {
            *colour = self.at(0.95 - index as f32 * 0.1);
        }
        colours[23] = self.at(1.0);
        colours
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skin::{Sheet, sprites};

    #[test]
    fn every_theme_has_its_own_name_and_is_found_by_it() {
        for (index, theme) in THEMES.iter().enumerate() {
            assert!(theme.id.starts_with(PREFIX));
            assert_eq!(find(theme.id), Some(theme));
            assert!(THEMES[..index].iter().all(|other| other.id != theme.id));
        }
        assert_eq!(find("Some Skin.wsz"), None);
    }

    #[test]
    fn a_theme_recolours_the_built_in_skin_and_keeps_its_shape() {
        let theme = &THEMES[0];
        let skin = skin(theme);
        let builtin = Skin::builtin();
        assert_eq!(skin.name, "Neon Night");
        for sheet in Sheet::ALL {
            let (ours, original) = (skin.sheet(sheet), builtin.sheet(sheet));
            assert_eq!((ours.width, ours.height), (original.width, original.height));
            // See-through pixels stay see-through.
            for (a, b) in ours.rgba.chunks(4).zip(original.rgba.chunks(4)) {
                assert_eq!(a[3], b[3]);
            }
        }
        assert_ne!(
            skin.sheet(sprites::PLAY.sheet).rgba,
            builtin.sheet(sprites::PLAY.sheet).rgba,
            "the colours changed"
        );
        assert!(Arc::ptr_eq(&skin, &super::skin(theme)), "made once");
    }

    #[test]
    fn brightness_runs_from_dark_to_bright_along_the_ramp() {
        let ramp = THEMES[2].ramp();
        assert_eq!(ramp.at(0.0), ramp.0[0]);
        assert_eq!(ramp.at(0.5), ramp.0[2]);
        assert_eq!(ramp.at(1.0), ramp.0[4]);
        let sum = |colour: Rgb| colour.iter().map(|&c| u32::from(c)).sum::<u32>();
        assert!(sum(ramp.map([20, 20, 20])) < sum(ramp.map([230, 230, 230])));
        let vis = ramp.vis_colors();
        assert!(
            sum(vis[2]) > sum(vis[17]),
            "the spectrum is brightest at the top"
        );
    }
}
