# Rust Record Presentation Mechanics Unification

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-24

## Problem

Encounter adjusted mechanics now render structured activity and damage facts in the encounter inspector, while normal record detail rendering still uses the rich prose-oriented presentation document. This is acceptable for the first encounter mechanics slice, but it creates two presentation paths for facts that should eventually feel like one coherent record view.

## Desired Outcome

Unify or standardize record presentation so rich prose, structured mechanics, activity facts, and adjusted encounter mechanics share a deliberate presentation model.

The end state should let record detail views display structured spells, strikes, saves, damage, and prose in one readable hierarchy while still allowing encounter runtime projections to overlay adjusted values and modifier explanations.

## Constraints

- Do not make the browser parse raw Foundry text or duplicate backend mechanics rules.
- Keep authored rich prose available; structured mechanics should supplement or organize it, not discard it.
- Preserve source/provenance links so activity facts can still open or reference the content they came from.
- Avoid rewriting the central record reader during the first encounter mechanics implementation unless the mechanics DTO shape requires it.

## Related

- [Encounter mechanics activity model](../../../scratch/plans/2026-06-24-encounter-mechanics-activity-model.md)
- [Rust encounter condition mechanics](./rust-encounter-condition-mechanics.md)
- [Rust encounter elite and weak projection](./rust-encounter-elite-weak-projection.md)
