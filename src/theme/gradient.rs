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

/// A seamless, repeating band of the three colours: first, second, third,
/// second, first again, along both diagonals, so it can slide endlessly.
fn looping_colour_at(stops: [Color32; 3], x: f32, y: f32) -> Color32 {
    let along = (x + y).rem_euclid(1.0);
    let wave = 1.0 - (2.0 * along - 1.0).abs();
    // Equal x and y walk colour_at's sweep from its first stop to its last.
    colour_at(stops, wave, wave)
}

/// The theme's own gradient: still, or drifting while the living gradient is
/// on. Only the texture's window moves; one texture serves every frame.
pub fn paint_theme_gradient(ui: &egui::Ui, rect: Rect, stops: [Color32; 3], radius: u8) {
    let Some(phase) = ui.data(|data| data.get_temp::<f32>(living_id())) else {
        return paint_gradient(ui, rect, stops, radius);
    };
    if !rect.is_finite() || !rect.is_positive() {
        return;
    }
    let id = egui::Id::new(("motion-gradient-loop", stops));
    let texture = ui
        .data(|data| data.get_temp::<egui::TextureHandle>(id))
        .unwrap_or_else(|| {
            let pixels = (0..SIZE)
                .flat_map(|y| {
                    (0..SIZE).map(move |x| {
                        looping_colour_at(stops, x as f32 / SIZE as f32, y as f32 / SIZE as f32)
                    })
                })
                .collect();
            let texture = ui.ctx().load_texture(
                "Motion living gradient",
                egui::ColorImage::new([SIZE, SIZE], pixels),
                egui::TextureOptions {
                    wrap_mode: egui::TextureWrapMode::Repeat,
                    ..egui::TextureOptions::LINEAR
                },
            );
            ui.data_mut(|data| data.insert_temp(id, texture.clone()));
            texture
        });
    // Half a tile across the window shows one sweep through the colours;
    // the window slides diagonally as the phase grows.
    let start = egui::pos2(phase, phase * 0.4);
    egui::Image::new(&texture)
        .uv(Rect::from_min_size(start, egui::vec2(0.5, 0.5)))
        .corner_radius(egui::CornerRadius::same(radius))
        .paint_at(ui, rect);
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
    fn the_living_gradient_loops_seamlessly_through_all_three_colours() {
        let stops = [
            Color32::from_rgb(200, 30, 60),
            Color32::from_rgb(40, 120, 220),
            Color32::from_rgb(250, 200, 40),
        ];
        // The pattern repeats across the tile's edges in both directions.
        for at in [0.0, 0.13, 0.5, 0.87] {
            assert_eq!(
                looping_colour_at(stops, at, 0.0),
                looping_colour_at(stops, at, 1.0)
            );
            assert_eq!(
                looping_colour_at(stops, 0.0, at),
                looping_colour_at(stops, 1.0, at)
            );
        }
        assert_eq!(looping_colour_at(stops, 0.0, 0.0), stops[0]);
        assert_eq!(looping_colour_at(stops, 0.25, 0.25), stops[2]);
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
