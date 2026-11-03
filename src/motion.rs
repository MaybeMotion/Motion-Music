//! Whether the desktop asks for less motion: Windows' "Animation effects",
//! macOS's "Reduce motion" and GNOME's "Animations". When it does, the
//! interface shows where each animation ends at once: egui's own easing and
//! scrolling to a place, the reveal of a new theme, the fades that follow
//! the song, the lyrics, and rows making room for a drag.

use std::time::{Duration, Instant};

/// How long an answer is trusted before the system is asked again.
const RECHECK: Duration = Duration::from_secs(1);

/// egui's easing time for hover, expansion and similar state changes.
pub const ANIMATION_TIME: f32 = 0.12;

/// Explicit scroll-to requests keep native easing with a shorter travel time.
const SCROLL_TIME_SCALE: f32 = 0.72;

/// An early speed ramp with a soft landing and no overshoot. Both endpoint
/// velocities are zero, but most of the movement happens in the first half.
/// Invalid progress settles safely instead of keeping a repaint loop alive.
pub fn ramp(progress: f32) -> f32 {
    if progress.is_nan() {
        return 1.0;
    }
    let t = progress.clamp(0.0, 1.0);
    let accelerated = t * (2.0 - t);
    accelerated * accelerated * (3.0 - 2.0 * accelerated)
}

#[derive(Clone, Copy)]
struct ValueRamp {
    from: f32,
    target: f32,
    started: f64,
    seconds: f32,
}

impl ValueRamp {
    fn value(self, now: f64) -> f32 {
        if self.seconds <= 0.0 || !now.is_finite() || !self.started.is_finite() {
            return self.target;
        }
        let progress = ((now - self.started).max(0.0) / f64::from(self.seconds)) as f32;
        let t = f64::from(ramp(progress));
        // Interpolate in f64 so even opposite finite f32 endpoints cannot
        // overflow their difference before reaching a finite display value.
        (f64::from(self.from) * (1.0 - t) + f64::from(self.target) * t) as f32
    }

    fn active(self, now: f64) -> bool {
        self.from != self.target
            && self.seconds > 0.0
            && now.is_finite()
            && self.started.is_finite()
            && now - self.started < f64::from(self.seconds)
    }
}

/// Animate a finite interface value, continuing from its visible position when
/// it changes direction. The initial draw is already at its target. Durations
/// are explicit; callers shorten them without changing their finish guards.
pub fn animate_value(ctx: &egui::Context, id: egui::Id, target: f32, duration: f32) -> f32 {
    let target = if target.is_finite() { target } else { 0.0 };
    let now = ctx.input(|input| input.time);
    let duration = seconds(ctx, duration);
    let duration = if duration.is_finite() {
        duration.max(0.0)
    } else {
        0.0
    };
    let key = id.with("motion-ramped-value");
    let mut state = ctx
        .data(|data| data.get_temp::<ValueRamp>(key))
        .unwrap_or(ValueRamp {
            from: target,
            target,
            started: now,
            seconds: duration,
        });
    if duration == 0.0 || !now.is_finite() {
        state = ValueRamp {
            from: target,
            target,
            started: now,
            seconds: 0.0,
        };
    } else if state.target != target || state.seconds != duration {
        state = ValueRamp {
            from: state.value(now),
            target,
            started: now,
            seconds: duration,
        };
    }
    let value = state.value(now);
    if state.active(now) {
        ctx.request_repaint();
    }
    ctx.data_mut(|data| data.insert_temp(key, state));
    value
}

/// Animate a control's on/off state with the shared ramp and control timing.
pub fn animate_bool(ctx: &egui::Context, id: egui::Id, target: bool) -> f32 {
    animate_bool_with_time(ctx, id, target, ANIMATION_TIME)
}

/// Like animate_bool with an explicit duration, retaining continuous values
/// through quick reversals rather than flipping an asymmetric easing curve.
pub fn animate_bool_with_time(
    ctx: &egui::Context,
    id: egui::Id,
    target: bool,
    duration: f32,
) -> f32 {
    animate_value(ctx, id, if target { 1.0 } else { 0.0 }, duration)
}

fn default_scroll() -> egui::style::ScrollAnimation {
    egui::style::ScrollAnimation::new(1600.0, egui::emath::Rangef::new(0.07, 0.22))
}

/// The system's preference, asked at most once a second.
#[derive(Default)]
pub struct Motion {
    reduced: bool,
    checked: Option<Instant>,
    /// What the Linux desktop portal said at the last check.
    desktop: Option<bool>,
}

impl Motion {
    /// Whether less motion is asked for, as of the last check.
    pub fn reduced(&self) -> bool {
        self.reduced
    }

    /// Asks the system again when the last answer is a second old, and
    /// answers whether the preference changed. `desktop_animations` is what
    /// the Linux desktop portal last said about animations, if anything.
    /// The portal wakes the app when that changes, and nothing asks again
    /// a second later, so a new answer from it applies at once.
    pub fn refresh(&mut self, now: Instant, desktop_animations: Option<bool>) -> bool {
        if desktop_animations == self.desktop
            && self
                .checked
                .is_some_and(|checked| now.duration_since(checked) < RECHECK)
        {
            return false;
        }
        self.checked = Some(now);
        self.desktop = desktop_animations;
        self.set(system_reduced(desktop_animations))
    }

    /// Takes `reduced` as the preference, and answers whether it changed.
    pub fn set(&mut self, reduced: bool) -> bool {
        let changed = self.reduced != reduced;
        self.reduced = reduced;
        changed
    }
}

/// Sets egui's easing and scrolling for `reduced`, none when less motion is
/// asked for, and keeps it for the views' own animations to read.
pub fn apply(ctx: &egui::Context, reduced: bool) {
    let time = if reduced { 0.0 } else { ANIMATION_TIME };
    let scroll = if reduced {
        egui::style::ScrollAnimation::none()
    } else {
        default_scroll()
    };
    ctx.all_styles_mut(|style| {
        style.animation_time = time;
        style.scroll_animation = scroll;
    });
    ctx.data_mut(|data| data.insert_temp(reduced_id(), reduced));
}

fn reduced_id() -> egui::Id {
    egui::Id::new("spotifast-reduced-motion")
}

/// Whether less motion is asked for, as last applied to `ctx`.
pub fn reduced(ctx: &egui::Context) -> bool {
    ctx.data(|data| data.get_temp(reduced_id()))
        .unwrap_or(false)
}

/// How long an animation of `seconds` takes: none when less motion is asked
/// for, so it shows where it ends at once.
pub fn seconds(ctx: &egui::Context, seconds: f32) -> f32 {
    if reduced(ctx) { 0.0 } else { seconds }
}

/// A scroll to a place with a shorter native easing duration, or at once
/// when less motion is asked for. Native scrolling keeps its own curve.
pub fn scroll(ctx: &egui::Context, seconds: f32) -> egui::style::ScrollAnimation {
    if reduced(ctx) {
        egui::style::ScrollAnimation::none()
    } else {
        egui::style::ScrollAnimation::duration(seconds * SCROLL_TIME_SCALE)
    }
}

#[cfg(windows)]
fn system_reduced(_desktop_animations: Option<bool>) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SPI_GETCLIENTAREAANIMATION, SystemParametersInfoW,
    };
    let mut enabled: windows_sys::core::BOOL = 1;
    let read = unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            (&mut enabled as *mut windows_sys::core::BOOL).cast(),
            0,
        )
    };
    read != 0 && enabled == 0
}

#[cfg(target_os = "macos")]
fn system_reduced(_desktop_animations: Option<bool>) -> bool {
    objc2_app_kit::NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion()
}

#[cfg(not(any(windows, target_os = "macos")))]
fn system_reduced(desktop_animations: Option<bool>) -> bool {
    desktop_animations == Some(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ramp_accelerates_early_and_finishes_without_overshoot() {
        assert_eq!(ramp(0.0), 0.0);
        assert_eq!(ramp(1.0), 1.0);
        assert_eq!(ramp(-1.0), 0.0);
        assert_eq!(ramp(f32::NEG_INFINITY), 0.0);
        assert_eq!(ramp(f32::INFINITY), 1.0);
        assert_eq!(ramp(f32::NAN), 1.0);
        assert!((ramp(0.5) - 0.84375).abs() < 0.00001);
        let mut previous = 0.0;
        for step in 0..=1000 {
            let value = ramp(step as f32 / 1000.0);
            assert!(value.is_finite() && (0.0..=1.0).contains(&value));
            assert!(value >= previous, "movement never reverses or overshoots");
            previous = value;
        }
        let epsilon = 0.001;
        assert!(ramp(epsilon) / epsilon < 0.02, "gentle initial velocity");
        assert!(
            (1.0 - ramp(1.0 - epsilon)) / epsilon < 0.02,
            "gentle landing"
        );
        assert!(
            ramp(0.10) - ramp(0.05) > ramp(0.05),
            "the start accelerates"
        );
        assert!(
            ramp(0.90) - ramp(0.85) < ramp(0.50) - ramp(0.45),
            "the finish slows down"
        );
    }

    fn animated_frame(ctx: &egui::Context, time: f64, target: f32, duration: f32) -> f32 {
        let mut value = f32::NAN;
        let mut output = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                ..Default::default()
            },
            |_ui| value = animate_value(ctx, egui::Id::new("ramped-test-value"), target, duration),
        );
        output.textures_delta.clear();
        value
    }

    #[test]
    fn value_reversals_continue_from_the_displayed_position_and_settle() {
        let ctx = egui::Context::default();
        assert_eq!(animated_frame(&ctx, 0.0, -4.0, 0.12), -4.0);
        assert_eq!(animated_frame(&ctx, 1.0, 8.0, 0.12), -4.0);
        let rising = animated_frame(&ctx, 1.04, 8.0, 0.12);
        assert!(rising > -4.0 && rising < 8.0);
        assert_eq!(
            animated_frame(&ctx, 1.04, -4.0, 0.12),
            rising,
            "retarget keeps the visible value"
        );
        let falling = animated_frame(&ctx, 1.07, -4.0, 0.12);
        assert!(falling < rising && falling > -4.0);
        assert_eq!(animated_frame(&ctx, 1.07, 8.0, 0.12), falling);
        assert_eq!(animated_frame(&ctx, 1.20, 8.0, 0.12), 8.0);
        let state = ctx
            .data(|data| {
                data.get_temp::<ValueRamp>(
                    egui::Id::new("ramped-test-value").with("motion-ramped-value"),
                )
            })
            .unwrap();
        assert!(
            !state.active(1.20),
            "settled motion needs no animation repaint"
        );
        for time in [20.0, 20.1, 20.2] {
            assert_eq!(animated_frame(&ctx, time, 8.0, 0.12), 8.0);
        }
        assert!(
            !ctx.has_requested_repaint(),
            "settled controls do not keep the interface drawing"
        );
    }

    #[test]
    fn shorter_durations_and_reduced_motion_preserve_a_finite_target() {
        let ctx = egui::Context::default();
        animated_frame(&ctx, 0.0, 0.0, 0.12);
        animated_frame(&ctx, 1.0, 1.0, 0.12);
        let before = animated_frame(&ctx, 1.03, 1.0, 0.12);
        assert_eq!(
            animated_frame(&ctx, 1.03, 1.0, 0.08),
            before,
            "duration changes do not jump"
        );
        assert_eq!(animated_frame(&ctx, 1.12, 1.0, 0.08), 1.0);
        apply(&ctx, true);
        assert_eq!(animated_frame(&ctx, 1.13, -6.0, 0.12), -6.0);
        apply(&ctx, false);
        assert_eq!(animated_frame(&ctx, 1.14, 3.0, 0.0), 3.0);
        assert_eq!(animated_frame(&ctx, 1.15, 4.0, f32::NAN), 4.0);
        assert_eq!(animated_frame(&ctx, 1.16, f32::NAN, 0.0), 0.0);
        let huge = ValueRamp {
            from: -f32::MAX,
            target: f32::MAX,
            started: 0.0,
            seconds: 1.0,
        };
        assert!(huge.value(0.5).is_finite());
        assert_eq!(huge.value(f64::INFINITY), f32::MAX);
    }

    #[test]
    fn the_system_is_asked_at_most_once_a_second() {
        let mut motion = Motion::default();
        let start = Instant::now();
        motion.refresh(start, None);
        motion.set(true);
        assert!(!motion.refresh(start + Duration::from_millis(500), None));
        assert!(motion.reduced(), "a fresh answer is kept");
    }

    #[test]
    fn a_change_is_reported_once() {
        let mut motion = Motion::default();
        assert!(motion.set(true));
        assert!(!motion.set(true));
        assert!(motion.set(false));
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    #[test]
    fn linux_follows_the_desktops_animations_setting() {
        assert!(system_reduced(Some(false)));
        assert!(!system_reduced(Some(true)));
        assert!(!system_reduced(None));
    }

    /// The portal wakes the app once when its switch changes; that answer
    /// applies then, even within a second of the last check.
    #[cfg(not(any(windows, target_os = "macos")))]
    #[test]
    fn a_portal_change_applies_within_the_second() {
        let mut motion = Motion::default();
        let start = Instant::now();
        assert!(!motion.refresh(start, Some(true)));
        assert!(motion.refresh(start + Duration::from_millis(100), Some(false)));
        assert!(motion.reduced());
        assert!(
            !motion.refresh(start + Duration::from_millis(200), Some(false)),
            "an unchanged answer waits out the second"
        );
        assert!(motion.refresh(start + Duration::from_millis(300), Some(true)));
        assert!(!motion.reduced());
    }

    #[test]
    fn reduced_motion_turns_egui_easing_off() {
        let ctx = egui::Context::default();
        apply(&ctx, true);
        assert_eq!(ctx.global_style().animation_time, 0.0);
        assert_eq!(
            ctx.global_style().scroll_animation,
            egui::style::ScrollAnimation::none()
        );
        apply(&ctx, false);
        assert_eq!(ctx.global_style().animation_time, ANIMATION_TIME);
        assert_eq!(ctx.global_style().scroll_animation, default_scroll());
    }

    /// The views' own animations read the same preference: none with less
    /// motion, their own length otherwise.
    #[test]
    fn the_views_own_animations_follow_the_preference() {
        let ctx = egui::Context::default();
        assert_eq!(seconds(&ctx, 0.3), 0.3, "motion until told otherwise");
        apply(&ctx, true);
        assert_eq!(seconds(&ctx, 0.3), 0.0);
        assert_eq!(scroll(&ctx, 0.45), egui::style::ScrollAnimation::none());
        apply(&ctx, false);
        assert_eq!(seconds(&ctx, 0.3), 0.3);
        assert_eq!(
            scroll(&ctx, 0.45),
            egui::style::ScrollAnimation::duration(0.45 * SCROLL_TIME_SCALE)
        );
    }
}
