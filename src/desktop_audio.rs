//! Windows output-loopback audio for visualizers when another app plays here.
//!
//! This reads only the default Windows output mix, never a microphone. Audio
//! stays in a bounded memory queue and the visualizer tap. Native playback
//! keeps its own post-EQ, pre-volume tap and always takes priority in the app.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::vis::AudioTap;

const FRESH: Duration = Duration::from_millis(350);
#[cfg(any(test, feature = "demo", windows))]
const SILENCE: f64 = 0.000_01;

/// Visualization-only gain. Follow peaks immediately and release slowly,
/// so a loud transient never gets the boost left over from a quiet section.
/// This runs once per 44.1 kHz stereo frame, independent of callback size.
#[cfg(any(test, feature = "demo", windows))]
struct Sensitivity {
    envelope: f64,
    release: f64,
    last_packet: Option<Instant>,
}

#[cfg(any(test, feature = "demo", windows))]
impl Default for Sensitivity {
    fn default() -> Self {
        Self {
            envelope: Self::TARGET / Self::MAX_BOOST,
            release: (-1.0_f64 / (44_100.0 * 0.7)).exp(),
            last_packet: None,
        }
    }
}

#[cfg(any(test, feature = "demo", windows))]
impl Sensitivity {
    const TARGET: f64 = 0.28;
    const MAX_BOOST: f64 = 64.0;

    fn process_at(&mut self, stereo: &[f64], at: Instant) -> Vec<f64> {
        if self
            .last_packet
            .is_some_and(|last| at.saturating_duration_since(last) > FRESH)
        {
            // WASAPI may send no callbacks while paused. A new quiet song
            // must not inherit the preceding song's loudness envelope.
            self.envelope = Self::TARGET / Self::MAX_BOOST;
        }
        self.last_packet = Some(at);
        self.process(stereo)
    }

    fn process(&mut self, stereo: &[f64]) -> Vec<f64> {
        let mut output = Vec::with_capacity(stereo.len());
        for frame in stereo.as_chunks::<2>().0 {
            let clean = frame.map(|sample| {
                if sample.is_finite() {
                    sample.clamp(-1.0, 1.0)
                } else {
                    0.0
                }
            });
            let peak = clean[0].abs().max(clean[1].abs());
            self.envelope = peak.max(self.envelope * self.release);
            let gain = (Self::TARGET / self.envelope.max(Self::TARGET / Self::MAX_BOOST))
                .clamp(1.0, Self::MAX_BOOST);
            output.extend(clean.map(|sample| sample * gain));
        }
        output
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Off,
    Starting,
    /// The output stream is open. It may currently be silent.
    Listening,
    Unavailable(String),
}

/// Aggregate counters for an explicit hardware diagnostic. No audio is saved.
#[derive(Clone, Debug, Default)]
pub struct Diagnostics {
    pub callbacks: u64,
    pub callback_samples: u64,
    pub queued_blocks: u64,
    pub dropped_blocks: u64,
    pub received_blocks: u64,
    pub accepted_blocks: u64,
    pub silent_blocks: u64,
    pub stream_errors: u64,
    pub nonfatal_errors: u64,
    pub raw_peak: f32,
    pub converted_peak: f32,
    pub endpoint: Option<String>,
}

#[derive(Default)]
struct Counters {
    callbacks: AtomicU64,
    callback_samples: AtomicU64,
    queued_blocks: AtomicU64,
    dropped_blocks: AtomicU64,
    received_blocks: AtomicU64,
    accepted_blocks: AtomicU64,
    silent_blocks: AtomicU64,
    stream_errors: AtomicU64,
    nonfatal_errors: AtomicU64,
    raw_peak: AtomicU64,
    converted_peak: AtomicU64,
    endpoint: Mutex<Option<String>>,
}

struct Shared {
    tap: Arc<AudioTap>,
    counters: Arc<Counters>,
    #[cfg(any(test, feature = "demo", windows))]
    sensitivity: Mutex<Sensitivity>,
    enabled: AtomicBool,
    shutdown: AtomicBool,
    generation: AtomicU64,
    origin: Instant,
    last_sound: AtomicU64,
    sound_generation: AtomicU64,
    status: Mutex<Status>,
    repaint: Mutex<Option<egui::Context>>,
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            tap: AudioTap::new(),
            counters: Arc::new(Counters::default()),
            #[cfg(any(test, feature = "demo", windows))]
            sensitivity: Mutex::new(Sensitivity::default()),
            enabled: AtomicBool::new(false),
            shutdown: AtomicBool::new(false),
            generation: AtomicU64::new(0),
            origin: Instant::now(),
            last_sound: AtomicU64::new(0),
            sound_generation: AtomicU64::new(0),
            status: Mutex::new(Status::Off),
            repaint: Mutex::new(None),
        }
    }
}

impl Shared {
    fn stamp(&self, at: Instant) -> u64 {
        at.saturating_duration_since(self.origin)
            .as_micros()
            .min(u128::from(u64::MAX - 1)) as u64
            + 1
    }

    fn current(&self, generation: u64) -> bool {
        self.enabled.load(Ordering::Acquire)
            && !self.shutdown.load(Ordering::Acquire)
            && self.generation.load(Ordering::Acquire) == generation
    }

    fn live_at(&self, now: Instant) -> bool {
        let generation = self.sound_generation.load(Ordering::Acquire);
        let last = self.last_sound.load(Ordering::Acquire);
        let now = self.stamp(now);
        self.current(generation)
            && last != 0
            && now >= last
            && now - last <= FRESH.as_micros() as u64
    }

    fn wake(&self) {
        if let Some(ctx) = self
            .repaint
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
        {
            ctx.request_repaint();
        }
    }

    fn set_status(&self, status: Status) {
        let changed = {
            let mut current = self.status.lock().unwrap_or_else(|p| p.into_inner());
            if *current == status {
                false
            } else {
                *current = status;
                true
            }
        };
        if changed {
            self.wake();
        }
    }

    fn clear(&self) {
        #[cfg(any(test, feature = "demo", windows))]
        {
            *self.sensitivity.lock().unwrap_or_else(|p| p.into_inner()) = Sensitivity::default();
        }
        self.clear_audio();
    }

    fn clear_audio(&self) {
        let had_sound = self.last_sound.swap(0, Ordering::AcqRel) != 0;
        self.tap.clear();
        if had_sound {
            self.wake();
        }
    }

    #[cfg(any(test, feature = "demo", windows))]
    fn accept_at(&self, stereo: &[f64], at: Instant, now: Instant, generation: u64) {
        if !self.current(generation) || now.saturating_duration_since(at) > FRESH {
            return;
        }
        // AudioTap's analyser reads the channel mean, so freshness follows
        // that same signal rather than an inaudible stereo cancellation.
        let audible = stereo.as_chunks::<2>().0.iter().any(|frame| {
            let mean = (frame[0] + frame[1]) * 0.5;
            mean.is_finite() && mean.abs() > SILENCE
        });
        // Detect silence before gain, so low-level device noise never
        // becomes a fresh audio source merely because it was amplified.
        let visible = self
            .sensitivity
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .process_at(stereo, at);
        if !audible {
            self.counters.silent_blocks.fetch_add(1, Ordering::Relaxed);
            // Keep the envelope's time-based release through a short silent
            // block or zero crossing. Lifecycle/staleness resets it instead.
            self.clear_audio();
            return;
        }
        self.counters
            .accepted_blocks
            .fetch_add(1, Ordering::Relaxed);
        let was_live = self.live_at(now);
        self.tap.push(&visible, 1.0);
        if !self.current(generation) {
            self.clear();
            return;
        }
        self.sound_generation.store(generation, Ordering::Release);
        self.last_sound.store(self.stamp(at), Ordering::Release);
        if !was_live {
            self.wake();
        }
    }

    #[cfg(any(test, all(windows, not(feature = "demo"))))]
    fn expire_at(&self, now: Instant) {
        if self.last_sound.load(Ordering::Acquire) != 0 && !self.live_at(now) {
            self.clear();
        }
    }
}

/// Owns no audio stream on the UI thread. Enabling lazily starts a worker;
/// disabling and dropping signal it without waiting for stream teardown.
#[derive(Default)]
pub struct DesktopAudio {
    shared: Arc<Shared>,
    worker: Option<std::thread::Thread>,
    #[cfg(all(windows, not(test), not(feature = "demo")))]
    worker_attempt: Option<Instant>,
}

impl DesktopAudio {
    pub fn tap(&self) -> &Arc<AudioTap> {
        &self.shared.tap
    }

    pub fn live(&self) -> bool {
        self.shared.live_at(Instant::now())
    }

    pub fn diagnostics(&self) -> Diagnostics {
        let c = &self.shared.counters;
        let get = |counter: &AtomicU64| counter.load(Ordering::Relaxed);
        Diagnostics {
            callbacks: get(&c.callbacks),
            callback_samples: get(&c.callback_samples),
            queued_blocks: get(&c.queued_blocks),
            dropped_blocks: get(&c.dropped_blocks),
            received_blocks: get(&c.received_blocks),
            accepted_blocks: get(&c.accepted_blocks),
            silent_blocks: get(&c.silent_blocks),
            stream_errors: get(&c.stream_errors),
            nonfatal_errors: get(&c.nonfatal_errors),
            raw_peak: f32::from_bits(get(&c.raw_peak) as u32),
            converted_peak: f32::from_bits(get(&c.converted_peak) as u32),
            endpoint: c.endpoint.lock().unwrap_or_else(|p| p.into_inner()).clone(),
        }
    }

    pub fn status(&self) -> Status {
        if !self.shared.enabled.load(Ordering::Acquire) {
            Status::Off
        } else {
            self.shared
                .status
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .clone()
        }
    }

    /// The worker wakes the UI for first sound, silence and status changes.
    /// Continuous packets use the app's existing visualizer frame pacing.
    pub fn set_repaint_context(&mut self, ctx: egui::Context) {
        *self
            .shared
            .repaint
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = Some(ctx);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        let changed = self.shared.enabled.swap(enabled, Ordering::AcqRel) != enabled;
        if changed {
            self.shared.generation.fetch_add(1, Ordering::AcqRel);
            self.shared.clear();
            self.shared.set_status(if enabled {
                Status::Starting
            } else {
                Status::Off
            });
            if let Some(worker) = &self.worker {
                worker.unpark();
            }
        }
        if enabled {
            self.ensure_worker();
        }
    }

    #[cfg(all(windows, not(test), not(feature = "demo")))]
    fn ensure_worker(&mut self) {
        if self.worker.is_some()
            || self
                .worker_attempt
                .is_some_and(|at| at.elapsed() < Duration::from_secs(2))
        {
            return;
        }
        self.worker_attempt = Some(Instant::now());
        let shared = Arc::clone(&self.shared);
        match std::thread::Builder::new()
            .name("windows-visualizer-audio".into())
            .spawn(move || capture::run(shared))
        {
            Ok(handle) => self.worker = Some(handle.thread().clone()),
            Err(error) => self.shared.set_status(Status::Unavailable(format!(
                "Could not start Windows audio: {error}"
            ))),
        }
    }

    #[cfg(test)]
    fn ensure_worker(&mut self) {}

    #[cfg(all(not(test), any(not(windows), feature = "demo")))]
    fn ensure_worker(&mut self) {
        self.shared.set_status(Status::Unavailable(
            if cfg!(feature = "demo") {
                "Windows audio capture is disabled in demo mode."
            } else {
                "System audio visualization is available on Windows."
            }
            .into(),
        ));
    }

    /// Deterministic app/UI tests use this without opening an audio device.
    #[cfg(any(test, feature = "demo"))]
    pub(crate) fn inject(&self, stereo: &[f64]) {
        let now = Instant::now();
        self.shared.accept_at(
            stereo,
            now,
            now,
            self.shared.generation.load(Ordering::Acquire),
        );
        if self.live() {
            self.shared.set_status(Status::Listening);
        }
    }
}

impl Drop for DesktopAudio {
    fn drop(&mut self) {
        self.shared.enabled.store(false, Ordering::Release);
        self.shared.shutdown.store(true, Ordering::Release);
        self.shared.generation.fetch_add(1, Ordering::AcqRel);
        if let Some(worker) = &self.worker {
            worker.unpark();
        }
        // The JoinHandle was detached. WASAPI stream destruction stays on
        // the worker, including when a device is slow or disconnected.
    }
}

#[cfg(any(test, all(windows, not(feature = "demo"))))]
struct StereoConverter {
    channels: usize,
    remainder: Vec<f32>,
    resampler: Option<crate::resample::Resampler>,
}

#[cfg(any(test, all(windows, not(feature = "demo"))))]
impl StereoConverter {
    fn new(rate: u32, channels: usize) -> Self {
        Self {
            channels,
            remainder: Vec::new(),
            resampler: crate::resample::Resampler::new(rate, 44_100, 2),
        }
    }

    fn process(&mut self, data: &[f32]) -> Vec<f64> {
        if self.channels == 0 {
            return Vec::new();
        }
        self.remainder.extend(data.iter().map(|sample| {
            if sample.is_finite() {
                sample.clamp(-1.0, 1.0)
            } else {
                0.0
            }
        }));
        let complete = self.remainder.len() / self.channels * self.channels;
        let mut stereo = Vec::with_capacity(complete / self.channels * 2);
        for frame in self.remainder[..complete].chunks_exact(self.channels) {
            match self.channels {
                1 => stereo.extend_from_slice(&[frame[0], frame[0]]),
                2 => stereo.extend_from_slice(frame),
                _ => {
                    let mono = frame.iter().sum::<f32>() / self.channels as f32;
                    stereo.extend_from_slice(&[mono, mono]);
                }
            }
        }
        self.remainder.drain(..complete);
        let stereo = match &mut self.resampler {
            Some(resampler) => resampler.process(&stereo),
            None => stereo,
        };
        stereo.into_iter().map(f64::from).collect()
    }
}

#[cfg(all(windows, any(test, not(feature = "demo"))))]
fn capture_error_is_fatal(kind: cpal::ErrorKind) -> bool {
    !matches!(
        kind,
        cpal::ErrorKind::Xrun | cpal::ErrorKind::RealtimeDenied
    )
}

#[cfg(all(windows, not(test), not(feature = "demo")))]
mod capture {
    use super::*;
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::mpsc::{self, Receiver, SyncSender};

    const QUEUE: usize = 8;
    const BLOCK_SAMPLES: usize = 16_384;
    const POLL: Duration = Duration::from_millis(40);
    const RETRY: Duration = Duration::from_secs(2);

    struct Block {
        samples: Vec<f32>,
        at: Instant,
    }

    pub(super) fn run(shared: Arc<Shared>) {
        while !shared.shutdown.load(Ordering::Acquire) {
            if !shared.enabled.load(Ordering::Acquire) {
                shared.clear();
                std::thread::park();
                continue;
            }
            let generation = shared.generation.load(Ordering::Acquire);
            // Explicitly use WASAPI. Only its output endpoint is opened.
            let attempt = cpal::host_from_id(cpal::HostId::Wasapi)
                .map_err(|error| format!("Windows audio is unavailable: {error}"))
                .and_then(|host| session(&host, &shared, generation));
            match attempt {
                Ok(()) => {}
                Err(error) if shared.current(generation) => {
                    shared.clear();
                    shared.set_status(Status::Unavailable(error));
                    let until = Instant::now() + RETRY;
                    while shared.current(generation) && Instant::now() < until {
                        std::thread::park_timeout(POLL);
                    }
                }
                Err(_) => {}
            }
        }
        shared.clear();
    }

    fn session(host: &cpal::Host, shared: &Shared, generation: u64) -> Result<(), String> {
        let device = host
            .default_output_device()
            .ok_or("No default Windows output device is available.")?;
        let id = device.id().map_err(|error| error.to_string())?;
        let supported = device
            .default_output_config()
            .map_err(|error| error.to_string())?;
        let rate = supported.sample_rate();
        let channels = usize::from(supported.channels());
        if rate == 0 || channels == 0 {
            return Err("The Windows output format has no channels or sample rate.".into());
        }
        let (sender, packets) = mpsc::sync_channel(QUEUE);
        let (errors, failure) = mpsc::sync_channel(1);
        *shared
            .counters
            .endpoint
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = Some(format!(
            "{id}; {rate} Hz; {channels} channels; {}",
            supported.sample_format()
        ));
        let stream = build(
            &device,
            supported,
            sender,
            errors,
            Arc::clone(&shared.counters),
        )?;
        if !shared.current(generation) {
            return Ok(());
        }
        stream.play().map_err(|error| error.to_string())?;
        shared.set_status(Status::Listening);
        let mut converter = StereoConverter::new(rate, channels);
        let mut check_output = Instant::now() + Duration::from_secs(1);
        while shared.current(generation) {
            if let Ok(error) = failure.try_recv() {
                return Err(format!("Windows output capture stopped: {error}"));
            }
            receive(&packets, &mut converter, shared, generation)?;
            let now = Instant::now();
            shared.expire_at(now);
            if now >= check_output {
                let current = host
                    .default_output_device()
                    .and_then(|device| device.id().ok());
                if current.as_ref() != Some(&id) {
                    shared.clear();
                    shared.set_status(Status::Starting);
                    return Ok(());
                }
                check_output = now + Duration::from_secs(1);
            }
        }
        shared.clear();
        Ok(())
    }

    fn receive(
        packets: &Receiver<Block>,
        converter: &mut StereoConverter,
        shared: &Shared,
        generation: u64,
    ) -> Result<(), String> {
        match packets.recv_timeout(POLL) {
            Ok(block) => {
                shared
                    .counters
                    .received_blocks
                    .fetch_add(1, Ordering::Relaxed);
                let now = Instant::now();
                if now.saturating_duration_since(block.at) <= FRESH {
                    let stereo = converter.process(&block.samples);
                    if !stereo.is_empty() {
                        let peak = stereo
                            .iter()
                            .map(|sample| sample.abs() as f32)
                            .filter(|sample| sample.is_finite())
                            .fold(0.0_f32, f32::max);
                        shared
                            .counters
                            .converted_peak
                            .fetch_max(u64::from(peak.to_bits()), Ordering::Relaxed);
                        shared.accept_at(&stereo, block.at, now, generation);
                    }
                }
                Ok(())
            }
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(()),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                Err("The Windows audio stream disconnected.".into())
            }
        }
    }

    fn build(
        device: &cpal::Device,
        config: cpal::SupportedStreamConfig,
        sender: SyncSender<Block>,
        errors: SyncSender<String>,
        counters: Arc<Counters>,
    ) -> Result<cpal::Stream, String> {
        macro_rules! stream {
            ($sample:ty) => {
                typed::<$sample>(device, config.into(), sender, errors, counters)
            };
        }
        match config.sample_format() {
            cpal::SampleFormat::F32 => stream!(f32),
            cpal::SampleFormat::F64 => stream!(f64),
            cpal::SampleFormat::I8 => stream!(i8),
            cpal::SampleFormat::I16 => stream!(i16),
            cpal::SampleFormat::I24 => stream!(cpal::I24),
            cpal::SampleFormat::I32 => stream!(i32),
            cpal::SampleFormat::I64 => stream!(i64),
            cpal::SampleFormat::U8 => stream!(u8),
            cpal::SampleFormat::U16 => stream!(u16),
            cpal::SampleFormat::U24 => stream!(cpal::U24),
            cpal::SampleFormat::U32 => stream!(u32),
            cpal::SampleFormat::U64 => stream!(u64),
            format => Err(format!(
                "The Windows output sample format {format} is unsupported."
            )),
        }
    }

    fn typed<T>(
        device: &cpal::Device,
        config: cpal::StreamConfig,
        sender: SyncSender<Block>,
        errors: SyncSender<String>,
        counters: Arc<Counters>,
    ) -> Result<cpal::Stream, String>
    where
        T: cpal::SizedSample,
        f32: cpal::FromSample<T>,
    {
        // Every queued block ends on a frame, including surround output.
        // Dropping a full queue block can never shift channel alignment.
        let channels = usize::from(config.channels).max(1);
        let block_samples = (BLOCK_SAMPLES / channels).max(1) * channels;
        let error_counters = Arc::clone(&counters);
        device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    // No tap mutex, resampling, waiting, file or network work in
                    // the audio callback. A full bounded queue drops the block.
                    counters.callbacks.fetch_add(1, Ordering::Relaxed);
                    counters
                        .callback_samples
                        .fetch_add(data.len() as u64, Ordering::Relaxed);
                    let at = Instant::now();
                    for part in data.chunks(block_samples) {
                        let mut peak = 0.0_f32;
                        let samples = part
                            .iter()
                            .map(|&sample| {
                                let sample = <f32 as cpal::FromSample<T>>::from_sample_(sample);
                                if sample.is_finite() {
                                    peak = peak.max(sample.abs());
                                }
                                sample
                            })
                            .collect();
                        counters
                            .raw_peak
                            .fetch_max(u64::from(peak.to_bits()), Ordering::Relaxed);
                        if sender.try_send(Block { samples, at }).is_err() {
                            counters.dropped_blocks.fetch_add(1, Ordering::Relaxed);
                            break;
                        }
                        counters.queued_blocks.fetch_add(1, Ordering::Relaxed);
                    }
                },
                move |error| {
                    error_counters.stream_errors.fetch_add(1, Ordering::Relaxed);
                    // A discontinuity/overrun loses a packet, not the stream.
                    // CPAL also reports best-effort real-time priority denial.
                    if !capture_error_is_fatal(error.kind()) {
                        error_counters
                            .nonfatal_errors
                            .fetch_add(1, Ordering::Relaxed);
                        return;
                    }
                    let _ = errors.try_send(error.to_string());
                },
                Some(Duration::from_secs(2)),
            )
            .map_err(|error| format!("Could not listen to Windows output: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quiet_tone(frames: usize, amplitude: f64) -> Vec<f64> {
        (0..frames)
            .flat_map(|frame| {
                let sample =
                    (frame as f64 * std::f64::consts::TAU * 440.0 / 44_100.0).sin() * amplitude;
                [sample, sample]
            })
            .collect()
    }

    #[test]
    fn quiet_desktop_audio_is_visible_without_changing_the_input() {
        let input = quiet_tone(8_000, 0.002);
        let unchanged = input.clone();
        let audio = active();
        audio.inject(&input);
        let samples = audio.tap().window(crate::vis::WIDE_SAMPLES, 0);
        let peak = samples
            .iter()
            .map(|sample| sample.abs())
            .fold(0.0_f32, f32::max);
        assert!(
            peak > 0.12 && peak < 0.14,
            "quiet sound should be clearly boosted: {peak}"
        );
        let trace = crate::vis::scope_line(&samples, 70);
        assert!(trace.iter().any(|sample| sample.abs() > 0.2));
        let levels = crate::vis::WideAnalyser::default().step(&samples, Instant::now());
        assert!(
            levels.iter().any(|level| *level > 0.25),
            "quiet audio must move the spectrum"
        );
        assert_eq!(input, unchanged, "gain must never change the source audio");
    }

    #[test]
    fn desktop_sensitivity_handles_loud_transients_and_callback_boundaries() {
        let mut input = quiet_tone(2_000, 0.002);
        input.extend([1.0, -0.95, -1.0, 0.95]);
        input.extend(quiet_tone(4_000, 0.01));
        input.extend([0.0; 8_000]);
        input.extend(quiet_tone(1_000, 0.003));
        let expected = Sensitivity::default().process(&input);
        let mut sensitivity = Sensitivity::default();
        let mut actual = Vec::new();
        let mut at = 0;
        for frames in [1, 137, 4_096, 2, 317].into_iter().cycle() {
            if at == input.len() {
                break;
            }
            let end = (at + frames * 2).min(input.len());
            actual.extend(sensitivity.process(&input[at..end]));
            at = end;
        }
        assert_eq!(
            actual, expected,
            "callback boundaries must not pump the gain"
        );
        assert!(
            actual
                .iter()
                .all(|sample| sample.is_finite() && sample.abs() <= 1.0)
        );
        assert_eq!(
            actual[4_000], 1.0,
            "a sudden loud sample must lose the quiet boost instantly"
        );
    }

    #[test]
    fn a_quiet_song_after_a_callback_gap_does_not_inherit_loudness() {
        let audio = active();
        let at = audio.shared.origin + Duration::from_secs(1);
        let generation = audio.shared.generation.load(Ordering::Acquire);
        audio.shared.accept_at(&[0.9; 512], at, at, generation);
        let silent = at + Duration::from_millis(10);
        audio
            .shared
            .accept_at(&[0.0; 512], silent, silent, generation);
        let resumed = at + Duration::from_secs(2);
        audio
            .shared
            .accept_at(&quiet_tone(1_000, 0.002), resumed, resumed, generation);
        assert!(audio.shared.live_at(resumed));
        assert!(
            audio
                .tap()
                .window(128, 0)
                .iter()
                .any(|sample| sample.abs() > 0.12),
            "quiet playback should be visible immediately after a long pause"
        );
    }

    #[test]
    fn desktop_sensitivity_keeps_silence_still_and_resets_between_sessions() {
        let mut audio = active();
        audio.inject(&[0.000_001; 512]);
        assert!(
            !audio.live(),
            "boost must not turn sub-threshold noise into music"
        );
        assert!(
            audio
                .tap()
                .window(128, 0)
                .iter()
                .all(|sample| *sample == 0.0)
        );
        audio.inject(&[0.9; 512]);
        audio.set_enabled(false);
        audio.set_enabled(true);
        audio.inject(&quiet_tone(1_000, 0.002));
        assert!(
            audio
                .tap()
                .window(128, 0)
                .iter()
                .any(|sample| sample.abs() > 0.12),
            "a new session must not inherit the old loudness envelope"
        );
        audio.inject(&[0.0; 512]);
        assert!(!audio.live());
        assert!(
            audio
                .tap()
                .window(128, 0)
                .iter()
                .all(|sample| *sample == 0.0)
        );
    }

    fn active() -> DesktopAudio {
        let mut audio = DesktopAudio::default();
        audio.set_enabled(true);
        audio
    }

    #[cfg(windows)]
    #[test]
    fn recoverable_capture_notices_keep_the_stream_running() {
        assert!(!capture_error_is_fatal(cpal::ErrorKind::Xrun));
        assert!(!capture_error_is_fatal(cpal::ErrorKind::RealtimeDenied));
        assert!(capture_error_is_fatal(cpal::ErrorKind::DeviceNotAvailable));
        assert!(capture_error_is_fatal(cpal::ErrorKind::DeviceChanged));
        assert!(capture_error_is_fatal(cpal::ErrorKind::BackendError));
    }

    #[test]
    fn lifecycle_is_lazy_and_disabled_input_cannot_reappear() {
        let mut audio = DesktopAudio::default();
        assert_eq!(audio.status(), Status::Off);
        assert!(audio.worker.is_none());
        audio.inject(&[0.5; 128]);
        assert!(!audio.live());
        audio.set_enabled(true);
        assert_eq!(audio.status(), Status::Starting);
        assert!(audio.worker.is_none(), "tests must never open a device");
        let generation = audio.shared.generation.load(Ordering::Acquire);
        audio.inject(&[0.5; 128]);
        assert!(audio.live());
        audio.set_enabled(false);
        assert!(!audio.live());
        assert_eq!(audio.status(), Status::Off);
        assert!(
            audio
                .tap()
                .window(64, 0)
                .iter()
                .all(|sample| *sample == 0.0)
        );
        audio.set_enabled(true);
        let now = Instant::now();
        audio.shared.accept_at(&[0.8; 128], now, now, generation);
        assert!(
            !audio.live(),
            "a previous capture generation must stay discarded"
        );
    }

    #[test]
    fn silence_and_stale_callbacks_clear_the_last_frame_without_sleeping() {
        let audio = active();
        let at = audio.shared.origin + Duration::from_secs(1);
        let generation = audio.shared.generation.load(Ordering::Acquire);
        audio.shared.accept_at(&[0.5; 128], at, at, generation);
        assert!(audio.shared.live_at(at + FRESH));
        audio
            .shared
            .expire_at(at + FRESH + Duration::from_micros(1));
        assert!(!audio.shared.live_at(at + FRESH + Duration::from_micros(1)));
        assert!(
            audio
                .tap()
                .window(32, 0)
                .iter()
                .all(|sample| *sample == 0.0)
        );
        audio.shared.accept_at(
            &[0.6; 128],
            at,
            at + FRESH + Duration::from_millis(1),
            generation,
        );
        assert!(!audio.shared.live_at(at + FRESH));
        audio.shared.accept_at(&[0.5; 128], at, at, generation);
        audio.shared.accept_at(&[0.0; 128], at, at, generation);
        assert!(!audio.shared.live_at(at));
        assert!(
            audio
                .tap()
                .window(32, 0)
                .iter()
                .all(|sample| *sample == 0.0)
        );
    }

    #[test]
    fn channel_conversion_handles_mono_stereo_surround_and_partial_frames() {
        assert_eq!(
            StereoConverter::new(44_100, 1).process(&[0.25, -0.5]),
            [0.25, 0.25, -0.5, -0.5]
        );
        assert_eq!(
            StereoConverter::new(44_100, 2).process(&[0.25, -0.5]),
            [0.25, -0.5]
        );
        let mut surround = StereoConverter::new(44_100, 6);
        assert!(surround.process(&[0.6, 0.0, 0.0]).is_empty());
        let out = surround.process(&[0.0, 0.0, 0.0]);
        assert!((out[0] - 0.1).abs() < 1e-6);
        assert_eq!(out[0], out[1]);
        assert!(StereoConverter::new(44_100, 0).process(&[0.5]).is_empty());
        assert_eq!(
            StereoConverter::new(44_100, 1).process(&[f32::NAN, f32::INFINITY]),
            [0.0; 4]
        );
    }

    #[test]
    fn rate_conversion_preserves_a_tone_and_is_independent_of_callback_chunks() {
        for rate in [22_050, 48_000, 96_000] {
            let input: Vec<f32> = (0..rate)
                .flat_map(|index| {
                    let sample =
                        (std::f32::consts::TAU * 440.0 * index as f32 / rate as f32).sin() * 0.25;
                    [sample, sample]
                })
                .collect();
            let expected = StereoConverter::new(rate, 2).process(&input);
            assert!((expected.len() as i64 / 2 - 44_100).abs() < 100);
            let mut converter = StereoConverter::new(rate, 2);
            let actual: Vec<f64> = input
                .chunks(137)
                .flat_map(|chunk| converter.process(chunk))
                .collect();
            assert_eq!(
                actual, expected,
                "{rate} Hz chunk boundaries changed the result"
            );
            assert!(actual.iter().all(|sample| sample.is_finite()));
            let crossings = actual
                .as_chunks::<2>()
                .0
                .iter()
                .map(|frame| frame[0])
                .collect::<Vec<_>>()
                .windows(2)
                .filter(|pair| pair[0] <= 0.0 && pair[1] > 0.0)
                .count();
            assert!(
                (crossings as i32 - 440).abs() <= 2,
                "{rate} Hz: {crossings} crossings"
            );
        }
    }
}
