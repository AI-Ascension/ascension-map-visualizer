# Harness fixture and schema origins

The copied harness artifacts in `contract-artifacts/harness/` and `fixtures/` span two immutable
`AI-Ascension/sts2-harness` revisions. `contract-artifacts/harness/provenance.json` records the
revision per file, and `tools/verify-harness-provenance.mjs` fetches each recorded revision and
verifies every copy byte-for-byte against it.

## Verified origins

| Copy group | Harness revision | Why that revision |
| --- | --- | --- |
| `contract-artifacts/harness/analysis.schema.json`, `bundle.schema.json`, `feed.schema.json` | `67e3c241912bf687695bdd9cf547ca53335e5b3c` | The revision that hardened the checkpoint-reference contract. `docs/map-view-bundle.schema.json` and `docs/map-feed.schema.json` changed at this commit and nowhere later. |
| `fixtures/conformance/*`, `fixtures/demo/*` | `3c4470f0940e3668519194e13b59f234e75821a7` | The artifact revision recorded in `docs/ASSEMBLY_MANIFEST.json` (`contracts.harness_artifact_revision`) and in `docs/IMPLEMENTATION_STATUS.md`. All ten copied fixture files are byte-identical to `crates/harness/tests/fixtures/map-bundle-v1/9ef1d35f...` and `crates/harness/tests/fixtures/map-bundle-demo-v1/827b997e...` at this revision. |

Byte verification (`sha256` of every copy compared with the file at the recorded revision) matched
all thirteen entries. `git fetch --depth=1 <sha>` resolves both revisions from the public owner
repository, including the earlier revision that later commits renamed.

## Durability of the recorded revisions

Verification fetches each recorded revision by SHA from the public owner repository. `67e3c241` is
reachable from the owner's default branch. `3c4470f` is contained in the owner's
`feat/map-visibility-20260906` and `codex/map-protocol-evaluation-20260907` branches; three dense-demo
blobs (`analysis.json`, `manifest.json`, `visible-map.json`) exist in no revision reachable from the
current default branch, so no default-branch revision can attribute those copies.

If the owner ever prunes those branches without tagging the artifact revision, `git fetch` of
`3c4470f` fails and `tools/verify-harness-provenance.mjs` fails closed instead of reporting a pass;
the record must then be refreshed with the owner. A durable follow-up would be an owner tag on the
artifact revision or a fixture re-baseline alongside an authorized protocol revision bump.

## Why a single revision pin cannot describe the copies

Harness commit `51a7d89` (`fix(map): refresh protocol contract fixtures`) renamed the fixture
directories to `5f50f23c...` and `bd114749...` and refreshed their contents, and harness commit
`67e3c241` then changed the bundle and feed schemas. No single harness revision contains both the
newer schemas and the copied fixture bytes. Both single-revision repairs were attempted and rejected
with recorded evidence:

- Substituting the fixture bytes from the pinned revision `67e3c241` failed native bundle admission:
  `Error: Invalid("unsupported snapshot contract")` in
  `crates/map-visualizer/tests/bundle_integration.rs`. The pinned `sts2-protocol` revision
  `d9ffb190ad8990e15f43d7992581dcb2d60b1971` fixes `RUNTIME_MAP_V1_SCHEMA_DIGEST` at
  `6340f3cbe6c1b5728144fe89fdfdf8645acf2f59a77c0e0c30ebfeafc77515d8`, while the refreshed fixture
  manifests carry `ceab0d2dfc471d1ec36d12edaf4654b8c7fdced06548bf47265e11c63f98115b` (protocol
  revision `0bc689ea`), so adopting those bytes requires a protocol revision bump outside this
  repair's scope.
- Substituting the schema copies from `3c4470f` failed checkpoint withdrawal:
  `checkpoint_reference_survives_real_bundle_render_and_historical_projection` and
  `feed_admission_binds_reference_to_bundle_and_rejects_unsafe_updates` both failed, because the
  older bundle schema pins `bundle_version` to `sts2.map-view-bundle-v1` and does not admit the
  `checkpoint_reference`/`v2` contract those tests exercise.

Because both naive substitutions regress accepted behavior, the repair records the real, separately
immutable origin of each copy and verifies every entry against its own revision. No check was
suppressed or relaxed: path, symlink, duplicate-entry, schema, digest and byte-identity checks are
unchanged, and `tools/contracts.mjs` now additionally rejects a fixture group that disagrees with the
artifact revision recorded in the assembly manifest.

## Reproduction commands

```sh
git clone --quiet https://github.com/AI-Ascension/sts2-harness.git /tmp/harness-ref
git -C /tmp/harness-ref show 67e3c241912bf687695bdd9cf547ca53335e5b3c:docs/map-view-bundle.schema.json | sha256sum
git -C /tmp/harness-ref show 3c4470f0940e3668519194e13b59f234e75821a7:crates/harness/tests/fixtures/map-bundle-v1/9ef1d35f8d29f9112b5fdaf08070c7b23293c5744eeed346212942e435c4406a/manifest.json | sha256sum
node tools/contracts.mjs
node --test tools/verify-harness-provenance.test.mjs
node tools/verify-harness-provenance.mjs
```

This evidence covers source and copied-artifact identity only. It does not assert producer
deployment, live host settlement, or provider delivery.
