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

#[test]
fn inner_artifact_io_errors_remain_distinct_from_invalid_artifacts() {
    let directory = tempfile::tempdir().unwrap();
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/conformance");
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    let id = manifest["bundle_digest"].as_str().unwrap();
    let copied = directory.path().join(id);
    fs::create_dir(&copied).unwrap();
    for entry in fs::read_dir(fixture).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            fs::copy(entry.path(), copied.join(entry.file_name())).unwrap();
        }
    }
    let entry = serde_json::json!({"sequence":1,"bundle_digest":id,
        "source_state_id":manifest["history"]["source_state_id"],"generation":manifest["history"]["generation"],
        "run_id":manifest["run_id"],"episode_id":manifest["episode_id"],"trajectory_id":manifest["trajectory_id"],
        "observed_at_unix_ms":0});
    let feed = serde_json::json!({"feed_version":"sts2.map-feed-v1","sequence":1,"head":id,"entries":[entry]});
    fs::write(
        directory.path().join("feed.json"),
        serde_json::to_vec(&feed).unwrap(),
    )
    .unwrap();
    let snapshot = copied.join("visible-map.json");
    let original = fs::read(&snapshot).unwrap();
    fs::remove_file(&snapshot).unwrap();
    for (command, flag, path) in [
        ("validate", "--bundle", copied.as_path()),
        ("serve", "--bundle", copied.as_path()),
        ("replay", "--root", directory.path()),
    ] {
        let result = cli().args([command, flag]).arg(path).output().unwrap();
        assert_eq!(
            result.status.code(),
            Some(4),
            "{command}: {:?}",
            result.stderr
        );
    }
    // A rejected file shape is a validation error, not an unavailable file.
    fs::create_dir(&snapshot).unwrap();
    assert_eq!(
        cli()
            .args(["validate", "--bundle"])
            .arg(&copied)
            .output()
            .unwrap()
            .status
            .code(),
        Some(3)
    );
    fs::remove_dir(&snapshot).unwrap();
    fs::write(&snapshot, original).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&snapshot, fs::Permissions::from_mode(0o000)).unwrap();
        // Permission enforcement can be bypassed by root; only exercise it where effective.
        if fs::read(&snapshot).is_err() {
            let result = cli()
                .args(["replay", "--root"])
                .arg(directory.path())
                .output()
                .unwrap();
            fs::set_permissions(&snapshot, fs::Permissions::from_mode(0o600)).unwrap();
            assert_eq!(result.status.code(), Some(4));
        } else {
            fs::set_permissions(&snapshot, fs::Permissions::from_mode(0o600)).unwrap();
        }
    }
}
