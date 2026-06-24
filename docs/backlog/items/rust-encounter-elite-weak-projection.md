# Rust Encounter Elite And Weak Projection

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-24

## Problem

Encounter v1 tracks creature state and can project basic elite/weak adjustments for typed creature stats. PF2e elite and weak adjustments also affect attacks, damage, DCs, spells, and offensive abilities, and projecting those incorrectly would be worse than leaving the base record untouched.

## Desired Outcome

Extend elite/weak projections beyond the initial typed-stat view.

The initial implementation covers structurally available creature fields such as HP, AC, saves, skills, perception, and ability modifiers, with tests for representative records. Unsupported or ambiguous text remains clearly unmodified through unapplied effect notes.

Elite/weak is stored as a first-class participant variant (`normal`, `elite`, or `weak`) rather than as freeform condition state. The adjusted view is produced by the shared modifier engine so future condition effects and participant variants explain their changes in the same shape.

## Constraints

- Store the selected adjustment in local state; derive the adjusted stat view from the active artifact at read time.
- Do not persist copied or mutated stat blocks.
- Do not silently rewrite freeform text unless the projection model can identify the value being changed.
- Base typed stat extraction belongs in `atlas-record`; encounter-specific variant and condition application belongs in `atlas-app-service`.
- In draft encounters, changing participant variant may update current HP by the projected HP delta; in running or completed encounters, changing participant variant must preserve current HP unless an explicit future mutation asks otherwise.
- Attack, damage, spellcasting, and formula-bearing effects must remain unapplied notes until the underlying typed targets exist.

## Related

- [Runnable encounters design](../../../scratch/plans/2026-06-22-runnable-encounters-design.md)
- [Rust encounter condition mechanics](./rust-encounter-condition-mechanics.md)
