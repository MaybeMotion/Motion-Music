//! Little touches drawn over the window: the cover drifting and breathing,
//! a card when the song changes, the logo reveal when the app opens, and
//! the weekly recap. Each has its own switch in Settings > Extras, and
//! none moves while the system asks for less motion.

use std::f32::consts::TAU;

use egui::{Align2, Color32, Context, Id, LayerId, Order, Rect, Vec2, pos2, vec2};

use crate::app::App;
use crate::i18n::gettext;
use crate::model::{Action, Page};
use crate::theme::{self, Icon};

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

/// The song the card is showing, and when it started.
#[derive(Clone)]
struct SongCard {
    title: String,
    subtitle: String,
    art: Option<String>,
    started: f64,
}

/// How long the song card stays, sliding in and floating away.
const CARD_SECONDS: f64 = 4.5;
const CARD_IN: f64 = 0.45;
const CARD_OUT: f64 = 0.7;

/// A card with the new song's cover, title and artist that slides up beside
/// the player bar's cover when the song changes, then floats away.
pub(crate) fn song_card(app: &App, ctx: &Context) {
    let id = Id::new("song-card");
    let last_id = id.with("last");
    let now = app.now_playing().filter(|now| !now.resuming);
    let uri = now.as_ref().map(|now| now.uri.clone());
    let time = ctx.input(|input| input.time);
    let last: Option<Option<String>> = ctx.data(|data| data.get_temp(last_id));
    // The song already playing as the app opens is no change of song.
    if starting(ctx) {
        ctx.data_mut(|data| data.insert_temp(last_id, uri));
        return;
    }
    if last.as_ref() != Some(&uri) {
        ctx.data_mut(|data| data.insert_temp(last_id, uri.clone()));
        let wanted = app.settings.song_card
            && app.flair
            && !crate::motion::reduced(ctx)
            && app.lyrics_fullscreen.is_none()
            && last.is_some();
        match now.as_ref().filter(|_| wanted) {
            Some(now) => ctx.data_mut(|data| {
                data.insert_temp(
                    id,
                    SongCard {
                        title: now.title.clone(),
                        subtitle: now.subtitle.clone(),
                        art: now.art_small.clone().or_else(|| now.art_url.clone()),
                        started: time,
                    },
                );
            }),
            None => ctx.data_mut(|data| data.remove::<SongCard>(id)),
        }
    }
    let Some(card) = ctx.data(|data| data.get_temp::<SongCard>(id)) else {
        return;
    };
    let age = time - card.started;
    if !(0.0..CARD_SECONDS).contains(&age) {
        ctx.data_mut(|data| data.remove::<SongCard>(id));
        return;
    }
    ctx.request_repaint();
    let entering = smooth((age / CARD_IN) as f32);
    let leaving = smooth(((CARD_SECONDS - age) / CARD_OUT) as f32);
    let opacity = entering.min(leaving);
    let size = vec2(300.0, 76.0);
    let window = ctx.content_rect();
    let home = pos2(
        window.left() + 16.0,
        window.bottom() - theme::PLAYER_BAR_HEIGHT - 14.0 - size.y,
    );
    // Rises from the player bar, then floats on up as it fades.
    let offset = vec2(0.0, (1.0 - entering) * 26.0 - (1.0 - leaving) * 18.0);
    let palette = app.palette;
    let colours = super::player_bar::visualiser_palette(app, None);
    egui::Area::new(id.with("area"))
        .order(Order::Foreground)
        .fixed_pos(home + offset)
        .interactable(false)
        .show(ctx, |ui| {
            ui.multiply_opacity(opacity);
            let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
            let corner = super::click_fx::corner(ui, 14.0);
            let painter = ui.painter();
            painter.add(
                egui::epaint::Shadow {
                    offset: [0, 10],
                    blur: 28,
                    spread: 0,
                    color: Color32::from_black_alpha(if palette.dark { 130 } else { 50 }),
                }
                .as_shape(rect, corner),
            );
            painter.rect_filled(rect, corner, palette.panel);
            // A soft wash of the visualizer's colours from the left.
            let mut wash = egui::Mesh::default();
            for (index, x) in [rect.left(), rect.center().x, rect.right()]
                .into_iter()
                .enumerate()
            {
                let colour = super::player_bar::swept(&colours, index as f32 / 2.0)
                    .gamma_multiply(0.18 * (1.0 - index as f32 / 2.0));
                wash.colored_vertex(pos2(x, rect.top() + 6.0), colour);
                wash.colored_vertex(pos2(x, rect.bottom() - 6.0), colour);
            }
            for at in [0, 2] {
                wash.add_triangle(at, at + 1, at + 3);
                wash.add_triangle(at, at + 3, at + 2);
            }
            painter.add(egui::Shape::mesh(wash));
            let cover = Rect::from_min_size(rect.min + vec2(10.0, 10.0), Vec2::splat(56.0));
            super::widgets::paint_cover(
                ui,
                &palette,
                card.art.as_deref(),
                cover,
                super::click_fx::radius(ui, 8.0),
                Icon::Music,
                Some(app.backend.art()),
            );
            let left = cover.right() + 12.0;
            let width = rect.right() - left - 12.0;
            let lines = [
                (
                    gettext(app.locale, "Now playing").into_owned(),
                    theme::semibold(11.0),
                    super::player_bar::swept(&colours, 0.2),
                ),
                (card.title.clone(), theme::semibold(15.0), palette.text),
                (
                    card.subtitle.clone(),
                    theme::regular(12.5),
                    palette.secondary,
                ),
            ];
            let mut y = rect.top() + 12.0;
            for (text, font, colour) in lines {
                let mut job = egui::text::LayoutJob::simple_singleline(text, font, colour);
                job.wrap = egui::text::TextWrapping::truncate_at_width(width);
                let galley = ui.painter().layout_job(job);
                let height = galley.size().y;
                ui.painter().galley(pos2(left, y), galley, colour);
                y += height + 2.0;
            }
        });
}

/// How long the logo reveal lasts, and when it starts to fade.
const STARTUP_SECONDS: f64 = 1.9;
const STARTUP_FADE: f64 = 0.6;

/// Whether the logo reveal is still showing.
pub(crate) fn starting(ctx: &Context) -> bool {
    ctx.data(|data| data.get_temp::<bool>(Id::new("startup-reveal-done")))
        .is_none_or(|done| !done)
}

/// The logo blooming in the middle of the window in the theme's colours
/// when the app opens, then the window fading in from behind it.
pub(crate) fn startup(app: &App, ctx: &Context) {
    let done_id = Id::new("startup-reveal-done");
    if ctx.data(|data| data.get_temp::<bool>(done_id)) == Some(true) {
        return;
    }
    if !app.settings.startup_animation || !app.flair || crate::motion::reduced(ctx) {
        ctx.data_mut(|data| data.insert_temp(done_id, true));
        return;
    }
    let time = ctx.input(|input| input.time);
    let start = ctx
        .data_mut(|data| *data.get_temp_mut_or_insert_with(Id::new("startup-reveal-at"), || time));
    let age = time - start;
    if age >= STARTUP_SECONDS {
        ctx.data_mut(|data| data.insert_temp(done_id, true));
        return;
    }
    ctx.request_repaint();
    let cover = smooth(((STARTUP_SECONDS - age) / STARTUP_FADE) as f32);
    let window = ctx.content_rect();
    let painter = ctx.layer_painter(LayerId::new(Order::Tooltip, Id::new("startup-reveal")));
    let palette = app.palette;
    painter.rect_filled(window, 0.0, palette.window.gamma_multiply(cover));
    let colours = super::player_bar::visualiser_palette(app, None);
    let centre = window.center() - vec2(0.0, 18.0);
    // Rings of the theme's colours spreading out from the logo.
    for ring in 0..3 {
        let progress = ((age - ring as f64 * 0.18) / 1.3).clamp(0.0, 1.0) as f32;
        if progress <= 0.0 {
            continue;
        }
        let radius = 50.0 + progress * window.size().min_elem() * 0.32;
        let colour = super::player_bar::swept(&colours, ring as f32 / 2.0)
            .gamma_multiply((1.0 - progress) * 0.5 * cover);
        painter.circle_stroke(centre, radius, egui::Stroke::new(3.0 - ring as f32, colour));
    }
    // The logo grows in with a soft overshoot.
    let grow = smooth((age / 0.7) as f32);
    let overshoot = 1.0 + 0.08 * (grow * std::f32::consts::PI).sin();
    let diameter = 112.0 * (0.7 + 0.3 * grow) * overshoot;
    let glow = super::player_bar::swept(&colours, 0.5);
    for (spread, alpha) in [(1.6, 0.06), (1.3, 0.1), (1.12, 0.16)] {
        painter.circle_filled(
            centre,
            diameter / 2.0 * spread,
            glow.gamma_multiply(alpha * grow * cover),
        );
    }
    let texture = logo_texture(ctx);
    painter.image(
        texture.id(),
        Rect::from_center_size(centre, Vec2::splat(diameter)),
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Color32::WHITE.gamma_multiply(grow * cover),
    );
    let words = smooth(((age - 0.45) / 0.5) as f32) * cover;
    painter.text(
        centre + vec2(0.0, 112.0 / 2.0 + 30.0 - (1.0 - words) * 8.0),
        Align2::CENTER_CENTER,
        "Motion Music",
        theme::bold(26.0),
        palette.text.gamma_multiply(words),
    );
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
    if !app.settings.weekly_recap || !app.flair || starting(ctx) {
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
