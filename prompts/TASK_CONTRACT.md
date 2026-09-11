# Bounded task handoff template

Every assignment records these fields in the private root-owned ledger before spawning:

```text
task_id:
parent_runtime_id:
declared_depth:
reservation_id:
requested_model: gpt-5.6-luna
requested_reasoning_effort: max
accepted_settings_evidence:
observed_settings_and_parentage_evidence:
repository_and_isolated_worktree:
input_commits_and_schema_digests:
allowed_write_paths:
forbidden_scope:
dependencies:
deliverables:
focused_and_regression_test_commands:
acceptance_criteria:
time_memory_build_and_provider_budget:
required_evidence:
cancellation_behavior:
handoff_commit_and_artifacts:
status_and_blockers:
```

Root grants every reservation and counts waiting leads/coordinators in the twelve-descendant ceiling. Release a spawned slot only after verifying terminal runtime status; an observation timeout is not termination. Unspawned reservations can be cancelled with a recorded capability failure. Do not reuse a task ID for concurrent work or write a shared file without an ownership handoff.

Depth-three specialists may not delegate. Independent reviewers review another author's implementation and return concrete findings and reproductions. All descendants use the exact requested model/effort pair; self-report is not verification. If the runtime lacks nesting or required model support, keep the gate incomplete and continue independent authorized work without a bypass.

On cancellation or a context limit, finish only the safe bounded operation already in flight, persist changes and concise evidence, report live process handles, and stop. Never duplicate a still-running worker or restart a game action because a read timed out.
