# Encounter Participant Baseline Reset

Status: deferred
Priority: later
Owner: unassigned
Last reviewed: 2026-10-27

Allow a participant to return atomically to a defined creation baseline for
reusable encounters. The abandoned integration implemented this capability;
current per-resource restoration and HP/status editing do not provide a baseline
reset. Keep today's source-backed architecture and richer encounter rules.

Define which HP, conditions, spell/ability/turn resources, initiative, variant and
hazard-state values belong to the baseline, and what happens to pending events
and the current turn. Preserve display name, notes, side and visibility unless
the reviewed product policy explicitly says otherwise. Distinguish reset from
healing, reactivation and source refresh. Baselines need a versioned durable owner,
explicit availability and atomic/retry-safe execution; a partial multi-request
reset is insufficient.

Validate independent participants, saved-state persistence, missing baselines,
concurrent edits and rollback across every affected domain. Agree the interaction
and consequences before implementation.

Related: [encounter polish](rust-web-encounter-runner-polish.md),
[capability recovery](rust-integration-recovery.md).
