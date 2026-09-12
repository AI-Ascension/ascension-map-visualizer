// SPDX-License-Identifier: MIT
use map_visualizer::{
    bundle::Bundle, json, presentation::Settings, source::Source, storage::ArtifactRoot,
};
use serde_json::{Value, json};
use std::{fs, path::Path};

fn reference() -> Value {
    json!({"schema":"ascension.exact_checkpoint_reference.v1",
        "reference_version":"exact-checkpoint-reference-v1", "handle":format!("ckpt-h1:{}", "a".repeat(64)),
        "occurrence":"occurrence:test", "boundary_kind":"stable_decision", "boundary_phase":"map",
        "assurance":"restore_verified", "restore_verified":true})
}

fn write_bundle(path: &Path, reference: Value, version: &str) -> Value {
    fs::create_dir_all(path).unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/conformance");
    for file in [
        "visible-map.json",
        "analysis.json",
        "decision.json",
        "viewer.json",
    ] {
        fs::copy(fixture.join(file), path.join(file)).unwrap();
    }
    let mut manifest: Value =
        serde_json::from_slice(&fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    manifest["bundle_version"] = json!(version);
    manifest["checkpoint_reference"] = reference;
    manifest["bundle_digest"] = json!(json::content_digest(&manifest, "bundle_digest").unwrap());
    fs::write(
        path.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    manifest
}

#[test]
fn checkpoint_reference_survives_real_bundle_render_and_historical_projection() {
    let directory = tempfile::tempdir().unwrap();
    write_bundle(directory.path(), reference(), "sts2.map-view-bundle-v2");
    let bundle = Bundle::load(&ArtifactRoot::open(directory.path()).unwrap()).unwrap();
    let viewer = bundle.viewer(true, false).unwrap();
    assert_eq!(viewer["historical"], true);
    assert_eq!(viewer["checkpoint"]["dispatchable"], false);
    assert_eq!(viewer["checkpoint"]["reference"], reference());
    assert_eq!(
        viewer["checkpoint"]["trajectory_id"],
        bundle.manifest()["trajectory_id"]
    );
    let rendered = directory.path().join("rendered");
    bundle
        .render_to(
            &rendered,
            Settings {
                width: 1200,
                height: 780,
            },
        )
        .unwrap();
    let reloaded = Bundle::load(&ArtifactRoot::open(&rendered).unwrap()).unwrap();
    assert_eq!(
        reloaded.viewer(true, false).unwrap()["checkpoint"],
        viewer["checkpoint"]
    );
}

#[test]
fn bundle_admission_rejects_privileged_unknown_and_inconsistent_references() {
    let mut rejected = Vec::new();
    for field in [
        "exact_state_id",
        "checkpoint_id",
        "blob_digest",
        "native_bytes",
        "extra",
    ] {
        let mut candidate = reference();
        candidate[field] = json!("private");
        rejected.push(candidate);
    }
    for (field, value) in [
        ("reference_version", json!("exact-checkpoint-reference-v2")),
        ("schema", json!("future")),
        ("handle", json!("asc-state:v1:sha256:private")),
        ("restore_verified", json!(false)),
        ("occurrence", json!("bad/id")),
        ("boundary_kind", json!("")),
    ] {
        let mut candidate = reference();
        candidate[field] = value;
        rejected.push(candidate);
    }
    rejected.push(Value::Null);
    for candidate in rejected {
        let directory = tempfile::tempdir().unwrap();
        write_bundle(directory.path(), candidate, "sts2.map-view-bundle-v2");
        assert!(Bundle::load(&ArtifactRoot::open(directory.path()).unwrap()).is_err());
    }
    for version in ["sts2.map-view-bundle-v1", "sts2.map-view-bundle-v99"] {
        let directory = tempfile::tempdir().unwrap();
        write_bundle(directory.path(), reference(), version);
        assert!(Bundle::load(&ArtifactRoot::open(directory.path()).unwrap()).is_err());
    }
}

#[test]
fn feed_admission_binds_reference_to_bundle_and_rejects_unsafe_updates() {
    let directory = tempfile::tempdir().unwrap();
    let temporary = directory.path().join("temporary");
    let manifest = write_bundle(&temporary, reference(), "sts2.map-view-bundle-v2");
    let id = manifest["bundle_digest"].as_str().unwrap();
    fs::rename(&temporary, directory.path().join(id)).unwrap();
    let entry = json!({"sequence":1, "bundle_digest":id,
        "source_state_id":manifest["history"]["source_state_id"], "generation":manifest["history"]["generation"],
        "run_id":manifest["run_id"], "episode_id":manifest["episode_id"], "trajectory_id":manifest["trajectory_id"],
        "observed_at_unix_ms":0, "checkpoint_reference":reference()});
    let feed =
        json!({"feed_version":"sts2.map-feed-v2", "sequence":1, "head":id, "entries":[entry]});
    let write = |value: &Value| {
        fs::write(
            directory.path().join("feed.json"),
            serde_json::to_vec(value).unwrap(),
        )
        .unwrap()
    };
    write(&feed);
    let source = Source::new();
    source.load_root(directory.path(), true).unwrap();
    for (field, value) in [
        ("occurrence", json!("different")),
        ("reference_version", json!("future")),
        ("exact_state_id", json!("private")),
    ] {
        let mut invalid = feed.clone();
        invalid["entries"][0]["checkpoint_reference"][field] = value;
        write(&invalid);
        assert!(source.load_root(directory.path(), true).is_err());
    }
    let mut missing = feed.clone();
    missing["entries"][0]
        .as_object_mut()
        .unwrap()
        .remove("checkpoint_reference");
    write(&missing);
    assert!(source.load_root(directory.path(), true).is_err());
    for version in ["sts2.map-feed-v1", "sts2.map-feed-v99"] {
        let mut invalid = feed.clone();
        invalid["feed_version"] = json!(version);
        write(&invalid);
        assert!(source.load_root(directory.path(), true).is_err());
    }
    write(&feed);
    source.load_root(directory.path(), true).unwrap();
}
