# Viewer presentation payload v1

Checkpoint-bearing owner bundles use `sts2.map-view-bundle-v2` and publish
`ascension-map-viewer-v2`; legacy bundles continue to produce the unchanged v1 payload above.
The v2 viewer requires one additional `checkpoint` object with exactly `reference`, `run_id`,
`episode_id`, `trajectory_id`, and `dispatchable` (always false). `reference` follows the frozen
protocol `ascension.exact_checkpoint_reference.v1` / `exact-checkpoint-reference-v1` public
shape: opaque `ckpt-h1:` handle, occurrence, boundary kind/phase, producer assurance, and
restore-verification boolean. Unknown versions, additional fields, malformed handles, and
inconsistent assurance/verification are rejected before admission. No exact-state, checkpoint,
compatibility, or blob digest is admitted in this reference.

The evidence panel labels assurance and verification as producer claims, shows the recorded
run/episode/trajectory lineage, and grants no dispatch or restore authority. Text is inserted
through DOM text nodes; imported markup is never executed. Historical frames remain historical,
including after a verified checkpoint reference is displayed. Returning to a legacy frame clears
the panel. The visualizer does not infer checkpoint equality, verify restored gameplay, or retrieve
privileged artifacts from a public handle.

The v2 operational feed may retain mixed v1/v2 bundle history. Each entry's optional reference must
match the referenced bundle exactly, including absence. A stale cached frame cannot make an altered
reference pass admission; invalid updates leave the previous validated view intact.

The Rust adapter produces `viewer.json` only after validating the upstream snapshot, analysis and manifest. This is a presentation artifact, not an alternative protocol or analysis authority. The browser never interprets graph adjacency as permission to act.

```json
{
  "schema": "ascension-map-viewer-v1",
  "bundle_id": "example",
  "snapshot_digest": "64-lowercase-hex-digits",
  "analysis_digest": "64-lowercase-hex-digits",
  "map": {
    "identity": "opaque map identity",
    "status": "AVAILABLE / COMPLETE / CURRENT",
    "nodes": [{"id":"n1","floor":0,"lane":0,"category":"rest","current":false,"legal":true,"reachable":true,"visited":false}],
    "edges": [{"from":"n1","to":"n2","selected":false}]
  },
  "aliases": {"n1":"N0001"},
  "candidates": [{"id":"route-1","nodes":["n1","n2"],"summary":"2 steps; 1 visible rest","selection":"heuristic"}],
  "historical": false,
  "complete": true,
  "available": true,
  "warnings": []
}
```

The example illustrates fields only, not a valid graph. Actual payloads require unique bounded nodes and aliases, validated endpoints, complete alias coverage, exact bundle/input digests, and candidates referencing existing nodes. Identity and text limits are 512 UTF-8 bytes; maps support at most 1024 nodes and 8192 edges at the presentation boundary, bounded more strictly by the upstream capability. All booleans and arrays are required; unknown root/map/node/edge/candidate fields are rejected. The browser must treat every string as data and never execute imported SVG or HTML.

The packaged HTML/JS can open generated `viewer.json` offline. A served viewer reads the exact same payload through `GET /api/current` on its own loopback origin and polls only that fixed route with bounded concurrency. It never uses a URL from an imported file. `GET /api/replay` supplies at most 4096 immutable frame descriptors (`bundle_id` and a root-confined relative `viewer` reference); selected frames load through a fixed server route. Disconnected status is separate from recorded freshness and becomes visible after a failed poll. Reconnection replaces the complete view atomically, never combines nodes across generations.

The base graph always remains present. Overlay toggles only change emphasis; routes are annotations. Fit-all, zoom/pan, keyboard focus/inspection, node/edge inspection, search by alias/canonical ID, candidate comparison, legend, replay selection, and a persistent overview operate locally without game/provider credentials.
