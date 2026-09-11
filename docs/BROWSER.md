# Browser viewer

The browser product is a small static, read-only surface in `web/`. It consumes the frozen `ascension-map-viewer-v1` presentation payload from `docs/VIEWER_PAYLOAD.md`; it does not infer permissions from graph adjacency and it has no game or provider credentials.

## Run it

From this repository, serve the `web/` directory on loopback:

```text
python3 -m http.server 4173 --bind 127.0.0.1 --directory web
```

Open `http://127.0.0.1:4173/index.html`. The hosted view asks only its own origin for `GET /api/current`, `GET /api/replay`, and `GET /api/frame/{safe bundle id}`. A server that does not implement those routes leaves the viewer visible and marks the source disconnected. The existing map stays intact until a complete, freshly validated payload arrives.

For an offline artifact, open `web/index.html` directly, choose a generated `viewer.json` with **Open artifact**, or use the bundled synthetic example. The file loader checks the UTF-8 byte limit before parsing, rejects duplicate object keys and unsupported shapes, and validates endpoint, alias, digest, identity, coordinate, collection, and text limits. An empty `bundle_id` is accepted only as an explicitly unbound offline artifact; live and replay responses must carry the descriptor's bound printable-ASCII ID. Imported paths are never fetched. The embedded example is synthetic and has no game connection.

## Interaction

Fit the whole declared graph with **Fit map** or `0`. The graph uses stable floor/lane coordinates and deterministic offsets for duplicate coordinates. Drag pans; the wheel and `+`/`−` zoom; the small overview remains visible during zoom. Tab reaches every node and directed edge. Enter inspects the focused element, and arrow keys move between a node's connected neighbours. `/` focuses search. Search accepts canonical IDs and display aliases.

The base graph remains in the SVG when an overlay is toggled. Legal, reachable, visited, and candidate-route controls change emphasis and annotations only. Nodes and directional edges have inspectable labels and shape/state treatments so color is not the only signal. Candidate rows compare the recorded node sequences and are never action controls. Node and edge inspection is always read-only.

Replay descriptors are capped at 4096 frames. Their `viewer` references are validated as root-confined relative metadata, but the browser never requests those paths. Selecting a frame calls only `/api/frame/{safe bundle id}`. A replay frame is labelled historical and its authorization is recorded-only. A failed live poll is shown as **Disconnected** and does not combine data from different bundle identities or generations.

## Browser checks

Install the locked development dependency and run the real browser suite:

```text
npm ci --no-audit --no-fund
npm run test:browser
```

The suite covers file loading, strict hostile-label rendering, fit/zoom, search, pointer and keyboard inspection, overlay preservation, candidate/replay state, fixed-route live loading, and desktop/mobile layouts. Playwright writes reports and screenshots under ignored `artifacts/` paths. No external network or provider call is needed.

When the generated artifacts and release CLI are available, `npx playwright test tests/browser/generated.integration.spec.js` runs the integration check separately from the synthetic fixtures. Generate fresh inputs from the checked-in conformance and demo sources before running it:

```sh
cargo build --release --locked --package map-visualizer
run_root="$(mktemp -d)"
small_bundle="$run_root/small"
dense_bundle="$run_root/dense"
target/release/map-visualizer render --bundle fixtures/conformance --out "$small_bundle"
target/release/map-visualizer demo --out "$dense_bundle"
ASCENSION_MAP_SMALL_BUNDLE="$small_bundle" \
ASCENSION_MAP_DENSE_BUNDLE="$dense_bundle" \
ASCENSION_MAP_VISUALIZER_BIN="$PWD/target/release/map-visualizer" \
npx playwright test tests/browser/generated.integration.spec.js
```

The check opens each exported `index.html` through `file://`, loads its `viewer.json` with the picker, verifies the complete graph and candidate summaries, records desktop/mobile screenshots for the dense artifact, and launches the release binary with `serve --bundle` to verify historical current/replay responses and fixed routes. Local runs skip with an explicit reason when these generated inputs are absent; CI treats missing inputs as a failure.

The view is intentionally bounded by the presentation contract: 2 MiB JSON input, 1024 nodes, 8192 edges, 4096 replay descriptors, signed 16-bit floor/lane coordinates, and 512 UTF-8 bytes per identity or text field. Canonical IDs and bundle IDs use printable ASCII; text labels reject control, bidi, and invisible formatting characters. Very dense or very tall maps remain complete but may require zooming to read labels. Raster overview images and game/native projection remain owned by the Rust adapter and upstream producers.
