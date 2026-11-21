//! Little touches drawn over the window: the cover drifting and breathing,
//! the logo reveal when the app opens, and the weekly recap. Each has its
//! own switch in Settings > Extras, and none moves while the system asks
//! for less motion.

use std::f32::consts::{PI, TAU};
use std::sync::atomic::{AtomicBool, Ordering};

use egui::{Align2, Color32, Context, Id, LayerId, Order, Rect, Vec2, pos2, vec2};

use crate::app::App;
use crate::i18n::gettext;
use crate::model::{Action, Page};
use crate::settings::StartupStyle;
use crate::theme;

/// How long the logo reveal lasts, so a quick launch stays quick, and how
/// much of it is the window fading in.
const STARTUP_SECONDS: f64 = 0.9;
const STARTUP_FADE: f64 = 0.35;

/// The reveal plays once per launch. Switching between the main window and
/// the mini player makes a new window, but it is the same launch.
static STARTUP_SHOWN: AtomicBool = AtomicBool::new(false);

/// Whether the logo reveal is still to finish.
pub(crate) fn starting() -> bool {
    !STARTUP_SHOWN.load(Ordering::Relaxed)
}

/// Any one style but Random, picked by the clock.
fn random_style() -> StartupStyle {
    let styles = StartupStyle::PLAYED;
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.subsec_nanos() as usize / 1000);
    styles[seed % styles.len()]
}

/// The style a launch shows: the one chosen, or for Random one picked
/// afresh each launch.
fn launch_style(chosen: StartupStyle) -> StartupStyle {
    if chosen != StartupStyle::Random {
        return chosen;
    }
    static PICKED: std::sync::OnceLock<StartupStyle> = std::sync::OnceLock::new();
    *PICKED.get_or_init(random_style)
}

/// The style being previewed from Settings, if one is.
fn preview_id() -> Id {
    Id::new("startup-reveal-preview")
}

/// Plays the logo reveal now in `style`, as Settings' Preview button does,
/// whether or not it plays at launch. Random previews a random one.
pub(crate) fn preview(ctx: &Context, style: StartupStyle) {
    let style = if style == StartupStyle::Random {
        random_style()
    } else {
        style
    };
    ctx.data_mut(|data| {
        data.insert_temp(preview_id(), style);
        data.remove::<f64>(Id::new("startup-reveal-at"));
    });
    STARTUP_SHOWN.store(false, Ordering::Relaxed);
    ctx.request_repaint();
}

/// The logo reveal when the app opens, in the chosen style, then the window
/// fading in from behind it. Under a second, once per launch.
pub(crate) fn startup(app: &App, ctx: &Context) {
    if !starting() {
        return;
    }
    let previewing = ctx.data(|data| data.get_temp::<StartupStyle>(preview_id()));
    let finish = || {
        STARTUP_SHOWN.store(true, Ordering::Relaxed);
        ctx.data_mut(|data| data.remove::<StartupStyle>(preview_id()));
    };
    let wanted = previewing.is_some() || (app.settings.startup_animation && app.flair);
    if !wanted || crate::motion::reduced(ctx) {
        finish();
        return;
    }
    let time = ctx.input(|input| input.time);
    let start = ctx
        .data_mut(|data| *data.get_temp_mut_or_insert_with(Id::new("startup-reveal-at"), || time));
    let age = time - start;
    if age >= STARTUP_SECONDS {
        finish();
        return;
    }
    ctx.request_repaint();
    let reveal = Reveal {
        // 0 to 1 through the whole reveal, and how much of the window is
        // still covered: all of it until the fade, then less and less.
        progress: (age / STARTUP_SECONDS) as f32,
        cover: smooth(((STARTUP_SECONDS - age) / STARTUP_FADE) as f32),
        window: ctx.content_rect(),
        colours: super::player_bar::visualiser_palette(app, None),
        background: app.palette.window,
        text: app.palette.text,
        painter: ctx.layer_painter(LayerId::new(Order::Tooltip, Id::new("startup-reveal"))),
        logo: logo_texture(ctx).id(),
    };
    match previewing.unwrap_or_else(|| launch_style(app.settings.startup_style)) {
        StartupStyle::Bloom | StartupStyle::Random => reveal.bloom(),
        StartupStyle::Iris => reveal.iris(),
        StartupStyle::Spin => reveal.spin(),
        StartupStyle::Curtain => reveal.curtain(),
        StartupStyle::Glow => reveal.glow(),
        StartupStyle::Sweep => reveal.sweep(),
        StartupStyle::Pulse => reveal.pulse(),
        StartupStyle::Zoom => reveal.zoom(),
        StartupStyle::Mosaic => reveal.mosaic(),
    }
}

/// One frame of the logo reveal.
struct Reveal {
    progress: f32,
    cover: f32,
    window: Rect,
    colours: Vec<Color32>,
    background: Color32,
    text: Color32,
    painter: egui::Painter,
    logo: egui::TextureId,
}

impl Reveal {
    const LOGO: f32 = 104.0;

    fn centre(&self) -> egui::Pos2 {
        self.window.center() - vec2(0.0, 16.0)
    }

    fn colour(&self, at: f32) -> Color32 {
        super::player_bar::swept(&self.colours, at)
    }

    /// The whole window in the theme's background, fading at the end.
    fn backdrop(&self) {
        self.painter
            .rect_filled(self.window, 0.0, self.background.gamma_multiply(self.cover));
    }

    /// The logo at `scale`, turned by `angle`, `shown` (0 to 1) visible.
    fn logo(&self, scale: f32, angle: f32, shown: f32) {
        let half = Self::LOGO * scale / 2.0;
        let centre = self.centre();
        let tint = Color32::WHITE.gamma_multiply(shown * self.cover);
        let mut mesh = egui::Mesh::with_texture(self.logo);
        let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
        for (x, y) in corners {
            let (sin, cos) = angle.sin_cos();
            let point = vec2(x * cos - y * sin, x * sin + y * cos) * half;
            mesh.vertices.push(egui::epaint::Vertex {
                pos: centre + point,
                uv: pos2((x + 1.0) / 2.0, (y + 1.0) / 2.0),
                color: tint,
            });
        }
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(0, 2, 3);
        self.painter.add(egui::Shape::mesh(mesh));
    }

    /// A soft halo of the theme's middle colour behind the logo.
    fn halo(&self, scale: f32, strength: f32) {
        let glow = self.colour(0.5);
        for (spread, alpha) in [(1.6, 0.06), (1.3, 0.1), (1.12, 0.16)] {
            self.painter.circle_filled(
                self.centre(),
                Self::LOGO * scale / 2.0 * spread,
                glow.gamma_multiply(alpha * strength * self.cover),
            );
        }
    }

    /// The name, rising softly into place under the logo.
    fn name(&self, shown: f32) {
        let shown = shown * self.cover;
        self.painter.text(
            self.centre() + vec2(0.0, Self::LOGO / 2.0 + 28.0 + (1.0 - shown) * 6.0),
            Align2::CENTER_CENTER,
            "Motion Music",
            theme::bold(24.0),
            self.text.gamma_multiply(shown),
        );
    }

    /// The logo grows in with a little overshoot as rings of the theme's
    /// colours spread out from it.
    fn bloom(&self) {
        self.backdrop();
        for ring in 0..3 {
            let progress = ((self.progress - ring as f32 * 0.12) / 0.75).clamp(0.0, 1.0);
            if progress <= 0.0 {
                continue;
            }
            let radius = 48.0 + progress * self.window.size().min_elem() * 0.34;
            let colour = self
                .colour(ring as f32 / 2.0)
                .gamma_multiply((1.0 - progress) * 0.55 * self.cover);
            self.painter.circle_stroke(
                self.centre(),
                radius,
                egui::Stroke::new(3.0 - ring as f32, colour),
            );
        }
        let grow = smooth(self.progress / 0.45);
        let scale = (0.7 + 0.3 * grow) * (1.0 + 0.08 * (grow * PI).sin());
        self.halo(scale, grow);
        self.logo(scale, 0.0, grow);
        self.name(smooth((self.progress - 0.25) / 0.3));
    }

    /// A circle opens from the middle, its rim lit in the theme's colours,
    /// showing the app through it while the logo shrinks away.
    fn iris(&self) {
        let open = smooth((self.progress - 0.3) / 0.7);
        let far = self.window.size().length() / 2.0;
        let radius = open * far;
        let centre = self.centre();
        // Everything outside the opening, drawn as a ring.
        let steps = 72;
        let mut ring = egui::Mesh::default();
        for step in 0..=steps {
            let angle = step as f32 / steps as f32 * TAU;
            let direction = vec2(angle.cos(), angle.sin());
            ring.colored_vertex(centre + direction * radius, self.background);
            ring.colored_vertex(centre + direction * (far + 4.0), self.background);
        }
        for step in 0..steps {
            let at = step * 2;
            ring.add_triangle(at, at + 1, at + 3);
            ring.add_triangle(at, at + 3, at + 2);
        }
        self.painter.add(egui::Shape::mesh(ring));
        if radius > 1.0 {
            for (index, width) in [6.0, 3.0].into_iter().enumerate() {
                let colour = self
                    .colour(index as f32)
                    .gamma_multiply(0.6 * (1.0 - open * 0.6));
                self.painter
                    .circle_stroke(centre, radius, egui::Stroke::new(width, colour));
            }
        }
        let shown = smooth(self.progress / 0.25) * (1.0 - open);
        self.halo(1.0, shown);
        self.logo(1.0 - 0.4 * open, 0.0, shown);
        self.name(shown);
    }

    /// The logo spins into place, a full turn slowing to a stop.
    fn spin(&self) {
        self.backdrop();
        let turn = smooth(self.progress / 0.6);
        let angle = (1.0 - turn) * -TAU;
        let scale = 0.5 + 0.5 * turn;
        // A comet of the theme's colours trails the turn.
        for trail in 0..6 {
            let behind = trail as f32 * 0.35;
            let point = self.centre()
                + vec2((angle + behind).cos(), (angle + behind).sin()) * Self::LOGO * 0.62;
            self.painter.circle_filled(
                point,
                5.0 - trail as f32 * 0.7,
                self.colour(trail as f32 / 5.0)
                    .gamma_multiply((1.0 - turn) * (1.0 - trail as f32 / 6.0) * self.cover),
            );
        }
        self.halo(scale, turn);
        self.logo(scale, angle, smooth(self.progress / 0.2));
        self.name(smooth((self.progress - 0.35) / 0.25));
    }

    /// The logo shows on a curtain that parts down the middle, sliding off
    /// either side.
    fn curtain(&self) {
        let part = smooth((self.progress - 0.35) / 0.65);
        let width = self.window.width() / 2.0;
        let slide = part * (width + 8.0);
        let left = Rect::from_min_size(
            self.window.min - vec2(slide, 0.0),
            vec2(width, self.window.height()),
        );
        let right = left.translate(vec2(width + 2.0 * slide, 0.0));
        for (half, at) in [(left, 0.0), (right, 1.0)] {
            self.painter.rect_filled(half, 0.0, self.background);
            // The parting edge lit in the theme's colours.
            let edge = if at == 0.0 {
                Rect::from_min_max(pos2(half.right() - 3.0, half.top()), half.right_bottom())
            } else {
                Rect::from_min_max(half.left_top(), pos2(half.left() + 3.0, half.bottom()))
            };
            self.painter.rect_filled(
                edge,
                0.0,
                self.colour(at).gamma_multiply(part.min(1.0 - part) * 1.6),
            );
        }
        let shown = smooth(self.progress / 0.25) * (1.0 - part);
        self.halo(1.0, shown);
        self.logo(0.85 + 0.15 * smooth(self.progress / 0.35), 0.0, shown);
        self.name(shown);
    }

    /// The quietest: the logo fades up in a soft glow of the theme's
    /// colours and the window comes in behind it.
    fn glow(&self) {
        self.backdrop();
        let shown = smooth(self.progress / 0.35);
        let breathe = 1.0 + 0.15 * (self.progress * PI).sin();
        self.halo(breathe * 1.15, shown);
        self.logo(0.92 + 0.08 * shown, 0.0, shown);
        self.name(smooth((self.progress - 0.2) / 0.3));
    }

    /// A slanted band of the theme's colours sweeps across the window,
    /// leaving the logo lit behind it.
    fn sweep(&self) {
        self.backdrop();
        let across = smooth(self.progress / 0.6);
        let width = self.window.width();
        let band = width * 0.35;
        let slant = self.window.height() * 0.35;
        // The band's middle runs from off the left to off the right.
        let middle = self.window.left() - band + across * (width + 2.0 * band);
        let mut mesh = egui::Mesh::default();
        let fade = (1.0 - across).min(across * 4.0).min(1.0) * self.cover;
        for (index, offset) in [-1.0, -0.4, 0.0, 0.4, 1.0].into_iter().enumerate() {
            let x = middle + offset * band / 2.0;
            let strength = (1.0 - f32::abs(offset)) * 0.55 * fade;
            let colour = self.colour(index as f32 / 4.0).gamma_multiply(strength);
            mesh.colored_vertex(pos2(x + slant, self.window.top()), colour);
            mesh.colored_vertex(pos2(x - slant, self.window.bottom()), colour);
        }
        for column in 0..4 {
            let at = column * 2;
            mesh.add_triangle(at, at + 1, at + 3);
            mesh.add_triangle(at, at + 3, at + 2);
        }
        self.painter.add(egui::Shape::mesh(mesh));
        // Lit once the band has passed the middle.
        let passed = smooth((middle - self.centre().x) / (band * 0.5) + 0.5);
        self.halo(1.0, passed);
        self.logo(0.9 + 0.1 * passed, 0.0, passed);
        self.name(smooth((self.progress - 0.4) / 0.25));
    }

    /// The logo beats twice like a heart, a ring bursting out on each beat.
    fn pulse(&self) {
        self.backdrop();
        let shown = smooth(self.progress / 0.15);
        let mut scale = 0.9;
        for (beat, at) in [0.18_f32, 0.42].into_iter().enumerate() {
            let since = self.progress - at;
            if since < 0.0 {
                continue;
            }
            // A quick swell and settle.
            scale += 0.16 * (-since * 14.0).exp() * smooth(since / 0.04);
            let ring = (since / 0.4).clamp(0.0, 1.0);
            if ring < 1.0 {
                self.painter.circle_stroke(
                    self.centre(),
                    Self::LOGO * 0.55 + ring * Self::LOGO * 1.6,
                    egui::Stroke::new(
                        3.0,
                        self.colour(beat as f32)
                            .gamma_multiply((1.0 - ring) * 0.7 * self.cover),
                    ),
                );
            }
        }
        self.halo(scale, shown);
        self.logo(scale, 0.0, shown);
        self.name(smooth((self.progress - 0.45) / 0.25));
    }

    /// The logo flies towards you and past, and the app is there behind it.
    fn zoom(&self) {
        let rush = ((self.progress - 0.3) / 0.7).clamp(0.0, 1.0);
        // The background goes early, as the logo comes forward.
        let backdrop = 1.0 - smooth(rush * 1.6);
        self.painter
            .rect_filled(self.window, 0.0, self.background.gamma_multiply(backdrop));
        let arrive = smooth(self.progress / 0.3);
        let scale = (0.6 + 0.4 * arrive) * (1.0 + rush * rush * 9.0);
        let shown = arrive * (1.0 - smooth(rush * 1.4));
        self.halo(scale, shown);
        self.logo(scale, 0.0, shown);
        self.name(arrive * (1.0 - smooth(rush * 3.0)));
    }

    /// The window is covered in tiles that tip away from the middle outward,
    /// each edged in the theme's colours as it goes.
    fn mosaic(&self) {
        let columns = 12;
        let size = self.window.width() / columns as f32;
        let rows = (self.window.height() / size).ceil() as usize;
        let reach = self.centre().distance(self.window.min).max(1.0);
        for row in 0..rows {
            for column in 0..columns {
                let tile = Rect::from_min_size(
                    self.window.min + vec2(column as f32 * size, row as f32 * size),
                    Vec2::splat(size + 0.5),
                );
                // Tiles near the middle go first.
                // The last, in the corners, are gone by 0.85 of the way.
                let delay = tile.center().distance(self.centre()) / reach * 0.3;
                let gone = smooth((self.progress - 0.3 - delay) / 0.25);
                if gone >= 1.0 {
                    continue;
                }
                let shrunk = Rect::from_center_size(tile.center(), tile.size() * (1.0 - gone));
                self.painter
                    .rect_filled(shrunk, 0.0, self.background.gamma_multiply(self.cover));
                if gone > 0.0 {
                    let edge = self
                        .colour(column as f32 / (columns - 1) as f32)
                        .gamma_multiply(0.6 * (1.0 - gone) * self.cover);
                    self.painter.rect_stroke(
                        shrunk,
                        2.0,
                        egui::Stroke::new(1.5, edge),
                        egui::StrokeKind::Inside,
                    );
                }
            }
        }
        let shown = smooth(self.progress / 0.2) * (1.0 - smooth((self.progress - 0.35) / 0.3));
        self.halo(1.0, shown);
        self.logo(1.0, 0.0, shown);
        self.name(shown);
    }
}

/// Eases from 0 to 1 gently at both ends.
fn smooth(amount: f32) -> f32 {
    let amount = amount.clamp(0.0, 1.0);
    amount * amount * (3.0 - 2.0 * amount)
}

/// Where to draw the playing cover `rect` this frame: drifted a little
/// towards the pointer when it is near, and breathing slowly while the
/// music plays.
pub(crate) fn moving_art(app: &App, ui: &egui::Ui, rect: Rect, key: &str, playing: bool) -> Rect {
    let ctx = ui.ctx();
    if !app.settings.moving_art || crate::motion::reduced(ctx) {
        return rect;
    }
    let time = ui.input(|input| input.time);
    let id = Id::new(("moving-art", key));
    // The motion eases in when the music starts and out when it stops.
    let swell =
        crate::motion::animate_value(ctx, id.with("swell"), f32::from(u8::from(playing)), 0.8);
    // A slow breath only: the cover does not jump with the beat.
    let breath = 0.5 - 0.5 * (time as f32 * TAU / 4.5).cos();
    let scale = 1.0 + swell * 0.014 * breath;
    // Towards the pointer, by at most a twentieth of the cover.
    let reach = rect.width() * 0.05;
    let toward = ui
        .input(|input| input.pointer.hover_pos())
        .filter(|pointer| rect.expand(rect.width() * 0.6).contains(*pointer))
        .map_or(Vec2::ZERO, |pointer| {
            let offset = (pointer - rect.center()) / (rect.width() * 0.8);
            vec2(offset.x.clamp(-1.0, 1.0), offset.y.clamp(-1.0, 1.0)) * reach
        });
    let x = crate::motion::animate_value(ctx, id.with("x"), toward.x, 0.35);
    let y = crate::motion::animate_value(ctx, id.with("y"), toward.y, 0.35);
    // The breath rides on frames the visualizers and beat pulse already
    // draw; it never keeps an otherwise still window drawing by itself.
    Rect::from_center_size(rect.center() + vec2(x, y), rect.size() * scale)
}

/// A soft round cloud of `colour` at `centre`, fading to nothing at
/// `radius`.
fn cloud(painter: &egui::Painter, centre: egui::Pos2, radius: f32, colour: Color32) {
    let steps = 40;
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(centre, colour);
    // A middle ring keeps the fall-off soft rather than a cone.
    for (ring, (reach, strength)) in [(0.5, 0.55), (1.0, 0.0)].into_iter().enumerate() {
        for step in 0..steps {
            let angle = step as f32 / steps as f32 * TAU;
            mesh.colored_vertex(
                centre + vec2(angle.cos(), angle.sin()) * radius * reach,
                colour.gamma_multiply(strength),
            );
        }
        let base = 1 + ring as u32 * steps;
        for step in 0..steps {
            let next = (step + 1) % steps;
            if ring == 0 {
                mesh.add_triangle(0, base + step, base + next);
            } else {
                let inner = base - steps;
                mesh.add_triangle(inner + step, base + step, base + next);
                mesh.add_triangle(inner + step, base + next, inner + next);
            }
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// Soft clouds of the visualizer's colours behind Home, at three depths:
/// the nearer ones move further with the pointer and with scrolling, so
/// the page has depth. Painted before the page's own content.
pub(crate) fn ambient(app: &App, ui: &egui::Ui) {
    let ctx = ui.ctx();
    if !app.settings.parallax || crate::motion::reduced(ctx) {
        return;
    }
    let view = ui.clip_rect();
    // How far the page has scrolled since its top.
    let scrolled = (view.top() - ui.cursor().top()).max(0.0);
    let toward = ui
        .input(|input| input.pointer.hover_pos())
        .filter(|pointer| view.contains(*pointer))
        .map_or(Vec2::ZERO, |pointer| {
            (pointer - view.center()) / view.size()
        });
    let x = crate::motion::animate_value(ctx, Id::new("ambient-x"), toward.x, 0.6);
    let y = crate::motion::animate_value(ctx, Id::new("ambient-y"), toward.y, 0.6);
    let time = ui.input(|input| input.time) as f32;
    let colours = super::player_bar::visualiser_palette(app, None);
    let strength = if app.palette.dark { 0.13 } else { 0.1 };
    let painter = ui.painter().with_clip_rect(view);
    // Where each sits, how big, and how near: nearer moves more.
    let clouds = [
        (0.15, 0.1, 0.5, 0.35),
        (0.8, 0.25, 0.42, 0.7),
        (0.5, 0.85, 0.55, 1.0),
    ];
    for (index, (at_x, at_y, size, depth)) in clouds.into_iter().enumerate() {
        // A slow drift of its own, which moves only on frames already drawn.
        let drift = vec2(
            (time * 0.07 + index as f32 * 2.1).sin(),
            (time * 0.05 + index as f32 * 1.3).cos(),
        ) * 14.0;
        let centre =
            view.min + vec2(at_x * view.width(), at_y * view.height()) + vec2(x, y) * 90.0 * depth
                - vec2(0.0, scrolled * 0.35 * depth)
                + drift;
        let radius = size * view.width().max(view.height());
        cloud(
            &painter,
            centre,
            radius,
            super::player_bar::swept(&colours, index as f32 / 2.0).gamma_multiply(strength),
        );
    }
}

/// The blurred cover behind an album or playlist header, scrolling at half
/// the page's speed and fading towards the page below it. Answers how much
/// to shrink and fade the cover itself as it scrolls away, 0 to 1.
pub(crate) fn parallax_header(
    app: &App,
    ui: &egui::Ui,
    texture: Option<&egui::TextureHandle>,
    height: f32,
) -> f32 {
    if !app.settings.parallax || crate::motion::reduced(ui.ctx()) {
        return 0.0;
    }
    let view = ui.clip_rect();
    let top = ui.cursor().top();
    let scrolled = (view.top() - top).max(0.0);
    let away = (scrolled / height).clamp(0.0, 1.0);
    let Some(texture) = texture else {
        return away;
    };
    // Twice as wide as tall a slice of the cover, filling the width.
    let area = Rect::from_min_size(
        pos2(ui.max_rect().left() - 32.0, top - 40.0 + scrolled * 0.5),
        vec2(ui.max_rect().width() + 64.0, height + 80.0),
    );
    let uv_height = (area.height() / area.width()).min(1.0);
    let uv = Rect::from_center_size(pos2(0.5, 0.5), vec2(1.0, uv_height));
    let strength = if app.palette.dark { 0.42 } else { 0.3 } * (1.0 - away * 0.6);
    let painter = ui.painter().with_clip_rect(view);
    let mut mesh = egui::Mesh::with_texture(texture.id());
    // Fully there at the top, gone by the foot, so the page shows through.
    for (row, fade) in [(0.0, 1.0), (0.55, 0.75), (1.0, 0.0)] {
        let y = area.top() + area.height() * row;
        let v = uv.top() + uv.height() * row;
        let tint = Color32::WHITE.gamma_multiply(strength * fade);
        mesh.vertices.push(egui::epaint::Vertex {
            pos: pos2(area.left(), y),
            uv: pos2(uv.left(), v),
            color: tint,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: pos2(area.right(), y),
            uv: pos2(uv.right(), v),
            color: tint,
        });
    }
    for row in 0..2 {
        let at = row * 2;
        mesh.add_triangle(at, at + 1, at + 3);
        mesh.add_triangle(at, at + 3, at + 2);
    }
    painter.add(egui::Shape::mesh(mesh));
    away
}

/// A soft band of light across a cover, inside its own corners, at
/// `along` (0 at the left to 1 at the right), `strength` 0 to 1.
pub(crate) fn sheen(
    painter: &egui::Painter,
    rect: Rect,
    corner: egui::CornerRadius,
    along: f32,
    strength: f32,
) {
    if strength <= 0.0 {
        return;
    }
    let band = rect.width() * 0.34;
    let middle = rect.left() - band / 2.0 + along.clamp(0.0, 1.0) * (rect.width() + band);
    let light = Color32::WHITE.gamma_multiply(0.32 * strength);
    let mut mesh = egui::Mesh::default();
    let rows = 12;
    for row in 0..=rows {
        let y = rect.top() + rect.height() * row as f32 / rows as f32;
        // Slanted, leaning right towards the top.
        let centre = middle + (rect.center().y - y) * 0.35;
        for (offset, colour) in [
            (-0.5, Color32::TRANSPARENT),
            (0.0, light),
            (0.5, Color32::TRANSPARENT),
        ] {
            let point = pos2(centre + offset * band, y);
            mesh.colored_vertex(super::click_fx::inside(rect, corner, point), colour);
        }
    }
    for row in 0..rows {
        let at = row * 3;
        for column in 0..2 {
            let a = at + column;
            mesh.add_triangle(a, a + 1, a + 4);
            mesh.add_triangle(a, a + 4, a + 3);
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// The app icon once, large enough to scale down smoothly.
fn logo_texture(ctx: &Context) -> egui::TextureHandle {
    let id = Id::new("startup-reveal-logo");
    if let Some(texture) = ctx.data(|data| data.get_temp::<egui::TextureHandle>(id)) {
        return texture;
    }
    let pixels = 256;
    let image = egui::ColorImage::from_rgba_unmultiplied(
        [pixels, pixels],
        &crate::util::app_icon_rgba(pixels),
    );
    let texture = ctx.load_texture("startup-reveal-logo", image, egui::TextureOptions::LINEAR);
    ctx.data_mut(|data| data.insert_temp(id, texture.clone()));
    texture
}

/// When a new week starts, a card with last week's listening: its numbers
/// counting up, its song and artist, and a way to My Wrapped. Shown once a
/// week, and only after a week with some music.
pub(crate) fn weekly_recap(app: &mut App, ctx: &Context) {
    if !app.settings.weekly_recap || !app.flair || starting() {
        return;
    }
    let this_week = crate::wrapped::week_id(jiff::Zoned::now().date());
    if app.settings.weekly_recap_seen.as_deref() == Some(this_week.as_str()) {
        return;
    }
    let summary = super::wrapped::summary(app, ctx);
    let (start, week) = &summary.week;
    if week.plays < 3 {
        // Nothing worth a recap; ask again next week.
        app.settings.weekly_recap_seen = Some(this_week);
        app.actions.push(Action::SettingsChanged);
        return;
    }
    let palette = app.palette;
    let locale = app.locale;
    let time = ctx.input(|input| input.time);
    let opened =
        ctx.data_mut(|data| *data.get_temp_mut_or_insert_with(Id::new("weekly-recap-at"), || time));
    let shown = smooth(((time - opened) / 0.35) as f32);
    if shown < 1.0 {
        ctx.request_repaint();
    }
    let window = ctx.content_rect();
    // Under the card, which is in the foreground, and over the page.
    ctx.layer_painter(LayerId::new(Order::Middle, Id::new("weekly-recap-dim")))
        .rect_filled(
            window,
            0.0,
            Color32::from_black_alpha((110.0 * shown) as u8),
        );
    let end = start
        .checked_add(jiff::Span::new().days(6))
        .unwrap_or(*start);
    let mut close = false;
    let mut open_wrapped = false;
    egui::Area::new(Id::new("weekly-recap"))
        .order(Order::Foreground)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, (1.0 - shown) * 24.0))
        .show(ctx, |ui| {
            ui.multiply_opacity(shown);
            egui::Frame::new()
                .fill(palette.panel)
                .corner_radius(super::click_fx::corner(ui, 18.0))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 16],
                    blur: 40,
                    spread: 0,
                    color: Color32::from_black_alpha(120),
                })
                .inner_margin(egui::Margin::same(24))
                .show(ui, |ui| {
                    ui.set_width(420.0_f32.min(window.width() - 64.0));
                    theme::text(
                        ui,
                        gettext(locale, "Your week in music").as_ref(),
                        theme::bold(26.0),
                        palette.text,
                    );
                    theme::subtle(
                        ui,
                        &palette,
                        &gettext(locale, "Last week, {first} to {last}")
                            .replace("{first}", &start.strftime("%-d %b").to_string())
                            .replace("{last}", &end.strftime("%-d %b").to_string()),
                    );
                    ui.add_space(14.0);
                    super::wrapped::week_recap(app, ui, week, "weekly-recap");
                    ui.add_space(18.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        if theme::pill_button(
                            ui,
                            &palette,
                            &gettext(locale, "See My Wrapped"),
                            true,
                        )
                        .clicked()
                        {
                            open_wrapped = true;
                        }
                        if theme::soft_button(ui, &palette, None, &gettext(locale, "Close"), false)
                            .clicked()
                        {
                            close = true;
                        }
                    });
                });
        });
    if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
        close = true;
    }
    if close || open_wrapped {
        app.settings.weekly_recap_seen = Some(this_week);
        app.actions.push(Action::SettingsChanged);
        ctx.data_mut(|data| data.remove::<f64>(Id::new("weekly-recap-at")));
        if open_wrapped {
            app.actions.push(Action::Open(Page::Wrapped));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chosen_style_plays_as_is_and_random_picks_one_for_the_whole_launch() {
        for style in StartupStyle::PLAYED {
            assert_eq!(launch_style(style), style);
        }
        let picked = launch_style(StartupStyle::Random);
        assert!(StartupStyle::PLAYED.contains(&picked));
        assert_eq!(
            launch_style(StartupStyle::Random),
            picked,
            "the same all launch"
        );
        // Quick enough that a fast launch stays fast.
        const { assert!(STARTUP_SECONDS <= 1.0) };
    }
}
