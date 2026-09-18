# Live-lane pin matrix and map env-contract record — 2026-09-17

Scope: acceptance criterion 1 preparation for
[Map#14](https://github.com/AI-Ascension/ascension-map-visualizer/issues/14) (`--campaign-map` lane).
Status: **source-derived record; nothing live verified**. Every value below was read from the
named commits with `git show`/`git grep` on 2026-09-17 after `git fetch origin` in local clones.
No host, Steam client, gateway, MCP server, harness runtime or provider was started. This record
does not decide the contract question in the last section; that belongs to the harness and game-mod
owners. Machine-readable copy: [live-lane-pin-matrix-20260917.json](live-lane-pin-matrix-20260917.json).

## 1. Current heads

| Component | Repository | Default | Head on 2026-09-17 | Note |
| --- | --- | --- | --- | --- |
| Protocol | AI-Ascension/sts2-protocol | `main` | `bfe28e455de48d6d9db466bbcf6062ab5d85e9af` | `schemas/runtime-map-v1.schema.json` SHA-256 `ceab0d2dfc471d1ec36d12edaf4654b8c7fdced06548bf47265e11c63f98115b`, identical to `artifacts/runtime-map-v1/schema.json` and to the digest recorded in [ASSEMBLY_MANIFEST.json](../ASSEMBLY_MANIFEST.json) for `0bc689ea` |
| Gateway | AI-Ascension/sts2-gateway | `main` | `804691c5e1b3121075d83b3e31b86606facb5e62` | runtime binary `sts2-gateway-runtime` (`crates/gateway/src/bin/sts2-gateway-runtime.rs`) |
| MCP server | AI-Ascension/sts2-mcp-server | `main` | `65cb405616ecddfb7bf7b76edf341be30a442ca2` | binary `sts2-mcp-server` (`crates/mcp-server/src/bin/sts2-mcp-server.rs`) |
| Harness | AI-Ascension/sts2-harness | `main` | `41c2d1171beed7b521dced79b9bf66ce75d7f196` | binaries `sts2-harness-runtime`, `sts2-astra-bridge` (`crates/harness/src/bin/`). Advanced past `f6736580` (cited in the 2026-09-17 verification) by two test-only commits `deb5df6d`, `41c2d117`; map env readers unchanged |
| Game mod | AI-Ascension/sts2-game-mod | `main` | `655d3954c8049978df2b6505d62c9619f3bdca86` | Advanced past `46b1ac6e` by `857baf1`, `655d395` (docs/schema). Launcher `experiments/managed-rust-interop/live-combat-session.sh` and runbook `docs/LIVE_COMBAT_DEMO.md` last changed at `da89456163e0a57d705b2099c2f8ff3d4fce5bb7` (2026-09-11); runbook byte-identical to the issue's `c947a930` link (`git diff --stat` empty) |
| Visualizer | AI-Ascension/ascension-map-visualizer | `bootstrap` | `3370db162be8a0618da1c846327be3facf86dd2b` | `node tools/build-release.mjs` on linux-x86_64 produced `map-visualizer` SHA-256 `554cf5b9c8e74d8b9154f3ee9c3fb25e1ee91b4365e6b67ea817235f4334b679`; `doctor` ran. Cross-machine reproducibility of that digest is `unverified` |
| Bridge revision | derived at launch | — | not computed | `STS2_EXO_REVISION` is not a commit: the launcher sets it to the SHA-256 of `--provider-binary` (`live-combat-session.sh:324-325`) and records it in the run manifest (`:357`). Historical value `d608c0290ce3f2b59b1c0be8a5e78c122f31cdede99626d5680e761ba66bbcd9` for `sts2-astra-bridge` at harness `423d9052`. No bridge was built at `41c2d117`; the current value is `unverified` |
| Host | disposable copy, outside all repositories | — | `sts2.dll` v0.107.1 release `59260271` | SHA-256 `a1f9e653f1e28e4076558fee1e60d218619cb7e057b887c6417f62c62c6d7a52`; `GodotSharp.dll` `0e4897ecdfb31456a97c7d8028dfb8d7dbdc632e2f73fc9b438d7b266a139289`; enforced by the launcher (`live-combat-session.sh:14-15`, manifest `:361`) |

Contract copies in this repository: the 13 files under `contract-artifacts/harness/` pinned to
harness `67e3c241` are byte-identical to the same paths at harness `41c2d117` (local SHA-256
comparison; `node tools/contracts.mjs` verified the copies). `67e3c241` is an ancestor of `41c2d117`.

Historical prepared executables (gateway `4b306321`, MCP `0d2035a0`, harness `423d9052`,
visualizer `5b1d196`) remain in [ASSEMBLY_MANIFEST.json](../ASSEMBLY_MANIFEST.json) with
`live_verified:false`. No executable has been built from the current heads except the visualizer.

## 2. Map env-contract mismatch (launcher vs harness)

Launcher side, game-mod `655d3954`, `experiments/managed-rust-interop/live-combat-session.sh`,
inside `if [[ "$campaign_map" == true ]]; then` (line 339):

```text
342:    export STS2_MAP_ARTIFACT_ROOT="$run/map-artifacts"
343:    export STS2_CAMPAIGN_MAP_BOUND=true
344:    export STS2_LIVE_EPISODE=true STS2_MAP_MODE=graph-image STS2_MAP_RENDERER_BINARY="$map_renderer"
345:    export STS2_MAP_RENDERER_SHA256="$map_renderer_sha256" STS2_MAX_STEPS=2
346:    export STS2_RECOVERY_MAX_ATTEMPTS=1 STS2_EXO_TIMEOUT_MILLIS=90000
```

`git grep STS2_ENABLE_MAP_CONTEXT 655d3954` in game-mod: zero hits. The runbook states the intent
at `docs/LIVE_COMBAT_DEMO.md:126`: "The launcher also enables `STS2_CAMPAIGN_MAP_BOUND=true` in the
harness." The host-side flag is internal to game-mod and consistent with itself
(`live-combat-demo.ps1:59` sets `STS2_LIVE_CAMPAIGN_MAP_BOUND=1`;
`game-loader/LiveCombatDemo.cs:20-21` reads it); it is not part of the mismatch.

Harness side, `41c2d117`:

```text
crates/harness/src/bin/runtime_support/config.rs:92:
        let map_context_enabled = flag_with_default("STS2_ENABLE_MAP_CONTEXT", false)?;
crates/harness/src/bin/runtime_support/runtime_v3_settings.rs:172:
            "STS2_EXO_MAX_REQUEST_BYTES must be {EXO_MAX_MAP_REQUEST_BYTES} when STS2_ENABLE_MAP_CONTEXT is true"
crates/harness/src/episode/runner_steps.rs:207-208:
        if self.config.map_context_enabled()
            && observation.stage() == super::super::observation::EpisodeStage::Map
crates/harness/src/map/evaluation_renderer.rs:39:
    let Ok(binary) = std::env::var("STS2_MAP_RENDERER_BINARY") else {
crates/harness/src/map/evaluation_renderer.rs:42:
    let Ok(expected_digest) = std::env::var("STS2_MAP_RENDERER_SHA256") else {
docs/COMPATIBILITY.md:370-371:
Map context is an additive, opt-in provider-request capability. `STS2_ENABLE_MAP_CONTEXT` defaults to
`false` and accepts only the exact values `true` or `false`.
experiments/exo-agent/README.md:50-52:  STS2_RUNTIME_PROFILE=runtime-v3-gameplay / STS2_ENABLE_MAP_CONTEXT=true / STS2_EXO_MAX_REQUEST_BYTES=393443
```

`git grep` at `41c2d117` for `STS2_CAMPAIGN_MAP_BOUND`, `STS2_MAP_MODE`, `STS2_MAP_ARTIFACT_ROOT`:
zero hits each (source, docs and tests). `render_image` (the only reader of
`STS2_MAP_RENDERER_BINARY`/`STS2_MAP_RENDERER_SHA256`) is called from
`crates/harness/src/map/evaluation_matrix.rs:92` only, the `sts2-map-evaluation-matrix` tool; no
file under `crates/harness/src/episode/` or `crates/harness/src/context_control/` references an
image or PNG. The live `sts2-harness-runtime` at main therefore renders no image.

Per-variable matrix for the `--campaign-map` block (harness hits are `"NAME"` string occurrences
in `crates/**/*.rs` at `41c2d117`):

| Variable | Launcher | Harness `41c2d117` | Result at current heads |
| --- | --- | --- | --- |
| `STS2_CAMPAIGN_MAP_BOUND=true` | exported `:343` | 0 hits | ignored; no bound start_run/select_map_node policy |
| `STS2_MAP_MODE=graph-image` | exported `:344` | 0 hits | ignored; no mode selection exists |
| `STS2_MAP_ARTIFACT_ROOT` | exported `:342` | 0 hits | ignored; no `map-artifacts/` bundles or feed written |
| `STS2_MAP_RENDERER_BINARY` / `_SHA256` | exported `:344-345` | `map/evaluation_renderer.rs:39,42` (evaluation tool only) | ignored by the live runtime binary |
| `STS2_ENABLE_MAP_CONTEXT` | never set | `config.rs:92`, default `false` | map context disabled: `runner_steps.rs:207` never requests a map snapshot |
| `STS2_EXO_MAX_REQUEST_BYTES` | never set | `runtime_v3_settings.rs:137`; map bound required when the flag is true | standard default applies; would need `393443` if the flag were enabled |
| `STS2_MAX_STEPS=2`, `STS2_RECOVERY_MAX_ATTEMPTS=1`, `STS2_EXO_TIMEOUT_MILLIS=90000`, `STS2_OBJECTIVE`, `STS2_HARD_CONSTRAINTS_JSON`, `STS2_LIVE_EPISODE`, `STS2_EXO_REVISION`, `STS2_EXO_BRIDGE_*`, gateway/lease identifiers | exported `:315-347` | read (`config.rs`, `runtime_v3_settings.rs`, `runtime_v3_admission.rs:79`) | honored |

Consequence (source-derived): running the runbook block at current heads starts
`sts2-harness-runtime` with map context disabled and no bound policy, while the launcher's
`verify_campaign_map_trace` (`live-combat-session.sh:253-260`, `live-campaign-trace.jq`) still
requires an ordered trace with image, bundle and map-action records. The lane cannot produce the
evidence Map#14 asks for until the contract is reconciled.

Where the launcher's names came from: harness commit `c4574555bf383014479b5651b59e0f90c0a40715`
(2026-09-07, "feat(runtime): measure map pipeline work and document cache bounds") reads
`STS2_CAMPAIGN_MAP_BOUND` (`crates/harness/src/bin/runtime_support/runtime_v3_campaign.rs:28,31`),
`STS2_MAP_MODE` (`runtime_v3_campaign.rs:37`, `runtime_v3_settings.rs:61,66`) and
`STS2_MAP_ARTIFACT_ROOT` (`runtime_v3_episode_map.rs:210-226`) and documents them in
`docs/map-campaign-validation.md:3,5,26`. That commit is not an ancestor of harness `main`, not an
ancestor of PR #35's head `51a7d89a`, not among PR #35's 22 commits, and on no remote branch. It
reads no `STS2_ENABLE_MAP_CONTEXT`; the two contracts have never coexisted on one harness revision.
[harness-assembly.json](harness-assembly.json) and [live-attempts.json](live-attempts.json) record
`c4574555` as the source of the historical synthetic checks and the first two failed live attempts.

## 3. Runbook command block with concrete pins

Block from game-mod `655d3954` `docs/LIVE_COMBAT_DEMO.md` (unchanged since `c947a930`). Every
binary digest is taken by the launcher at run time into `binaries.sha256` (`:389`); none has been
built from the current heads except the visualizer, so the digests below are labelled.

| Field | Runbook value | Concrete pin on 2026-09-17 | Label |
| --- | --- | --- | --- |
| `--host-dir` | `/path/to/disposable-sts2` | disposable copy of STS2 v0.107.1 release `59260271`; `sts2.dll` `a1f9e653…d7a52`, `GodotSharp.dll` `0e4897ec…9289` (launcher `:14-15`) | source-derived (hashes); host availability `unverified` |
| `--user-dir` | `'C:\Temp\sts2-map-user-20260907'` | operator-chosen absolute Windows path outside host/repo/addon/artifacts | operator input |
| `--addon-dir` | `/path/to/disposable-sts2/mods` | `AIAscensionSTS2GameMod.dll`, `AIAscensionSTS2GameModNative.dll`, `AIAscensionSTS2GameMod.json` staged from game-mod `655d3954`; hashes recorded to `addon.sha256` at run time | `unverified` (no current-head addon built) |
| `--artifacts-dir` | `/tmp/sts2-map-artifacts` | external directory; receives `baseline.sha256`, `addon.sha256`, `binaries.sha256`, `trajectory.jsonl`, `map-artifacts/` | operator input |
| `--gateway-binary` | `/path/to/sts2-gateway-runtime` | build `sts2-gateway-runtime` from gateway `804691c5`; historical `2c4af205…76a5` at `4b306321` | `unverified` (current-head digest not computed) |
| `--mcp-binary` | `/path/to/sts2-mcp-server` | build `sts2-mcp-server` from MCP `65cb4056`; historical `edd40879…85f1` at `0d2035a0` | `unverified` |
| `--harness-binary` | `/path/to/sts2-harness-runtime` | build from harness `41c2d117`; historical `58d1fc0e…999a` at `423d9052`. Subject to section 2 | `unverified` |
| `--provider-binary` | `/path/to/sts2-astra-bridge` | build from harness `41c2d117`; its SHA-256 becomes `STS2_EXO_REVISION`; historical `d608c029…bcd9` at `423d9052`; requires provider kind `openai-astra` (`:154-157`) | `unverified` |
| `--map-renderer-binary` | `/path/to/map-visualizer` | visualizer `3370db16`, `node tools/build-release.mjs`, linux-x86_64: `554cf5b9c8e74d8b9154f3ee9c3fb25e1ee91b4365e6b67ea817235f4334b679`; must live outside the game-mod repository (`:139-144`) | source-derived (this machine); reproducibility `unverified` |
| `--map-renderer-sha256` | `LOWERCASE_SHA256` | the digest above for that exact file; launcher exits 2 on mismatch (`:149-153`) | same as above |
| `--powershell-binary` | `/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe` | WSL path to the host PowerShell; runs `live-combat-demo.ps1` with `-CampaignMapBound` (`:296`) | host tool, no repository pin |
| `--campaign-map` | flag | requires `--run-kind campaign --campaign-mode standard` (defaults), no seed, no replay (`:133-137,158-161`) | source-derived |

## 4. Acceptance criteria of Map#14

All six remain `unverified`. External gate text is quoted from the issue.

| AC | Criterion (abridged) | Status | Gate |
| --- | --- | --- | --- |
| AC1 | Reverify and record compatible protocol, host, gateway, MCP, harness, reviewed bridge and visualizer pins | `unverified` — pins recorded here; a *compatible* tuple does not exist until section 5 is decided | owner decision (harness or game-mod) |
| AC2 | "Use the owner campaign/map runbook … and the existing `--campaign-map` lane, with at most two model decisions under the owner's approved budget." | `unverified` | authorized disposable Windows/Steam host + provider budget |
| AC3 | "Verify native permitted topology, including all in-scope nodes/edges and zero read side effects, through the real fenced transport." | `unverified` (`acceptance.json` `host_map_snapshot`) | same host |
| AC4 | "Capture sanitized evidence that the real provider receives the complete graph and matching image; distinguish delivery from model comprehension." | `unverified` (`provider_graph_image_input`; only synthetic [provider-cli-capture.json](provider-cli-capture.json)) | host + real provider run |
| AC5 | "Verify a current legal map action settles through existing host witnesses and is followed by fresh observation." | `unverified` (`settled_map_action`) | host |
| AC6 | "Record cleanup/baseline restoration, exact commands, results, artifact digests, and independent review." | `unverified` — no run to record | after a run; independent reviewer |

## 5. Open owner decision (recorded, not decided)

Two reconciliations are possible; this record chooses neither.

- Harness re-lands the bound policy on `main`: readers for `STS2_CAMPAIGN_MAP_BOUND`,
  `STS2_MAP_MODE`, `STS2_MAP_ARTIFACT_ROOT`, plus live image rendering and bundle/feed output as at
  `c4574555`, aligned with the trace shape `live-campaign-trace.jq` expects. Launcher and runbook
  stay as they are.
- Launcher switches to the harness-main contract: export `STS2_ENABLE_MAP_CONTEXT=true` and
  `STS2_EXO_MAX_REQUEST_BYTES=393443`, drop or redefine the three unread names, and revise
  `docs/LIVE_COMBAT_DEMO.md:126`, the run manifest fields (`:382-386`) and
  `verify_campaign_map_trace` to the graph-only, no-artifact-root behavior that harness `main`
  implements (no image, no bound action policy).

Either side must be merged and its pins recorded before any host handoff; a mixed tuple would run
with map context disabled and could not satisfy AC3-AC5. The decision belongs to the harness and
game-mod owners under Map#1 coordination.
