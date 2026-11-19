//! "My Wrapped": listening stats worked out from the plays this computer has
//! recorded. Nothing is sent anywhere; Spotify's own top artists and songs
//! are shown beside these on the page.

use std::collections::{BTreeSet, HashMap};

use crate::api::models::PlayHistory;

/// One song or artist and how often it was played.
#[derive(Clone, Debug, PartialEq)]
pub struct Ranked {
    pub name: String,
    /// The artist's name for a song; empty for an artist.
    pub detail: String,
    /// What it opens: a track URI, or an artist id.
    pub key: String,
    pub image: Option<String>,
    pub plays: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stats {
    pub plays: usize,
    pub minutes: u64,
    pub songs: usize,
    pub artists: usize,
    pub top_songs: Vec<Ranked>,
    pub top_artists: Vec<Ranked>,
    pub top_albums: Vec<Ranked>,
    /// Plays started in each hour of the day, local time.
    pub hours: [usize; 24],
    /// Plays on each weekday, Monday first.
    pub weekdays: [usize; 7],
    /// The most days in a row with at least one play.
    pub longest_streak: usize,
    /// The first and last day the plays cover.
    pub first_day: Option<jiff::civil::Date>,
    pub last_day: Option<jiff::civil::Date>,
    /// Plays on each day that had any, oldest first.
    pub days: Vec<(jiff::civil::Date, usize)>,
    /// The share of plays by the most played artist and song, 0 to 1.
    pub top_artist_share: f32,
}

/// A fun title earned from the way someone listens.
#[derive(Clone, Debug, PartialEq)]
pub struct Badge {
    pub emoji: &'static str,
    pub title: &'static str,
    pub detail: &'static str,
}

/// The titles these stats earn, most telling first. Too few plays earn none.
pub fn badges(stats: &Stats) -> Vec<Badge> {
    let mut earned = Vec::new();
    if stats.plays < 10 {
        return earned;
    }
    let share = |count: usize| count as f32 / stats.plays as f32;
    let night: usize = stats.hours[22..].iter().chain(&stats.hours[..4]).sum();
    let morning: usize = stats.hours[5..10].iter().sum();
    let weekend: usize = stats.weekdays[5..].iter().sum();
    if share(night) >= 0.3 {
        earned.push(Badge {
            emoji: "🦉",
            title: "Night owl",
            detail: "A lot of your listening happens after 10 at night",
        });
    }
    if share(morning) >= 0.3 {
        earned.push(Badge {
            emoji: "🌅",
            title: "Early bird",
            detail: "You start the day with music",
        });
    }
    if stats.top_artist_share >= 0.3 {
        earned.push(Badge {
            emoji: "💜",
            title: "Superfan",
            detail: "One artist takes up almost a third of your plays",
        });
    }
    if stats.artists as f32 >= stats.plays as f32 * 0.5 && stats.plays >= 20 {
        earned.push(Badge {
            emoji: "🧭",
            title: "Explorer",
            detail: "You rarely play the same artist twice in a row",
        });
    }
    if stats.top_songs.first().is_some_and(|song| song.plays >= 8) {
        earned.push(Badge {
            emoji: "🔁",
            title: "On repeat",
            detail: "There is a song you just cannot stop playing",
        });
    }
    if share(weekend) >= 0.4 {
        earned.push(Badge {
            emoji: "🎉",
            title: "Weekend warrior",
            detail: "Saturdays and Sundays are when the music really plays",
        });
    }
    if stats.longest_streak >= 7 {
        earned.push(Badge {
            emoji: "🔥",
            title: "On a streak",
            detail: "A week or more of music every single day",
        });
    }
    if stats.minutes >= 1_000 {
        earned.push(Badge {
            emoji: "🎧",
            title: "Marathon listener",
            detail: "Over a thousand minutes of music",
        });
    }
    earned
}

impl Stats {
    /// The hour of the day with the most plays.
    pub fn favourite_hour(&self) -> Option<usize> {
        busiest(&self.hours)
    }

    /// The weekday with the most plays, Monday as 0.
    pub fn favourite_weekday(&self) -> Option<usize> {
        busiest(&self.weekdays)
    }
}

fn busiest(counts: &[usize]) -> Option<usize> {
    let most = *counts.iter().max()?;
    (most > 0).then(|| counts.iter().position(|&count| count == most))?
}

/// Works out the stats for `plays` (any order) in `zone`.
pub fn stats(plays: &[PlayHistory], zone: &jiff::tz::TimeZone) -> Stats {
    let mut stats = Stats::default();
    let mut songs: HashMap<&str, Ranked> = HashMap::new();
    let mut artists: HashMap<String, Ranked> = HashMap::new();
    let mut albums: HashMap<String, Ranked> = HashMap::new();
    let mut days = BTreeSet::new();
    let mut day_counts: std::collections::BTreeMap<jiff::civil::Date, usize> = Default::default();
    for play in plays {
        let track = &play.track;
        if track.uri.is_empty() {
            continue;
        }
        stats.plays += 1;
        stats.minutes += u64::from(track.duration_ms) / 60_000;
        let artist = track.artists.first();
        let image = track
            .album
            .as_ref()
            .and_then(|album| album.images.last().or(album.images.first()))
            .map(|image| image.url.clone());
        songs
            .entry(track.uri.as_str())
            .or_insert_with(|| Ranked {
                name: track.name.clone(),
                detail: artist.map(|artist| artist.name.clone()).unwrap_or_default(),
                key: track.uri.clone(),
                image: image.clone(),
                plays: 0,
            })
            .plays += 1;
        if let Some(album) = track.album.as_ref().filter(|album| !album.id.is_empty()) {
            albums
                .entry(album.id.clone())
                .or_insert_with(|| Ranked {
                    name: album.name.clone(),
                    detail: artist.map(|artist| artist.name.clone()).unwrap_or_default(),
                    key: album.id.clone(),
                    image: image.clone(),
                    plays: 0,
                })
                .plays += 1;
        }
        for artist in &track.artists {
            let key = artist
                .id
                .clone()
                .filter(|id| !id.is_empty())
                .unwrap_or_else(|| artist.name.to_lowercase());
            if key.is_empty() {
                continue;
            }
            artists
                .entry(key.clone())
                .or_insert_with(|| Ranked {
                    name: artist.name.clone(),
                    detail: String::new(),
                    key: artist.id.clone().unwrap_or_default(),
                    image: None,
                    plays: 0,
                })
                .plays += 1;
        }
        if let Some(at) = play
            .played_at
            .as_deref()
            .and_then(|at| at.parse::<jiff::Timestamp>().ok())
        {
            let local = at.to_zoned(zone.clone());
            stats.hours[usize::from(local.hour() as u8) % 24] += 1;
            stats.weekdays[local.weekday().to_monday_zero_offset() as usize] += 1;
            *day_counts.entry(local.date()).or_insert(0) += 1;
            days.insert(local.date());
        }
    }
    stats.days = day_counts.into_iter().collect();
    stats.songs = songs.len();
    stats.artists = artists.len();
    stats.top_songs = top(songs.into_values().collect());
    stats.top_artist_share = if stats.plays == 0 {
        0.0
    } else {
        artists
            .values()
            .map(|artist| artist.plays)
            .max()
            .unwrap_or(0) as f32
            / stats.plays as f32
    };
    stats.top_artists = top(artists.into_values().collect());
    stats.top_albums = top(albums.into_values().collect());
    stats.first_day = days.first().copied();
    stats.last_day = days.last().copied();
    let mut run = 0;
    let mut previous: Option<jiff::civil::Date> = None;
    for day in days {
        run = match previous.and_then(|previous| previous.tomorrow().ok()) {
            Some(next) if next == day => run + 1,
            _ => 1,
        };
        stats.longest_streak = stats.longest_streak.max(run);
        previous = Some(day);
    }
    stats
}

/// The song played most on the day a week before today, for a throwback.
pub fn week_ago(
    plays: &[PlayHistory],
    today: jiff::civil::Date,
    zone: &jiff::tz::TimeZone,
) -> Option<Ranked> {
    let day = today.checked_sub(jiff::Span::new().days(7)).ok()?;
    let then: Vec<PlayHistory> = plays
        .iter()
        .filter(|play| {
            play.played_at
                .as_deref()
                .and_then(|at| at.parse::<jiff::Timestamp>().ok())
                .is_some_and(|at| at.to_zoned(zone.clone()).date() == day)
        })
        .cloned()
        .collect();
    stats(&then, zone).top_songs.into_iter().next()
}

/// The five most played, most plays first, then by name for a stable order.
fn top(mut ranked: Vec<Ranked>) -> Vec<Ranked> {
    ranked.sort_by(|a, b| {
        b.plays
            .cmp(&a.plays)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    ranked.truncate(5);
    ranked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::models::{ArtistRef, Track};

    fn play(uri: &str, song: &str, artist: &str, at: &str) -> PlayHistory {
        PlayHistory {
            track: Track {
                uri: uri.into(),
                name: song.into(),
                duration_ms: 180_000,
                artists: vec![ArtistRef {
                    id: Some(format!("id-{artist}")),
                    name: artist.into(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            played_at: Some(at.into()),
            context: None,
        }
    }

    #[test]
    fn counts_songs_artists_minutes_hours_and_streaks() {
        let plays = vec![
            play(
                "spotify:track:a",
                "Rosewood",
                "Bonobo",
                "2026-11-10T21:05:00Z",
            ),
            play(
                "spotify:track:a",
                "Rosewood",
                "Bonobo",
                "2026-11-11T21:30:00Z",
            ),
            play(
                "spotify:track:b",
                "Kerala",
                "Bonobo",
                "2026-11-12T08:00:00Z",
            ),
            play(
                "spotify:track:c",
                "Otomo",
                "Khruangbin",
                "2026-11-12T21:10:00Z",
            ),
            play(
                "spotify:track:a",
                "Rosewood",
                "Bonobo",
                "2026-11-15T21:45:00Z",
            ),
        ];
        let stats = stats(&plays, &jiff::tz::TimeZone::UTC);
        assert_eq!(stats.plays, 5);
        assert_eq!(stats.minutes, 15);
        assert_eq!(stats.songs, 3);
        assert_eq!(stats.artists, 2);
        assert_eq!(stats.top_songs[0].name, "Rosewood");
        assert_eq!(stats.top_songs[0].plays, 3);
        assert_eq!(stats.top_songs[0].detail, "Bonobo");
        assert_eq!(stats.top_artists[0].name, "Bonobo");
        assert_eq!(stats.top_artists[0].plays, 4);
        assert_eq!(stats.top_artists[0].key, "id-Bonobo");
        assert_eq!(stats.favourite_hour(), Some(21));
        assert_eq!(stats.hours[8], 1);
        // 10th, 11th and 12th in a row; then a gap before the 15th.
        assert_eq!(stats.longest_streak, 3);
        assert_eq!(stats.first_day, Some(jiff::civil::date(2026, 11, 10)));
        assert_eq!(stats.last_day, Some(jiff::civil::date(2026, 11, 15)));
        // 12 November 2026 is a Thursday, with two plays.
        assert_eq!(stats.favourite_weekday(), Some(3));
        assert_eq!(stats.days.len(), 4);
        assert_eq!(stats.days[2], (jiff::civil::date(2026, 11, 12), 2));
        assert!((stats.top_artist_share - 0.8).abs() < 0.001);
    }

    #[test]
    fn albums_are_ranked_and_a_week_ago_finds_that_days_favourite() {
        let mut plays = vec![
            play(
                "spotify:track:a",
                "Rosewood",
                "Bonobo",
                "2026-11-12T21:05:00Z",
            ),
            play(
                "spotify:track:a",
                "Rosewood",
                "Bonobo",
                "2026-11-12T21:30:00Z",
            ),
            play(
                "spotify:track:b",
                "Kerala",
                "Bonobo",
                "2026-11-12T08:00:00Z",
            ),
            play(
                "spotify:track:c",
                "Otomo",
                "Khruangbin",
                "2026-11-19T10:00:00Z",
            ),
        ];
        for play in &mut plays {
            play.track.album = Some(crate::api::models::Album {
                id: format!("album-{}", play.track.artists[0].name),
                name: format!("{} album", play.track.artists[0].name),
                ..Default::default()
            });
        }
        let utc = jiff::tz::TimeZone::UTC;
        let stats = stats(&plays, &utc);
        assert_eq!(stats.top_albums[0].name, "Bonobo album");
        assert_eq!(stats.top_albums[0].plays, 3);
        assert_eq!(stats.top_albums[0].key, "album-Bonobo");
        let throwback = week_ago(&plays, jiff::civil::date(2026, 11, 19), &utc).unwrap();
        assert_eq!(throwback.name, "Rosewood");
        assert_eq!(throwback.plays, 2);
        assert!(week_ago(&plays, jiff::civil::date(2026, 11, 20), &utc).is_none());
    }

    #[test]
    fn listening_habits_earn_badges_and_a_few_plays_earn_none() {
        let late: Vec<_> = (0..30)
            .map(|day| {
                play(
                    "spotify:track:a",
                    "Rosewood",
                    "Bonobo",
                    &format!("2026-10-{:02}T23:15:00Z", day % 28 + 1),
                )
            })
            .collect();
        let stats = stats(&late, &jiff::tz::TimeZone::UTC);
        let titles: Vec<_> = badges(&stats).iter().map(|badge| badge.title).collect();
        assert!(titles.contains(&"Night owl"), "{titles:?}");
        assert!(titles.contains(&"Superfan"), "{titles:?}");
        assert!(titles.contains(&"On repeat"), "{titles:?}");
        assert!(titles.contains(&"On a streak"), "{titles:?}");
        assert!(!titles.contains(&"Early bird"));
        assert!(!titles.contains(&"Explorer"));
        assert!(badges(&super::stats(&late[..5], &jiff::tz::TimeZone::UTC)).is_empty());
    }

    #[test]
    fn local_time_moves_late_plays_to_the_next_day_and_empty_history_is_empty() {
        let plays = vec![play(
            "spotify:track:a",
            "Rosewood",
            "Bonobo",
            "2026-11-10T23:30:00Z",
        )];
        let tokyo = jiff::tz::TimeZone::get("Asia/Tokyo").unwrap_or(jiff::tz::TimeZone::UTC);
        let stats = stats(&plays, &tokyo);
        if tokyo != jiff::tz::TimeZone::UTC {
            assert_eq!(stats.favourite_hour(), Some(8));
            assert_eq!(stats.first_day, Some(jiff::civil::date(2026, 11, 11)));
        }
        let empty = super::stats(&[], &jiff::tz::TimeZone::UTC);
        assert_eq!(empty, Stats::default());
        assert_eq!(empty.favourite_hour(), None);
        assert_eq!(empty.longest_streak, 0);
    }

    #[test]
    fn plays_without_a_time_still_count_and_only_five_are_ranked() {
        let mut plays: Vec<_> = (0..8)
            .map(|index| {
                let mut play = play(
                    &format!("spotify:track:{index}"),
                    &format!("Song {index}"),
                    &format!("Artist {index}"),
                    "not a time",
                );
                play.played_at = None;
                play
            })
            .collect();
        plays.push(play("", "", "", "2026-11-10T21:05:00Z"));
        let stats = stats(&plays, &jiff::tz::TimeZone::UTC);
        assert_eq!(stats.plays, 8, "a play with no song is skipped");
        assert_eq!(stats.top_songs.len(), 5);
        assert_eq!(stats.top_artists.len(), 5);
        assert_eq!(stats.hours.iter().sum::<usize>(), 0);
        assert_eq!(stats.longest_streak, 0);
    }
}
