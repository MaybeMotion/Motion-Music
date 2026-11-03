//! The app's circular theme reveal, with the same speed ramp as page motion.
//! Captures stay in the renderer; no image files or background work are needed.
use egui::{Color32, Context, Id, LayerId, Mesh, Order, Pos2, Rect, TextureHandle, Vec2};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

pub(crate) const DURATION: f64 = 0.4;
const CAPTURE_WAIT: f64 = 0.25;
const FEATHER: f32 = 48.0;
const SEGMENTS: usize = 96;

#[derive(Clone, Debug)]
struct Request(u64);

#[derive(Clone, Copy)]
struct Geometry {
    rect: Rect,
    scale: f32,
}
impl Geometry {
    fn current(ctx: &Context) -> Self {
        Self {
            rect: ctx.viewport_rect(),
            scale: ctx.pixels_per_point(),
        }
    }
    fn matches(self, ctx: &Context) -> bool {
        self.rect.is_finite()
            && self.rect.is_positive()
            && self.rect == ctx.viewport_rect()
            && self.scale == ctx.pixels_per_point()
    }
    fn image_fits(self, image: &egui::ColorImage) -> bool {
        let size = self.rect.size() * self.scale;
        image.size[0].abs_diff(size.x.round().max(1.0) as usize) <= 1
            && image.size[1].abs_diff(size.y.round().max(1.0) as usize) <= 1
    }
}
struct Picture {
    texture: TextureHandle,
    geometry: Geometry,
    started: f64,
}
#[derive(Default)]
enum State {
    #[default]
    Idle,
    Capturing {
        request: u64,
        since: f64,
        geometry: Geometry,
        previous: Option<Picture>,
    },
    Revealing(Picture),
}
#[derive(Default)]
pub(crate) struct ThemeTransition {
    state: State,
}
impl ThemeTransition {
    pub fn active(&self) -> bool {
        !matches!(self.state, State::Idle)
    }

    pub fn begin(&mut self, ctx: &Context) {
        if matches!(self.state, State::Capturing { .. }) {
            return;
        }
        if crate::motion::reduced(ctx) {
            self.state = State::Idle;
            return;
        }
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let request = NEXT.fetch_add(1, Ordering::Relaxed);
        let previous = match std::mem::take(&mut self.state) {
            State::Revealing(picture) if picture.geometry.matches(ctx) => Some(picture),
            _ => None,
        };
        self.state = State::Capturing {
            request,
            since: ctx.input(|input| input.time),
            geometry: Geometry::current(ctx),
            previous,
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
            Request(request),
        )));
        ctx.request_repaint();
    }

    /// Keep the old palette until its own marked frame arrives, with a bounded
    /// wait for hidden windows or renderers which cannot supply a screenshot.
    pub fn holding(&mut self, ctx: &Context) -> bool {
        let State::Capturing {
            request,
            since,
            geometry,
            ..
        } = &self.state
        else {
            return false;
        };
        let now = ctx.input(|input| input.time);
        if crate::motion::reduced(ctx) || !now.is_finite() || !geometry.matches(ctx) {
            self.state = State::Idle;
            return false;
        }
        let image = ctx.input(|input| {
            input.raw.events.iter().find_map(|event| {
                let egui::Event::Screenshot {
                    user_data, image, ..
                } = event
                else {
                    return None;
                };
                user_data
                    .data
                    .as_ref()
                    .and_then(|data| data.downcast_ref::<Request>())
                    .filter(|marker| marker.0 == *request)
                    .map(|_| Arc::clone(image))
            })
        });
        if let Some(image) = image {
            if !geometry.image_fits(&image) {
                self.state = State::Idle;
                return false;
            }
            let picture = Picture {
                texture: ctx.load_texture(
                    "motion-theme-reveal",
                    egui::ImageData::Color(image),
                    egui::TextureOptions::LINEAR,
                ),
                geometry: *geometry,
                started: now,
            };
            self.state = State::Revealing(picture);
            ctx.request_repaint();
            return false;
        }
        if now - since >= CAPTURE_WAIT || now < *since {
            self.state = State::Idle;
            return false;
        }
        ctx.request_repaint();
        true
    }

    /// Returning to the already applied palette makes a pending capture moot.
    pub fn cancel_capture(&mut self) {
        if matches!(self.state, State::Capturing { .. }) {
            let State::Capturing { previous, .. } = std::mem::take(&mut self.state) else {
                unreachable!();
            };
            self.state = previous.map_or(State::Idle, State::Revealing);
        }
    }

    pub fn paint(&mut self, ctx: &Context) {
        if crate::motion::reduced(ctx) {
            self.state = State::Idle;
            return;
        }
        let now = ctx.input(|input| input.time);
        let picture = match &mut self.state {
            State::Idle => return,
            State::Capturing {
                since,
                geometry,
                previous,
                ..
            } => {
                if !now.is_finite()
                    || now < *since
                    || now - *since >= CAPTURE_WAIT
                    || !geometry.matches(ctx)
                {
                    self.state = State::Idle;
                    return;
                }
                if previous
                    .as_ref()
                    .is_some_and(|picture| finished(picture, now, ctx))
                {
                    *previous = None;
                }
                ctx.request_repaint();
                let Some(picture) = previous.as_ref() else {
                    return;
                };
                picture
            }
            State::Revealing(picture) => {
                if finished(picture, now, ctx) {
                    self.state = State::Idle;
                    return;
                }
                picture
            }
        };
        let progress = crate::motion::ramp(((now - picture.started) / DURATION) as f32);
        let painter = ctx.layer_painter(LayerId::new(Order::Debug, Id::new("motion-theme-reveal")));
        painter
            .with_clip_rect(picture.geometry.rect)
            .add(egui::Shape::mesh(circle_mesh(
                picture.geometry.rect,
                picture.texture.id(),
                progress,
            )));
        ctx.request_repaint();
    }
}
fn finished(picture: &Picture, now: f64, ctx: &Context) -> bool {
    !now.is_finite()
        || now < picture.started
        || now - picture.started + 1e-9 >= DURATION
        || !picture.geometry.matches(ctx)
}

/// Three radial bands describe a clear centre, a soft edge and opaque outside.
/// Every vertex keeps the original frame's screen-to-texture mapping.
fn circle_mesh(rect: Rect, texture: egui::TextureId, progress: f32) -> Mesh {
    if progress <= 0.0 {
        let mut mesh = Mesh::with_texture(texture);
        mesh.add_rect_with_uv(
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
        return mesh;
    }
    let centre = rect.center();
    let diagonal = rect.size().length() * 0.5;
    let outer = progress.clamp(0.0, 1.0) * (diagonal + FEATHER);
    let radii = [(outer - FEATHER).max(0.0), outer, diagonal * 2.0 + FEATHER];
    let mut mesh = Mesh::with_texture(texture);
    let size = rect.size().max(Vec2::splat(1.0));
    for (band, radius) in radii.into_iter().enumerate() {
        for segment in 0..SEGMENTS {
            let angle = std::f32::consts::TAU * segment as f32 / SEGMENTS as f32;
            let point = centre + Vec2::angled(angle) * radius;
            mesh.vertices.push(egui::epaint::Vertex {
                pos: point,
                uv: Pos2::new(
                    (point.x - rect.min.x) / size.x,
                    (point.y - rect.min.y) / size.y,
                ),
                color: if band == 0 {
                    Color32::TRANSPARENT
                } else {
                    Color32::WHITE
                },
            });
        }
    }
    for band in 0..2 {
        for segment in 0..SEGMENTS {
            let a = (band * SEGMENTS + segment) as u32;
            let b = (band * SEGMENTS + (segment + 1) % SEGMENTS) as u32;
            let c = a + SEGMENTS as u32;
            let d = b + SEGMENTS as u32;
            mesh.add_triangle(a, c, d);
            mesh.add_triangle(a, d, b);
        }
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(
        ctx: &Context,
        time: f64,
        events: Vec<egui::Event>,
        draw: impl FnMut(&mut egui::Ui),
    ) -> egui::FullOutput {
        let mut out = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(160.0, 120.0))),
                events,
                ..Default::default()
            },
            draw,
        );
        out.textures_delta.clear();
        out
    }
    fn screenshot(request: u64) -> egui::Event {
        egui::Event::Screenshot {
            viewport_id: egui::ViewportId::ROOT,
            user_data: egui::UserData::new(Request(request)),
            image: Arc::new(egui::ColorImage::filled([160, 120], Color32::RED)),
        }
    }
    fn request(transition: &ThemeTransition) -> u64 {
        let State::Capturing { request, .. } = transition.state else {
            panic!("capture requested");
        };
        request
    }
    fn start(ctx: &Context, transition: &mut ThemeTransition) {
        frame(ctx, 0.0, vec![], |ui| {
            transition.begin(ui.ctx());
            assert!(transition.holding(ui.ctx()));
        });
        let id = request(transition);
        frame(ctx, 0.05, vec![screenshot(id)], |ui| {
            assert!(!transition.holding(ui.ctx()));
            transition.paint(ui.ctx());
        });
        assert!(matches!(transition.state, State::Revealing(_)));
    }
    #[test]
    fn marked_frame_holds_once_and_reveal_finishes_in_four_hundred_ms() {
        let ctx = Context::default();
        let mut transition = ThemeTransition::default();
        start(&ctx, &mut transition);
        frame(&ctx, 0.25, vec![], |ui| transition.paint(ui.ctx()));
        assert!(transition.active());
        frame(&ctx, 0.45, vec![], |ui| transition.paint(ui.ctx()));
        assert!(!transition.active());
        frame(&ctx, 0.6, vec![], |ui| transition.paint(ui.ctx()));
        assert!(!transition.active(), "settled motion never restarts");
    }
    #[test]
    fn capture_ignores_stale_requests_and_has_a_bounded_wait() {
        let ctx = Context::default();
        let mut transition = ThemeTransition::default();
        frame(&ctx, 0.0, vec![], |ui| transition.begin(ui.ctx()));
        let stale = request(&transition);
        transition.cancel_capture();
        frame(&ctx, 0.01, vec![], |ui| transition.begin(ui.ctx()));
        let current = request(&transition);
        assert_ne!(stale, current);
        frame(&ctx, 0.02, vec![screenshot(stale)], |ui| {
            assert!(transition.holding(ui.ctx()))
        });
        assert_eq!(request(&transition), current);
        frame(&ctx, 0.27, vec![], |ui| {
            assert!(!transition.holding(ui.ctx()))
        });
        assert!(!transition.active());
    }
    #[test]
    fn rapid_theme_request_keeps_the_visible_picture_while_capturing() {
        let ctx = Context::default();
        let mut transition = ThemeTransition::default();
        start(&ctx, &mut transition);
        frame(&ctx, 0.15, vec![], |ui| {
            transition.begin(ui.ctx());
            assert!(transition.holding(ui.ctx()));
            transition.paint(ui.ctx());
        });
        assert!(matches!(
            transition.state,
            State::Capturing {
                previous: Some(_),
                ..
            }
        ));
        let id = request(&transition);
        frame(&ctx, 0.16, vec![screenshot(id)], |ui| {
            assert!(!transition.holding(ui.ctx()));
            transition.paint(ui.ctx());
        });
        assert!(matches!(transition.state, State::Revealing(_)));
    }
    #[test]
    fn cancelling_a_retarget_preserves_the_visible_reveal_and_its_clock() {
        let ctx = Context::default();
        let mut transition = ThemeTransition::default();
        start(&ctx, &mut transition);
        let State::Revealing(original) = &transition.state else {
            unreachable!();
        };
        let texture = original.texture.id();
        let started = original.started;
        frame(&ctx, 0.15, vec![], |ui| {
            transition.begin(ui.ctx());
            transition.cancel_capture();
            transition.paint(ui.ctx());
        });
        let State::Revealing(restored) = &transition.state else {
            panic!("the visible reveal must survive cancellation of a pending retarget");
        };
        assert_eq!(restored.texture.id(), texture);
        assert_eq!(restored.started, started);
        frame(&ctx, 0.45, vec![], |ui| transition.paint(ui.ctx()));
        assert!(
            !transition.active(),
            "cancellation must not extend the reveal"
        );
    }
    #[test]
    fn reduced_motion_and_geometry_changes_release_the_picture() {
        for reduced in [true, false] {
            let ctx = Context::default();
            let mut transition = ThemeTransition::default();
            start(&ctx, &mut transition);
            if reduced {
                crate::motion::apply(&ctx, true);
            } else {
                ctx.set_pixels_per_point(1.5);
            }
            frame(&ctx, 0.2, vec![], |ui| transition.paint(ui.ctx()));
            assert!(!transition.active());
        }
    }
    #[test]
    fn circle_has_exact_start_and_no_opaque_end_pixels_inside_the_frame() {
        let rect = Rect::from_min_max(egui::pos2(30.0, 40.0), egui::pos2(1310.0, 940.0));
        let texture = egui::TextureId::Managed(7);
        let start = circle_mesh(rect, texture, 0.0);
        assert_eq!(start.vertices.len(), 4);
        assert!(start.vertices.iter().all(|v| v.color == Color32::WHITE));
        let end = circle_mesh(rect, texture, 1.0);
        for v in &end.vertices {
            if v.color.a() > 0 {
                assert!(v.pos.distance(rect.center()) > rect.size().length() * 0.5);
            }
            let uv = egui::pos2(
                (v.pos.x - rect.min.x) / rect.width(),
                (v.pos.y - rect.min.y) / rect.height(),
            );
            assert!((uv - v.uv).length() < 1e-5);
        }
        assert_eq!(end.vertices.len(), SEGMENTS * 3);
    }
}
