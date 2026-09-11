// SPDX-License-Identifier: MIT
use map_visualizer::{bundle::Bundle, storage::ArtifactRoot};
use std::path::{Path, PathBuf};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/conformance")
}

#[test]
fn copied_owner_contracts_and_fixture_documents_are_strict_json() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for path in [
        "contract-artifacts/harness/bundle.schema.json",
        "contract-artifacts/harness/analysis.schema.json",
        "contract-artifacts/harness/feed.schema.json",
        "fixtures/conformance/manifest.json",
        "fixtures/conformance/analysis.json",
        "fixtures/conformance/visible-map.json",
        "fixtures/conformance/viewer.json",
        "fixtures/conformance/decision.json",
    ] {
        let bytes = std::fs::read(root.join(path)).unwrap();
        assert!(
            map_visualizer::json::decode(&bytes, 256 * 1024).is_ok(),
            "strict JSON rejected {path}"
        );
    }
}

#[test]
fn harness_fixture_loads_all_nodes_and_bindings_without_a_host_or_provider() {
    let bundle = Bundle::load(&ArtifactRoot::open(&fixture()).unwrap()).unwrap();
    let map = bundle.presentation().unwrap();
    assert_eq!(map.nodes.len(), 4);
    assert_eq!(map.edges.len(), 4);
    assert_eq!(map.nodes.iter().filter(|node| node.legal).count(), 2);
    let payload = bundle.viewer(true, false).unwrap();
    assert_eq!(payload["historical"], true);
    assert_eq!(payload["bundle_id"], bundle.id());
}
