//! The app's side of the MilkDrop child process.
//!
//! `Host` starts this binary with `--milkdrop-child`, connects the audio tap to
//! shared memory, sends settings over stdin, and reads window state from stdout.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use serde::Deserialize;

use super::shm::Ring;
use crate::vis::AudioTap;

/// The tag the child puts before its event lines, so libprojectM's own
/// output on the same stream is easy to skip.
pub const EVENT_PREFIX: &str = "@MD@";

/// What the child reported back, gathered by the reader thread for the app to
/// take each frame.
#[derive(Default)]
struct Reported {
    closed: bool,
    error: Option<String>,
    visible: Option<bool>,
    pos: Option<[f32; 2]>,
    size: Option<[f32; 2]>,
    /// What the window's keys asked of the player, oldest first.
    commands: Vec<String>,
    screen_hz: Option<u32>,
}

/// One line of what the child says on its stdout.
#[derive(Deserialize)]
struct Event {
    closed: Option<bool>,
    error: Option<String>,
    visible: Option<bool>,
    pos: Option<[f32; 2]>,
    size: Option<[f32; 2]>,
    command: Option<String>,
    screen_hz: Option<u32>,
}

/// What the app takes from the window each frame.
pub struct Poll {
    /// The window closed itself (its X, Esc, or the user quit it).
    pub closed: bool,
    pub error: Option<String>,
    pub pos: Option<[f32; 2]>,
    pub size: Option<[f32; 2]>,
    /// What its keys asked of the player since the last look.
    pub commands: Vec<String>,
    /// How often the screen it is on refreshes, when it has said.
    pub screen_hz: Option<u32>,
}

struct Running {
    process: Child,
    ring: Arc<Ring>,
    visible: bool,
    stdin: std::process::ChildStdin,
    reported: Arc<Mutex<Reported>>,
    fps: u32,
    seconds: u32,
    scale: u32,
    /// The song last told to the window, which overlays a change.
    song: Option<Vec<String>>,
}

pub struct Host {
    tap: Arc<AudioTap>,
    lag: usize,
    /// Where the shared-memory ring file lives, one per app run.
    shm_path: PathBuf,
    running: Option<Running>,
}

impl Host {
    pub fn new(tap: Arc<AudioTap>) -> Self {
        let shm_path = std::env::temp_dir().join(format!("spotifast-milkdrop-{}.pcm", pid()));
        Self {
            tap,
            lag: super::LAG,
            shm_path,
            running: None,
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.is_some()
    }

    /// Whether the child's own window is visible, independent of the main
    /// window. The child reports minimisation and occlusion changes.
    pub fn is_visible(&self) -> bool {
        self.running.as_ref().is_some_and(|running| running.visible)
    }

    /// Select native post-EQ/pre-volume sound or Windows loopback. Source
    /// handoffs quiesce the previous producer before connecting the next.
    pub fn set_audio(&mut self, tap: Arc<AudioTap>, lag: usize) {
        let lag = lag.min(super::shm::FRAMES);
        if !Arc::ptr_eq(&self.tap, &tap) {
            handoff_audio(
                &self.tap,
                &tap,
                self.running.as_ref().map(|running| &running.ring),
            );
            self.tap = tap;
        }
        if self.lag != lag {
            self.lag = lag;
            if let Some(running) = &mut self.running {
                let line = format!("{{\"lag\":{lag}}}\n");
                if running.stdin.write_all(line.as_bytes()).is_err() {
                    self.tap.set_shm(None);
                }
            }
        }
    }

    /// Starts the child window, pointing the tap at a fresh ring.
    #[allow(clippy::too_many_arguments)]
    pub fn open(
        &mut self,
        presets: &Path,
        size: [f32; 2],
        pos: Option<[f32; 2]>,
        fullscreen: bool,
        fps: u32,
        seconds: u32,
        scale: u32,
    ) -> Result<(), String> {
        if self.running.is_some() {
            return Ok(());
        }
        // Old builds accidentally saved Retina physical pixels as logical
        // points. Do not reopen an enormous, unreachable window from that
        // corrupt geometry.
        let size = sane_size(size).unwrap_or(super::DEFAULT_SIZE);
        let pos = pos.filter(|pos| sane_position(*pos));
        let ring = match Ring::create(&self.shm_path) {
            Ok(ring) => Arc::new(ring),
            Err(error) => {
                return Err(format!(
                    "MilkDrop could not create its audio buffer: {error}"
                ));
            }
        };
        let exe = match std::env::current_exe() {
            Ok(exe) => exe,
            Err(error) => {
                return Err(format!("MilkDrop could not locate its executable: {error}"));
            }
        };
        let mut command = Command::new(exe);
        command
            .arg("--milkdrop-child")
            .arg("--milkdrop-shm")
            .arg(&self.shm_path)
            .arg("--milkdrop-presets")
            .arg(presets)
            .arg("--milkdrop-size")
            .arg(format!("{}x{}", size[0], size[1]))
            .arg("--milkdrop-fps")
            .arg(fps.to_string())
            .arg("--milkdrop-seconds")
            .arg(seconds.to_string())
            .arg("--milkdrop-scale")
            .arg(scale.to_string())
            .arg("--milkdrop-lag")
            .arg(self.lag.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        if let Some(pos) = pos {
            command
                .arg("--milkdrop-pos")
                .arg(format!("{},{}", pos[0], pos[1]));
        }
        if fullscreen {
            command.arg("--milkdrop-fullscreen");
        }
        let mut process = match command.spawn() {
            Ok(process) => process,
            Err(error) => {
                return Err(format!("MilkDrop could not start its window: {error}"));
            }
        };
        let stdin = process.stdin.take().expect("child stdin was piped");
        let stdout = process.stdout.take().expect("child stdout was piped");
        let reported = Arc::new(Mutex::new(Reported::default()));
        spawn_reader(stdout, Arc::clone(&reported));

        // The child keeps the ring mapped by its own path; this side keeps
        // writing the sound into it.
        self.tap.set_shm(Some(Arc::clone(&ring)));
        self.running = Some(Running {
            process,
            ring,
            visible: false,
            stdin,
            reported,
            fps,
            seconds,
            scale,
            song: None,
        });
        Ok(())
    }

    /// Sends new settings to the window, if they changed.
    pub fn update(&mut self, fps: u32, seconds: u32, scale: u32) {
        let Some(running) = &mut self.running else {
            return;
        };
        if running.fps == fps && running.seconds == seconds && running.scale == scale {
            return;
        }
        running.fps = fps;
        running.seconds = seconds;
        running.scale = scale;
        let line = format!("{{\"fps\":{fps},\"seconds\":{seconds},\"scale\":{scale}}}\n");
        if running.stdin.write_all(line.as_bytes()).is_err() {
            // The child is gone; the next poll will report it closed.
            self.tap.set_shm(None);
        }
    }

    /// Tells the window the playing song when it changes; the window
    /// overlays it, the way MilkDrop showed the title.
    pub fn song(&mut self, lines: Option<Vec<String>>) {
        let Some(running) = &mut self.running else {
            return;
        };
        let Some(lines) = lines else {
            return;
        };
        if running.song.as_ref() == Some(&lines) {
            return;
        }
        let Ok(value) = serde_json::to_string(&serde_json::json!({ "song": lines })) else {
            return;
        };
        running.song = Some(lines);
        if running.stdin.write_all((value + "\n").as_bytes()).is_err() {
            self.tap.set_shm(None);
        }
    }

    /// Takes what the window reported, and notices it closing on its own.
    pub fn poll(&mut self) -> Poll {
        let Some(running) = &mut self.running else {
            return Poll {
                closed: false,
                error: None,
                pos: None,
                size: None,
                commands: Vec::new(),
                screen_hz: None,
            };
        };
        // The child exiting is a close, even if it said nothing.
        let exit = running.process.try_wait().ok().flatten();
        let (closed, error, pos, size, commands, screen_hz) = {
            let mut reported = running.reported.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(visible) = reported.visible.take() {
                running.visible = visible;
            }
            let error = reported.error.take().or_else(|| {
                exit.filter(|status| !status.success()).map(|status| {
                    format!("MilkDrop's window could not start or stopped unexpectedly ({status}).")
                })
            });
            (
                reported.closed || exit.is_some(),
                error,
                reported.pos.take(),
                reported.size.take(),
                std::mem::take(&mut reported.commands),
                reported.screen_hz.take(),
            )
        };
        if closed {
            self.stop();
        }
        Poll {
            closed,
            error,
            pos,
            size,
            commands,
            screen_hz,
        }
    }

    /// Closes the window and reaps the child.
    pub fn close(&mut self) {
        if let Some(running) = &mut self.running {
            let _ = running.stdin.write_all(b"{\"close\":true}\n");
        }
        self.stop();
    }

    fn stop(&mut self) {
        self.tap.set_shm(None);
        if let Some(mut running) = self.running.take() {
            // Ask nicely, then make sure it is gone.
            drop(running.stdin);
            match running.process.try_wait() {
                Ok(Some(_)) => {}
                _ => {
                    let _ = running.process.kill();
                    let _ = running.process.wait();
                }
            }
        }
        let _ = std::fs::remove_file(&self.shm_path);
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Reads the child's stdout, keeping the latest of what it reports and
/// skipping libprojectM's own output on the same stream.
fn spawn_reader(stdout: std::process::ChildStdout, reported: Arc<Mutex<Reported>>) {
    std::thread::Builder::new()
        .name("milkdrop-reader".into())
        .spawn(move || {
            let mut lines = std::io::BufReader::new(stdout).lines();
            for line in lines.by_ref() {
                let Ok(line) = line else { break };
                let Some(json) = line.strip_prefix(EVENT_PREFIX) else {
                    continue;
                };
                let Ok(event) = serde_json::from_str::<Event>(json) else {
                    continue;
                };
                let mut reported = reported.lock().unwrap_or_else(|p| p.into_inner());
                if event.closed == Some(true) {
                    reported.closed = true;
                }
                if event.error.is_some() {
                    reported.error = event.error;
                }
                if event.visible.is_some() {
                    reported.visible = event.visible;
                }
                if event.pos.is_some() {
                    reported.pos = event.pos;
                }
                if event.size.is_some() {
                    reported.size = event.size;
                }
                if event.screen_hz.is_some() {
                    reported.screen_hz = event.screen_hz;
                }
                if let Some(command) = event.command {
                    // A key held down cannot pile up more than a moment's
                    // worth before the app takes them.
                    if reported.commands.len() < 64 {
                        reported.commands.push(command);
                    }
                }
            }
            // Stdout closed: the child is gone.
            reported.lock().unwrap_or_else(|p| p.into_inner()).closed = true;
        })
        .ok();
}

/// Disconnect and quiesce the previous audio callback before the new one
/// can write. The ring has exactly one producer, including during handoff.
fn handoff_audio(previous: &AudioTap, next: &AudioTap, ring: Option<&Arc<Ring>>) {
    previous.set_shm(None);
    if let Some(ring) = ring {
        // Keep the child's cursor monotonic while replacing retained PCM.
        ring.push(&vec![0.0; super::shm::FRAMES * 2]);
        next.set_shm(Some(Arc::clone(ring)));
    }
}

fn pid() -> u32 {
    std::process::id()
}

fn sane_size([width, height]: [f32; 2]) -> Option<[f32; 2]> {
    (width.is_finite()
        && height.is_finite()
        && (super::MIN_SIZE[0]..=8192.0).contains(&width)
        && (super::MIN_SIZE[1]..=8192.0).contains(&height))
    .then_some([width, height])
}

fn sane_position([x, y]: [f32; 2]) -> bool {
    x.is_finite() && y.is_finite() && x.abs() <= 100_000.0 && y.abs() <= 100_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_active_previous_source_cannot_write_after_the_handoff() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let path = std::env::temp_dir().join(format!("motion-milkdrop-handoff-{}", pid()));
        let ring = Arc::new(Ring::create(&path).unwrap());
        let reader = Ring::open(&path).unwrap();
        let previous = AudioTap::new();
        let next = AudioTap::new();
        previous.set_shm(Some(Arc::clone(&ring)));
        let keep_writing = Arc::new(AtomicBool::new(true));
        let callback_tap = Arc::clone(&previous);
        let callback_wanted = Arc::clone(&keep_writing);
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let callback = std::thread::spawn(move || {
            callback_tap.push(&[0.875, -0.875], 1.0);
            ready_tx.send(()).unwrap();
            while callback_wanted.load(Ordering::Acquire) {
                callback_tap.push(&[0.875, -0.875], 1.0);
            }
        });
        ready_rx.recv().unwrap();
        handoff_audio(&previous, &next, Some(&ring));
        next.push(&[0.25, -0.5], 1.0);
        // The old callback remains alive during the new source's write.
        keep_writing.store(false, Ordering::Release);
        callback.join().unwrap();
        let frames = reader.since(&mut 0, 0);
        assert_eq!(frames.last(), Some(&[0.25, -0.5]));
        assert!(
            frames[..frames.len() - 1]
                .iter()
                .all(|frame| *frame == [0.0, 0.0]),
            "no previous-source PCM survives or arrives after handoff"
        );
        next.set_shm(None);
        drop((previous, next, reader, ring));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn child_events_preserve_actionable_errors_and_independent_visibility() {
        let event: Event = serde_json::from_str(
            r#"{"closed":true,"error":"MilkDrop needs OpenGL 3.3","visible":false}"#,
        )
        .unwrap();
        assert_eq!(event.closed, Some(true));
        assert_eq!(event.visible, Some(false));
        assert_eq!(event.error.as_deref(), Some("MilkDrop needs OpenGL 3.3"));
        let event: Event = serde_json::from_str(r#"{"visible":true}"#).unwrap();
        assert_eq!(event.visible, Some(true));
        assert!(event.error.is_none());
        assert!(event.closed.is_none());
    }

    #[test]
    fn corrupt_saved_window_geometry_is_rejected() {
        assert_eq!(sane_size([640.0, 480.0]), Some([640.0, 480.0]));
        assert_eq!(sane_size([2_621_440.0, 1_966_080.0]), None);
        assert!(sane_position([-1920.0, 40.0]));
        assert!(!sane_position([553_984.0, 1_042_432.0]));
    }
}
