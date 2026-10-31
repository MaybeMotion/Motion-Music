//! Short, finite presentation transitions with matching paint and hit geometry.
use egui::{Id, Ui, Vec2};
use rand::{Rng, seq::SliceRandom};

#[cfg(any(test, feature = "demo"))]
use rand::{SeedableRng, rngs::StdRng};

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

/// Six short arrivals share their final layout and keep their controls live.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PageTransition {
    Rise,
    GlideLeft,
    GlideRight,
    SoftDrop,
    DiagonalLift,
    Dissolve,
}

impl PageTransition {
    const ALL: [Self; 6] = [
        Self::Rise,
        Self::GlideLeft,
        Self::GlideRight,
        Self::SoftDrop,
        Self::DiagonalLift,
        Self::Dissolve,
    ];

    fn presentation(self, remaining: f32) -> PagePresentation {
        let (offset, fade) = match self {
            // Keep the original v7 slide and fade exactly as one choice.
            Self::Rise => (egui::vec2(0.0, 18.0), 0.58),
            Self::GlideLeft => (egui::vec2(-26.0, 0.0), 0.48),
            Self::GlideRight => (egui::vec2(26.0, 0.0), 0.48),
            Self::SoftDrop => (egui::vec2(0.0, -18.0), 0.52),
            Self::DiagonalLift => (egui::vec2(18.0, 14.0), 0.50),
            Self::Dissolve => (Vec2::ZERO, 0.72),
        };
        PagePresentation {
            opacity: 1.0 - fade * remaining,
            offset: offset * remaining,
        }
    }
}

/// Each shuffled group uses every style once. Its first choice cannot repeat
/// the previous group's last choice. Only a real page change advances it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PageTransitions {
    order: [PageTransition; 6],
    next: usize,
    last: Option<PageTransition>,
}

impl Default for PageTransitions {
    fn default() -> Self {
        Self {
            order: PageTransition::ALL,
            next: PageTransition::ALL.len(),
            last: None,
        }
    }
}

impl PageTransitions {
    fn pick(&mut self, rng: &mut impl Rng) -> PageTransition {
        if self.next == self.order.len() {
            self.order.shuffle(rng);
            if Some(self.order[0]) == self.last {
                let other = rng.random_range(1..self.order.len());
                self.order.swap(0, other);
            }
            self.next = 0;
        }
        let selected = self.order[self.next];
        self.next += 1;
        self.last = Some(selected);
        selected
    }
}

#[cfg(any(test, feature = "demo"))]
#[derive(Clone)]
struct SeededPageTransitions(StdRng);

#[cfg(any(test, feature = "demo"))]
fn seed_id() -> Id {
    Id::new("motion-page-transition-seed")
}

/// Make an isolated native demo reproducible. This is absent from normal
/// builds and never writes a preference or changes production randomness.
#[cfg(any(test, feature = "demo"))]
pub(crate) fn seed_page_transitions(ctx: &egui::Context, seed: u64) {
    ctx.data_mut(|data| {
        data.insert_temp(
            seed_id(),
            SeededPageTransitions(StdRng::seed_from_u64(seed)),
        );
    });
}

fn next_page_transition(_ctx: &egui::Context, choices: &mut PageTransitions) -> PageTransition {
    #[cfg(any(test, feature = "demo"))]
    if let Some(mut seeded) = _ctx.data(|data| data.get_temp::<SeededPageTransitions>(seed_id())) {
        let selected = choices.pick(&mut seeded.0);
        _ctx.data_mut(|data| data.insert_temp(seed_id(), seeded));
        return selected;
    }
    choices.pick(&mut rand::rng())
}

#[derive(Clone, Copy)]
struct PageArrival {
    key: Id,
    started: Option<f64>,
    transition: PageTransition,
    choices: PageTransitions,
}

impl PageArrival {
    fn new(key: Id) -> Self {
        Self {
            key,
            started: None,
            transition: PageTransition::Rise,
            choices: PageTransitions::default(),
        }
    }

    fn presentation(self, now: f64, seconds: f32) -> PagePresentation {
        let Some(started) = self.started else {
            return PagePresentation::SETTLED;
        };
        let elapsed = now - started;
        if seconds <= 0.0 || !elapsed.is_finite() || elapsed >= f64::from(seconds) {
            return PagePresentation::SETTLED;
        }
        let t = (elapsed.max(0.0) / f64::from(seconds)).clamp(0.0, 1.0) as f32;
        self.transition.presentation((1.0 - t).powi(3))
    }
}

/// Draw the new page immediately with a randomly shuffled 260 ms arrival.
/// The original rise-and-fade stays in the mix, without consecutive repeats.
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
        .unwrap_or_else(|| PageArrival::new(key));
    if state.key != key {
        state.key = key;
        if seconds > 0.0 {
            state.transition = next_page_transition(ui.ctx(), &mut state.choices);
            state.started = Some(now);
        } else {
            state.started = None;
        }
    }
    // Cancel any in-progress arrival as soon as reduced motion is enabled.
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
                    // Keep the tested target inside the fixed viewport in every
                    // direction, so its newly exposed clickable edge is visible.
                    ui.add_space(32.0);
                    ui.horizontal(|ui| {
                        ui.add_space(40.0);
                        let (rect, response) =
                            ui.allocate_exact_size(egui::vec2(200.0, 44.0), egui::Sense::click());
                        ui.painter().rect_filled(rect, 0, egui::Color32::WHITE);
                        response
                    })
                    .inner
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

    fn page_state(ctx: &egui::Context) -> PageArrival {
        ctx.data(|data| data.get_temp::<PageArrival>(Id::new("page-slide-test")))
            .unwrap()
    }

    fn choose_next(ctx: &egui::Context, transition: PageTransition) {
        // Force only this test's first choice. Production still uses the same
        // random bag; the separate shuffle tests exercise its actual ordering.
        let mut state = page_state(ctx);
        let position = state
            .choices
            .order
            .iter()
            .position(|&item| item == transition)
            .unwrap();
        state.choices.order.swap(0, position);
        state.choices.next = 0;
        state.choices.last = None;
        ctx.data_mut(|data| data.insert_temp(Id::new("page-slide-test"), state));
    }

    fn click_events(point: egui::Pos2) -> Vec<egui::Event> {
        vec![
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
        ]
    }

    #[test]
    fn original_rise_stays_in_the_mix_and_playlist_changes_start_new_arrivals() {
        let ctx = egui::Context::default();
        seed_page_transitions(&ctx, 7);
        let (home, header, extent, alpha) = page_frame(&ctx, 0.0, "home", vec![]);
        assert_eq!(alpha, 255, "the first page starts fully visible");
        choose_next(&ctx, PageTransition::Rise);
        let (first, first_header, first_extent, first_alpha) =
            page_frame(&ctx, 1.0, "playlist:first", vec![]);
        assert!((first.rect.top() - home.rect.top() - 18.0).abs() < 0.01);
        assert!(
            first_alpha > 0 && first_alpha < 160,
            "the original visible fade"
        );
        assert_eq!(first_header, header);
        assert_eq!(first_extent, extent, "scroll content height stays constant");
        let (middle, _, middle_extent, middle_alpha) =
            page_frame(&ctx, 1.13, "playlist:first", vec![]);
        assert!((middle.rect.top() - home.rect.top() - 2.25).abs() < 0.01);
        assert!(middle_alpha > first_alpha && middle_alpha < 255);
        assert_eq!(middle_extent, extent);
        let (settled, _, settled_extent, settled_alpha) =
            page_frame(&ctx, 1.3, "playlist:first", vec![]);
        assert_eq!(settled.rect, home.rect);
        assert_eq!(settled_extent, extent);
        assert_eq!(settled_alpha, 255);
        let (_, _, _, second_alpha) = page_frame(&ctx, 1.4, "playlist:second", vec![]);
        let state = page_state(&ctx);
        assert_eq!(state.key, Id::new("playlist:second"));
        assert_eq!(state.started, Some(1.4));
        assert_ne!(
            state.transition,
            PageTransition::Rise,
            "a different arrival is next"
        );
        assert!(second_alpha > 0 && second_alpha < 255);
    }

    #[test]
    fn shuffled_groups_cover_all_styles_without_repeats_and_are_reproducible_when_seeded() {
        let sequence = |seed| {
            let mut choices = PageTransitions::default();
            let mut rng = StdRng::seed_from_u64(seed);
            (0..600).map(|_| choices.pick(&mut rng)).collect::<Vec<_>>()
        };
        let first = sequence(7);
        assert_eq!(first, sequence(7));
        assert_ne!(first, sequence(8), "different seeds shuffle differently");
        for group in first.as_chunks::<{ PageTransition::ALL.len() }>().0 {
            for style in PageTransition::ALL {
                assert_eq!(group.iter().filter(|&&value| value == style).count(), 1);
            }
        }
        assert!(first.windows(2).all(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn all_arrivals_keep_controls_clickable_at_their_painted_positions() {
        for transition in PageTransition::ALL {
            let ctx = egui::Context::default();
            seed_page_transitions(&ctx, 7);
            let (original, header, extent, _) = page_frame(&ctx, 0.0, "home", vec![]);
            choose_next(&ctx, transition);
            let (moved, moved_header, moved_extent, alpha) =
                page_frame(&ctx, 1.0, "playlist:click", vec![egui::Event::PointerGone]);
            let expected = transition.presentation(1.0).offset;
            assert!((moved.rect.min - original.rect.min - expected).length() < 0.01);
            assert_eq!(moved_header, header);
            assert_eq!(moved_extent, extent);
            assert!(alpha > 0 && alpha < 255);
            let point = if expected.x > 0.0 {
                egui::pos2(moved.rect.right() - 3.0, moved.rect.center().y)
            } else if expected.x < 0.0 {
                egui::pos2(moved.rect.left() + 3.0, moved.rect.center().y)
            } else if expected.y > 0.0 {
                egui::pos2(moved.rect.center().x, moved.rect.bottom() - 3.0)
            } else if expected.y < 0.0 {
                egui::pos2(moved.rect.center().x, moved.rect.top() + 3.0)
            } else {
                moved.rect.center()
            };
            if expected != Vec2::ZERO {
                assert!(
                    !original.rect.contains(point),
                    "a newly exposed edge for {transition:?}"
                );
            }
            assert!(moved.rect.contains(point));
            let (clicked, _, _, _) = page_frame(&ctx, 1.001, "playlist:click", click_events(point));
            assert!(
                clicked.clicked(),
                "the painted {transition:?} target acts immediately"
            );
            let (middle, _, middle_extent, middle_alpha) =
                page_frame(&ctx, 1.13, "playlist:click", vec![]);
            assert!((middle.rect.min - original.rect.min - expected * 0.125).length() < 0.01);
            assert!(middle_alpha > alpha && middle_alpha < 255);
            assert_eq!(middle_extent, extent);
            let (settled, _, settled_extent, settled_alpha) =
                page_frame(&ctx, 1.3, "playlist:click", vec![]);
            assert_eq!(settled.rect, original.rect);
            assert_eq!(settled_extent, extent);
            assert_eq!(settled_alpha, 255);
            assert!(page_state(&ctx).started.is_none());
        }
    }

    #[test]
    fn first_draw_and_normal_redraws_do_not_consume_transition_choices() {
        let ctx = egui::Context::default();
        seed_page_transitions(&ctx, 7);
        page_frame(&ctx, 0.0, "home", vec![]);
        assert_eq!(page_state(&ctx).choices, PageTransitions::default());
        let mut actual = ctx
            .data(|data| data.get_temp::<SeededPageTransitions>(seed_id()))
            .unwrap()
            .0;
        let mut untouched = StdRng::seed_from_u64(7);
        assert_eq!(actual.random::<u64>(), untouched.random::<u64>());
        page_frame(&ctx, 1.0, "playlist:a", vec![]);
        let first = page_state(&ctx);
        for time in [1.03, 1.13, 1.4, 2.0] {
            page_frame(&ctx, time, "playlist:a", vec![]);
            let redraw = page_state(&ctx);
            assert_eq!(redraw.transition, first.transition);
            assert_eq!(redraw.choices, first.choices);
        }
        page_frame(&ctx, 2.1, "home", vec![]);
        let returned = page_state(&ctx);
        assert_ne!(returned.transition, first.transition);
        assert_eq!(returned.choices.next, first.choices.next + 1);
        assert_eq!(returned.started, Some(2.1));
    }

    #[test]
    fn rapid_page_changes_keep_only_the_current_arrival_and_redraws_do_not_restart_it() {
        let ctx = egui::Context::default();
        seed_page_transitions(&ctx, 7);
        page_frame(&ctx, 0.0, "home", vec![]);
        page_frame(&ctx, 1.0, "playlist:a", vec![]);
        page_frame(&ctx, 1.03, "playlist:b", vec![]);
        page_frame(&ctx, 1.06, "album:c", vec![]);
        let state = page_state(&ctx);
        assert_eq!(state.key, Id::new("album:c"));
        assert_eq!(state.choices.next, 3);
        let (settled, _, _, alpha) = page_frame(&ctx, 1.4, "album:c", vec![]);
        assert_eq!(alpha, 255);
        let (redraw, _, _, redraw_alpha) = page_frame(&ctx, 2.0, "album:c", vec![]);
        assert_eq!(redraw.rect, settled.rect);
        assert_eq!(redraw_alpha, 255);
        assert!(page_state(&ctx).started.is_none());
        assert_eq!(page_state(&ctx).choices, state.choices);
    }

    #[test]
    fn reduced_motion_cancels_every_style_and_never_consumes_or_replays_an_arrival() {
        for transition in PageTransition::ALL {
            let ctx = egui::Context::default();
            seed_page_transitions(&ctx, 7);
            let (home, _, _, _) = page_frame(&ctx, 0.0, "home", vec![]);
            choose_next(&ctx, transition);
            page_frame(&ctx, 1.0, "playlist:a", vec![]);
            let choices = page_state(&ctx).choices;
            crate::motion::apply(&ctx, true);
            let (snapped, _, _, alpha) = page_frame(&ctx, 1.05, "playlist:a", vec![]);
            assert_eq!(snapped.rect, home.rect);
            assert_eq!(alpha, 255);
            let (next, _, _, next_alpha) = page_frame(&ctx, 1.1, "playlist:b", vec![]);
            assert_eq!(next.rect, home.rect);
            assert_eq!(next_alpha, 255);
            assert_eq!(page_state(&ctx).choices, choices);
            crate::motion::apply(&ctx, false);
            let (later, _, _, later_alpha) = page_frame(&ctx, 1.15, "playlist:b", vec![]);
            assert_eq!(later.rect, home.rect);
            assert_eq!(later_alpha, 255);
            assert_eq!(page_state(&ctx).choices, choices);
            page_frame(&ctx, 1.2, "playlist:c", vec![]);
            assert_ne!(page_state(&ctx).transition, transition);
            assert_eq!(page_state(&ctx).started, Some(1.2));
        }
    }

    #[test]
    fn every_style_finishes_exactly_and_nonfinite_timing_cannot_leave_it_running() {
        for transition in PageTransition::ALL {
            let mut state = PageArrival::new(Id::new("timing"));
            state.transition = transition;
            state.started = Some(1.0);
            assert_eq!(state.presentation(1.26, 0.26), PagePresentation::SETTLED);
            assert_eq!(state.presentation(1.0, 0.0), PagePresentation::SETTLED);
            assert_eq!(
                state.presentation(f64::NAN, 0.26),
                PagePresentation::SETTLED
            );
            assert_eq!(
                state.presentation(f64::INFINITY, 0.26),
                PagePresentation::SETTLED
            );
        }
    }
}
