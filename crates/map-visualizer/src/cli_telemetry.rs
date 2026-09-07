// SPDX-License-Identifier: MIT

use crate::{
    bundle::Bundle,
    telemetry::{Event, Exporter, Stage},
};
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
use sts2_protocol::{RuntimeMapV1Completeness, RuntimeMapV1Freshness};

pub(crate) struct Telemetry(Option<Exporter>);

impl Telemetry {
    pub fn new(enabled: bool) -> Self {
        if !enabled {
            return Self(None);
        }
        let Ok(value) = std::env::var("ASCENSION_MAP_OTLP_PORT") else {
            return Self(None);
        };
        let exporter = value
            .parse::<u16>()
            .ok()
            .and_then(|port| Exporter::new(port).ok());
        if exporter.is_none() {
            eprintln!(
                "Optional telemetry disabled: invalid or unavailable loopback Collector port."
            );
        }
        Self(exporter)
    }

    pub fn record(&self, bundle: &Bundle, stage: Stage, started: Instant, image_bytes: u32) {
        let Some(exporter) = &self.0 else {
            return;
        };
        let accepted = exporter.emit(Event {
            stage,
            duration_micros: started.elapsed().as_micros().try_into().unwrap_or(u64::MAX),
            nodes: bundle.snapshot.nodes.len() as u32,
            edges: bundle.snapshot.edges.len() as u32,
            graph_bytes: bundle.snapshot_bytes.len() as u32,
            image_bytes,
            cache_hit: false,
            stale: matches!(bundle.snapshot.freshness, RuntimeMapV1Freshness::Historical),
            incomplete: !matches!(
                bundle.snapshot.completeness,
                RuntimeMapV1Completeness::Complete
            ),
            image_capable: false,
            invalid_action: false,
            trajectory_id: bundle.manifest["trajectory_id"].as_str().map(str::to_owned),
        });
        if !accepted {
            eprintln!("Optional telemetry event rejected or queue full.");
        }
    }
}

impl Drop for Telemetry {
    fn drop(&mut self) {
        if let Some(exporter) = &self.0 {
            let flushed = exporter.flush(Duration::from_millis(500));
            if !flushed || exporter.counts.failed.load(Ordering::Relaxed) > 0 {
                eprintln!(
                    "Optional telemetry export failed or exceeded its bounded flush deadline."
                );
            }
        }
    }
}
