//! Short, finite presentation transitions with matching paint and hit geometry.
use egui::{Id, Ui, Vec2};

#[derive(Clone, Copy)]
struct Arrival {
    key: Id,
    started: f64,
    from: f32,
}

impl Arrival {
    fn opacity(self, now: f64, seconds: f32) -> f32 {
        if seconds <= 0.0 || self.from >= 1.0 {
            return 1.0;
        }
        let elapsed = now - self.started;
        if !elapsed.is_finite() {
            return 1.0;
        }
        let t = (elapsed.max(0.0) / f64::from(seconds)).clamp(0.0, 1.0) as f32;
        // Ease out smoothly without a long tail of almost-settled frames.
        let eased = 1.0 - (1.0 - t).powi(3);
        self.from + (1.0 - self.from) * eased
    }
}

/// A new page or collection softly arrives while all its controls already work.
/// One slot per view retains only its current identity, never every visited page.
/// The first draw is fully visible and ordinary redraws do not restart the fade.
pub(crate) fn arrival(ui: &Ui, id: Id, key: Id) -> f32 {
    let now = ui.input(|input| input.time);
    let seconds = crate::motion::seconds(ui.ctx(), 0.20);
    let mut state = ui
        .data(|data| data.get_temp::<Arrival>(id))
        .unwrap_or(Arrival {
            key,
            started: now,
            from: 1.0,
        });
    if state.key != key {
        state = Arrival {
            key,
            started: now,
            from: 0.74,
        };
    }
    let opacity = state.opacity(now, seconds);
    if opacity < 1.0 {
        ui.ctx().request_repaint();
    }
    ui.data_mut(|data| data.insert_temp(id, state));
    opacity
}

/// The body of a newly selected page, including a different playlist or album.
#[derive(Clone, Copy, Debug, PartialEq)]
struct PagePresentation {
    opacity: f32,
    offset: Vec2,
}

impl PagePresentation {
    const SETTLED: Self = Self {
        opacity: 1.0,
        offset: Vec2::ZERO,
    };
}

#[derive(Clone, Copy)]
struct PageArrival {
    key: Id,
    started: Option<f64>,
}

impl PageArrival {
    fn presentation(self, now: f64, seconds: f32) -> PagePresentation {
        let Some(started) = self.started else {
            return PagePresentation::SETTLED;
        };
        let elapsed = now - started;
        if seconds <= 0.0 || !elapsed.is_finite() || elapsed >= f64::from(seconds) {
            return PagePresentation::SETTLED;
        }
        let t = (elapsed.max(0.0) / f64::from(seconds)).clamp(0.0, 1.0) as f32;
        let remaining = (1.0 - t).powi(3);
        PagePresentation {
            opacity: 1.0 - 0.58 * remaining,
            offset: egui::vec2(0.0, 18.0 * remaining),
        }
    }
}

/// Draw the new page immediately, easing it into place over 260 ms.
///
/// The child Ui's real coordinates move, so painted controls and pointer,
/// keyboard and accessibility targets agree throughout the animation. The
/// parent measures its untranslated size, keeping the scroll extent stable.
/// The clip remains the fixed page viewport. No screenshot of the old page
/// can cover new controls, and only one current page identity is retained.
pub(crate) fn page<R>(ui: &mut Ui, id: Id, key: Id, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    let now = ui.input(|input| input.time);
    let seconds = crate::motion::seconds(ui.ctx(), 0.26);
    let mut state = ui
        .data(|data| data.get_temp::<PageArrival>(id))
        .unwrap_or(PageArrival { key, started: None });
    if state.key != key {
        state = PageArrival {
            key,
            started: Some(now),
        };
    }
    // Cancel an in-progress slide as soon as reduced motion is enabled.
    // Turning it back off must not replay a partially completed transition.
    if seconds <= 0.0 {
        state.started = None;
    }
    let presentation = state.presentation(now, seconds);
    if presentation == PagePresentation::SETTLED {
        state.started = None;
    } else {
        ui.ctx().request_repaint();
    }
    ui.data_mut(|data| data.insert_temp(id, state));

    let clip = ui.clip_rect();
    let rect = ui
        .available_rect_before_wrap()
        .translate(presentation.offset);
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .id_salt("page-motion-body")
            .max_rect(rect),
    );
    child.set_clip_rect(clip);
    child.multiply_opacity(presentation.opacity);
    let result = add_contents(&mut child);
    let measured = child.min_rect().translate(-presentation.offset);
    drop(child);
    ui.advance_cursor_after_rect(measured);
    result
}

/// Toasts ease in and out at their existing lifetime. Reduced motion keeps
/// them fully visible until removal, without animating their opacity.
pub(crate) fn toast_opacity(age: f32, reduced: bool) -> f32 {
    if age >= 3.2 {
        return 0.0;
    }
    if reduced {
        return 1.0;
    }
    let smooth = |t: f32| {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    if age < 0.18 {
        smooth(age / 0.18)
    } else if age > 2.8 {
        1.0 - smooth((age - 2.8) / 0.4)
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_arrivals_finish_without_restarting_on_redraw_or_return_visits() {
        let ctx = egui::Context::default();
        let frame = |time: f64, key: &str| {
            let mut opacity = 0.0;
            let mut output = ctx.run_ui(
                egui::RawInput {
                    time: Some(time),
                    ..Default::default()
                },
                |ui| opacity = arrival(ui, Id::new("page-arrival-test"), Id::new(key)),
            );
            output.textures_delta.clear();
            opacity
        };
        assert_eq!(frame(0.0, "home"), 1.0);
        assert_eq!(frame(1.0, "settings"), 0.74);
        let midpoint = frame(1.1, "settings");
        assert!(midpoint > 0.74 && midpoint < 1.0);
        assert_eq!(frame(1.3, "settings"), 1.0);
        assert_eq!(frame(2.0, "settings"), 1.0, "redraw never restarts motion");
        assert_eq!(frame(2.1, "home"), 0.74, "returning to a page also arrives");
        assert_eq!(frame(2.5, "home"), 1.0);
    }

    #[test]
    fn reduced_motion_snaps_a_running_arrival_and_future_page_changes() {
        let ctx = egui::Context::default();
        let frame = |time: f64, key: &str| {
            let mut opacity = 0.0;
            let mut output = ctx.run_ui(
                egui::RawInput {
                    time: Some(time),
                    ..Default::default()
                },
                |ui| opacity = arrival(ui, Id::new("page-reduced-test"), Id::new(key)),
            );
            output.textures_delta.clear();
            opacity
        };
        assert_eq!(frame(0.0, "home"), 1.0);
        assert_eq!(frame(1.0, "settings"), 0.74);
        crate::motion::apply(&ctx, true);
        assert_eq!(frame(1.05, "settings"), 1.0);
        assert_eq!(frame(1.1, "home"), 1.0);
    }

    #[test]
    fn toast_easing_keeps_a_reading_plateau_and_respects_reduced_motion() {
        assert_eq!(toast_opacity(0.0, false), 0.0);
        assert!((toast_opacity(0.09, false) - 0.5).abs() < 0.001);
        for age in [0.18, 1.0, 2.8] {
            assert_eq!(toast_opacity(age, false), 1.0);
        }
        assert!((toast_opacity(3.0, false) - 0.5).abs() < 0.001);
        assert_eq!(toast_opacity(3.2, false), 0.0);
        for age in [0.0, 0.09, 2.9, 3.19] {
            assert_eq!(toast_opacity(age, true), 1.0);
        }
    }
    fn page_frame(
        ctx: &egui::Context,
        time: f64,
        key: &str,
        events: Vec<egui::Event>,
    ) -> (egui::Response, egui::Rect, f32, u8) {
        let mut result = None;
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640.0, 480.0),
                )),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ui| {
                let (header, _) =
                    ui.allocate_exact_size(egui::vec2(200.0, 24.0), egui::Sense::hover());
                let before = ui.cursor().min.y;
                let clip = ui.clip_rect();
                let response = page(ui, Id::new("page-slide-test"), Id::new(key), |ui| {
                    assert_eq!(ui.clip_rect(), clip, "the viewport stays anchored");
                    let (rect, response) =
                        ui.allocate_exact_size(egui::vec2(200.0, 44.0), egui::Sense::click());
                    ui.painter().rect_filled(rect, 0, egui::Color32::WHITE);
                    response
                });
                result = Some((response, header, ui.cursor().min.y - before));
            },
        );
        let (response, header, extent) = result.unwrap();
        let alpha = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Rect(rect) if rect.rect == response.rect => Some(rect.fill.a()),
                _ => None,
            })
            .expect("the page's control is painted");
        output.textures_delta.clear();
        (response, header, extent, alpha)
    }

    #[test]
    fn playlist_id_changes_slide_and_fade_without_moving_header_or_scroll_extent() {
        let ctx = egui::Context::default();
        let (home, header, extent, alpha) = page_frame(&ctx, 0.0, "home", vec![]);
        assert_eq!(alpha, 255, "the first page starts fully visible");
        let (first, first_header, first_extent, first_alpha) =
            page_frame(&ctx, 1.0, "playlist:first", vec![]);
        assert!((first.rect.top() - home.rect.top() - 18.0).abs() < 0.01);
        assert!(
            first_alpha > 0 && first_alpha < 160,
            "a visible arrival fade"
        );
        assert_eq!(first_header, header);
        assert_eq!(first_extent, extent, "scroll content height stays constant");
        let (middle, _, middle_extent, middle_alpha) =
            page_frame(&ctx, 1.13, "playlist:first", vec![]);
        assert!(middle.rect.top() > home.rect.top() && middle.rect.top() < first.rect.top());
        assert!(middle_alpha > first_alpha && middle_alpha < 255);
        assert_eq!(middle_extent, extent);
        let (settled, _, settled_extent, settled_alpha) =
            page_frame(&ctx, 1.3, "playlist:first", vec![]);
        assert_eq!(settled.rect, home.rect);
        assert_eq!(settled_extent, extent);
        assert_eq!(settled_alpha, 255);
        let (second, _, _, second_alpha) = page_frame(&ctx, 1.4, "playlist:second", vec![]);
        assert_eq!(
            second.rect, first.rect,
            "another playlist gets a new arrival"
        );
        assert_eq!(second_alpha, first_alpha);
    }

    #[test]
    fn a_sliding_pages_visible_control_is_clickable_at_its_painted_position() {
        let ctx = egui::Context::default();
        let (original, _, _, _) = page_frame(&ctx, 0.0, "home", vec![]);
        let (moved, _, _, _) =
            page_frame(&ctx, 1.0, "playlist:click", vec![egui::Event::PointerGone]);
        let point = egui::pos2(moved.rect.center().x, moved.rect.bottom() - 3.0);
        assert!(
            !original.rect.contains(point),
            "the point is outside the unshifted target"
        );
        assert!(moved.rect.contains(point));
        let events = vec![
            egui::Event::PointerMoved(point),
            egui::Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
        ];
        let (clicked, _, _, _) = page_frame(&ctx, 1.001, "playlist:click", events);
        assert!(
            clicked.clicked(),
            "the visible target acts immediately during the slide"
        );
    }

    #[test]
    fn rapid_page_changes_keep_only_the_current_arrival_and_redraws_do_not_restart_it() {
        let ctx = egui::Context::default();
        page_frame(&ctx, 0.0, "home", vec![]);
        page_frame(&ctx, 1.0, "playlist:a", vec![]);
        page_frame(&ctx, 1.03, "playlist:b", vec![]);
        page_frame(&ctx, 1.06, "album:c", vec![]);
        let state = ctx
            .data(|data| data.get_temp::<PageArrival>(Id::new("page-slide-test")))
            .unwrap();
        assert_eq!(state.key, Id::new("album:c"));
        let (settled, _, _, alpha) = page_frame(&ctx, 1.4, "album:c", vec![]);
        assert_eq!(alpha, 255);
        let (redraw, _, _, redraw_alpha) = page_frame(&ctx, 2.0, "album:c", vec![]);
        assert_eq!(redraw.rect, settled.rect);
        assert_eq!(redraw_alpha, 255);
        assert!(
            ctx.data(|data| data.get_temp::<PageArrival>(Id::new("page-slide-test")))
                .unwrap()
                .started
                .is_none()
        );
    }

    #[test]
    fn reduced_motion_cancels_the_slide_without_replaying_it_when_reenabled() {
        let ctx = egui::Context::default();
        let (home, _, _, _) = page_frame(&ctx, 0.0, "home", vec![]);
        page_frame(&ctx, 1.0, "playlist:a", vec![]);
        crate::motion::apply(&ctx, true);
        let (snapped, _, _, alpha) = page_frame(&ctx, 1.05, "playlist:a", vec![]);
        assert_eq!(snapped.rect, home.rect);
        assert_eq!(alpha, 255);
        crate::motion::apply(&ctx, false);
        let (later, _, _, later_alpha) = page_frame(&ctx, 1.1, "playlist:a", vec![]);
        assert_eq!(later.rect, home.rect);
        assert_eq!(later_alpha, 255);
        crate::motion::apply(&ctx, true);
        let (next, _, _, next_alpha) = page_frame(&ctx, 1.2, "playlist:b", vec![]);
        assert_eq!(next.rect, home.rect);
        assert_eq!(next_alpha, 255);
    }
}
