# Operations

## Artifact acquisition

Use the harness-owned map bundle publisher after the additive `runtime-map-v1` capability has been negotiated through the gateway/MCP path. The legacy gameplay catalog remains separate. Install order is protocol/host producer, fenced transport, harness adapter, then visualizer. Consult the [companion assembly manifest](ASSEMBLY_MANIFEST.json) before assembling binaries; source availability is not deployment proof.

The visualizer needs read access only to a bundle or artifact root. A complete bundle is acquired and validated before replacing the browser's immutable in-memory snapshot. Parsing, schema, identity, digest, image, or feed errors reject the replacement. Retained frames remain historical. HTTP workers do not perform filesystem reads or rasterization.

A live feed head older than 30 seconds is unavailable as current; acquisition failures stop freshness renewal and become disconnected after three seconds. A reconnect loads a complete valid feed before presenting current context again. Generation regressions within the same run/episode/trajectory/map epoch are rejected. Different epochs carry their own identity and do not restore authorization from a previous run.

Replay requires only the recorded root and zero game/provider connections. No later information is added to historical snapshots. Feed lineage must match each bundle's source state/generation and run, episode, and trajectory. A fresh process revalidates disk artifacts; an in-process validated immutable frame can be reused by exact digest.

## Limits and troubleshooting

`doctor` reports the contract digest and enforced limits. Initial limits are 256 nodes, 1024 edges, and 256 bindings under the protocol; the snapshot parser is bounded at 256 KiB. Replay accepts at most 4096 entries and 128 MiB of retained payloads. PNG/SVG files are bounded at 16 MiB each. There is one acquisition in flight, four HTTP workers, finite socket deadlines, and no arbitrary file route.

- Rejected contract: use matching producer/harness artifacts; do not edit the manifest or relax the schema.
- Rejected digest or image: regenerate from the original valid bundle with the recorded renderer version. This renderer deliberately rejects images that do not reproduce from the exact graph and settings.
- Existing output: choose a new directory. Publication never overwrites existing artifacts.
- Unavailable current feed: inspect producer availability/freshness and publish a fresh complete feed. Open replay for explicitly historical inspection.
- Oversized feed: apply bounded retention in the harness and retain archived roots separately. Repeated reads do not bypass the memory limit.
- Crowded image: increase dimensions within pixel limits and inspect the interactive overview. Sparse logical coordinates and extremely tall graphs can remain unreadable at fit-all resolution; exact graph data is retained.

A map read cannot open the map screen or mutate gameplay. Closed/modal/unsupported native scope must be explicit from the producer. No live supported-build claim is made until the native acceptance evidence records it.

## Optional telemetry

Set `ASCENSION_MAP_OTLP_PORT` to the existing loopback collector port (for example `14318`) for validation/render/demo telemetry. Leave it unset to disable export. The product's bounded OTLP/HTTP exporter sends only approved stage/count/boolean data and optional opaque trajectory linkage to a loopback collector. A trajectory is eligible for linkage only when the incoming value is a producer-issued `opaque-v1:` token followed by exactly 64 lowercase hexadecimal characters. The exporter generates a fresh 256-bit key from the operating system for each exporter instance and emits a domain-separated HMAC-SHA-256 digest; the raw token and key never leave the process. The same token links events during one exporter lifetime, while exporter instances and process restarts are intentionally unlinkable. Cross-process linkage requires a separately reviewed configured-secret design, which is not enabled here. Descriptive harness IDs such as `synthetic-trajectory-001` remain accepted for event accounting, but their linkage attribute is the literal `redacted` marker; they are never hashed. Values outside the bounded ASCII representation, including malformed reserved `opaque-v1:` values, are rejected before queueing. Rendering works when export is disabled or the collector is down. No map text, decision transcript, image bytes, credentials, or node IDs are metric labels. The observability companion documents the existing collector integration; deploying that stack is a separate operation.

## Rollback

Stop the viewer with Ctrl-C and retain recorded artifact roots as needed. Disable map capability in the harness to return to the unchanged legacy gameplay profile; label any next-move-only fallback explicitly. Never treat rollback, a cached bundle, or a successful read as a settlement witness. Reacquire the current host state/catalog before future authorized gameplay actions.
