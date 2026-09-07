# Implementation status

Status: implemented components under integration; the complete host/provider feature is not yet accepted.

The repository was created privately under the assignment's fallback visibility rule. Work is on `feat/complete-map-visibility` above the initial `bootstrap` default branch. No merge, release, deployment, service installation, game launch, or provider call has been performed by this task. Companion changes use isolated worktrees; unrelated shared changes remain preserved.

## Product evidence

The Rust product implements strict bundle validation, deterministic SVG/PNG rendering, bounded loopback serving, historical replay, optional OTLP export, and an embedded offline demo. The browser implements read-only inspection, search, overlays, route comparison, and replay controls. CLI tests exercise the actual executable from an unrelated working directory.

The dense harness fixture at `3c4470f0940e3668519194e13b59f234e75821a7` contains 76 nodes and 182 edges. Native Linux and Windows produced byte-identical deterministic bundle files, including PNG and SVG; see [native comparison](evidence/native-demo-comparison.json). Both platform archives were extracted, checksum-verified, and used to reproduce the offline demo. These were working-tree builds; a final exact-commit rebuild remains required after review repairs.

The Linux 25-test and Windows 23-test suites passed before the latest error-classification regression was added. The subsequent focused CLI/feed tests and Linux Clippy passed. Unix-only path/symlink tests explain the platform count difference. Dependency advisories/licenses/sources and workflow validators passed. Actual generated-bundle browser integration and final review are still running; the existing synthetic browser suite has 12 passing tests.

## Companion integration

The accepted neutral protocol dependency is pinned to `d9ffb190ad8990e15f43d7992581dcb2d60b1971`. Copied harness contracts and fixture provenance are machine-checked by `tools/contracts.mjs`.

The host repair at `4ce632bc10dfcfed4d8eeec449ff4df6048490fb` has owner-reported exact-build, managed probe, Rust, and policy passes, with independent review pending. Gateway is at `4b306321f869f0dffb99110cc127dcdac731df4a`. MCP authority repair `0d2035a08cbab9210e89ee0c1adc560fcf47d327` adds pre-forward fencing and zero-forwarding regressions, under independent review.

Harness graph analysis passed the independent nine-test oracle at analysis commit `1213e0e` and fixture commit `e76e6cd`. The later dense fixture repair does not establish full runtime integration. Strict snapshot acquisition, the executable four-mode evaluation, actual provider serialization, and bounded renderer invocation remain under implementation/review. Real provider graph/image delivery and a host-settled map action remain unverified.

## Orchestration evidence

Installed client `0.153.4` accepted explicit `gpt-5.6-luna` / `max` settings for eight descendants. Independent runtime metadata confirmed that pair for all eight, each directly under the root at depth one. The root remained `gpt-6-astra`. The peak of eight descendants stayed below the project ceiling of twelve.

The mandatory three descendant layers are blocked: the child tool surface exposes no delegation operation. No flat tree is represented as depth three, and no alternative clients or independent roots were used to bypass the limitation. Supported project keys were parsed, but the current session's project trust layer was disabled; no effective reload is claimed. See [sanitized orchestration evidence](orchestration-observed.json).

The complete acceptance gates and remaining work are recorded in [acceptance.json](acceptance.json) and the [requirement-to-test matrix](REQUIREMENTS.md). Component tests, synthetic captures, and successful packaging do not establish the live vertical slice.
