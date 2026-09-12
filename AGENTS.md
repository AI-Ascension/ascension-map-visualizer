# Map visualizer implementation

Follow the complete assignment in `prompts/ASCENSION_MAP_VISUALIZER_ORCHESTRATION_PROMPT.md` and AI-Ascension organization policy. Rust owns rendering and CLI; a small browser frontend is permitted. No proprietary assets, credentials, private transcripts, or personal paths in committed artifacts. Protocol owns contracts; harness owns analysis. Preserve legacy profiles and unrelated changes.

All descendants must request `gpt-5.6-luna` and reasoning effort `max`. Three descendant levels maximum; depth-three workers may not delegate. Root grants every spawn reservation through the private registry, with at most twelve open descendants including waiting managers. No spawning without a root grant. Read-only specialists still require task scopes and independent verification. Do not restart workers because observation timed out. No independent client roots.

Use isolated branches and explicitly stage only owned files. No merge, release, deployment, or game mutation without scope authorization. Run Rust format, clippy, locked tests, and applicable security/browser checks. Separate synthetic, source, native, provider, and deployment evidence.

## Workspace, branch, and artifact hygiene

Before creating an isolated checkout, declare the exact absolute worktree path and the exact branch name. Create it only with `git worktree add <absolute-path> -b <branch-name>` (or attach the explicitly named existing branch). Do not create branch copies, sibling checkouts, backup trees, archive trees, or `*-tmp*` directories as substitutes for a Git worktree; do not use generated or random paths for branch isolation.

Perform edits and validation only in the declared checkout. Put build output, test fixtures, logs, and other derived artifacts in the repository's designated rebuildable output directory (such as `target/`) or a single declared task scratch path, never beside repositories or directly under `/home/agent`. Remove task scratch/output after it is no longer needed, and remove the worktree with `git worktree remove <the-same-absolute-path>` once its branch is integrated or abandoned. Preserve source, committed evidence, and any path explicitly retained by the task owner.
