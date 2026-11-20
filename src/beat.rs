//! Beat pulse: finds the kick of the music in the audio the visualisers
//! already receive, so the interface can flash gently in time with it.
//!
//! It looks only at the bass. A beat is a moment when the bass is clearly
//! louder than it has been on average for the last fraction of a second.
//! No audio is stored; each frame's window is read and forgotten.

/// How quickly the low-pass filter follows the signal: a little above 100 Hz
/// at Spotify's 44.1 kHz.
const LOW_PASS: f32 = 0.016;
/// How much louder than its recent average the bass must be to count.
const BEAT_RATIO: f32 = 1.35;
/// The bass must fall back under this much of its average before another
/// beat can start.
const REARM_RATIO: f32 = 1.1;
/// How long after a beat the next can start even if the bass stays up.
const REARM_TIME: f64 = 0.5;
/// Below this, the bass is too quiet to call anything a beat. It is low so
/// the pulse still follows quiet listening on the Windows output.
const QUIET: f32 = 0.0015;
/// Beats closer together than this are one beat: about 240 bpm at most.
const REFRACTORY: f64 = 0.25;
/// How long the average takes to follow the bass, in seconds. It is timed
/// by the clock, not by frames: at a high refresh rate a per-frame average
/// caught up with each kick before it could stand out.
const AVERAGE_TIME: f64 = 0.35;
/// How long a pulse takes to swell to its height, so it eases in rather
/// than snapping on.
const ATTACK: f64 = 0.06;
/// How long a pulse then takes to fade to about a third of its height.
const FADE: f64 = 0.3;

#[derive(Debug, Default, Clone)]
pub struct BeatPulse {
    average: f32,
    last_step: Option<f64>,
    last_beat: Option<f64>,
    /// Set again once the bass falls back towards its average, so one long
    /// swell is one beat.
    spent: bool,
}

impl BeatPulse {
    /// Reads this frame's audio at `now` seconds. Answers whether a beat
    /// started.
    pub fn step(&mut self, samples: &[f32], now: f64) -> bool {
        let energy = bass_energy(samples);
        // Ready again once the bass falls back, or after a moment anyway, so
        // a song that starts loud still pulses.
        if energy < self.average * REARM_RATIO
            || self.last_beat.is_some_and(|last| now - last >= REARM_TIME)
        {
            self.spent = false;
        }
        let beat = !self.spent
            && energy > QUIET
            && energy > self.average * BEAT_RATIO
            && self
                .last_beat
                .is_none_or(|last| !(now - last < REFRACTORY && now >= last));
        if beat {
            self.last_beat = Some(now);
            self.spent = true;
        }
        let elapsed = self
            .last_step
            .map_or(1.0 / 60.0, |last| (now - last).clamp(0.0, 0.25));
        self.last_step = Some(now);
        let follow = 1.0 - (-elapsed / AVERAGE_TIME).exp();
        self.average += (energy - self.average) * follow as f32;
        beat
    }

    /// The pulse's height at `now`: easing up to 1 just after a beat, then
    /// fading smoothly towards 0.
    pub fn pulse(&self, now: f64) -> f32 {
        match self.last_beat {
            Some(last) if now >= last && now - last < ATTACK + FADE * 5.0 => {
                let since = now - last;
                if since < ATTACK {
                    let rise = since / ATTACK;
                    (rise * rise * (3.0 - 2.0 * rise)) as f32
                } else {
                    (-(since - ATTACK) / FADE).exp() as f32
                }
            }
            _ => 0.0,
        }
    }

    /// Forget everything, as when the music stops.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// The mean loudness of the window's bass.
fn bass_energy(samples: &[f32]) -> f32 {
    let Some(&first) = samples.first() else {
        return 0.0;
    };
    // Two low-pass stages, so high notes, however loud, barely register.
    let first = if first.is_finite() { first } else { 0.0 };
    let (mut low, mut lower) = (first, first);
    let mut total = 0.0;
    for &sample in samples {
        let sample = if sample.is_finite() { sample } else { 0.0 };
        low += (sample - low) * LOW_PASS;
        lower += (low - lower) * LOW_PASS;
        total += lower.abs();
    }
    total / samples.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(frequency: f32, amplitude: f32) -> Vec<f32> {
        (0..1024)
            .map(|index| {
                (index as f32 / 44_100.0 * frequency * std::f32::consts::TAU).sin() * amplitude
            })
            .collect()
    }

    #[test]
    fn a_kick_after_quiet_bass_is_a_beat_and_fades() {
        let mut pulse = BeatPulse::default();
        let quiet = tone(60.0, 0.05);
        let kick = tone(60.0, 0.8);
        for frame in 0..30 {
            pulse.step(&quiet, frame as f64 / 60.0);
        }
        assert!(pulse.step(&kick, 0.6), "a loud kick is a beat");
        // It eases in rather than flashing on.
        assert_eq!(pulse.pulse(0.6), 0.0);
        assert!(pulse.pulse(0.63) > 0.3 && pulse.pulse(0.63) < 0.7);
        assert!((pulse.pulse(0.6 + ATTACK) - 1.0).abs() < 1e-6);
        assert!(pulse.pulse(0.8) < 0.7 && pulse.pulse(0.8) > 0.4);
        assert!(pulse.pulse(1.3) < 0.15);
        assert_eq!(pulse.pulse(5.0), 0.0, "it settles completely");
        assert!(!pulse.step(&kick, 0.65), "one kick is one beat");
    }

    #[test]
    fn steady_sound_treble_and_silence_are_not_beats() {
        let mut pulse = BeatPulse::default();
        let hum = tone(60.0, 0.4);
        let mut beats = 0;
        for frame in 0..120 {
            beats += usize::from(pulse.step(&hum, frame as f64 / 60.0));
        }
        assert!(
            beats <= 1,
            "a steady hum is at most its first onset: {beats}"
        );

        let mut pulse = BeatPulse::default();
        let hiss = tone(9_000.0, 0.8);
        assert!((0..60).all(|frame| !pulse.step(&hiss, frame as f64 / 60.0)));

        let mut pulse = BeatPulse::default();
        assert!(!pulse.step(&[], 0.0));
        assert!(!pulse.step(&[0.0; 512], 0.1));
        assert!(!pulse.step(&[f32::NAN; 512], 0.2));
        assert_eq!(pulse.pulse(0.2), 0.0);
    }

    #[test]
    fn quiet_kicks_at_a_high_refresh_rate_still_pulse() {
        // 170 frames a second, the kick swelling over a few frames, and the
        // whole song at a low listening volume.
        let mut pulse = BeatPulse::default();
        let quiet = tone(60.0, 0.004);
        let mut beats = 0;
        for frame in 0..(170 * 4) {
            let now = frame as f64 / 170.0;
            let into_beat = frame % 85;
            let samples = match into_beat {
                0..=7 => tone(60.0, 0.012 + into_beat as f32 * 0.008),
                _ => quiet.clone(),
            };
            beats += usize::from(pulse.step(&samples, now));
        }
        assert!(
            (7..=9).contains(&beats),
            "{beats} beats in four seconds at 120 bpm"
        );
    }

    #[test]
    fn regular_kicks_each_pulse_and_reset_forgets() {
        let mut pulse = BeatPulse::default();
        let quiet = tone(60.0, 0.03);
        let kick = tone(60.0, 0.7);
        let mut beats = 0;
        // 120 bpm for four seconds: a kick every half second.
        for frame in 0..240 {
            let now = frame as f64 / 60.0;
            let samples = if frame % 30 < 3 { &kick } else { &quiet };
            beats += usize::from(pulse.step(samples, now));
        }
        assert!((7..=9).contains(&beats), "{beats} beats");
        pulse.reset();
        assert_eq!(pulse.pulse(4.0), 0.0);
    }
}
