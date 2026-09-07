// SPDX-License-Identifier: MIT
use std::{fs, process::Command};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_map-visualizer"))
}

#[test]
fn portable_demo_roundtrip_and_exit_codes() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("demo");
    let result = cli()
        .current_dir(directory.path())
        .args(["demo", "--out"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["dispatchable"], false);
    assert!(output.join("overview.png").is_file());
    assert!(output.join("index.html").is_file());
    let validated = cli()
        .args(["validate", "--bundle"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        validated.status.success(),
        "{}",
        String::from_utf8_lossy(&validated.stderr)
    );
    let unchanged = fs::read(output.join("manifest.json")).unwrap();
    assert_eq!(
        cli()
            .args(["demo", "--out"])
            .arg(&output)
            .output()
            .unwrap()
            .status
            .code(),
        Some(4)
    );
    assert_eq!(fs::read(output.join("manifest.json")).unwrap(), unchanged);
    fs::write(output.join("overview.png"), b"invalid image").unwrap();
    assert_eq!(
        cli()
            .args(["validate", "--bundle"])
            .arg(&output)
            .output()
            .unwrap()
            .status
            .code(),
        Some(3)
    );
    assert_eq!(
        cli()
            .args(["render", "--width", "999999"])
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
    let doctor = cli().arg("doctor").output().unwrap();
    assert!(doctor.status.success());
    let doctor: serde_json::Value = serde_json::from_slice(&doctor.stdout).unwrap();
    assert_eq!(doctor["capabilities"]["game_authority"], false);
}

#[test]
fn serve_and_replay_distinguish_missing_roots_from_rejected_feed() {
    let directory = tempfile::tempdir().unwrap();
    let missing = directory.path().join("absent");
    for command in ["serve", "replay"] {
        let result = cli()
            .args([command, "--root"])
            .arg(&missing)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(4));
        let result = cli()
            .args([command, "--root"])
            .arg(directory.path())
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(4));
    }
    fs::write(directory.path().join("feed.json"), b"{bad").unwrap();
    let result = cli()
        .args(["replay", "--root"])
        .arg(directory.path())
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(3));
}
