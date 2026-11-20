//! My Wrapped: big-number listening stats from this computer's plays, beside
//! Spotify's own top artists and songs.

use egui::{Color32, CornerRadius, Frame, Margin, Rect, Sense, Ui, pos2, vec2};

use super::widgets;
use crate::app::App;
use crate::i18n::gettext;
use crate::model::{Action, Page};
use crate::theme::{self, Icon};

const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

pub fn show(app: &mut App, ui: &mut Ui) {
    let palette = app.palette;
    let locale = app.locale;
    let zone = jiff::tz::TimeZone::system();
    let stats = crate::wrapped::stats(app.plays.plays(), &zone);

    ui.add_space(12.0);
    // Everything from the title to the listening calendar makes the share
    // picture, so its top is remembered here.
    let share_top = ui.cursor().top();
    let save_label = gettext(locale, "Save as image");
    ui.horizontal(|ui| {
        theme::text(
            ui,
            gettext(locale, "My Wrapped").as_ref(),
            theme::bold(40.0),
            palette.text,
        );
        if stats.plays > 0 {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if theme::soft_button(ui, &palette, Some(Icon::Sparkles), &save_label, false)
                    .on_hover_text(gettext(
                        locale,
                        "Save this summary as a picture in your Pictures folder, ready to share",
                    ))
                    .clicked()
                {
                    app.actions.push(Action::SaveWrappedImage);
                }
            });
        }
    });
    let range = match (stats.first_day, stats.last_day) {
        (Some(first), Some(last)) => gettext(
            locale,
            "From your plays on this computer, {first} to {last}",
        )
        .replace("{first}", &first.strftime("%-d %b %Y").to_string())
        .replace("{last}", &last.strftime("%-d %b %Y").to_string()),
        _ => gettext(
            locale,
            "Play some music here and your stats will appear. Spotify's own top lists are below.",
        )
        .into_owned(),
    };
    theme::subtle(ui, &palette, &range);
    ui.add_space(12.0);

    // The titles these habits earn.
    let earned = crate::wrapped::badges(&stats);
    if !earned.is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            for badge in &earned {
                Frame::new()
                    .fill(palette.surface)
                    .stroke(egui::Stroke::new(1.0, palette.accent.gamma_multiply(0.5)))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(Margin::symmetric(12, 6))
                    .show(ui, |ui| {
                        theme::text(
                            ui,
                            format!("{} {}", badge.emoji, gettext(locale, badge.title)),
                            theme::semibold(14.0),
                            palette.text,
                        )
                    })
                    .response
                    .on_hover_text(gettext(locale, badge.detail));
            }
        });
        ui.add_space(14.0);
    }

    // The headline numbers, as large cards.
    let cards = [
        (
            stats.minutes.to_string(),
            gettext(locale, "minutes listened"),
        ),
        (stats.plays.to_string(), gettext(locale, "plays")),
        (stats.songs.to_string(), gettext(locale, "different songs")),
        (
            stats.artists.to_string(),
            gettext(locale, "different artists"),
        ),
        (
            stats.longest_streak.to_string(),
            gettext(locale, "days in a row, your longest streak"),
        ),
    ];
    let gap = 12.0;
    let columns = ((ui.available_width() + gap) / (190.0 + gap))
        .floor()
        .clamp(1.0, 5.0) as usize;
    let width = (ui.available_width() - gap * (columns - 1) as f32) / columns as f32;
    for row in cards.chunks(columns) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (index, (number, label)) in row.iter().enumerate() {
                stat_card(app, ui, width, number, label, index);
            }
        });
        ui.add_space(gap);
    }

    // When you listen.
    if let (Some(hour), Some(day)) = (stats.favourite_hour(), stats.favourite_weekday()) {
        ui.add_space(8.0);
        heading(app, ui, &gettext(locale, "When you listen"));
        let hour_text = format!("{:02}:00 to {:02}:00", hour, (hour + 1) % 24);
        theme::subtle(
            ui,
            &palette,
            &gettext(
                locale,
                "Your favourite time is {hour}, and your busiest day is {day}.",
            )
            .replace("{hour}", &hour_text)
            .replace("{day}", &gettext(locale, WEEKDAYS[day])),
        );
        ui.add_space(8.0);
        listening_clock(app, ui, &stats.hours);
        ui.add_space(16.0);
        heading(app, ui, &gettext(locale, "Your listening calendar"));
        theme::subtle(
            ui,
            &palette,
            &gettext(
                locale,
                "The last twelve weeks, one square a day. Brighter is more music.",
            ),
        );
        ui.add_space(8.0);
        listening_calendar(app, ui, &stats.days);
        ui.add_space(16.0);
    }
    let share_bottom = ui.cursor().top();
    let share = Rect::from_min_max(
        pos2(ui.max_rect().left() - 8.0, share_top - 8.0),
        pos2(ui.max_rect().right() + 8.0, share_bottom),
    );
    ui.ctx()
        .data_mut(|data| data.insert_temp(egui::Id::new("wrapped-share-rect"), share));

    // A throwback to the song of the day a week ago.
    let today = jiff::Zoned::now().date();
    if let Some(song) = crate::wrapped::week_ago(app.plays.plays(), today, &zone) {
        heading(app, ui, &gettext(locale, "A week ago today"));
        let plays =
            gettext(locale, "{count} plays that day").replace("{count}", &song.plays.to_string());
        if ranked_row(
            app,
            ui,
            0,
            song.image.as_deref(),
            &song.name,
            &song.detail,
            &plays,
        ) {
            app.actions.push(Action::PlayUris {
                uris: vec![song.key.clone()],
                index: 0,
            });
        }
        ui.add_space(16.0);
    }

    // On repeat, from this computer's plays.
    if !stats.top_songs.is_empty() {
        heading(app, ui, &gettext(locale, "On repeat"));
        for (rank, song) in stats.top_songs.iter().enumerate() {
            let plays =
                gettext(locale, "{count} plays").replace("{count}", &song.plays.to_string());
            if ranked_row(
                app,
                ui,
                rank,
                song.image.as_deref(),
                &song.name,
                &song.detail,
                &plays,
            ) {
                app.actions.push(Action::PlayUris {
                    uris: vec![song.key.clone()],
                    index: 0,
                });
            }
        }
        ui.add_space(16.0);
    }
    if !stats.top_albums.is_empty() {
        heading(app, ui, &gettext(locale, "Albums you loved"));
        for (rank, album) in stats.top_albums.iter().take(3).enumerate() {
            let plays =
                gettext(locale, "{count} plays").replace("{count}", &album.plays.to_string());
            if ranked_row(
                app,
                ui,
                rank,
                album.image.as_deref(),
                &album.name,
                &album.detail,
                &plays,
            ) {
                app.actions
                    .push(Action::Open(Page::Album(album.key.clone())));
            }
        }
        ui.add_space(16.0);
    }
    if !stats.top_artists.is_empty() {
        heading(app, ui, &gettext(locale, "Your most played artists here"));
        for (rank, artist) in stats.top_artists.iter().enumerate() {
            let plays =
                gettext(locale, "{count} plays").replace("{count}", &artist.plays.to_string());
            if ranked_row(app, ui, rank, None, &artist.name, "", &plays) && !artist.key.is_empty() {
                app.actions
                    .push(Action::Open(Page::Artist(artist.key.clone())));
            }
        }
        ui.add_space(16.0);
    }

    // Spotify's own lists: the last six months of artists, the last four
    // weeks of songs.
    let spotify_artists = app.home.top_artists.get().cloned().unwrap_or_default();
    if !spotify_artists.is_empty() {
        heading(
            app,
            ui,
            &gettext(locale, "Spotify's top artists, last six months"),
        );
        let genres = top_genres(&spotify_artists);
        if !genres.is_empty() {
            genre_mix(app, ui, &genres);
            ui.add_space(10.0);
        }
        for (rank, artist) in spotify_artists.iter().take(5).enumerate() {
            let image = crate::api::models::pick_image(&artist.images, 64).map(str::to_owned);
            let genre = artist.genres.first().cloned().unwrap_or_default();
            if ranked_row(app, ui, rank, image.as_deref(), &artist.name, &genre, "") {
                app.actions
                    .push(Action::Open(Page::Artist(artist.id.clone())));
            }
        }
        ui.add_space(16.0);
    }
    let spotify_songs = app.home.top_songs.get().cloned().unwrap_or_default();
    if !spotify_songs.is_empty() {
        heading(
            app,
            ui,
            &gettext(locale, "Spotify's top songs, last four weeks"),
        );
        for (rank, track) in spotify_songs.iter().take(5).enumerate() {
            let image = track.album.as_ref().and_then(|album| {
                crate::api::models::pick_image(&album.images, 64).map(str::to_owned)
            });
            let artist = track
                .artists
                .first()
                .map(|artist| artist.name.clone())
                .unwrap_or_default();
            if ranked_row(app, ui, rank, image.as_deref(), &track.name, &artist, "") {
                app.actions.push(Action::PlayUris {
                    uris: vec![track.uri.clone()],
                    index: 0,
                });
            }
        }
    }
}

/// The genres that come up most among the top artists, with how often.
fn top_genres(artists: &[crate::api::models::Artist]) -> Vec<(String, usize)> {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for genre in artists.iter().flat_map(|artist| &artist.genres) {
        match counts.iter_mut().find(|(name, _)| name == genre) {
            Some((_, count)) => *count += 1,
            None => counts.push((genre.clone(), 1)),
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    counts.into_iter().take(6).collect()
}

/// Your sound: the top genres as bars, longest first.
fn genre_mix(app: &App, ui: &mut Ui, genres: &[(String, usize)]) {
    let palette = app.palette;
    let most = genres.first().map_or(1, |(_, count)| *count).max(1);
    let width = ui.available_width().min(560.0);
    for (index, (genre, count)) in genres.iter().enumerate() {
        let (rect, _) = ui.allocate_exact_size(vec2(width, 26.0), Sense::hover());
        let label_width = 170.0_f32.min(width * 0.4);
        ui.painter().text(
            rect.left_center(),
            egui::Align2::LEFT_CENTER,
            genre,
            theme::medium(13.5),
            palette.text,
        );
        let track = Rect::from_min_max(
            pos2(rect.left() + label_width, rect.center().y - 6.0),
            pos2(rect.right(), rect.center().y + 6.0),
        );
        ui.painter()
            .rect_filled(track, CornerRadius::same(6), palette.surface);
        let mut hue = egui::ecolor::Hsva::from(palette.accent);
        hue.h = (hue.h + index as f32 * 0.06).fract();
        let fill = Rect::from_min_size(
            track.min,
            vec2(track.width() * *count as f32 / most as f32, track.height()),
        );
        ui.painter()
            .rect_filled(fill, CornerRadius::same(6), Color32::from(hue));
    }
}

/// Twelve weeks of days as a grid of squares, Monday at the top, lit by how
/// much music each day had.
fn listening_calendar(app: &App, ui: &mut Ui, days: &[(jiff::civil::Date, usize)]) {
    let palette = app.palette;
    let today = jiff::Zoned::now().date();
    let weeks = 12;
    let size = 15.0;
    let gap = 4.0;
    let (rect, _) = ui.allocate_exact_size(
        vec2(weeks as f32 * (size + gap), 7.0 * (size + gap)),
        Sense::hover(),
    );
    let most = days
        .iter()
        .map(|(_, count)| *count)
        .max()
        .unwrap_or(1)
        .max(1);
    let monday_offset = i64::from(today.weekday().to_monday_zero_offset());
    for week in 0..weeks {
        for weekday in 0..7_i64 {
            let back = (weeks - 1 - week) as i64 * 7 + monday_offset - weekday;
            if back < 0 {
                continue;
            }
            let Ok(day) = today.checked_sub(jiff::Span::new().days(back)) else {
                continue;
            };
            let count = days
                .iter()
                .find(|(date, _)| *date == day)
                .map_or(0, |(_, count)| *count);
            let cell = Rect::from_min_size(
                pos2(
                    rect.left() + week as f32 * (size + gap),
                    rect.top() + weekday as f32 * (size + gap),
                ),
                vec2(size, size),
            );
            let colour = if count == 0 {
                palette.surface
            } else {
                theme::motion::blend(
                    palette.surface_active,
                    palette.accent,
                    0.3 + 0.7 * count as f32 / most as f32,
                )
            };
            ui.painter()
                .rect_filled(cell, CornerRadius::same(4), colour);
        }
    }
}

fn heading(app: &App, ui: &mut Ui, text: &str) {
    theme::text(ui, text, theme::bold(22.0), app.palette.text);
    ui.add_space(6.0);
}

/// One big number on a card washed in the accent, each a little different.
fn stat_card(app: &App, ui: &mut Ui, width: f32, number: &str, label: &str, index: usize) {
    let palette = app.palette;
    let (rect, _) = ui.allocate_exact_size(vec2(width, 112.0), Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let corner = super::click_fx::corner(ui, 14.0);
    let painter = ui.painter();
    painter.rect_filled(rect, corner, palette.surface);
    // A diagonal wash of the accent, turned a little further for each card.
    let mut hue = egui::ecolor::Hsva::from(palette.accent);
    hue.h = (hue.h + index as f32 * 0.07).fract();
    let wash = Color32::from(hue);
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), wash.gamma_multiply(0.35));
    mesh.colored_vertex(rect.right_top(), wash.gamma_multiply(0.12));
    mesh.colored_vertex(rect.right_bottom(), Color32::TRANSPARENT);
    mesh.colored_vertex(rect.left_bottom(), wash.gamma_multiply(0.12));
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(egui::Shape::mesh(mesh));
    painter.text(
        rect.left_top() + vec2(16.0, 14.0),
        egui::Align2::LEFT_TOP,
        number,
        theme::bold(36.0),
        palette.text,
    );
    let galley = painter.layout(
        label.to_owned(),
        theme::medium(13.0),
        palette.secondary,
        width - 32.0,
    );
    painter.galley(
        rect.left_bottom() + vec2(16.0, -14.0 - galley.size().y),
        galley,
        palette.secondary,
    );
}

/// Twenty-four bars, one for each hour, the busiest lit in the accent.
fn listening_clock(app: &App, ui: &mut Ui, hours: &[usize; 24]) {
    let palette = app.palette;
    let height = 90.0;
    let (rect, _) = ui.allocate_exact_size(
        vec2(ui.available_width().min(760.0), height + 18.0),
        Sense::hover(),
    );
    let most = hours.iter().copied().max().unwrap_or(0).max(1);
    let slot = rect.width() / 24.0;
    let painter = ui.painter();
    for (hour, &count) in hours.iter().enumerate() {
        let fill = count as f32 / most as f32;
        let bar_height = (height * fill).max(3.0);
        let bar = Rect::from_min_max(
            pos2(
                rect.left() + hour as f32 * slot + 2.0,
                rect.top() + height - bar_height,
            ),
            pos2(
                rect.left() + (hour + 1) as f32 * slot - 2.0,
                rect.top() + height,
            ),
        );
        let colour = if count == most && count > 0 {
            palette.accent
        } else {
            widgets_blend(palette.surface_active, palette.accent, fill * 0.6)
        };
        painter.rect_filled(bar, CornerRadius::same(3), colour);
        if hour % 6 == 0 {
            painter.text(
                pos2(bar.left(), rect.top() + height + 4.0),
                egui::Align2::LEFT_TOP,
                format!("{hour:02}"),
                theme::regular(11.0),
                palette.dim,
            );
        }
    }
}

fn widgets_blend(from: Color32, to: Color32, amount: f32) -> Color32 {
    theme::motion::blend(from, to, amount)
}

/// A numbered row with a small picture. Answers whether it was clicked.
fn ranked_row(
    app: &App,
    ui: &mut Ui,
    rank: usize,
    image: Option<&str>,
    title: &str,
    detail: &str,
    trailing: &str,
) -> bool {
    let palette = app.palette;
    let response = Frame::new()
        .inner_margin(Margin::symmetric(8, 6))
        .corner_radius(CornerRadius::same(8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                theme::text(
                    ui,
                    format!("{}", rank + 1),
                    theme::bold(20.0),
                    if rank == 0 {
                        palette.accent
                    } else {
                        palette.secondary
                    },
                );
                let art = super::click_fx::radius(ui, 6.0);
                widgets::cover(ui, &palette, image, 44.0, art, Icon::Music);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    theme::text(ui, title, theme::semibold(15.0), palette.text);
                    if !detail.is_empty() {
                        theme::text(ui, detail, theme::regular(13.0), palette.secondary);
                    }
                });
                if !trailing.is_empty() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        theme::text(ui, trailing, theme::medium(13.0), palette.secondary);
                    });
                }
            });
        })
        .response;
    let response = ui.interact(
        response.rect,
        response.id.with("wrapped-row"),
        Sense::click(),
    );
    if response.hovered() {
        ui.painter().rect_filled(
            response.rect,
            super::click_fx::corner(ui, 8.0),
            palette.surface_hover.gamma_multiply(0.5),
        );
    }
    super::click_fx::note(ui, &response, 8.0);
    response.clicked()
}
