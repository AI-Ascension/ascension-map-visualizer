// SPDX-License-Identifier: MIT

use map_visualizer::telemetry::{Event, Exporter, Stage};
use serde_json::Value;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use std::time::Duration;

fn event(trajectory_id: Option<&str>) -> Event {
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
        trajectory_id: trajectory_id.map(str::to_owned),
    }
}

fn collector(requests: usize) -> (u16, JoinHandle<Vec<Value>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let worker = std::thread::spawn(move || {
        let mut values = Vec::with_capacity(requests);
        for _ in 0..requests {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(1)))
                .unwrap();
            let mut bytes = Vec::new();
            let mut chunk = [0; 1024];
            let (body_start, body_length) = loop {
                let length = socket.read(&mut chunk).unwrap();
                assert!(length > 0);
                bytes.extend_from_slice(&chunk[..length]);
                if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                    let header = std::str::from_utf8(&bytes[..index]).unwrap();
                    let count = header
                        .lines()
                        .find_map(|line| line.strip_prefix("Content-Length: "))
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();
                    if bytes.len() >= index + 4 + count {
                        break (index + 4, count);
                    }
                }
                assert!(bytes.len() <= 9216);
            };
            let body = &bytes[body_start..body_start + body_length];
            values.push(serde_json::from_slice(body).unwrap());
            socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}")
                .unwrap();
        }
        values
    });
    (port, worker)
}

fn trajectory_value(body: &Value) -> &str {
    body["resourceSpans"][0]["scopeSpans"][0]["spans"][0]["attributes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|attribute| attribute["key"] == "sts2.trajectory_digest")
        .and_then(|attribute| attribute["value"]["stringValue"].as_str())
        .unwrap()
}

#[test]
fn descriptive_lineage_is_redacted_instead_of_hashed() {
    let legacy = "synthetic-trajectory-001";
    let (port, collector) = collector(1);
    let exporter = Exporter::new(port).unwrap();
    assert!(exporter.emit(event(Some(legacy))));
    assert!(exporter.flush(Duration::from_millis(500)));
    let bodies = collector.join().unwrap();
    let body = &bodies[0];
    assert_eq!(trajectory_value(body), "redacted");
    assert!(!body.to_string().contains(legacy));
    assert_eq!(exporter.counts.sent.load(Ordering::Relaxed), 1);
}

#[test]
fn approved_opaque_lineage_is_digest_linked() {
    let token = "opaque-v1:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let (port, collector) = collector(1);
    let exporter = Exporter::new(port).unwrap();
    assert!(exporter.emit(event(Some(token))));
    assert!(exporter.flush(Duration::from_millis(500)));
    let bodies = collector.join().unwrap();
    let digest = trajectory_value(&bodies[0]);
    assert_eq!(digest.len(), 64);
    assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_ne!(digest, "redacted");
    assert!(!bodies[0].to_string().contains(token));
}

#[test]
fn approved_linkage_is_stable_within_an_exporter_and_unlinkable_after_restart() {
    let token = "opaque-v1:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let (port, first_collector) = collector(2);
    let exporter = Exporter::new(port).unwrap();
    assert!(exporter.emit(event(Some(token))));
    assert!(exporter.emit(event(Some(token))));
    assert!(exporter.flush(Duration::from_millis(500)));
    let bodies = first_collector.join().unwrap();
    let first = trajectory_value(&bodies[0]).to_owned();
    assert_eq!(first, trajectory_value(&bodies[1]));

    let (port, second_collector) = collector(1);
    let exporter = Exporter::new(port).unwrap();
    assert!(exporter.emit(event(Some(token))));
    assert!(exporter.flush(Duration::from_millis(500)));
    let bodies = second_collector.join().unwrap();
    assert_ne!(first, trajectory_value(&bodies[0]));
}

#[test]
fn malformed_opaque_lineage_is_rejected_before_queueing() {
    let exporter = Exporter::new(1).unwrap();
    assert!(!exporter.emit(event(Some("opaque-v1:too-short"))));
    assert_eq!(exporter.counts.accepted.load(Ordering::Relaxed), 0);
    assert_eq!(exporter.counts.dropped.load(Ordering::Relaxed), 1);
}
