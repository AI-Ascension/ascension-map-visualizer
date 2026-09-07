// SPDX-License-Identifier: MIT

use map_visualizer::telemetry::{Event, Exporter, Stage};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

fn event() -> Event {
    Event {
        stage: Stage::Render,
        duration_micros: 2500,
        nodes: 24,
        edges: 38,
        graph_bytes: 8000,
        image_bytes: 32000,
        cache_hit: false,
        stale: false,
        incomplete: false,
        image_capable: true,
        invalid_action: false,
        trajectory_id: Some("synthetic-trajectory".into()),
    }
}

#[test]
fn actual_otlp_request_contains_only_bounded_metrics_and_sanitized_lineage() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let worker = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let mut bytes = Vec::new();
        let mut chunk = [0; 1024];
        loop {
            let length = socket.read(&mut chunk).unwrap();
            assert!(length > 0);
            bytes.extend_from_slice(&chunk[..length]);
            if let Some(index) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                let header = std::str::from_utf8(&bytes[..index]).unwrap();
                let count = header
                    .lines()
                    .find_map(|s| s.strip_prefix("Content-Length: "))
                    .unwrap()
                    .parse::<usize>()
                    .unwrap();
                if bytes.len() >= index + 4 + count {
                    break;
                }
            }
            assert!(bytes.len() <= 8192);
        }
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}")
            .unwrap();
        bytes
    });
    let exporter = Exporter::new(port).unwrap();
    assert!(exporter.emit(event()));
    assert!(exporter.flush(Duration::from_millis(500)));
    let bytes = worker.join().unwrap();
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.starts_with("POST /v1/traces HTTP/1.1"));
    assert!(!text.contains("Authorization"));
    assert!(!text.contains("synthetic-trajectory"));
    assert!(text.contains("sts2.trajectory_digest"));
    let body: serde_json::Value =
        serde_json::from_str(text.split_once("\r\n\r\n").unwrap().1).unwrap();
    let span = &body["resourceSpans"][0]["scopeSpans"][0]["spans"][0];
    assert_eq!(span["name"], "map.render");
    assert_eq!(span["traceId"].as_str().unwrap().len(), 32);
    assert!(
        span["attributes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["key"] == "map.nodes" && a["value"]["intValue"] == "24")
    );
    assert_eq!(exporter.counts.sent.load(Ordering::Relaxed), 1);
}

#[test]
fn outage_and_invalid_lineage_are_explicit_and_do_not_block_rendering() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let exporter = Exporter::new(port).unwrap();
    let mut bad = event();
    bad.trajectory_id = Some("/private/token".into());
    assert!(!exporter.emit(bad));
    let started = Instant::now();
    assert!(exporter.emit(event()));
    assert!(exporter.flush(Duration::from_millis(500)));
    assert!(started.elapsed() < Duration::from_secs(1));
    assert_eq!(exporter.counts.failed.load(Ordering::Relaxed), 1);
    assert_eq!(exporter.counts.dropped.load(Ordering::Relaxed), 1);
}
