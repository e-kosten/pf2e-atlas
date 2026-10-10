# Rust Encounter Elite And Weak Projection

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-10-09

## Problem

Encounters derive a bounded adjusted view from authored Foundry DTOs. Broader
PF2e automation for spells, offensive abilities and situational effects needs
explicit typed targets and reliable rule evidence.

## Desired Outcome

Extend elite/weak projections beyond the initial typed-stat view.

The initial implementation covers structurally available creature fields such as HP, AC, saves, skills, perception, and ability modifiers, with tests for representative records. Unsupported or ambiguous text remains clearly unmodified through unapplied effect notes.

Elite/weak is stored as a first-class participant variant (`normal`, `elite`, or `weak`) rather than as freeform condition state. The adjusted view is produced by the shared modifier engine so future condition effects and participant variants explain their changes in the same shape.

## Constraints

- Store the selected adjustment in local state; derive the adjusted stat view from the active artifact at read time.
- Do not persist copied or mutated stat blocks.
- Do not silently rewrite freeform text unless the projection model can identify the value being changed.
- Base typed stat extraction belongs in `atlas-record`; encounter-specific variant and condition application belongs in `atlas-app-service`.
- Use the current HP policy in ADR 0046: pristine draft HP follows a known derived
  maximum; edited/active HP preserves known damage when a known maximum changes.
  Unknown, explicit override and out-of-range manual values remain explicit.
  Rebuilds never reinitialize durable HP. A known authored NPC adjustment applies
  once; an explicit participant choice replaces it.
- Preserve authored attack modifiers and damage formulas. Additional automation
  requires identified typed targets and rule evidence; unavailable or situational
  effects remain explicit notes.

## Related

- Runnable encounters design (historical untracked reference: `../../../scratch/plans/2026-06-22-runnable-encounters-design.md`)
- Encounter mechanics activity model (historical untracked reference: `../../../scratch/plans/2026-06-24-encounter-mechanics-activity-model.md`)
- [Rust encounter condition mechanics](./rust-encounter-condition-mechanics.md)
