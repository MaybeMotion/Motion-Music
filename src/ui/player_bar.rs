//! The now-playing bar along the bottom of the window.

use egui::{Align, Color32, Frame, Layout, Margin, Rect, Sense, UiBuilder, Vec2, pos2, vec2};

use crate::app::{App, NowPlaying};
use crate::i18n::gettext;
use crate::model::{Action, DragTrack, Page};
use crate::player::RepeatMode;
use crate::theme::{self, Icon};
use crate::util;

use super::widgets::{SliderEvent, thin_slider};

/// How much of the playing art's tint the bar's fill carries.
const TINT_STRENGTH: f32 = 0.12;
/// How long the bar takes to cross over to a new song's tint.
const TINT_FADE_SECONDS: f32 = 0.45;
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
            if let Some(stops) = app.theme_gradient() {
                let stops = stops.map(|stop| stop.gamma_multiply(0.45));
                theme::paint_gradient(ui, behind, stops, 0);
            }
            if visualizer(app, ui, behind, now.as_ref()) {
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
fn visualizer(app: &mut App, ui: &egui::Ui, rect: Rect, now: Option<&NowPlaying>) -> bool {
    use crate::settings::PlayerBarVis;
    use crate::vis;
    let mode = app.settings.player_bar_vis;
    if mode == PlayerBarVis::Off {
        return false;
    }
    let count = match mode {
        PlayerBarVis::Waveform | PlayerBarVis::Mirror => {
            (rect.width() / 8.0).clamp(75.0, 320.0) as usize * vis::SCOPE_STEP
        }
        _ => vis::WIDE_SAMPLES,
    };
    let samples = app.visualizer_samples(now, count);
    let sounding = samples.is_some();
    let dark = ui.visuals().dark_mode;
    let gradient_colours = app
        .theme_gradient()
        .map(|stops| stops.map(|colour| [colour.r(), colour.g(), colour.b()]));
    let source_colours = app
        .now_playing_palette()
        .or_else(|| gradient_colours.as_ref().map(|colours| colours.as_slice()));
    let colours = vis_colours(source_colours, app.palette.accent, dark);
    let strength = if dark { 1.0 } else { LIGHT_STRENGTH };
    let painter = ui.painter().with_clip_rect(rect);
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
            if !sounding && app.player_bar_analyser.settled() {
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
                spectrum_effect(mode, &painter, rect, &levels, &colours, strength, time);
            }
            sounding || !app.player_bar_analyser.settled()
        }
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
fn swept(colours: &[Color32], at: f32) -> Color32 {
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
        PlayerBarVis::Aurora => aurora(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Radial => radial(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Orbit => orbit(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Embers => embers(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Tunnel => tunnel(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Waterfall => waterfall(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Prism => prism(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Helix => helix(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Starfield => starfield(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Plasma => plasma(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Ripples => ripples(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Lasers => lasers(painter, rect, levels, colours, strength, time),
        PlayerBarVis::Fireworks => fireworks(painter, rect, levels, colours, strength, time),
        _ => {}
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

/// Four translucent curtains, with their contours lifted by different bands.
fn aurora(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = mean_level(levels);
    for layer in 0..4 {
        let lane = layer as f32;
        let colour = swept(colours, lane / 3.0);
        let mut mesh = egui::Mesh::default();
        let mut edge = Vec::with_capacity(65);
        for index in 0..=64 {
            let at = index as f32 / 64.0;
            let band = level_at(levels, (at + lane * 0.17).fract());
            let bend = (at * std::f32::consts::TAU * (1.0 + lane * 0.22)
                + time * (0.35 + lane * 0.11)
                + lane)
                .sin();
            let y = rect.center().y
                + bend * rect.height() * (0.07 + energy * 0.16)
                + (lane - 1.5) * rect.height() * 0.08;
            let reach = rect.height() * (0.05 + band * 0.22) * energy.sqrt();
            let top = pos2(rect.left() + at * rect.width(), y - reach);
            let bottom = pos2(top.x, y + reach);
            edge.push(top);
            let base = mesh.vertices.len() as u32;
            mesh.colored_vertex(top, colour.gamma_multiply(0.28 * strength * energy));
            mesh.colored_vertex(bottom, Color32::TRANSPARENT);
            if index > 0 {
                mesh.add_triangle(base - 2, base, base + 1);
                mesh.add_triangle(base - 2, base + 1, base - 1);
            }
        }
        painter.add(egui::Shape::mesh(mesh));
        neon_line(painter, edge, colour, 0.42 * strength * energy);
    }
}

/// Three halos whose individual rays follow the spectrum from bass to treble.
fn radial(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = mean_level(levels);
    for halo in 0..3 {
        let center = pos2(
            rect.left() + rect.width() * (halo as f32 + 0.5) / 3.0,
            rect.center().y,
        );
        let radius = rect.height() * (0.14 + energy * 0.04);
        for (index, level) in levels.iter().enumerate() {
            let at = index as f32 / levels.len() as f32;
            let angle = at * std::f32::consts::TAU + time * 0.12 * (halo as f32 - 1.0);
            let direction = vec2(angle.cos(), angle.sin());
            let ray = [
                center + direction * radius,
                center + direction * (radius + level * rect.height() * 0.27),
            ];
            let colour =
                swept(colours, at).gamma_multiply(strength * (0.12 + level * 0.48) * energy.sqrt());
            painter.line_segment(ray, egui::Stroke::new(5.0, colour.gamma_multiply(0.12)));
            painter.line_segment(ray, egui::Stroke::new(1.4, colour));
        }
    }
}

/// Elliptical neon tracks with travelling satellites and fading comet tails.
fn orbit(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = mean_level(levels);
    for ring in 0..5 {
        let at = ring as f32 / 4.0;
        let band = level_at(levels, at);
        let radius = vec2(
            rect.width() * (0.12 + at * 0.34),
            rect.height() * (0.10 + at * 0.28),
        ) * (0.8 + band * 0.2);
        let colour = swept(colours, at);
        let point =
            |angle: f32| rect.center() + vec2(angle.cos() * radius.x, angle.sin() * radius.y);
        let track = (0..=64)
            .map(|index| point(index as f32 / 64.0 * std::f32::consts::TAU))
            .collect();
        painter.add(egui::Shape::line(
            track,
            egui::Stroke::new(1.0, colour.gamma_multiply(0.16 * energy * strength)),
        ));
        let angle = time * (0.5 + at * 0.6) + at * 4.0;
        for trail in 0..14 {
            let fade = 1.0 - trail as f32 / 14.0;
            let head = point(angle - trail as f32 * 0.07);
            painter.circle_filled(
                head,
                (1.2 + band * 2.3) * fade,
                colour.gamma_multiply(0.65 * fade * energy * strength),
            );
        }
    }
}

/// Deterministic particles, each driven by a frequency band. No random state
/// or growing particle list is kept between frames.
fn embers(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = mean_level(levels);
    for index in 0..64 {
        let seed = index as f32;
        let at = (seed * 0.618_034).fract();
        let level = level_at(levels, at);
        let rise = (seed * 0.381_966 + time * (0.025 + at * 0.04)).fract();
        let drift = (time * 0.35 + seed * 2.4).sin() * rect.width() * 0.012 * level;
        let head = pos2(
            rect.left() + at * rect.width() + drift,
            rect.bottom() - rise * rect.height(),
        );
        let fade = (std::f32::consts::PI * rise).sin() * energy.sqrt();
        let colour = swept(colours, at).gamma_multiply(strength * fade * (0.2 + level * 0.6));
        let radius = 0.7 + level * 2.4;
        painter.line_segment(
            [head + vec2(0.0, 3.0 + level * 7.0), head],
            egui::Stroke::new(1.0, colour.gamma_multiply(0.35)),
        );
        painter.circle_filled(head, radius * 2.8, colour.gamma_multiply(0.1));
        painter.circle_filled(head, radius, colour);
    }
}

/// Concentric rings expand from a vanishing point, with bass controlling
/// the opening and the upper bands putting ripples around each ring.
fn tunnel(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = mean_level(levels);
    let bass = mean_level(&levels[..levels.len().min(8)]);
    for ring in 0..10 {
        let depth = (ring as f32 / 10.0 + time * 0.065).fract();
        let radius = vec2(rect.width() * 0.53, rect.height() * 0.48)
            * depth.powf(1.6)
            * (0.88 + bass * 0.12);
        let points = (0..=64)
            .map(|index| {
                let at = index as f32 / 64.0;
                let angle = at * std::f32::consts::TAU;
                let ripple = 1.0 + level_at(levels, at) * 0.12 * (angle * 6.0 + time).sin();
                rect.center() + vec2(angle.cos() * radius.x, angle.sin() * radius.y) * ripple
            })
            .collect();
        let fade = (std::f32::consts::PI * depth).sin();
        neon_line(
            painter,
            points,
            swept(colours, depth),
            0.48 * strength * energy * fade,
        );
    }
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

/// Stars accelerate out of a central vanishing point, stretching into
/// short light trails as their frequency band gets louder.
fn starfield(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = scene_energy(levels);
    // Stratify depth independently of direction. Using the same golden
    // fraction for both bunches the outer stars on one side of the bar.
    for star in 0..96 {
        let seed = star as f32;
        let at = (seed * 0.618_034).fract();
        let band = scene_band(levels, at, energy);
        let depth = ((star / 16) as f32 / 6.0 + time * (0.12 + band * 0.09)).fract();
        let angle = (star % 16) as f32 / 16.0 * std::f32::consts::TAU + (seed * 1.3).sin() * 0.08;
        let direction = vec2(
            angle.cos() * rect.width() * 0.48,
            angle.sin() * rect.height() * 0.43,
        );
        let head = rect.center() + direction * depth.powi(2);
        let tail = rect.center() + direction * (depth - 0.045 - band * 0.13).max(0.0).powi(2);
        let fade = depth * (1.0 - depth).sqrt() * 2.6 * energy;
        let colour = swept(colours, at).gamma_multiply(strength * fade * (0.65 + band * 0.35));
        painter.line_segment(
            [tail, head],
            egui::Stroke::new(4.5, colour.gamma_multiply(0.14)),
        );
        painter.line_segment([tail, head], egui::Stroke::new(1.3 + band * 1.5, colour));
        let radius = 1.0 + band * depth * 2.1;
        painter.circle_filled(head, radius * 2.5, colour.gamma_multiply(0.12));
        painter.circle_filled(head, radius, colour);
    }
}

/// A smooth low-resolution interference field, built as one coloured mesh.
/// Audio bends the field and changes its light, rather than just its speed.
fn plasma(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    const COLUMNS: usize = 48;
    const ROWS: usize = 8;
    let energy = scene_energy(levels);
    let bass = mean_level(&levels[..levels.len().min(8)]);
    let mut mesh = egui::Mesh::default();
    for row in 0..=ROWS {
        for column in 0..=COLUMNS {
            let x = column as f32 / COLUMNS as f32;
            let y = row as f32 / ROWS as f32;
            let band = level_at(levels, x);
            let radius = ((x - 0.5).powi(2) * 5.0 + (y - 0.5).powi(2)).sqrt();
            let field = ((x * 12.0 + time * 0.45 + band * 3.0).sin()
                + (y * 5.0 - time * 0.6 + bass * 2.0).sin()
                + (radius * 15.0 - time * 0.8).cos())
                / 3.0;
            let light = field * 0.5 + 0.5;
            let ridge = (1.0 - (light - 0.5).abs() * 12.0).max(0.0);
            let colour = swept(colours, (x * 0.45 + light * 0.55).clamp(0.0, 1.0)).gamma_multiply(
                (strength * energy * (0.07 + light.powi(2) * 0.40 + ridge * 0.18)).min(0.55),
            );
            mesh.colored_vertex(
                pos2(
                    rect.left() + x * rect.width(),
                    rect.top() + y * rect.height(),
                ),
                colour,
            );
            if row > 0 && column > 0 {
                let current = (row * (COLUMNS + 1) + column) as u32;
                let above = current - (COLUMNS + 1) as u32;
                mesh.add_triangle(above - 1, above, current);
                mesh.add_triangle(above - 1, current, current - 1);
            }
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// Four separate pools of concentric ripples, with small bright droplets.
/// Different bands drive each pool so percussion scatters across the bar.
fn ripples(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = scene_energy(levels);
    for pool in 0..4 {
        let at = pool as f32 / 3.0;
        let band = scene_band(levels, at, energy);
        let center = pos2(
            rect.left() + rect.width() * (0.18 + at * 0.64),
            rect.center().y + (pool as f32 * 2.1).sin() * rect.height() * 0.12,
        );
        for ring in 0..5 {
            let depth = (ring as f32 / 5.0 + time * 0.18 + at * 0.31).fract();
            let radius =
                vec2(rect.width() * 0.155, rect.height() * 0.23) * depth * (0.7 + band * 0.3);
            // A just-born ring is a droplet, not a zero-length stroked path.
            if radius.y < 0.5 {
                continue;
            }
            let points = (0..=40)
                .map(|index| {
                    let angle = index as f32 / 40.0 * std::f32::consts::TAU;
                    center + vec2(angle.cos() * radius.x, angle.sin() * radius.y)
                })
                .collect();
            let fade = (1.0 - depth).sqrt() * (0.25 + depth * 0.75);
            scene_neon_line(
                painter,
                points,
                swept(colours, (at + depth * 0.3).min(1.0)),
                1.25 * strength * energy * fade,
            );
        }
        painter.circle_filled(
            center,
            1.5 + band * 2.5,
            swept(colours, at).gamma_multiply(0.9 * strength * energy),
        );
    }
}

/// Three moving laser projectors paint crossing fans of narrow beams.
/// Each beam has its own spectral strength and a translucent light cone.
fn lasers(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = scene_energy(levels);
    let mut mesh = egui::Mesh::default();
    for projector in 0..3 {
        let source = pos2(
            rect.left() + rect.width() * (0.08 + projector as f32 * 0.42),
            rect.bottom() - rect.height() * 0.06,
        );
        for beam in 0..12 {
            let at = beam as f32 / 11.0;
            let band = scene_band(levels, (at + projector as f32 * 0.19).fract(), energy);
            let sweep = (time * 0.4 + projector as f32 + at * 2.0).sin();
            let x = (at * 0.9 + 0.05 + sweep * 0.04).clamp(0.02, 0.98);
            let target = pos2(
                rect.left() + rect.width() * x,
                rect.top() + rect.height() * (0.04 + 0.12 * sweep.abs()),
            );
            let end = source.lerp(target, (0.72 + energy * 0.28) * (0.78 + band * 0.22));
            let colour = swept(colours, at);
            let alpha = (strength * energy * (0.55 + band * 0.45)).min(0.95);
            let spread = (rect.width() * 0.006).min(7.0);
            triangle(
                &mut mesh,
                [source, end - vec2(spread, 0.0), end + vec2(spread, 0.0)],
                [
                    colour.gamma_multiply(alpha * 0.05),
                    colour.gamma_multiply(alpha * 0.18),
                    Color32::TRANSPARENT,
                ],
            );
            painter.line_segment(
                [source, end],
                egui::Stroke::new(5.5, colour.gamma_multiply(alpha * 0.15)),
            );
            painter.line_segment(
                [source, end],
                egui::Stroke::new(1.8 + band, colour.gamma_multiply(alpha)),
            );
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// Small, staggered bursts explode into sparks with curved falling trails.
/// No particle simulation state is retained; every spark has a fixed seed.
fn fireworks(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = scene_energy(levels);
    for burst in 0..6 {
        let at = (burst as f32 + 0.5) / 6.0;
        let band = scene_band(levels, at, energy);
        let age = (time * 0.23 + burst as f32 * 0.163).fract();
        let center = pos2(
            rect.left() + rect.width() * (0.12 + burst as f32 * 0.152),
            rect.center().y + (burst as f32 * 2.4).sin() * rect.height() * 0.06,
        );
        let radius = vec2(
            (rect.width() * 0.08).min(rect.height() * 1.4),
            rect.height() * 0.31,
        ) * (0.7 + band * 0.3);
        let point = |angle: f32, age: f32| {
            center
                + vec2(angle.cos() * radius.x, angle.sin() * radius.y) * age.sqrt()
                + vec2(0.0, age.powi(2) * rect.height() * 0.08)
        };
        let fade = (1.0 - age).sqrt() * energy;
        let colour = swept(colours, at);
        for spark in 0..18 {
            let angle = spark as f32 / 18.0 * std::f32::consts::TAU + burst as f32;
            let head = point(angle, age);
            let tail = point(angle, (age - 0.14).max(0.0));
            let light = colour.gamma_multiply(strength * fade * (0.6 + band * 0.4));
            painter.line_segment(
                [tail, head],
                egui::Stroke::new(1.8, light.gamma_multiply(0.85)),
            );
            painter.circle_filled(head, 3.8 + band * 2.0, light.gamma_multiply(0.12));
            painter.circle_filled(head, 1.2 + band * 1.8, light);
        }
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
        ctx.animate_value_with_time(
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

    super::widgets::playing_cover(
        ui,
        &palette,
        app.backend.art(),
        "player-bar",
        app.small_art(now, cover_rect.width()),
        cover_rect,
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
        for mode in [
            PlayerBarVis::Aurora,
            PlayerBarVis::Radial,
            PlayerBarVis::Orbit,
            PlayerBarVis::Embers,
            PlayerBarVis::Tunnel,
            PlayerBarVis::Waterfall,
            PlayerBarVis::Prism,
            PlayerBarVis::Helix,
            PlayerBarVis::Starfield,
            PlayerBarVis::Plasma,
            PlayerBarVis::Ripples,
            PlayerBarVis::Lasers,
            PlayerBarVis::Fireworks,
        ] {
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

    const NEW_SCENES: [PlayerBarVis; 8] = [
        PlayerBarVis::Waterfall,
        PlayerBarVis::Prism,
        PlayerBarVis::Helix,
        PlayerBarVis::Starfield,
        PlayerBarVis::Plasma,
        PlayerBarVis::Ripples,
        PlayerBarVis::Lasers,
        PlayerBarVis::Fireworks,
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
                        "{mode:?} at {time} escaped its bar"
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
    fn spectrum_interpolation_handles_edges_and_empty_input() {
        assert_eq!(level_at(&[], 0.5), 0.0);
        assert_eq!(level_at(&[0.4], 0.7), 0.4);
        assert_eq!(level_at(&[0.0, 1.0], 0.25), 0.25);
        assert_eq!(level_at(&[0.2, 0.8], -0.5), 0.2);
        assert_eq!(level_at(&[0.2, 0.8], 1.5), 0.8);
        assert_eq!(mean_level(&[]), 0.0);
    }
}
