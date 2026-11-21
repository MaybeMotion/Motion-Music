//! Click effects and box shapes, as chosen in Settings > Appearance.
//!
//! A click on a page link, playlist, card or song is noted here, and its
//! effect is drawn above the page for a moment, in the visualiser's colours.
//! Box shapes change the corners of cards, covers and highlighted rows.

use crate::settings::{BoxShape, ClickEffect, RowHover};
use egui::{Color32, Context, CornerRadius, Id, Pos2, Rect, Stroke, pos2, vec2};
use std::f32::consts::TAU;

/// How long a click's effect lasts, in seconds. Confetti takes a little
/// longer to fall.
const SECONDS: f64 = 0.55;
const CONFETTI_SECONDS: f64 = 0.9;

/// The last click, while its effect is shown.
#[derive(Clone, Copy, Debug)]
struct Click {
    rect: Rect,
    at: Pos2,
    start: f64,
    corner: CornerRadius,
}

fn click_id() -> Id {
    Id::new("motion-click-effect")
}

fn effect_id() -> Id {
    Id::new("motion-click-effect-style")
}

fn shape_id() -> Id {
    Id::new("motion-box-shape")
}

fn hover_id() -> Id {
    Id::new("motion-row-hover")
}

/// Takes this frame's choices from the settings, before anything is drawn.
pub fn configure(ctx: &Context, effect: ClickEffect, shape: BoxShape, hover: RowHover) {
    ctx.data_mut(|data| {
        data.insert_temp(effect_id(), effect);
        data.insert_temp(shape_id(), shape);
        data.insert_temp(hover_id(), hover);
    });
}

/// A song row's hover highlight, `hover` (0 to 1) of the way in, in the
/// style chosen in Settings > Customise: the usual `fill`, an outline, an
/// accent bar or a glow in `accent`.
pub fn row_hover(
    ui: &egui::Ui,
    rect: Rect,
    corner: CornerRadius,
    fill: Color32,
    accent: Color32,
    hover: f32,
) {
    let style = ui
        .ctx()
        .data(|data| data.get_temp::<RowHover>(hover_id()))
        .unwrap_or_default();
    let painter = ui.painter();
    match style {
        RowHover::Fill => {
            painter.rect_filled(rect, corner, fill);
        }
        RowHover::Outline => {
            painter.rect_filled(rect, corner, fill.gamma_multiply(0.4));
            painter.rect_stroke(
                rect.shrink(0.75),
                corner,
                Stroke::new(1.5, accent.gamma_multiply(0.8 * hover)),
                egui::StrokeKind::Inside,
            );
        }
        RowHover::Bar => {
            painter.rect_filled(rect, corner, fill.gamma_multiply(0.6));
            let bar = Rect::from_min_size(
                rect.left_top() + vec2(0.0, rect.height() * 0.2),
                vec2(3.0, rect.height() * 0.6),
            );
            painter.rect_filled(bar, 1.5, accent.gamma_multiply(hover));
        }
        RowHover::Glow => {
            painter.add(
                egui::epaint::Shadow {
                    offset: [0, 0],
                    blur: 14,
                    spread: 0,
                    color: accent.gamma_multiply(0.35 * hover),
                }
                .as_shape(rect, corner),
            );
            painter.rect_filled(rect, corner, fill);
        }
    }
}

fn effect(ctx: &Context) -> ClickEffect {
    ctx.data(|data| data.get_temp(effect_id()))
        .unwrap_or_default()
}

/// The chosen box shape.
pub fn shape(ctx: &Context) -> BoxShape {
    ctx.data(|data| data.get_temp(shape_id()))
        .unwrap_or_default()
}

/// A corner usually `base` points round, in the chosen shape. Leaf's round
/// corners are this size.
pub fn radius(ui: &egui::Ui, base: f32) -> f32 {
    shape(ui.ctx()).radius(base)
}

fn corner_of(shape: BoxShape, radius: f32) -> CornerRadius {
    let [nw, ne, sw, se] = shape
        .corners(radius)
        .map(|corner| corner.round().clamp(0.0, 127.0) as u8);
    CornerRadius { nw, ne, sw, se }
}

/// The corners of a box usually `base` points round, in the chosen shape.
pub fn corner(ui: &egui::Ui, base: f32) -> CornerRadius {
    let shape = shape(ui.ctx());
    corner_of(shape, shape.radius(base))
}

/// The corners of a cover `radius` points round. A round cover, such as an
/// artist's portrait, stays round; others take the chosen shape's pattern,
/// so Leaf covers match Leaf cards.
pub fn cover_corner(ui: &egui::Ui, rect: Rect, radius: f32) -> CornerRadius {
    let shape = shape(ui.ctx());
    let side = rect.width().min(rect.height());
    // A Pill cover is a soft rounded square: a circle would read as an
    // artist's portrait.
    if shape == BoxShape::Pill && radius >= BoxShape::Pill.radius(0.0) - 0.5 {
        return CornerRadius::same((side * 0.3).round().clamp(0.0, 127.0) as u8);
    }
    if radius >= side / 2.0 - 0.5 {
        return CornerRadius::same(radius.round().clamp(0.0, 127.0) as u8);
    }
    corner_of(shape, radius)
}

/// Notes a click on `response`, whose box has corners usually `base`
/// points round, so its effect can be drawn.
pub fn note(ui: &egui::Ui, response: &egui::Response, base: f32) {
    if !response.clicked() {
        return;
    }
    let ctx = ui.ctx();
    if effect(ctx) == ClickEffect::Off || crate::motion::reduced(ctx) {
        return;
    }
    let click = Click {
        rect: response.rect,
        // A key press has no pointer: it starts from the middle.
        at: response
            .interact_pointer_pos()
            .filter(|at| response.rect.contains(*at))
            .unwrap_or(response.rect.center()),
        start: ctx.input(|input| input.time),
        corner: corner(ui, base),
    };
    ctx.data_mut(|data| data.insert_temp(click_id(), click));
    ctx.request_repaint();
}

/// A fixed pseudo-random number from 0 to 1 for `seed`.
fn hash(seed: f32) -> f32 {
    ((seed * 12.9898).sin() * 43_758.547).fract().abs()
}

/// The nearest point to `point` inside `rect` with `corner`s, so a shape
/// drawn through it keeps within the box's rounded corners.
pub(crate) fn inside(rect: Rect, corner: CornerRadius, point: Pos2) -> Pos2 {
    let point = point.clamp(rect.min, rect.max);
    let most = rect.width().min(rect.height()) / 2.0;
    let round = |value: u8| f32::from(value).min(most);
    let corners = [
        (round(corner.nw), -1.0, -1.0),
        (round(corner.ne), 1.0, -1.0),
        (round(corner.sw), -1.0, 1.0),
        (round(corner.se), 1.0, 1.0),
    ];
    for (radius, sx, sy) in corners {
        if radius <= 0.0 {
            continue;
        }
        let x = if sx < 0.0 {
            rect.left() + radius
        } else {
            rect.right() - radius
        };
        let y = if sy < 0.0 {
            rect.top() + radius
        } else {
            rect.bottom() - radius
        };
        let centre = pos2(x, y);
        let outside = (point.x - x) * sx > 0.0 && (point.y - y) * sy > 0.0;
        let offset = point - centre;
        if outside && offset.length() > radius {
            return centre + offset.normalized() * radius;
        }
    }
    point
}

/// Draws the last click's effect above the page while it lasts, in
/// `colours` swept from the left of the window to the right, as the
/// visualisers are.
pub fn paint(ctx: &Context, colours: &[Color32]) {
    let Some(click) = ctx.data(|data| data.get_temp::<Click>(click_id())) else {
        return;
    };
    let style = effect(ctx);
    let seconds = if style == ClickEffect::Confetti {
        CONFETTI_SECONDS
    } else {
        SECONDS
    };
    let now = ctx.input(|input| input.time);
    let t = ((now - click.start) / seconds) as f32;
    if !(0.0..1.0).contains(&t) || style == ClickEffect::Off || colours.is_empty() {
        ctx.data_mut(|data| data.remove::<Click>(click_id()));
        return;
    }
    ctx.request_repaint();
    // Eases out: quick at first, then settling softly.
    let ease = 1.0 - (1.0 - t).powi(3);
    let fade = (1.0 - t).powi(2);
    let window = ctx.content_rect();
    let at = |x: f32| ((x - window.left()) / window.width().max(1.0)).clamp(0.0, 1.0);
    let colour = swept(colours, at(click.at.x));
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        Id::new("motion-click-effect-layer"),
    ));
    let grown = |grow: f32| {
        let add = |corner: u8| (f32::from(corner) + grow).round().clamp(0.0, 127.0) as u8;
        CornerRadius {
            nw: add(click.corner.nw),
            ne: add(click.corner.ne),
            sw: add(click.corner.sw),
            se: add(click.corner.se),
        }
    };
    match style {
        ClickEffect::Off => {}
        ClickEffect::Ripple => {
            // A disc of light, brightest at its spreading edge, kept inside
            // the item's own corners.
            let reach = [
                click.rect.left_top(),
                click.rect.right_top(),
                click.rect.left_bottom(),
                click.rect.right_bottom(),
            ]
            .into_iter()
            .map(|corner| corner.distance(click.at))
            .fold(0.0_f32, f32::max);
            let size = reach * ease;
            let mut mesh = egui::Mesh::default();
            mesh.colored_vertex(click.at, colour.gamma_multiply(0.1 * fade));
            let steps = 48;
            for step in 0..=steps {
                let angle = step as f32 / steps as f32 * TAU;
                let edge = click.at + vec2(angle.cos(), angle.sin()) * size;
                mesh.colored_vertex(
                    inside(click.rect, click.corner, edge),
                    colour.gamma_multiply(0.32 * fade),
                );
                if step > 0 {
                    mesh.add_triangle(0, step, step + 1);
                }
            }
            painter.add(egui::Shape::mesh(mesh));
        }
        ClickEffect::Glow => {
            painter.rect_filled(click.rect, click.corner, colour.gamma_multiply(0.2 * fade));
            for ring in 1..=3 {
                let grow = ring as f32 * 2.0 * ease;
                painter.rect_stroke(
                    click.rect.expand(grow),
                    grown(grow),
                    Stroke::new(2.0, colour.gamma_multiply(0.14 / ring as f32 * fade)),
                    egui::StrokeKind::Outside,
                );
            }
        }
        ClickEffect::Pop => {
            let grow = 10.0 * ease;
            painter.rect_stroke(
                click.rect.expand(grow),
                grown(grow),
                Stroke::new(2.0, colour.gamma_multiply(0.6 * fade)),
                egui::StrokeKind::Outside,
            );
        }
        ClickEffect::Sparkle => {
            let seed_base = (click.start.fract() * 10.0) as f32;
            for index in 0..10 {
                let seed = index as f32 + seed_base;
                let angle = index as f32 / 10.0 * TAU + hash(seed) * 0.6;
                let distance = (10.0 + 22.0 * hash(seed + 1.0)) * ease + 4.0;
                let place = click.at + vec2(angle.cos(), angle.sin()) * distance;
                let spark = crate::theme::motion::blend(
                    swept(colours, index as f32 / 9.0),
                    Color32::WHITE,
                    0.4,
                );
                painter.circle_filled(place, 0.6 + 2.2 * (1.0 - t), spark.gamma_multiply(fade));
            }
        }
        ClickEffect::Rings => {
            // Three rings, each setting off a moment after the last.
            for ring in 0..3 {
                let start = ring as f32 * 0.18;
                let local = ((t - start) / (1.0 - start)).clamp(0.0, 1.0);
                if local <= 0.0 {
                    continue;
                }
                let grow = 1.0 - (1.0 - local).powi(3);
                let light = (1.0 - local).powi(2);
                painter.circle_stroke(
                    click.at,
                    4.0 + 38.0 * grow,
                    Stroke::new(
                        2.0 - ring as f32 * 0.4,
                        swept(colours, ring as f32 / 2.0).gamma_multiply(0.6 * light),
                    ),
                );
            }
        }
        ClickEffect::Bloom => {
            // A soft disc with no edge at all: bright at the heart, fading
            // to nothing.
            let size = 10.0 + 46.0 * ease;
            let mut mesh = egui::Mesh::default();
            let heart = crate::theme::motion::blend(colour, Color32::WHITE, 0.25);
            mesh.colored_vertex(click.at, heart.gamma_multiply(0.65 * fade));
            let steps = 40;
            for step in 0..=steps {
                let angle = step as f32 / steps as f32 * TAU;
                // Petal-like: the edge swells and dips five times around.
                let petal = 1.0 + 0.12 * (angle * 5.0 + t * 2.0).sin();
                mesh.colored_vertex(
                    click.at + vec2(angle.cos(), angle.sin()) * size * petal,
                    Color32::TRANSPARENT,
                );
                if step > 0 {
                    mesh.add_triangle(0, step, step + 1);
                }
            }
            painter.add(egui::Shape::mesh(mesh));
        }
        ClickEffect::Confetti => {
            let seed_base = (click.start.fract() * 10.0) as f32;
            let mut mesh = egui::Mesh::default();
            for piece in 0..16 {
                let seed = piece as f32 * 1.7 + seed_base;
                let angle = -TAU / 4.0 + (hash(seed) - 0.5) * 2.2;
                let speed = 70.0 + 70.0 * hash(seed + 2.0);
                let seconds = t * CONFETTI_SECONDS as f32;
                // Thrown up and out, then falling.
                let place = click.at
                    + vec2(angle.cos(), angle.sin()) * speed * seconds
                    + vec2(0.0, 160.0 * seconds * seconds);
                let spin = seed * 3.0 + seconds * (6.0 + 6.0 * hash(seed + 4.0));
                let (half_w, half_h) = (3.0, 1.6 + 1.4 * spin.cos().abs());
                let piece_colour = swept(colours, hash(seed + 6.0)).gamma_multiply(fade.sqrt());
                let base = mesh.vertices.len() as u32;
                let (sin, cos) = spin.sin_cos();
                for (x, y) in [
                    (-half_w, -half_h),
                    (half_w, -half_h),
                    (half_w, half_h),
                    (-half_w, half_h),
                ] {
                    mesh.colored_vertex(
                        place + vec2(x * cos - y * sin, x * sin + y * cos),
                        piece_colour,
                    );
                }
                mesh.add_triangle(base, base + 1, base + 2);
                mesh.add_triangle(base, base + 2, base + 3);
            }
            painter.add(egui::Shape::mesh(mesh));
        }
        ClickEffect::Wave => {
            // A band of light with a rolling, curved front washes across
            // the item from where it was clicked, kept inside its corners.
            let rect = click.rect;
            let travel = rect.width() * 1.4;
            let front = rect.left() + (click.at.x - rect.left()) * (1.0 - ease) + travel * ease;
            let band = (rect.width() * 0.35).max(24.0);
            let rows = 16;
            let mut mesh = egui::Mesh::default();
            for row in 0..=rows {
                let y = rect.top() + rect.height() * row as f32 / rows as f32;
                let curl = (row as f32 / rows as f32 * TAU * 0.75 + t * 6.0).sin() * 8.0;
                let lead = front + curl;
                let tail = lead - band;
                mesh.colored_vertex(
                    inside(rect, click.corner, pos2(tail, y)),
                    Color32::TRANSPARENT,
                );
                mesh.colored_vertex(
                    inside(rect, click.corner, pos2(lead, y)),
                    swept(colours, at(lead)).gamma_multiply(0.38 * fade),
                );
                // The crest itself softens out ahead, like foam, rather
                // than ending in a hard line.
                mesh.colored_vertex(
                    inside(rect, click.corner, pos2(lead + 12.0, y)),
                    Color32::TRANSPARENT,
                );
                if row > 0 {
                    let base = (row * 3) as u32;
                    let above = base - 3;
                    for lane in 0..2 {
                        mesh.add_triangle(above + lane, above + lane + 1, base + lane + 1);
                        mesh.add_triangle(above + lane, base + lane + 1, base + lane);
                    }
                }
            }
            painter.add(egui::Shape::mesh(mesh));
        }
    }
}

/// The colour `at` the given fraction of the way along `colours`.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn click() -> Click {
        Click {
            rect: Rect::from_min_size(Pos2::ZERO, vec2(100.0, 40.0)),
            at: pos2(20.0, 20.0),
            start: 0.0,
            corner: CornerRadius::same(12),
        }
    }

    #[test]
    fn every_effect_draws_then_clears() {
        let ctx = Context::default();
        configure(&ctx, ClickEffect::Pop, BoxShape::Soft, RowHover::Fill);
        assert_eq!(shape(&ctx), BoxShape::Soft);
        let colours = [Color32::RED, Color32::BLUE];
        for style in ClickEffect::ALL {
            configure(&ctx, style, BoxShape::Rounded, RowHover::Fill);
            for time in [0.05, 0.3] {
                let input = egui::RawInput {
                    time: Some(time),
                    ..Default::default()
                };
                let mut output = ctx.run_ui(input, |ui| {
                    ui.ctx()
                        .data_mut(|data| data.insert_temp(click_id(), click()));
                    paint(ui.ctx(), &colours);
                });
                // This headless check does not upload textures.
                output.textures_delta.clear();
                let drew = !output.shapes.is_empty();
                assert_eq!(drew, style != ClickEffect::Off, "{style:?} at {time}");
            }
        }
        // Long after, it is forgotten.
        let late = egui::RawInput {
            time: Some(5.0),
            ..Default::default()
        };
        ctx.run_ui(late, |ui| paint(ui.ctx(), &colours))
            .textures_delta
            .clear();
        assert!(
            ctx.data(|data| data.get_temp::<Click>(click_id()))
                .is_none()
        );
    }

    #[test]
    fn shapes_give_their_corners_and_keep_effects_inside_them() {
        assert_eq!(corner_of(BoxShape::Rounded, 8.0), CornerRadius::same(8));
        assert_eq!(corner_of(BoxShape::Square, 0.0), CornerRadius::ZERO);
        let leaf = corner_of(BoxShape::Leaf, BoxShape::Leaf.radius(8.0));
        assert!(leaf.nw > leaf.ne && leaf.se > leaf.sw, "{leaf:?}");
        assert_eq!(
            corner_of(BoxShape::Pill, BoxShape::Pill.radius(8.0)),
            CornerRadius::same(127)
        );

        let rect = Rect::from_min_size(Pos2::ZERO, vec2(100.0, 40.0));
        let round = CornerRadius::same(20);
        // Points well inside stay put; the very corner is pulled onto the
        // rounded edge; points beyond the box come back to its edge.
        assert_eq!(inside(rect, round, pos2(50.0, 20.0)), pos2(50.0, 20.0));
        let corner = inside(rect, round, pos2(0.0, 0.0));
        assert!(
            (corner.distance(pos2(20.0, 20.0)) - 20.0).abs() < 1e-3,
            "{corner:?}"
        );
        assert_eq!(inside(rect, round, pos2(50.0, -30.0)), pos2(50.0, 0.0));
        assert_eq!(
            inside(rect, CornerRadius::ZERO, pos2(-5.0, -5.0)),
            pos2(0.0, 0.0)
        );
    }
}
