# Third-party notices

This project is MIT licensed. The original map symbols and block glyphs are authored for this project and contain no game assets or operating-system fonts.

Direct runtime dependencies are pinned in `Cargo.toml` and resolved in `Cargo.lock`:

| Package | License | Purpose |
| --- | --- | --- |
| resvg 0.48.1 | Apache-2.0 OR MIT | Deterministic rasterization of project-authored SVG |
| serde 1.0.229 | MIT OR Apache-2.0 | Typed inert contract serialization |
| serde_json 1.0.151 | MIT OR Apache-2.0 | Bounded JSON adapters |
| sha2 0.11.0 | MIT OR Apache-2.0 | Content digests |
| cap-std 4.0.3 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | Filesystem confinement by directory capability |

`resvg` has no default features enabled: system fonts, font memory mapping, SVGZ and external raster-image decoders are excluded. Only internally generated SVG reaches the rasterizer. Its transitive `usvg`, `tiny-skia`, PNG and geometry dependencies retain their upstream notices.

Development-only property tests use proptest 1.11.0 (MIT OR Apache-2.0) and tempfile 3.27.0 (MIT OR Apache-2.0). Browser test dependencies and lockfile are recorded separately in `package-lock.json` when installed. Transitive license metadata is reported by `cargo metadata --locked --format-version 1`; the release package includes the resolved license inventory.

Neutral STS2 contracts and harness-owned bundle schemas remain under their upstream MIT license. Copied inert artifacts include provenance and exact source revisions. No upstream product implementation is copied into this repository.
