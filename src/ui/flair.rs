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

/// The style a launch shows: the one chosen, or for Random one picked
/// afresh each launch.
fn launch_style(chosen: StartupStyle) -> StartupStyle {
    if chosen != StartupStyle::Random {
        return chosen;
    }
    static PICKED: std::sync::OnceLock<StartupStyle> = std::sync::OnceLock::new();
    *PICKED.get_or_init(|| {
        let styles = StartupStyle::PLAYED;
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.subsec_nanos() as usize);
        styles[seed % styles.len()]
    })
}

/// The logo reveal when the app opens, in the chosen style, then the window
/// fading in from behind it. Under a second, once per launch.
pub(crate) fn startup(app: &App, ctx: &Context) {
    if !starting() {
        return;
    }
    if !app.settings.startup_animation || !app.flair || crate::motion::reduced(ctx) {
        STARTUP_SHOWN.store(true, Ordering::Relaxed);
        return;
    }
    let time = ctx.input(|input| input.time);
    let start = ctx
        .data_mut(|data| *data.get_temp_mut_or_insert_with(Id::new("startup-reveal-at"), || time));
    let age = time - start;
    if age >= STARTUP_SECONDS {
        STARTUP_SHOWN.store(true, Ordering::Relaxed);
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
    match launch_style(app.settings.startup_style) {
        StartupStyle::Bloom | StartupStyle::Random => reveal.bloom(),
        StartupStyle::Iris => reveal.iris(),
        StartupStyle::Spin => reveal.spin(),
        StartupStyle::Curtain => reveal.curtain(),
        StartupStyle::Glow => reveal.glow(),
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
}

/// Eases from 0 to 1 gently at both ends.
fn smooth(amount: f32) -> f32 {
    let amount = amount.clamp(0.0, 1.0);
    amount * amount * (3.0 - 2.0 * amount)
}

/// Where to draw the playing cover `rect` this frame: drifted a little
/// towards the pointer when it is near, and swelling softly with the beat
/// (or a slow breath without the beat pulse) while the music plays.
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
    let beat = if app.settings.beat_pulse {
        app.beat.pulse(time)
    } else {
        0.0
    };
    let breath = 0.5 - 0.5 * (time as f32 * TAU / 4.5).cos();
    let scale = 1.0 + swell * (0.014 * breath + 0.03 * beat);
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
