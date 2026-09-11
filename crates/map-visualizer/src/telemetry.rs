// SPDX-License-Identifier: MIT
//! Optional bounded OTLP/HTTP export to an operator-selected loopback Collector.

use hmac::{Hmac, KeyInit, Mac};
use serde_json::json;
use sha2::Sha256;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc::{self, SyncSender},
};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy)]
pub enum Stage {
    Capture,
    Validation,
    Analysis,
    Render,
    ProviderDelivery,
    ActionValidation,
    Cache,
}
impl Stage {
    fn name(self) -> &'static str {
        match self {
            Self::Capture => "capture",
            Self::Validation => "validation",
            Self::Analysis => "analysis",
            Self::Render => "render",
            Self::ProviderDelivery => "provider_delivery",
            Self::ActionValidation => "action_validation",
            Self::Cache => "cache",
        }
    }
}

pub struct Event {
    pub stage: Stage,
    pub duration_micros: u64,
    pub nodes: u32,
    pub edges: u32,
    pub graph_bytes: u32,
    pub image_bytes: u32,
    pub cache_hit: bool,
    pub stale: bool,
    pub incomplete: bool,
    pub image_capable: bool,
    pub invalid_action: bool,
    /// Optional harness lineage. Only the documented `opaque-v1:` token form
    /// is eligible for a stable digest; legacy or descriptive IDs are emitted
    /// as a redacted linkage marker.
    pub trajectory_id: Option<String>,
}

const OPAQUE_LINEAGE_PREFIX: &str = "opaque-v1:";
const REDACTED_LINEAGE: &str = "redacted";

#[derive(Default)]
pub struct Counts {
    pub accepted: AtomicU64,
    pub cancelled: AtomicU64,
    pub sent: AtomicU64,
    pub failed: AtomicU64,
    pub dropped: AtomicU64,
}

pub struct Exporter {
    sender: Option<SyncSender<Event>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    pub counts: Arc<Counts>,
}

impl Exporter {
    pub fn new(port: u16) -> std::io::Result<Self> {
        if port == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid Collector port",
            ));
        }
        let (sender, receiver) = mpsc::sync_channel::<Event>(64);
        let stop = Arc::new(AtomicBool::new(false));
        let counts = Arc::new(Counts::default());
        let mut lineage_key = [0u8; 32];
        getrandom::fill(&mut lineage_key)
            .map_err(|error| std::io::Error::other(format!("lineage key unavailable: {error}")))?;
        let worker_stop = Arc::clone(&stop);
        let worker_counts = Arc::clone(&counts);
        let worker = std::thread::Builder::new()
            .name("map-telemetry".into())
            .spawn(move || {
                while !worker_stop.load(Ordering::Acquire) {
                    match receiver.recv_timeout(Duration::from_millis(50)) {
                        Ok(event) => {
                            let ok = encode(event, &lineage_key)
                                .and_then(|bytes| send(port, &bytes).ok())
                                .is_some();
                            if ok {
                                worker_counts.sent.fetch_add(1, Ordering::Relaxed);
                            } else {
                                worker_counts.failed.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }
            })?;
        Ok(Self {
            sender: Some(sender),
            stop,
            worker: Some(worker),
            counts,
        })
    }

    /// Nonblocking; invalid or overloaded telemetry cannot block the structured graph path.
    pub fn emit(&self, event: Event) -> bool {
        if !valid(&event)
            || self
                .sender
                .as_ref()
                .is_none_or(|sender| sender.try_send(event).is_err())
        {
            self.counts.dropped.fetch_add(1, Ordering::Relaxed);
            false
        } else {
            self.counts.accepted.fetch_add(1, Ordering::Relaxed);
            true
        }
    }

    /// Explicit bounded flush for short-lived CLI invocations; never changes product success.
    pub fn flush(&self, timeout: Duration) -> bool {
        let start = std::time::Instant::now();
        let timeout = timeout.min(Duration::from_millis(500));
        loop {
            if self.counts.sent.load(Ordering::Relaxed) + self.counts.failed.load(Ordering::Relaxed)
                >= self.counts.accepted.load(Ordering::Relaxed)
            {
                return true;
            }
            if start.elapsed() >= timeout {
                return false;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// Stops queued export with a bounded network operation before joining the owned worker.
    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.sender.take();
        if let Some(worker) = self.worker.take()
            && worker.join().is_err()
        {
            self.counts.failed.fetch_add(1, Ordering::Relaxed);
        }
        let unfinished = self.counts.accepted.load(Ordering::Relaxed).saturating_sub(
            self.counts.sent.load(Ordering::Relaxed)
                + self.counts.failed.load(Ordering::Relaxed)
                + self.counts.cancelled.load(Ordering::Relaxed),
        );
        if unfinished > 0 {
            self.counts.dropped.fetch_add(unfinished, Ordering::Relaxed);
            self.counts
                .cancelled
                .fetch_add(unfinished, Ordering::Relaxed);
        }
    }
}

impl Drop for Exporter {
    fn drop(&mut self) {
        self.stop();
    }
}

fn valid(event: &Event) -> bool {
    event.duration_micros <= 600_000_000
        && event.nodes <= 1024
        && event.edges <= 8192
        && event.graph_bytes <= 2 * 1024 * 1024
        && event.image_bytes <= 16 * 1024 * 1024
        && event
            .trajectory_id
            .as_ref()
            .is_none_or(|id| valid_trajectory_id(id))
}

fn valid_trajectory_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-.:".contains(&b))
        && (!id.starts_with(OPAQUE_LINEAGE_PREFIX) || approved_lineage(id))
}

fn encode(event: Event, lineage_key: &[u8; 32]) -> Option<Vec<u8>> {
    if !valid(&event) {
        return None;
    }
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let end = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_nanos();
    let start = end.saturating_sub(u128::from(event.duration_micros) * 1000);
    let identity = format!(
        "{end}:{}:{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    );
    let hash = crate::digest::sha256(identity.as_bytes());
    let mut attributes =
        vec![json!({"key":"map.stage","value":{"stringValue":event.stage.name()}})];
    for (key, value) in [
        ("map.duration_micros", event.duration_micros),
        ("map.nodes", u64::from(event.nodes)),
        ("map.edges", u64::from(event.edges)),
        ("map.graph_bytes", u64::from(event.graph_bytes)),
        ("map.image_bytes", u64::from(event.image_bytes)),
    ] {
        attributes.push(json!({"key":key,"value":{"intValue":value.to_string()}}));
    }
    for (key, value) in [
        ("map.cache_hit", event.cache_hit),
        ("map.stale", event.stale),
        ("map.incomplete", event.incomplete),
        ("map.image_capable", event.image_capable),
        ("map.invalid_action", event.invalid_action),
    ] {
        attributes.push(json!({"key":key,"value":{"boolValue":value}}));
    }
    if let Some(id) = event.trajectory_id {
        // A raw run/episode/trajectory identifier may be descriptive, stable,
        // or otherwise enumerable. Hashing that value would make the digest a
        // reversible lookup handle. Linkage is therefore derived only from a
        // producer-issued 256-bit opaque token with an explicit namespace and
        // a per-exporter secret; older IDs remain useful for event accounting
        // but are never hashed.
        let lineage = if approved_lineage(&id) {
            lineage_digest(lineage_key, &id)
        } else {
            REDACTED_LINEAGE.to_owned()
        };
        attributes.push(json!({"key":"sts2.trajectory_digest","value":{"stringValue":lineage}}));
    }
    let body = json!({"resourceSpans":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"ascension-map-visualizer"}}]},"scopeSpans":[{"scope":{"name":"ascension-map","version":"1"},"spans":[{"traceId":&hash[..32],"spanId":&hash[32..48],"name":format!("map.{}",event.stage.name()),"kind":1,"startTimeUnixNano":start.to_string(),"endTimeUnixNano":end.to_string(),"attributes":attributes}]}]}]});
    let bytes = serde_json::to_vec(&body).ok()?;
    if bytes.len() > 8192 {
        None
    } else {
        Some(bytes)
    }
}

fn approved_lineage(value: &str) -> bool {
    value
        .strip_prefix(OPAQUE_LINEAGE_PREFIX)
        .is_some_and(crate::digest::valid)
}

fn lineage_digest(key: &[u8; 32], value: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("fixed-size lineage key");
    mac.update(b"ascension-map-trajectory-v2:");
    mac.update(value.as_bytes());
    let output = mac.finalize().into_bytes();
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut digest = String::with_capacity(output.len() * 2);
    for byte in output {
        digest.push(char::from(HEX[usize::from(byte >> 4)]));
        digest.push(char::from(HEX[usize::from(byte & 15)]));
    }
    digest
}

fn send(port: u16, body: &[u8]) -> std::io::Result<()> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let mut socket = TcpStream::connect_timeout(&address, Duration::from_millis(100))?;
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;
    socket.set_write_timeout(Some(Duration::from_millis(100)))?;
    let header = format!(
        "POST /v1/traces HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    socket.write_all(header.as_bytes())?;
    socket.write_all(body)?;
    let mut response = [0; 1024];
    let length = socket.read(&mut response)?;
    if response[..length].starts_with(b"HTTP/1.1 200")
        || response[..length].starts_with(b"HTTP/1.1 204")
    {
        Ok(())
    } else {
        Err(std::io::Error::other("Collector rejected trace"))
    }
}
