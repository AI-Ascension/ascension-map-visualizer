# ADR 0001: ownership and additive map capability

Status: accepted for this implementation assignment; remote merge, deployment and live capability remain independently unverified.

## Context

At the recorded baselines, gameplay observations expose only immediate travel options. The closed `runtime-v3-gameplay` schema and consumers reject unknown fields. Adding a graph under the same profile would break compatibility and obscure observation authority.

## Decision

Protocol owns the independent `runtime-map-v1` read envelope and `visible-map-v1` host snapshot contract, with exact schema bytes, SHA-256 identity, finite limits and conformance vectors. Existing profiles, schemas, digests and catalogs retain their behavior. The game mod owns main-thread permitted full-map extraction, stable map identity and exact current legal bindings; the gateway owns routing/authentication/fencing; MCP maps the bounded read capability; the harness owns validation, graph analysis, route intent, provider input, bundles, cache, replay and artifact lineage.

The visualizer consumes accepted neutral packages pinned by exact revision or copied inert contracts with provenance. It does not import sibling product implementation internals. Its own presentation DTO only controls layout and display. The browser receives the complete declared graph and never receives mutation authority. Graph adjacency, image content, a cache hit and a historical action ID cannot grant permission to act.

The harness's analysis and bundle schemas are independently versioned. A bundle binds source-time snapshot, matching analysis, deterministic renderer inputs/outputs, provenance and decision lineage. Operational timestamps and live-feed freshness remain outside deterministic render content. Renderer failure preserves the exact structured graph. Unsupported/incomplete/unavailable sources remain explicit; strict map acceptance cannot silently fall back to immediate options.

No game-core change is required for presentation or harness-owned graph policy. A future pure domain semantic change needs its own demonstrated consumer and owner decision.

## Integration and rollback

Producer and protocol changes precede gateway/MCP admission; the harness enables strict map context only when the complete capability is available. Exact implementation commits and copied artifact digests must be recorded before cross-repository acceptance. An unmerged producer is never described as deployed. Feature-off rollback selects the unchanged gameplay profile and disables map delivery; mixed versions return an explicit unsupported/unavailable error.

## Evidence

The baseline pins are in `../BASELINE_COMMITS.json`; the complete required gates are in `../acceptance.json`. Implementation issues are visualizer #1, protocol #17, game-mod #51, gateway #18, MCP #23 and harness #32. These issue references authorize implementation scope and do not prove completed contracts, native behavior or provider delivery.
