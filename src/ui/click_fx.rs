//! Click effects and box shapes, as chosen in Settings > Appearance.
//!
//! A click on a page link, playlist, card or song is noted here, and its
//! effect is drawn above the page for half a second. Box shapes change the
//! corners of cards, covers and highlighted rows.

use crate::settings::{BoxShape, ClickEffect};
use egui::{Color32, Context, CornerRadius, Id, Pos2, Rect, Stroke};

/// How long a click's effect lasts, in seconds.
const SECONDS: f64 = 0.5;

/// The last click, while its effect is shown.
#[derive(Clone, Copy, Debug)]
struct Click {
    rect: Rect,
    at: Pos2,
    start: f64,
    radius: f32,
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

/// Takes this frame's choices from the settings, before anything is drawn.
pub fn configure(ctx: &Context, effect: ClickEffect, shape: BoxShape) {
    ctx.data_mut(|data| {
        data.insert_temp(effect_id(), effect);
        data.insert_temp(shape_id(), shape);
    });
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

/// A corner usually `base` points round, in the chosen shape.
pub fn radius(ui: &egui::Ui, base: f32) -> f32 {
    shape(ui.ctx()).radius(base)
}

/// [`radius`] as egui corners.
pub fn corner(ui: &egui::Ui, base: f32) -> CornerRadius {
    CornerRadius::same(radius(ui, base).round().clamp(0.0, 127.0) as u8)
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
        at: response
            .interact_pointer_pos()
            .unwrap_or(response.rect.center()),
        start: ctx.input(|input| input.time),
        radius: radius(ui, base),
    };
    ctx.data_mut(|data| data.insert_temp(click_id(), click));
    ctx.request_repaint();
}

/// A fixed pseudo-random number from 0 to 1 for `seed`.
fn hash(seed: f32) -> f32 {
    ((seed * 12.9898).sin() * 43_758.547).fract().abs()
}

/// Draws the last click's effect above the page while it lasts.
pub fn paint(ctx: &Context, accent: Color32) {
    let Some(click) = ctx.data(|data| data.get_temp::<Click>(click_id())) else {
        return;
    };
    let now = ctx.input(|input| input.time);
    let t = ((now - click.start) / SECONDS) as f32;
    let style = effect(ctx);
    if !(0.0..1.0).contains(&t) || style == ClickEffect::Off {
        ctx.data_mut(|data| data.remove::<Click>(click_id()));
        return;
    }
    ctx.request_repaint();
    // Eases out: quick at first, then settling softly.
    let ease = 1.0 - (1.0 - t).powi(3);
    let fade = (1.0 - t).powi(2);
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        Id::new("motion-click-effect-layer"),
    ));
    let corner =
        |grow: f32| CornerRadius::same((click.radius + grow).round().clamp(0.0, 127.0) as u8);
    match style {
        ClickEffect::Off => {}
        ClickEffect::Ripple => {
            let painter = painter.with_clip_rect(click.rect);
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
            painter.circle_filled(click.at, size, accent.gamma_multiply(0.22 * fade));
            painter.circle_stroke(
                click.at,
                size,
                Stroke::new(2.0, accent.gamma_multiply(0.3 * fade)),
            );
        }
        ClickEffect::Glow => {
            painter.rect_filled(click.rect, corner(0.0), accent.gamma_multiply(0.2 * fade));
            for ring in 1..=3 {
                let grow = ring as f32 * 2.0 * ease;
                painter.rect_stroke(
                    click.rect.expand(grow),
                    corner(grow),
                    Stroke::new(2.0, accent.gamma_multiply(0.14 / ring as f32 * fade)),
                    egui::StrokeKind::Outside,
                );
            }
        }
        ClickEffect::Pop => {
            let grow = 10.0 * ease;
            painter.rect_stroke(
                click.rect.expand(grow),
                corner(grow),
                Stroke::new(2.0, accent.gamma_multiply(0.6 * fade)),
                egui::StrokeKind::Outside,
            );
        }
        ClickEffect::Sparkle => {
            let spark = crate::theme::motion::blend(accent, Color32::WHITE, 0.4);
            for index in 0..10 {
                let seed = index as f32 + click.start.fract() as f32 * 10.0;
                let angle = index as f32 / 10.0 * std::f32::consts::TAU + hash(seed) * 0.6;
                let distance = (10.0 + 22.0 * hash(seed + 1.0)) * ease + 4.0;
                let place = click.at + egui::vec2(angle.cos(), angle.sin()) * distance;
                painter.circle_filled(place, 0.6 + 2.2 * (1.0 - t), spark.gamma_multiply(fade));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_click_shows_its_effect_then_clears() {
        let ctx = Context::default();
        configure(&ctx, ClickEffect::Pop, BoxShape::Soft);
        assert_eq!(shape(&ctx), BoxShape::Soft);
        let click = Click {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 40.0)),
            at: egui::pos2(20.0, 20.0),
            start: 0.0,
            radius: 12.0,
        };
        ctx.data_mut(|data| data.insert_temp(click_id(), click));
        for style in ClickEffect::ALL {
            configure(&ctx, style, BoxShape::Rounded);
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                ctx.data_mut(|data| data.insert_temp(click_id(), click));
                paint(ui.ctx(), Color32::RED);
            });
            // This headless check does not upload textures.
            output.textures_delta.clear();
            let drew = !output.shapes.is_empty();
            assert_eq!(drew, style != ClickEffect::Off, "{style:?}");
        }
        // Long after, it is forgotten.
        let late = egui::RawInput {
            time: Some(5.0),
            ..Default::default()
        };
        ctx.run_ui(late, |ui| paint(ui.ctx(), Color32::RED))
            .textures_delta
            .clear();
        assert!(
            ctx.data(|data| data.get_temp::<Click>(click_id()))
                .is_none()
        );
    }
}
