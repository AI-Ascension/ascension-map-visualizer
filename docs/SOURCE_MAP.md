# Source map

Paths below are relative to their owner repository. They identify integration seams, not a claim that any companion branch has been merged or deployed.

| Repository | Source seam | Responsibility |
|---|---|---|
| `sts2-protocol` | `crates/protocol/src/visible_map/`, `schemas/runtime-map-v1.schema.json`, `artifacts/runtime-map-v1/` | Neutral bounded map types, strict decoding, semantic validation, canonical digests, conformance vectors |
| `sts2-game-mod` | `experiments/managed-rust-interop/game-loader/LiveCombatSource.MapProjection.cs`, `LiveCombatSource.MapSupport.cs`, `RuntimeMapV1*.cs` | Player-permitted native scope, stable map identities, current legal bindings, main-thread snapshot and fixed native route |
| `sts2-game-mod` | `LiveCombatSource.MapNavigation.cs`, `RuntimeV3GameplayObservation.cs` in the same loader directory | Existing next-choice action catalog and gameplay observation; kept distinct from complete topology |
| `sts2-gateway` | `crates/gateway/src/bin/runtime_support/service_map.rs`, `service_lease.rs`, `runtime_map_forwarder.rs` | Fixed authenticated route, authority fences, bounded downstream response validation |
| `sts2-mcp-server` | `crates/mcp-server/src/mapping_runtime_map.rs`, `projection_runtime_map.rs`, `bin/runtime_support/binding.rs` | Explicit map tool/profile, request authority, validated response projection |
| `sts2-harness` | `crates/harness/src/map/` and `docs/map-*.schema.json` | Graph analysis, bounded candidates, cache, bundle/feed/replay, evaluator and independent artifact contracts |
| `sts2-harness` | `crates/harness/src/episode/map*.rs`, `exo/protocol/request.rs`, `exo/sandbox/schema.rs` | Typed map context, fair-play allowlist, complete graph and matching image request representation |
| `sts2-harness` | `crates/harness/src/bin/runtime_support/runtime_map_wire.rs`, `runtime_v3_map_renderer.rs`, `bin/astra_support/` | Actual MCP process acquisition, official visualizer execution, reviewed provider serialization |
| `ascension-map-visualizer` | `crates/map-visualizer/src/bundle.rs`, `contracts.rs`, `view.rs` | Consumer validation of exact artifacts and presentation adapter, without harness policy duplication |
| `ascension-map-visualizer` | `crates/map-visualizer/src/drawing/`, `render_bundle.rs` | Original deterministic glyphs/layout, SVG/PNG rasterization, updated matching manifest |
| `ascension-map-visualizer` | `source.rs`, `feed.rs`, `storage.rs`, `publication.rs`, `server.rs`, `http.rs` | Bounded root-confined acquisition, atomic publication, read-only loopback delivery |
| `ascension-map-visualizer` | `web/`, `arguments.rs`, `cli.rs`, `demo.rs` | Human interaction, offline executable, embedded demo and explicit CLI grammar |
| `ai-agent-observability` | `docs/MAP_ARTIFACTS.md` | Existing collector/artifact privacy and operating integration; no new stack |

No game-core change is justified by this implementation: host facts, protocol validation, harness analysis, and visual presentation already have distinct owners. Baseline organization/game-core commits are still included in the assembly inventory.
