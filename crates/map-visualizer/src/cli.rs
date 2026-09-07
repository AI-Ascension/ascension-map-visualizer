// SPDX-License-Identifier: MIT

use crate::{
    RENDERER_VERSION,
    arguments::{Command, Input},
    bundle::Bundle,
    cli_telemetry::Telemetry,
    presentation,
    server::{Assets, Server, Source},
    source::AcquisitionError,
    storage::{ArtifactRoot, File},
    telemetry::Stage,
};
use serde_json::{Value, json};
use std::{
    io::{self, Write},
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

pub struct Failure {
    pub code: u8,
    pub message: &'static str,
}

pub fn execute(command: Command) -> Result<(), Failure> {
    let telemetry = Telemetry::new(matches!(
        &command,
        Command::Validate { .. } | Command::Render { .. } | Command::Demo { .. }
    ));
    match command {
        Command::Help => write_text(crate::arguments::HELP),
        Command::Doctor => write_json(&doctor()),
        Command::Validate { bundle } => write_json(&load(&bundle, &telemetry)?.report()),
        Command::Render {
            bundle,
            out,
            settings,
        } => {
            let bundle = load(&bundle, &telemetry)?;
            let started = Instant::now();
            let report = bundle.render_to(&out, settings).map_err(io_failure)?;
            telemetry.record(
                &bundle,
                Stage::Render,
                started,
                report["png_bytes"].as_u64().unwrap_or(0) as u32,
            );
            write_json(&report)
        }
        Command::Serve {
            input,
            port,
            historical,
        } => serve(input, port, historical),
        Command::Demo {
            out,
            serve: start_server,
            port,
        } => {
            let bundle = crate::demo::load().map_err(|message| Failure { code: 3, message })?;
            let started = Instant::now();
            let report = bundle
                .render_to(&out, presentation::Settings::default())
                .map_err(io_failure)?;
            telemetry.record(
                &bundle,
                Stage::Render,
                started,
                report["png_bytes"].as_u64().unwrap_or(0) as u32,
            );
            write_json(&report)?;
            if start_server {
                serve(Input::Bundle(out), port, true)
            } else {
                Ok(())
            }
        }
    }
}

fn load(path: &Path, telemetry: &Telemetry) -> Result<Bundle, Failure> {
    let started = Instant::now();
    let root = ArtifactRoot::open(path).map_err(|_| io_failure("bundle root unavailable"))?;
    let bundle = Bundle::load(&root).map_err(|message| Failure { code: 3, message })?;
    telemetry.record(&bundle, Stage::Validation, started, 0);
    Ok(bundle)
}

fn doctor() -> Value {
    json!({"product":"ascension-map-visualizer", "version":env!("CARGO_PKG_VERSION"),
        "platform":{"os":std::env::consts::OS, "architecture":std::env::consts::ARCH},
        "renderer_version":RENDERER_VERSION, "rasterizer":"resvg-0.48.1", "glyphs":"project-ascii-block-v1",
        "snapshot_profile":"runtime-map-v1", "snapshot_schema_digest":sts2_protocol::RUNTIME_MAP_V1_SCHEMA_DIGEST,
        "analysis_version":"sts2.map-analysis-v1", "bundle_version":"sts2.map-view-bundle-v1",
        "limits":{"snapshot_bytes":File::Snapshot.limit(), "analysis_bytes":File::Analysis.limit(),
            "viewer_bytes":File::Viewer.limit(), "image_bytes":File::Png.limit(), "image_pixels":presentation::MAX_PIXELS,
            "feed_frames":4096, "replay_memory_bytes":crate::source::MAX_MEMORY_BYTES, "http_workers":4},
        "capabilities":{"offline_render":true, "svg":true, "png":true, "loopback_viewer":true,
            "game_authority":false, "provider_required":false, "telemetry_required":false,
            "atomic_new_directory":cfg!(any(target_os="linux", target_os="windows"))}})
}

fn serve(input: Input, port: u16, historical: bool) -> Result<(), Failure> {
    let source = Arc::new(Source::new());
    match &input {
        Input::Bundle(path) => source.load_bundle(path),
        Input::Root(path) => source.load_root(path, historical),
    }
    .map_err(acquisition_failure)?;
    let server = Server::bind(port).map_err(|_| io_failure("loopback listener unavailable"))?;
    let port = server.port();
    let stop = Arc::new(AtomicBool::new(false));
    let signal = stop.clone();
    ctrlc::set_handler(move || signal.store(true, Ordering::Release))
        .map_err(|_| io_failure("shutdown handler unavailable"))?;
    let server_stop = stop.clone();
    let server_source = source.clone();
    let worker = std::thread::spawn(move || {
        server.run(
            server_source,
            Arc::new(Assets {
                html: include_bytes!("../../../web/index.html").to_vec(),
                script: include_bytes!("../../../web/app.js").to_vec(),
                style: include_bytes!("../../../web/style.css").to_vec(),
            }),
            server_stop,
        )
    });
    let output = write_text(&format!(
        "Read-only viewer: http://127.0.0.1:{port}/\nPress Ctrl-C to stop.\n"
    ));
    if output.is_err() {
        stop.store(true, Ordering::Release);
    }
    let mut ticks = 0;
    let mut last_failure = None;
    while !stop.load(Ordering::Acquire) && !worker.is_finished() {
        std::thread::sleep(Duration::from_millis(100));
        ticks += 1;
        if ticks >= 10 {
            ticks = 0;
            if let Input::Root(path) = &input
                && !historical
            {
                let failure = source.load_root(path, false).err();
                if failure != last_failure {
                    if let Some(message) = failure {
                        eprintln!(
                            "Artifact feed unavailable: {}",
                            acquisition_failure(message).message
                        );
                    }
                    last_failure = failure;
                }
            }
        }
    }
    stop.store(true, Ordering::Release);
    worker
        .join()
        .map_err(|_| io_failure("viewer thread failed"))?
        .map_err(|_| io_failure("viewer server failed"))?;
    output
}

fn write_json(value: &Value) -> Result<(), Failure> {
    let bytes = serde_json::to_string_pretty(value)
        .map_err(|_| io_failure("report serialization failed"))?;
    write_text(&format!("{bytes}\n"))
}

fn write_text(value: &str) -> Result<(), Failure> {
    let mut output = io::stdout().lock();
    output
        .write_all(value.as_bytes())
        .and_then(|_| output.flush())
        .map_err(|_| io_failure("output unavailable"))
}

fn io_failure(message: &'static str) -> Failure {
    Failure { code: 4, message }
}

fn acquisition_failure(error: AcquisitionError) -> Failure {
    match error {
        AcquisitionError::Io(message) => Failure { code: 4, message },
        AcquisitionError::Invalid(message) => Failure { code: 3, message },
    }
}
