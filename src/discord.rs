//! Optional Discord Rich Presence over the desktop client's local IPC socket.
//!
//! This module never authenticates to Discord or handles an account token.
//! Only an explicitly enabled public Application ID and now-playing metadata
//! reach the local client. Socket IO and reconnects run on a background thread.

use std::collections::VecDeque;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::watch;
use tokio::time::Instant;

const MAX_FRAME: usize = 65_536;
const IO_TIMEOUT: Duration = Duration::from_secs(2);
const READY_TIMEOUT: Duration = Duration::from_secs(3);
const RETRY: Duration = Duration::from_secs(5);
const MIN_UPDATE: Duration = Duration::from_secs(5);
const RATE_WINDOW: Duration = Duration::from_secs(20);
const RATE_COUNT: usize = 5;
const HANDSHAKE: u32 = 0;
const FRAME: u32 = 1;
const CLOSE: u32 = 2;
const PING: u32 = 3;
const PONG: u32 = 4;

/// Metadata for a custom Listening activity, not Spotify's partner activity.
/// Its timestamp fields are Unix seconds, as required by the local RPC API.
#[derive(Clone, PartialEq, Eq)]
pub struct Activity {
    pub details: String,
    pub state: Option<String>,
    pub large_image: Option<String>,
    pub large_text: Option<String>,
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub url: Option<String>,
}

fn short_text(value: &str, limit: usize) -> String {
    let value = value.trim();
    let mut end = value.len().min(limit);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].chars().filter(|c| !c.is_control()).collect()
}

fn public_image(value: &str) -> Option<String> {
    let url = reqwest::Url::parse(value).ok()?;
    (url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && matches!(
            url.host_str(),
            Some(
                "i.scdn.co"
                    | "mosaic.scdn.co"
                    | "image-cdn-ak.spotifycdn.com"
                    | "image-cdn-fa.spotifycdn.com"
            )
        )
        && value.len() <= 2048)
        .then(|| value.to_owned())
}

fn spotify_link(value: &str) -> Option<String> {
    let url = reqwest::Url::parse(value).ok()?;
    (url.scheme() == "https"
        && url.host_str() == Some("open.spotify.com")
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && value.len() <= 512
        && (url.path().starts_with("/track/") || url.path().starts_with("/episode/")))
    .then(|| value.to_owned())
}

impl Activity {
    fn payload(&self) -> Value {
        let mut result = json!({"type":2,"details":short_text(&self.details,128)});
        if let Some(state) = self
            .state
            .as_deref()
            .map(|s| short_text(s, 128))
            .filter(|s| !s.is_empty())
        {
            result["state"] = state.into();
        }
        if let Some(image) = self.large_image.as_deref().and_then(public_image) {
            result["assets"] = json!({"large_image":image});
            if let Some(text) = self
                .large_text
                .as_deref()
                .map(|s| short_text(s, 128))
                .filter(|s| !s.is_empty())
            {
                result["assets"]["large_text"] = text.into();
            }
        }
        if let Some(start) = self.start.filter(|&n| n >= 0) {
            result["timestamps"] = json!({"start":start});
            if let Some(end) = self.end.filter(|&n| n >= start) {
                result["timestamps"]["end"] = end.into();
            }
        }
        if let Some(url) = self.url.as_deref().and_then(spotify_link) {
            result["buttons"] = json!([{"label":"Open in Spotify","url":url}]);
        }
        result
    }
}

/// Decimal public application identifiers only. Never accept a Discord token.
pub fn valid_application_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 20
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && value.parse::<u64>().is_ok_and(|number| number != 0)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Disabled,
    Waiting,
    Connecting,
    Connected,
    Error(String),
}

#[derive(Clone, PartialEq, Eq)]
enum Desired {
    Disabled,
    Invalid,
    Enabled {
        application_id: String,
        activity: Option<Activity>,
    },
    Shutdown,
}

#[derive(Default)]
struct Shared {
    status: Mutex<Status>,
    repaint: Mutex<Option<egui::Context>>,
}

impl Shared {
    fn status(&self, status: Status) {
        let mut old = self.status.lock().unwrap_or_else(|p| p.into_inner());
        if *old == status {
            return;
        }
        *old = status;
        drop(old);
        if let Some(ctx) = self
            .repaint
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
        {
            ctx.request_repaint();
        }
    }
}

struct Worker {
    desired: watch::Sender<Desired>,
    // Dropping a JoinHandle detaches; neither UI updates nor Drop wait on IO.
    _thread: std::thread::JoinHandle<()>,
}

/// Inert until enabled with a valid public Application ID. Demo/test callers
/// should not call update with real metadata or an enabled application ID.
#[derive(Default)]
pub struct Client {
    shared: Arc<Shared>,
    worker: Option<Worker>,
}

impl Client {
    pub fn attach(&self, ctx: &egui::Context) {
        *self
            .shared
            .repaint
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = Some(ctx.clone());
    }

    pub fn update(&mut self, application_id: Option<&str>, activity: Option<Activity>) {
        let desired = match application_id {
            None => Desired::Disabled,
            Some(id) if !valid_application_id(id) => Desired::Invalid,
            Some(id) => Desired::Enabled {
                application_id: id.to_owned(),
                activity,
            },
        };
        if let Some(worker) = &self.worker {
            worker.desired.send_if_modified(|old| {
                if *old == desired {
                    false
                } else {
                    *old = desired;
                    true
                }
            });
            return;
        }
        match desired {
            Desired::Disabled => self.shared.status(Status::Disabled),
            Desired::Invalid => self.shared.status(Status::Error(
                "Enter a valid Discord Application ID.".into(),
            )),
            Desired::Enabled { .. } => {
                let (sender, receiver) = watch::channel(desired);
                let shared = Arc::clone(&self.shared);
                match std::thread::Builder::new()
                    .name("discord-presence".into())
                    .spawn(move || {
                        match tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        {
                            Ok(runtime) => runtime.block_on(run(receiver, shared)),
                            Err(_) => shared
                                .status(Status::Error("Discord sharing could not start.".into())),
                        }
                    }) {
                    Ok(thread) => {
                        self.worker = Some(Worker {
                            desired: sender,
                            _thread: thread,
                        })
                    }
                    Err(_) => self
                        .shared
                        .status(Status::Error("Discord sharing could not start.".into())),
                }
            }
            Desired::Shutdown => {}
        }
    }

    pub fn status(&self) -> Status {
        self.shared
            .status
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }

    /// Requests a bounded best-effort clear and drops the socket off-thread.
    pub fn shutdown(&mut self) {
        if let Some(worker) = self.worker.take() {
            worker.desired.send_replace(Desired::Shutdown);
        }
        self.shared.status(Status::Disabled);
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.shutdown();
    }
}

trait Socket: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> Socket for T {}
type Stream = Box<dyn Socket>;

#[cfg(windows)]
async fn connect() -> io::Result<Stream> {
    use tokio::net::windows::named_pipe::ClientOptions;
    for index in 0..10 {
        if let Ok(pipe) = ClientOptions::new().open(format!(r"\\?\pipe\discord-ipc-{index}")) {
            return Ok(Box::new(pipe));
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Discord is not running",
    ))
}

#[cfg(unix)]
async fn connect() -> io::Result<Stream> {
    let mut prefixes = Vec::new();
    for variable in ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"] {
        if let Some(directory) = std::env::var_os(variable).map(std::path::PathBuf::from)
            && directory.is_absolute()
            && !prefixes.contains(&directory)
        {
            prefixes.push(directory);
        }
    }
    prefixes.push(std::path::PathBuf::from("/tmp"));
    for directory in prefixes {
        for sub in ["", "app/com.discordapp.Discord", "snap.discord"] {
            for index in 0..10 {
                let path = directory.join(sub).join(format!("discord-ipc-{index}"));
                if let Ok(Ok(socket)) = tokio::time::timeout(
                    Duration::from_millis(100),
                    tokio::net::UnixStream::connect(path),
                )
                .await
                {
                    return Ok(Box::new(socket));
                }
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Discord is not running",
    ))
}

#[cfg(not(any(windows, unix)))]
async fn connect() -> io::Result<Stream> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Discord desktop IPC is unavailable",
    ))
}

struct Framed<S> {
    stream: S,
    received: Vec<u8>,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Framed<S> {
    fn new(stream: S) -> Self {
        Self {
            stream,
            received: Vec::new(),
        }
    }

    // The accumulated bytes live outside the awaitable: cancelling a read
    // for a new desired state never loses a partial header or JSON payload.
    async fn read(&mut self) -> io::Result<(u32, Vec<u8>)> {
        loop {
            if self.received.len() >= 8 {
                let opcode = u32::from_le_bytes(self.received[..4].try_into().unwrap());
                let length = u32::from_le_bytes(self.received[4..8].try_into().unwrap()) as usize;
                if length > MAX_FRAME {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Discord frame too large",
                    ));
                }
                if self.received.len() >= length + 8 {
                    let payload = self.received[8..length + 8].to_vec();
                    self.received.drain(..length + 8);
                    return Ok((opcode, payload));
                }
            }
            let mut bytes = [0; 4096];
            let count = self.stream.read(&mut bytes).await?;
            if count == 0 {
                return Err(io::ErrorKind::UnexpectedEof.into());
            }
            self.received.extend_from_slice(&bytes[..count]);
        }
    }

    async fn write(&mut self, opcode: u32, payload: &[u8]) -> io::Result<()> {
        if payload.len() > MAX_FRAME {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        let mut bytes = Vec::with_capacity(payload.len() + 8);
        bytes.extend_from_slice(&opcode.to_le_bytes());
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(payload);
        tokio::time::timeout(IO_TIMEOUT, self.stream.write_all(&bytes))
            .await
            .map_err(|_| io::ErrorKind::TimedOut)?
    }
}

#[derive(Default)]
struct Rate {
    sent: VecDeque<Instant>,
    last_activity: Option<Instant>,
}

impl Rate {
    fn due(&mut self, now: Instant, clearing: bool) -> Instant {
        while self
            .sent
            .front()
            .is_some_and(|at| now.duration_since(*at) >= RATE_WINDOW)
        {
            self.sent.pop_front();
        }
        let window = if self.sent.len() >= RATE_COUNT {
            self.sent[0] + RATE_WINDOW
        } else {
            now
        };
        if clearing {
            window
        } else {
            window.max(self.last_activity.map_or(now, |at| at + MIN_UPDATE))
        }
    }
    fn note(&mut self, now: Instant, clearing: bool) {
        self.sent.push_back(now);
        if !clearing {
            self.last_activity = Some(now);
        }
    }
}

fn nonce() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    format!(
        "{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

fn command(activity: Option<&Activity>, nonce: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({"cmd":"SET_ACTIVITY","args":{"pid":std::process::id(),"activity":activity.map(Activity::payload)},"nonce":nonce})).expect("a finite Discord activity serializes")
}

async fn clear<S: AsyncRead + AsyncWrite + Unpin>(socket: &mut Framed<S>, rate: &mut Rate) {
    let now = Instant::now();
    if rate.due(now, true) <= now {
        rate.note(now, true);
        // Closing the IPC connection also removes its presence. A stalled
        // client must never hold teardown or an opt-out beyond this bound.
        let _ = tokio::time::timeout(
            Duration::from_millis(250),
            socket.write(FRAME, &command(None, &nonce())),
        )
        .await;
    }
}

enum Exit {
    Changed,
    Lost,
    Rejected,
}

async fn session<S: AsyncRead + AsyncWrite + Unpin>(
    stream: S,
    id: &str,
    desired: &mut watch::Receiver<Desired>,
    shared: &Shared,
    rate: &mut Rate,
) -> Exit {
    let mut socket = Framed::new(stream);
    let handshake = serde_json::to_vec(&json!({"v":1,"client_id":id})).expect("handshake JSON");
    if socket.write(HANDSHAKE, &handshake).await.is_err() {
        return Exit::Lost;
    }
    let ready_by = Instant::now() + READY_TIMEOUT;
    loop {
        tokio::select! {
            biased;
            result = desired.changed() => { let _ = result; return Exit::Changed; }
            _ = tokio::time::sleep_until(ready_by) => return Exit::Lost,
            incoming = socket.read() => match incoming {
                Ok((PING,bytes)) => if socket.write(PONG,&bytes).await.is_err() { return Exit::Lost; },
                Ok((FRAME,bytes)) => {
                    let Ok(value) = serde_json::from_slice::<Value>(&bytes) else { return Exit::Lost; };
                    if value["evt"]=="ERROR" { return Exit::Rejected; }
                    if value["cmd"]=="DISPATCH" && value["evt"]=="READY" { break; }
                }
                Ok((PONG,_)) => {},
                _ => return Exit::Lost,
            }
        }
    }
    let mut last: Option<Option<Activity>> = None;
    let mut waiting: Option<(String, Instant)> = None;
    loop {
        let current = desired.borrow_and_update().clone();
        let activity = match current {
            Desired::Enabled {
                application_id,
                activity,
            } if application_id == id => activity,
            _ => {
                clear(&mut socket, rate).await;
                return Exit::Changed;
            }
        };
        let now = Instant::now();
        let changed = last.as_ref() != Some(&activity);
        let due = rate.due(now, activity.is_none());
        // A clear which would exceed Discord's rolling limit instead closes
        // the socket immediately; no stale activity survives an opt-out.
        if changed && activity.is_none() && due > now {
            return Exit::Changed;
        }
        if changed && (waiting.is_none() || activity.is_none()) && due <= now {
            let number = nonce();
            if socket
                .write(FRAME, &command(activity.as_ref(), &number))
                .await
                .is_err()
            {
                return Exit::Lost;
            }
            rate.note(now, activity.is_none());
            last = Some(activity);
            waiting = Some((number, Instant::now() + IO_TIMEOUT));
        }
        let wake = waiting.as_ref().map_or_else(
            || {
                if changed {
                    due
                } else {
                    now + Duration::from_secs(86_400)
                }
            },
            |(_, at)| *at,
        );
        tokio::select! {
            biased;
            result = desired.changed() => {
                if result.is_err() { clear(&mut socket,rate).await; return Exit::Changed; }
            }
            _ = tokio::time::sleep_until(wake) => if waiting.is_some() { return Exit::Lost; },
            incoming = socket.read() => match incoming {
                Ok((PING,bytes)) => if socket.write(PONG,&bytes).await.is_err() { return Exit::Lost; },
                Ok((FRAME,bytes)) => {
                    let Ok(value) = serde_json::from_slice::<Value>(&bytes) else { return Exit::Lost; };
                    if value["evt"]=="ERROR" { return Exit::Rejected; }
                    if let Some((number,_))=&waiting
                        && value["cmd"]=="SET_ACTIVITY" && value["nonce"].as_str()==Some(number.as_str()) {
                        waiting=None;
                        shared.status(Status::Connected);
                    }
                }
                Ok((PONG,_)) => {},
                Ok((CLOSE,_)) => return Exit::Lost,
                _ => return Exit::Lost,
            }
        }
    }
}

async fn run(desired: watch::Receiver<Desired>, shared: Arc<Shared>) {
    run_with(desired, shared, connect).await;
}

async fn run_with<F, Fut>(
    mut desired: watch::Receiver<Desired>,
    shared: Arc<Shared>,
    mut connect: F,
) where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = io::Result<Stream>>,
{
    let mut rate = Rate::default();
    loop {
        let current = desired.borrow_and_update().clone();
        let id = match current {
            Desired::Shutdown => {
                shared.status(Status::Disabled);
                return;
            }
            Desired::Disabled | Desired::Invalid => {
                shared.status(if current == Desired::Invalid {
                    Status::Error("Enter a valid Discord Application ID.".into())
                } else {
                    Status::Disabled
                });
                if desired.changed().await.is_err() {
                    return;
                }
                continue;
            }
            Desired::Enabled { application_id, .. } => application_id,
        };
        shared.status(Status::Waiting);
        let connected = tokio::select! {
            biased;
            result=desired.changed() => { if result.is_err() {return;} continue; },
            result=tokio::time::timeout(READY_TIMEOUT,connect()) => result,
        };
        if let Ok(Ok(stream)) = connected {
            shared.status(Status::Connecting);
            match session(stream, &id, &mut desired, &shared, &mut rate).await {
                Exit::Changed => {
                    // A rate-limited clear closes instead. Wait for the next
                    // permitted command before opening an idle socket again.
                    let current = desired.borrow().clone();
                    if matches!(current, Desired::Enabled { activity: None, .. }) {
                        let due = rate.due(Instant::now(), true);
                        tokio::select! { _=tokio::time::sleep_until(due)=>{},result=desired.changed()=>if result.is_err(){return;} }
                    }
                    continue;
                }
                Exit::Rejected => shared.status(Status::Error(
                    "Discord rejected the activity. Check the Application ID.".into(),
                )),
                Exit::Lost => shared.status(Status::Waiting),
            }
        }
        tokio::select! {
            _=tokio::time::sleep(RETRY)=>{},
            result=desired.changed()=>if result.is_err(){return;},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{DuplexStream, duplex};

    const ID: &str = "123456789012345678";

    fn song(title: &str) -> Activity {
        Activity {
            details: title.into(),
            state: Some("A musician".into()),
            large_image: Some("https://i.scdn.co/image/cover".into()),
            large_text: Some("An album".into()),
            start: Some(1_800_000_000),
            end: Some(1_800_000_180),
            url: Some("https://open.spotify.com/track/track123".into()),
        }
    }
    fn enabled(activity: Option<Activity>) -> Desired {
        Desired::Enabled {
            application_id: ID.into(),
            activity,
        }
    }
    fn decoded(bytes: &[u8]) -> Value {
        serde_json::from_slice(bytes).unwrap()
    }
    async fn message(peer: &mut Framed<DuplexStream>) -> Value {
        let (opcode, bytes) = peer.read().await.unwrap();
        assert_eq!(opcode, FRAME);
        decoded(&bytes)
    }
    async fn ready(peer: &mut Framed<DuplexStream>) {
        let (opcode, bytes) = peer.read().await.unwrap();
        assert_eq!(opcode, HANDSHAKE);
        assert_eq!(decoded(&bytes), json!({"v":1,"client_id":ID}));
        peer.write(FRAME, br#"{"cmd":"DISPATCH","evt":"READY","data":{}}"#)
            .await
            .unwrap();
    }
    async fn ack(peer: &mut Framed<DuplexStream>, command: &Value) {
        peer.write(
            FRAME,
            &serde_json::to_vec(&json!({"cmd":"SET_ACTIVITY","nonce":command["nonce"],"data":{}}))
                .unwrap(),
        )
        .await
        .unwrap();
        tokio::task::yield_now().await;
    }

    #[test]
    fn clients_are_inert_until_a_valid_public_application_is_enabled() {
        let mut client = Client::default();
        assert_eq!(client.status(), Status::Disabled);
        assert!(client.worker.is_none());
        client.update(None, Some(song("Unshared song")));
        assert!(client.worker.is_none());
        for invalid in [
            "",
            "0",
            "000",
            "a token",
            "123/456",
            "18446744073709551616",
            "123456789012345678901",
        ] {
            assert!(!valid_application_id(invalid));
            client.update(Some(invalid), None);
            assert!(matches!(client.status(), Status::Error(_)));
            assert!(client.worker.is_none());
        }
        assert!(valid_application_id(ID));
        assert!(valid_application_id("1"));
        client.shutdown();
        assert_eq!(client.status(), Status::Disabled);
    }

    #[test]
    fn listening_payload_uses_seconds_utf8_limits_and_only_public_spotify_art() {
        let mut activity = song(&"🦊".repeat(100));
        activity.state = Some("one\ntwo\rthree".into());
        let value = activity.payload();
        assert_eq!(value["type"], 2);
        assert_eq!(value["details"].as_str().unwrap().len(), 128);
        assert_eq!(value["state"], "onetwothree");
        assert_eq!(value["timestamps"]["start"], 1_800_000_000_i64);
        assert_eq!(value["timestamps"]["end"], 1_800_000_180_i64);
        assert_eq!(value["buttons"][0]["label"], "Open in Spotify");
        for invalid in [
            "file:///C:/private/cover.png",
            "http://i.scdn.co/image/a",
            "https://localhost/cover",
            "https://password@i.scdn.co/image/a",
            "https://i.scdn.co.evil.example/a",
        ] {
            activity.large_image = Some(invalid.into());
            assert!(activity.payload().get("assets").is_none());
        }
        activity.url = Some("https://private.example/secret".into());
        assert!(activity.payload().get("buttons").is_none());
        activity.end = Some(1);
        assert!(activity.payload()["timestamps"].get("end").is_none());
    }

    #[tokio::test(start_paused = true)]
    async fn fragmented_and_cancelled_reads_keep_every_header_and_payload_byte() {
        let (client, mut peer) = duplex(64);
        let mut client = Framed::new(client);
        let payload = br#"{"evt":"READY"}"#;
        let mut bytes = FRAME.to_le_bytes().to_vec();
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(payload);
        peer.write_all(&bytes[..3]).await.unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(1), client.read())
                .await
                .is_err()
        );
        assert_eq!(client.received.len(), 3);
        peer.write_all(&bytes[3..12]).await.unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(1), client.read())
                .await
                .is_err()
        );
        assert_eq!(client.received.len(), 12);
        peer.write_all(&bytes[12..]).await.unwrap();
        let (opcode, received) = client.read().await.unwrap();
        assert_eq!(opcode, FRAME);
        assert_eq!(received, payload);
    }

    #[tokio::test]
    async fn oversized_frames_fail_before_allocating_the_announced_body() {
        let (client, mut peer) = duplex(64);
        let mut client = Framed::new(client);
        peer.write_all(&FRAME.to_le_bytes()).await.unwrap();
        peer.write_all(&u32::MAX.to_le_bytes()).await.unwrap();
        assert_eq!(
            client.read().await.unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(client.received.len(), 8);
    }

    #[tokio::test(start_paused = true)]
    async fn ready_precedes_activity_and_only_its_matching_ack_confirms_connection() {
        let (client, peer) = duplex(4096);
        let mut peer = Framed::new(peer);
        let (sender, mut receiver) = watch::channel(enabled(Some(song("First song"))));
        let shared = Arc::new(Shared::default());
        shared.status(Status::Connecting);
        let state = Arc::clone(&shared);
        let task = tokio::spawn(async move {
            session(client, ID, &mut receiver, &state, &mut Rate::default()).await
        });
        let (opcode, bytes) = peer.read().await.unwrap();
        assert_eq!(opcode, HANDSHAKE);
        assert_eq!(decoded(&bytes)["client_id"], ID);
        assert!(
            tokio::time::timeout(Duration::from_millis(10), peer.read())
                .await
                .is_err()
        );
        peer.write(PING, b"raw ping bytes").await.unwrap();
        assert_eq!(
            peer.read().await.unwrap(),
            (PONG, b"raw ping bytes".to_vec())
        );
        peer.write(FRAME, br#"{"cmd":"DISPATCH","evt":"READY","data":{}}"#)
            .await
            .unwrap();
        let command = message(&mut peer).await;
        assert_eq!(command["args"]["pid"], std::process::id());
        assert_eq!(command["args"]["activity"]["details"], "First song");
        assert_eq!(command["args"]["activity"]["type"], 2);
        peer.write(
            FRAME,
            br#"{"cmd":"SET_ACTIVITY","nonce":"wrong","data":{}}"#,
        )
        .await
        .unwrap();
        tokio::task::yield_now().await;
        assert_eq!(*shared.status.lock().unwrap(), Status::Connecting);
        ack(&mut peer, &command).await;
        assert_eq!(*shared.status.lock().unwrap(), Status::Connected);
        sender.send_replace(enabled(None));
        let cleared = message(&mut peer).await;
        assert!(cleared["args"]["activity"].is_null());
        ack(&mut peer, &cleared).await;
        sender.send_replace(Desired::Shutdown);
        assert!(message(&mut peer).await["args"]["activity"].is_null());
        assert!(matches!(task.await.unwrap(), Exit::Changed));
    }

    #[tokio::test(start_paused = true)]
    async fn missing_ready_and_activity_acks_have_finite_deadlines() {
        for ready_reply in [false, true] {
            let (client, peer) = duplex(4096);
            let mut peer = Framed::new(peer);
            let (_sender, mut receiver) = watch::channel(enabled(Some(song("Never confirmed"))));
            let shared = Shared::default();
            let task = tokio::spawn(async move {
                session(client, ID, &mut receiver, &shared, &mut Rate::default()).await
            });
            if ready_reply {
                ready(&mut peer).await;
                let _ = message(&mut peer).await;
            } else {
                let _ = peer.read().await.unwrap();
            }
            assert!(matches!(task.await.unwrap(), Exit::Lost));
        }
    }

    #[tokio::test(start_paused = true)]
    async fn pause_supersedes_an_unacknowledged_activity_immediately() {
        let (client, peer) = duplex(4096);
        let mut peer = Framed::new(peer);
        let (sender, mut receiver) = watch::channel(enabled(Some(song("Still pending"))));
        let shared = Arc::new(Shared::default());
        shared.status(Status::Connecting);
        let state = Arc::clone(&shared);
        let task = tokio::spawn(async move {
            session(client, ID, &mut receiver, &state, &mut Rate::default()).await
        });
        ready(&mut peer).await;
        let pending = message(&mut peer).await;
        sender.send_replace(enabled(None));
        let cleared = message(&mut peer).await;
        assert!(cleared["args"]["activity"].is_null());
        assert_ne!(pending["nonce"], cleared["nonce"]);
        ack(&mut peer, &pending).await;
        assert_eq!(*shared.status.lock().unwrap(), Status::Connecting);
        ack(&mut peer, &cleared).await;
        assert_eq!(*shared.status.lock().unwrap(), Status::Connected);
        sender.send_replace(Desired::Shutdown);
        let _ = message(&mut peer).await;
        assert!(matches!(task.await.unwrap(), Exit::Changed));
    }

    #[tokio::test(start_paused = true)]
    async fn command_rejection_never_becomes_a_connected_status() {
        let (client, peer) = duplex(4096);
        let mut peer = Framed::new(peer);
        let (_sender, mut receiver) = watch::channel(enabled(Some(song("Rejected"))));
        let shared = Arc::new(Shared::default());
        shared.status(Status::Connecting);
        let state = Arc::clone(&shared);
        let task = tokio::spawn(async move {
            session(client, ID, &mut receiver, &state, &mut Rate::default()).await
        });
        ready(&mut peer).await;
        let command = message(&mut peer).await;
        peer.write(FRAME,&serde_json::to_vec(&json!({"cmd":"SET_ACTIVITY","nonce":command["nonce"],"evt":"ERROR","data":{"code":4000,"message":"Do not expose remote text"}})).unwrap()).await.unwrap();
        assert!(matches!(task.await.unwrap(), Exit::Rejected));
        assert_eq!(*shared.status.lock().unwrap(), Status::Connecting);
    }

    #[tokio::test(start_paused = true)]
    async fn rapid_track_changes_coalesce_to_the_latest_after_the_rate_gate() {
        let (client, peer) = duplex(4096);
        let mut peer = Framed::new(peer);
        let (sender, mut receiver) = watch::channel(enabled(Some(song("First"))));
        let task = tokio::spawn(async move {
            session(
                client,
                ID,
                &mut receiver,
                &Shared::default(),
                &mut Rate::default(),
            )
            .await
        });
        ready(&mut peer).await;
        let first = message(&mut peer).await;
        ack(&mut peer, &first).await;
        for title in ["Skipped A", "Skipped B", "Current song"] {
            sender.send_replace(enabled(Some(song(title))));
        }
        assert!(
            tokio::time::timeout(Duration::from_secs(1), peer.read())
                .await
                .is_err()
        );
        tokio::time::advance(MIN_UPDATE).await;
        let latest = message(&mut peer).await;
        assert_eq!(latest["args"]["activity"]["details"], "Current song");
        ack(&mut peer, &latest).await;
        sender.send_replace(Desired::Disabled);
        let _ = message(&mut peer).await;
        assert!(matches!(task.await.unwrap(), Exit::Changed));
    }

    #[tokio::test(start_paused = true)]
    async fn pause_closes_instead_of_exceeding_the_rolling_discord_limit() {
        let (client, peer) = duplex(4096);
        let mut peer = Framed::new(peer);
        let (sender, mut receiver) = watch::channel(enabled(Some(song("Last allowed update"))));
        let now = Instant::now();
        let mut rate = Rate {
            sent: VecDeque::from([now; 4]),
            last_activity: None,
        };
        let task = tokio::spawn(async move {
            let result = session(client, ID, &mut receiver, &Shared::default(), &mut rate).await;
            (result, rate.sent.len())
        });
        ready(&mut peer).await;
        let command = message(&mut peer).await;
        ack(&mut peer, &command).await;
        sender.send_replace(enabled(None));
        let (exit, count) = task.await.unwrap();
        assert!(matches!(exit, Exit::Changed));
        assert_eq!(count, 5);
        assert_eq!(
            peer.read().await.unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
    }

    #[test]
    fn rolling_budget_does_not_reset_on_clear_or_connection_replacement() {
        let at = Instant::now();
        let mut rate = Rate::default();
        for index in 0..5 {
            let now = at + Duration::from_secs(index);
            assert!(rate.due(now, true) <= now);
            rate.note(now, true);
        }
        assert_eq!(
            rate.due(at + Duration::from_secs(5), true),
            at + RATE_WINDOW
        );
        assert_eq!(rate.due(at + RATE_WINDOW, true), at + RATE_WINDOW);
        assert_eq!(rate.sent.len(), 4);
    }

    #[tokio::test(start_paused = true)]
    async fn reconnect_resends_only_the_current_song_and_disabled_never_reconnects() {
        let (client1, peer1) = duplex(4096);
        let (client2, peer2) = duplex(4096);
        let mut peer1 = Framed::new(peer1);
        let mut peer2 = Framed::new(peer2);
        let sockets = Arc::new(Mutex::new(VecDeque::from([
            Box::new(client1) as Stream,
            Box::new(client2) as Stream,
        ])));
        let attempts = Arc::new(AtomicU64::new(0));
        let recorded = Arc::clone(&attempts);
        let connector = move || {
            recorded.fetch_add(1, Ordering::Relaxed);
            std::future::ready(
                sockets
                    .lock()
                    .unwrap()
                    .pop_front()
                    .ok_or_else(|| io::ErrorKind::NotFound.into()),
            )
        };
        let (sender, receiver) = watch::channel(enabled(Some(song("Before disconnect"))));
        let shared = Arc::new(Shared::default());
        let task = tokio::spawn(run_with(receiver, shared, connector));
        ready(&mut peer1).await;
        let first = message(&mut peer1).await;
        ack(&mut peer1, &first).await;
        drop(peer1);
        tokio::task::yield_now().await;
        sender.send_replace(enabled(Some(song("After disconnect"))));
        ready(&mut peer2).await;
        tokio::time::advance(MIN_UPDATE).await;
        let latest = message(&mut peer2).await;
        assert_eq!(latest["args"]["activity"]["details"], "After disconnect");
        ack(&mut peer2, &latest).await;
        sender.send_replace(Desired::Disabled);
        assert!(message(&mut peer2).await["args"]["activity"].is_null());
        tokio::task::yield_now().await;
        let count = attempts.load(Ordering::Relaxed);
        tokio::time::advance(Duration::from_secs(60)).await;
        assert_eq!(attempts.load(Ordering::Relaxed), count);
        sender.send_replace(Desired::Shutdown);
        task.await.unwrap();
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn native_windows_pipe_carries_a_real_handshake_activity_and_clear() {
        use tokio::net::windows::named_pipe::{ClientOptions, ServerOptions};
        let path = format!(
            r"\\?\pipe\motion-music-discord-test-{}-{}",
            std::process::id(),
            nonce()
        );
        let server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(&path)
            .unwrap();
        let client = ClientOptions::new().open(&path).unwrap();
        server.connect().await.unwrap();
        let (sender, mut receiver) =
            watch::channel(enabled(Some(song("Synthetic native pipe song"))));
        let task = tokio::spawn(async move {
            session(
                client,
                ID,
                &mut receiver,
                &Shared::default(),
                &mut Rate::default(),
            )
            .await
        });
        let mut peer = Framed::new(server);
        let (opcode, bytes) = peer.read().await.unwrap();
        assert_eq!(opcode, HANDSHAKE);
        assert_eq!(decoded(&bytes)["client_id"], ID);
        peer.write(FRAME, br#"{"cmd":"DISPATCH","evt":"READY","data":{}}"#)
            .await
            .unwrap();
        let (opcode, bytes) = peer.read().await.unwrap();
        assert_eq!(opcode, FRAME);
        let value = decoded(&bytes);
        assert_eq!(
            value["args"]["activity"]["details"],
            "Synthetic native pipe song"
        );
        peer.write(
            FRAME,
            &serde_json::to_vec(&json!({"cmd":"SET_ACTIVITY","nonce":value["nonce"],"data":{}}))
                .unwrap(),
        )
        .await
        .unwrap();
        sender.send_replace(Desired::Disabled);
        let (_, bytes) = peer.read().await.unwrap();
        assert!(decoded(&bytes)["args"]["activity"].is_null());
        assert!(matches!(task.await.unwrap(), Exit::Changed));
    }
}
