//! The now-playing bar along the bottom of the window.

use egui::{Align, Color32, Frame, Layout, Margin, Rect, Sense, UiBuilder, Vec2, pos2, vec2};

use crate::app::{App, NowPlaying};
use crate::i18n::gettext;
use crate::model::{Action, DragTrack, Page};
use crate::player::RepeatMode;
use crate::theme::{self, Icon};
use crate::util;

use super::widgets::{SliderEvent, thin_slider};

mod scenes;
use scenes::{circuit, fireflies, heartbeat, led_meter, ocean, skyline};

/// How much of the playing art's tint the bar's fill carries.
const TINT_STRENGTH: f32 = 0.12;
/// How long the bar takes to cross over to a new song's tint.
const TINT_FADE_SECONDS: f32 = 0.30;
const TINT_SESSION_ID: &str = "player-bar-tint-session";
/// The Add to playlist menu a liked song's heart opens.
const LIKED_POPUP_ID: &str = "player-bar-liked-playlists";
/// How strongly the visualizer shows through behind the controls: the
/// spectrum's bars fade from their foot to their top, with a glow around
/// them, a brighter cap above, and a pulse along the foot with the bass.
const SPECTRUM_ALPHA: (f32, f32) = (0.28, 0.08);
const GLOW_ALPHA: f32 = 0.12;
const PEAK_ALPHA: f32 = 0.7;
const PULSE_ALPHA: f32 = 0.35;
/// The peak cap's thickness, and its gap above the band.
const PEAK_HEIGHT: f32 = 2.0;
const PEAK_GAP: f32 = 2.0;
/// The waveform's line, as layers from the widest glow to the core, the
/// shading under it, and its mirrored echo.
const WAVE_LAYERS: [(f32, f32); 3] = [(9.0, 0.06), (4.0, 0.16), (1.6, 0.85)];
const WAVE_FILL_ALPHA: f32 = 0.14;
const WAVE_ECHO_ALPHA: f32 = 0.18;
/// How much more strongly the visualizer paints over a light bar.
const LIGHT_STRENGTH: f32 = 1.5;
/// The gap between spectrum bars.
const SPECTRUM_GAP: f32 = 2.0;
/// How much more strongly the full-window stage paints every effect.
const STAGE_STRENGTH: f32 = 1.6;

/// Forget this bar's animation session while the sign-in screen is shown.
pub(crate) fn end_tint_session(ctx: &egui::Context) {
    ctx.data_mut(|data| data.remove::<u64>(egui::Id::new(TINT_SESSION_ID)));
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let fill = eased_fill(ui.ctx(), palette.panel, app.now_playing_tint());
    egui::Panel::bottom("player-bar")
        .exact_size(theme::PLAYER_BAR_HEIGHT)
        .resizable(false)
        .show_separator_line(false)
        .frame(
            Frame::new()
                .fill(fill)
                .inner_margin(Margin::symmetric(16, 0)),
        )
        .show(ui, |ui| {
            let rect = ui.max_rect();
            let now = app.now_playing();
            // The whole bar, margins included, behind everything else.
            let behind = rect.expand2(vec2(16.0, 0.0));
            if let Some(stops) = app.background_gradient() {
                let stops = stops.map(|stop| stop.gamma_multiply(0.45));
                theme::paint_theme_gradient(ui, behind, stops, 0);
            }
            // The stage, when open, draws the visualizer big instead.
            if !app.visualiser_stage && visualizer(app, ui, behind, now.as_ref(), false) {
                theme::visualiser_frame(ui.ctx(), app.visualiser_fps());
            }
            // Its empty space is the visualizer's control, as Winamp's
            // visualizer was: a click moves to the next mode. The controls
            // drawn after it take their own clicks.
            if app.settings.player_bar_vis_click {
                let empty = ui.interact(
                    behind,
                    ui.id().with("player-bar-visualizer"),
                    Sense::click(),
                );
                // An open tooltip stays while the pointer is inside its widget's
                // rect, and this one spans the bar: offer it only while the empty
                // space itself is hovered, not the controls over it.
                let empty = if empty.hovered() {
                    empty.on_hover_text_at_pointer(gettext(
                        app.locale,
                        "Click to change the visualizer",
                    ))
                } else {
                    empty
                };
                if empty.clicked() {
                    app.actions.push(Action::CyclePlayerBarVis);
                }
            }
            let clock = ui.input(|input| input.time);
            let reduced = crate::motion::reduced(ui.ctx());
            let pulse =
                beat_pulse(app, ui, now.as_ref()) * speaker_fade(app, now.as_ref(), clock, reduced);
            if pulse > 0.01 {
                paint_pulse_glow(
                    ui.painter(),
                    behind,
                    rect.top(),
                    &visualiser_palette(app, None),
                    pulse * app.settings.beat_pulse_strength.amount(),
                );
            }
            desktop_audio_badge(app, ui, behind);
            ui.painter().hline(
                rect.x_range(),
                rect.top() + 0.5,
                egui::Stroke::new(1.0, palette.outline),
            );
            let width = rect.width();
            let side = (width * 0.3).clamp(200.0, 420.0);
            let cy = rect.center().y;
            let left = Rect::from_min_max(rect.min, pos2(rect.left() + side, rect.bottom()));
            let center = Rect::from_min_max(
                pos2(rect.left() + side, rect.top()),
                pos2(rect.right() - side, rect.bottom()),
            );

            // egui's cross-axis centring is unreliable across nested layouts of
            // mixed heights, so each region is placed in an explicit band that
            // is sized to its content and centred on the bar's midline.
            now_playing_block(app, ui, left, now.as_ref());

            transport(app, ui, now.as_ref(), center);

            let right_band =
                Rect::from_min_size(pos2(rect.right() - side, cy - 15.0), vec2(side, 30.0));
            let mut right_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(right_band)
                    .layout(Layout::right_to_left(Align::Center)),
            );
            let transport_right = center.center().x + transport_width() / 2.0;
            extras(app, &mut right_ui, now.as_ref(), transport_right);
        });
}

/// A soft glow falling from the top edge of the bar, `amount` (0 to 1) of
/// its full brightness, swept across `colours` from left to right as the
/// visualisers are. It fades out smoothly down the bar, so even at full
/// strength there is no hard edge to flash.
fn paint_pulse_glow(
    painter: &egui::Painter,
    behind: Rect,
    top: f32,
    colours: &[Color32],
    amount: f32,
) {
    let mut glow = egui::Mesh::default();
    // Bands, each fainter, so the light thins out gradually down the bar,
    // and enough columns for the colours to blend smoothly across it.
    let stops = [(0.0, 0.55), (8.0, 0.3), (20.0, 0.1), (34.0, 0.0)];
    let columns = (colours.len().max(1) * 6) as u32;
    let width = behind.width() / columns as f32;
    for column in 0..columns {
        let (from_at, to_at) = (
            column as f32 / columns as f32,
            (column + 1) as f32 / columns as f32,
        );
        let (left, right) = (
            behind.left() + column as f32 * width,
            behind.left() + (column + 1) as f32 * width,
        );
        for pair in stops.windows(2) {
            let ((from, high), (to, low)) = (pair[0], pair[1]);
            let index = glow.vertices.len() as u32;
            let (c0, c1) = (swept(colours, from_at), swept(colours, to_at));
            glow.colored_vertex(pos2(left, top + from), c0.gamma_multiply(high * amount));
            glow.colored_vertex(pos2(right, top + from), c1.gamma_multiply(high * amount));
            glow.colored_vertex(pos2(right, top + to), c1.gamma_multiply(low * amount));
            glow.colored_vertex(pos2(left, top + to), c0.gamma_multiply(low * amount));
            glow.add_triangle(index, index + 1, index + 2);
            glow.add_triangle(index, index + 2, index + 3);
        }
    }
    painter.add(egui::Shape::mesh(glow));
    // At Strong the whole bar warms a touch with each beat as well.
    let wash = ((amount - 0.6) / 0.4).clamp(0.0, 1.0) * 0.06 * amount;
    if wash > 0.0 {
        painter.rect_filled(behind, 0, swept(colours, 0.5).gamma_multiply(wash));
    }
}
/// Reads this frame's audio for beats when the beat pulse is on, and keeps
/// frames coming while there is sound. Answers the pulse's height now.
fn beat_pulse(app: &mut App, ui: &egui::Ui, now: Option<&NowPlaying>) -> f32 {
    if !app.settings.beat_pulse {
        return 0.0;
    }
    let time = ui.input(|input| input.time);
    // A beat starts at nothing and eases in, so its first frame asks for
    // the next even before there is anything to see.
    let started = match app.visualizer_samples(now, crate::vis::WIDE_SAMPLES) {
        Some(samples) => {
            theme::visualiser_frame(ui.ctx(), app.visualiser_fps());
            app.beat.step(&samples, time)
        }
        None => {
            app.beat.reset();
            false
        }
    };
    let pulse = app.beat.pulse(time);
    if started || pulse > 0.01 {
        ui.ctx().request_repaint();
    }
    if crate::motion::reduced(ui.ctx()) {
        0.0
    } else {
        pulse
    }
}

/// Names the fallback honestly: this is the PC's mix, not remote-device PCM.
fn desktop_audio_badge(app: &App, ui: &egui::Ui, rect: Rect) {
    use crate::desktop_audio::Status;
    let status = app.desktop_audio.status();
    if matches!(status, Status::Off) {
        return;
    }
    let text = match status {
        Status::Unavailable(_) => "Audio unavailable",
        _ => "Windows audio",
    };
    let badge = Rect::from_min_size(
        pos2(rect.right() - 116.0, rect.top() + 4.0),
        vec2(100.0, 15.0),
    );
    ui.painter().text(
        badge.right_center(),
        egui::Align2::RIGHT_CENTER,
        text,
        egui::FontId::proportional(10.0),
        app.palette.secondary,
    );
    ui.interact(badge, ui.id().with("desktop-audio-source"), Sense::hover())
        .on_hover_text(app.desktop_audio_hint());
}

/// Draws the chosen backdrop from native PCM or fresh Windows output audio.
/// Native PCM remains post-EQ and pre-volume. Returns whether it is moving.
fn visualizer(
    app: &mut App,
    ui: &egui::Ui,
    rect: Rect,
    now: Option<&NowPlaying>,
    stage: bool,
) -> bool {
    use crate::settings::PlayerBarVis;
    let clock = ui.input(|input| input.time);
    shuffle_visualiser(app, now, clock);
    let wanted = match app
        .vis_motion
        .shuffled
        .unwrap_or(app.settings.player_bar_vis)
    {
        // The stage always shows something; with the bar's visualizer off
        // it opens on the galaxy.
        PlayerBarVis::Off if stage => PlayerBarVis::Galaxy,
        mode => mode,
    };
    let reduced = crate::motion::reduced(ui.ctx());
    let (mode, opacity) = app.vis_motion.crossfade(wanted, clock, reduced);
    let handover = speaker_fade(app, now, clock, reduced);
    let opacity = opacity * handover;
    let fading = opacity < 1.0;
    if mode == PlayerBarVis::Off {
        return fading;
    }
    let moving = visualizer_mode(app, ui, rect, now, stage, mode, opacity);
    moving || fading
}

/// How long one visualiser takes to give way to the next: it fades out over
/// the first half and the next fades in over the second.
const VIS_CROSSFADE: f64 = 0.7;
/// How long Every minute leaves each visualiser.
const VIS_SHUFFLE_SECONDS: f64 = 60.0;

/// Which visualiser is showing, the one it is fading from, and the one
/// visualiser shuffle picked.
#[derive(Clone, Debug, Default)]
pub struct VisMotion {
    shown: Option<crate::settings::PlayerBarVis>,
    fade: Option<(crate::settings::PlayerBarVis, f64)>,
    /// The visualiser shuffle chose, shown instead of the saved choice.
    pub shuffled: Option<crate::settings::PlayerBarVis>,
    /// The song it was chosen for, or when, so the next comes on time.
    shuffled_for: Option<String>,
    shuffled_at: f64,
    /// The saved choice last seen, so a new choice shows at once.
    chosen: Option<crate::settings::PlayerBarVis>,
    /// The speaker last playing (`None` for this computer), and when the
    /// music last moved to another one.
    speaker: Option<Option<String>>,
    handover_at: Option<f64>,
}

/// How long a change of speaker dims the visualizers and beat pulse: out
/// over the first half, back in over the second.
const SPEAKER_FADE: f64 = 1.4;

impl VisMotion {
    /// How strongly to draw the visualizers at `now` (0 to 1) while
    /// `speaker` plays: a move to another speaker fades them out and back in
    /// rather than cutting from one sound to the next.
    pub(crate) fn speaker_fade(
        &mut self,
        speaker: Option<Option<String>>,
        now: f64,
        enabled: bool,
    ) -> f32 {
        if let Some(speaker) = speaker
            && self.speaker.as_ref() != Some(&speaker)
        {
            if self.speaker.is_some() && enabled {
                self.handover_at = Some(now);
            }
            self.speaker = Some(speaker);
        }
        let Some(start) = self.handover_at.filter(|_| enabled) else {
            return 1.0;
        };
        let progress = ((now - start) / SPEAKER_FADE).max(0.0) as f32;
        if progress >= 1.0 {
            self.handover_at = None;
            1.0
        } else {
            // Down to nothing at the middle, then back up.
            smooth((progress * 2.0 - 1.0).abs())
        }
    }

    /// The visualiser to draw at `now` while `wanted` is chosen, and how
    /// strongly: a change fades the old one out and the new one in.
    fn crossfade(
        &mut self,
        wanted: crate::settings::PlayerBarVis,
        now: f64,
        reduced: bool,
    ) -> (crate::settings::PlayerBarVis, f32) {
        let shown = *self.shown.get_or_insert(wanted);
        if shown != wanted {
            self.shown = Some(wanted);
            // A change in the middle of a fade continues from what is
            // visible rather than jumping back.
            let from = match self.fade {
                Some((from, start)) if (now - start) / VIS_CROSSFADE < 0.5 => from,
                _ => shown,
            };
            self.fade = (!reduced).then_some((from, now));
        }
        let Some((from, start)) = self.fade else {
            return (wanted, 1.0);
        };
        let progress = ((now - start) / VIS_CROSSFADE).max(0.0) as f32;
        if progress >= 1.0 {
            self.fade = None;
            (wanted, 1.0)
        } else if progress < 0.5 {
            (from, smooth(1.0 - progress * 2.0))
        } else {
            (wanted, smooth(progress * 2.0 - 1.0))
        }
    }
}

/// How strongly the visualizers and pulse show through a change of speaker.
fn speaker_fade(app: &mut App, now: Option<&NowPlaying>, clock: f64, reduced: bool) -> f32 {
    let speaker = now
        .filter(|now| !now.resuming)
        .map(|now| (!now.local).then(|| now.device_name.clone().unwrap_or_default()));
    let enabled = app.settings.speaker_fade && !reduced;
    app.vis_motion.speaker_fade(speaker, clock, enabled)
}

/// Eases from 0 to 1 gently at both ends.
fn smooth(amount: f32) -> f32 {
    let amount = amount.clamp(0.0, 1.0);
    amount * amount * (3.0 - 2.0 * amount)
}

/// Picks another visualiser when shuffle says it is time: for each new song,
/// or every minute. Only while a visualiser is on, and never Off.
fn shuffle_visualiser(app: &mut App, now: Option<&NowPlaying>, clock: f64) {
    use crate::settings::{PlayerBarVis, VisShuffle};
    let on = app.settings.player_bar_vis != PlayerBarVis::Off || app.visualiser_stage;
    let motion = &mut app.vis_motion;
    // A visualiser chosen in Settings shows straight away, until shuffle
    // next moves on.
    if motion.chosen.replace(app.settings.player_bar_vis) != Some(app.settings.player_bar_vis) {
        motion.shuffled = None;
        motion.shuffled_at = clock;
    }
    if app.settings.visualiser_shuffle == VisShuffle::Off || !on {
        motion.shuffled = None;
        motion.shuffled_for = None;
        return;
    }
    let due = match app.settings.visualiser_shuffle {
        VisShuffle::Off => false,
        VisShuffle::Song => {
            let song = now.map(|now| now.uri.clone());
            let changed = song.is_some() && song != motion.shuffled_for;
            if changed {
                motion.shuffled_for = song;
            }
            changed
        }
        VisShuffle::Minute => {
            clock - motion.shuffled_at >= VIS_SHUFFLE_SECONDS || clock < motion.shuffled_at
        }
    };
    if !due {
        return;
    }
    let showing = motion.shuffled.unwrap_or(app.settings.player_bar_vis);
    let choices: Vec<_> = PlayerBarVis::ALL
        .into_iter()
        .filter(|mode| *mode != PlayerBarVis::Off && *mode != showing)
        .collect();
    let pick = ((clock * 7919.0).sin() * 43_758.547).fract().abs();
    motion.shuffled = Some(choices[(pick * choices.len() as f64) as usize % choices.len()]);
    motion.shuffled_at = clock;
}

/// Draws visualiser `mode` at `opacity` (0 to 1). Returns whether it is
/// moving.
fn visualizer_mode(
    app: &mut App,
    ui: &egui::Ui,
    rect: Rect,
    now: Option<&NowPlaying>,
    stage: bool,
    mode: crate::settings::PlayerBarVis,
    opacity: f32,
) -> bool {
    use crate::settings::PlayerBarVis;
    use crate::vis;
    let count = match mode {
        PlayerBarVis::Waveform | PlayerBarVis::Mirror => {
            (rect.width() / 8.0).clamp(75.0, 320.0) as usize * vis::SCOPE_STEP
        }
        _ => vis::WIDE_SAMPLES,
    };
    let samples = app.visualizer_samples(now, count);
    let sounding = samples.is_some();
    let dark = ui.visuals().dark_mode;
    // On the stage nothing is read over the scene, so colours stay at full
    // brightness and every effect is drawn more strongly.
    let colours = visualiser_palette(app, if stage { None } else { Some(dark) });
    let strength = if stage {
        STAGE_STRENGTH
    } else if dark {
        1.0
    } else {
        LIGHT_STRENGTH
    };
    let mut painter = ui.painter().with_clip_rect(rect);
    painter.multiply_opacity(opacity);
    // Audio still reacts when reduced motion is requested, but decorative
    // travel and rotation stay still.
    let time = if crate::motion::reduced(ui.ctx()) {
        0.0
    } else {
        ui.input(|input| input.time.rem_euclid(3600.0) as f32)
    };
    match mode {
        PlayerBarVis::Off => false,
        PlayerBarVis::Waveform | PlayerBarVis::Mirror => {
            if !sounding {
                return false;
            }
            let count = (rect.width() / 8.0).clamp(75.0, 320.0) as usize;
            let trace = vis::scope_line(samples.as_deref().unwrap_or_default(), count);
            if mode == PlayerBarVis::Mirror {
                mirror_wave(&painter, rect, &trace, &colours, strength);
            } else {
                waveform(&painter, rect, &trace, &colours, strength);
            }
            true
        }
        _ => {
            if !sounding && app.player_bar_analyser.settled() && app.player_bar_glide.settled() {
                return false;
            }
            let levels = app.player_bar_analyser.step(
                samples.as_deref().unwrap_or_default(),
                std::time::Instant::now(),
            );
            if mode == PlayerBarVis::Spectrum {
                let peaks = app.player_bar_analyser.peaks();
                spectrum(&painter, rect, &levels, &peaks, &colours, strength);
            } else {
                let levels = app
                    .player_bar_glide
                    .step(&levels, std::time::Instant::now());
                spectrum_effect(mode, &painter, rect, &levels, &colours, strength, time);
            }
            sounding || !app.player_bar_analyser.settled() || !app.player_bar_glide.settled()
        }
    }
}

/// The colours the visualisers are drawn in, from the bass to the treble:
/// the playing cover's, or the theme's, as Visualizer colours chooses. With
/// `behind_text` (whether the theme is dark) they are kept dim enough to read
/// words over; without, they are at full brightness, as on the stage and in
/// the beat pulse's soft glow.
pub(crate) fn visualiser_palette(app: &App, behind_text: Option<bool>) -> Vec<Color32> {
    let gradient_colours = app
        .theme_gradient()
        .map(|stops| stops.map(|colour| [colour.r(), colour.g(), colour.b()]));
    // Theme colours keep a gradient theme's own three colours, or the
    // accent, whatever is playing; pages may still follow the cover.
    let art_colours = match app.settings.visualizer_colours {
        crate::settings::VisColours::Auto => app.now_playing_palette(),
        crate::settings::VisColours::Theme => None,
    };
    let source_colours =
        art_colours.or_else(|| gradient_colours.as_ref().map(|colours| colours.as_slice()));
    match behind_text {
        Some(dark) => vis_colours(source_colours, app.palette.accent, dark),
        None => stage_colours(source_colours, app.palette.accent),
    }
}

/// The visualizer's colours, swept across from the bass to the treble. With
/// colour from album art they are the cover's own: its hues in the cover's
/// order, then its grey, drawn as a plain grey even when it is a near-black
/// with a cast, so the bass and its pulse carry a hue whenever the cover has
/// one. Otherwise they are the theme's accent for the bass, and the same
/// turned a sixth of the way round the colour wheel for the treble. Each hue
/// is made vivid, then all are kept to a band of
/// brightness, so the words the bars pass behind always stand out: no lighter
/// than [`DARK_CEILING`] under the dark theme's white text, no darker than
/// [`LIGHT_FLOOR`] under the light theme's dark text.
/// The visualizer's colours at full brightness, for the stage.
fn stage_colours(art: Option<&[[u8; 3]]>, accent: Color32) -> Vec<Color32> {
    use crate::images::has_hue;
    use egui::ecolor::Hsva;
    let keep = |mut colour: Hsva, vivid: bool| {
        colour.s = if vivid { colour.s.max(0.65) } else { 0.0 };
        colour.v = 1.0;
        colour.a = 1.0;
        Color32::from(colour)
    };
    if let Some(art) = art.filter(|art| !art.is_empty()) {
        let colours: Vec<_> = art
            .iter()
            .filter(|&&rgb| has_hue(rgb))
            .map(|&rgb| keep(Hsva::from_srgb(rgb), true))
            .collect();
        if colours.len() >= 2 {
            return colours;
        }
    }
    let low = Hsva::from(accent);
    let mut high = low;
    high.h = (high.h + 1.0 / 6.0).fract();
    let mut far = low;
    far.h = (far.h + 0.45).fract();
    vec![keep(low, true), keep(high, true), keep(far, true)]
}

/// The full-window visualiser stage: the chosen effect drawn across the
/// whole window in full colour, with the playing song along the bottom.
/// A click or Esc closes it.
pub fn stage(app: &mut App, ctx: &egui::Context) {
    if !app.visualiser_stage {
        return;
    }
    let rect = ctx.content_rect();
    let now = app.now_playing();
    egui::Area::new(egui::Id::new("visualiser-stage"))
        .order(egui::Order::Foreground)
        .fixed_pos(rect.min)
        .show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(rect.size(), Sense::click());
            let painter = ui.painter();
            painter.rect_filled(rect, 0, Color32::from_rgb(6, 6, 12));
            if app.settings.beat_pulse && !crate::motion::reduced(ui.ctx()) {
                let pulse = app.beat.pulse(ui.input(|input| input.time))
                    * app.settings.beat_pulse_strength.amount();
                if pulse > 0.01 {
                    // The visualiser's own colours, rising softly from the
                    // foot of the stage.
                    let colours = visualiser_palette(app, None);
                    let foot = rect.height() * 0.45;
                    let mut glow = egui::Mesh::default();
                    let columns = colours.len().max(1) * 6;
                    for column in 0..columns {
                        let (a, b) = (
                            column as f32 / columns as f32,
                            (column + 1) as f32 / columns as f32,
                        );
                        let index = glow.vertices.len() as u32;
                        let x = |at: f32| rect.left() + at * rect.width();
                        glow.colored_vertex(pos2(x(a), rect.bottom() - foot), Color32::TRANSPARENT);
                        glow.colored_vertex(pos2(x(b), rect.bottom() - foot), Color32::TRANSPARENT);
                        glow.colored_vertex(
                            pos2(x(b), rect.bottom()),
                            swept(&colours, b).gamma_multiply(0.22 * pulse),
                        );
                        glow.colored_vertex(
                            pos2(x(a), rect.bottom()),
                            swept(&colours, a).gamma_multiply(0.22 * pulse),
                        );
                        glow.add_triangle(index, index + 1, index + 2);
                        glow.add_triangle(index, index + 2, index + 3);
                    }
                    painter.add(egui::Shape::mesh(glow));
                }
            }
            if let Some(stops) = app.background_gradient() {
                let stops = stops.map(|stop| stop.gamma_multiply(0.18));
                theme::paint_theme_gradient(ui, rect, stops, 0);
            }
            // The effect fills the middle band; the scenes are made for a
            // wide strip, so it keeps a cinematic shape on any window.
            let band = Rect::from_center_size(
                rect.center(),
                vec2(rect.width(), (rect.width() * 0.42).min(rect.height() * 0.8)),
            );
            if visualizer(app, ui, band, now.as_ref(), true) {
                theme::visualiser_frame(ui.ctx(), app.visualiser_fps());
            }
            if let Some(now) = &now {
                let at = rect.left_bottom() + vec2(40.0, -40.0);
                painter.text(
                    at - vec2(0.0, 30.0),
                    egui::Align2::LEFT_BOTTOM,
                    &now.title,
                    theme::semibold(30.0),
                    Color32::WHITE,
                );
                painter.text(
                    at,
                    egui::Align2::LEFT_BOTTOM,
                    &now.subtitle,
                    theme::regular(17.0),
                    Color32::from_white_alpha(170),
                );
            }
            painter.text(
                rect.right_bottom() + vec2(-40.0, -40.0),
                egui::Align2::RIGHT_BOTTOM,
                gettext(app.locale, "Click or press Esc to close"),
                theme::regular(13.0),
                Color32::from_white_alpha(110),
            );
            if response.clicked() {
                app.actions.push(Action::ToggleVisualiserStage);
            }
        });
}

fn vis_colours(art: Option<&[[u8; 3]]>, accent: Color32, dark: bool) -> Vec<Color32> {
    use crate::images::has_hue;
    use egui::ecolor::Hsva;
    let keep = |mut colour: Hsva, vivid: bool| {
        colour.s = if vivid { colour.s.max(0.55) } else { 0.0 };
        colour.v = 1.0;
        colour.a = 1.0;
        within_brightness(Color32::from(colour), dark)
    };
    if let Some(art) = art.filter(|art| !art.is_empty()) {
        let colours = |vivid: bool| {
            art.iter()
                .filter(move |&&rgb| has_hue(rgb) == vivid)
                .map(move |&rgb| keep(Hsva::from_srgb(rgb), vivid))
        };
        return colours(true).chain(colours(false)).collect();
    }
    let low = Hsva::from(accent);
    let mut high = low;
    high.h = (high.h + 1.0 / 6.0).fract();
    vec![keep(low, true), keep(high, true)]
}

/// The colour `at` the given fraction of the way along `colours`, from the
/// first at 0 to the last at 1.
pub(crate) fn swept(colours: &[Color32], at: f32) -> Color32 {
    let Some(last) = colours.len().checked_sub(1) else {
        return Color32::TRANSPARENT;
    };
    if last == 0 {
        return colours[0];
    }
    let place = at.clamp(0.0, 1.0) * last as f32;
    let index = (place as usize).min(last - 1);
    colours[index].lerp_to_gamma(colours[index + 1], place - index as f32)
}

/// The most luminance a visualizer colour has under the dark theme, and the
/// band it is kept in under the light theme, as relative luminance from 0
/// to 1. On white the colour must be dark enough to show at all yet light
/// enough that dark words stay clear over it.
const DARK_CEILING: f32 = 0.35;
const LIGHT_FLOOR: f32 = 0.2;
const LIGHT_CEILING: f32 = 0.4;

/// `colour` with its hue kept and its brightness moved into the band the
/// theme's text reads against: darkened towards black when too light,
/// lightened towards white when too dark.
fn within_brightness(colour: Color32, dark: bool) -> Color32 {
    let linear = egui::Rgba::from(colour);
    let luminance = 0.2126 * linear.r() + 0.7152 * linear.g() + 0.0722 * linear.b();
    let ceiling = if dark { DARK_CEILING } else { LIGHT_CEILING };
    let adjusted = if luminance > ceiling {
        linear * (ceiling / luminance)
    } else if !dark && luminance < LIGHT_FLOOR {
        let toward_white = (LIGHT_FLOOR - luminance) / (1.0 - luminance).max(f32::EPSILON);
        egui::Rgba::from_rgb(
            linear.r() + (1.0 - linear.r()) * toward_white,
            linear.g() + (1.0 - linear.g()) * toward_white,
            linear.b() + (1.0 - linear.b()) * toward_white,
        )
    } else {
        linear
    };
    Color32::from(egui::Rgba::from_rgb(
        adjusted.r(),
        adjusted.g(),
        adjusted.b(),
    ))
}

/// Adds a rectangle shaded from `top` to `bottom`.
fn shaded(mesh: &mut egui::Mesh, rect: Rect, top: Color32, bottom: Color32) {
    let base = mesh.vertices.len() as u32;
    mesh.colored_vertex(rect.left_top(), top);
    mesh.colored_vertex(rect.right_top(), top);
    mesh.colored_vertex(rect.right_bottom(), bottom);
    mesh.colored_vertex(rect.left_bottom(), bottom);
    mesh.add_triangle(base, base + 1, base + 2);
    mesh.add_triangle(base, base + 2, base + 3);
}

/// Bars rising from the foot of the player bar, swept across the
/// visualizer's colours from the bass to the treble, with a soft glow, a
/// peak cap hanging above each, and the whole foot of the bar breathing with
/// the bass.
fn spectrum(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    peaks: &[f32],
    colours: &[Color32],
    strength: f32,
) {
    let bands = levels.len() as f32;
    let width = (rect.width() - SPECTRUM_GAP * (bands - 1.0)) / bands;
    // A full band reaches the top, with room above for its cap.
    let reach = rect.height() - PEAK_GAP - PEAK_HEIGHT - 1.0;
    let mut mesh = egui::Mesh::default();

    // The bass pulse: a glow along the foot, as strong as the low bands.
    let bass = levels.iter().take(levels.len() / 8).copied().sum::<f32>()
        / (levels.len() / 8).max(1) as f32;
    let pulse = Rect::from_min_max(pos2(rect.left(), rect.center().y), rect.right_bottom());
    shaded(
        &mut mesh,
        pulse,
        Color32::TRANSPARENT,
        swept(colours, 0.0).gamma_multiply(PULSE_ALPHA * strength * bass * bass),
    );

    for (index, (level, peak)) in levels.iter().zip(peaks).enumerate() {
        let left = rect.left() + index as f32 * (width + SPECTRUM_GAP);
        let colour = swept(colours, index as f32 / (bands - 1.0));
        let height = level * reach;
        if height >= 1.0 {
            let bar = Rect::from_min_max(
                pos2(left, rect.bottom() - height),
                pos2(left + width, rect.bottom()),
            );
            // The glow: the bar again, wider and fainter.
            shaded(
                &mut mesh,
                bar.expand2(vec2(SPECTRUM_GAP, 3.0)),
                colour.gamma_multiply(GLOW_ALPHA * strength * 0.3),
                colour.gamma_multiply(GLOW_ALPHA * strength),
            );
            let (foot, top) = (SPECTRUM_ALPHA.0 * strength, SPECTRUM_ALPHA.1 * strength);
            shaded(
                &mut mesh,
                bar,
                colour.gamma_multiply(foot + (top - foot) * level),
                colour.gamma_multiply(foot),
            );
        }
        let cap = peak * reach;
        if cap >= 2.0 {
            let y = rect.bottom() - cap - PEAK_GAP;
            shaded(
                &mut mesh,
                Rect::from_min_max(pos2(left, y - PEAK_HEIGHT), pos2(left + width, y)),
                colour.gamma_multiply(PEAK_ALPHA),
                colour.gamma_multiply(PEAK_ALPHA),
            );
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// The wave as a neon line in layers of glow, swept across the visualizer's
/// colours from the bass to the treble, over a faint shading down to the
/// midline and a dim mirrored echo.
fn waveform(
    painter: &egui::Painter,
    rect: Rect,
    trace: &[f32],
    colours: &[Color32],
    strength: f32,
) {
    if trace.len() < 2 {
        return;
    }
    let reach = rect.height() * 0.46;
    let step = rect.width() / (trace.len() - 1) as f32;
    let midline = rect.center().y;
    let point =
        |index: usize, value: f32| pos2(rect.left() + index as f32 * step, midline - value * reach);
    let colour_at = |index: usize| swept(colours, index as f32 / (trace.len() - 1) as f32);

    // The shading between the wave and the midline.
    let mut mesh = egui::Mesh::default();
    for index in 0..trace.len() - 1 {
        let colour = colour_at(index).gamma_multiply(WAVE_FILL_ALPHA * strength);
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(point(index, trace[index]), colour);
        mesh.colored_vertex(point(index + 1, trace[index + 1]), colour);
        mesh.colored_vertex(pos2(point(index + 1, 0.0).x, midline), Color32::TRANSPARENT);
        mesh.colored_vertex(pos2(point(index, 0.0).x, midline), Color32::TRANSPARENT);
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base, base + 2, base + 3);
    }
    painter.add(egui::Shape::mesh(mesh));

    // Short runs of line, so the colour can sweep along the wave.
    let run = 8;
    let stroke = |mirror: f32, width: f32, alpha: f32| {
        let mut start = 0;
        while start < trace.len() - 1 {
            let end = (start + run).min(trace.len() - 1);
            let points = (start..=end)
                .map(|index| point(index, trace[index] * mirror))
                .collect();
            let colour = colour_at((start + end) / 2).gamma_multiply(alpha);
            painter.add(egui::Shape::line(points, egui::Stroke::new(width, colour)));
            start = end;
        }
    };
    stroke(-0.6, 1.0, WAVE_ECHO_ALPHA);
    for (width, alpha) in WAVE_LAYERS {
        stroke(1.0, width, alpha);
    }
}

/// The new scenes share the existing analyser and its time-based decay.
/// Silence paints nothing, so paused playback settles without idle animation.
fn spectrum_effect(
    mode: crate::settings::PlayerBarVis,
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    use crate::settings::PlayerBarVis;
    if levels.is_empty() || !levels.iter().any(|level| *level > 0.0) {
        return;
    }
    match mode {
        PlayerBarVis::Waterfall => waterfall(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Prism => prism(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Helix => helix(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Synthwave => synthwave(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Terrain => terrain(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Lava => lava(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Galaxy => galaxy(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Matrix => matrix(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Spectrogram => spectrogram(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Orb => orb(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Bubbles => bubbles(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Skyline => skyline(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Ocean => ocean(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Led => led_meter(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Fireflies => fireflies(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Heartbeat => heartbeat(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Circuit => circuit(painter, rect, levels, colours, strength, time),
        _ => {}
    }
}

/// The bass, as one smooth number: the lowest bands carry the beat.
fn bass_level(levels: &[f32]) -> f32 {
    mean_level(&levels[..levels.len().min(8)])
}

/// A retro sunset: a striped sun that swells with the bass sits on a horizon
/// of spectrum mountains, over a neon grid rushing towards the listener.
fn synthwave(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = scene_energy(levels);
    let bass = bass_level(levels).sqrt();
    let horizon = rect.top() + rect.height() * 0.55;
    let sun_colour = |at: f32| swept(colours, at);
    // The sun: horizontal slices of a disc, with gaps that widen downwards.
    let centre = pos2(rect.center().x, horizon);
    let radius = rect.height() * (0.38 + bass * 0.12);
    let slices = 18;
    let mut sun = egui::Mesh::default();
    for slice in 0..slices {
        let top = slice as f32 / slices as f32;
        let gap = if slice > slices / 3 {
            (slice - slices / 3) as f32 / slices as f32 * 0.55
        } else {
            0.0
        };
        let bottom = (slice + 1) as f32 / slices as f32 - gap / slices as f32 * 3.0;
        if bottom <= top {
            continue;
        }
        let y0 = centre.y - radius + top * radius;
        let y1 = centre.y - radius + bottom * radius;
        let half = |y: f32| (radius * radius - (centre.y - y).powi(2)).max(0.0).sqrt();
        // Never past full opacity, which would wash the sun out to white.
        let colour = sun_colour(top).gamma_multiply((strength * (0.35 + energy * 0.45)).min(0.95));
        let base = sun.vertices.len() as u32;
        sun.colored_vertex(pos2(centre.x - half(y0), y0), colour);
        sun.colored_vertex(pos2(centre.x + half(y0), y0), colour);
        sun.colored_vertex(pos2(centre.x + half(y1), y1), colour);
        sun.colored_vertex(pos2(centre.x - half(y1), y1), colour);
        sun.add_triangle(base, base + 1, base + 2);
        sun.add_triangle(base, base + 2, base + 3);
    }
    painter.circle_filled(
        centre,
        radius * 0.95,
        sun_colour(0.2).gamma_multiply(0.06 * strength * energy),
    );
    painter.add(egui::Shape::mesh(sun));
    // Spectrum mountains along the horizon, mirrored from the middle.
    let ridge: Vec<_> = (0..=96)
        .map(|index| {
            let at = index as f32 / 96.0;
            let mirrored = (at - 0.5).abs() * 2.0;
            let level = scene_band(levels, mirrored, energy);
            pos2(
                rect.left() + at * rect.width(),
                horizon - level * rect.height() * 0.3 * (0.4 + mirrored * 0.6),
            )
        })
        .collect();
    scene_neon_line(
        painter,
        ridge,
        swept(colours, 0.7),
        0.55 * strength * energy,
    );
    // The grid: lines from the vanishing point, and rows rushing forward.
    let grid = swept(colours, 0.9);
    let floor = rect.bottom() - horizon;
    for column in -12..=12 {
        let x = rect.center().x + column as f32 * rect.width() / 24.0;
        painter.line_segment(
            [
                pos2(rect.center().x + column as f32 * 6.0, horizon),
                pos2(x, rect.bottom()),
            ],
            egui::Stroke::new(1.0, grid.gamma_multiply(0.35 * strength * energy)),
        );
    }
    let speed = 0.6 + energy * 1.4;
    for row in 0..8 {
        let depth = (row as f32 / 8.0 + time * speed * 0.25).fract();
        let y = horizon + (floor - 3.0).max(0.0) * depth * depth;
        painter.line_segment(
            [pos2(rect.left(), y), pos2(rect.right(), y)],
            egui::Stroke::new(
                1.0 + depth,
                grid.gamma_multiply(strength * energy * (0.15 + depth * 0.5)),
            ),
        );
    }
}

/// How many spectra the terrain remembers, and how often it takes one.
const TERRAIN_ROWS: usize = 22;
const TERRAIN_STEP: f32 = 1.0 / 24.0;

#[derive(Clone, Default)]
struct TerrainHistory {
    rows: std::collections::VecDeque<Vec<f32>>,
    taken_at: f32,
}

/// Ridges of recent spectra receding into the distance, newest at the front:
/// the music's last second as a landscape of glowing contour lines.
fn terrain(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let id = egui::Id::new("player-bar-terrain-history");
    let mut history = painter
        .ctx()
        .data(|data| data.get_temp::<TerrainHistory>(id))
        .unwrap_or_default();
    if history.rows.is_empty() || time - history.taken_at >= TERRAIN_STEP || time < history.taken_at
    {
        let energy = scene_energy(levels);
        let row: Vec<f32> = (0..48)
            .map(|index| scene_band(levels, index as f32 / 47.0, energy))
            .collect();
        // The first spectrum fills the whole landscape at once, rather than
        // the hills growing in row by row.
        if history.rows.is_empty() {
            history
                .rows
                .extend(std::iter::repeat_n(row.clone(), TERRAIN_ROWS - 1));
        }
        history.rows.push_front(row);
        history.rows.truncate(TERRAIN_ROWS);
        history.taken_at = time;
    }
    let energy = scene_energy(levels);
    // Far rows first, so nearer ridges are drawn over them.
    for (index, row) in history.rows.iter().enumerate().rev() {
        let depth = 1.0 - index as f32 / TERRAIN_ROWS as f32;
        let spread = 0.45 + depth * 0.55;
        let base = rect.top() + rect.height() * (0.2 + depth * 0.75);
        let lift = rect.height() * 0.55 * (0.35 + depth * 0.65);
        let points: Vec<_> = row
            .iter()
            .enumerate()
            .map(|(column, level)| {
                let at = column as f32 / (row.len() - 1) as f32;
                // Taper at both sides, like hills on the edge of a valley.
                let taper = (std::f32::consts::PI * at).sin().max(0.0).powf(0.6);
                let swell = (time * 0.9 + at * 7.0 + depth * 3.0).sin() * rect.height() * 0.012;
                pos2(
                    rect.center().x + (at - 0.5) * rect.width() * spread,
                    base - level * lift * taper + swell,
                )
            })
            .collect();
        let mut fill = egui::Mesh::default();
        let colour = swept(colours, 1.0 - depth);
        for point in &points {
            let base_index = fill.vertices.len() as u32;
            fill.colored_vertex(*point, colour.gamma_multiply(0.12 * strength * depth));
            fill.colored_vertex(pos2(point.x, base + 2.0), Color32::TRANSPARENT);
            if base_index > 0 {
                fill.add_triangle(base_index - 2, base_index, base_index + 1);
                fill.add_triangle(base_index - 2, base_index + 1, base_index - 1);
            }
        }
        painter.add(egui::Shape::mesh(fill));
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(
                0.8 + depth * 1.2,
                colour.gamma_multiply(strength * (0.12 + depth * 0.6) * energy.max(0.3)),
            ),
        ));
    }
    painter.ctx().data_mut(|data| data.insert_temp(id, history));
}

/// Soft glowing blobs drift up and down like wax in a lava lamp, swelling
/// with their part of the spectrum where they overlap and merge.
fn lava(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = scene_energy(levels);
    for blob in 0..11 {
        let seed = blob as f32;
        let at = (seed * 0.618_034 + 0.1).fract();
        let level = scene_band(levels, at, energy);
        let rise = ((time * (0.08 + at * 0.06) + seed * 0.37) * std::f32::consts::TAU).sin();
        // Room at both ends for the widest glow, so no blob leaves the bar.
        let margin = rect.height() * 0.4 + rect.width() * 0.02;
        let centre = pos2(
            rect.left()
                + margin
                + at * (rect.width() - margin * 2.0).max(0.0)
                + (time * 0.2 + seed).sin() * rect.width() * 0.02,
            rect.center().y + rise * rect.height() * 0.18,
        );
        let radius = rect.height() * (0.11 + level * 0.15);
        let colour = swept(colours, at);
        // Layered glows read as one gooey shape where blobs meet.
        for (scale, alpha) in [(1.5, 0.06), (1.2, 0.12), (1.0, 0.24), (0.65, 0.32)] {
            painter.circle_filled(
                centre,
                radius * scale,
                colour.gamma_multiply(alpha * strength * (0.4 + energy * 0.6)),
            );
        }
    }
}

/// Columns of glowing glyph-like marks fall like digital rain, each column
/// falling faster and brighter with its own part of the spectrum.
fn matrix(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = scene_energy(levels);
    let columns = ((rect.width() / 16.0) as usize).clamp(12, 110);
    let width = rect.width() / columns as f32;
    let cell = vec2((width * 0.5).clamp(2.0, 7.0), 5.0);
    // One mesh for every mark: thousands of small shapes cost far more.
    let mut mesh = egui::Mesh::default();
    for column in 0..columns {
        let seed = column as f32;
        let at = (seed * 0.618_034).fract();
        let level = scene_band(levels, at, energy);
        let speed = 0.25 + level * 0.9;
        let travel = rect.height() * 1.6;
        let head = rect.top() - rect.height() * 0.3
            + ((time * speed * 0.5 + seed * 0.37).fract()) * travel;
        let x = rect.left() + (column as f32 + 0.5) * width;
        let colour = swept(colours, at);
        for trail in 0..10 {
            let y = head - trail as f32 * 8.0;
            if y < rect.top() + cell.y || y > rect.bottom() - cell.y {
                continue;
            }
            let fade = 1.0 - trail as f32 / 10.0;
            // The leading mark burns almost white.
            let mark = if trail == 0 {
                motion_blend(colour, Color32::WHITE, 0.6)
            } else {
                colour
            };
            // Glyph-like: some marks are tall, some short and wide.
            let shape =
                if ((seed + trail as f32 * 3.7 + (time * 4.0).floor()) * 1.618).fract() > 0.5 {
                    vec2(cell.x, cell.y)
                } else {
                    vec2(cell.x * 1.4, cell.y * 0.6)
                };
            mesh.add_colored_rect(
                Rect::from_center_size(pos2(x, y), shape),
                mark.gamma_multiply(strength * fade * (0.15 + level * 0.75) * energy.max(0.3)),
            );
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

fn motion_blend(from: Color32, to: Color32, amount: f32) -> Color32 {
    theme::motion::blend(from, to, amount)
}

/// How many moments the spectrogram keeps, and how often it takes one.
const SPECTROGRAM_COLUMNS: usize = 64;
const SPECTROGRAM_STEP: f32 = 1.0 / 30.0;
const SPECTROGRAM_BANDS: usize = 24;

#[derive(Clone, Default)]
struct SpectrogramHistory {
    columns: std::collections::VecDeque<Vec<f32>>,
    taken_at: f32,
}

/// A scrolling heat map: time runs from right to left, low notes at the
/// bottom and high ones at the top, each cell glowing with its loudness.
fn spectrogram(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let id = egui::Id::new("player-bar-spectrogram-history");
    let mut history = painter
        .ctx()
        .data(|data| data.get_temp::<SpectrogramHistory>(id))
        .unwrap_or_default();
    let energy = scene_energy(levels);
    if history.columns.is_empty()
        || time - history.taken_at >= SPECTROGRAM_STEP
        || time < history.taken_at
    {
        let column: Vec<f32> = (0..SPECTROGRAM_BANDS)
            .map(|band| {
                // The loudest of the bands this row covers, so a single
                // sounding note is never missed between rows.
                let from = band * levels.len() / SPECTROGRAM_BANDS;
                let to = ((band + 1) * levels.len() / SPECTROGRAM_BANDS).max(from + 1);
                let loudest = levels[from..to.min(levels.len())]
                    .iter()
                    .copied()
                    .fold(0.0_f32, f32::max);
                (loudest.sqrt() * 0.8 + energy * 0.2).min(1.0)
            })
            .collect();
        if history.columns.is_empty() {
            history
                .columns
                .extend(std::iter::repeat_n(column.clone(), SPECTROGRAM_COLUMNS - 1));
        }
        history.columns.push_front(column);
        history.columns.truncate(SPECTROGRAM_COLUMNS);
        history.taken_at = time;
    }
    let cell = vec2(
        rect.width() / SPECTROGRAM_COLUMNS as f32,
        rect.height() / SPECTROGRAM_BANDS as f32,
    );
    let mut mesh = egui::Mesh::default();
    for (age, column) in history.columns.iter().enumerate() {
        let x = rect.right() - (age + 1) as f32 * cell.x;
        let fade = 1.0 - age as f32 / SPECTROGRAM_COLUMNS as f32 * 0.6;
        for (band, level) in column.iter().enumerate() {
            let shimmer = 0.92 + 0.08 * (time * 3.0 + band as f32 + age as f32 * 0.3).sin();
            let alpha = strength * level * level * fade * shimmer * 0.85;
            if alpha < 0.01 {
                continue;
            }
            let at = band as f32 / (SPECTROGRAM_BANDS - 1) as f32;
            let colour = swept(colours, at).gamma_multiply(alpha);
            let y = rect.bottom() - (band + 1) as f32 * cell.y;
            mesh.add_colored_rect(
                Rect::from_min_size(pos2(x, y), cell - vec2(1.0, 1.0)),
                colour,
            );
        }
    }
    painter.add(egui::Shape::mesh(mesh));
    painter.ctx().data_mut(|data| data.insert_temp(id, history));
}

/// A spinning wireframe globe that swells with the bass, its lines bulging
/// with the spectrum, inside a tilted ring of light like a planet's.
fn orb(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    use std::f32::consts::{PI, TAU};
    let energy = scene_energy(levels);
    let bass = bass_level(levels).sqrt();
    let centre = rect.center();
    let radius = rect.height() * (0.3 + bass * 0.1);
    let spin = time * 0.6;
    let tilt = 0.45_f32;
    // Project a point on the sphere: turn about the vertical, then tip it.
    let project = |latitude: f32, longitude: f32, swell: f32| {
        let r = radius * (1.0 + swell);
        let x = latitude.cos() * (longitude + spin).cos();
        let y = latitude.sin();
        let z = latitude.cos() * (longitude + spin).sin();
        let y_tilted = y * tilt.cos() - z * tilt.sin();
        let depth = y * tilt.sin() + z * tilt.cos();
        (centre + vec2(x * r, -y_tilted * r), depth)
    };
    for ring in 1..8 {
        let latitude = -PI / 2.0 + ring as f32 * PI / 8.0;
        let at = ring as f32 / 8.0;
        let points: Vec<_> = (0..=48)
            .map(|step| {
                let longitude = step as f32 / 48.0 * TAU;
                let swell = level_at(levels, (step as f32 / 48.0 + at).fract()) * 0.08;
                project(latitude, longitude, swell).0
            })
            .collect();
        neon_line(painter, points, swept(colours, at), 0.7 * strength * energy);
    }
    for meridian in 0..12 {
        let longitude = meridian as f32 / 12.0 * TAU;
        let at = meridian as f32 / 12.0;
        let points: Vec<_> = (0..=24)
            .map(|step| {
                let latitude = -PI / 2.0 + step as f32 / 24.0 * PI;
                let swell = level_at(levels, step as f32 / 24.0) * 0.1;
                project(latitude, longitude, swell)
            })
            .map(|(point, _)| point)
            .collect();
        neon_line(
            painter,
            points,
            swept(colours, at),
            0.45 * strength * energy,
        );
    }
    // The ring: wide and flat, its light following the spectrum around it.
    let ring = vec2(rect.width() * 0.46, rect.height() * 0.16);
    let points: Vec<_> = (0..=120)
        .map(|step| {
            let angle = step as f32 / 120.0 * TAU + time * 0.2;
            let lift = level_at(levels, step as f32 / 120.0) * rect.height() * 0.06;
            centre + vec2(angle.cos() * ring.x, angle.sin() * (ring.y + lift))
        })
        .collect();
    scene_neon_line(
        painter,
        points,
        swept(colours, 0.6),
        0.5 * strength * energy,
    );
}

/// Bubbles rise from the bottom of the bar, swelling with their part of the
/// spectrum and fading as they reach the top.
fn bubbles(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = scene_energy(levels);
    let largest = rect.height() * 0.17;
    for bubble in 0..44 {
        let seed = bubble as f32;
        let at = (seed * 0.618_034).fract();
        let level = scene_band(levels, at, energy);
        let radius = rect.height() * (0.04 + level * 0.13);
        let rise = (time * (0.08 + at * 0.1) + seed * 0.37).fract();
        let margin = largest + 2.0;
        let centre = pos2(
            rect.left()
                + margin
                + at * (rect.width() - margin * 2.0).max(0.0)
                + (time * 1.3 + seed).sin() * 4.0,
            rect.bottom() - margin - rise * (rect.height() - margin * 2.0).max(0.0),
        );
        let fade = (std::f32::consts::PI * rise).sin();
        let colour =
            swept(colours, at).gamma_multiply(strength * fade * (0.3 + level * 0.7) * energy);
        painter.circle_filled(centre, radius, colour.gamma_multiply(0.12));
        painter.circle_stroke(centre, radius, egui::Stroke::new(1.2, colour));
        // A bright highlight, up and to the left.
        painter.circle_filled(
            centre + vec2(-radius * 0.35, -radius * 0.35),
            (radius * 0.2).max(0.6),
            colour.gamma_multiply(1.2),
        );
    }
}

/// A spiral galaxy: three arms of stars turning around a core that flares
/// with the bass, each star lit by the band at its distance from the centre.
fn galaxy(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    use std::f32::consts::TAU;
    let energy = scene_energy(levels);
    let bass = bass_level(levels).sqrt();
    let centre = rect.center();
    let scale = vec2(rect.width() * 0.46, rect.height() * 0.46);
    let core = rect.height() * (0.08 + bass * 0.1);
    painter.circle_filled(
        centre,
        core * 1.8,
        swept(colours, 0.0).gamma_multiply(0.06 * strength * energy),
    );
    painter.circle_filled(
        centre,
        core,
        swept(colours, 0.0).gamma_multiply(0.45 * strength * energy),
    );
    for star in 0..280 {
        let seed = star as f32;
        let along = (seed * 0.618_034).fract().powf(0.8);
        let arm = star % 3;
        let jitter = ((seed * 12.9898).sin() * 43_758.547).fract() - 0.5;
        let angle = arm as f32 / 3.0 * TAU + along * 4.2 + time * 0.18 + jitter * 0.5;
        let level = scene_band(levels, along, energy);
        let distance = along * (1.0 + jitter * 0.12) * (0.92 + level * 0.12);
        let place = centre + vec2(angle.cos() * scale.x, angle.sin() * scale.y) * distance;
        if !rect.shrink(4.0).contains(place) {
            continue;
        }
        let colour =
            swept(colours, along).gamma_multiply(strength * (0.25 + level * 0.75) * energy);
        let size = 0.6 + level * 1.8 * (1.0 - along * 0.5);
        if level > 0.3 {
            painter.circle_filled(place, size * 2.2, colour.gamma_multiply(0.12));
        }
        painter.circle_filled(place, size, colour);
    }
}

/// Interpolate the analyser's bands rather than snapping geometry between them.
fn level_at(levels: &[f32], at: f32) -> f32 {
    let Some(last) = levels.len().checked_sub(1) else {
        return 0.0;
    };
    let place = at.clamp(0.0, 1.0) * last as f32;
    let index = place as usize;
    let low = levels[index];
    let high = levels[(index + 1).min(last)];
    (low + (high - low) * (place - index as f32)).clamp(0.0, 1.0)
}

fn mean_level(levels: &[f32]) -> f32 {
    if levels.is_empty() {
        0.0
    } else {
        (levels.iter().sum::<f32>() / levels.len() as f32).clamp(0.0, 1.0)
    }
}

fn neon_line(painter: &egui::Painter, points: Vec<egui::Pos2>, colour: Color32, alpha: f32) {
    painter.add(egui::Shape::line(
        points.clone(),
        egui::Stroke::new(6.0, colour.gamma_multiply(alpha * 0.12)),
    ));
    painter.add(egui::Shape::line(
        points,
        egui::Stroke::new(1.25, colour.gamma_multiply(alpha)),
    ));
}

/// A narrow musical peak should light a scene without needing every FFT
/// band to be full. This remains zero for silence and decays with the analyser.
fn scene_energy(levels: &[f32]) -> f32 {
    let peak = levels
        .iter()
        .copied()
        .fold(0.0_f32, f32::max)
        .clamp(0.0, 1.0);
    mean_level(levels).sqrt().max(peak.sqrt() * 0.8)
}

/// Keep frequency contrast while letting the real overall pulse support
/// the rest of the composition when only a few bands are sounding.
fn scene_band(levels: &[f32], at: f32, energy: f32) -> f32 {
    (level_at(levels, at).sqrt() * 0.8 + energy * 0.2).min(1.0)
}

fn scene_neon_line(painter: &egui::Painter, points: Vec<egui::Pos2>, colour: Color32, alpha: f32) {
    for (width, glow) in [(10.0, 0.08), (5.0, 0.2), (1.9, 1.0)] {
        painter.add(egui::Shape::line(
            points.clone(),
            egui::Stroke::new(width, colour.gamma_multiply(alpha * glow)),
        ));
    }
}

/// Frequency tiles cascade in offset columns, with soft trails behind each
/// head. This uses the current spectrum; it does not invent audio history.
fn waterfall(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = scene_energy(levels);
    let columns = 32;
    let width = rect.width() / columns as f32;
    let mut mesh = egui::Mesh::default();
    for column in 0..columns {
        let at = (column as f32 + 0.5) / columns as f32;
        let band = scene_band(levels, at, energy);
        let head = (time * 0.28 + at * 1.618).fract();
        let colour = swept(colours, at);
        for tile in 0..7 {
            let trail = tile as f32 / 7.0;
            let y = (head - trail).rem_euclid(1.0);
            let center = pos2(
                rect.left() + at * rect.width(),
                rect.top() + y * rect.height(),
            );
            let size = vec2((width - 2.0).max(1.0), rect.height() * (0.04 + band * 0.07));
            let cell = Rect::from_center_size(center, size);
            let alpha = (1.0 - trail).powf(1.4) * (0.42 + band * 0.55) * energy * strength;
            shaded(
                &mut mesh,
                cell,
                colour.gamma_multiply(alpha * 0.25),
                colour.gamma_multiply(alpha),
            );
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

fn triangle(mesh: &mut egui::Mesh, points: [egui::Pos2; 3], colours: [Color32; 3]) {
    let base = mesh.vertices.len() as u32;
    for (point, colour) in points.into_iter().zip(colours) {
        mesh.colored_vertex(point, colour);
    }
    mesh.add_triangle(base, base + 1, base + 2);
}

/// An interlocking cut-glass skyline. Each crystal has independently lit
/// facets and tips driven by neighbouring frequency bands.
fn prism(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = scene_energy(levels);
    let mut mesh = egui::Mesh::default();
    for crystal in 0..20 {
        let at = (crystal as f32 + 0.5) / 20.0;
        let band = scene_band(levels, at, energy);
        let x = rect.left() + rect.width() * at;
        let half = rect.width() / 40.0;
        let reach = rect.height() * (0.16 + band * 0.29);
        let glint = (time * 0.65 + crystal as f32 * 0.8).sin() * 0.5 + 0.5;
        let axis = rect.center().y + (crystal as f32 * 1.7).sin() * rect.height() * 0.04;
        let top = pos2(x, axis - reach);
        let bottom = pos2(x, axis + reach);
        let left = pos2(x - half, axis);
        let right = pos2(x + half, axis);
        let seam = pos2(x + half * (glint - 0.5) * 0.6, axis);
        let colour = swept(colours, at);
        let lit = colour.gamma_multiply(strength * energy * (0.18 + glint * 0.32));
        let shadow = colour.gamma_multiply(strength * energy * 0.08);
        triangle(&mut mesh, [top, left, seam], [lit, shadow, shadow]);
        triangle(&mut mesh, [top, seam, right], [lit, lit, shadow]);
        triangle(&mut mesh, [bottom, left, seam], [shadow, shadow, lit]);
        triangle(&mut mesh, [bottom, seam, right], [lit, shadow, shadow]);
        painter.add(egui::Shape::closed_line(
            vec![top, left, bottom, right],
            egui::Stroke::new(1.2, colour.gamma_multiply(0.55 * strength * energy)),
        ));
        painter.line_segment(
            [top, seam],
            egui::Stroke::new(1.0, colour.gamma_multiply(0.95 * energy * strength)),
        );
        painter.line_segment(
            [seam, bottom],
            egui::Stroke::new(1.0, colour.gamma_multiply(0.65 * energy * strength)),
        );
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// Two twisting strands with depth-shaded rungs. Bass widens the double
/// helix while the spectrum changes the separation all along its length.
fn helix(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = scene_energy(levels);
    let mut first = Vec::with_capacity(97);
    let mut second = Vec::with_capacity(97);
    for index in 0..=96 {
        let at = index as f32 / 96.0;
        let phase = at * std::f32::consts::TAU * 2.5 - time * 0.9;
        let reach =
            rect.height() * (0.22 + scene_band(levels, at, energy) * 0.20) * (0.65 + energy * 0.35);
        let x = rect.left() + at * rect.width();
        let a = pos2(x, rect.center().y + phase.sin() * reach);
        let b = pos2(x, rect.center().y - phase.sin() * reach);
        first.push(a);
        second.push(b);
        if index % 3 == 0 {
            let colour = swept(colours, at);
            let depth = phase.cos() * 0.5 + 0.5;
            painter.line_segment(
                [a, b],
                egui::Stroke::new(
                    1.3 + depth,
                    colour.gamma_multiply(strength * energy * (0.18 + depth * 0.48)),
                ),
            );
            painter.circle_filled(
                a,
                1.8 + depth,
                colour.gamma_multiply(0.95 * strength * energy),
            );
            painter.circle_filled(
                b,
                2.8 - depth,
                colour.gamma_multiply(0.78 * strength * energy),
            );
        }
    }
    for start in (0..96).step_by(8) {
        let end = start + 8;
        let colour = swept(colours, (start + 4) as f32 / 96.0);
        scene_neon_line(
            painter,
            first[start..=end].to_vec(),
            colour,
            0.98 * strength * energy,
        );
        scene_neon_line(
            painter,
            second[start..=end].to_vec(),
            colour,
            0.80 * strength * energy,
        );
    }
}

/// A symmetric, filled neon wave. Both contours come directly from the
/// PCM trace, so the central ribbon opens and closes with the song.
fn mirror_wave(
    painter: &egui::Painter,
    rect: Rect,
    trace: &[f32],
    colours: &[Color32],
    strength: f32,
) {
    if trace.len() < 2 {
        return;
    }
    let mut mesh = egui::Mesh::default();
    let mut upper = Vec::with_capacity(trace.len());
    let mut lower = Vec::with_capacity(trace.len());
    for (index, sample) in trace.iter().enumerate() {
        let at = index as f32 / (trace.len() - 1) as f32;
        let reach = sample.abs() * rect.height() * 0.44;
        let top = pos2(rect.left() + at * rect.width(), rect.center().y - reach);
        let bottom = pos2(top.x, rect.center().y + reach);
        upper.push(top);
        lower.push(bottom);
        let colour = swept(colours, at).gamma_multiply(0.14 * strength);
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(top, colour);
        mesh.colored_vertex(bottom, colour);
        if index > 0 {
            mesh.add_triangle(base - 2, base, base + 1);
            mesh.add_triangle(base - 2, base + 1, base - 1);
        }
    }
    painter.add(egui::Shape::mesh(mesh));
    // Keep the colour sweep in short runs, just like the original scope.
    for start in (0..trace.len() - 1).step_by(12) {
        let end = (start + 12).min(trace.len() - 1);
        let colour = swept(
            colours,
            (start + end) as f32 * 0.5 / (trace.len() - 1) as f32,
        );
        neon_line(
            painter,
            upper[start..=end].to_vec(),
            colour,
            0.65 * strength,
        );
        neon_line(
            painter,
            lower[start..=end].to_vec(),
            colour,
            0.65 * strength,
        );
    }
}

/// Ease the final fill's RGB. Untinted custom panels keep their alpha while
/// the colour returns to the panel colour.
fn eased_fill(ctx: &egui::Context, panel: Color32, tint: Option<Color32>) -> Color32 {
    let target = tint.map_or(panel, |tint| super::blend(panel, tint, TINT_STRENGTH));
    // Color32 stores premultiplied RGB, so interpolate unmultiplied channels.
    let [r, g, b, _] = target.to_srgba_unmultiplied();
    // A new pass after sign-out gets new ids without clearing other animations.
    let pass = ctx.cumulative_pass_nr();
    let session = ctx.data_mut(|data| {
        *data.get_temp_mut_or_insert_with(egui::Id::new(TINT_SESSION_ID), || pass)
    });
    let channel = |axis: &'static str, value: u8| {
        crate::motion::animate_value(
            ctx,
            egui::Id::new(("player-bar-tint", session, axis)),
            f32::from(value),
            crate::motion::seconds(ctx, TINT_FADE_SECONDS),
        )
        .round() as u8
    };
    let eased = [channel("r", r), channel("g", g), channel("b", b)];
    if eased == [r, g, b] {
        target
    } else {
        Color32::from_rgba_unmultiplied(eased[0], eased[1], eased[2], target.a())
    }
}

fn now_playing_block(app: &mut App, ui: &mut egui::Ui, region: Rect, now: Option<&NowPlaying>) {
    let palette = app.palette;
    let cy = region.center().y;
    let cover_rect = Rect::from_min_size(pos2(region.left() + 4.0, cy - 28.0), Vec2::splat(56.0));

    let Some(now) = now else {
        super::widgets::paint_cover(ui, &palette, None, cover_rect, 6.0, Icon::Music, None);
        let text_left = cover_rect.right() + 12.0;
        let text_rect = Rect::from_min_size(
            pos2(text_left, cy - 17.0),
            vec2((region.right() - text_left - 8.0).max(40.0), 34.0),
        );
        let mut text_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(text_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        text_ui.spacing_mut().item_spacing.y = 2.0;
        theme::text(
            &mut text_ui,
            gettext(app.locale, "Nothing playing"),
            theme::medium(14.0),
            palette.secondary,
        );
        theme::text(
            &mut text_ui,
            gettext(app.locale, "Pick a song, album, or playlist"),
            theme::regular(12.0),
            palette.dim,
        );
        return;
    };

    let shown_cover = super::flair::moving_art(app, ui, cover_rect, "player-bar", now.playing);
    super::widgets::playing_cover(
        ui,
        &palette,
        app.backend.art(),
        "player-bar",
        app.small_art(now, cover_rect.width()),
        shown_cover,
        6.0,
    );
    let song = app.now_playing_item();
    let drag_sense = if song.is_some() {
        Sense::click_and_drag()
    } else {
        Sense::click()
    };
    let cover_response = ui
        .interact(cover_rect, egui::Id::new("now-playing-cover"), drag_sense)
        .on_hover_cursor(if song.is_some() {
            egui::CursorIcon::Grab
        } else {
            egui::CursorIcon::Default
        });
    // Hovering the cover offers to dock the art large at the sidebar's
    // bottom, the way Spotify expands it. (#92)
    let art_available = now.art_url.is_some() || now.art_small.is_some();
    let expand_rect = Rect::from_center_size(
        pos2(cover_rect.right() - 10.0, cover_rect.top() + 10.0),
        Vec2::splat(18.0),
    );
    let offer_expand = art_available && !app.settings.art_expanded && app.settings.sidebar_visible;
    let over_expand = offer_expand && ui.rect_contains_pointer(expand_rect);
    if cover_response.clicked() && !over_expand {
        if let Some(id) = &now.album_id {
            app.actions.push(Action::Open(Page::Album(id.clone())));
        } else if let Some(id) = &now.show_id {
            app.actions.push(Action::Open(Page::Show(id.clone())));
        }
    }
    if offer_expand && (cover_response.hovered() || over_expand) {
        let expand = ui.interact(
            expand_rect,
            egui::Id::new("now-playing-art-expand"),
            Sense::click(),
        );
        ui.painter()
            .circle_filled(expand_rect.center(), 9.0, palette.panel.gamma_multiply(0.9));
        Icon::ChevronUp.image(palette.text, 12.0).paint_at(
            ui,
            Rect::from_center_size(expand_rect.center(), Vec2::splat(12.0)),
        );
        if expand.clicked() {
            app.settings.art_expanded = true;
            app.actions.push(Action::SettingsChanged);
        }
    }
    let heart_width = if now.is_episode { 0.0 } else { 42.0 };
    let text_left = cover_rect.right() + 12.0;
    let text_width = (region.right() - text_left - heart_width).max(40.0);
    let displayed_text_width: f32 = {
        let title =
            ui.painter()
                .layout_no_wrap(now.title.clone(), theme::medium(14.0), palette.text);
        let subtitle = ui.painter().layout_no_wrap(
            now.subtitle.clone(),
            theme::regular(12.0),
            palette.secondary,
        );
        title.size().x.max(subtitle.size().x).min(text_width)
    };
    let info_rect = Rect::from_min_size(pos2(text_left, cy - 18.0), vec2(text_width, 36.0));

    let mut text_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(info_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    text_ui.set_clip_rect(info_rect.intersect(ui.clip_rect()));
    text_ui.spacing_mut().item_spacing.y = 2.0;
    let title_response = theme::link(&mut text_ui, &now.title, theme::medium(14.0), palette.text)
        .interact(drag_sense);
    if title_response.clicked() {
        if let Some(id) = &now.album_id {
            app.actions.push(Action::Open(Page::Album(id.clone())));
        } else if let Some(id) = &now.show_id {
            app.actions.push(Action::Open(Page::Show(id.clone())));
        }
    }
    let mut responses = vec![cover_response, title_response];
    text_ui.horizontal_top(|ui| {
        if now.artists.is_empty() {
            let response = theme::link(ui, &now.subtitle, theme::regular(12.0), palette.secondary)
                .interact(drag_sense);
            if response.clicked()
                && let Some(id) = &now.show_id
            {
                app.actions.push(Action::Open(Page::Show(id.clone())));
            }
            responses.push(response);
        } else {
            ui.spacing_mut().item_spacing.x = 0.0;
            for (index, artist) in now.artists.iter().enumerate() {
                if index > 0 {
                    theme::text(ui, ", ", theme::regular(12.0), palette.secondary);
                }
                let response = if artist.id.is_some() {
                    theme::link(ui, &artist.name, theme::regular(12.0), palette.secondary)
                } else {
                    theme::text(ui, &artist.name, theme::regular(12.0), palette.secondary)
                }
                .interact(drag_sense);
                if response.clicked()
                    && let Some(id) = &artist.id
                {
                    app.actions.push(Action::Open(Page::Artist(id.clone())));
                }
                responses.push(response);
            }
        }
    });
    if responses
        .iter()
        .any(|response| response.drag_started_by(egui::PointerButton::Primary))
        && let Some(item) = &song
    {
        egui::DragAndDrop::set_payload(
            ui.ctx(),
            DragTrack {
                title: item.name().to_string(),
                image: app.item_image(item, 64),
                items: vec![item.clone()],
                from: None,
                source_playlist: None,
            },
        );
    }

    // The playing thing answers the same right-click menu as a table row,
    // from the cover or the words.
    if let Some(item) = &song {
        let context = app.editable_context_playlist();
        for response in &responses {
            egui::Popup::context_menu(response)
                .frame(super::widgets::menu_frame(&palette))
                .show(|ui| super::widgets::item_menu(ui, app, item, context.as_ref(), None));
        }
    }

    if !now.is_episode {
        let saved = app.is_saved(&now.uri).unwrap_or(false);
        // A liked heart stays green under the pointer, like shuffle.
        let (icon, color, hover, tooltip) = if saved {
            (
                Icon::HeartFilled,
                palette.accent,
                palette.accent_hover,
                gettext(app.locale, "Add to playlist"),
            )
        } else {
            (
                Icon::Heart,
                palette.secondary,
                palette.text,
                gettext(app.locale, "Save to Liked Songs"),
            )
        };
        // Sit the heart just past the actual text, not at the region's far
        // edge, so it stays visually attached to the title.
        let heart_x = (text_left + displayed_text_width + 21.0).min(region.right() - 21.0);
        let heart_rect = Rect::from_center_size(pos2(heart_x, cy), Vec2::splat(30.0));
        let mut heart_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(heart_rect)
                .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
        );
        let response = theme::icon_button(&mut heart_ui, icon, 17.0, color, hover, &tooltip);
        let popup_id = egui::Id::new(LIKED_POPUP_ID);
        // A liked song's heart opens Add to playlist, as Spotify's does,
        // with Remove from Liked Songs below the playlists (#792). The first
        // click on a song not yet liked still likes it at once.
        match song.as_ref().filter(|_| saved) {
            None => {
                egui::Popup::close_id(ui.ctx(), popup_id);
                if response.clicked() {
                    app.actions.push(Action::ToggleSaved(now.uri.clone()));
                }
            }
            Some(item) => {
                let query_id = popup_id.with("query");
                let shown = egui::Popup::menu(&response)
                    .id(popup_id)
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                    .frame(super::widgets::menu_frame(&palette))
                    .show(|ui| {
                        let stored = ui.data(|data| data.get_temp::<String>(query_id));
                        let fresh = stored.is_none();
                        let mut query = stored.unwrap_or_default();
                        let before = app.actions.len();
                        let field = super::widgets::playlist_picker(
                            ui,
                            app,
                            std::slice::from_ref(item),
                            &mut query,
                        );
                        if fresh {
                            field.request_focus();
                        }
                        ui.data_mut(|data| data.insert_temp(query_id, query));
                        super::widgets::menu_separator(ui, &palette);
                        if super::widgets::menu_item(
                            ui,
                            &palette,
                            Some(Icon::HeartFilled),
                            &gettext(app.locale, "Remove from Liked Songs"),
                        ) {
                            app.actions.push(Action::ToggleSaved(now.uri.clone()));
                        }
                        // A playlist, New playlist, or Remove closes it.
                        if app.actions.len() > before {
                            ui.close();
                        }
                    });
                if shown.is_none() {
                    ui.data_mut(|data| data.remove::<String>(query_id));
                }
            }
        }
    }
}

/// The transport's button widths: icon buttons occupy icon size + 12; the
/// disc is 36.
const TRANSPORT_WIDTHS: [f32; 5] = [29.0, 30.0, 36.0, 30.0, 29.0];
const TRANSPORT_GAP: f32 = 10.0;

/// The transport's buttons from Shuffle to Repeat, centred in the bar.
fn transport_width() -> f32 {
    TRANSPORT_WIDTHS.iter().sum::<f32>() + TRANSPORT_GAP * 4.0
}

fn transport(app: &mut App, ui: &mut egui::Ui, now: Option<&NowPlaying>, region: Rect) {
    let palette = app.palette;
    // Everything here is placed with explicit rects: egui's implicit rows
    // centre each widget in the row height known when it is added, which
    // left earlier icons riding high next to the play disc.
    //
    // The buttons row (36) and the progress row (~15, after a 6px gap) form
    // one cluster, centred as a group in the 88px bar: the buttons sit 8px
    // above the bar's midline and the progress row 23px below it. Measured
    // on screen this puts equal breathing room above and beneath the
    // cluster.
    let cy = region.center().y - 8.0;
    let enabled = now.is_some_and(|now| now.can_control) || app.is_connected();
    let playing = now.is_some_and(|now| now.playing);
    let loading = now.is_some_and(|now| now.loading);
    let shuffle = now.map_or_else(|| app.playing_context_shuffle(), |now| now.shuffle);
    let repeat = now.map(|now| now.repeat).unwrap_or_default();
    let dim = if enabled {
        palette.secondary
    } else {
        palette.dim
    };

    let widths = TRANSPORT_WIDTHS;
    let mut x = region.center().x - transport_width() / 2.0;
    let mut slot = |width: f32| {
        let rect = Rect::from_center_size(pos2(x + width / 2.0, cy), vec2(width, 36.0));
        x += width + TRANSPORT_GAP;
        rect
    };
    let centered = |ui: &mut egui::Ui, rect: Rect| {
        ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
        )
    };

    let shuffle_color = if shuffle { palette.accent } else { dim };
    let mut cell = centered(ui, slot(widths[0]));
    let shuffle_button = theme::icon_button(
        &mut cell,
        Icon::Shuffle,
        17.0,
        shuffle_color,
        if shuffle {
            palette.accent_hover
        } else {
            palette.text
        },
        &gettext(app.locale, "Shuffle"),
    );
    shuffle_button.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::Checkbox,
            cell.is_enabled(),
            shuffle,
            gettext(app.locale, "Shuffle"),
        )
    });
    if shuffle_button.clicked() {
        app.actions.push(Action::ToggleShuffle);
    }

    let mut cell = centered(ui, slot(widths[1]));
    if theme::icon_button(
        &mut cell,
        Icon::SkipBackFilled,
        18.0,
        dim,
        palette.text,
        &gettext(app.locale, "Previous"),
    )
    .clicked()
    {
        app.actions.push(Action::Previous);
    }

    let disc = slot(widths[2]);
    let pulse = if app.settings.beat_pulse {
        app.beat.pulse(ui.input(|input| input.time))
    } else {
        0.0
    };
    if pulse > 0.01 && !crate::motion::reduced(ui.ctx()) {
        // A soft halo that breathes out from the play button on each beat,
        // in the visualiser's colour where the button sits along the bar,
        // so it matches the glow above it.
        let amount = pulse * app.settings.beat_pulse_strength.amount();
        let window = ui.ctx().content_rect();
        let along = ((disc.center().x - window.left()) / window.width().max(1.0)).clamp(0.0, 1.0);
        let colour = swept(&visualiser_palette(app, None), along);
        let ring = 19.0 + 8.0 * (1.0 - pulse);
        for (extra, alpha) in [(8.0, 0.05), (4.0, 0.1), (0.0, 0.16)] {
            ui.painter().circle_filled(
                disc.center(),
                ring + extra,
                colour.gamma_multiply(alpha * amount),
            );
        }
        ui.painter().circle_stroke(
            disc.center(),
            ring,
            egui::Stroke::new(1.5, colour.gamma_multiply(0.7 * amount)),
        );
    }
    if loading || app.any_play_pending() {
        ui.painter()
            .circle_filled(disc.center(), 18.0, palette.text);
        let mut cell = centered(ui, disc);
        theme::spinner(&mut cell, 22.0, palette.window);
    } else {
        let icon = if playing {
            Icon::PauseFilled
        } else {
            Icon::PlayFilled
        };
        let hover = if palette.dark {
            egui::Color32::WHITE
        } else {
            palette.text
        };
        let mut cell = centered(ui, disc);
        if theme::circle_button(
            &mut cell,
            icon,
            36.0,
            palette.text,
            hover,
            palette.window,
            &if playing {
                gettext(app.locale, "Pause")
            } else {
                gettext(app.locale, "Play")
            },
        )
        .clicked()
        {
            app.actions.push(Action::TogglePlay);
        }
    }

    let mut cell = centered(ui, slot(widths[3]));
    if theme::icon_button(
        &mut cell,
        Icon::SkipForwardFilled,
        18.0,
        dim,
        palette.text,
        &gettext(app.locale, "Next"),
    )
    .clicked()
    {
        app.actions.push(Action::Next);
    }

    let (repeat_icon, repeat_color, tooltip) = match repeat {
        RepeatMode::Off => (Icon::Repeat, dim, gettext(app.locale, "Repeat")),
        RepeatMode::Context => (
            Icon::Repeat,
            palette.accent,
            gettext(app.locale, "Repeat one"),
        ),
        RepeatMode::Track => (
            Icon::Repeat1,
            palette.accent,
            gettext(app.locale, "Repeat off"),
        ),
    };
    let mut cell = centered(ui, slot(widths[4]));
    if theme::icon_button(
        &mut cell,
        repeat_icon,
        17.0,
        repeat_color,
        if repeat == RepeatMode::Off {
            palette.text
        } else {
            palette.accent_hover
        },
        &tooltip,
    )
    .clicked()
    {
        app.actions.push(Action::CycleRepeat);
    }

    // Progress row, just below the buttons (disc bottom + 6px gap + half of
    // the time text's line height).
    let row_cy = cy + 31.0;
    let slider_width = (region.width() - 120.0).clamp(120.0, 620.0);
    let (position, duration) = now
        .map(|now| (now.position_ms, now.duration_ms))
        .unwrap_or((0, 0));
    let shown_position = match app.seek_preview {
        Some(fraction) => (fraction * duration as f32) as u32,
        None => position,
    };
    let time_color = if now.is_some() {
        palette.secondary
    } else {
        palette.dim
    };
    let slider_left = region.center().x - slider_width / 2.0;
    ui.painter().text(
        pos2(slider_left - 8.0, row_cy),
        egui::Align2::RIGHT_CENTER,
        util::format_duration_ms(shown_position),
        theme::regular(11.5),
        time_color,
    );
    let slider_rect =
        Rect::from_center_size(pos2(region.center().x, row_cy), vec2(slider_width, 16.0));
    let mut slider_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(slider_rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    let fraction = if duration > 0 {
        position as f32 / duration as f32
    } else {
        0.0
    };
    match thin_slider(
        &mut slider_ui,
        &palette,
        egui::Id::new("seek-slider"),
        &gettext(app.locale, "Playback position (%)"),
        fraction,
        slider_width,
        None,
    ) {
        SliderEvent::Dragging(value) => app.seek_preview = Some(value),
        SliderEvent::Committed(value) => {
            app.seek_preview = None;
            if duration > 0 {
                app.actions
                    .push(Action::Seek((value * duration as f32) as u32));
            }
        }
        SliderEvent::None => {}
    }
    ui.painter().text(
        pos2(slider_left + slider_width + 8.0, row_cy),
        egui::Align2::LEFT_CENTER,
        util::format_duration_ms(duration),
        theme::regular(11.5),
        time_color,
    );
}

fn extras(app: &mut App, ui: &mut egui::Ui, now: Option<&NowPlaying>, transport_right: f32) {
    let palette = app.palette;
    ui.spacing_mut().item_spacing.x = 6.0;
    let volume = now
        .map(|now| now.volume_percent)
        .unwrap_or_else(|| crate::app::volume_to_percent(app.local.volume));
    let shown = match app.volume_preview {
        Some(fraction) => (fraction * 100.0).round() as u8,
        None => volume,
    };
    // Some remote devices, such as a phone playing over Bluetooth, refuse
    // volume changes from other apps: show their volume but don't offer to
    // change it.
    let adjustable = now.is_none_or(|now| now.can_set_volume);
    let controls = ui.add_enabled_ui(adjustable, |ui| {
        match thin_slider(
            ui,
            &palette,
            egui::Id::new("volume-slider"),
            &gettext(app.locale, "Volume (%)"),
            shown as f32 / 100.0,
            92.0,
            Some(0.05),
        ) {
            SliderEvent::Dragging(value) => {
                app.volume_preview = Some(value);
                // Local volume is cheap to apply continuously; remote goes on release.
                if now.is_none_or(|now| now.local) {
                    app.actions
                        .push(Action::PreviewVolume((value * 100.0).round() as u8));
                }
            }
            SliderEvent::Committed(value) => {
                app.volume_preview = None;
                app.actions
                    .push(Action::SetVolume((value * 100.0).round() as u8));
            }
            SliderEvent::None => {}
        }
        let volume_icon = match shown {
            0 => Icon::VolumeX,
            1..=33 => Icon::Volume,
            34..=66 => Icon::Volume1,
            _ => Icon::Volume2,
        };
        if theme::icon_button(
            ui,
            volume_icon,
            18.0,
            palette.secondary,
            palette.text,
            &if shown == 0 {
                gettext(app.locale, "Unmute")
            } else {
                gettext(app.locale, "Mute")
            },
        )
        .clicked()
        {
            app.actions.push(Action::ToggleMute);
        }
    });
    if !adjustable {
        ui.interact(
            controls.response.rect,
            egui::Id::new("volume-fixed"),
            egui::Sense::hover(),
        )
        .on_hover_text(gettext(
            app.locale,
            "This device's volume can't be changed from Motion Music",
        ));
    }
    ui.add_space(4.0);
    let remote = now.is_some_and(|now| !now.local);
    // Playing elsewhere, the button shows that device's own icon.
    let icon = now
        .filter(|now| !now.local)
        .and_then(|now| now.device_kind.as_deref())
        .map_or(Icon::Speaker, super::devices::device_icon);
    let devices = theme::icon_button(
        ui,
        icon,
        18.0,
        if remote {
            palette.accent
        } else {
            palette.secondary
        },
        palette.text,
        &gettext(app.locale, "Connect to a device"),
    );
    ui.ctx().data_mut(|data| {
        data.insert_temp(egui::Id::new(super::devices::BUTTON_RECT_ID), devices.rect)
    });
    if devices.clicked() {
        app.actions.push(Action::ToggleDevicesPopup);
    }
    let queue_open = app.show_queue_panel || matches!(app.page(), Page::Queue);
    let queue_button = theme::icon_button(
        ui,
        Icon::ListVideo,
        18.0,
        if queue_open {
            palette.accent
        } else {
            palette.secondary
        },
        palette.text,
        &gettext(app.locale, "Queue"),
    );
    if queue_button.clicked() {
        app.actions.push(Action::ToggleQueuePanel);
    }
    // A dragged song dropped on the queue button queues it, same as the
    // "Add to queue" menu item.
    if let Some(track) = queue_button.dnd_release_payload::<DragTrack>() {
        app.actions.push(Action::QueueMany {
            songs: track
                .items
                .iter()
                .map(|item| (item.uri().to_string(), item.name().to_string()))
                .collect(),
        });
    }
    if theme::icon_button(
        ui,
        Icon::Mic,
        18.0,
        if app.show_lyrics_panel {
            palette.accent
        } else {
            palette.secondary
        },
        palette.text,
        &gettext(app.locale, "Lyrics"),
    )
    .clicked()
    {
        app.actions.push(Action::ToggleLyricsPanel);
    }
    // The Listening activity button is the first to go in a narrow window,
    // so it never crowds the transport.
    let left = ui.min_rect().left() - ui.spacing().item_spacing.x - (18.0 + theme::ICON_BUTTON_PAD);
    if left - transport_right >= TRANSPORT_GAP
        && theme::icon_button(
            ui,
            Icon::Users,
            18.0,
            if app.show_friends_panel {
                palette.accent
            } else {
                palette.secondary
            },
            palette.text,
            &gettext(app.locale, "Listening activity"),
        )
        .clicked()
    {
        app.actions.push(Action::ToggleFriendsPanel);
    }
    sleep_timer_button(app, ui, transport_right);
    let left = ui.min_rect().left() - ui.spacing().item_spacing.x - (18.0 + theme::ICON_BUTTON_PAD);
    if left - transport_right >= TRANSPORT_GAP
        && theme::icon_button(
            ui,
            Icon::Expand,
            18.0,
            if app.visualiser_stage {
                palette.accent
            } else {
                palette.secondary
            },
            palette.text,
            super::keys::platform_shortcut(
                &gettext(app.locale, "Visualiser stage (Ctrl+Shift+S)"),
                &gettext(app.locale, "Visualiser stage (Cmd+Shift+S)"),
            ),
        )
        .clicked()
    {
        app.actions.push(Action::ToggleVisualiserStage);
    }
}

/// A moon-and-clock button for the sleep timer, when there is room for it.
fn sleep_timer_button(app: &mut App, ui: &mut egui::Ui, transport_right: f32) {
    if !app.settings.sleep_timer_button {
        return;
    }
    let palette = app.palette;
    let left = ui.min_rect().left() - ui.spacing().item_spacing.x - (18.0 + theme::ICON_BUTTON_PAD);
    if left - transport_right < TRANSPORT_GAP {
        return;
    }
    let remaining = app.sleep_timer.map(|timer| {
        timer
            .ends
            .saturating_duration_since(std::time::Instant::now())
    });
    let tooltip = match remaining {
        Some(left) => gettext(app.locale, "Sleep timer: {minutes} min left")
            .replace("{minutes}", &left.as_secs().div_ceil(60).to_string()),
        None => gettext(app.locale, "Sleep timer").into_owned(),
    };
    let response = theme::icon_button(
        ui,
        Icon::Clock,
        18.0,
        if remaining.is_some() {
            palette.accent
        } else {
            palette.secondary
        },
        palette.text,
        &tooltip,
    );
    egui::Popup::menu(&response)
        .id(egui::Id::new("player-bar-sleep-timer"))
        .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
        .show(|ui| {
            ui.set_min_width(170.0);
            theme::text(
                ui,
                gettext(app.locale, "Fade out and pause in").as_ref(),
                theme::semibold(13.0),
                palette.secondary,
            );
            for minutes in [5_u32, 15, 30, 45, 60, 90] {
                let label =
                    gettext(app.locale, "{count} minutes").replace("{count}", &minutes.to_string());
                if ui.button(label).clicked() {
                    app.actions.push(Action::SetSleepTimer(Some(minutes)));
                }
            }
            if remaining.is_some()
                && ui
                    .button(gettext(app.locale, "Turn off").as_ref())
                    .clicked()
            {
                app.actions.push(Action::SetSleepTimer(None));
            }
            ui.separator();
            // Settings > Extras brings it back.
            if ui
                .button(gettext(app.locale, "Remove from the player bar").as_ref())
                .on_hover_text(gettext(
                    app.locale,
                    "Settings > Extras brings it back. The timer still works from there and the tray.",
                ))
                .clicked()
            {
                app.settings.sleep_timer_button = false;
                app.actions.push(Action::SettingsChanged);
            }
        });
}

#[cfg(test)]
mod player_bar_tint_tests {
    use super::*;
    use crate::theme::Palette;

    /// Run one frame at `time` and report the bar's actual fill.
    fn frame(ctx: &egui::Context, time: f64, panel: Color32, tint: Option<Color32>) -> Color32 {
        let mut fill = Color32::PLACEHOLDER;
        let mut output = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                ..Default::default()
            },
            |ui| fill = eased_fill(ui.ctx(), panel, tint),
        );
        output.textures_delta.clear();
        fill
    }

    #[test]
    fn a_song_without_art_preserves_translucent_panel() {
        for panel in [
            Palette::dark().panel,
            Palette::light().panel,
            Color32::from_rgba_unmultiplied(80, 120, 180, 128),
        ] {
            let ctx = egui::Context::default();
            assert_eq!(frame(&ctx, 0.0, panel, None), panel);
            let art = Color32::from_rgb(200, 40, 90);
            frame(&ctx, 0.1, panel, Some(art));
            frame(&ctx, 0.6, panel, Some(art));
            assert_eq!(frame(&ctx, 0.7, panel, None).a(), panel.a());
            assert_eq!(frame(&ctx, 1.2, panel, None), panel);
        }
    }

    #[test]
    fn the_first_frame_shows_the_tint_without_fading_in() {
        let ctx = egui::Context::default();
        let panel = Palette::dark().panel;
        let art = Color32::from_rgb(200, 40, 90);
        assert_eq!(
            frame(&ctx, 0.0, panel, Some(art)),
            super::super::blend(panel, art, TINT_STRENGTH)
        );
    }

    #[test]
    fn first_art_after_an_untinted_frame_fades_in() {
        let ctx = egui::Context::default();
        let panel = Palette::dark().panel;
        let art = Color32::from_rgb(220, 140, 160);
        let target = super::super::blend(panel, art, TINT_STRENGTH);
        let fade = f64::from(TINT_FADE_SECONDS);

        assert_eq!(frame(&ctx, 0.0, panel, None), panel);
        assert_eq!(frame(&ctx, 0.1, panel, Some(art)), panel);
        let middle = frame(&ctx, 0.1 + fade / 2.0, panel, Some(art));
        assert_ne!(middle, panel);
        assert_ne!(middle, target);
        // Most of the colour change is visible by halfway through the
        // shortened transition, while the final part settles gently.
        let from = panel.to_srgba_unmultiplied();
        let to = target.to_srgba_unmultiplied();
        let shown = middle.to_srgba_unmultiplied();
        for channel in 0..3 {
            let span = from[channel].abs_diff(to[channel]);
            if span >= 8 {
                assert!(
                    f32::from(from[channel].abs_diff(shown[channel])) >= f32::from(span) * 0.75
                );
            }
        }
        assert_eq!(frame(&ctx, 0.1 + fade, panel, Some(art)), target);
    }

    #[test]
    fn changing_songs_mid_fade_continues_from_the_visible_colour() {
        let ctx = egui::Context::default();
        let panel = Palette::dark().panel;
        let first = Color32::from_rgb(20, 40, 60);
        let second = Color32::from_rgb(220, 140, 160);
        let third = Color32::from_rgb(40, 230, 30);
        let first_fill = super::super::blend(panel, first, TINT_STRENGTH);
        let second_fill = super::super::blend(panel, second, TINT_STRENGTH);
        let third_fill = super::super::blend(panel, third, TINT_STRENGTH);
        let fade = f64::from(TINT_FADE_SECONDS);

        assert_eq!(frame(&ctx, 0.0, panel, Some(first)), first_fill);
        assert_eq!(frame(&ctx, 0.05, panel, Some(second)), first_fill);
        let middle = frame(&ctx, 0.2, panel, Some(second));
        assert_ne!(middle, first_fill);
        assert_ne!(middle, second_fill);
        assert_eq!(frame(&ctx, 0.2, panel, Some(third)), middle);
        assert_ne!(frame(&ctx, 0.2 + fade / 2.0, panel, Some(third)), middle);
        assert_eq!(frame(&ctx, 0.2 + fade, panel, Some(third)), third_fill);
    }

    #[test]
    fn reduced_motion_snaps_the_artwork_tint_in_both_directions() {
        let ctx = egui::Context::default();
        let panel = Palette::dark().panel;
        let art = Color32::from_rgb(220, 140, 160);
        let target = super::super::blend(panel, art, TINT_STRENGTH);
        frame(&ctx, 0.0, panel, None);
        crate::motion::apply(&ctx, true);
        assert_eq!(frame(&ctx, 0.1, panel, Some(art)), target);
        assert_eq!(frame(&ctx, 0.2, panel, None), panel);
    }

    #[test]
    fn a_new_session_does_not_reuse_the_previous_tint() {
        let ctx = egui::Context::default();
        let panel = Palette::dark().panel;
        let first = Color32::from_rgb(20, 40, 60);
        let second = Color32::from_rgb(220, 140, 160);
        let next_session = Color32::from_rgb(40, 230, 30);

        frame(&ctx, 0.0, panel, Some(first));
        frame(&ctx, 0.1, panel, Some(second));
        assert_ne!(
            frame(&ctx, 0.2, panel, Some(second)),
            super::super::blend(panel, second, TINT_STRENGTH)
        );

        end_tint_session(&ctx);
        assert_eq!(
            frame(&ctx, 0.21, panel, Some(next_session)),
            super::super::blend(panel, next_session, TINT_STRENGTH)
        );

        end_tint_session(&ctx);
        assert_eq!(frame(&ctx, 0.22, panel, None), panel);
    }

    /// However light or dark the cover or the theme's accent, the
    /// visualizer's colours stay in the band the theme's text reads against,
    /// and keep their hue.
    #[test]
    fn visualizer_colours_stay_behind_the_words() {
        let luminance = |colour: Color32| {
            let linear = egui::Rgba::from(colour);
            0.2126 * linear.r() + 0.7152 * linear.g() + 0.0722 * linear.b()
        };
        for base in [
            Color32::from_rgb(255, 240, 80),
            Color32::from_rgb(20, 30, 90),
            Color32::WHITE,
            Color32::BLACK,
            Color32::from_rgb(30, 215, 96),
        ] {
            let art = [[base.r(), base.g(), base.b()]];
            for (art, accent) in [(Some(&art[..]), Color32::RED), (None, base)] {
                for colour in vis_colours(art, accent, true) {
                    assert!(
                        luminance(colour) <= DARK_CEILING + 0.01,
                        "{base:?} on dark: {colour:?}"
                    );
                }
                for colour in vis_colours(art, accent, false) {
                    assert!(
                        (LIGHT_FLOOR - 0.01..=LIGHT_CEILING + 0.01).contains(&luminance(colour)),
                        "{base:?} on light: {colour:?}"
                    );
                }
            }
        }
        // A yellow cover stays yellow, only deeper.
        let low = vis_colours(Some(&[[255, 240, 80]]), Color32::RED, true)[0];
        assert!(low.r() > low.b() && low.g() > low.b(), "{low:?}");
    }

    /// With colour from album art the bars take the cover's own colours,
    /// its accent at the bass and its last colour at the treble, and a grey
    /// cover's bars stay grey. Without, they turn the theme's accent.
    #[test]
    fn visualizer_sweeps_across_the_covers_colours() {
        let art = [[200, 40, 40], [40, 200, 60], [40, 40, 200]];
        let colours = vis_colours(Some(&art), Color32::WHITE, true);
        assert_eq!(colours.len(), 3);
        let [red, green, blue] = [colours[0], colours[1], colours[2]];
        assert!(red.r() > red.g() && red.r() > red.b(), "{red:?}");
        assert!(green.g() > green.r() && green.g() > green.b(), "{green:?}");
        assert!(blue.b() > blue.r() && blue.b() > blue.g(), "{blue:?}");
        assert_eq!(swept(&colours, 0.0), red);
        assert_eq!(swept(&colours, 0.5), green);
        assert_eq!(swept(&colours, 1.0), blue);
        assert_eq!(swept(&colours[..1], 0.7), red);

        let spread = |c: Color32| c.r().max(c.g()).max(c.b()) - c.r().min(c.g()).min(c.b());
        for cover in [[150, 140, 140], [40, 20, 8]] {
            let grey = vis_colours(Some(&[cover]), Color32::RED, true)[0];
            assert!(spread(grey) < 4, "{cover:?} stays grey: {grey:?}");
        }
        // A grey accent gives the bass to the cover's hue.
        let colours = vis_colours(
            Some(&[[150, 140, 140], [200, 40, 40]]),
            Color32::WHITE,
            true,
        );
        assert!(colours[0].r() > colours[0].g() * 2, "{colours:?}");
        assert!(spread(colours[1]) < 24, "{colours:?}");

        let accent = Color32::from_rgb(30, 215, 96);
        assert_eq!(vis_colours(None, accent, true).len(), 2);
    }
}

#[cfg(test)]
mod visualizer_scene_tests {
    use super::*;
    use crate::settings::PlayerBarVis;

    fn geometry(
        mode: PlayerBarVis,
        levels: &[f32],
        time: f32,
        width: f32,
        dark: bool,
    ) -> Vec<egui::epaint::Vertex> {
        let ctx = egui::Context::default();
        let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(width, 88.0));
        let colours = vis_colours(None, Color32::from_rgb(190, 70, 250), dark);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let painter = ui.painter().with_clip_rect(rect);
            if mode == PlayerBarVis::Mirror {
                mirror_wave(&painter, rect, levels, &colours, 1.0);
            } else {
                spectrum_effect(mode, &painter, rect, levels, &colours, 1.0, time);
            }
        });
        // This headless geometry check does not upload textures.
        output.textures_delta.clear();
        ctx.tessellate(output.shapes, output.pixels_per_point)
            .into_iter()
            .flat_map(|primitive| match primitive.primitive {
                egui::epaint::Primitive::Mesh(mesh) => mesh.vertices,
                egui::epaint::Primitive::Callback(_) => Vec::new(),
            })
            .collect()
    }

    #[test]
    fn animated_scenes_react_to_audio_and_stay_finite_at_both_window_sizes() {
        for mode in NEW_SCENES {
            for width in [360.0, 1440.0] {
                for dark in [false, true] {
                    assert!(
                        geometry(mode, &[0.0; 75], 0.0, width, dark).is_empty(),
                        "{mode:?} paints silence"
                    );
                    let first = geometry(mode, &[0.6; 75], 0.0, width, dark);
                    assert!(!first.is_empty(), "{mode:?} has no visible geometry");
                    assert!(
                        first
                            .iter()
                            .all(|vertex| vertex.pos.x.is_finite() && vertex.pos.y.is_finite()),
                        "{mode:?} has invalid geometry"
                    );
                    assert_ne!(
                        first,
                        geometry(mode, &[0.6; 75], 1.0, width, dark),
                        "{mode:?} does not animate"
                    );
                    assert_ne!(
                        first,
                        geometry(mode, &[0.3; 75], 0.0, width, dark),
                        "{mode:?} ignores level changes"
                    );
                }
            }
        }
    }

    const NEW_SCENES: [PlayerBarVis; 17] = [
        PlayerBarVis::Waterfall,
        PlayerBarVis::Prism,
        PlayerBarVis::Helix,
        PlayerBarVis::Synthwave,
        PlayerBarVis::Terrain,
        PlayerBarVis::Lava,
        PlayerBarVis::Galaxy,
        PlayerBarVis::Matrix,
        PlayerBarVis::Spectrogram,
        PlayerBarVis::Orb,
        PlayerBarVis::Bubbles,
        PlayerBarVis::Skyline,
        PlayerBarVis::Ocean,
        PlayerBarVis::Led,
        PlayerBarVis::Fireflies,
        PlayerBarVis::Heartbeat,
        PlayerBarVis::Circuit,
    ];

    #[test]
    fn new_scenes_keep_geometry_near_the_bar_with_a_fixed_vertex_budget() {
        for mode in NEW_SCENES {
            for width in [240.0, 1440.0] {
                let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(width, 88.0));
                for time in [0.0, 10.3, 3599.9] {
                    let vertices = geometry(mode, &[1.0; 75], time, width, true);
                    assert!(!vertices.is_empty(), "{mode:?} at {time}");
                    assert!(
                        vertices.len() < 20_000,
                        "{mode:?} exceeded its fixed drawing budget: {}",
                        vertices.len()
                    );
                    // Glow/antialiasing may extend a few pixels beyond the
                    // clipped bar. No scene generates unbounded offscreen work.
                    assert!(
                        vertices
                            .iter()
                            .all(|vertex| rect.expand(8.0).contains(vertex.pos)),
                        "{mode:?} at {time} escaped its bar: {:?}",
                        vertices
                            .iter()
                            .find(|vertex| !rect.expand(8.0).contains(vertex.pos))
                            .map(|vertex| vertex.pos)
                    );
                    let left = vertices
                        .iter()
                        .map(|vertex| vertex.pos.x)
                        .fold(f32::INFINITY, f32::min);
                    let right = vertices
                        .iter()
                        .map(|vertex| vertex.pos.x)
                        .fold(f32::NEG_INFINITY, f32::max);
                    assert!(
                        right - left >= width * 0.6,
                        "{mode:?} did not use the player bar's width"
                    );
                }
            }
        }
    }

    #[test]
    fn sparse_music_keeps_new_scene_cores_visible_without_lighting_silence() {
        assert_eq!(scene_energy(&[]), 0.0);
        assert_eq!(scene_energy(&[0.0; 75]), 0.0);
        let mut sparse = [0.0; 75];
        sparse[37] = 0.25;
        assert!(
            scene_energy(&sparse) >= 0.39,
            "one musical peak must not be averaged into darkness"
        );
        sparse[37] = 1.0;
        for mode in NEW_SCENES {
            let vertices = geometry(mode, &sparse, 0.4, 960.0, true);
            let brightest = vertices
                .iter()
                .map(|vertex| vertex.color.a())
                .max()
                .unwrap_or(0);
            assert!(
                brightest >= 64,
                "{mode:?} has no clearly visible core for sparse audio: {brightest}"
            );
            let top = vertices
                .iter()
                .map(|vertex| vertex.pos.y)
                .fold(f32::INFINITY, f32::min);
            let bottom = vertices
                .iter()
                .map(|vertex| vertex.pos.y)
                .fold(f32::NEG_INFINITY, f32::max);
            assert!(
                bottom - top >= 88.0 * 0.35,
                "{mode:?} collapsed into a tiny line"
            );
            assert!(geometry(mode, &[0.0; 75], 0.4, 960.0, true).is_empty());
        }
    }

    #[test]
    fn new_scenes_follow_frequency_distribution_and_have_distinct_compositions() {
        let mut low = [0.0; 75];
        low[..8].fill(1.0);
        let mut high = [0.0; 75];
        high[67..].fill(1.0);
        let mut seen = Vec::new();
        for mode in NEW_SCENES {
            assert_ne!(
                geometry(mode, &low, 0.4, 960.0, true),
                geometry(mode, &high, 0.4, 960.0, true),
                "{mode:?} ignored which frequencies were sounding"
            );
            let composition = geometry(mode, &[0.7; 75], 0.4, 960.0, true);
            assert!(
                !seen.contains(&composition),
                "{mode:?} repeated another scene's composition"
            );
            seen.push(composition);
            assert!(geometry(mode, &[], 0.4, 960.0, true).is_empty());
            let frozen = geometry(mode, &[0.7], 0.0, 960.0, true);
            assert_eq!(
                frozen,
                geometry(mode, &[0.7], 0.0, 960.0, true),
                "{mode:?} moved despite a frozen decorative clock"
            );
        }
    }

    #[test]
    fn mirror_wave_follows_the_pcm_envelope_without_decorative_motion() {
        assert!(geometry(PlayerBarVis::Mirror, &[], 0.0, 640.0, true).is_empty());
        assert!(geometry(PlayerBarVis::Mirror, &[0.5], 0.0, 640.0, true).is_empty());
        let trace = [0.0, 0.2, -0.6, 0.9, -0.5, 0.1, 0.0];
        let first = geometry(PlayerBarVis::Mirror, &trace, 0.0, 640.0, true);
        assert!(!first.is_empty());
        assert_eq!(
            first,
            geometry(PlayerBarVis::Mirror, &trace, 9.0, 640.0, true)
        );
        assert_ne!(
            first,
            geometry(PlayerBarVis::Mirror, &[0.1; 7], 0.0, 640.0, true)
        );
    }

    #[test]
    fn moving_to_another_speaker_dims_the_visualizers_and_brings_them_back() {
        let mut motion = VisMotion::default();
        let phone = Some(Some("Phone".to_owned()));
        assert_eq!(motion.speaker_fade(None, 0.0, true), 1.0);
        assert_eq!(
            motion.speaker_fade(Some(None), 0.0, true),
            1.0,
            "first speaker"
        );
        assert_eq!(motion.speaker_fade(phone.clone(), 1.0, true), 1.0);
        let middle = motion.speaker_fade(phone.clone(), 1.0 + SPEAKER_FADE / 2.0, true);
        assert!(middle < 0.05, "dark halfway through: {middle}");
        let late = motion.speaker_fade(phone.clone(), 1.0 + SPEAKER_FADE * 0.9, true);
        assert!(late > 0.5 && late < 1.0, "coming back: {late}");
        assert_eq!(
            motion.speaker_fade(phone.clone(), 1.0 + SPEAKER_FADE, true),
            1.0
        );
        // Nothing playing is not a change of speaker, and off means no fade.
        assert_eq!(motion.speaker_fade(None, 9.0, true), 1.0);
        assert_eq!(motion.speaker_fade(Some(None), 10.0, false), 1.0);
        assert_eq!(motion.speaker_fade(Some(None), 10.5, true), 1.0);
    }

    #[test]
    fn a_new_visualiser_fades_in_after_the_old_one_fades_out() {
        let mut motion = VisMotion::default();
        let (ocean, galaxy) = (PlayerBarVis::Ocean, PlayerBarVis::Galaxy);
        assert_eq!(motion.crossfade(ocean, 10.0, false), (ocean, 1.0));
        // The old one fades out over the first half...
        let (mode, early) = motion.crossfade(galaxy, 10.0, false);
        assert_eq!((mode, early), (ocean, 1.0));
        let (mode, fading) = motion.crossfade(galaxy, 10.0 + VIS_CROSSFADE * 0.25, false);
        assert_eq!(mode, ocean);
        assert!(fading > 0.0 && fading < 1.0);
        // ...and the new one fades in over the second.
        let (mode, rising) = motion.crossfade(galaxy, 10.0 + VIS_CROSSFADE * 0.75, false);
        assert_eq!(mode, galaxy);
        assert!(rising > 0.0 && rising < 1.0);
        assert_eq!(
            motion.crossfade(galaxy, 10.0 + VIS_CROSSFADE, false),
            (galaxy, 1.0)
        );
        // Reduced motion changes at once.
        assert_eq!(motion.crossfade(ocean, 20.0, true), (ocean, 1.0));
        assert_eq!(smooth(0.0), 0.0);
        assert_eq!(smooth(1.0), 1.0);
    }

    #[test]
    fn spectrum_interpolation_handles_edges_and_empty_input() {
        assert_eq!(level_at(&[], 0.5), 0.0);
        assert_eq!(level_at(&[0.4], 0.7), 0.4);
        assert_eq!(level_at(&[0.0, 1.0], 0.25), 0.25);
        assert_eq!(level_at(&[0.2, 0.8], -0.5), 0.2);
        assert_eq!(level_at(&[0.2, 0.8], 1.5), 0.8);
        assert_eq!(mean_level(&[]), 0.0);
    }
}
