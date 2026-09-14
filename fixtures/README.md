# Synthetic fixtures

All fixture JSON is copied from the harness owner at the exact commit and file hashes in `contract-artifacts/harness/provenance.json`, under MIT. The `conformance/` and `demo/` entries cite harness artifact revision `3c4470f0940e3668519194e13b59f234e75821a7`, the revision recorded in `docs/ASSEMBLY_MANIFEST.json` and the one compatible with the pinned `sts2-protocol` package; the schema copies in the same manifest cite the later revision that hardened the checkpoint-reference contract. No game assets, native process data, model transcript, or credentials are included.

`conformance/` preserves the small protocol-derived four-node bundle. `demo/` is the separately generated dense harness example used by the standalone executable. The harness fixture tests regenerate analysis and manifests from their declared public input snapshots. Consumer CLI validation remains an independent requirement: successful generation alone does not establish protocol conformance.

The browser has separate authored interaction fixtures. They exercise UI states and do not prove host extraction or provider delivery.
