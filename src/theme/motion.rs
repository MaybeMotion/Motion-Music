//! Short interaction transitions shared by native widgets.
//!
//! Only a changed interaction asks for frames. Stable controls are completely
//! idle, and the desktop's reduced-motion preference snaps to the endpoint.
use egui::{Color32, Id, Ui};

/// A symmetric easing curve keeps a reversal continuous: leaving a control
/// halfway through its hover never jumps to a differently eased colour.
fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

/// An eased hover or press amount, with an independent, stable widget key.
/// Keyboard focus may be painted at full strength by the caller so its
/// location is immediately clear while pointer transitions ease in and out.
pub fn hover(ui: &Ui, id: Id, on: bool) -> f32 {
    let seconds = ui.style().animation_time;
    let seconds = if crate::motion::reduced(ui.ctx()) || !seconds.is_finite() {
        0.0
    } else {
        seconds.clamp(0.0, 0.5)
    };
    ui.ctx().animate_bool_with_time_and_easing(
        id.with("motion-interaction"),
        on && ui.is_enabled(),
        seconds,
        smoothstep,
    )
}

/// Blend premultiplied colours, including transparent hover surfaces.
pub fn blend(from: Color32, to: Color32, amount: f32) -> Color32 {
    let amount = if amount.is_finite() {
        amount.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let from = from.to_array();
    let to = to.to_array();
    let channels: [u8; 4] = std::array::from_fn(|index| {
        (from[index] as f32 + (to[index] as f32 - from[index] as f32) * amount).round() as u8
    });
    Color32::from_rgba_premultiplied(channels[0], channels[1], channels[2], channels[3])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(ctx: &egui::Context, time: f64, on: bool) -> f32 {
        let mut value = 0.0;
        let mut output = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                predicted_dt: 1.0 / 60.0,
                ..Default::default()
            },
            |ui| value = hover(ui, Id::new("control"), on),
        );
        output.textures_delta.clear();
        value
    }

    #[test]
    fn hover_eases_monotonically_and_finishes_without_an_idle_tail() {
        let ctx = egui::Context::default();
        crate::motion::apply(&ctx, false);
        assert_eq!(frame(&ctx, 0.0, false), 0.0);
        let mut previous = 0.0;
        let mut intermediate = false;
        for tick in 1..=40 {
            let value = frame(&ctx, f64::from(tick) / 60.0, true);
            assert!((previous..=1.0).contains(&value));
            intermediate |= 0.0 < value && value < 1.0;
            previous = value;
        }
        assert!(intermediate, "hover must pass through intermediate colours");
        assert_eq!(previous, 1.0);
        assert_eq!(frame(&ctx, 2.0, true), 1.0);
        for tick in 1..=40 {
            let value = frame(&ctx, 2.0 + f64::from(tick) / 60.0, false);
            assert!((0.0..=previous).contains(&value));
            previous = value;
        }
        assert_eq!(previous, 0.0);
    }

    #[test]
    fn rapid_retargeting_stays_continuous_and_bounded() {
        let ctx = egui::Context::default();
        crate::motion::apply(&ctx, false);
        frame(&ctx, 0.0, false);
        let mut previous = 0.0;
        for tick in 1..=100 {
            let value = frame(&ctx, f64::from(tick) / 120.0, (tick / 4) % 2 == 0);
            assert!((0.0..=1.0).contains(&value));
            assert!((value - previous).abs() < 0.25, "reversal must not jump");
            previous = value;
        }
    }

    #[test]
    fn reduced_motion_and_zero_duration_snap_in_both_directions() {
        let ctx = egui::Context::default();
        crate::motion::apply(&ctx, false);
        frame(&ctx, 0.0, false);
        assert!(frame(&ctx, 0.02, true) < 1.0);
        crate::motion::apply(&ctx, true);
        assert_eq!(frame(&ctx, 0.04, true), 1.0);
        assert_eq!(frame(&ctx, 0.06, false), 0.0);
        crate::motion::apply(&ctx, false);
        ctx.all_styles_mut(|style| style.animation_time = 0.0);
        assert_eq!(frame(&ctx, 0.08, true), 1.0);
        assert_eq!(frame(&ctx, 0.10, false), 0.0);
    }

    #[test]
    fn transparent_blends_preserve_endpoints_and_valid_premultiplication() {
        let colour = Color32::from_rgb(90, 140, 220);
        assert_eq!(
            blend(Color32::TRANSPARENT, colour, 0.0),
            Color32::TRANSPARENT
        );
        assert_eq!(blend(Color32::TRANSPARENT, colour, 1.0), colour);
        let [r, g, b, a] = blend(Color32::TRANSPARENT, colour, 0.5).to_array();
        assert!((127..=128).contains(&a));
        assert!(r <= a && g <= a && b <= a);
        assert_eq!(blend(colour, Color32::WHITE, f32::NAN), colour);
    }
}
