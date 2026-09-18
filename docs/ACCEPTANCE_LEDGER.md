# Map acceptance ledger — 2026-09-15

This audit covers every section 18 criterion and section 17 gate in the
[original assignment](../prompts/ASCENSION_MAP_VISUALIZER_ORCHESTRATION_PROMPT.md).
The feature remains incomplete. GitHub delivery state is distinct from the immutable
test pins in existing evidence. No historical test was rerun or promoted to native
acceptance by this documentation audit.

## Current delivery

The product default branch is `bootstrap`, inspected at
`abc0249d1852faaefd59260d63058ec59e833353`. GitHub confirms the following merges:

| Owner | PR | Merge commit |
| --- | --- | --- |
| Visualizer product | [#2](https://github.com/AI-Ascension/ascension-map-visualizer/pull/2) | `b7b5c6ef3ac0cdb302fe01f989321c12186d6d9b` |
| Visualizer provenance | [#7](https://github.com/AI-Ascension/ascension-map-visualizer/pull/7) | `b4dd51ba8acbf3c4f9cd88a89b76b360c737b7a2` |
| Visualizer protocol pin | [#9](https://github.com/AI-Ascension/ascension-map-visualizer/pull/9) | `2f7a626e1dba8ba4e2de65a003201c328800f9ae` |
| Visualizer availability | [#11](https://github.com/AI-Ascension/ascension-map-visualizer/pull/11) | `cf3ffed278d4f5471fcaf6c1902a620bd2804953` |
| Visualizer dependency | [#13](https://github.com/AI-Ascension/ascension-map-visualizer/pull/13) | `abc0249d1852faaefd59260d63058ec59e833353` |
| Protocol | [#18](https://github.com/AI-Ascension/sts2-protocol/pull/18) | `7c448bd8d7a695ada48830176f3d738286caafe4` |
| Game mod | [#54](https://github.com/AI-Ascension/sts2-game-mod/pull/54) | `1ec9431de14070f8f94fc281d44b3ae5d41573a0` |
| Gateway | [#19](https://github.com/AI-Ascension/sts2-gateway/pull/19) | `02c816b6ac43c6b5b62292da0e5f19259c414773` |
| MCP | [#24](https://github.com/AI-Ascension/sts2-mcp-server/pull/24) | `89b20245d6ac8b9f59dd5655dc2fca973e04bd9f` |
| Harness | [#35](https://github.com/AI-Ascension/sts2-harness/pull/35) | `4c5b1012efb0b8ffd0e95b76654678b170de4720` |

Observability [PR #12](https://github.com/AI-Ascension/ai-agent-observability/pull/12)
was **closed without merge**, superseded by the reviewed
[MAP_ARTIFACTS.md at `c309c8e`](https://github.com/AI-Ascension/ai-agent-observability/blob/c309c8e532574754609d9491f0a10be584f83a83/docs/MAP_ARTIFACTS.md).
That source retains bounded loopback export, privacy, backend retrieval evidence and
exporter-off rollback. It is a proposed integration contract, not backend ingestion evidence.

Game-mod [#56](https://github.com/AI-Ascension/sts2-game-mod/issues/56) is closed following
[PR #75](https://github.com/AI-Ascension/sts2-game-mod/pull/75), merged as
`8c4adc8f7475c45774dbd621164655fa5173a6a3`. Its completion record is launcher/preflight
source and synthetic evidence. It neither establishes current Steam readiness nor
proves the integrated map lane. That lane remains with
[Map#14](https://github.com/AI-Ascension/ascension-map-visualizer/issues/14).

## Section 18: criterion-to-evidence mapping

Evidence links below retain their own exact source pins. “Historical component” means
the cited record reports scoped checks; it is not a new check of the current default branches.

| ID | Original criterion | Evidence, test and owner | Status / remaining task |
| --- | --- | --- | --- |
| D01 | Repository, product, policy, documentation and reproducible builds | Visualizer #2 and repairs above; native `doctor`, demo, validation and archive checks at `5b1d196` in [native packaging](evidence/native-packaging.json) and [privacy review](evidence/release-privacy-review.md) | Historical component evidence; refreshed coherent release checks belong to R3 |
| D02 | Actual Luna Max settings, three descendant layers, bounded depth/concurrency | [Observed orchestration](orchestration-observed.json) records fourteen Luna/max tasks, maximum depth one and peak eight | Incomplete: R1; historical missing child tools are not a claim about current tool availability |
| D03 | Every permitted native node/edge, explicit completeness, no hidden leakage | Game-mod source/probe and campaign records in [implementation status](IMPLEMENTATION_STATUS.md) and [campaign review](evidence/campaign-guard-review.json) | Native topology comparison remains unverified: Map#14 |
| D04 | Real fenced gateway/MCP/harness transport, legacy compatibility | Merged protocol/gateway/MCP/harness PRs; pinned [harness assembly](evidence/harness-assembly.json); [contract inventory](../contract-artifacts/harness/provenance.json) | Historical component evidence; current coherent transport/legacy matrix: R3; native lane: Map#14 |
| D05 | Stable graph identities separate from current action identities | Harness graph oracle, fresh catalog/payload and stale-generation tests in [cache review](evidence/runtime-cache-review.json); [contract checks](../tools/contracts.mjs) | Historical source/synthetic evidence; current replay/transport corroboration: R2/R3 |
| D06 | Provider gets the full graph and matching image; unsupported capability explicit | Actual bridge plus fake CLI capture at pinned source in [provider capture](evidence/provider-cli-capture.json) and [replay capture](evidence/provider-cli-capture-replay.md) | Synthetic delivery passes; real provider/image and settled action remain unverified: Map#14 |
| D07 | Deterministic bounded route analysis, truthful estimates, no stale dispatch | Nine-test graph oracle, four-mode measured synthetic evaluation at `c457455` in [harness assembly](evidence/harness-assembly.json); fresh binding checks in [cache review](evidence/runtime-cache-review.json) | Historical component evidence; current evaluation reproduction and uncertainty report: R3 |
| D08 | Complete readable viewer, inspection, replay, safe live update, no mutation | Product Rust renderer/feed/CLI suites and eighteen browser tests at `5b1d196`; [requirements](REQUIREMENTS.md); availability repair #11 | Historical component/browser evidence; current packaged browser regression: R3 |
| D09 | Cache, persistence, replay and reconnect preserve source-time knowledge and authority | [Cache review](evidence/runtime-cache-review.json), [harness assembly](evidence/harness-assembly.json) and replay repair `423d905` | Replay repair independent review is explicitly incomplete: R2; current reconnect matrix: R3 |
| D10 | Automated/native/provider/packaging/review results; evaluation runner and run status | [Packaging](evidence/native-packaging.json), [privacy review](evidence/release-privacy-review.md), [campaign review](evidence/campaign-guard-review.json), [assembly evaluation](evidence/harness-assembly.json), [failed live attempts](evidence/live-attempts.json) | Four synthetic modes, one four-node case each, zero real provider calls; no outcome benefit claim. R2/R3 and Map#14 remain |
| D11 | Exact companion commits/PRs/artifacts and honest integration/merge status | Current delivery table above; [assembly manifest](ASSEMBLY_MANIFEST.json); per-artifact [provenance](../contract-artifacts/harness/provenance.json) | Delivery metadata reconciled; historical test pins preserved. A newly qualified release tuple remains R3 |

## Section 17: gates A–G

| Gate | Criteria and evidence | Remaining owner |
| --- | --- | --- |
| A — preflight, discovery, preservation, bootstrap | D01/D02; repository exists and merged; historical orchestration cannot prove depth three | R1 and fresh host preflight in Map#14 |
| B — ownership, versions, fixtures, baseline, traceability | D04/D05/D11; copied artifact provenance and protocol correction #9; this complete ledger | R3 checks a coherent current tuple and mixed-version refusal |
| C — native projection/transport and renderer | D03/D04/D08; renderer and source probes are scoped evidence | Map#14 native comparison; R3 transport/legacy checks |
| D — harness analysis/context/cache/replay/provider and viewer | D05–D09; historical source and synthetic reports | R2 replay review; R3 regression; Map#14 provider |
| E — assembly, compatibility, offline demo, packaging, observability | D01/D04/D08/D10/D11; historical package checks and proposed observability contract | R3; if exporter enabled, authorized backend retrieval remains [Obs#8](https://github.com/AI-Ascension/ai-agent-observability/issues/8) |
| F — authorized native/provider validation and evaluation | D03/D06/D07/D10; four synthetic modes executed, live attempts failed before provider/action | Map#14; R3 preserves reproducible evaluation inputs and sample-size limits |
| G — independent reviews, repairs, reruns, handoff | D09/D10/D11; packaging/cache/campaign reviews exist; replay review interrupted | R2/R3; this reconciliation does not fabricate independent approval |

## Bounded remaining non-live tasks

These tasks remain owned by [Map#1](https://github.com/AI-Ascension/ascension-map-visualizer/issues/1);
they are not requirements waived by merging the documentation for Map#15.

- **R1 — orchestration evidence**, visualizer orchestration owner: revalidate actual nesting
  capabilities, carry useful implementation/review work through three real descendant levels
  with the specified Luna/max settings and twelve-descendant ceiling, and record requested,
  accepted and independently observed ancestry/settings. Do not rewrite the historical flat
  tree or satisfy the criterion with empty agents.
- **R2 — replay review**, harness owner with an independent reviewer: review the `423d905`
  repair and its current successor, exercise source-time snapshot/analysis/source-binding
  retention and non-dispatchability after pruning/restart, address findings in owner PRs,
  then retain exact reviewed pins and focused/full regression results. The interrupted
  historical review is not approval.
- **R3 — coherent current assembly and regression**, visualizer integration owner coordinating
  protocol/harness/gateway/MCP: choose exact compatible revisions and run the real-process
  synthetic map/legacy/refusal path, graph oracle, four-mode evaluation, renderer/CLI/browser,
  restart/reconnect and packaging checks. Record executable/fixture digests, evaluation
  sample sizes and uncertainty, independent review, and feature-off/mixed-version outcomes.
  Keep native/provider/backend evidence separate and preserve the original historical files.

- **Map#14 AC1 preparation (2026-09-17)**: the [live-lane pin matrix](evidence/live-lane-pin-matrix-20260917.md) records the current protocol/gateway/MCP/harness/game-mod/visualizer heads and the launcher-vs-harness map env-contract mismatch (`STS2_CAMPAIGN_MAP_BOUND`/`STS2_MAP_MODE`/`STS2_MAP_ARTIFACT_ROOT` exported, only `STS2_ENABLE_MAP_CONTEXT` read); it chooses no side and verifies nothing live.

The native/provider task remains Map#14. This ledger adds no host launch, provider spend,
installation, deployment or release authorization. The old “Steam unavailable” and “no child
delegation tool” statements describe their recorded attempts, not a current environment probe.
