# Contributing

Follow the AI-Ascension organization governance and `AGENTS.md`. Keep one cohesive task per branch and open a pull request; only maintainers merge and publish releases. Existing branch protection and review requirements remain in force.

The visualizer owns presentation, its bounded read-only CLI/server, and validation adapters for accepted upstream contracts. Protocol owns the snapshot; the harness owns graph analysis, bundles, route intent and provider delivery. Do not import sibling implementation internals or add game/provider credentials to this product.

Before changing a dependency, external interface, listener or retention rule, reference an implementation issue and record the owner decision. Preserve unrelated work and stage exact owned paths. Rust source should stay below 400 nonblank lines per production file and 600 per test file; split cohesive responsibilities before files exceed those limits.

Run formatting, locked Clippy with warnings denied, workspace tests, browser tests when affected, and `git diff --check`. Add meaningful independent expectations and bounded parser/loader fuzzing. Never weaken a failing assertion or change a golden artifact without inspecting why the result changed. Record source, build, synthetic, native, provider, and deployment evidence separately.

All contributions are MIT licensed. Never include proprietary game assemblies, assets or saves, private transcripts, credentials, personal paths or unlicensed fixtures. The complete implementation assignment in `prompts/` governs acceptance until fulfilled.
