# Ascension Map Visualizer — Complete Implementation Orchestration Prompt

## 1. Mission

You are the root implementation orchestrator for AI-Ascension. Create `AI-Ascension/ascension-map-visualizer` and deliver the complete, integrated map-visibility system specified below. Use actual Luna Max subagents through three descendant levels. This is an implementation assignment, not a request for another plan, a mockup, scaffolding, or disconnected pull requests.

The outcome is precise: at every supported map decision, the gameplay agent receives the entire player-visible topology of the current Slay the Spire 2 map, its current position, exact legal next-move bindings, and useful downstream route comparisons. A human can inspect the same graph in an interactive viewer. A capability-tested provider path can also deliver a deterministic full-map image to the agent.

“All paths at once” means preserving every permitted node and connection in one coherent graph and whole-map overview. It does not mean enumerating exponentially many complete routes, revealing hidden outcomes, or showing only a few recommended paths.

Make the new repository the visualizer product and orchestration home. Implement required companion changes in their existing owner repositories. Do not duplicate host, gateway, MCP, protocol, or harness implementations inside the visualizer simply to keep all edits in one repository.

Deliver production source, versioned contracts, integration adapters, tests, build and packaging automation, an offline demonstration, documentation, and reproducible evidence. Complete the entire dependency graph and repair integration failures. A working viewer alone is not completion.

## 2. Scope and operating authority

Create the new repository using authorized organization access. First check whether that exact repository already exists; resume legitimate existing work instead of overwriting it or creating a similarly named replacement. Follow organization visibility policy. Where no policy resolves visibility, create it privately and report that choice. Do not change another repository's visibility.

This assignment authorizes necessary source and test changes, repository initialization, isolated worktrees/branches, builds, synthetic tests, narrowly scoped commits and pushes, implementation issues, and pull requests. Assign issues and PRs to the authenticated implementing account where permitted. Preserve branch protection and required review. Do not assume permission to merge or publish a release; report local integration, remote publication, and merge status separately.

Read applicable `AGENTS.md`, architecture decisions, coding standards, licensing rules, and workflow instructions before editing each repository. Preserve unrelated dirty work. Do not reset, clean, force-push, broadly stage, rewrite history, weaken sandboxing, or disable approval policy.

Use existing approved disposable game-host and provider configurations for live validation. Do not purchase services, provision paid infrastructure, install system services, access unapproved hosts, mutate valued saves, or start unbounded paid campaigns. Bound authorized provider calls and game actions using existing experiment policy. No credential, proprietary game asset, raw private transcript, or personal machine path belongs in source control or public artifacts.

Choose ordinary implementation details without requesting unnecessary confirmation. When an external capability or authorization is genuinely missing, finish every independent implementable task, provide executable gated validation tooling, and identify the exact blocked acceptance criteria. Never convert a synthetic test into claimed live evidence.

## 3. Mandatory Luna Max orchestration

### Model and effort

Keep the root's current orchestration model unless the operator explicitly changes it. Every descendant—including leads, coordinators, implementers, researchers, and reviewers—must use this exact pair:

```text
model: gpt-5.6-luna
reasoning effort: max
```

Do not treat `luna-max` as a model identifier, substitute another model, or replace `max` with `xhigh`. This requirement governs the implementation agents; do not silently replace the project's configured gameplay provider.

Before significant delegation, inspect the installed client version, effective configuration, model availability, custom-agent overrides, spawn schema, nesting behavior, and thread limits. Verify requested settings, accepted settings, and observed execution metadata separately. A child declaring its model is not proof.

For a compatible Codex installation, validate and apply these project-local settings without overwriting unrelated configuration:

```toml
[agents]
enabled = true
max_concurrent_threads_per_session = 12
default_subagent_model = "gpt-5.6-luna"
default_subagent_reasoning_effort = "max"
```

Set `model` and `model_reasoning_effort` explicitly in supported custom-agent definitions too. Include their required name, description, and developer instructions. Confirm effective configuration for each spawned session; editing a file does not prove an existing session reloaded it.

Use only supported configuration keys and tool arguments. Do not add an obsolete or unsupported `max_depth` key. Do not launch hidden clients, alternative providers, or independent roots to bypass runtime restrictions. If genuine nesting or Luna Max is unavailable, record the capability failure, continue independent authorized work, and leave orchestration verification incomplete.

### Three-level depth contract

```text
Depth 0 — Root orchestrator: integration authority and final acceptance
  Depth 1 — Workstream leads: Luna / max
    Depth 2 — Bounded work-package coordinators: Luna / max
      Depth 3 — Implementation, test, and review specialists: Luna / max
```

Three levels means three descendant layers below the root, not three agents total and not three total layers including the root. Depth-3 agents must not spawn children. Exercise genuine depth-3 implementation and independent-review work where appropriate; do not manufacture empty management agents to satisfy the diagram.

Use native ancestry/depth restrictions when supported, plus a root-owned spawn registry. Restrict leaf delegation tools where possible. Record actual runtime parent/child identities. A flat group of children with hierarchical names does not satisfy this requirement. Never relabel a descendant as a new root to reset depth.

### Concurrency and scheduling

Use a global project ceiling of 12 open descendants across the entire tree, reduced to actual runtime/account/resource limits. Twelve is the project budget, not a claim about a platform maximum. Count waiting leads and coordinators as open threads. A per-session setting alone does not prove a global nested-tree limit.

The root owns atomic slot reservations and lifecycle records. Every spawn needs a granted reservation, parent identity, declared depth, task ID, and scope. Release capacity only after confirming termination. No uncoordinated recursive fan-out, duplicate active tasks, busy polling, or respawning a timed-out task whose earlier worker might still run.

Schedule workstreams in waves and reserve room for leaf work. For example, two leads, two coordinators, and four leaves use eight descendant slots while leaving four for review or recovery. Do not fill the budget with managers waiting for unavailable children.

### Workstream organization

| Depth-1 lead | Depth-2 packages | Example depth-3 specialists |
|---|---|---|
| Architecture and contracts | Ownership/versioning; map schema and conformance | Schema implementer; compatibility tester; independent contract reviewer |
| Host and transport | Native projection; gateway/MCP mapping | Host extractor; visibility tester; fenced-route implementer |
| Harness and agent context | Graph analysis; provider delivery; cache/replay | Graph implementer; provider serializer tester; replay reviewer |
| Visualizer product | Deterministic rendering; browser interaction; CLI | SVG/PNG engineer; frontend engineer; accessibility/visual tester |
| Integration and packaging | Cross-repository assembly; CI; observability | Integration-test engineer; packaging engineer; telemetry adapter engineer |
| Independent verification | Security; topology fidelity; evidence audit | Adversarial tester; independent graph-oracle engineer; release-readiness reviewer |

Leads coordinate coherent packages; coordinators assign narrow leaf tasks. Independent verification must not merely approve its own implementation. The root resolves ownership disputes and merges compatible changes into the integration worktree.

### Task contract and records

Give every assignment a task ID, parent/depth, model/effort, input commit/schema pins, repository/worktree, allowed write paths, forbidden scope, dependencies, deliverables, test commands, acceptance criteria, resource budget, and required evidence. Include exact handoff expectations and cancellation behavior.

Use isolated worktrees and a single active writer per conflicting file or schema. Contract changes require coordination before dependent implementations diverge. Children return concise findings, commits, exact test results, blockers, and evidence locations—not full copied repository contents or private reasoning transcripts.

Maintain a dependency-aware task ledger and private runtime registry. Suggested local paths are `.orchestration/tasks.json`, `.orchestration/agents.jsonl`, and `.orchestration/ownership.json`; ignore sensitive runtime records in Git. Commit sanitized orchestration evidence and `docs/IMPLEMENTATION_STATUS.md`. Validate the ledger against actual runtime ancestry and worktree state when resuming.

Persist completed work before context limits. Resume existing tasks rather than restart discovery or duplicate agents. Rendering, graph analysis, polling, and normal product operation must not require LLM calls.

## 4. Discovery and baseline verification

Inspect these repositories at their current accessible heads, recording exact commits, relevant policies, existing issues/PRs, and the actual runtime path:

```text
AI-Ascension/.github
AI-Ascension/sts2-game-mod
AI-Ascension/sts2-game-core
AI-Ascension/sts2-protocol
AI-Ascension/sts2-gateway
AI-Ascension/sts2-mcp-server
AI-Ascension/sts2-harness
AI-Ascension/ai-agent-observability
AI-Ascension/ascension-map-visualizer
```

Use the following as discovery leads from a prior source review, not immutable current facts:

- `sts2-game-mod/experiments/managed-rust-interop/game-loader/LiveCombatSource.Campaign.cs`: inspect `ProjectCampaign`, `TravelablePoints`, `MapId`, `CurrentNodeId`, and `CampaignActions`.
- Inspect `LiveCombatSource.MapNavigation.cs`, `RuntimeV3GameplayObservation.cs`, fair-play serialization, native dispatch, and postcondition verification.
- Locate the canonical gameplay schemas and copied release-like artifacts; do not assume a consumer copy is the owner.
- Inspect `sts2-harness/crates/harness/src/exo/sandbox.rs`, its schema modules, `exo/protocol/request.rs`, `exo/protocol.rs`, and `exo/decision.rs`.
- Trace episode context assembly, the reviewed provider/Exo bridge, artifact storage, decision records, replay, MCP catalogs, and fixed gateway routes.
- Inspect observability deployment and privacy policies before adding integration.

Determine whether the current map representation still exposes only immediately travelable options; whether current node IDs differ from action-option IDs; where strict observation allowlists reject new fields; and every request/image size limit across the complete path.

Do not invent native map-edge APIs, provider attachment capabilities, transport endpoints, or supported runtime profiles. Treat names such as `sts2-map-v1` and `sts2-play-v1` as proposals unless current repository decisions establish them. Inspect the real accepted profile before choosing the additive integration strategy.

Produce a compact source map, requirement-to-test matrix, dependency graph, and ownership/versioning ADR. Continue directly into implementation; these documents are inputs, not the final deliverable.

## 5. Ownership and architecture

Preserve this authoritative observation/control path:

```text
Game host -> game-mod projection -> gateway -> MCP -> harness -> agent
                                                   |
                                                   +-> map artifacts
                                                        -> visualizer
                                                        -> observability
```

The visualizer may consume validated bundles and follow a harness-published artifact feed. It must not read game memory, load game assemblies, call the mod directly, bypass MCP/gateway, or obtain mutation credentials.

| Owner | Required responsibility |
|---|---|
| game-mod | Main-thread extraction of permitted map information and authoritative current legal-action bindings |
| protocol | Accepted neutral map contract, version/digest policy, and conformance vectors |
| game-core | Only explicitly needed pure domain semantics/validation through an accepted seam |
| gateway | Fixed authenticated map-read routing, instance/session/lease fencing, limits, cancellation |
| MCP | Bounded tool/resource schemas and reviewed mapping to the gateway |
| harness | Snapshot validation, graph analysis, decision context, route intent, caching, lineage, replay |
| ascension-map-visualizer | Deterministic presentation, SVG/PNG output, read-only viewer, offline CLI |
| observability | Approved artifact/trace integration and deployment documentation |

Keep runtime communication separate from compile-time dependencies. Consume approved versioned packages or inert contract artifacts, not sibling implementation internals. Do not add meaningless changes to game-core merely to touch every repository. Any justified owner change requires an ADR, actual consumers, and tests.

Use an independently versioned additive map capability or profile. Preserve legacy catalogs, envelopes, digests, and behavior. A strict consumer that rejects additional fields requires explicit version/capability handling; do not silently modify an existing profile under an unchanged identity.

## 6. Map data contracts

Implement three separately owned representations: host-projected `VisibleMapSnapshot`, harness-derived `MapAnalysis`, and harness-owned `MapViewBundle`. These are proposed type names; use consistent accepted names after discovery.

### VisibleMapSnapshot

Include a stable map-instance identity, act/scope identity, source state ID and generation, projection version, supported game/mod provenance, and explicit freshness. Preserve instance, session, lease, run, episode, trajectory, request, operation, and artifact namespaces at their proper boundaries; do not force the host to invent harness identities.

Represent nodes with stable IDs, logical coordinates, visible room category or explicit unknown, and permitted observed status. Represent connections as explicit directed edges with validated endpoints. Include current position or an explicit pre-start state, permitted visible history, and generation-bound mappings to exact host-produced legal actions.

Separate availability, completeness, and freshness. Distinguish unavailable/not-observable/unsupported from an available but incomplete graph and from a complete empty scope. Include bounded reason codes. Distinguish a historical cached map from a current host observation.

Use map-instance plus act and logical coordinates where suitable, with an opaque disambiguator when required. Stable node IDs must survive room-label changes and must not collide across runs, resets, repeated acts, or same-seed episodes. Do not use generation-scoped action IDs as graph IDs. Treat identifiers as opaque outside their defined owner.

Define complete schema constraints: required fields, null semantics, unknown values, duplicate handling, numeric ranges, collection limits, string bounds, normalization, sorting, canonical bytes, digest algorithm, extension policy, and independent schema versions. Bound parser input before allocation; reject duplicate object keys and invalid Unicode/identity forms where applicable.

Set initial deployment limits from observed supported maps plus a documented margin. Keep byte/element/pixel budgets finite and configurable within validated maxima. An oversized map must fail explicitly or use a reviewed consistent-snapshot transfer—not silently lose nodes or edges. Pagination, where needed for transport, must finish reassembly before claiming whole-map agent delivery.

### MapAnalysis

Keep derived values separate from host facts. Include input digests, analysis/evaluator versions, declared assumptions, structural reachability, route counts, route features, bounded concrete candidates, completeness, and any approximation flags.

Do not place estimated survival, rewards, or future legality into authoritative observation fields. A room category can be observed while its future encounter outcome remains unknown.

### MapViewBundle

Bind one snapshot, its matching analysis, presentation settings, and decision/history references. A manifest must identify exact input/output digests, independent versions, origin, generator, license/provenance, and relevant run/episode/trajectory/model-execution identities.

Support an artifact layout equivalent to:

```text
manifest.json
visible-map.json
analysis.json
overview.svg
overview.png
decision.json
```

Separate deterministic content from timestamps and operational metadata so reproducible rendering does not depend on wall-clock time. Publish bundles atomically; consumers must never observe half-written or cross-generation combinations.

## 7. Fair-play projection and native integration

Implement a cohesive host-owned map projection module, such as `LiveCombatSource.MapProjection.cs`, following the actual repository layout. Keep immediate travel-choice extraction separate from full-map extraction.

Validate the native representation against the supported build. Determine how visible nodes and connections are represented, whether scrolling virtualizes the map, what ordinary players can inspect off-screen, how special connections appear, and what is available when the map is closed. Coordinates alone are not evidence of connectivity.

Capture a consistent snapshot on the required host thread, copy permitted values into owned DTOs, and perform expensive serialization/rendering away from the host callback. Bound traversal, work, cancellation, and retained data. Do not retain host objects across threads.

Project all legitimately inspectable topology in the declared current-map scope, including visible branches that are no longer reachable. Do not use future acts, hidden room outcomes, unrevealed encounters/rewards, RNG state, seed reconstruction, internal debug knowledge, or private metadata to enhance the graph.

A read must not open the map, scroll the game, select a node, or mutate gameplay. Where ordinary inspection requires an explicit interaction, represent it through an admitted action with separate evidence; do not hide it inside a read. Return explicit unavailability where necessary while allowing clearly labeled historical context.

Hashes, node IDs, route summaries, and images must depend only on permitted information. Test paired host states that differ only in hidden information but have identical public observations: their projected content and deterministic downstream artifacts must match. Account explicitly for independent correlation/time metadata rather than letting it mask leaks.

Bind legal next moves to the exact host catalog at the same source boundary. Handle transitions, modal blocking, changing topology, and snapshot/catalog races without synthesizing authority. Unsupported special movement stays explicit; never infer permission from graph adjacency.

## 8. Gateway and MCP integration

Add the accepted fixed map-read route/tool or profile using existing authentication, instance/session identity, leases, epochs, deadlines, correlation, limits, and error conventions. Tool naming must follow the current project registry; do not implement an arbitrary downstream URL proxy.

Return consistent snapshot identity and action-catalog bindings. When map and general state are obtained separately, verify their source identity/generation and reject or retry bounded read-only acquisition on mismatch. Never combine unrelated responses because their map coordinates happen to agree.

Keep read scope sufficient for reading and insufficient for mutation. Preserve legacy tool lists/profile behavior. Test unknown capabilities, digest mismatch, malformed data, stale leases, wrong instances, timeout, disconnect, cancellation, and body limits at each boundary.

Structured graph results are mandatory. Any MCP image/resource representation must be explicitly supported and correctly mapped. Do not claim the agent sees an image merely because MCP can return a URI or the browser can open it.

## 9. Deterministic graph analysis and route intent

Implement graph analysis in the harness-owned boundary. Validate endpoints, identities, graph scope, completeness, and supported topology before analysis. Use a DAG algorithm only after acyclicity is established; return a typed unsupported/invalid result for cycles rather than recurse indefinitely or invent route counts.

Compute downstream reachability from the current position and each legal next move, ancestor/descendant sets, branch/merge structure, distances to visible categories, and structural route features. Treat host-legal special movement separately from ordinary adjacency.

For a validated acyclic graph, count structural paths to explicitly identified terminal destinations with reverse-topological dynamic programming. Use checked arbitrary-precision counts or an explicit bounded-count status; serialize large counts safely, such as decimal strings. Do not mistake every leaf for a boss, count incomplete boundaries as confirmed endpoints, or describe path counts as win probabilities.

Represent the full graph even when path counts are enormous. Bound only candidate enumeration and analysis resources. Compare an initial configurable maximum of eight concrete routes, with deterministic selection/tie-breaking and an explicit statement that candidates do not enumerate every route.

Useful comparisons include distance to visible rest sites, access to visible shops, visible elite counts, elite exposure before rest opportunities, and retained branching options. Return actual node sequences and their first current legal move. Never combine independent extrema into a nonexistent route. Do not claim global Pareto optimality after bounded heuristic pruning.

Keep scoring configurable, versioned, and explainable in terms of public inputs. Do not invent expected rewards or survival probabilities. Treat unavailable build features as unknown rather than silently substituting defaults. Record whether candidate selection is exact, heuristic, or incomplete.

Store route intent as node preferences/waypoints with assumptions and input versions, not a queue of future action IDs. At each decision, acquire fresh state/catalog, validate the next move, choose one action, dispatch through existing boundaries, verify settlement, and reobserve. Preserve stable operation identity and reconcile unknown outcomes; no blind mutation retry and no map-read response treated as an effect witness.

## 10. Agent delivery and image support

Implement versioned, validated agent context carrying the complete bounded graph, exact current action bindings, analysis, and freshness/provenance. Reuse unchanged topology where the provider/session actually retains it; otherwise include enough data to reconstruct the full graph in the decision request.

Trace the real context path through sanitization, request assembly, the reviewed Exo/provider bridge, provider serialization, and model input. Extend allowlists and typed schemas explicitly. Do not hide new information in a text field to evade validation or stuff base64 images into legacy observation JSON.

Implement capability-negotiated PNG attachments for the configured supported image-capable provider path. Validate media type, dimensions, byte count, artifact identity, and graph/image digest correspondence. Resolve resources into actual supported image inputs. A local path, resource link, or image description alone does not prove delivery.

Inspect the exact bridge revision and its authorized source owner. Make compatible changes at the proper boundary; do not patch opaque binaries or assume an external bridge forwards unknown fields. Where external access is missing, complete authorized adapters/tests and report the precise remaining live-delivery gate.

Test the actual serialized provider request at a capture boundary, not merely an internal DTO. Verify all permitted nodes and edges are represented, stale bindings cannot dispatch, and attached image bytes correspond to the same bundle. Keep synthetic capture evidence separate from a real provider call, and actual provider delivery separate from evidence of model comprehension.

Retain the existing bounded decision/rationale policy. Record structured decisions and concise rationales, not hidden chain-of-thought or raw private provider transcripts. Export only approved sanitized evidence.

## 11. Visualizer implementation

Use a deterministic Rust rendering core producing SVG and PNG through a pinned compatible `resvg` dependency. Keep rendering pure with explicit inputs; isolate filesystem/server behavior. Use a small browser frontend in the new repository, with locked dependencies only where needed. Do not add a graph database, a new observability stack, or an LLM rendering service.

Use stable logical floor/lane layout rather than force-directed positions. Handle branching, merges, crossed edges, and unknown nodes without confusing crossings with connections. Assign readable display aliases with a reversible mapping to canonical identities. Ensure distinct nodes/edges remain distinguishable.

The default viewport must include the entire declared graph. Preserve visible unreachable and passed branches with reduced emphasis, not deletion. Clearly distinguish current position, immediate legal moves, downstream reachability, visited history, selected candidate routes, unknown information, and stale/incomplete state. Use labels/shapes as well as color.

Provide fit-all, zoom/pan, keyboard navigation, node/edge inspection, legend, search by canonical/display ID, candidate comparison, overlay toggles, and replay navigation. Filtering or highlighting must be visibly identified and must not silently redefine the base graph. Maintain a whole-map overview when inspecting details.

Default node clicks inspect or highlight only. Do not add game-action buttons or mutation endpoints. Route selection in the UI is an annotation unless a separate explicitly authorized future feature changes that boundary.

Generate high-resolution overview images with configurable dimensions and pixel/memory limits. Include full topology in one image; provide supplemental detail views when necessary without replacing the overview or exact graph. Test representative dense/tall maps and document readability limits rather than crop silently.

Pin layout, rasterization settings, and reproducible font/glyph handling with appropriate provenance. Use original project-owned symbols rather than proprietary game assets. Escape untrusted labels, reject external resources/scripts, and never execute SVG or HTML supplied by a bundle. JSON data must render as data, not instructions.

Support offline bundle opening and a loopback-only live view of harness-published artifacts. Live updates must preserve map/epoch identity, publish atomically, visibly mark disconnection/staleness, and recover by loading a complete fresh snapshot. The visualizer has no direct game credentials and does not need a network connection to render a bundle.

## 12. Cache, persistence, and replay

Separate cache identity for projected topology, navigation/legal-action overlays, analysis inputs/objectives, and rendered artifacts. Include independent version/digest settings and appropriate run/session isolation. Do not use seed alone as a cache key or include private host-state hashes.

Invalidate overlays on position/catalog/generation changes and topology on visible graph/scope/projection changes. Invalidate analysis when its public player-state inputs or policy change. Recompute cheaply and deterministically; do not ask an LLM to rediscover graph properties.

Keep artifact deduplication separate from authorization and from provider prompt caching. A matching content hash never restores a lease or legal action. Measure provider token usage independently instead of promising savings from local caching.

Bound cache size, retention, concurrency, and in-flight work. Use atomic writes and consistent manifests; reject corruption and partial bundles. Handle concurrent renderer requests without duplicated unbounded work.

Replay must reconstruct exactly the graph, allowed knowledge, analysis version, and action bindings available at each historical decision. Do not incorporate information discovered later. Pure replay/rendering requires zero provider calls and zero game connections. Show historical authorization as historical, never dispatchable.

Renderer failure must leave a valid structured-graph path usable. Graph unavailability must never produce a fabricated graph. Provide explicit strict-map and labeled next-move-only degradation policies, with strict-map used for feature acceptance. Reacquire authoritative state after restart/reconnect before any action.

## 13. Product packaging and repository deliverables

Create a cohesive repository rather than empty placeholder crates. A suitable layout is:

```text
ascension-map-visualizer/
  AGENTS.md
  README.md
  LICENSE
  THIRD_PARTY_NOTICES.md
  Cargo.toml
  Cargo.lock
  rust-toolchain.toml
  crates/map-visualizer/       # rendering, validation adapters, CLI/server modules
  web/                        # read-only browser viewer
  contract-artifacts/         # approved copied contracts with provenance
  fixtures/                   # synthetic, licensed, deterministic
  tests/                      # integration, security, replay and visual tests
  docs/                       # architecture, contracts, operations, evidence
  prompts/                    # this prompt and bounded child-role templates
  .codex/                     # supported project configuration and agent roles
  .github/workflows/          # checks and build packaging
```

Adjust module/crate boundaries only where cohesion or repository policy justifies it. Keep protocol authority upstream and analysis policy in the harness; viewer-side validation and presentation must not create competing semantic owners.

Implement a documented CLI with functionality equivalent to `validate`, `render`, `serve`, `replay`, and `doctor`. `doctor` reports versions, artifact compatibility, limits, and capabilities without exposing secrets or mutating the host. Document exact implemented flags and exit codes; do not publish commands that do not exist.

Provide a one-command synthetic demonstration that produces a full-map SVG/PNG and opens or serves the read-only viewer. It must work without game files, provider credentials, or the observability deployment. Package Windows x86-64 and Linux x86-64 build artifacts through supported CI; distinguish cross-compilation from native execution evidence.

Document setup, integration order, profile negotiation, privacy, snapshot scope, unavailable-state behavior, troubleshooting, rollback, replay, supported builds, and known limits. Store this complete orchestration prompt in `prompts/ASCENSION_MAP_VISUALIZER_ORCHESTRATION_PROMPT.md`.

## 14. Observability and operational security

Integrate with the existing observability system through its accepted interfaces. Track capture/validation/analysis/render latency, node/edge counts, byte/image sizes, cache hits, stale/incomplete observations, provider-delivery capability, invalid-action rejection, and decision/trajectory links.

Avoid high-cardinality node/run identifiers in metric labels; keep necessary lineage in approved traces/artifacts. Observability must be optional: an outage cannot block otherwise valid local rendering or grant gameplay authority. Use bounded asynchronous export and explicit failures.

The local server must restrict served files to the configured artifact root, reject traversal/symlink escape and arbitrary fetch URLs, validate Host/origin behavior, restrict CORS, and disable executable uploads. Add finite request/image/connection limits and appropriate content security policy. Do not expose a public listener or forward game/provider credentials to the browser.

Review dependencies, licenses, workflow permissions, untrusted-PR behavior, and generated artifacts. Pin actions/dependencies according to organization policy. Do not weaken checks or use privileged workflows merely to make CI green.

## 15. Required automated validation

Build an independent synthetic graph oracle and a requirement-to-test matrix. Implement these test families:

1. Topology: branches, merges, crossed edges, disconnected nodes, multiple terminals, pre-start states, duplicate coordinates across map instances, same-seed resets, room-label changes, malformed endpoints, duplicate identities, and unsupported cycles.
2. Projection: off-screen inspection, map-open/closed behavior, unknown room types, history, modal blocking, special movement, changing topology, host-thread constraints, bounded traversal, and zero side effects from reads.
3. Fair play: paired observationally equivalent hidden states produce identical permitted graph content, hashes, summaries, and deterministic images. Hidden sentinels never reach context, logs, manifests, filenames, or public artifacts.
4. Transport: every identity fence, profile/digest mismatch, unsupported fields, byte limits, timeout/disconnect/cancellation, snapshot/catalog races, and unchanged legacy behavior.
5. Analysis: compare small graphs with exhaustive independent enumeration; stress exponentially many represented routes without enumeration; test count overflow, incomplete terminals, deterministic ties, candidate validity, and explicit heuristic labels.
6. Rendering: exact node/edge representation, layout stability, readability checks, unknown/stale markers, SVG escaping, pixel/element limits, reproducibility in pinned environments, and actual visual inspection of representative outputs.
7. Browser: offline load, fit-all, zoom/pan, keyboard inspection, route overlays, replay, live reconnect, safe rendering of hostile labels, no console errors, and no mutation/network bypass.
8. Agent delivery: inspect actual serialized graph/image requests, retain full topology within supported budgets, reject stale image/context combinations, test strict/degraded capability modes, and preserve host-bound action validation.
9. Cache/replay: atomic publication, corrupted manifests, invalidation, bounded eviction, concurrent requests, restart isolation, time-correct knowledge, and zero provider calls during replay.
10. Packaging/security: clean checkout builds, native platform smoke tests where available, CLI exit codes, path/origin defenses, secret/license scans, least-privilege workflows, and offline-demo reproduction.

Use property-based tests and bounded fuzzing for parsers, graph invariants, and artifact loaders. Do not derive every expected result from the same implementation under test. Visual snapshots alone cannot establish semantic correctness.

Run each changed repository's policy and validation entrypoints. For Rust changes, include:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```

Run `cargo run --locked --package repo-policy -- --strict` where that repository supplies it, plus schema/digest/conformance checks, frontend type/lint/browser tests, and applicable managed-code builds/tests. Preserve existing tests. Record exact commands, exit codes, environments, commits, and artifact hashes. Missing dependencies or skipped tests are not passes.

## 16. Integrated host/provider validation and evaluation

Assemble an exact cross-repository commit/artifact manifest. Test the actual executables in the path, not only mocks sharing helper code. Complete one vertical slice before declaring any component release-ready:

```text
native permitted map
 -> complete versioned snapshot
 -> fenced gateway/MCP read
 -> harness validation and route analysis
 -> actual provider graph/image input
 -> one current legal decision
 -> existing dispatch and host settlement
 -> fresh map/context update
 -> human viewer and recorded replay
```

On authorized disposable hosts, validate complete topology including off-screen branches, current position, legal next-move correspondence, and an action that settles through existing witness rules. Capture enough sanitized evidence to compare source topology with serialized and rendered outputs. A successful HTTP call, file write, or model response alone is insufficient.

Implement a reproducible evaluation runner comparing: existing next-move-only context; full graph; graph plus analysis; and graph plus analysis plus image. Use controlled public inputs, identical experiment policies, bounded runs, and separate artifacts per configuration. Track topology/comprehension errors, missed-route opportunities, invalid proposals, latency, bytes/tokens, and downstream outcomes where actually observed.

Synthetic decision tasks must run without paid calls. Live provider/game evaluations require existing authorization and budgets; report sample sizes and uncertainty. Do not promise improved win rate from a small sample or claim a model used an image merely because an API accepted it.

## 17. Execution order and review loop

Execute these gates in dependency order, parallelizing independent work within the approved agent budget:

A. Runtime/model/nesting preflight; repository discovery; existing-work preservation; repository bootstrap.
B. Ownership ADR, contract/version acceptance, initial fixtures, baseline tests, and traceability matrix.
C. Host projection and transport implementation, alongside renderer development against frozen fixtures.
D. Harness analysis, context, cache, replay, and provider serialization; browser/CLI completion.
E. Cross-repository assembly, compatibility tests, offline demo, packaging, and observability integration.
F. Authorized native/provider validation and reproducible evaluation; evidence collection.
G. Independent security, correctness, visual, and evidence review; repairs; complete relevant reruns; handoff.

For every package, require implementation, focused tests, independent review, integration, and regression checks. A lead's status message is not proof. Root acceptance must inspect the diff and evidence.

Keep an explicit merge/dependency order for companion PRs and use coherent commits. Do not reference an unmerged upstream artifact as publicly available. Record feature-off rollback and mixed-version failure behavior. Never leave consumers accepting a contract their deployed producer cannot supply without an explicit unavailable response.

When tests fail, find the cause, fix it at the correct owner, and rerun the affected gates. Do not suppress failures, update golden images blindly, skip requirements, or weaken security/compatibility rules to manufacture completion.

## 18. Definition of done and final response

Maintain machine-readable acceptance status and its supporting evidence. Mark the whole assignment complete only when all required, authorized gates are satisfied:

- The exact new repository exists with working product source, policies, documentation, and reproducible builds.
- Actual Luna Max descendant settings and genuine three-level ancestry are evidenced, with depth/concurrency limits respected.
- The native supported map projection includes every permitted node/edge in scope, with explicit completeness and no hidden-information leakage.
- Versioned map data traverses the real fenced gateway/MCP/harness path without breaking legacy profiles.
- Stable graph identities remain separate from current host-generated action identities.
- The gameplay provider receives the full graph and a verified matching image on the supported image path; absent capabilities remain explicit.
- Route analysis is deterministic, bounded, and truthful about estimates; future intent cannot dispatch stale actions.
- The viewer presents the complete graph, readable distinctions, inspection, replay, and safe live updates without mutation authority.
- Cache, persistence, replay, and reconnect behavior preserve source-time knowledge and authorization boundaries.
- Automated suites, native/provider gates, packaging, and independent review have concrete results; the evaluation runner and its actual run status are documented.
- Every companion change has an exact commit/PR/artifact reference and an honest integration/merge status.

Do not substitute screenshots, successful compilation, fake-provider output, or checklists for the missing gates. When external access blocks a gate, finish all unblocked work and report `implemented; live gate blocked/unverified`, not `complete`. Do not end early merely because the assignment spans multiple repositories.

The final response must state what was built; repository and PR references; exact validated commit set; requested/accepted/observed model settings and agent-tree summary; build/test/live-evidence results; demo/render/serve/replay commands; locations of example artifacts; known limits and specific blockers; and whether anything was committed, pushed, merged, released, installed, or launched.

Start with runtime preflight and source verification. Then create or resume `AI-Ascension/ascension-map-visualizer`, execute the Luna Max hierarchy, integrate the complete system, and validate it. Do not return only another implementation proposal.