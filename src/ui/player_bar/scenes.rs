//! The visualiser scenes added in v17: a city skyline, ocean waves, a retro
//! LED meter, fireflies, a heart monitor and a circuit board. Like the other
//! scenes they share the analyser's levels and its time-based decay, paint
//! nothing in silence, and keep to the rect they are given.

use super::{scene_band, scene_energy, scene_neon_line, shaded, swept};
use crate::theme;
use egui::{Color32, Rect, pos2, vec2};
use std::f32::consts::TAU;

/// A fixed pseudo-random number from 0 to 1 for `seed`.
fn hash(seed: f32) -> f32 {
    ((seed * 12.9898).sin() * 43_758.547).fract().abs()
}

/// A city at night: buildings rise with their part of the spectrum, their
/// windows lighting up and twinkling as it gets louder, over a glow on the
/// horizon.
pub(super) fn skyline(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = scene_energy(levels);
    let mut mesh = egui::Mesh::default();
    shaded(
        &mut mesh,
        Rect::from_min_max(pos2(rect.left(), rect.center().y), rect.right_bottom()),
        Color32::TRANSPARENT,
        swept(colours, 0.0).gamma_multiply(0.16 * strength * energy),
    );
    let buildings = ((rect.width() / 34.0) as usize).clamp(8, 48);
    let slot = rect.width() / buildings as f32;
    for building in 0..buildings {
        let seed = building as f32;
        let at = (seed + 0.5) / buildings as f32;
        let level = scene_band(levels, at, energy);
        let shape = hash(seed + 1.0);
        let width = slot * (0.62 + shape * 0.3);
        let height = rect.height() * (0.22 + shape * 0.18 + level * 0.5);
        let left = rect.left() + seed * slot + (slot - width) / 2.0;
        let body = Rect::from_min_max(
            pos2(left, rect.bottom() - height),
            pos2(left + width, rect.bottom()),
        );
        let colour = swept(colours, at);
        shaded(
            &mut mesh,
            body,
            colour.gamma_multiply(0.3 * strength * energy),
            colour.gamma_multiply(0.08 * strength * energy),
        );
        // A lit roof line.
        mesh.add_colored_rect(
            Rect::from_min_size(body.min, vec2(width, 1.5)),
            colour.gamma_multiply(0.9 * strength * level),
        );
        let columns = ((width / 6.0) as usize).clamp(1, 4);
        let rows = (((height - 6.0) / 7.0).max(0.0) as usize).min(10);
        let pane = vec2(((width - 4.0) / columns as f32 - 2.0).max(1.0), 3.0);
        let window = theme::motion::blend(colour, Color32::WHITE, 0.45);
        for row in 0..rows {
            for column in 0..columns {
                let lamp = hash(seed * 7.1 + row as f32 * 3.3 + column as f32 * 1.7);
                // Louder music lights more of the windows.
                if lamp > 0.25 + level * 0.6 {
                    continue;
                }
                let twinkle = 0.7 + 0.3 * (time * 1.8 + lamp * 40.0).sin();
                let min = pos2(
                    left + 2.0 + column as f32 * (width - 4.0) / columns as f32,
                    body.top() + 5.0 + row as f32 * 7.0,
                );
                mesh.add_colored_rect(
                    Rect::from_min_size(min, pane),
                    window.gamma_multiply(strength * energy * twinkle * (0.3 + 0.6 * level)),
                );
            }
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// Four layers of waves rolling in like the sea, each swelling with the part
/// of the spectrum beneath it, with foam along every crest.
pub(super) fn ocean(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = scene_energy(levels);
    let steps = ((rect.width() / 12.0) as usize).clamp(24, 120);
    for layer in 0..4 {
        let depth = layer as f32;
        let base = rect.top() + rect.height() * (0.38 + depth * 0.14);
        let colour = swept(colours, depth / 3.0);
        let crest: Vec<_> = (0..=steps)
            .map(|step| {
                let along = step as f32 / steps as f32;
                let level = scene_band(levels, along, energy);
                let swell = rect.height() * (0.05 + level * 0.14);
                let roll = along * TAU * (2.0 + depth * 0.5) - time * (0.6 + depth * 0.25);
                let wave = roll.sin() * 0.7 + (roll * 2.3 + depth).sin() * 0.3;
                let y = (base - wave * swell).clamp(rect.top() + 3.0, rect.bottom() - 2.0);
                pos2(rect.left() + along * rect.width(), y)
            })
            .collect();
        // The water below the crest, deepest at the front.
        let mut mesh = egui::Mesh::default();
        let top = colour.gamma_multiply((0.12 + depth * 0.05) * strength * energy);
        let bottom = colour.gamma_multiply(0.04 * strength * energy);
        for pair in crest.windows(2) {
            let index = mesh.vertices.len() as u32;
            mesh.colored_vertex(pair[0], top);
            mesh.colored_vertex(pair[1], top);
            mesh.colored_vertex(pos2(pair[1].x, rect.bottom()), bottom);
            mesh.colored_vertex(pos2(pair[0].x, rect.bottom()), bottom);
            mesh.add_triangle(index, index + 1, index + 2);
            mesh.add_triangle(index, index + 2, index + 3);
        }
        painter.add(egui::Shape::mesh(mesh));
        let foam = theme::motion::blend(colour, Color32::WHITE, 0.35);
        painter.add(egui::Shape::line(
            crest.clone(),
            egui::Stroke::new(4.0, foam.gamma_multiply(0.12 * strength * energy)),
        ));
        painter.add(egui::Shape::line(
            crest,
            egui::Stroke::new(
                1.4,
                foam.gamma_multiply((0.45 + depth * 0.15) * strength * energy),
            ),
        ));
    }
}

/// A retro hi-fi meter: columns of LED segments light up from the bottom
/// with their part of the spectrum, a gentle shimmer running across them.
pub(super) fn led_meter(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    const SEGMENTS: usize = 12;
    let energy = scene_energy(levels);
    let columns = ((rect.width() / 22.0) as usize).clamp(10, 64);
    let cell = vec2(
        rect.width() / columns as f32,
        rect.height() / SEGMENTS as f32,
    );
    let mut mesh = egui::Mesh::default();
    for column in 0..columns {
        let at = (column as f32 + 0.5) / columns as f32;
        let level = scene_band(levels, at, energy);
        let lit = (level * SEGMENTS as f32).ceil() as usize;
        let shimmer = 0.85 + 0.15 * (time * 3.0 + column as f32 * 0.5).sin();
        for segment in 0..SEGMENTS {
            let rise = segment as f32 / (SEGMENTS - 1) as f32;
            // Low segments take the bass colour and high ones the treble's,
            // as a hi-fi meter turns from green to red.
            let colour = swept(colours, rise);
            let alpha = if segment < lit {
                strength * energy * shimmer * (0.35 + 0.6 * level)
            } else {
                strength * energy * 0.06
            };
            let min = pos2(
                rect.left() + column as f32 * cell.x + 1.5,
                rect.bottom() - (segment + 1) as f32 * cell.y + 1.0,
            );
            mesh.add_colored_rect(
                Rect::from_min_size(min, cell - vec2(3.0, 2.0)),
                colour.gamma_multiply(alpha),
            );
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// Fireflies drifting in the dark, each blinking softly and glowing brighter
/// with its own part of the spectrum.
pub(super) fn fireflies(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = scene_energy(levels);
    // Bigger on the stage than in the player bar.
    let scale = (rect.height() / 88.0).clamp(1.0, 2.5);
    let margin = 12.0 * scale;
    let span = (rect.width() - margin * 2.0).max(0.0);
    for fly in 0..60 {
        let seed = fly as f32;
        let at = (seed * 0.618_034).fract();
        let level = scene_band(levels, at, energy);
        let pace = 0.6 + hash(seed) * 0.8;
        let centre = pos2(
            rect.left()
                + margin
                + at * span
                + (time * 0.3 * pace + seed).sin() * span.min(600.0) * 0.03,
            rect.center().y + (time * 0.4 * pace + seed * 2.0).sin() * rect.height() * 0.32,
        );
        let blink = 0.5 + 0.5 * (time * (1.5 + hash(seed + 3.0) * 2.0) + seed * 5.0).sin();
        let glow = (0.45 + 0.55 * blink) * (0.4 + 0.6 * level) * energy * strength;
        let colour = theme::motion::blend(swept(colours, at), Color32::WHITE, 0.3);
        painter.circle_filled(
            centre,
            (3.0 + level * 6.0) * scale,
            colour.gamma_multiply(0.14 * glow),
        );
        painter.circle_filled(
            centre,
            (1.2 + level * 1.6) * scale,
            colour.gamma_multiply((glow * 1.2).min(1.0)),
        );
    }
}

/// The height of one heartbeat at `phase` through it: a small P wave, the
/// sharp QRS spike, then a broad T wave, each sized by its own part of the
/// spectrum.
fn heartbeat_shape(phase: f32, bass: f32, middle: f32, treble: f32) -> f32 {
    let bump = |centre: f32, width: f32| (-((phase - centre) / width).powi(2)).exp();
    0.14 * middle * bump(0.15, 0.04) - 0.15 * bump(0.3, 0.012)
        + (0.45 + 0.55 * bass) * bump(0.33, 0.014)
        - 0.3 * bump(0.36, 0.012)
        + 0.28 * treble * bump(0.6, 0.06)
}

/// A heart monitor: a glowing trace sweeps across a faint grid, its spikes
/// leaping with the bass and its gentler waves with the rest of the music.
pub(super) fn heartbeat(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    let energy = scene_energy(levels);
    let grid = swept(colours, 0.5).gamma_multiply(0.07 * strength * energy);
    let mut mesh = egui::Mesh::default();
    for row in 0..=4 {
        let y = rect.top() + 1.0 + row as f32 * (rect.height() - 2.0) / 4.0;
        mesh.add_colored_rect(
            Rect::from_min_max(pos2(rect.left(), y - 0.5), pos2(rect.right(), y + 0.5)),
            grid,
        );
    }
    let mut x = rect.left();
    while x <= rect.right() {
        mesh.add_colored_rect(
            Rect::from_min_max(pos2(x - 0.5, rect.top()), pos2(x + 0.5, rect.bottom())),
            grid,
        );
        x += 40.0;
    }
    painter.add(egui::Shape::mesh(mesh));

    let steps = ((rect.width() / 4.0) as usize).clamp(60, 360);
    let reach = rect.height() * 0.42;
    let head = (time * 0.25).fract();
    let points: Vec<_> = (0..=steps)
        .map(|step| {
            let along = step as f32 / steps as f32;
            // Three beats across, the newest written at the head.
            let phase = (along * 3.0 - time * 0.75).rem_euclid(1.0);
            let bass = scene_band(levels, 0.05 + along * 0.1, energy);
            let middle = scene_band(levels, 0.3 + along * 0.3, energy);
            let treble = scene_band(levels, 0.65 + along * 0.3, energy);
            let height = heartbeat_shape(phase, bass, middle, treble).clamp(-1.0, 1.0);
            pos2(
                rect.left() + along * rect.width(),
                rect.center().y - height * reach,
            )
        })
        .collect();
    // The trace fades behind the sweeping head, as on a real monitor.
    let run = 8;
    let mut start = 0;
    while start < steps {
        let end = (start + run).min(steps);
        let along = (start + end) as f32 / 2.0 / steps as f32;
        let behind = (head - along).rem_euclid(1.0);
        let fade = 0.25 + 0.75 * (1.0 - behind).powi(2);
        scene_neon_line(
            painter,
            points[start..=end].to_vec(),
            swept(colours, along),
            strength * energy * fade,
        );
        start = end;
    }
    let tip = points[((head * steps as f32) as usize).min(steps)];
    painter.circle_filled(
        tip,
        3.5,
        theme::motion::blend(swept(colours, head), Color32::WHITE, 0.6)
            .gamma_multiply(strength * energy),
    );
}

/// The path of one circuit trace across `rect`: straight runs joined by
/// diagonal jogs, the same every frame.
fn circuit_path(rect: Rect, track: usize, tracks: usize) -> Vec<egui::Pos2> {
    let lane = rect.height() / tracks as f32;
    let home = rect.top() + lane * (track as f32 + 0.5);
    let step = (rect.width() / 14.0).clamp(30.0, 110.0);
    let mut points = vec![pos2(rect.left(), home)];
    let mut x = rect.left();
    let mut y = home;
    let mut turn = 0.0;
    while x < rect.right() {
        turn += 1.0;
        let run = step * (0.6 + hash(track as f32 * 5.3 + turn) * 0.8);
        x = (x + run).min(rect.right());
        points.push(pos2(x, y));
        if x >= rect.right() {
            break;
        }
        // Jog up or down half a lane, then back home later.
        let target = if (y - home).abs() > 0.5 {
            home
        } else if hash(track as f32 * 2.1 + turn * 1.3) > 0.5 {
            home + lane * 0.45
        } else {
            home - lane * 0.45
        };
        let rise = (target - y).abs();
        x = (x + rise).min(rect.right());
        y = target;
        points.push(pos2(x, y));
    }
    points.dedup_by(|a, b| a.distance(*b) < 0.5);
    points
}

/// The point `along` (0 to 1) the way down `path` by length.
fn along_path(path: &[egui::Pos2], lengths: &[f32], along: f32) -> egui::Pos2 {
    let total = lengths.last().copied().unwrap_or(0.0);
    let wanted = along.clamp(0.0, 1.0) * total;
    let index = lengths
        .iter()
        .position(|length| *length >= wanted)
        .unwrap_or(lengths.len() - 1)
        .max(1);
    let (from, to) = (path[index - 1], path[index]);
    let span = (lengths[index] - lengths[index - 1]).max(f32::EPSILON);
    from + (to - from) * ((wanted - lengths[index - 1]) / span).clamp(0.0, 1.0)
}

/// A circuit board: pulses of light race along the traces, faster with
/// their part of the spectrum, and the pads glow with the band beneath them.
pub(super) fn circuit(
    painter: &egui::Painter,
    rect: Rect,
    levels: &[f32],
    colours: &[Color32],
    strength: f32,
    time: f32,
) {
    const TRACKS: usize = 6;
    let energy = scene_energy(levels);
    let inner = rect.shrink2(vec2(2.0, 4.0));
    for track in 0..TRACKS {
        let at = (track as f32 + 0.5) / TRACKS as f32;
        let level = scene_band(levels, at, energy);
        let colour = swept(colours, at);
        let path = circuit_path(inner, track, TRACKS);
        painter.add(egui::Shape::line(
            path.clone(),
            egui::Stroke::new(1.2, colour.gamma_multiply(0.16 * strength * energy)),
        ));
        // Pads at every corner, lit by the band beneath them.
        for point in &path[1..path.len() - 1] {
            let under = scene_band(levels, (point.x - rect.left()) / rect.width(), energy);
            painter.circle_filled(
                *point,
                2.2,
                colour.gamma_multiply(strength * energy * (0.12 + 0.6 * under * under)),
            );
        }
        let mut lengths = vec![0.0];
        for pair in path.windows(2) {
            lengths.push(lengths.last().copied().unwrap_or(0.0) + pair[0].distance(pair[1]));
        }
        let speed = 0.05 + 0.04 * hash(track as f32 + 9.0);
        for packet in 0..3 {
            let head = (time * speed + packet as f32 / 3.0 + hash(track as f32)).fract();
            let mut trail: Vec<_> = (0..=6)
                .map(|step| along_path(&path, &lengths, (head - step as f32 * 0.012).max(0.0)))
                .collect();
            // Near the start the trail's tail bunches up on one point; a
            // line through a repeated point has no direction to draw along.
            trail.dedup_by(|a, b| a.distance(*b) < 0.5);
            if trail.len() >= 2 {
                scene_neon_line(
                    painter,
                    trail,
                    colour,
                    strength * energy * (0.45 + 0.55 * level),
                );
            }
            painter.circle_filled(
                along_path(&path, &lengths, head),
                1.5 + level * 1.5,
                theme::motion::blend(colour, Color32::WHITE, 0.5)
                    .gamma_multiply(strength * energy * (0.5 + 0.5 * level)),
            );
        }
    }
}
