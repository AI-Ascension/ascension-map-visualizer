# Actual Astra provider CLI capture

Status: **confirmed for this bounded local boundary test**.

This record covers one serialized `sts2.exo-decision-map-v1` request through the built `sts2-astra-bridge`, the actual pinned product renderer, and a private `PATH` shim named `codex`. The shim ran once, captured the prompt, image bytes, and argv, and wrote the bounded decision file requested by the bridge. No real provider process or network call was used.

## Source and artifact pins

- Product renderer: SHA-256 `782d0d24d795a35335b84dcb8b08458af5e97eeca71b853914cc8b590ad1462f`; build commit `5b1d196480685a313bc2417d5b4450a63cc89ce5`.
- Astra bridge executable: SHA-256 `d608c0290ce3f2b59b1c0be8a5e78c122f31cdede99626d5680e761ba66bbcd9`.
- Provider revision in request: `d608c0290ce3f2b59b1c0be8a5e78c122f31cdede99626d5680e761ba66bbcd9`.
- Request schema: `sts2.exo-decision-map-v1`; serialized size: 352905 bytes.
- Bridge preflight: `sts2-astra-bridge --describe`; renderer preflight: `map-visualizer validate --bundle <full-demo-bundle>`, followed by `render --width 1600 --height 2400`.
- Finite timeouts: bridge descriptor 5 seconds, renderer validation/render 20 seconds each, outer bridge capture 20 seconds; the bridge provider subprocess is bounded by its configured 90-second timeout.

## Graph and image evidence

- Full graph verified at request and prompt boundaries: 76 nodes, 182 directed edges, 3 legal bindings; every edge endpoint resolves to a captured node and no duplicate edge was accepted.
- Provider graph digest (normalized visible-map-v1): `925385c7c93c4a24786f6ca1d3f61f4150389e7db334ce1dc9ce2c852191f4ba`; renderer/fixture snapshot bytes: `5c55a98446b548f5ce50cc83a23623bde1ebe7a59c8998c978b702db5e9a2fad`. The two digests are recorded separately because the provider normalizes graph arrays while the renderer manifest hashes source snapshot bytes.
- Renderer output: 1600×2400, 253586 bytes, SHA-256 `e0491fc18092ddb798ab21dc2212d891526f68fb16e87a8db63cb4d9703691e3`; captured `--image` bytes: SHA-256 `e0491fc18092ddb798ab21dc2212d891526f68fb16e87a8db63cb4d9703691e3`.
- Renderer JSON PNG digest: `e0491fc18092ddb798ab21dc2212d891526f68fb16e87a8db63cb4d9703691e3`; captured image and rendered PNG are byte-identical.

## Execution identity and CLI evidence

- `model_execution_id`: `replay-capture-423d905`; `state_id`: `demo-state-42`; `generation`: 42.
- Captured prompt SHA-256: `0a0cf4eca5902d937cf36549e88b135dc618ed37ce19ae2aab0747f0bcb3c528`; the prompt contains the request identities and graph but no `bytes_base64` field.
- Captured argv SHA-256: `37c55e6e590e1a1387c8da59866f45c81dae6033c632725254e074e42e3e7810`; the verifier checked the bridge’s exact sandbox, disabled-tool, model, output-file, image, and stdin flags.
- Shim decision bytes: 81, SHA-256 `3fb201fa5991733381c5dba50ab4df82688517c44c0d9a64b40d54d327c43b5c`; bridge output SHA-256: `6c78f19b0afef102e55c42dddda29188d222622feb66076d8220f02c78d8ec06`; both selected the first supplied legal action and passed bridge validation.

## Reproduction

```text
./tools/map-provider-capture/capture.sh \
  --bridge <built-sts2-astra-bridge> \
  --renderer <map-visualizer-at-pinned-sha> \
  --fixture <map-bundle-demo-v1> \
  --report .orchestration/provider-cli-capture.md
```

The harness worktree was not used to launch a game, MCP server, gateway, account, or real provider. Those boundaries remain outside this evidence.
