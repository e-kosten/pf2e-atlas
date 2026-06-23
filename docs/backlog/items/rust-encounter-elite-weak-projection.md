# Rust Encounter Elite And Weak Projection

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-23

## Problem

Encounter v1 tracks creature state but does not mutate stat blocks. PF2e elite and weak adjustments are common enough that the runner should eventually present adjusted creature stats, but doing that incorrectly would be worse than leaving the base record untouched.

## Desired Outcome

Add creature-only effective stat projections for elite and weak encounter adjustments.

The first implementation should cover structurally available creature fields such as HP, AC, saves, skills, perception, attacks, and DCs, with tests for representative records. Unsupported or ambiguous text should remain clearly unmodified.

## Constraints

- Store the selected adjustment in local state; derive the adjusted stat view from the active artifact at read time.
- Do not persist copied or mutated stat blocks.
- Do not silently rewrite freeform text unless the projection model can identify the value being changed.
- Prefer a reusable projection owner such as `atlas-record` once reuse beyond the web runner is real.

## Related

- [Runnable encounters design](../../../scratch/plans/2026-06-22-runnable-encounters-design.md)
- [Rust encounter condition mechanics](./rust-encounter-condition-mechanics.md)
