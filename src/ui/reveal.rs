//! CPU geometry for short native page reveals. No renderer callbacks or screenshots.
use std::{collections::HashMap, sync::Arc};

use egui::epaint::{ClippedPrimitive, ClippedShape, Mesh, Primitive, Vertex};
use egui::{Color32, Pos2, Rect, Shape, Vec2};

const CIRCLE_SIDES: usize = 96;
const MAX_SHAPES: usize = 6_000;
const MAX_VERTICES: usize = 180_000;
const MAX_INDICES: usize = 540_000;
const EDGE: f32 = 1.25;
/// Half the soft edge of the straight and angled reveals: a fade about 20
/// points wide, so no style shows a knife-sharp line.
const SOFT_EDGE: f32 = 10.0;
/// The clock hand's edge stays narrower: its quarters are separate parts,
/// and a wide fade would be cut off where they meet at the centre.
const CLOCK_EDGE: f32 = 3.0;
/// The ocean wave: the length and height of its main swell, in points.
const WAVE_LENGTH: f32 = 260.0;
const WAVE_HEIGHT: f32 = 20.0;
/// How far, in points, each straight piece of the wave's curved edge spans.
const WAVE_STEP: f32 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Kind {
    CircleOut,
    CircleIn,
    CornerCircle,
    Diagonal,
    Curtain,
    SwipeLeft,
    SwipeRight,
    SwipeUp,
    SwipeDown,
    Diamond,
    /// A circle opening from a point, such as the click that changed the page.
    Spot(Pos2),
    /// Horizontal slats that each open downwards together.
    Blinds,
    /// Columns that drop open one after another from the left.
    Cascade,
    /// Horizontal bands that slide open in alternating directions.
    Zipper,
    /// Opens from the horizontal midline towards the top and bottom.
    Letterbox,
    /// An arrowhead sweeping from left to right.
    Chevron,
    /// A clock hand sweeping once around the centre.
    Clock,
    /// A soft, rolling wave edge washing across the page in one direction,
    /// like the sea running up a beach.
    Wave(Wipe),
}

const BLIND_ROWS: usize = 8;
const ZIPPER_ROWS: usize = 6;
const CASCADE_COLUMNS: usize = 7;
/// How much of a cascade's time is spent starting later columns.
const CASCADE_STAGGER: f32 = 0.55;
const CHEVRON_SLOPE: f32 = 0.6;

/// The direction a wipe travels: the new page appears from the opposite side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Wipe {
    Left,
    Right,
    Up,
    Down,
}

impl Wipe {
    /// The unit direction of travel, and the unit direction along the edge.
    fn axes(self) -> (Vec2, Vec2) {
        match self {
            Self::Right => (Vec2::X, Vec2::Y),
            Self::Left => (-Vec2::X, Vec2::Y),
            Self::Down => (Vec2::Y, Vec2::X),
            Self::Up => (-Vec2::Y, Vec2::X),
        }
    }
}

/// One rectangular part of the page with its own straight wipe. `rect` is
/// the visible travel; `region` extends the outermost strips without limit so
/// geometry beyond the page edge still belongs to exactly one strip.
#[derive(Clone, Copy)]
struct Strip {
    rect: Rect,
    region: Rect,
    wipe: Wipe,
    progress: f32,
    edge: f32,
}

impl Strip {
    fn boundary(&self) -> f32 {
        let progress = self.progress.clamp(0.0, 1.0);
        let edge = self.edge;
        match self.wipe {
            Wipe::Left => {
                self.rect.right() + edge * 2.0 - (self.rect.width() + edge * 4.0) * progress
            }
            Wipe::Right => {
                self.rect.left() - edge * 2.0 + (self.rect.width() + edge * 4.0) * progress
            }
            Wipe::Up => {
                self.rect.bottom() + edge * 2.0 - (self.rect.height() + edge * 4.0) * progress
            }
            Wipe::Down => {
                self.rect.top() - edge * 2.0 + (self.rect.height() + edge * 4.0) * progress
            }
        }
    }
    fn plane(&self) -> Plane {
        let boundary = self.boundary();
        let edge = self.edge;
        match self.wipe {
            Wipe::Left => Plane {
                normal: Vec2::X,
                offset: boundary + edge,
            },
            Wipe::Right => Plane {
                normal: -Vec2::X,
                offset: -boundary + edge,
            },
            Wipe::Up => Plane {
                normal: Vec2::Y,
                offset: boundary + edge,
            },
            Wipe::Down => Plane {
                normal: -Vec2::Y,
                offset: -boundary + edge,
            },
        }
    }
    fn partition(&self) -> Vec<Plane> {
        let mut planes = Vec::with_capacity(2);
        if self.region.left().is_finite() {
            planes.push(Plane {
                normal: -Vec2::X,
                offset: -self.region.left(),
            });
        }
        if self.region.right().is_finite() {
            planes.push(Plane {
                normal: Vec2::X,
                offset: self.region.right(),
            });
        }
        if self.region.top().is_finite() {
            planes.push(Plane {
                normal: -Vec2::Y,
                offset: -self.region.top(),
            });
        }
        if self.region.bottom().is_finite() {
            planes.push(Plane {
                normal: Vec2::Y,
                offset: self.region.bottom(),
            });
        }
        planes
    }
}

/// The old page's opacity across a feather `edge * 2` wide ending at `plane`,
/// whose normal has unit length.
fn plane_alpha(plane: Plane, p: Pos2, edge: f32) -> f32 {
    (-plane.distance(p) / (edge * 2.0)).clamp(0.0, 1.0)
}

/// A unit direction `angle` radians clockwise from straight up.
fn clockwise(angle: f32) -> Vec2 {
    Vec2::new(angle.sin(), -angle.cos())
}

#[derive(Clone, Copy)]
struct Plane {
    normal: Vec2,
    offset: f32,
}
impl Plane {
    fn distance(self, p: Pos2) -> f32 {
        self.normal.dot(p.to_vec2()) - self.offset
    }
}

/// Body metadata recorded while its Ui exists. Shapes are collected only
/// after the enclosing Frame has replaced its background placeholder.
#[derive(Clone)]
pub(super) struct Capture {
    key: egui::Id,
    scene: egui::Id,
    rect: Rect,
    pixels_per_point: f32,
    appearance: egui::Visuals,
    layer: egui::LayerId,
}

#[derive(Clone, Copy)]
pub(super) struct Cover {
    pub kind: Kind,
    pub progress: f32,
    pub first_shape: usize,
    pub last_shape: usize,
    pub opaque: bool,
}

#[derive(Clone)]
pub(super) struct Snapshot {
    key: egui::Id,
    scene: egui::Id,
    rect: Rect,
    pixels_per_point: f32,
    appearance: egui::Visuals,
    shapes: Arc<[ClippedShape]>,
    font_size: [usize; 2],
    opaque: bool,
    // The underlay precedes the cached old-page overlay. On retarget we can
    // discard only its safely occluded geometry, rather than stacking pages.
    cover: Option<(Kind, f32, usize, usize)>,
    // Keep album art alive even when navigation drops its last application handle.
    textures: Arc<[egui::TextureHandle]>,
}

#[derive(Clone)]
pub(super) struct Picture {
    scene: egui::Id,
    rect: Rect,
    pixels_per_point: f32,
    appearance: egui::Visuals,
    meshes: Arc<[ClippedPrimitive]>,
    opaque: bool,
    textures: Arc<[(egui::TextureId, [usize; 2])]>,
    _handles: Arc<[egui::TextureHandle]>,
}

fn shape_textures(shape: &Shape, ids: &mut Vec<egui::TextureId>) -> bool {
    if let Shape::Vec(shapes) = shape {
        return shapes.iter().all(|shape| shape_textures(shape, ids));
    }
    if matches!(shape, Shape::Callback(_)) {
        return false;
    }
    let id = shape.texture_id();
    // User-owned GL resources have no reference-counted egui lifetime.
    if matches!(id, egui::TextureId::User(_)) {
        return false;
    }
    if !ids.contains(&id) {
        ids.push(id);
    }
    true
}

fn retain_textures(
    ctx: &egui::Context,
    shapes: &[ClippedShape],
) -> Option<Arc<[egui::TextureHandle]>> {
    let mut ids = Vec::new();
    if !shapes
        .iter()
        .all(|shape| shape_textures(&shape.shape, &mut ids))
    {
        return None;
    }
    let manager = ctx.tex_manager();
    let mut locked = manager.write();
    if ids.iter().any(|&id| locked.meta(id).is_none()) {
        return None;
    }
    for &id in &ids {
        locked.retain(id);
    }
    drop(locked);
    Some(
        ids.into_iter()
            .map(|id| egui::TextureHandle::new(manager.clone(), id))
            .collect(),
    )
}

pub(super) fn prepare(ui: &egui::Ui, key: egui::Id, scene: egui::Id) -> Option<Capture> {
    let rect = ui.clip_rect();
    (rect.is_finite() && rect.is_positive()).then(|| Capture {
        key,
        scene,
        rect,
        pixels_per_point: ui.pixels_per_point(),
        appearance: ui.visuals().clone(),
        layer: ui.layer_id(),
    })
}

fn mesh_budget(shape: &Shape) -> (usize, usize) {
    match shape {
        Shape::Mesh(mesh) => (mesh.vertices.len(), mesh.indices.len()),
        Shape::Vec(shapes) => shapes.iter().map(mesh_budget).fold((0, 0), |a, b| {
            (a.0.saturating_add(b.0), a.1.saturating_add(b.1))
        }),
        _ => (0, 0),
    }
}

impl Capture {
    pub(super) fn complete(&self, ctx: &egui::Context, cover: Option<Cover>) -> Option<Snapshot> {
        let rect = self.rect;
        let (shapes, underlay_end, overlay_end, opaque) = ctx.graphics(|g| {
            let list = g.get(self.layer)?;
            let mut shapes = Vec::new();
            let (mut underlay_end, mut overlay_end) = (0, 0);
            let (mut vertices, mut indices) = (0_usize, 0_usize);
            let mut opaque = false;
            for (index, entry) in list.all_entries().enumerate() {
                if entry.clip_rect.intersects(rect)
                    && entry.shape.visual_bounding_rect().intersects(rect)
                {
                    let cost = mesh_budget(&entry.shape);
                    vertices = vertices.saturating_add(cost.0);
                    indices = indices.saturating_add(cost.1);
                    if shapes.len() == MAX_SHAPES
                        || vertices > MAX_VERTICES
                        || indices > MAX_INDICES
                    {
                        return None;
                    }
                    if let Shape::Rect(background) = &entry.shape {
                        opaque |= background.fill.a() == 255
                            && background.corner_radius == egui::CornerRadius::ZERO
                            && background.brush.is_none()
                            && background.blur_width == 0.0
                            && background.angle == 0.0
                            && background.rect.contains_rect(rect)
                            && entry.clip_rect.contains_rect(rect);
                    }
                    if cover.is_some_and(|cover| index < cover.first_shape) {
                        underlay_end += 1;
                    }
                    if cover.is_some_and(|cover| index <= cover.last_shape) {
                        overlay_end += 1;
                    }
                    shapes.push(ClippedShape {
                        clip_rect: entry.clip_rect.intersect(rect),
                        shape: entry.shape.clone(),
                    });
                }
            }
            Some((shapes, underlay_end, overlay_end, opaque))
        })?;
        let textures = retain_textures(ctx, &shapes)?;
        Some(Snapshot {
            key: self.key,
            scene: self.scene,
            rect,
            pixels_per_point: self.pixels_per_point,
            appearance: self.appearance.clone(),
            shapes: shapes.into(),
            font_size: ctx.fonts(|fonts| fonts.font_image_size()),
            opaque,
            cover: cover
                .filter(|cover| cover.opaque)
                .map(|cover| (cover.kind, cover.progress, underlay_end, overlay_end)),
            textures,
        })
    }
}

#[cfg(test)]
fn capture(ui: &egui::Ui, key: egui::Id, scene: egui::Id) -> Option<Snapshot> {
    prepare(ui, key, scene)?.complete(ui.ctx(), None)
}

fn renormalize_font_meshes(shape: &mut Shape, old: [usize; 2], new: [usize; 2]) {
    match shape {
        Shape::Mesh(mesh) if mesh.texture_id == egui::TextureId::default() && old != new => {
            let scale = egui::vec2(old[0] as f32 / new[0] as f32, old[1] as f32 / new[1] as f32);
            for vertex in &mut Arc::make_mut(mesh).vertices {
                vertex.uv = (vertex.uv.to_vec2() * scale).to_pos2();
            }
        }
        Shape::Vec(shapes) => {
            for shape in shapes {
                renormalize_font_meshes(shape, old, new);
            }
        }
        _ => {}
    }
}

fn matching(ui: &egui::Ui, rect: Rect, ppp: f32, colours: &egui::Visuals) -> bool {
    rect == ui.clip_rect() && ppp == ui.pixels_per_point() && colours == ui.visuals()
}

impl Snapshot {
    pub(super) fn matches(&self, ui: &egui::Ui, previous: egui::Id, scene: egui::Id) -> bool {
        self.key == previous
            && self.scene == scene
            && matching(ui, self.rect, self.pixels_per_point, &self.appearance)
    }
    /// Tessellate only when a real navigation starts, after the new page has
    /// laid out its text so atlas growth cannot invalidate the initial UVs.
    pub(super) fn picture(
        &self,
        ui: &egui::Ui,
        previous: egui::Id,
        scene: egui::Id,
    ) -> Option<Picture> {
        if !self.matches(ui, previous, scene) {
            return None;
        }
        let font_size = ui.ctx().fonts(|fonts| fonts.font_image_size());
        let mut shapes = self.shapes.to_vec();
        // Composite snapshots include normalized glyph meshes from the prior
        // reveal as well as raw text. Atlas growth preserves glyph pixels,
        // so only the existing meshes need their UV denominator updated.
        for shape in &mut shapes {
            renormalize_font_meshes(&mut shape.shape, self.font_size, font_size);
        }
        let meshes = if let Some((kind, progress, underlay_end, overlay_end)) = self.cover {
            let mut result = Vec::new();
            if progress > 0.0 {
                let underlay = ui
                    .ctx()
                    .tessellate(shapes[..underlay_end].to_vec(), self.pixels_per_point);
                let mask = Mask::new(self.rect, kind, progress);
                for mut primitive in underlay {
                    let Primitive::Mesh(mesh) = &primitive.primitive else {
                        return None;
                    };
                    let mesh = mask.underlay(mesh);
                    if !mesh.indices.is_empty() {
                        primitive.primitive = Primitive::Mesh(mesh);
                        result.push(primitive);
                    }
                }
            }
            result.extend(ui.ctx().tessellate(
                shapes[underlay_end..overlay_end].to_vec(),
                self.pixels_per_point,
            ));
            result.extend(
                ui.ctx()
                    .tessellate(shapes[overlay_end..].to_vec(), self.pixels_per_point),
            );
            result
        } else {
            ui.ctx().tessellate(shapes, self.pixels_per_point)
        };
        let mut vertices = 0;
        let mut indices = 0;
        let mut textures = Vec::new();
        // The texture manager receives the atlas update at end_pass. During
        // layout the font image already has its new size, which tessellate
        // uses to normalize glyph UVs.
        let manager = ui.ctx().tex_manager();
        let manager = manager.read();
        for primitive in &meshes {
            let Primitive::Mesh(mesh) = &primitive.primitive else {
                return None;
            };
            vertices += mesh.vertices.len();
            indices += mesh.indices.len();
            if vertices > MAX_VERTICES || indices > MAX_INDICES {
                return None;
            }
            if let egui::TextureId::Managed(_) = mesh.texture_id {
                let size = if mesh.texture_id == egui::TextureId::default() {
                    font_size
                } else {
                    manager.meta(mesh.texture_id)?.size
                };
                if !textures.iter().any(|&(id, _)| id == mesh.texture_id) {
                    textures.push((mesh.texture_id, size));
                }
            }
        }
        Some(Picture {
            scene: self.scene,
            rect: self.rect,
            pixels_per_point: self.pixels_per_point,
            appearance: self.appearance.clone(),
            meshes: meshes.into(),
            opaque: self.opaque,
            textures: textures.into(),
            _handles: self.textures.clone(),
        })
    }
}

impl Picture {
    pub(super) fn opaque(&self) -> bool {
        self.opaque
    }

    #[cfg(test)]
    pub(super) fn primitives(&self) -> &[ClippedPrimitive] {
        &self.meshes
    }

    pub(super) fn valid(&self, ui: &egui::Ui, scene: egui::Id) -> bool {
        if self.scene != scene || !matching(ui, self.rect, self.pixels_per_point, &self.appearance)
        {
            return false;
        }
        let font_size = ui.ctx().fonts(|fonts| fonts.font_image_size());
        let manager = ui.ctx().tex_manager();
        let manager = manager.read();
        self.textures.iter().all(|&(id, size)| {
            if id == egui::TextureId::default() {
                font_size == size
            } else {
                manager.meta(id).is_some_and(|meta| meta.size == size)
            }
        })
    }

    pub(super) fn paint(
        &self,
        ui: &egui::Ui,
        kind: Kind,
        progress: f32,
    ) -> Vec<egui::layers::ShapeIdx> {
        let mask = Mask::new(self.rect, kind, progress);
        let mut indices = Vec::new();
        for primitive in self.meshes.iter() {
            let Primitive::Mesh(mesh) = &primitive.primitive else {
                continue;
            };
            let mesh = mask.mesh(mesh);
            if !mesh.indices.is_empty() {
                indices.push(
                    ui.painter()
                        .with_clip_rect(primitive.clip_rect.intersect(self.rect))
                        .add(Shape::mesh(mesh)),
                );
            }
        }
        indices
    }
}

/// The old page is opaque where this mask is one, clear where it is zero.
/// A narrow feather is split geometrically, never stretched across a large
/// page/background triangle. UVs and premultiplied colours interpolate too.
struct Mask {
    rect: Rect,
    kind: Kind,
    progress: f32,
}
impl Mask {
    fn new(rect: Rect, kind: Kind, progress: f32) -> Self {
        Self {
            rect,
            kind,
            progress: progress.clamp(0.0, 1.0),
        }
    }
    fn is_circle(&self) -> bool {
        matches!(
            self.kind,
            Kind::CircleOut | Kind::CircleIn | Kind::CornerCircle | Kind::Spot(_)
        )
    }
    fn circle_sides(&self) -> usize {
        // Corner and off-centre reveals travel up to twice as far as centre
        // circles. Keep their feather just as smooth without increasing
        // existing circle costs.
        if matches!(self.kind, Kind::CornerCircle | Kind::Spot(_)) {
            CIRCLE_SIDES * 2
        } else {
            CIRCLE_SIDES
        }
    }
    fn origin(&self) -> Pos2 {
        match self.kind {
            Kind::CornerCircle => self.rect.min,
            // A point outside the page, or an invalid one, opens from the
            // nearest place on it.
            Kind::Spot(point) if point.x.is_finite() && point.y.is_finite() => {
                point.clamp(self.rect.min, self.rect.max)
            }
            _ => self.rect.center(),
        }
    }
    fn radius(&self) -> f32 {
        let reach = match self.kind {
            Kind::CornerCircle => self.rect.size().length(),
            Kind::Spot(_) => {
                let origin = self.origin();
                [
                    self.rect.left_top(),
                    self.rect.right_top(),
                    self.rect.left_bottom(),
                    self.rect.right_bottom(),
                ]
                .into_iter()
                .map(|corner| corner.distance(origin))
                .fold(0.0, f32::max)
            }
            _ => self.rect.size().length() * 0.5,
        } + EDGE * 2.0;
        reach
            * if self.kind == Kind::CircleIn {
                1.0 - self.progress
            } else {
                self.progress
            }
    }
    /// Half the width of the fade along the moving edge. Circles keep their
    /// fine edge; every straight, angled or wave edge is soft.
    fn edge(&self) -> f32 {
        match self.kind {
            Kind::CircleOut | Kind::CircleIn | Kind::CornerCircle | Kind::Spot(_) => EDGE,
            Kind::Clock => CLOCK_EDGE,
            _ => SOFT_EDGE,
        }
    }
    fn horizontal_boundary(&self) -> f32 {
        let edge = self.edge();
        let travel = (self.rect.width() + edge * 4.0) * self.progress;
        if self.kind == Kind::SwipeLeft {
            self.rect.right() + edge * 2.0 - travel
        } else {
            self.rect.left() - edge * 2.0 + travel
        }
    }
    fn vertical_boundary(&self) -> f32 {
        let edge = self.edge();
        let travel = (self.rect.height() + edge * 4.0) * self.progress;
        if self.kind == Kind::SwipeUp {
            self.rect.bottom() + edge * 2.0 - travel
        } else {
            self.rect.top() - edge * 2.0 + travel
        }
    }
    fn diamond_radius(&self) -> f32 {
        ((self.rect.width() + self.rect.height()) * 0.5 * std::f32::consts::FRAC_1_SQRT_2
            + self.edge() * 2.0)
            * self.progress
    }
    /// The length of the diagonal sweep's (1, 0.65) direction.
    fn diagonal_length() -> f32 {
        (1.0_f32 + 0.65 * 0.65).sqrt()
    }
    fn boundary(&self) -> f32 {
        // Measured along the unnormalised (1, 0.65) direction, so the margin
        // that keeps the fade off the page at either end is scaled to match.
        let margin = self.edge() * 2.0 * Self::diagonal_length();
        (self.rect.width() + self.rect.height() * 0.65 + margin * 2.0) * self.progress - margin
    }
    /// The wave's edge as connected straight pieces across the page, each
    /// with its own feathered plane, the strip of the page it covers, and a
    /// bounding box. The edge rolls sideways and swells as it travels, like
    /// surf running up a beach, but never moves back over revealed page.
    fn wave_parts(&self, wipe: Wipe) -> Vec<(Plane, Vec<Plane>, Rect)> {
        use std::f32::consts::{PI, TAU};
        let edge = self.edge();
        let (along, across) = wipe.axes();
        let corners = [
            self.rect.left_top(),
            self.rect.right_top(),
            self.rect.left_bottom(),
            self.rect.right_bottom(),
        ]
        .map(Pos2::to_vec2);
        let range = |axis: Vec2| {
            corners
                .iter()
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), c| {
                    (lo.min(c.dot(axis)), hi.max(c.dot(axis)))
                })
        };
        let (start, end) = range(along);
        let (from, to) = range(across);
        let span = (end - start).max(1.0);
        let width = (to - from).max(1.0);
        // Gentle on a small page; the full swell on a normal one. The swell
        // is small enough against the travel that the edge only ever moves
        // forwards.
        let main = (span * 0.05).min(WAVE_HEIGHT);
        let ripple = main * 0.4;
        let frequency = TAU / WAVE_LENGTH;
        let steepest = (main + ripple * 2.3) * frequency;
        let margin = main + ripple + edge * 2.0 * (1.0 + steepest);
        let progress = self.progress;
        let swell = 0.4 + 0.6 * (PI * progress).sin();
        let roll = 1.5 * PI * progress;
        let front = start - margin + (span + margin * 2.0) * progress;
        let height = |u: f32| {
            front
                + swell
                    * (main * (frequency * u + roll).sin()
                        + ripple * (2.3 * frequency * u - 1.7 * roll + 1.1).sin())
        };
        let pieces = ((width / WAVE_STEP).ceil() as usize).clamp(16, 200);
        (0..pieces)
            .map(|index| {
                let u0 = from + width * index as f32 / pieces as f32;
                let u1 = from + width * (index + 1) as f32 / pieces as f32;
                let (b0, b1) = (height(u0), height(u1));
                let slope = (b1 - b0) / (u1 - u0);
                let base = b0 - slope * u0;
                // The old page stays where it is further along than the edge.
                // Distances are measured along the direction of travel, so a
                // point only ever goes from covered to revealed as it moves.
                let plane = Plane {
                    normal: -(along - across * slope),
                    offset: -base + edge,
                };
                let first = index == 0;
                let last = index + 1 == pieces;
                let mut partition = Vec::with_capacity(2);
                if !first {
                    partition.push(Plane {
                        normal: -across,
                        offset: -u0,
                    });
                }
                if !last {
                    partition.push(Plane {
                        normal: across,
                        offset: u1,
                    });
                }
                let lo = if first { f32::NEG_INFINITY } else { u0 };
                let hi = if last { f32::INFINITY } else { u1 };
                let bounds = if across == Vec2::Y {
                    Rect::from_min_max(
                        Pos2::new(f32::NEG_INFINITY, lo),
                        Pos2::new(f32::INFINITY, hi),
                    )
                } else {
                    Rect::from_min_max(
                        Pos2::new(lo, f32::NEG_INFINITY),
                        Pos2::new(hi, f32::INFINITY),
                    )
                };
                (plane, partition, bounds)
            })
            .collect()
    }
    /// The independent straight wipes of the strip styles; empty otherwise.
    fn strips(&self) -> Vec<Strip> {
        let rect = self.rect;
        let edge = self.edge();
        let rows = |count: usize, wipe: &dyn Fn(usize) -> Wipe| {
            (0..count)
                .map(|index| {
                    let top = rect.top() + rect.height() * index as f32 / count as f32;
                    let bottom = rect.top() + rect.height() * (index + 1) as f32 / count as f32;
                    Strip {
                        rect: Rect::from_min_max(
                            Pos2::new(rect.left(), top),
                            Pos2::new(rect.right(), bottom),
                        ),
                        region: Rect::from_min_max(
                            Pos2::new(
                                f32::NEG_INFINITY,
                                if index == 0 { f32::NEG_INFINITY } else { top },
                            ),
                            Pos2::new(
                                f32::INFINITY,
                                if index + 1 == count {
                                    f32::INFINITY
                                } else {
                                    bottom
                                },
                            ),
                        ),
                        wipe: wipe(index),
                        progress: self.progress,
                        edge,
                    }
                })
                .collect::<Vec<_>>()
        };
        match self.kind {
            Kind::Blinds => rows(BLIND_ROWS, &|_| Wipe::Down),
            Kind::Zipper => rows(ZIPPER_ROWS, &|index| {
                if index % 2 == 0 {
                    Wipe::Right
                } else {
                    Wipe::Left
                }
            }),
            Kind::Letterbox => rows(2, &|index| if index == 0 { Wipe::Up } else { Wipe::Down }),
            Kind::Cascade => (0..CASCADE_COLUMNS)
                .map(|index| {
                    let count = CASCADE_COLUMNS as f32;
                    let left = rect.left() + rect.width() * index as f32 / count;
                    let right = rect.left() + rect.width() * (index + 1) as f32 / count;
                    // Every column starts closed and ends open; later columns
                    // start later and all of them finish with the reveal.
                    let delay = CASCADE_STAGGER * index as f32 / (count - 1.0);
                    Strip {
                        rect: Rect::from_min_max(
                            Pos2::new(left, rect.top()),
                            Pos2::new(right, rect.bottom()),
                        ),
                        region: Rect::from_min_max(
                            Pos2::new(
                                if index == 0 { f32::NEG_INFINITY } else { left },
                                f32::NEG_INFINITY,
                            ),
                            Pos2::new(
                                if index + 1 == CASCADE_COLUMNS {
                                    f32::INFINITY
                                } else {
                                    right
                                },
                                f32::INFINITY,
                            ),
                        ),
                        wipe: Wipe::Down,
                        progress: (self.progress * (1.0 + CASCADE_STAGGER) - delay).clamp(0.0, 1.0),
                        edge,
                    }
                })
                .collect(),
            _ => Vec::new(),
        }
    }
    /// Where the chevron's tip is along the horizontal midline.
    fn chevron_boundary(&self) -> f32 {
        let margin = self.edge() * 2.0 * (1.0 + CHEVRON_SLOPE * CHEVRON_SLOPE).sqrt();
        self.rect.left() - margin
            + (self.rect.width() + CHEVRON_SLOPE * self.rect.height() * 0.5 + margin * 2.0)
                * self.progress
    }
    /// The quarter turn the clock hand is in, counted clockwise from the top.
    fn clock_quadrant(&self) -> usize {
        ((self.progress * 4.0).floor() as usize).min(3)
    }
    fn alpha(&self, p: Pos2) -> f32 {
        if self.progress <= 0.0 {
            return 1.0;
        }
        if self.progress >= 1.0 {
            return 0.0;
        }
        match self.kind {
            Kind::CircleOut | Kind::CornerCircle | Kind::Spot(_) => {
                ((p.distance(self.origin()) - self.radius() + EDGE) / (EDGE * 2.0)).clamp(0.0, 1.0)
            }
            Kind::CircleIn => ((self.radius() - p.distance(self.rect.center()) + EDGE)
                / (EDGE * 2.0))
                .clamp(0.0, 1.0),
            // Every other style is its bands: the first whose part of the page
            // holds the point decides, exactly as the mesh is cut. A point in
            // no band is in a part already fully revealed.
            _ => {
                let edge = self.edge();
                self.bands()
                    .into_iter()
                    .find(|(_, partition)| partition.iter().all(|plane| plane.distance(p) <= 0.0))
                    .map_or(0.0, |(plane, _)| plane_alpha(plane, p, edge))
            }
        }
    }
    fn circle(&self, radius: f32) -> Vec<Plane> {
        let center = self.origin().to_vec2();
        (0..self.circle_sides())
            .map(|n| {
                let angle = n as f32 * std::f32::consts::TAU / self.circle_sides() as f32;
                let normal = Vec2::new(angle.cos(), angle.sin());
                Plane {
                    normal,
                    offset: normal.dot(center) + radius.max(0.0),
                }
            })
            .collect()
    }
    fn ring_sectors(&self) -> Vec<(Rect, [Plane; 4])> {
        let radius = self.radius();
        let inner = (radius - EDGE).max(0.0);
        let outer = (radius + EDGE) / (std::f32::consts::PI / self.circle_sides() as f32).cos();
        (0..self.circle_sides())
            .map(|n| {
                let a = (n as f32 - 0.5) * std::f32::consts::TAU / self.circle_sides() as f32;
                let b = (n as f32 + 0.5) * std::f32::consts::TAU / self.circle_sides() as f32;
                let a = Vec2::new(a.cos(), a.sin());
                let b = Vec2::new(b.cos(), b.sin());
                let center = self.origin();
                let corners = [
                    center + a * inner,
                    center + a * outer,
                    center + b * outer,
                    center + b * inner,
                ];
                let planes = std::array::from_fn(|i| {
                    let edge = corners[(i + 1) % corners.len()] - corners[i];
                    let normal = Vec2::new(edge.y, -edge.x);
                    Plane {
                        normal,
                        offset: normal.dot(corners[i].to_vec2()),
                    }
                });
                (Rect::from_points(&corners), planes)
            })
            .collect()
    }
    fn bands(&self) -> Vec<(Plane, Vec<Plane>)> {
        let edge = self.edge();
        match self.kind {
            Kind::Diagonal => {
                // A unit normal, so the soft edge is as wide as the others.
                let length = Self::diagonal_length();
                vec![(
                    Plane {
                        normal: Vec2::new(-1.0, -0.65) / length,
                        offset: -(self.rect.left() + self.rect.top() * 0.65 + self.boundary())
                            / length
                            + edge,
                    },
                    Vec::new(),
                )]
            }
            Kind::Wave(wipe) => self
                .wave_parts(wipe)
                .into_iter()
                .map(|(plane, partition, _)| (plane, partition))
                .collect(),
            Kind::Curtain => {
                let center = self.rect.center().x;
                let half = (self.rect.width() * 0.5 + edge * 2.0) * self.progress;
                vec![
                    (
                        Plane {
                            normal: Vec2::X,
                            offset: center - half + edge,
                        },
                        vec![Plane {
                            normal: Vec2::X,
                            offset: center,
                        }],
                    ),
                    (
                        Plane {
                            normal: -Vec2::X,
                            offset: -(center + half) + edge,
                        },
                        vec![Plane {
                            normal: -Vec2::X,
                            offset: -center,
                        }],
                    ),
                ]
            }
            Kind::SwipeLeft => vec![(
                Plane {
                    normal: Vec2::X,
                    offset: self.horizontal_boundary() + edge,
                },
                Vec::new(),
            )],
            Kind::SwipeRight => vec![(
                Plane {
                    normal: -Vec2::X,
                    offset: -self.horizontal_boundary() + edge,
                },
                Vec::new(),
            )],
            Kind::SwipeUp => vec![(
                Plane {
                    normal: Vec2::Y,
                    offset: self.vertical_boundary() + edge,
                },
                Vec::new(),
            )],
            Kind::SwipeDown => vec![(
                Plane {
                    normal: -Vec2::Y,
                    offset: -self.vertical_boundary() + edge,
                },
                Vec::new(),
            )],
            Kind::Diamond => {
                let center = self.rect.center().to_vec2();
                let radius = self.diamond_radius();
                [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
                    .into_iter()
                    .map(|(x, y)| {
                        let normal = Vec2::new(x, y) * std::f32::consts::FRAC_1_SQRT_2;
                        (
                            Plane {
                                normal: -normal,
                                offset: -normal.dot(center) - radius + edge,
                            },
                            vec![
                                Plane {
                                    normal: Vec2::new(-x, 0.0),
                                    offset: -x * center.x,
                                },
                                Plane {
                                    normal: Vec2::new(0.0, -y),
                                    offset: -y * center.y,
                                },
                            ],
                        )
                    })
                    .collect()
            }
            Kind::Blinds | Kind::Cascade | Kind::Zipper | Kind::Letterbox => self
                .strips()
                .iter()
                .map(|strip| (strip.plane(), strip.partition()))
                .collect(),
            Kind::Chevron => {
                let length = (1.0 + CHEVRON_SLOPE * CHEVRON_SLOPE).sqrt();
                let middle = self.rect.center().y;
                let boundary = self.chevron_boundary();
                vec![
                    (
                        Plane {
                            normal: Vec2::new(-1.0, CHEVRON_SLOPE) / length,
                            offset: -(boundary - CHEVRON_SLOPE * middle) / length + edge,
                        },
                        vec![Plane {
                            normal: Vec2::Y,
                            offset: middle,
                        }],
                    ),
                    (
                        Plane {
                            normal: Vec2::new(-1.0, -CHEVRON_SLOPE) / length,
                            offset: -(boundary + CHEVRON_SLOPE * middle) / length + edge,
                        },
                        vec![Plane {
                            normal: -Vec2::Y,
                            offset: -middle,
                        }],
                    ),
                ]
            }
            Kind::Clock => {
                let centre = self.rect.center().to_vec2();
                let active = self.clock_quadrant();
                (active..4)
                    .map(|quadrant| {
                        let start = clockwise(quadrant as f32 * std::f32::consts::FRAC_PI_2);
                        let end = clockwise((quadrant + 1) as f32 * std::f32::consts::FRAC_PI_2);
                        let after = Vec2::new(start.y, -start.x);
                        let before = Vec2::new(-end.y, end.x);
                        let partition = vec![
                            Plane {
                                normal: after,
                                offset: after.dot(centre),
                            },
                            Plane {
                                normal: before,
                                offset: before.dot(centre),
                            },
                        ];
                        let plane = if quadrant == active {
                            let hand = clockwise(std::f32::consts::TAU * self.progress);
                            let normal = Vec2::new(hand.y, -hand.x);
                            Plane {
                                normal,
                                offset: normal.dot(centre) + edge,
                            }
                        } else {
                            // The hand has not reached this quarter: all of
                            // it still shows the old page.
                            Plane {
                                normal: Vec2::ZERO,
                                offset: 1.0e9,
                            }
                        };
                        (plane, partition)
                    })
                    .collect()
            }
            Kind::CircleOut | Kind::CircleIn | Kind::CornerCircle | Kind::Spot(_) => Vec::new(),
        }
    }
    /// A conservative box around each band's partition, in the order of
    /// [`Self::bands`], so a triangle skips the bands it cannot touch.
    fn band_bounds(&self) -> Vec<Rect> {
        let centre = self.rect.center();
        match self.kind {
            Kind::Blinds | Kind::Cascade | Kind::Zipper | Kind::Letterbox => {
                self.strips().iter().map(|strip| strip.region).collect()
            }
            Kind::Wave(wipe) => self
                .wave_parts(wipe)
                .into_iter()
                .map(|(_, _, bounds)| bounds)
                .collect(),
            Kind::Chevron => vec![
                Rect::from_min_max(
                    Pos2::new(f32::NEG_INFINITY, f32::NEG_INFINITY),
                    Pos2::new(f32::INFINITY, centre.y),
                ),
                Rect::from_min_max(
                    Pos2::new(f32::NEG_INFINITY, centre.y),
                    Pos2::new(f32::INFINITY, f32::INFINITY),
                ),
            ],
            Kind::Clock => (self.clock_quadrant()..4)
                .map(|quadrant| {
                    let right = quadrant < 2;
                    let top = quadrant == 0 || quadrant == 3;
                    Rect::from_min_max(
                        Pos2::new(
                            if right { centre.x } else { f32::NEG_INFINITY },
                            if top { f32::NEG_INFINITY } else { centre.y },
                        ),
                        Pos2::new(
                            if right { f32::INFINITY } else { centre.x },
                            if top { centre.y } else { f32::INFINITY },
                        ),
                    )
                })
                .collect(),
            _ => vec![Rect::EVERYTHING; self.bands().len()],
        }
    }
    fn underlay(&self, source: &Mesh) -> Mesh {
        if self.progress <= 0.0 {
            return Mesh::with_texture(source.texture_id);
        }
        if self.progress >= 1.0 {
            return source.clone();
        }
        let mut output = Mesh::with_texture(source.texture_id);
        let circle = self.is_circle();
        // A whole extra feather width keeps all partially transparent edge
        // pixels backed by the original new page, with no double attenuation.
        let radius = if self.kind == Kind::CircleIn {
            (self.radius() - EDGE * 2.0).max(0.0)
                * (std::f32::consts::PI / self.circle_sides() as f32).cos()
        } else {
            self.radius() + EDGE * 2.0
        };
        let planes = if circle {
            self.circle(radius)
        } else {
            Vec::new()
        };
        let bands = self.bands();
        let band_bounds = self.band_bounds();
        let edge = self.edge();
        for triangle in source.indices.as_chunks::<3>().0 {
            let points = triangle.map(|index| source.vertices[index as usize]);
            let bounds = Rect::from_points(&points.map(|vertex| vertex.pos));
            if !bounds.intersects(self.rect) {
                continue;
            }
            if circle {
                let center = self.origin();
                let outside = center.clamp(bounds.min, bounds.max).distance(center)
                    > radius / (std::f32::consts::PI / self.circle_sides() as f32).cos();
                let inside = points
                    .iter()
                    .all(|vertex| vertex.pos.distance(center) <= radius);
                if outside || inside {
                    if (self.kind == Kind::CircleIn && outside)
                        || (self.kind != Kind::CircleIn && inside)
                    {
                        emit(&mut output, &points);
                    }
                    continue;
                }
            }
            let polygons = if circle {
                if self.kind == Kind::CircleIn {
                    difference(points.to_vec(), &planes)
                } else {
                    vec![intersection(points.to_vec(), &planes)]
                }
            } else {
                let mut remaining = vec![points.to_vec()];
                for ((plane, partition), band) in bands.iter().zip(&band_bounds) {
                    if !band.intersects(bounds) {
                        continue;
                    }
                    let mut opaque = partition.clone();
                    opaque.push(Plane {
                        offset: plane.offset - edge * 3.0,
                        ..*plane
                    });
                    remaining = remaining
                        .into_iter()
                        .flat_map(|polygon| difference(polygon, &opaque))
                        .collect();
                }
                remaining
            };
            for polygon in polygons {
                emit(&mut output, &polygon);
            }
        }
        compact_mesh(output)
    }

    fn mesh(&self, source: &Mesh) -> Mesh {
        if self.progress <= 0.0 {
            return source.clone();
        }
        if self.progress >= 1.0 {
            return Mesh::with_texture(source.texture_id);
        }
        let mut output = Mesh::with_texture(source.texture_id);
        // Build circle planes once per mesh, not for every glyph triangle.
        let circle = self.is_circle();
        let radius = self.radius();
        // Inscribe the solid/empty inner edge in the true circle so the
        // feather starts at zero alpha without a polygon-shaped seam.
        let inner = if circle {
            self.circle(
                (radius - EDGE).max(0.0)
                    * (std::f32::consts::PI / self.circle_sides() as f32).cos(),
            )
        } else {
            Vec::new()
        };
        let outer = if circle {
            self.circle(radius + EDGE)
        } else {
            Vec::new()
        };
        let sectors = if circle {
            self.ring_sectors()
        } else {
            Vec::new()
        };
        let bands = self.bands();
        let band_bounds = self.band_bounds();
        let edge = self.edge();
        for triangle in source.indices.as_chunks::<3>().0 {
            let points = [
                source.vertices[triangle[0] as usize],
                source.vertices[triangle[1] as usize],
                source.vertices[triangle[2] as usize],
            ];
            let bounds = Rect::from_points(&points.map(|v| v.pos));
            if !bounds.intersects(self.rect) {
                continue;
            }
            if circle {
                let center = self.origin();
                let closest = center.clamp(bounds.min, bounds.max);
                let outside = closest.distance(center)
                    > (radius + EDGE) / ((std::f32::consts::PI / self.circle_sides() as f32).cos());
                let inside = points
                    .iter()
                    .all(|v| v.pos.distance(center) <= (radius - EDGE).max(0.0));
                if outside || inside {
                    let visible = if self.kind != Kind::CircleIn {
                        outside
                    } else {
                        inside
                    };
                    if visible {
                        emit(&mut output, &points);
                    }
                    continue;
                }
                if self.kind != Kind::CircleIn {
                    for polygon in difference(points.to_vec(), &outer) {
                        emit(&mut output, &polygon);
                    }
                } else {
                    let solid = intersection(points.to_vec(), &inner);
                    emit(&mut output, &solid);
                }
                // Small angular sectors keep the curved feather accurate.
                // A large crescent polygon would interpolate alpha across
                // long chords and produce a visibly thick, uneven edge.
                for (sector_bounds, planes) in &sectors {
                    if !sector_bounds.intersects(bounds) {
                        continue;
                    }
                    let mut polygon = intersection(points.to_vec(), planes);
                    for vertex in &mut polygon {
                        vertex.color = vertex.color.gamma_multiply(self.alpha(vertex.pos));
                    }
                    emit(&mut output, &polygon);
                }
            } else {
                for ((plane, partition), band) in bands.iter().zip(&band_bounds) {
                    if !band.intersects(bounds) {
                        continue;
                    }
                    let plane = *plane;
                    let points = intersection(points.to_vec(), partition);
                    // Split at both ends of the feather, retaining exact texture UVs.
                    let solid = clip(
                        points.clone(),
                        Plane {
                            offset: plane.offset - edge * 2.0,
                            ..plane
                        },
                        true,
                    );
                    emit(&mut output, &solid);
                    let feather = clip(
                        clip(points, plane, true),
                        Plane {
                            offset: plane.offset - edge * 2.0,
                            ..plane
                        },
                        false,
                    );
                    let feather = feather
                        .into_iter()
                        .map(|mut v| {
                            v.color = v.color.gamma_multiply(plane_alpha(plane, v.pos, edge));
                            v
                        })
                        .collect::<Vec<_>>();
                    emit(&mut output, &feather);
                }
            }
        }
        compact_mesh(output)
    }
}

pub(super) fn revealed(rect: Rect, kind: Kind, progress: f32, point: Pos2) -> bool {
    Mask::new(rect, kind, progress).alpha(point) < 0.5
}

fn interpolate(a: Vertex, b: Vertex, t: f32) -> Vertex {
    let t = t.clamp(0.0, 1.0);
    let channels: [u8; 4] = std::array::from_fn(|i| {
        (a.color.to_array()[i] as f32
            + (b.color.to_array()[i] as f32 - a.color.to_array()[i] as f32) * t)
            .round() as u8
    });
    Vertex {
        pos: a.pos.lerp(b.pos, t),
        uv: a.uv.lerp(b.uv, t),
        color: Color32::from_rgba_premultiplied(channels[0], channels[1], channels[2], channels[3]),
    }
}
fn clip(polygon: Vec<Vertex>, plane: Plane, inside: bool) -> Vec<Vertex> {
    if polygon.is_empty() {
        return polygon;
    }
    let mut result = Vec::with_capacity(polygon.len() + 1);
    let mut previous = *polygon.last().unwrap();
    let mut previous_distance = plane.distance(previous.pos);
    let mut previous_in = if inside {
        previous_distance <= 0.0
    } else {
        previous_distance >= 0.0
    };
    for current in polygon {
        let distance = plane.distance(current.pos);
        let current_in = if inside {
            distance <= 0.0
        } else {
            distance >= 0.0
        };
        if current_in != previous_in {
            result.push(interpolate(
                previous,
                current,
                previous_distance / (previous_distance - distance),
            ));
        }
        if current_in {
            result.push(current);
        }
        previous = current;
        previous_distance = distance;
        previous_in = current_in;
    }
    result
}
fn intersection(mut polygon: Vec<Vertex>, planes: &[Plane]) -> Vec<Vertex> {
    for &plane in planes {
        polygon = clip(polygon, plane, true);
        if polygon.is_empty() {
            break;
        }
    }
    polygon
}
fn difference(mut remaining: Vec<Vertex>, planes: &[Plane]) -> Vec<Vec<Vertex>> {
    let mut result = Vec::new();
    for &plane in planes {
        let outside = clip(remaining.clone(), plane, false);
        if outside.len() >= 3 {
            result.push(outside);
        }
        remaining = clip(remaining, plane, true);
        if remaining.len() < 3 {
            break;
        }
    }
    result
}
fn emit(mesh: &mut Mesh, polygon: &[Vertex]) {
    if polygon.len() < 3 {
        return;
    }
    let start = mesh.vertices.len() as u32;
    mesh.vertices.extend_from_slice(polygon);
    for n in 1..polygon.len() - 1 {
        mesh.add_triangle(start, start + n as u32, start + n as u32 + 1);
    }
}

/// Clipping adjacent triangles repeatedly otherwise retains three copies of
/// shared corners and zero-area fan triangles on every retarget. Re-index only
/// exactly identical vertices, preserving positions, colours, UVs and draw order.
fn compact_mesh(source: Mesh) -> Mesh {
    let mut mesh = Mesh::with_texture(source.texture_id);
    let mut vertices = HashMap::with_capacity(source.vertices.len());
    let mut remap = vec![None; source.vertices.len()];
    for triangle in source.indices.as_chunks::<3>().0 {
        let [a, b, c] = triangle.map(|index| source.vertices[index as usize]);
        let cross = f64::from(b.pos.x - a.pos.x) * f64::from(c.pos.y - a.pos.y)
            - f64::from(b.pos.y - a.pos.y) * f64::from(c.pos.x - a.pos.x);
        if cross == 0.0
            || [a, b, c]
                .iter()
                .all(|vertex| vertex.color == Color32::TRANSPARENT)
        {
            continue;
        }
        for &index in triangle {
            let mapped = *remap[index as usize].get_or_insert_with(|| {
                let vertex = source.vertices[index as usize];
                let bits = |value: f32| if value == 0.0 { 0 } else { value.to_bits() };
                let key = [
                    bits(vertex.pos.x),
                    bits(vertex.pos.y),
                    bits(vertex.uv.x),
                    bits(vertex.uv.y),
                    u32::from_le_bytes(vertex.color.to_array()),
                ];
                *vertices.entry(key).or_insert_with(|| {
                    let index = mesh.vertices.len() as u32;
                    mesh.vertices.push(vertex);
                    index
                })
            });
            mesh.indices.push(mapped);
        }
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    const KINDS: [Kind; 22] = [
        Kind::Wave(Wipe::Left),
        Kind::Wave(Wipe::Right),
        Kind::Wave(Wipe::Up),
        Kind::Wave(Wipe::Down),
        Kind::CircleOut,
        Kind::CircleIn,
        Kind::CornerCircle,
        Kind::Diagonal,
        Kind::Curtain,
        Kind::SwipeLeft,
        Kind::SwipeRight,
        Kind::SwipeUp,
        Kind::SwipeDown,
        Kind::Diamond,
        // A click inside the page, and one in the sidebar to its left.
        Kind::Spot(Pos2::new(150.0, 95.0)),
        Kind::Spot(Pos2::new(-400.0, 300.0)),
        Kind::Blinds,
        Kind::Cascade,
        Kind::Zipper,
        Kind::Letterbox,
        Kind::Chevron,
        Kind::Clock,
    ];

    fn page_mesh(rect: Rect) -> Mesh {
        let mut mesh = Mesh::with_texture(egui::TextureId::Managed(23));
        mesh.add_rect_with_uv(
            rect,
            Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
            Color32::from_rgba_premultiplied(180, 120, 60, 240),
        );
        mesh
    }

    fn sampled_alpha(mesh: &Mesh, point: Pos2) -> f32 {
        let mut result: f32 = 0.0;
        for triangle in mesh.indices.as_chunks::<3>().0 {
            let [a, b, c] = triangle.map(|index| mesh.vertices[index as usize]);
            let cross = |a: Vec2, b: Vec2| a.x * b.y - a.y * b.x;
            let area = cross(b.pos - a.pos, c.pos - a.pos);
            if area.abs() < 0.00001 {
                continue;
            }
            let first = cross(b.pos - point, c.pos - point) / area;
            let second = cross(c.pos - point, a.pos - point) / area;
            let third = 1.0 - first - second;
            if first >= -0.00001 && second >= -0.00001 && third >= -0.00001 {
                result = result.max(
                    first * f32::from(a.color.a())
                        + second * f32::from(b.color.a())
                        + third * f32::from(c.color.a()),
                );
            }
        }
        result / 240.0
    }

    #[test]
    fn masks_reveal_real_old_page_geometry_without_holes_or_texture_stretching() {
        let rect = Rect::from_min_size(egui::pos2(30.0, 50.0), egui::vec2(640.0, 480.0));
        let original = page_mesh(rect);
        for kind in KINDS {
            for progress in [0.1, 0.35, 0.5, 0.9] {
                let mask = Mask::new(rect, kind, progress);
                let mesh = mask.mesh(&original);
                assert_eq!(mesh.texture_id, original.texture_id);
                for vertex in &mesh.vertices {
                    let expected = (vertex.pos - rect.min) / rect.size();
                    assert!(
                        (vertex.uv.to_vec2() - expected).length() < 0.0001,
                        "{kind:?} preserves image/text UVs"
                    );
                    assert!(
                        vertex.color.r() <= 180
                            && vertex.color.g() <= 120
                            && vertex.color.b() <= 60
                            && vertex.color.a() <= 240
                    );
                }
                for y in 0..19 {
                    for x in 0..23 {
                        let point = rect.min
                            + rect.size()
                                * egui::vec2((x as f32 + 0.41) / 23.0, (y as f32 + 0.37) / 19.0);
                        let expected = mask.alpha(point);
                        let actual = sampled_alpha(&mesh, point);
                        assert!(
                            (expected - actual).abs() < 0.18,
                            "{kind:?} at {progress}: {point:?}, expected {expected}, got {actual}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn endpoints_keep_the_old_page_exactly_then_remove_it_completely() {
        let rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(640.0, 480.0));
        let original = page_mesh(rect);
        for kind in KINDS {
            for point in [rect.center(), rect.min, rect.max] {
                assert_eq!(Mask::new(rect, kind, 0.0).alpha(point), 1.0);
                assert_eq!(Mask::new(rect, kind, 1.0).alpha(point), 0.0);
            }
            assert_eq!(Mask::new(rect, kind, 0.0).mesh(&original), original);
            let finished = Mask::new(rect, kind, 1.0).mesh(&original);
            assert!(finished.indices.is_empty());
            assert!(finished.vertices.is_empty());
        }
    }

    #[test]
    fn circle_directions_and_sweeps_expose_the_expected_regions() {
        let rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(640.0, 480.0));
        let center = rect.center();
        let corner = rect.min + egui::vec2(4.0, 4.0);
        assert!(revealed(rect, Kind::CircleOut, 0.5, center));
        assert!(!revealed(rect, Kind::CircleOut, 0.5, corner));
        assert!(!revealed(rect, Kind::CircleIn, 0.5, center));
        assert!(revealed(rect, Kind::CircleIn, 0.5, corner));
        assert!(revealed(rect, Kind::Diagonal, 0.5, corner));
        assert!(!revealed(
            rect,
            Kind::Diagonal,
            0.5,
            rect.max - egui::vec2(4.0, 4.0)
        ));
        assert!(revealed(rect, Kind::Curtain, 0.5, center));
        assert!(!revealed(rect, Kind::Curtain, 0.5, corner));
    }

    #[test]
    fn corner_circle_feather_stays_smooth_at_wide_and_narrow_native_sizes() {
        for size in [
            egui::vec2(720.0, 900.0),
            egui::vec2(1280.0, 900.0),
            egui::vec2(1920.0, 1080.0),
        ] {
            let rect = Rect::from_min_size(egui::pos2(30.0, 50.0), size);
            for progress in [0.1, 0.35, 0.7, 0.9] {
                let mask = Mask::new(rect, Kind::CornerCircle, progress);
                assert_eq!(mask.circle_sides(), 192);
                assert_eq!(
                    Mask::new(rect, Kind::CircleOut, progress).circle_sides(),
                    96
                );
                let mesh = mask.mesh(&page_mesh(rect));
                for angle in [0.01_f32, 0.17, 0.39, 0.64, 0.91, 1.27, 1.55] {
                    let direction = Vec2::new(angle.cos(), angle.sin());
                    for edge_offset in [-0.7, 0.0, 0.7] {
                        let point = rect.min + direction * (mask.radius() + edge_offset);
                        if rect.contains(point) {
                            let expected = mask.alpha(point);
                            let actual = sampled_alpha(&mesh, point);
                            assert!(
                                (expected - actual).abs() < 0.18,
                                "corner circle at {size:?}, {progress}: expected {expected}, got {actual}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn directional_swipes_reveal_without_translating_texture_coordinates() {
        let rect = Rect::from_min_size(egui::pos2(30.0, 50.0), egui::vec2(640.0, 480.0));
        for (kind, first, last) in [
            (
                Kind::SwipeRight,
                rect.left_center() + Vec2::X * 30.0,
                rect.right_center() - Vec2::X * 30.0,
            ),
            (
                Kind::SwipeLeft,
                rect.right_center() - Vec2::X * 30.0,
                rect.left_center() + Vec2::X * 30.0,
            ),
            (
                Kind::SwipeDown,
                rect.center_top() + Vec2::Y * 30.0,
                rect.center_bottom() - Vec2::Y * 30.0,
            ),
            (
                Kind::SwipeUp,
                rect.center_bottom() - Vec2::Y * 30.0,
                rect.center_top() + Vec2::Y * 30.0,
            ),
        ] {
            let mask = Mask::new(rect, kind, 0.5);
            assert_eq!(mask.alpha(first), 0.0, "{kind:?} reveals its starting edge");
            assert_eq!(mask.alpha(last), 1.0, "{kind:?} retains its finishing edge");
            for progress in [0.0, 0.1, 0.5, 0.9, 1.0] {
                let mesh = Mask::new(rect, kind, progress).mesh(&page_mesh(rect));
                for vertex in &mesh.vertices {
                    assert!(
                        (vertex.uv.to_vec2() - (vertex.pos - rect.min) / rect.size()).length()
                            < 0.0001
                    );
                }
            }
        }
    }

    #[test]
    fn corner_circle_and_diamond_open_from_their_actual_origins() {
        let rect = Rect::from_min_size(egui::pos2(30.0, 50.0), egui::vec2(640.0, 480.0));
        let near = rect.min + Vec2::splat(4.0);
        let far = rect.max - Vec2::splat(4.0);
        assert!(revealed(rect, Kind::CornerCircle, 0.25, near));
        assert!(!revealed(rect, Kind::CornerCircle, 0.25, far));
        assert!(revealed(rect, Kind::Diamond, 0.5, rect.center()));
        for corner in [rect.min, rect.max, rect.left_bottom(), rect.right_top()] {
            assert!(!revealed(rect, Kind::Diamond, 0.5, corner));
        }
        // A diamond edge is halfway between opposite quadrant boundaries.
        let mask = Mask::new(rect, Kind::Diamond, 0.5);
        let edge = rect.center() + Vec2::X * mask.diamond_radius() * std::f32::consts::SQRT_2;
        assert!((mask.alpha(edge) - 0.5).abs() < 0.0001);
    }

    #[test]
    fn a_spot_opens_from_its_point_and_reaches_the_furthest_corner() {
        let rect = Rect::from_min_size(egui::pos2(30.0, 50.0), egui::vec2(640.0, 480.0));
        let click = rect.left_bottom() + egui::vec2(60.0, -40.0);
        assert!(revealed(rect, Kind::Spot(click), 0.2, click));
        assert!(!revealed(rect, Kind::Spot(click), 0.2, rect.center()));
        assert!(!revealed(
            rect,
            Kind::Spot(click),
            0.9,
            rect.right_top() - egui::vec2(2.0, -2.0)
        ));
        let mask = Mask::new(rect, Kind::Spot(click), 1.0);
        assert!(mask.radius() > click.distance(rect.right_top()));
        assert_eq!(mask.circle_sides(), 192);
        // A click beside the page opens from the nearest edge; an invalid
        // point opens from the centre like the plain circle.
        let outside = Mask::new(rect, Kind::Spot(egui::pos2(-200.0, 300.0)), 0.5);
        assert_eq!(outside.origin(), egui::pos2(rect.left(), 300.0));
        let invalid = Mask::new(rect, Kind::Spot(egui::pos2(f32::NAN, 1.0)), 0.5);
        assert_eq!(invalid.origin(), rect.center());
    }

    #[test]
    fn waves_travel_their_way_with_a_rolling_curved_soft_edge() {
        let rect = Rect::from_min_size(egui::pos2(30.0, 50.0), egui::vec2(900.0, 600.0));
        for (wipe, near, far) in [
            (
                Wipe::Right,
                rect.left_center() + Vec2::X * 40.0,
                rect.right_center() - Vec2::X * 40.0,
            ),
            (
                Wipe::Left,
                rect.right_center() - Vec2::X * 40.0,
                rect.left_center() + Vec2::X * 40.0,
            ),
            (
                Wipe::Down,
                rect.center_top() + Vec2::Y * 40.0,
                rect.center_bottom() - Vec2::Y * 40.0,
            ),
            (
                Wipe::Up,
                rect.center_bottom() - Vec2::Y * 40.0,
                rect.center_top() + Vec2::Y * 40.0,
            ),
        ] {
            let kind = Kind::Wave(wipe);
            assert!(
                revealed(rect, kind, 0.5, near),
                "{wipe:?} starts at its side"
            );
            assert!(!revealed(rect, kind, 0.5, far), "{wipe:?} has not finished");
            let mask = Mask::new(rect, kind, 0.5);
            let (along, across) = wipe.axes();
            // Find where the edge crosses several lines across the page:
            // a wave's edge is curved, not straight, and it is soft.
            let crossing = |u: f32| {
                let line = |s: f32| rect.center() + across * u + along * s;
                let mut s = -700.0;
                while mask.alpha(line(s)) < 0.5 && s < 700.0 {
                    s += 0.25;
                }
                s
            };
            let crossings: Vec<f32> = (-4..=4).map(|step| crossing(step as f32 * 45.0)).collect();
            let low = crossings.iter().copied().fold(f32::INFINITY, f32::min);
            let high = crossings.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            assert!(high - low > 12.0, "{wipe:?} has a wavy edge: {crossings:?}");
            let middle = crossing(0.0);
            let soft = |s: f32| mask.alpha(rect.center() + along * s);
            assert!(
                soft(middle - 4.0) > 0.0 && soft(middle + 4.0) < 1.0,
                "{wipe:?} fades over several points, not one"
            );
            // The swell rolls along the shore as the wave comes in.
            let later = Mask::new(rect, kind, 0.6);
            let shape = |m: &Mask, u: f32| {
                let mut s = -700.0;
                while m.alpha(rect.center() + across * u + along * s) < 0.5 && s < 700.0 {
                    s += 0.25;
                }
                s
            };
            let early_shape: Vec<f32> = (-4..=4)
                .map(|step| shape(&mask, step as f32 * 45.0) - shape(&mask, 0.0))
                .collect();
            let late_shape: Vec<f32> = (-4..=4)
                .map(|step| shape(&later, step as f32 * 45.0) - shape(&later, 0.0))
                .collect();
            assert_ne!(early_shape, late_shape, "{wipe:?} rolls as it travels");
        }
    }

    #[test]
    fn straight_and_angled_styles_have_a_soft_edge_and_circles_stay_fine() {
        let rect = Rect::from_min_size(egui::pos2(30.0, 50.0), egui::vec2(640.0, 480.0));
        for kind in KINDS {
            let mask = Mask::new(rect, kind, 0.5);
            let expected = match kind {
                Kind::CircleOut | Kind::CircleIn | Kind::CornerCircle | Kind::Spot(_) => EDGE,
                Kind::Clock => CLOCK_EDGE,
                _ => SOFT_EDGE,
            };
            assert_eq!(mask.edge(), expected, "{kind:?}");
        }
        // Across the diamond's edge the old page fades over the full soft
        // width rather than switching within a couple of points.
        let mask = Mask::new(rect, Kind::Diamond, 0.5);
        let edge = rect.center() + Vec2::X * mask.diamond_radius() * std::f32::consts::SQRT_2;
        let inside = mask.alpha(edge - Vec2::X * SOFT_EDGE * 0.7);
        let outside = mask.alpha(edge + Vec2::X * SOFT_EDGE * 0.7);
        assert!(inside > 0.05 && inside < 0.5, "{inside}");
        assert!(outside > 0.5 && outside < 0.95, "{outside}");
    }

    #[test]
    fn the_part_styles_open_where_and_when_they_should() {
        let rect = Rect::from_min_size(egui::pos2(30.0, 50.0), egui::vec2(640.0, 480.0));
        let slat = rect.height() / BLIND_ROWS as f32;
        // Every slat opens from its own top edge.
        for row in 0..BLIND_ROWS {
            let top = rect.top() + slat * row as f32;
            let x = rect.center().x;
            assert!(revealed(rect, Kind::Blinds, 0.5, egui::pos2(x, top + 4.0)));
            assert!(!revealed(
                rect,
                Kind::Blinds,
                0.5,
                egui::pos2(x, top + slat - 4.0)
            ));
        }
        // Neighbouring zipper bands open from opposite sides.
        let band = rect.height() / ZIPPER_ROWS as f32;
        let (left, right) = (rect.left() + 20.0, rect.right() - 20.0);
        let first = rect.top() + band * 0.5;
        let second = rect.top() + band * 1.5;
        assert!(revealed(rect, Kind::Zipper, 0.3, egui::pos2(left, first)));
        assert!(!revealed(rect, Kind::Zipper, 0.3, egui::pos2(right, first)));
        assert!(revealed(rect, Kind::Zipper, 0.3, egui::pos2(right, second)));
        assert!(!revealed(rect, Kind::Zipper, 0.3, egui::pos2(left, second)));
        // The first cascade column leads the last, and all finish together.
        let near_top = rect.top() + 60.0;
        assert!(revealed(
            rect,
            Kind::Cascade,
            0.3,
            egui::pos2(left, near_top)
        ));
        assert!(!revealed(
            rect,
            Kind::Cascade,
            0.3,
            egui::pos2(right, near_top)
        ));
        assert!(revealed(
            rect,
            Kind::Cascade,
            0.99,
            egui::pos2(right, rect.bottom() - 30.0)
        ));
        // The letterbox opens from the middle; the chevron leads with its tip.
        assert!(revealed(rect, Kind::Letterbox, 0.4, rect.center()));
        assert!(!revealed(
            rect,
            Kind::Letterbox,
            0.4,
            rect.center_top() + Vec2::Y * 8.0
        ));
        assert!(!revealed(
            rect,
            Kind::Letterbox,
            0.4,
            rect.center_bottom() - Vec2::Y * 8.0
        ));
        let tip_x = rect.left() + rect.width() * 0.45;
        assert!(revealed(
            rect,
            Kind::Chevron,
            0.5,
            egui::pos2(tip_x, rect.center().y)
        ));
        assert!(!revealed(
            rect,
            Kind::Chevron,
            0.5,
            egui::pos2(tip_x, rect.top() + 6.0)
        ));
        // The clock hand starts at twelve and sweeps clockwise.
        let centre = rect.center();
        let at = |x: f32, y: f32| centre + egui::vec2(x, y);
        assert!(revealed(rect, Kind::Clock, 0.2, at(100.0, -100.0)));
        assert!(!revealed(rect, Kind::Clock, 0.2, at(100.0, 100.0)));
        assert!(revealed(rect, Kind::Clock, 0.45, at(100.0, 100.0)));
        assert!(!revealed(rect, Kind::Clock, 0.45, at(-100.0, 100.0)));
        assert!(revealed(rect, Kind::Clock, 0.7, at(-100.0, 100.0)));
        assert!(!revealed(rect, Kind::Clock, 0.7, at(-100.0, -100.0)));
        assert!(revealed(rect, Kind::Clock, 0.95, at(-100.0, -60.0)));
    }

    #[test]
    fn every_mask_progressively_exposes_the_page_and_has_exact_endpoints() {
        let rect = Rect::from_min_size(egui::pos2(30.0, 50.0), egui::vec2(640.0, 480.0));
        for kind in KINDS {
            for y in 0..7 {
                for x in 0..9 {
                    let point = rect.min
                        + rect.size()
                            * egui::vec2((x as f32 + 0.37) / 9.0, (y as f32 + 0.41) / 7.0);
                    let mut previous = 1.0;
                    for step in 0..=20 {
                        let alpha = Mask::new(rect, kind, step as f32 / 20.0).alpha(point);
                        assert!(
                            alpha <= previous + 0.0001,
                            "{kind:?} must not re-cover an exposed point"
                        );
                        previous = alpha;
                    }
                    assert_eq!(Mask::new(rect, kind, 0.0).alpha(point), 1.0);
                    assert_eq!(Mask::new(rect, kind, 1.0).alpha(point), 0.0);
                }
            }
        }
    }

    #[test]
    fn clipping_interpolates_premultiplied_colour_and_uv_at_new_vertices() {
        let a = Vertex {
            pos: egui::pos2(0.0, 0.0),
            uv: egui::pos2(0.0, 0.0),
            color: Color32::from_rgba_premultiplied(20, 40, 60, 100),
        };
        let b = Vertex {
            pos: egui::pos2(8.0, 4.0),
            uv: egui::pos2(1.0, 0.5),
            color: Color32::from_rgba_premultiplied(100, 120, 140, 200),
        };
        let middle = interpolate(a, b, 0.25);
        assert_eq!(middle.pos, egui::pos2(2.0, 1.0));
        assert_eq!(middle.uv, egui::pos2(0.25, 0.125));
        assert_eq!(middle.color.to_array(), [40, 60, 80, 125]);
    }

    #[test]
    fn exact_mesh_compaction_preserves_additive_colours_uvs_and_overdraw_order() {
        let mut source = Mesh::default();
        source.add_rect_with_uv(
            Rect::from_min_size(Pos2::ZERO, egui::vec2(40.0, 30.0)),
            Rect::from_min_max(egui::pos2(0.2, 0.3), egui::pos2(0.4, 0.7)),
            Color32::from_rgb_additive(120, 80, 30),
        );
        let expected = source
            .indices
            .iter()
            .map(|&index| source.vertices[index as usize])
            .collect::<Vec<_>>();
        source.vertices.extend(source.vertices.clone());
        source
            .indices
            .extend(source.indices.clone().into_iter().map(|index| index + 4));
        // Degenerate and wholly invisible triangles may disappear. Repeated
        // visible triangles must remain: their overdraw affects compositing.
        source.indices.extend([0, 0, 1]);
        source.add_colored_rect(
            Rect::from_min_size(egui::pos2(50.0, 50.0), egui::vec2(10.0, 10.0)),
            Color32::TRANSPARENT,
        );
        let compact = compact_mesh(source);
        assert_eq!(compact.vertices.len(), 4);
        assert_eq!(compact.indices.len(), 12);
        let actual = compact
            .indices
            .iter()
            .map(|&index| compact.vertices[index as usize])
            .collect::<Vec<_>>();
        assert_eq!(actual, expected.repeat(2));
    }

    #[test]
    fn pruning_hidden_underlay_preserves_an_opaque_composite_including_large_circle_edges() {
        for size in [egui::vec2(640.0, 480.0), egui::vec2(3840.0, 2160.0)] {
            let rect = Rect::from_min_size(egui::pos2(17.0, 31.0), size);
            let mut source = page_mesh(rect);
            for vertex in &mut source.vertices {
                vertex.color = Color32::WHITE;
            }
            for kind in KINDS {
                for progress in [0.001, 0.02, 0.3, 0.6, 0.95, 0.999] {
                    let mask = Mask::new(rect, kind, progress);
                    let old = mask.mesh(&source);
                    let new = mask.underlay(&source);
                    let mut points = (0..12)
                        .flat_map(|y| {
                            (0..16).map(move |x| {
                                rect.min
                                    + egui::vec2(
                                        (x as f32 + 0.37) * size.x / 16.0,
                                        (y as f32 + 0.61) * size.y / 12.0,
                                    )
                            })
                        })
                        .collect::<Vec<_>>();
                    if mask.is_circle() {
                        for angle in 0..mask.circle_sides() * 2 {
                            let angle =
                                angle as f32 * std::f32::consts::PI / mask.circle_sides() as f32;
                            for edge in [-2.0, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0] {
                                let point = mask.origin()
                                    + egui::vec2(angle.cos(), angle.sin())
                                        * (mask.radius() + EDGE * edge);
                                if rect.shrink(0.01).contains(point) {
                                    points.push(point);
                                }
                            }
                        }
                    }
                    for point in points {
                        // sampled_alpha's existing helper normalizes by its
                        // usual 240-alpha fixture; these canvases are 255.
                        let old = (sampled_alpha(&old, point) * 240.0 / 255.0).clamp(0.0, 1.0);
                        let new = (sampled_alpha(&new, point) * 240.0 / 255.0).clamp(0.0, 1.0);
                        assert!(
                            old + new * (1.0 - old) > 0.995,
                            "{kind:?}, {size:?}, {progress}, {point:?}: old {old}, new {new}"
                        );
                        if old < 0.99 {
                            assert!(
                                new > 0.995,
                                "partially transparent feathers retain the complete underlay"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn composite_font_meshes_keep_their_glyph_pixels_after_real_atlas_growth() {
        let ctx = egui::Context::default();
        let key = egui::Id::new("composite");
        let scene = egui::Id::new("font composite scene");
        let mut snapshot = None;
        let mut original = None;
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.label("Raw new text");
            let glyphs = ui.painter().layout_no_wrap(
                "Cached old text".to_owned(),
                egui::FontId::proportional(18.0),
                Color32::WHITE,
            );
            let shapes = vec![ClippedShape {
                clip_rect: ui.clip_rect(),
                shape: Shape::galley(ui.cursor().min, glyphs, Color32::WHITE),
            }];
            let primitives = ctx.tessellate(shapes, ui.pixels_per_point());
            let Primitive::Mesh(mesh) = &primitives[0].primitive else {
                unreachable!()
            };
            original = Some((mesh.clone(), ctx.fonts(|fonts| fonts.font_image_size())));
            ui.painter().add(Shape::mesh(mesh.clone()));
            snapshot = capture(ui, key, scene);
        });
        output.textures_delta.clear();
        let snapshot = snapshot.unwrap();
        assert!(
            snapshot
                .shapes
                .iter()
                .any(|shape| matches!(shape.shape, Shape::Text(_)))
        );
        assert!(
            snapshot
                .shapes
                .iter()
                .any(|shape| matches!(shape.shape, Shape::Mesh(_)))
        );
        let (original, old_size) = original.unwrap();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.label(
                egui::RichText::new(
                    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
                )
                .size(160.0),
            );
            let size = ctx.fonts(|fonts| fonts.font_image_size());
            assert_ne!(old_size, size, "the fixture really grows the atlas");
            let picture = snapshot.picture(ui, key, scene).unwrap();
            assert!(picture.valid(ui, scene));
            let cached = picture
                .meshes
                .iter()
                .find_map(|primitive| {
                    let Primitive::Mesh(mesh) = &primitive.primitive else {
                        return None;
                    };
                    mesh.vertices
                        .windows(original.vertices.len())
                        .find(|vertices| {
                            vertices
                                .iter()
                                .zip(&original.vertices)
                                .all(|(a, b)| a.pos == b.pos)
                        })
                })
                .expect("the cached glyph mesh is retained beside the re-tessellated raw text");
            for (cached, old) in cached.iter().zip(&original.vertices) {
                assert!(
                    (cached.uv.x * size[0] as f32 - old.uv.x * old_size[0] as f32).abs() < 0.001
                );
                assert!(
                    (cached.uv.y * size[1] as f32 - old.uv.y * old_size[1] as f32).abs() < 0.001
                );
            }
        });
        output.textures_delta.clear();
    }

    #[test]
    fn opacity_proof_rejects_rounding_blur_rotation_and_textured_rectangles() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let square = egui::epaint::RectShape::filled(ui.clip_rect(), 0, Color32::WHITE);
            for variant in 0..5 {
                let mut shape = square.clone();
                match variant {
                    1 => shape.corner_radius = 12.into(),
                    2 => shape.blur_width = 10.0,
                    3 => shape.angle = 0.2,
                    4 => {
                        shape.brush = Some(Arc::new(egui::epaint::Brush {
                            fill_texture_id: egui::TextureId::default(),
                            uv: Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
                        }))
                    }
                    _ => {}
                }
                let index = ui.painter().add(Shape::Rect(shape));
                let snapshot =
                    capture(ui, egui::Id::new("proof"), egui::Id::new("proof scene")).unwrap();
                assert_eq!(snapshot.opaque, variant == 0);
                ctx.graphics_mut(|graphics| {
                    graphics.get_mut(ui.layer_id()).unwrap().reset_shape(index)
                });
            }
        });
        output.textures_delta.clear();
    }

    #[test]
    fn snapshots_retain_image_textures_until_the_last_cached_picture_drops() {
        let ctx = egui::Context::default();
        let texture = ctx.load_texture(
            "temporary album",
            egui::ColorImage::filled([4, 4], Color32::RED),
            egui::TextureOptions::LINEAR,
        );
        let id = texture.id();
        let shapes = vec![ClippedShape {
            clip_rect: Rect::EVERYTHING,
            shape: Shape::image(
                id,
                Rect::from_min_size(Pos2::ZERO, egui::vec2(20.0, 20.0)),
                Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            ),
        }];
        let handles = retain_textures(&ctx, &shapes).unwrap();
        drop(texture);
        assert!(ctx.tex_manager().read().meta(id).is_some());
        let second = handles.clone();
        drop(handles);
        assert!(ctx.tex_manager().read().meta(id).is_some());
        drop(second);
        assert!(ctx.tex_manager().read().meta(id).is_none());
    }

    #[test]
    fn cached_pictures_cancel_on_viewport_dpi_visuals_gradient_or_atlas_change() {
        let ctx = egui::Context::default();
        let mut snapshot = None;
        let mut picture = None;
        let old_key = egui::Id::new("old");
        let scene = egui::Id::new("red gradient");
        let frame = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(640.0, 480.0))),
            ..Default::default()
        };
        let mut output = ctx.run_ui(frame.clone(), |ui| {
            ui.label("Old playlist");
            snapshot = capture(ui, old_key, scene);
        });
        output.textures_delta.clear();
        let snapshot = snapshot.unwrap();
        let mut output = ctx.run_ui(frame, |ui| {
            ui.label("New playlist");
            assert!(snapshot.matches(ui, old_key, scene));
            assert!(!snapshot.matches(ui, old_key, egui::Id::new("blue gradient")));
            picture = snapshot.picture(ui, old_key, scene);
            let picture = picture.as_ref().unwrap();
            assert!(picture.valid(ui, scene));
            assert!(!picture.valid(ui, egui::Id::new("blue gradient")));
            let mut changed = picture.clone();
            changed.rect = changed.rect.shrink(1.0);
            assert!(!changed.valid(ui, scene));
            changed = picture.clone();
            changed.pixels_per_point += 0.5;
            assert!(!changed.valid(ui, scene));
            changed = picture.clone();
            changed.appearance.widgets.inactive.bg_fill = Color32::GREEN;
            assert!(!changed.valid(ui, scene));
            changed = picture.clone();
            changed.textures = vec![(egui::TextureId::Managed(0), [1, 1])].into();
            assert!(!changed.valid(ui, scene));
            let before = ui.ctx().fonts(|fonts| fonts.font_image_size());
            let uploaded = ui
                .ctx()
                .tex_manager()
                .read()
                .meta(egui::TextureId::default())
                .unwrap()
                .size;
            // Force genuine same-pass growth, before end_pass updates the
            // texture manager, to catch the one-frame corrupted-glyph case.
            ui.label(
                egui::RichText::new(
                    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
                )
                .size(160.0),
            );
            let after = ui.ctx().fonts(|fonts| fonts.font_image_size());
            assert_ne!(before, after, "the probe must actually grow the atlas");
            assert_eq!(
                ui.ctx()
                    .tex_manager()
                    .read()
                    .meta(egui::TextureId::default())
                    .unwrap()
                    .size,
                uploaded
            );
            assert!(
                !picture.valid(ui, scene),
                "same-pass growth cancels immediately"
            );
            let rebuilt = snapshot.picture(ui, old_key, scene).unwrap();
            assert!(
                rebuilt.valid(ui, scene),
                "a newly tessellated picture uses the current atlas size"
            );
        });
        output.textures_delta.clear();
        assert!(picture.is_some());
    }
}
