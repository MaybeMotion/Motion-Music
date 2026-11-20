//! Cached native gradients, still or slowly drifting. No files or network.
use egui::{Color32, Rect};

const SIZE: usize = 64;

/// A three-colour diagonal sweep. The centre retains the middle hue rather
/// than averaging complementary endpoints into a grey patch.
fn colour_at(stops: [Color32; 3], x: f32, y: f32) -> Color32 {
    let t = (x * 0.65 + y * 0.35).clamp(0.0, 1.0) * 2.0;
    let segment = if t < 1.0 { 0 } else { 1 };
    let amount = t - segment as f32;
    let a = stops[segment].to_array();
    let b = stops[segment + 1].to_array();
    let channels: [u8; 4] =
        std::array::from_fn(|i| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * amount).round() as u8);
    Color32::from_rgba_premultiplied(channels[0], channels[1], channels[2], channels[3])
}

/// Draws a rounded gradient with linear GPU filtering. Each colour trio is
/// generated and uploaded once per egui context, shared by cards and pages.
pub fn paint_gradient(ui: &egui::Ui, rect: Rect, stops: [Color32; 3], radius: u8) {
    if !rect.is_finite() || !rect.is_positive() {
        return;
    }
    paint_gradient_window(
        ui,
        rect,
        stops,
        radius,
        Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
    );
}

/// Paints the part `window` (in 0 to 1 texture coordinates) of the gradient
/// over `rect`.
fn paint_gradient_window(ui: &egui::Ui, rect: Rect, stops: [Color32; 3], radius: u8, window: Rect) {
    let id = egui::Id::new(("motion-gradient", stops));
    let texture = ui
        .data(|data| data.get_temp::<egui::TextureHandle>(id))
        .unwrap_or_else(|| {
            let pixels = (0..SIZE)
                .flat_map(|y| {
                    (0..SIZE).map(move |x| {
                        colour_at(
                            stops,
                            x as f32 / (SIZE - 1) as f32,
                            y as f32 / (SIZE - 1) as f32,
                        )
                    })
                })
                .collect();
            let texture = ui.ctx().load_texture(
                "Motion gradient",
                egui::ColorImage::new([SIZE, SIZE], pixels),
                egui::TextureOptions::LINEAR,
            );
            ui.data_mut(|data| data.insert_temp(id, texture.clone()));
            texture
        });
    egui::Image::new(&texture)
        .uv(window)
        .corner_radius(egui::CornerRadius::same(radius))
        .paint_at(ui, rect);
}

/// Where the living gradient has drifted to, when it is on.
fn living_id() -> egui::Id {
    egui::Id::new("motion-living-gradient-phase")
}

/// Turn the living gradient on at phase (any number; one unit is one
/// full drift), or off with None.
pub fn set_living_phase(ctx: &egui::Context, phase: Option<f32>) {
    ctx.data_mut(|data| {
        if let Some(phase) = phase {
            data.insert_temp(living_id(), phase.rem_euclid(1.0));
        } else {
            data.remove::<f32>(living_id());
        }
    });
}

/// How far the living gradient's window sways from the centre of the still
/// gradient, as a share of it. Small, so it keeps the theme's own colours
/// and only drifts gently between them.
const SWAY: f32 = 0.12;

/// The part of the still gradient shown at `phase` (one unit is one full
/// sway): a window a little smaller than the whole, circling slowly.
fn living_window(phase: f32) -> Rect {
    let angle = phase * std::f32::consts::TAU;
    let start = egui::pos2(SWAY * (1.0 + angle.cos()), SWAY * (1.0 + angle.sin()));
    Rect::from_min_size(start, egui::vec2(1.0 - 2.0 * SWAY, 1.0 - 2.0 * SWAY))
}

/// The theme's own gradient: still, or swaying gently while the living
/// gradient is on. It is the same gradient either way; only the part of it
/// shown moves, so the colours never get stronger or turn into bands.
pub fn paint_theme_gradient(ui: &egui::Ui, rect: Rect, stops: [Color32; 3], radius: u8) {
    let Some(phase) = ui.data(|data| data.get_temp::<f32>(living_id())) else {
        return paint_gradient(ui, rect, stops, radius);
    };
    if !rect.is_finite() || !rect.is_positive() {
        return;
    }
    paint_gradient_window(ui, rect, stops, radius, living_window(phase));
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_preserves_all_three_hues_and_opacity() {
        let stops = [
            Color32::from_rgb(20, 0, 80),
            Color32::from_rgb(0, 60, 90),
            Color32::from_rgb(80, 0, 30),
        ];
        assert_eq!(colour_at(stops, 0.0, 0.0), stops[0]);
        assert_eq!(colour_at(stops, 0.5, 0.5), stops[1]);
        assert_eq!(colour_at(stops, 1.0, 1.0), stops[2]);
        for y in 0..16 {
            for x in 0..16 {
                assert_eq!(colour_at(stops, x as f32 / 15.0, y as f32 / 15.0).a(), 255);
            }
        }
    }

    #[test]
    fn the_living_gradient_sways_gently_inside_the_still_one() {
        // Every window stays inside the still gradient, keeps most of it in
        // view, and moves smoothly round and back to where it started.
        for step in 0..=100 {
            let window = living_window(step as f32 / 100.0);
            assert!(window.min.x >= 0.0 && window.min.y >= 0.0, "{window:?}");
            assert!(window.max.x <= 1.0 + 1e-6 && window.max.y <= 1.0 + 1e-6);
            assert!(window.width() >= 0.75 && window.height() >= 0.75);
            let next = living_window((step + 1) as f32 / 100.0);
            assert!(window.min.distance(next.min) < 0.01, "no jumps");
        }
        assert!(living_window(0.0).min.distance(living_window(1.0).min) < 1e-5);
        let ctx = egui::Context::default();
        set_living_phase(&ctx, Some(3.25));
        assert_eq!(
            ctx.data(|data| data.get_temp::<f32>(living_id())),
            Some(0.25)
        );
        set_living_phase(&ctx, None);
        assert_eq!(ctx.data(|data| data.get_temp::<f32>(living_id())), None);
    }

    #[test]
    fn repeated_gradient_draws_share_one_texture_and_need_no_animation() {
        let ctx = egui::Context::default();
        let stops = [Color32::RED, Color32::BLUE, Color32::GREEN];
        let mut first = None;
        let mut second = None;
        for pass in 0..3 {
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                paint_gradient(
                    ui,
                    Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(100.0, 100.0)),
                    stops,
                    7,
                );
                let id = ui
                    .data(|data| {
                        data.get_temp::<egui::TextureHandle>(egui::Id::new((
                            "motion-gradient",
                            stops,
                        )))
                    })
                    .unwrap()
                    .id();
                if pass == 0 {
                    first = Some(id);
                } else {
                    second = Some(id);
                }
            });
            output.textures_delta.clear();
        }
        assert_eq!(first, second);
    }
}
