// SPDX-License-Identifier: MIT
use map_visualizer::{
    bundle::Bundle,
    server::{Assets, Server, Source},
    storage::ArtifactRoot,
};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

fn request(port: u16, path: &str) -> String {
    let mut socket = TcpStream::connect(("127.0.0.1", port)).unwrap();
    socket
        .set_read_timeout(Some(std::time::Duration::from_secs(3)))
        .unwrap();
    write!(
        socket,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n"
    )
    .unwrap();
    let mut body = String::new();
    socket.read_to_string(&mut body).unwrap();
    body
}

#[test]
fn replay_retains_source_time_and_rejects_partial_or_spliced_feed() {
    let directory = tempfile::tempdir().unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/conformance");
    let bundle = Bundle::load(&ArtifactRoot::open(&fixture).unwrap()).unwrap();
    let id = bundle.id();
    let manifest = bundle.manifest();
    let copied = directory.path().join(id);
    fs::create_dir(&copied).unwrap();
    for file in [
        "manifest.json",
        "visible-map.json",
        "analysis.json",
        "decision.json",
        "viewer.json",
    ] {
        fs::copy(fixture.join(file), copied.join(file)).unwrap();
    }
    let entry = json!({"sequence":1,"bundle_digest":id,
        "source_state_id":manifest["history"]["source_state_id"],"generation":manifest["history"]["generation"],
        "run_id":manifest["run_id"],"episode_id":manifest["episode_id"],"trajectory_id":manifest["trajectory_id"],
        "observed_at_unix_ms":0});
    let feed = json!({"feed_version":"sts2.map-feed-v1","sequence":1,"head":id,"entries":[entry]});
    let write_feed = |feed: &Value| {
        fs::write(
            directory.path().join("feed.json"),
            serde_json::to_vec(feed).unwrap(),
        )
        .unwrap()
    };
    write_feed(&feed);
    let source = Arc::new(Source::new());
    source.load_root(directory.path(), true).unwrap();
    let server = Server::bind(0).unwrap();
    let port = server.port();
    let stop = Arc::new(AtomicBool::new(false));
    let worker_source = source.clone();
    let worker_stop = stop.clone();
    let worker = std::thread::spawn(move || {
        server.run(
            worker_source,
            Arc::new(Assets {
                html: vec![],
                script: vec![],
                style: vec![],
            }),
            worker_stop,
        )
    });
    let response = request(port, &format!("/api/frame/{id}"));
    let payload: Value = serde_json::from_str(response.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(payload["historical"], true);
    assert_eq!(payload["bundle_id"], id);
    assert_eq!(payload["map"]["nodes"].as_array().unwrap().len(), 4);
    let mut splice = feed.clone();
    splice["entries"][0]["run_id"] = json!("other-run");
    write_feed(&splice);
    assert!(source.load_root(directory.path(), true).is_err());
    assert_eq!(request(port, &format!("/api/frame/{id}")), response);
    fs::write(directory.path().join("feed.json"), b"{\"feed_version\":").unwrap();
    assert!(source.load_root(directory.path(), true).is_err());
    assert_eq!(request(port, &format!("/api/frame/{id}")), response);
    write_feed(&feed);
    source.load_root(directory.path(), false).unwrap();
    assert!(request(port, "/api/current").starts_with("HTTP/1.1 503"));
    assert!(request(port, &format!("/api/frame/{id}")).starts_with("HTTP/1.1 200"));
    stop.store(true, Ordering::Release);
    worker.join().unwrap().unwrap();
    // Restart does not reuse validation of on-disk artifacts.
    fs::write(copied.join("analysis.json"), b"{}").unwrap();
    assert!(Source::new().load_root(directory.path(), true).is_err());
}
