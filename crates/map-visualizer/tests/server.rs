// SPDX-License-Identifier: MIT

use map_visualizer::server::{Assets, Server, Source};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

fn request(port: u16, path: &str, extra: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(3)))
        .unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{extra}\r\n"
    )
    .unwrap();
    let mut bytes = String::new();
    stream.read_to_string(&mut bytes).unwrap();
    bytes
}

#[test]
fn actual_loopback_server_has_only_fixed_read_routes_and_origin_fences() {
    let server = Server::bind(0).unwrap();
    let port = server.port();
    let stop = Arc::new(AtomicBool::new(false));
    let shutdown = Arc::clone(&stop);
    let worker = std::thread::spawn(move || {
        server.run(
            Arc::new(Source::new()),
            Arc::new(Assets {
                html: b"offline".to_vec(),
                script: b"/* owned */".to_vec(),
                style: b"/* owned */".to_vec(),
            }),
            shutdown,
        )
    });
    let current = request(port, "/api/current", "");
    assert!(current.starts_with("HTTP/1.1 503"));
    assert!(current.contains("Content-Security-Policy:"));
    assert!(request(port, "/api/replay", "").ends_with("[]"));
    assert!(
        request(
            port,
            "/api/current",
            "Origin: https://untrusted.example\r\n"
        )
        .starts_with("HTTP/1.1 403")
    );
    for path in [
        "/api/dispatch",
        "/api/action",
        "/private.json",
        "//example.com/x",
    ] {
        assert!(request(port, path, "").starts_with("HTTP/1.1 404"));
    }
    assert!(request(port, "/api/frame/recorded", "").starts_with("HTTP/1.1 503"));
    assert!(request(port, "/api/frame/%2e%2e", "").starts_with("HTTP/1.1 400"));
    assert!(!current.contains("Access-Control-Allow-Origin"));
    stop.store(true, Ordering::Release);
    worker.join().unwrap().unwrap();
}
