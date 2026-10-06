# Rust Encounter Actor Context Effects

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-25

## Problem

The encounter mechanics projection currently adjusts a participant's own stat block when a condition or variant has deterministic self-contained effects. Some PF2e conditions and encounter states are not just properties of one actor's stat block. Visibility, targeting, cover, perception state, attacker/defender relationship, and similar effects depend on which actor is acting against which other actor.

Treating those effects as simple stat modifiers would either be wrong or would hide important GM context. For example, invisible, hidden, undetected, concealed, observed, grabbed, prone, controlled, confused, and related states are often most useful as tracked reminders with quick rules reference rather than automatic changes to one participant's AC, checks, or DCs.

## Desired Outcome

Design an encounter context model that can represent actor-vs-actor relationships and situational effects without forcing every condition into participant-local stat mutation.

The end state should let the encounter runner answer questions such as:

- what is actor A's visibility or targeting state relative to actor B;
- which effects apply when actor A attacks, targets, observes, or affects actor B;
- which effects are global participant annotations versus pairwise relationship state;
- which condition or rule reference explains the current runtime state;
- how the UI should expose quick condition/rules popovers for GM reminders even when no automation is available.

## Constraints

- Do not block broader canonical condition tracking on actor-vs-actor automation.
- Do not approximate pairwise effects as participant-local stat modifiers unless the PF2e rule is actually participant-local.
- Keep condition chips and rows able to open canonical condition/rule references so non-automated state is still useful at the table.
- Prefer explicit relationship state over hidden inference from prose or display names.
- Keep the first implementation opt-in and explainable; GM judgment should remain visible where automation is incomplete.
- Avoid building a full rules engine before the encounter data model can represent the relevant acting actor, target actor, action/activity, and observed relationship.

## Related

- [Rust encounter condition mechanics](./rust-encounter-condition-mechanics.md)
- [Rust record presentation mechanics unification](../history/items/rust-record-presentation-mechanics-unification.md)
- [Encounter mechanics activity model](../../../scratch/plans/2026-06-24-encounter-mechanics-activity-model.md)
