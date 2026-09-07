# Ascension Map Visualizer

A read-only, offline-capable viewer and deterministic Rust SVG/PNG renderer for player-visible Slay the Spire 2 map bundles. The game mod supplies permitted topology and current legal bindings; the harness supplies analysis and recorded decision context. This product validates and presents those artifacts.

**Implementation is in progress.** The offline product has passing synthetic tests. Complete native host/provider integration, final companion pins, independent review, and platform packaging have separate acceptance gates in [implementation status](docs/IMPLEMENTATION_STATUS.md). No release or deployed gameplay compatibility is implied.

## Run the synthetic demonstration

Build with the pinned Rust toolchain, then generate and serve an embedded, harness-generated synthetic map:

```sh
cargo build --release --locked --package map-visualizer
./target/release/map-visualizer demo --out ./map-demo --serve
```

On Windows use `target\release\map-visualizer.exe`. Open the printed loopback URL. Ctrl-C stops the server. The output directory must not already exist. The executable embeds its demo and browser assets and requires no game files, credentials, provider calls, or observability service. Initial source builds need access to the pinned dependencies.

The output contains the validated bundle, `overview.svg`, `overview.png`, `viewer.json`, and an offline browser app. Open `index.html` and select `viewer.json` using the file picker for fully offline inspection. Bundle-provided SVG/HTML is never executed by the server.

## Commands

```sh
map-visualizer doctor
map-visualizer validate --bundle ./map-demo
map-visualizer render --bundle ./input-bundle --out ./new-render --width 2400 --height 3600
map-visualizer serve --bundle ./map-demo --port 18791
map-visualizer serve --root ./harness-artifacts --port 18791
map-visualizer replay --root ./harness-artifacts --port 18791
```

`doctor` reports native platform, contract identities, renderer versions, limits, and capabilities. `validate` checks strict JSON, protocol and harness schemas, lineage, digests, viewer correspondence, and exact image reproduction for this renderer. `render` atomically publishes a new directory and refuses replacement. Standalone bundles and replay frames are historical and never dispatchable.

`serve --root` follows the harness's bounded `feed.json`; `replay --root` loads historical frames without polling. Feed bundles live in directories named by their bundle digest. The default port is zero, which chooses an available loopback port. No command accepts a public bind address, remote artifact URL, game credentials, or a gameplay action.

Exit codes: `0` success; `2` invalid arguments; `3` rejected artifact; `4` I/O, rendering, or server failure. `help` prints the complete flag grammar. Image dimensions are 320–8192 wide and 320–16384 high, with at most 33,554,432 pixels. Limits are validated before allocation.

## Viewer and replay

The browser supports fit-all, zoom/pan, keyboard inspection, canonical/display ID search, node/edge details, route comparison, visible overlay controls, and historical frame navigation. The whole graph remains present when routes are highlighted; annotations never dispatch actions. See [browser controls](docs/BROWSER.md) and the [viewer payload contract](docs/VIEWER_PAYLOAD.md).

Aliases identify presentation nodes and retain the canonical mapping. Current, legal, visited, unreachable, unknown, stale, and incomplete states use labels/shapes as well as color. Layout follows logical floor/lane positions with deterministic disambiguation for duplicate coordinates. Dense maps may require higher resolution and interactive detail inspection; readability warnings never authorize cropping the graph.

## Integration and operations

The supported architecture is game host → game mod → gateway → MCP → harness → bundle/feed → visualizer. The visualizer cannot contact the game or grant authority from a stored action ID. See [ownership and rollout](docs/decisions/0001-ownership-and-additive-map-capability.md), [operations](docs/OPERATIONS.md), [privacy](SECURITY.md), and [acceptance tracking](docs/acceptance.json).

The protocol is consumed from its exact Git revision in `Cargo.toml` and `Cargo.lock`. Harness schemas are inert copied contracts under `contract-artifacts/`. Analysis policy remains in the harness; local graph presentation does not estimate rewards or future legality.

## Build, test, and package

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo deny check
npm ci --ignore-scripts
npx --no-install playwright install chromium
npm run test:browser
cargo build --release --locked --package map-visualizer
node tools/package.mjs /tmp/map-distribution
```

Choose a new output path appropriate for the operating system. Packaging supports native Linux x86-64 and Windows x86-64/MSVC, includes dependency notices and checksums, and creates a `.tar.gz` beside the distribution directory. Extract the archive to preserve Linux executable permissions. `build.json` records the native target, source commit, and dirty-worktree flag. CI builds and tests on both native platforms and retains build artifacts; it does not publish a release.

All symbols/glyphs are original project assets. See [LICENSE](LICENSE) and [third-party notices](THIRD_PARTY_NOTICES.md).
