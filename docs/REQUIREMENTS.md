# Requirement-to-test matrix

This matrix distinguishes test implementation from final acceptance. [acceptance.json](acceptance.json) is the overall gate record; only evidence at the final exact companion set can satisfy an integrated gate.

| Requirement family | Owning implementation and validation | Current evidence boundary |
|---|---|---|
| Complete bounded topology and stable identities | Protocol `visible_map` decoder/conformance; mod main-thread map projection; harness graph oracle | Protocol and synthetic graphs tested; final native projection review and live comparison pending |
| Native fair-play scope and zero read side effects | Mod native API probe, map-open/closed/modal tests, bounded traversal, paired hidden-state projection | Managed/native source tests differ from a real player-visible off-screen comparison; live gate pending |
| Instance/session/lease/profile/digest fences | Gateway map route tests, MCP wire and executable adapter capture tests | MCP repair `0d2035a` passed independent zero-forwarding lease/epoch regressions and hosted CI; final host/runtime assembly remains pending |
| Deterministic analysis and valid bounded candidates | Harness map analysis tests and non-authoring exhaustive oracle | Independent nine-test oracle passed at `dff1b947`; executable evaluation still needs real mode inputs, actual PNG, and measured latency |
| Exact graph/image presentation | Product `rendering`, `bundle_integration`, `cli`; manual PNG inspection | Validated small 4-node/4-edge and dense 76-node/182-edge fixtures; ten generated demo files match across native Linux and Windows at `5b1d196` |
| Browser read-only interaction | Playwright `tests/browser`, actual generated payload integration | All 18 desktop/mobile tests passed with fresh CLI-generated small/dense bundles; missing inputs fail CI |
| Provider graph/image serialization and legal dispatch | Harness episode, Exo, Astra bridge capture; actual MCP process and visualizer executable | Adapters under integration; real provider delivery and host-settled map action unverified |
| Atomic cache/replay and source-time knowledge | Harness bundle-store/cache tests; product `publication`, `feed`, `cli` | Product tests exercise concurrent no-replace publication, corrupted artifacts, lineage splice rejection, stale current versus historical replay, and restart validation |
| Packaging and security | Native Linux/Windows Rust suites; `cargo deny`; `actionlint`; `zizmor`; dependency notice collection | Native Linux/Windows tests, packages, extracted CLI smoke tests, private-path scans, and all hosted checks passed at `5b1d196`; independent distribution review is pending |
| Optional bounded observability | Product actual OTLP request/outage tests; observability policy/compose checks | Exporter synthetic capture/outage tests passed; existing collector deployment and end-to-end trace ingestion not claimed |

## Product test entrypoints

Run the locked workspace suite for all product Rust tests. Parser property tests cover duplicate JSON keys and bounded hostile inputs. Renderer tests assert exact node/edge representation, deterministic ordering, unknown-label escaping, duplicate-coordinate separation, and finite pixel/element limits. These expected values are structural assertions; image snapshots alone are not the oracle.

`cli` runs the actual executable from an unrelated temporary working directory, renders an embedded fixture, loads the result back, rejects corruption, checks exit codes, and proves an existing destination stays unchanged. `feed` reaches the actual loopback server and distinguishes historical frames from an expired current head.

The browser test suite runs Chromium locally with synthetic public data. The native Windows Rust suite exercises Windows publication semantics and the same executable CLI; Unix-specific non-UTF-8 path and symlink tests are explicitly platform-specific. Full source/runtime/native/live/provider evidence must remain separately labeled.
