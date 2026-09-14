# Contract artifacts

Protocol authority stays in `AI-Ascension/sts2-protocol`, consumed as a neutral Rust package at the exact revision in the product Cargo manifest and lockfile. Its legacy gameplay profile remains unchanged.

`harness/` contains inert copies of the independently owned analysis, bundle, and operational-feed JSON schemas. `harness/provenance.json` records each copied artifact's exact source commit, source/copy paths, license, and SHA-256 values, together with the copied synthetic fixtures. A single artifact set may legitimately span more than one historical owner commit (for example, a schema refreshed at a later pin while its compatible fixtures remain at their original bundle commit), so each entry carries its own commit. Never patch a schema copy without an accepted corresponding owner change and a refreshed provenance record.

Version matching and hashes establish compatibility and content identity. They do not authorize a game action, prove that a producer is deployed, or establish native player-visible scope.
