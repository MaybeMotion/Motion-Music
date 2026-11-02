//! CPU geometry for short native page reveals. No renderer callbacks or screenshots.
use std::sync::Arc;

use egui::epaint::{ClippedPrimitive, ClippedShape, Mesh, Primitive, Vertex};
use egui::{Color32, Pos2, Rect, Shape, Vec2};

const CIRCLE_SIDES: usize = 96;
const MAX_SHAPES: usize = 6_000;
const MAX_VERTICES: usize = 180_000;
const MAX_INDICES: usize = 540_000;
const EDGE: f32 = 1.25;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone)]
pub(super) struct Snapshot {
    key: egui::Id,
    scene: egui::Id,
    rect: Rect,
    pixels_per_point: f32,
    appearance: egui::Visuals,
    shapes: Arc<[ClippedShape]>,
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

/// The current layer includes the page canvas painted before the body. Clip
/// that canvas and all content to the body viewport, excluding fixed chrome.
pub(super) fn capture(ui: &egui::Ui, key: egui::Id, scene: egui::Id) -> Option<Snapshot> {
    let rect = ui.clip_rect();
    if !rect.is_finite() || !rect.is_positive() {
        return None;
    }
    let shapes = ui.ctx().graphics(|g| {
        let list = g.get(ui.layer_id())?;
        let mut shapes = Vec::new();
        for entry in list.all_entries() {
            if entry.clip_rect.intersects(rect)
                && entry.shape.visual_bounding_rect().intersects(rect)
            {
                if shapes.len() == MAX_SHAPES {
                    return None;
                }
                shapes.push(ClippedShape {
                    clip_rect: entry.clip_rect.intersect(rect),
                    shape: entry.shape.clone(),
                });
            }
        }
        Some(shapes)
    })?;
    let textures = retain_textures(ui.ctx(), &shapes)?;
    Some(Snapshot {
        key,
        scene,
        rect,
        pixels_per_point: ui.pixels_per_point(),
        appearance: ui.visuals().clone(),
        shapes: shapes.into(),
        textures,
    })
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
        let meshes = ui
            .ctx()
            .tessellate(self.shapes.to_vec(), self.pixels_per_point);
        let mut vertices = 0;
        let mut indices = 0;
        let mut textures = Vec::new();
        // The texture manager receives the atlas update at end_pass. During
        // layout the font image already has its new size, which tessellate
        // uses to normalize glyph UVs.
        let font_size = ui.ctx().fonts(|fonts| fonts.font_image_size());
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
            textures: textures.into(),
            _handles: self.textures.clone(),
        })
    }
}

impl Picture {
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
            Kind::CircleOut | Kind::CircleIn | Kind::CornerCircle
        )
    }
    fn circle_sides(&self) -> usize {
        // Corner reveals travel twice as far as centre circles. Keep their
        // feather just as smooth without increasing existing circle costs.
        if self.kind == Kind::CornerCircle {
            CIRCLE_SIDES * 2
        } else {
            CIRCLE_SIDES
        }
    }
    fn origin(&self) -> Pos2 {
        if self.kind == Kind::CornerCircle {
            self.rect.min
        } else {
            self.rect.center()
        }
    }
    fn radius(&self) -> f32 {
        let reach = self.rect.size().length()
            * if self.kind == Kind::CornerCircle {
                1.0
            } else {
                0.5
            }
            + EDGE * 2.0;
        reach
            * if self.kind == Kind::CircleIn {
                1.0 - self.progress
            } else {
                self.progress
            }
    }
    fn horizontal_boundary(&self) -> f32 {
        let travel = (self.rect.width() + EDGE * 4.0) * self.progress;
        if self.kind == Kind::SwipeLeft {
            self.rect.right() + EDGE * 2.0 - travel
        } else {
            self.rect.left() - EDGE * 2.0 + travel
        }
    }
    fn vertical_boundary(&self) -> f32 {
        let travel = (self.rect.height() + EDGE * 4.0) * self.progress;
        if self.kind == Kind::SwipeUp {
            self.rect.bottom() + EDGE * 2.0 - travel
        } else {
            self.rect.top() - EDGE * 2.0 + travel
        }
    }
    fn diamond_radius(&self) -> f32 {
        ((self.rect.width() + self.rect.height()) * 0.5 * std::f32::consts::FRAC_1_SQRT_2
            + EDGE * 2.0)
            * self.progress
    }
    fn projection(&self, p: Pos2) -> f32 {
        (p.x - self.rect.left()) + (p.y - self.rect.top()) * 0.65
    }
    fn boundary(&self) -> f32 {
        (self.rect.width() + self.rect.height() * 0.65 + EDGE * 4.0) * self.progress - EDGE * 2.0
    }
    fn alpha(&self, p: Pos2) -> f32 {
        if self.progress <= 0.0 {
            return 1.0;
        }
        if self.progress >= 1.0 {
            return 0.0;
        }
        match self.kind {
            Kind::CircleOut | Kind::CornerCircle => {
                ((p.distance(self.origin()) - self.radius() + EDGE) / (EDGE * 2.0)).clamp(0.0, 1.0)
            }
            Kind::CircleIn => ((self.radius() - p.distance(self.rect.center()) + EDGE)
                / (EDGE * 2.0))
                .clamp(0.0, 1.0),
            Kind::Diagonal => {
                ((self.projection(p) - self.boundary() + EDGE) / (EDGE * 2.0)).clamp(0.0, 1.0)
            }
            Kind::SwipeLeft => {
                ((self.horizontal_boundary() - p.x + EDGE) / (EDGE * 2.0)).clamp(0.0, 1.0)
            }
            Kind::SwipeRight => {
                ((p.x - self.horizontal_boundary() + EDGE) / (EDGE * 2.0)).clamp(0.0, 1.0)
            }
            Kind::SwipeUp => {
                ((self.vertical_boundary() - p.y + EDGE) / (EDGE * 2.0)).clamp(0.0, 1.0)
            }
            Kind::SwipeDown => {
                ((p.y - self.vertical_boundary() + EDGE) / (EDGE * 2.0)).clamp(0.0, 1.0)
            }
            Kind::Diamond => {
                let delta = p - self.rect.center();
                (((delta.x.abs() + delta.y.abs()) * std::f32::consts::FRAC_1_SQRT_2
                    - self.diamond_radius()
                    + EDGE)
                    / (EDGE * 2.0))
                    .clamp(0.0, 1.0)
            }
            Kind::Curtain => {
                let half = (self.rect.width() * 0.5 + EDGE * 2.0) * self.progress;
                (((p.x - self.rect.center().x).abs() - half + EDGE) / (EDGE * 2.0)).clamp(0.0, 1.0)
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
        match self.kind {
            Kind::Diagonal => vec![(
                Plane {
                    normal: Vec2::new(-1.0, -0.65),
                    offset: -(self.rect.left() + self.rect.top() * 0.65 + self.boundary()) + EDGE,
                },
                Vec::new(),
            )],
            Kind::Curtain => {
                let center = self.rect.center().x;
                let half = (self.rect.width() * 0.5 + EDGE * 2.0) * self.progress;
                vec![
                    (
                        Plane {
                            normal: Vec2::X,
                            offset: center - half + EDGE,
                        },
                        vec![Plane {
                            normal: Vec2::X,
                            offset: center,
                        }],
                    ),
                    (
                        Plane {
                            normal: -Vec2::X,
                            offset: -(center + half) + EDGE,
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
                    offset: self.horizontal_boundary() + EDGE,
                },
                Vec::new(),
            )],
            Kind::SwipeRight => vec![(
                Plane {
                    normal: -Vec2::X,
                    offset: -self.horizontal_boundary() + EDGE,
                },
                Vec::new(),
            )],
            Kind::SwipeUp => vec![(
                Plane {
                    normal: Vec2::Y,
                    offset: self.vertical_boundary() + EDGE,
                },
                Vec::new(),
            )],
            Kind::SwipeDown => vec![(
                Plane {
                    normal: -Vec2::Y,
                    offset: -self.vertical_boundary() + EDGE,
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
                                offset: -normal.dot(center) - radius + EDGE,
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
            Kind::CircleOut | Kind::CircleIn | Kind::CornerCircle => Vec::new(),
        }
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
                for (plane, partition) in &bands {
                    let plane = *plane;
                    let points = intersection(points.to_vec(), partition);
                    // Split at both ends of the feather, retaining exact texture UVs.
                    let solid = clip(
                        points.clone(),
                        Plane {
                            offset: plane.offset - EDGE * 2.0,
                            ..plane
                        },
                        true,
                    );
                    emit(&mut output, &solid);
                    let feather = clip(
                        clip(points, plane, true),
                        Plane {
                            offset: plane.offset - EDGE * 2.0,
                            ..plane
                        },
                        false,
                    );
                    let feather = feather
                        .into_iter()
                        .map(|mut v| {
                            v.color = v.color.gamma_multiply(self.alpha(v.pos));
                            v
                        })
                        .collect::<Vec<_>>();
                    emit(&mut output, &feather);
                }
            }
        }
        output
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

#[cfg(test)]
mod tests {
    use super::*;

    const KINDS: [Kind; 10] = [
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
