//! Publishable track metadata and stable Discord progress timestamps.
//! No transport, credentials, playback requests or disk access live here.
use crate::app::NowPlaying;
use crate::discord::Activity;

/// This is a public Discord application identifier, never a secret.
/// The application owner can replace it in Settings for a personal build.
pub const DEFAULT_APPLICATION_ID: &str = "1566813127557783714";

#[derive(Default)]
pub struct Projection {
    anchor: Option<(String, i64)>,
    sharing: bool,
}

impl Projection {
    pub fn is_sharing(&self) -> bool {
        self.sharing
    }

    /// Paused/private/unconfirmed content clears the activity. Ordinary
    /// progress ticks keep one timestamp; a real seek establishes a new one.
    pub fn update(
        &mut self,
        now: Option<&NowPlaying>,
        private: bool,
        unix_seconds: i64,
    ) -> Option<Activity> {
        let eligible = now.filter(|now| now.playing && !now.loading && !now.resuming && !private);
        let result = eligible.and_then(|now| {
            let (kind, id) = catalog_item(&now.uri)?;
            if now.title.trim().is_empty()
                || (now.duration_ms > 0 && now.position_ms >= now.duration_ms)
            {
                return None;
            }
            let title = now.title.trim();
            let details = if title.chars().count() == 1 {
                format!("♪ {title}")
            } else {
                title.to_owned()
            };
            let (start, end) = if now.duration_ms == 0 || unix_seconds < 0 {
                self.anchor = None;
                (None, None)
            } else {
                let candidate = unix_seconds
                    .saturating_sub(i64::from(now.position_ms / 1000))
                    .max(0);
                let start = self
                    .anchor
                    .as_ref()
                    .filter(|(uri, start)| uri == &now.uri && start.abs_diff(candidate) <= 2)
                    .map_or(candidate, |(_, start)| *start);
                self.anchor = Some((now.uri.clone(), start));
                (
                    Some(start),
                    Some(start.saturating_add(i64::from(now.duration_ms.div_ceil(1000)))),
                )
            };
            Some(Activity {
                details,
                state: (!now.subtitle.trim().is_empty()).then(|| now.subtitle.trim().to_owned()),
                large_image: now.art_url.clone(),
                large_text: (!now.album_name.trim().is_empty())
                    .then(|| now.album_name.trim().to_owned()),
                start,
                end,
                url: Some(format!("https://open.spotify.com/{kind}/{id}")),
            })
        });
        self.sharing = result.is_some();
        if !self.sharing {
            self.anchor = None;
        }
        result
    }
}

fn catalog_item(uri: &str) -> Option<(&str, &str)> {
    let mut parts = uri.split(':');
    if parts.next()? != "spotify" {
        return None;
    }
    let kind = parts.next()?;
    let id = parts.next()?;
    (matches!(kind, "track" | "episode")
        && parts.next().is_none()
        && id.len() == 22
        && id.bytes().all(|byte| byte.is_ascii_alphanumeric()))
    .then_some((kind, id))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn song() -> NowPlaying {
        NowPlaying {
            local: true,
            device_name: None,
            device_kind: None,
            uri: "spotify:track:0123456789ABCDEFGHIJKL".into(),
            id: Some("0123456789ABCDEFGHIJKL".into()),
            title: "A song".into(),
            artists: Vec::new(),
            subtitle: "An artist".into(),
            album_name: "An album".into(),
            album_id: None,
            show_id: None,
            art_url: Some("https://i.scdn.co/image/public-cover".into()),
            art_small: None,
            duration_ms: 180000,
            position_ms: 30000,
            playing: true,
            loading: false,
            shuffle: false,
            repeat: crate::player::RepeatMode::Off,
            volume_percent: 10,
            can_control: true,
            can_set_volume: true,
            is_episode: false,
            resuming: false,
        }
    }
    #[test]
    fn projects_catalog_metadata_and_a_bounded_progress_range() {
        let mut state = Projection::default();
        let a = state.update(Some(&song()), false, 1000).unwrap();
        assert_eq!(a.details, "A song");
        assert_eq!(a.state.as_deref(), Some("An artist"));
        assert_eq!(a.start, Some(970));
        assert_eq!(a.end, Some(1150));
        assert_eq!(
            a.url.as_deref(),
            Some("https://open.spotify.com/track/0123456789ABCDEFGHIJKL")
        );
        assert!(state.is_sharing());
    }
    #[test]
    fn normal_progress_ticks_are_identical_but_a_seek_and_repeat_update_timestamps() {
        let mut p = Projection::default();
        let mut now = song();
        let first = p.update(Some(&now), false, 1000).unwrap();
        now.position_ms += 1100;
        let next = p.update(Some(&now), false, 1001).unwrap();
        assert!(first == next);
        now.position_ms = 90000;
        let seek = p.update(Some(&now), false, 1001).unwrap();
        assert_eq!(seek.start, Some(911));
        now.position_ms = 0;
        assert_eq!(p.update(Some(&now), false, 1002).unwrap().start, Some(1002));
    }
    #[test]
    fn pause_private_load_resume_and_end_never_publish() {
        for mode in 0..6 {
            let mut now = song();
            let mut p = Projection::default();
            assert!(p.update(Some(&now), false, 1000).is_some());
            match mode {
                0 => now.playing = false,
                1 => now.loading = true,
                2 => now.resuming = true,
                3 => now.position_ms = now.duration_ms,
                4 => now.title.clear(),
                _ => {}
            }
            assert!(p.update(Some(&now), mode == 5, 1001).is_none());
            assert!(!p.is_sharing());
            assert!(p.anchor.is_none());
        }
    }
    #[test]
    fn never_links_or_publishes_local_files_or_arbitrary_uris() {
        for uri in [
            "spotify:local:private:folder:file",
            "file:///C:/private.wav",
            "https://example.com/private",
            "spotify:track:bad/id",
            "spotify:track:0123456789ABCDEFGHIJKL:extra",
        ] {
            let mut now = song();
            now.uri = uri.into();
            assert!(
                Projection::default()
                    .update(Some(&now), false, 1000)
                    .is_none()
            );
        }
    }
    #[test]
    fn episodes_and_unknown_durations_are_supported_without_a_fake_timer() {
        let mut now = song();
        now.uri = now.uri.replace(":track:", ":episode:");
        now.duration_ms = 0;
        let a = Projection::default()
            .update(Some(&now), false, 1000)
            .unwrap();
        assert!(a.start.is_none() && a.end.is_none());
        assert!(a.url.unwrap().contains("/episode/"));
    }
    #[test]
    fn clearing_then_resuming_uses_the_new_clock_and_position() {
        let mut p = Projection::default();
        let now = song();
        p.update(Some(&now), false, 1000);
        assert!(p.update(None, false, 1010).is_none());
        assert_eq!(p.update(Some(&now), false, 1030).unwrap().start, Some(1000));
    }
}
