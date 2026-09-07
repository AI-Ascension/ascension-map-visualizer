# Requirement-to-test matrix

This matrix distinguishes test implementation from final acceptance. [acceptance.json](acceptance.json) is the overall gate record; only evidence at the final exact companion set can satisfy an integrated gate.

| Requirement family | Owning implementation and validation | Current evidence boundary |
|---|---|---|
| Complete bounded topology and stable identities | Protocol `visible_map` decoder/conformance; mod main-thread map projection; harness graph oracle | Protocol and synthetic graphs tested; final native projection review and live comparison pending |
| Native fair-play scope and zero read side effects | Mod native API probe, map-open/closed/modal tests, bounded traversal, paired hidden-state projection | Managed/native source tests differ from a real player-visible off-screen comparison; live gate pending |
| Instance/session/lease/profile/digest fences | Gateway map route tests, MCP wire and executable adapter capture tests | Independent review found a pre-forward lease/epoch issue; repair required before transport acceptance |
| Deterministic analysis and valid bounded candidates | Harness map analysis tests and non-authoring exhaustive oracle | Configurable public route features implemented; final oracle rerun and executable evaluation pending |
| Exact graph/image presentation | Product `rendering`, `bundle_integration`, `cli`; manual PNG inspection | Complete small fixture passed; larger harness demo integration exposed invalid current-position metadata and is being repaired upstream |
| Browser read-only interaction | Playwright `tests/browser`, actual generated payload integration | Synthetic desktop/mobile interaction tested; generated bundle and dense-map follow-up running |
| Provider graph/image serialization and legal dispatch | Harness episode, Exo, Astra bridge capture; actual MCP process and visualizer executable | Adapters under integration; real provider delivery and host-settled map action unverified |
| Atomic cache/replay and source-time knowledge | Harness bundle-store/cache tests; product `publication`, `feed`, `cli` | Product tests exercise concurrent no-replace publication, corrupted artifacts, lineage splice rejection, stale current versus historical replay, and restart validation |
| Packaging and security | Native Linux/Windows Rust suites; `cargo deny`; `actionlint`; `zizmor`; dependency notice collection | Native product tests passed before dense fixture refresh; release packaging and final exact-head reruns pending |
| Optional bounded observability | Product actual OTLP request/outage tests; observability policy/compose checks | Exporter synthetic capture/outage tests passed; existing collector deployment and end-to-end trace ingestion not claimed |

## Product test entrypoints

Run the locked workspace suite for all product Rust tests. Parser property tests cover duplicate JSON keys and bounded hostile inputs. Renderer tests assert exact node/edge representation, deterministic ordering, unknown-label escaping, duplicate-coordinate separation, and finite pixel/element limits. These expected values are structural assertions; image snapshots alone are not the oracle.

`cli` runs the actual executable from an unrelated temporary working directory, renders an embedded fixture, loads the result back, rejects corruption, checks exit codes, and proves an existing destination stays unchanged. `feed` reaches the actual loopback server and distinguishes historical frames from an expired current head.

The browser test suite runs Chromium locally with synthetic public data. The native Windows Rust suite exercises Windows publication semantics and the same executable CLI; Unix-specific non-UTF-8 path and symlink tests are explicitly platform-specific. Full source/runtime/native/live/provider evidence must remain separately labeled.
