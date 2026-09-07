# Viewer presentation payload v1

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

The packaged HTML/JS can open generated `viewer.json` offline. A served viewer reads the exact same payload through `GET /api/current` on its own loopback origin and polls only that fixed route with bounded concurrency. It never uses a URL from an imported file. `GET /api/replay` supplies at most 256 immutable frame descriptors (`bundle_id` and a root-confined relative `viewer` reference); selected frames load through a fixed server route. Disconnected status is separate from recorded freshness and becomes visible after a failed poll. Reconnection replaces the complete view atomically, never combines nodes across generations.

The base graph always remains present. Overlay toggles only change emphasis; routes are annotations. Fit-all, zoom/pan, keyboard focus/inspection, node/edge inspection, search by alias/canonical ID, candidate comparison, legend, replay selection, and a persistent overview operate locally without game/provider credentials.
