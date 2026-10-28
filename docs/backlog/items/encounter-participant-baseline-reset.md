# Encounter Participant Baseline Reset

Status: in_progress
Priority: current
Owner: encounter-reset-rest
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

## Current implementation

Whole-encounter and participant resets restore versioned creation gameplay facts.
Whole reset returns to Draft/round 1. Individual reset ends the current turn when
resetting its owner. Names, notes, assigned level, sides, roster dimming and roster order are retained;
creation initiative is restored with a collision-free tie position. Pending
consequences and turn decisions are invalidated; historical receipts remain.
Existing participants explicitly lack baselines and must be re-added to capture
one. A whole reset cannot silently skip one of these participants.

Rest overnight previews eight hours of recovery and daily preparation for selected
living creatures. It reduces saved Drained/Doomed sources, removes Fatigued,
expires timed saved conditions, clears temporary HP, restores spell resources and
eligible daily/shorter ability frequencies, and removes Wounded at full HP. HP
recovery offers calculated overnight healing with editable amounts, or full HP.
All living non-hazard participants start selected.
Ability toggles and weekly/longer frequencies persist. Source-granted conditions
stay attached to their abilities. Reset is source-independent; rest uses a guarded
source revision. Interaction acceptance and final validation remain pending.
