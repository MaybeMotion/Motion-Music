//! Explicit manual check of the real desktop Discord client, using synthetic metadata.
//! Run with --connect; ordinary tests and demos never start this probe.
use spotifast::discord::{Activity, Client, Status};
use std::time::{Duration, Instant};
fn main() {
    if !std::env::args().any(|arg| arg == "--connect") {
        println!("Pass --connect to publish a short synthetic Motion Music activity.");
        return;
    }
    let mut client = Client::default();
    let now = jiff::Timestamp::now().as_second();
    client.update(
        Some(spotifast::discord_activity::DEFAULT_APPLICATION_ID),
        Some(Activity {
            details: "Motion Music connection check".into(),
            state: Some("Synthetic integration test".into()),
            large_image: None,
            large_text: None,
            start: Some(now),
            end: Some(now + 60),
            url: None,
        }),
    );
    let start = Instant::now();
    let mut previous = None;
    let mut connected = false;
    while start.elapsed() < Duration::from_secs(45) {
        let status = client.status();
        if previous.as_ref() != Some(&status) {
            println!("{status:?}");
            previous = Some(status.clone());
        }
        connected |= status == Status::Connected;
        std::thread::sleep(Duration::from_millis(100));
    }
    client.update(None, None);
    std::thread::sleep(Duration::from_millis(500));
    client.shutdown();
    std::thread::sleep(Duration::from_millis(200));
    println!("connected={connected}; clear requested");
    if !connected {
        std::process::exit(1);
    }
}
