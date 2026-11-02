//! Short, finite presentation transitions with matching paint and hit geometry.
use super::reveal;
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

/// Every page transition reveals a fixed-layout new page through the old one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PageTransition {
    CircleOut,
    CircleIn,
    CornerCircle,
    DiagonalSweep,
    Curtain,
    SwipeLeft,
    SwipeRight,
    SwipeUp,
    SwipeDown,
    DiamondIris,
}

impl PageTransition {
    const ALL: [Self; 10] = [
        Self::CircleOut,
        Self::CircleIn,
        Self::CornerCircle,
        Self::DiagonalSweep,
        Self::Curtain,
        Self::SwipeLeft,
        Self::SwipeRight,
        Self::SwipeUp,
        Self::SwipeDown,
        Self::DiamondIris,
    ];

    fn reveal(self) -> reveal::Kind {
        match self {
            Self::CircleOut => reveal::Kind::CircleOut,
            Self::CircleIn => reveal::Kind::CircleIn,
            Self::CornerCircle => reveal::Kind::CornerCircle,
            Self::DiagonalSweep => reveal::Kind::Diagonal,
            Self::Curtain => reveal::Kind::Curtain,
            Self::SwipeLeft => reveal::Kind::SwipeLeft,
            Self::SwipeRight => reveal::Kind::SwipeRight,
            Self::SwipeUp => reveal::Kind::SwipeUp,
            Self::SwipeDown => reveal::Kind::SwipeDown,
            Self::DiamondIris => reveal::Kind::Diamond,
        }
    }

    fn seconds(self) -> f32 {
        0.56
    }
}

/// Each shuffled group uses every style once. Its first choice cannot repeat
/// the previous group's last choice. Only a real page change advances it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PageTransitions {
    order: [PageTransition; PageTransition::ALL.len()],
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
            transition: PageTransition::CircleOut,
            choices: PageTransitions::default(),
        }
    }
}

#[derive(Clone)]
struct PageReveal {
    source: Option<reveal::Snapshot>,
    picture: Option<reveal::Picture>,
    previous: Id,
}

#[derive(Clone)]
struct PendingOverlay {
    pass: u64,
    layer: egui::LayerId,
    shapes: Vec<egui::layers::ShapeIdx>,
    font_size: [usize; 2],
    arrival: Id,
    kind: reveal::Kind,
    progress: f32,
    opaque: bool,
}

#[derive(Clone)]
struct PendingCapture {
    pass: u64,
    request: reveal::Capture,
    snapshot: Id,
    arrival: Id,
}

fn pending_capture_id() -> Id {
    Id::new("motion-page-pending-capture")
}

fn pending_overlay_id() -> Id {
    Id::new("motion-page-pending-overlay")
}

fn discard_overlay(ctx: &egui::Context, pending: &PendingOverlay) {
    ctx.graphics_mut(|graphics| {
        if let Some(layer) = graphics.get_mut(pending.layer) {
            for &index in &pending.shapes {
                layer.reset_shape(index);
            }
        }
    });
    ctx.data_mut(|data| {
        if let Some(mut state) = data.get_temp::<PageArrival>(pending.arrival) {
            state.started = None;
            data.insert_temp(pending.arrival, state);
        }
        data.remove::<PageReveal>(pending.arrival.with("active-page-reveal"));
    });
    ctx.request_repaint();
}

/// Finish after enclosing frames paint their backgrounds and later labels lay
/// out glyphs. The one snapshot is the complete currently displayed body,
/// including an in-progress reveal, so a retarget starts from that image.
pub(crate) fn finish(ctx: &egui::Context) {
    let (overlay, capture) = ctx.data_mut(|data| {
        let overlay = data.get_temp::<PendingOverlay>(pending_overlay_id());
        let capture = data.get_temp::<PendingCapture>(pending_capture_id());
        data.remove::<PendingOverlay>(pending_overlay_id());
        data.remove::<PendingCapture>(pending_capture_id());
        (overlay, capture)
    });
    let mut overlay = overlay.filter(|pending| pending.pass == ctx.cumulative_pass_nr());
    if overlay
        .as_ref()
        .is_some_and(|pending| pending.font_size != ctx.fonts(|fonts| fonts.font_image_size()))
    {
        discard_overlay(ctx, overlay.as_ref().unwrap());
        overlay = None;
    }
    let Some(capture) = capture.filter(|pending| pending.pass == ctx.cumulative_pass_nr()) else {
        return;
    };
    let cover = overlay
        .as_ref()
        .filter(|pending| pending.arrival == capture.arrival)
        .and_then(|pending| {
            Some(reveal::Cover {
                kind: pending.kind,
                progress: pending.progress,
                first_shape: pending.shapes.first()?.0,
                last_shape: pending.shapes.last()?.0,
                opaque: pending.opaque,
            })
        });
    if let Some(snapshot) = capture.request.complete(ctx, cover) {
        ctx.data_mut(|data| data.insert_temp(capture.snapshot, snapshot));
    } else {
        // An unsupported callback or oversized scene must not grow the cache
        // indefinitely. Settle this exceptional frame and recover next draw.
        ctx.data_mut(|data| data.remove::<reveal::Snapshot>(capture.snapshot));
        if let Some(pending) = overlay.as_ref() {
            discard_overlay(ctx, pending);
        }
    }
}

fn reveal_progress(state: PageArrival, now: f64) -> f32 {
    let Some(start) = state.started else {
        return 1.0;
    };
    let elapsed = now - start;
    if !elapsed.is_finite() {
        return 1.0;
    }
    let t = (elapsed / f64::from(state.transition.seconds())).clamp(0.0, 1.0) as f32;
    t * t * (3.0 - 2.0 * t)
}

fn interrupt_reveal(ui: &Ui) -> bool {
    ui.input(|input| {
        input.smooth_scroll_delta != Vec2::ZERO
            || input.events.iter().any(|event| {
                matches!(
                    event,
                    egui::Event::MouseWheel { .. }
                        | egui::Event::Key { pressed: true, .. }
                        | egui::Event::Text(_)
                        | egui::Event::Paste(_)
                        | egui::Event::Cut
                        | egui::Event::Copy
                        | egui::Event::Ime(_)
                )
            })
    })
}

/// Page bodies use ten native wallpaper-style reveals without moving or fading
/// their contents. Fixed chrome stays outside the cached viewport. Only the previous
/// displayed composite is retained; geometry and existing textures stay in memory, with no
/// screenshot, readback, file operation or new renderer dependency.
///
/// New controls already have their actual layout. A covered pointer cannot
/// activate them. Clicking an exposed control works without skipping the
/// animation, and repeated clicks never restart its clock. Keyboard and scroll
/// input can still settle a reveal immediately.
pub(crate) fn page<R>(
    ui: &mut Ui,
    id: Id,
    key: Id,
    scene: Id,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let now = ui.input(|input| input.time);
    let reduced = crate::motion::reduced(ui.ctx());
    let snapshot_id = id.with("last-page-picture");
    let reveal_id = id.with("active-page-reveal");
    let mut state = ui
        .data(|data| data.get_temp::<PageArrival>(id))
        .unwrap_or_else(|| PageArrival::new(key));
    let mut active = ui.data(|data| data.get_temp::<PageReveal>(reveal_id));
    if state.key != key {
        let previous = state.key;
        state.key = key;
        active = None;
        if !reduced {
            state.transition = next_page_transition(ui.ctx(), &mut state.choices);
            state.started = Some(now);
            let source = ui.data(|data| data.get_temp::<reveal::Snapshot>(snapshot_id));
            if source
                .as_ref()
                .is_some_and(|s| s.matches(ui, previous, scene))
            {
                active = Some(PageReveal {
                    source,
                    picture: None,
                    previous,
                });
            } else {
                state.started = None;
            }
        } else {
            state.started = None;
        }
    }
    if reduced || interrupt_reveal(ui) {
        state.started = None;
        active = None;
    }
    let seconds = crate::motion::seconds(ui.ctx(), state.transition.seconds());
    if state
        .started
        .is_some_and(|start| !now.is_finite() || now - start >= f64::from(seconds))
    {
        state.started = None;
        active = None;
    }
    if active.as_ref().is_some_and(|a| {
        a.picture.as_ref().is_some_and(|p| !p.valid(ui, scene))
            || a.source
                .as_ref()
                .is_some_and(|s| !s.matches(ui, a.previous, scene))
    }) {
        state.started = None;
        active = None;
    }
    let progress = reveal_progress(state, now);
    let kind = state.transition.reveal();
    let body_pointer = ui
        .input(|input| input.pointer.interact_pos())
        .filter(|&point| ui.clip_rect().contains(point));
    let covered_pointer = active.is_some()
        && body_pointer
            .is_some_and(|point| !reveal::revealed(ui.clip_rect(), kind, progress, point));
    if state.started.is_some() {
        ui.ctx().request_repaint();
    }
    let clip = ui.clip_rect();
    let rect = ui.available_rect_before_wrap();
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .id_salt("page-motion-body")
            .max_rect(rect),
    );
    child.set_clip_rect(clip);
    if covered_pointer {
        let opacity = child.opacity();
        child.disable();
        // egui's disabled treatment is painter opacity, not different widget
        // colours. Keep the original paint while gating covered hit targets.
        child.set_opacity(opacity);
    }
    let result = add_contents(&mut child);
    let measured = child.min_rect();
    drop(child);
    ui.advance_cursor_after_rect(measured);
    // On a new reveal this runs after text layout, so it uses the font atlas
    // size that the new body actually needs. Later growth cancels safely.
    if let Some(reveal) = active.as_mut() {
        if reveal.picture.is_none() {
            reveal.picture = reveal
                .source
                .take()
                .and_then(|s| s.picture(ui, reveal.previous, scene));
            if reveal.picture.is_none() {
                state.started = None;
            }
        }
        // The new body can grow the font atlas during this very draw. Never
        // paint a cached mesh with UVs normalized for its previous size.
        if let Some(picture) = &reveal.picture {
            if picture.valid(ui, scene) {
                let shapes = picture.paint(ui, kind, progress);
                let pending = PendingOverlay {
                    pass: ui.ctx().cumulative_pass_nr(),
                    layer: ui.layer_id(),
                    shapes,
                    font_size: ui.ctx().fonts(|fonts| fonts.font_image_size()),
                    arrival: id,
                    kind,
                    progress,
                    opaque: picture.opaque(),
                };
                ui.data_mut(|data| data.insert_temp(pending_overlay_id(), pending));
            } else {
                state.started = None;
            }
        }
    }
    if state.started.is_none() {
        active = None;
    }
    // Enclosing Frames still hold Noop background slots at this point. Keep
    // only metadata now and collect painted shapes in finish(), not here.
    if let Some(request) = reveal::prepare(ui, key, scene) {
        let pass = ui.ctx().cumulative_pass_nr();
        ui.data_mut(|data| {
            data.insert_temp(
                pending_capture_id(),
                PendingCapture {
                    pass,
                    request,
                    snapshot: snapshot_id,
                    arrival: id,
                },
            )
        });
    } else {
        ui.data_mut(|data| data.remove::<reveal::Snapshot>(snapshot_id));
    }
    ui.data_mut(|data| {
        data.insert_temp(id, state);
        if let Some(active) = active {
            data.insert_temp(reveal_id, active);
        } else {
            data.remove::<PageReveal>(reveal_id);
        }
    });
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
                let response = page(
                    ui,
                    Id::new("page-slide-test"),
                    Id::new(key),
                    Id::new("test-scene"),
                    |ui| {
                        assert_eq!(ui.clip_rect(), clip, "the viewport stays anchored");
                        // Keep the tested target inside the fixed viewport in every
                        // direction, so its newly exposed clickable edge is visible.
                        ui.add_space(32.0);
                        ui.horizontal(|ui| {
                            ui.add_space(40.0);
                            let (rect, response) = ui
                                .allocate_exact_size(egui::vec2(200.0, 44.0), egui::Sense::click());
                            ui.painter().rect_filled(rect, 0, egui::Color32::WHITE);
                            response
                        })
                        .inner
                    },
                );
                result = Some((response, header, ui.cursor().min.y - before));
                finish(ui.ctx());
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
    fn circular_playlist_changes_keep_the_body_fixed_and_start_fresh_reveals() {
        let ctx = egui::Context::default();
        seed_page_transitions(&ctx, 7);
        let (home, header, extent, alpha) = page_frame(&ctx, 0.0, "home", vec![]);
        assert_eq!(alpha, 255, "the first page starts fully visible");
        choose_next(&ctx, PageTransition::CircleOut);
        for time in [1.0, 1.13, 1.28, 1.57] {
            let (page, page_header, page_extent, alpha) =
                page_frame(&ctx, time, "playlist:first", vec![]);
            assert_eq!(page.rect, home.rect, "page content never slides");
            assert_eq!(page_header, header);
            assert_eq!(page_extent, extent, "scroll content height stays constant");
            assert_eq!(alpha, 255, "new page contents never fade");
        }
        assert!(page_state(&ctx).started.is_none());
        let (second, _, _, alpha) = page_frame(&ctx, 1.7, "playlist:second", vec![]);
        let state = page_state(&ctx);
        assert_eq!(state.key, Id::new("playlist:second"));
        assert_eq!(state.started, Some(1.7));
        assert_ne!(state.transition, PageTransition::CircleOut);
        assert_eq!(second.rect, home.rect);
        assert_eq!(alpha, 255);
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
    fn every_transition_keeps_real_control_and_scroll_geometry_fixed() {
        for transition in PageTransition::ALL {
            let ctx = egui::Context::default();
            let (original, header, extent, _) = page_frame(&ctx, 0.0, "home", vec![]);
            choose_next(&ctx, transition);
            for time in [1.0, 1.05, 1.2, 1.4, 1.57] {
                let (page, page_header, page_extent, alpha) =
                    page_frame(&ctx, time, "playlist:fixed", vec![egui::Event::PointerGone]);
                assert_eq!(
                    page.rect, original.rect,
                    "{transition:?} never translates controls"
                );
                assert_eq!(page_header, header);
                assert_eq!(page_extent, extent);
                assert_eq!(alpha, 255, "{transition:?} never fades the new page");
            }
            assert!(page_state(&ctx).started.is_none());
            let (clicked, _, _, _) = page_frame(
                &ctx,
                1.6,
                "playlist:fixed",
                click_events(original.rect.center()),
            );
            assert!(
                clicked.clicked(),
                "settled controls still act at their actual positions"
            );
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
            assert_eq!(reveal_progress(state, 1.0), 0.0);
            assert_eq!(reveal_progress(state, 1.56), 1.0);
            assert_eq!(reveal_progress(state, f64::NAN), 1.0);
            assert_eq!(reveal_progress(state, f64::INFINITY), 1.0);
            assert_eq!(reveal_progress(state, 0.0), 0.0);
            let ctx = egui::Context::default();
            page_frame(&ctx, 0.0, "home", vec![]);
            choose_next(&ctx, transition);
            page_frame(&ctx, 1.0, "playlist:finish", vec![]);
            page_frame(&ctx, 1.57, "playlist:finish", vec![]);
            assert!(page_state(&ctx).started.is_none());
        }
    }

    #[test]
    fn reveals_keep_layout_and_full_paint_and_block_covered_clicks() {
        for transition in PageTransition::ALL {
            let ctx = egui::Context::default();
            let (original, header, extent, _) = page_frame(&ctx, 0.0, "home", vec![]);
            choose_next(&ctx, transition);
            let (started, new_header, new_extent, alpha) =
                page_frame(&ctx, 1.0, "playlist:reveal", vec![]);
            assert_eq!(
                started.rect, original.rect,
                "{transition:?} never shifts layout"
            );
            assert_eq!(new_header, header);
            assert_eq!(new_extent, extent);
            assert_eq!(alpha, 255);
            let (covered, _, _, alpha) = page_frame(
                &ctx,
                1.001,
                "playlist:reveal",
                click_events(started.rect.center()),
            );
            assert!(
                !covered.clicked(),
                "{transition:?} never clicks through old content"
            );
            assert!(!covered.enabled());
            assert_eq!(
                alpha, 255,
                "blocking a covered hit must not gray the new body"
            );
            assert!(page_state(&ctx).started.is_some());
            let (finished, _, _, alpha) = page_frame(
                &ctx,
                1.57,
                "playlist:reveal",
                vec![egui::Event::PointerGone],
            );
            assert!(finished.enabled());
            assert_eq!(finished.rect, original.rect);
            assert_eq!(alpha, 255);
            assert!(page_state(&ctx).started.is_none());
        }
    }

    #[test]
    fn clicking_a_revealed_control_acts_without_skipping_the_reveal() {
        for transition in PageTransition::ALL {
            let ctx = egui::Context::default();
            page_frame(&ctx, 0.0, "home", vec![]);
            choose_next(&ctx, transition);
            page_frame(&ctx, 1.0, "playlist:reveal", vec![]);
            let mut exposed = None;
            for step in 1..10 {
                let time = 1.0 + 0.056 * f64::from(step);
                let (response, _, _, _) = page_frame(
                    &ctx,
                    time,
                    "playlist:reveal",
                    vec![egui::Event::PointerGone],
                );
                assert!(page_state(&ctx).started.is_some());
                let progress = reveal_progress(page_state(&ctx), time);
                let point = (1..10)
                    .flat_map(|y| {
                        (1..10).map(move |x| {
                            egui::pos2(
                                response.rect.left() + response.rect.width() * x as f32 / 10.0,
                                response.rect.top() + response.rect.height() * y as f32 / 10.0,
                            )
                        })
                    })
                    .find(|&point| {
                        reveal::revealed(
                            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(640.0, 480.0)),
                            transition.reveal(),
                            progress,
                            point,
                        )
                    });
                if let Some(point) = point {
                    exposed = Some((time, point));
                    break;
                }
            }
            let (time, point) = exposed.expect("the control is exposed before the reveal ends");
            let (clicked, _, _, _) =
                page_frame(&ctx, time + 0.001, "playlist:reveal", click_events(point));
            assert!(
                clicked.clicked(),
                "the revealed {transition:?} control acts immediately"
            );
            assert_eq!(page_state(&ctx).started, Some(1.0));
        }
    }

    #[test]
    fn keyboard_and_scroll_finish_reveals_without_consuming_another_style() {
        for event in [
            egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, 30.0),
                phase: egui::TouchPhase::Move,
                modifiers: egui::Modifiers::NONE,
            },
        ] {
            let ctx = egui::Context::default();
            page_frame(&ctx, 0.0, "home", vec![]);
            choose_next(&ctx, PageTransition::CircleOut);
            page_frame(&ctx, 1.0, "playlist:reveal", vec![]);
            let choices = page_state(&ctx).choices;
            page_frame(&ctx, 1.1, "playlist:reveal", vec![event]);
            assert!(page_state(&ctx).started.is_none());
            assert_eq!(page_state(&ctx).choices, choices);
        }
    }

    #[test]
    fn rapid_retargeting_reveals_from_the_current_composite_without_snapping() {
        let ctx = egui::Context::default();
        page_frame(&ctx, 0.0, "home", vec![]);
        choose_next(&ctx, PageTransition::CircleOut);
        page_frame(&ctx, 1.0, "playlist:first", vec![]);
        page_frame(&ctx, 1.1, "playlist:first", vec![]);
        choose_next(&ctx, PageTransition::CircleIn);
        page_frame(&ctx, 1.11, "playlist:second", vec![]);
        assert_eq!(page_state(&ctx).started, Some(1.11));
        assert!(
            ctx.data(|data| data
                .get_temp::<PageReveal>(Id::new("page-slide-test").with("active-page-reveal")))
                .is_some()
        );
        let state = page_state(&ctx);
        page_frame(&ctx, 1.2, "playlist:second", vec![]);
        assert_eq!(page_state(&ctx).choices, state.choices);
        assert_eq!(page_state(&ctx).started, Some(1.11));
    }

    #[test]
    fn fixed_chrome_clicks_do_not_snap_a_body_reveal() {
        let ctx = egui::Context::default();
        page_frame(&ctx, 0.0, "home", vec![]);
        choose_next(&ctx, PageTransition::CircleOut);
        page_frame(&ctx, 1.0, "playlist:reveal", vec![]);
        page_frame(
            &ctx,
            1.1,
            "playlist:reveal",
            click_events(egui::pos2(-10.0, 20.0)),
        );
        assert!(page_state(&ctx).started.is_some());
    }

    #[test]
    fn blocking_a_covered_button_preserves_its_normal_theme_colours() {
        let ctx = egui::Context::default();
        let frame = |time: f64, key: &str, pointer| {
            let mut response = None;
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(640.0, 480.0),
                    )),
                    time: Some(time),
                    events: pointer,
                    ..Default::default()
                },
                |ui| {
                    response = Some(page(
                        ui,
                        Id::new("page-slide-test"),
                        Id::new(key),
                        Id::new("button-theme"),
                        |ui| ui.add_sized([200.0, 44.0], egui::Button::new("Play album")),
                    ));
                    finish(ui.ctx());
                },
            );
            let response = response.unwrap();
            let colours = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Rect(rect) if rect.rect == response.rect => {
                        Some((rect.fill, rect.stroke))
                    }
                    _ => None,
                })
                .expect("a native themed button is painted");
            output.textures_delta.clear();
            (response, colours)
        };
        let (initial, _) = frame(0.0, "home", vec![]);
        // Match the pointer state: egui computes button styles from the
        // previous widget's interaction, including this frame's hover.
        let (_, normal) = frame(
            0.01,
            "home",
            vec![egui::Event::PointerMoved(initial.rect.center())],
        );
        choose_next(&ctx, PageTransition::CircleOut);
        let (blocked, current) = frame(
            1.0,
            "playlist:reveal",
            vec![egui::Event::PointerMoved(initial.rect.center())],
        );
        assert!(!blocked.enabled());
        assert_eq!(
            normal, current,
            "covered controls retain normal colours and opacity"
        );
    }

    #[test]
    fn late_toast_atlas_growth_drops_only_the_old_overlay_and_preserves_the_shuffle() {
        let ctx = egui::Context::default();
        page_frame(&ctx, 0.0, "home", vec![]);
        choose_next(&ctx, PageTransition::CircleOut);
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640.0, 480.0),
                )),
                time: Some(1.0),
                ..Default::default()
            },
            |ui| {
                page(
                    ui,
                    Id::new("page-slide-test"),
                    Id::new("playlist:late-toast"),
                    Id::new("test-scene"),
                    |ui| {
                        ui.label("New playlist");
                    },
                );
                let pending = ctx
                    .data(|data| data.get_temp::<PendingOverlay>(pending_overlay_id()))
                    .unwrap();
                let choices = page_state(&ctx).choices;
                assert!(!pending.shapes.is_empty());
                ui.label(
                    egui::RichText::new(
                        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
                    )
                    .size(160.0),
                );
                assert_ne!(
                    pending.font_size,
                    ctx.fonts(|fonts| fonts.font_image_size())
                );
                finish(&ctx);
                assert!(page_state(&ctx).started.is_none());
                assert_eq!(page_state(&ctx).choices, choices);
                assert!(
                    ctx.data(|data| data
                        .get_temp::<PageReveal>(pending.arrival.with("active-page-reveal")))
                        .is_none()
                );
                ctx.graphics(|graphics| {
                    let shapes = graphics
                        .get(pending.layer)
                        .unwrap()
                        .all_entries()
                        .collect::<Vec<_>>();
                    for index in pending.shapes {
                        assert!(matches!(shapes[index.0].shape, egui::Shape::Noop));
                    }
                    assert!(
                        shapes
                            .iter()
                            .any(|shape| matches!(shape.shape, egui::Shape::Text(_))),
                        "the new page and later toast stay painted"
                    );
                });
            },
        );
        output.textures_delta.clear();
    }

    // Sample the real tessellated paint, including glyph coverage from the
    // current atlas. This catches a missing canvas that layout-only tests miss.
    fn sample_paint(
        ctx: &egui::Context,
        primitives: &[egui::epaint::ClippedPrimitive],
        points: &[egui::Pos2],
    ) -> Vec<[f32; 4]> {
        let atlas = ctx.fonts(|fonts| fonts.image());
        let cross = |a: Vec2, b: Vec2| a.x * b.y - a.y * b.x;
        points
            .iter()
            .map(|&point| {
                let mut colour = [0.0; 4];
                for primitive in primitives {
                    if !primitive.clip_rect.contains(point) {
                        continue;
                    }
                    let egui::epaint::Primitive::Mesh(mesh) = &primitive.primitive else {
                        panic!("the native body has no renderer callbacks");
                    };
                    assert_eq!(mesh.texture_id, egui::TextureId::default());
                    for triangle in mesh.indices.as_chunks::<3>().0 {
                        let [a, b, c] = triangle.map(|index| mesh.vertices[index as usize]);
                        let area = cross(b.pos - a.pos, c.pos - a.pos);
                        if area.abs() < 0.00001 {
                            continue;
                        }
                        let wa = cross(b.pos - point, c.pos - point) / area;
                        let wb = cross(c.pos - point, a.pos - point) / area;
                        let wc = 1.0 - wa - wb;
                        if wa < 0.0 || wb < 0.0 || wc < 0.0 {
                            continue;
                        }
                        let uv = a.uv.to_vec2() * wa + b.uv.to_vec2() * wb + c.uv.to_vec2() * wc;
                        let x = ((uv.x * atlas.size[0] as f32) as usize).min(atlas.size[0] - 1);
                        let y = ((uv.y * atlas.size[1] as f32) as usize).min(atlas.size[1] - 1);
                        let texel = atlas[(x, y)].to_array();
                        let a = a.color.to_array();
                        let b = b.color.to_array();
                        let c = c.color.to_array();
                        let source: [f32; 4] = std::array::from_fn(|channel| {
                            (f32::from(a[channel]) * wa
                                + f32::from(b[channel]) * wb
                                + f32::from(c[channel]) * wc)
                                * f32::from(texel[channel])
                                / (255.0 * 255.0)
                        });
                        for channel in 0..4 {
                            colour[channel] = source[channel] + colour[channel] * (1.0 - source[3]);
                        }
                    }
                }
                colour
            })
            .collect()
    }

    fn paint_points() -> Vec<egui::Pos2> {
        (0..12)
            .flat_map(|y| {
                (0..16).map(move |x| egui::pos2(31.37 + x as f32 * 35.0, 73.61 + y as f32 * 31.0))
            })
            .collect()
    }

    fn framed_page(
        ctx: &egui::Context,
        time: f64,
        index: usize,
        fill: egui::Color32,
        gradient: bool,
        probe_placeholder: bool,
    ) -> (Vec<[f32; 4]>, Option<bool>) {
        let mut before_frame_end = None;
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640.0, 480.0),
                )),
                time: Some(time),
                ..Default::default()
            },
            |ui| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(fill))
                    .show(ui, |ui| {
                        if gradient {
                            let rect = ui.max_rect();
                            let mut mesh = egui::epaint::Mesh::default();
                            mesh.add_colored_rect(rect, fill);
                            mesh.vertices[2].color = egui::Color32::from_rgb(115, 23, 63);
                            mesh.vertices[3].color = egui::Color32::from_rgb(115, 23, 63);
                            ui.painter().add(egui::Shape::mesh(mesh));
                        }
                        ui.add_space(52.0);
                        ui.set_clip_rect(egui::Rect::from_min_max(
                            egui::pos2(20.0, 60.0),
                            egui::pos2(620.0, 460.0),
                        ));
                        let key = Id::new(("framed page", index));
                        let scene = Id::new(("framed scene", gradient));
                        page(ui, Id::new("page-slide-test"), key, scene, |ui| {
                            let origin = egui::pos2(30.0, 66.0 + (index % 3) as f32 * 17.0);
                            for row in 0..16 {
                                ui.painter().text(
                                    origin + egui::vec2(0.0, row as f32 * 22.0),
                                    egui::Align2::LEFT_TOP,
                                    format!("Playlist {index}, song {row}: abcdefghijklmnopqrstuvwxyz 0123456789"),
                                    egui::FontId::proportional(16.0),
                                    if fill.r() > 200 { egui::Color32::BLACK } else { egui::Color32::WHITE },
                                );
                            }
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(
                                    egui::pos2(440.0, 100.0 + (index % 4) as f32 * 65.0),
                                    egui::vec2(150.0, 50.0),
                                ),
                                6,
                                egui::Color32::from_rgb(60 + (index % 5) as u8 * 30, 50, 90),
                            );
                        });
                        if probe_placeholder {
                            let pending = ctx.data(|data| data.get_temp::<PendingCapture>(pending_capture_id())).unwrap();
                            before_frame_end = Some(
                                pending.request.complete(ctx, None).unwrap()
                                    .picture(ui, key, scene).unwrap().opaque(),
                            );
                        }
                    });
                // The real show() calls finish at this stage, after the
                // CentralPanel's delayed background replacement.
                finish(ctx);
            },
        );
        let primitives = ctx.tessellate(std::mem::take(&mut output.shapes), ctx.pixels_per_point());
        let samples = sample_paint(ctx, &primitives, &paint_points());
        output.textures_delta.clear();
        (samples, before_frame_end)
    }

    fn assert_same_paint(before: &[[f32; 4]], after: &[[f32; 4]], context: &str) {
        assert_eq!(before.len(), after.len());
        for (index, (before, after)) in before.iter().zip(after).enumerate() {
            for channel in 0..4 {
                assert!(
                    (before[channel] - after[channel]).abs() < 0.025,
                    "{context}: sample {index}, channel {channel}: {before:?} -> {after:?}"
                );
            }
        }
    }

    #[test]
    fn delayed_panel_canvas_is_captured_before_old_and_new_text_are_composited() {
        for fill in [
            egui::Color32::from_rgb(26, 15, 22),
            egui::Color32::from_rgb(246, 245, 250),
        ] {
            for gradient in [false, true] {
                for transition in PageTransition::ALL {
                    let ctx = egui::Context::default();
                    let (old, before_end) = framed_page(&ctx, 0.0, 0, fill, gradient, true);
                    assert_eq!(
                        before_end,
                        Some(false),
                        "Frame still has a Noop background inside its closure"
                    );
                    choose_next(&ctx, transition);
                    let (started, _) = framed_page(&ctx, 1.0, 1, fill, gradient, false);
                    let active = ctx
                        .data(|data| {
                            data.get_temp::<PageReveal>(
                                Id::new("page-slide-test").with("active-page-reveal"),
                            )
                        })
                        .unwrap();
                    assert!(
                        active.picture.unwrap().opaque(),
                        "the completed real Frame supplies an opaque old canvas"
                    );
                    assert_same_paint(
                        &old,
                        &started,
                        &format!("{transition:?}, gradient {gradient}, fill {fill:?}"),
                    );
                    assert!(
                        started.iter().all(|pixel| pixel[3] > 0.999),
                        "old text has its own opaque canvas above all new text"
                    );
                }
            }
        }
    }

    #[test]
    fn rapid_navigation_starts_from_the_visible_composite_and_bounds_retained_geometry() {
        let ctx = egui::Context::default();
        seed_page_transitions(&ctx, 7);
        let fill = egui::Color32::from_rgb(26, 15, 22);
        let (mut previous, _) = framed_page(&ctx, 0.0, 0, fill, false, false);
        let mut max_vertices = 0;
        for index in 1..=32 {
            let now = 1.0 + (index - 1) as f64 * 0.16;
            let (first, _) = framed_page(&ctx, now, index, fill, false, false);
            assert_eq!(
                page_state(&ctx).started,
                Some(now),
                "retarget {index} keeps a running reveal"
            );
            assert_same_paint(&previous, &first, &format!("rapid target {index}"));
            let active = ctx
                .data(|data| {
                    data.get_temp::<PageReveal>(
                        Id::new("page-slide-test").with("active-page-reveal"),
                    )
                })
                .unwrap();
            let (vertices, indices) =
                active
                    .picture
                    .unwrap()
                    .primitives()
                    .iter()
                    .fold((0, 0), |cost, item| {
                        let egui::epaint::Primitive::Mesh(mesh) = &item.primitive else {
                            unreachable!()
                        };
                        (cost.0 + mesh.vertices.len(), cost.1 + mesh.indices.len())
                    });
            assert!(vertices <= 180_000 && indices <= 540_000);
            eprintln!("retarget {index}: {vertices} vertices, {indices} indices");
            max_vertices = max_vertices.max(vertices);
            (previous, _) = framed_page(&ctx, now + 0.12, index, fill, false, false);
        }
        assert!(
            max_vertices < 90_000,
            "occluded pages must be pruned, observed {max_vertices} vertices"
        );
        framed_page(&ctx, 7.0, 32, fill, false, false);
        assert!(page_state(&ctx).started.is_none());
        assert!(
            ctx.data(|data| data
                .get_temp::<PageReveal>(Id::new("page-slide-test").with("active-page-reveal")))
                .is_none()
        );
    }

    #[test]
    fn repeated_same_destination_clicks_preserve_the_clock_and_shuffle() {
        let ctx = egui::Context::default();
        page_frame(&ctx, 0.0, "home", vec![]);
        choose_next(&ctx, PageTransition::CircleOut);
        page_frame(&ctx, 1.0, "playlist:a", vec![]);
        let initial = page_state(&ctx);
        for time in [1.1, 1.2, 1.3, 1.4] {
            page_frame(
                &ctx,
                time,
                "playlist:a",
                click_events(egui::pos2(320.0, 240.0)),
            );
            let state = page_state(&ctx);
            assert_eq!(state.started, initial.started);
            assert_eq!(state.choices, initial.choices);
        }
    }

    #[test]
    fn an_oversized_composite_settles_and_recovers_without_retaining_the_huge_mesh() {
        let ctx = egui::Context::default();
        let id = Id::new("page-slide-test");
        page_frame(&ctx, 0.0, "home", vec![]);
        choose_next(&ctx, PageTransition::CircleOut);
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640.0, 480.0),
                )),
                time: Some(1.0),
                ..Default::default()
            },
            |ui| {
                page(ui, id, Id::new("huge"), Id::new("test-scene"), |ui| {
                    let mut mesh = egui::epaint::Mesh::default();
                    for _ in 0..45_001 {
                        mesh.add_colored_rect(ui.clip_rect(), egui::Color32::WHITE);
                    }
                    ui.painter().add(egui::Shape::mesh(mesh));
                });
                assert!(page_state(&ctx).started.is_some());
                finish(&ctx);
                assert!(page_state(&ctx).started.is_none());
                assert!(
                        ctx.data(
                            |data| data.get_temp::<reveal::Snapshot>(id.with("last-page-picture"))
                        )
                        .is_none()
                    );
                assert!(
                    ctx.data(|data| data.get_temp::<PageReveal>(id.with("active-page-reveal")))
                        .is_none()
                );
            },
        );
        output.textures_delta.clear();
        page_frame(&ctx, 1.1, "huge", vec![]);
        assert!(
            ctx.data(|data| data.get_temp::<reveal::Snapshot>(id.with("last-page-picture")))
                .is_some()
        );
        page_frame(&ctx, 1.2, "recovered", vec![]);
        assert_eq!(page_state(&ctx).started, Some(1.2));
    }
}
