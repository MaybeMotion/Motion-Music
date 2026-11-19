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
    /// Plays started in each hour of the day, local time.
    pub hours: [usize; 24],
    /// Plays on each weekday, Monday first.
    pub weekdays: [usize; 7],
    /// The most days in a row with at least one play.
    pub longest_streak: usize,
    /// The first and last day the plays cover.
    pub first_day: Option<jiff::civil::Date>,
    pub last_day: Option<jiff::civil::Date>,
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
    let mut days = BTreeSet::new();
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
            days.insert(local.date());
        }
    }
    stats.songs = songs.len();
    stats.artists = artists.len();
    stats.top_songs = top(songs.into_values().collect());
    stats.top_artists = top(artists.into_values().collect());
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
