//! Explicit Windows hardware diagnostic. Plays a quiet two-second tone on
//! the default output and checks that output-loopback feeds the visualizer.
//! Run without demo or MilkDrop: cargo run --no-default-features --example desktop_visualiser_probe

#[cfg(windows)]
fn main() -> anyhow::Result<()> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use spotifast::desktop_audio::{DesktopAudio, Status};
    use std::sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    };
    use std::time::{Duration, Instant};

    anyhow::ensure!(
        !cfg!(feature = "demo"),
        "Run this hardware probe without --features demo."
    );
    let mut audio = DesktopAudio::default();
    audio.set_enabled(true);
    let deadline = Instant::now() + Duration::from_secs(6);
    while audio.status() != Status::Listening && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(20));
    }
    anyhow::ensure!(
        audio.status() == Status::Listening,
        "Loopback did not open: {:?}",
        audio.status()
    );
    let device = cpal::default_host()
        .default_output_device()
        .ok_or_else(|| anyhow::anyhow!("No default Windows output"))?;
    let config = device.default_output_config()?;
    println!(
        "Output-loopback open; playing a quiet two-second tone at {} Hz.",
        config.sample_rate()
    );
    println!("Tone endpoint: {}", device.id()?);
    println!("Capture endpoint: {:?}", audio.diagnostics().endpoint);
    let output_frames = Arc::new(AtomicU64::new(0));
    macro_rules! tone {
        ($sample:ty) => {
            build_tone::<$sample>(&device, config.into(), Arc::clone(&output_frames))?
        };
    }
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => tone!(f32),
        cpal::SampleFormat::F64 => tone!(f64),
        cpal::SampleFormat::I8 => tone!(i8),
        cpal::SampleFormat::I16 => tone!(i16),
        cpal::SampleFormat::I24 => tone!(cpal::I24),
        cpal::SampleFormat::I32 => tone!(i32),
        cpal::SampleFormat::I64 => tone!(i64),
        cpal::SampleFormat::U8 => tone!(u8),
        cpal::SampleFormat::U16 => tone!(u16),
        cpal::SampleFormat::U24 => tone!(cpal::U24),
        cpal::SampleFormat::U32 => tone!(u32),
        cpal::SampleFormat::U64 => tone!(u64),
        format => anyhow::bail!("Unsupported output format {format}"),
    };
    stream.play()?;
    let started = Instant::now();
    let mut peak = 0.0_f32;
    let mut seen_live = false;
    let mut next_report = Duration::from_millis(500);
    while started.elapsed() < Duration::from_secs(2) {
        seen_live |= audio.live();
        peak = audio
            .tap()
            .window(2048, 0)
            .into_iter()
            .map(f32::abs)
            .fold(peak, f32::max);
        if started.elapsed() >= next_report {
            println!(
                "At {:?}: status={:?}, live={}, tone_frames={}, {:?}",
                started.elapsed(),
                audio.status(),
                audio.live(),
                output_frames.load(Ordering::Relaxed),
                audio.diagnostics()
            );
            next_report += Duration::from_millis(500);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    println!(
        "Final capture status: {:?}; tone_frames={}; {:?}",
        audio.status(),
        output_frames.load(Ordering::Relaxed),
        audio.diagnostics()
    );
    drop(stream);
    audio.set_enabled(false);
    anyhow::ensure!(
        seen_live && peak > 0.000_01,
        "No non-silent loopback audio arrived: live={seen_live}, peak={peak}"
    );
    anyhow::ensure!(!audio.live(), "Disabled capture is still live");
    anyhow::ensure!(
        audio
            .tap()
            .window(2048, 0)
            .iter()
            .all(|sample| *sample == 0.0),
        "Disabled capture retained the last frame"
    );
    println!("PASS: captured output mix, peak {peak:.5}; disabling cleared the visualizer tap.");
    Ok(())
}

#[cfg(windows)]
fn build_tone<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    output_frames: std::sync::Arc<std::sync::atomic::AtomicU64>,
) -> Result<cpal::Stream, cpal::Error>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    use cpal::traits::DeviceTrait;
    let channels = usize::from(config.channels);
    let step = std::f32::consts::TAU * 440.0 / config.sample_rate as f32;
    let mut phase = 0.0_f32;
    device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            output_frames.fetch_add(
                (data.len() / channels) as u64,
                std::sync::atomic::Ordering::Relaxed,
            );
            for frame in data.chunks_mut(channels) {
                phase = (phase + step) % std::f32::consts::TAU;
                let sample = <T as cpal::FromSample<f32>>::from_sample_(phase.sin() * 0.02);
                frame.fill(sample);
            }
        },
        |error| eprintln!("Probe output error: {error}"),
        Some(std::time::Duration::from_secs(2)),
    )
}

#[cfg(not(windows))]
fn main() {
    println!("This output-loopback hardware diagnostic runs on Windows.");
}
