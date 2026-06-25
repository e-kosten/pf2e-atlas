# Rust Encounter Condition Mechanics

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-24

## Problem

Encounter v1 stores condition annotations with value, duration, source, and notes. The initial mechanics projection applies a deterministic subset of conditions to typed creature stats, but broader PF2e condition handling still needs careful scope because many effects are situational or target mechanics the model does not yet type.

## Desired Outcome

Extend condition mechanics beyond the initial deterministic projection.

The initial implementation supports these conditions through the shared typed target and modifier-resolution machinery:

- frightened;
- sickened;
- off-guard;
- clumsy;
- enfeebled;
- stupefied.

Remaining design and implementation work should answer:

- how numeric condition values are interpreted;
- which actor stats are adjusted and which remain annotation-only;
- how unsupported or situational effects are displayed without implying automation;
- whether duration decrementing stays manual or becomes opt-in automation;
- how broad the modeled-condition catalog should become before unsupported conditions get first-class annotation-only definitions.

Condition automation uses a backend-owned modeled-condition catalog for the first supported set. New UI-created modeled condition rows carry canonical `conditionitems` refs, and mechanical automation should require a canonical modeled `condition_key`. Rows without a modeled key remain durable annotations. A future freeform/custom-effect path can build on the existing local-state fields, but fuzzy condition names should remain annotation-only.

## Constraints

- Keep unsupported conditions as durable annotations.
- Do not auto-apply effects that require GM judgment or encounter context the model does not represent.
- Make each mechanically applied condition test-backed against representative records.
- Preserve the v1 freeform condition path for non-standard effects.
- Keep the browser picker driven by app-model/app-service condition definitions rather than a frontend-owned condition list.
- Expose unapplied effects for modeled conditions when a rule targets attack, damage, spellcasting, formula-bearing values, or other targets that the first typed stat projection does not yet model.
- Track follow-up work for attack, damage, spellcasting, and formula-backed transitive effects instead of silently ignoring them or approximating them from display strings.

## Related

- [Runnable encounters design](../../../scratch/plans/2026-06-22-runnable-encounters-design.md)
- [Encounter mechanics activity model](../../../scratch/plans/2026-06-24-encounter-mechanics-activity-model.md)
- [Rust encounter elite and weak projection](./rust-encounter-elite-weak-projection.md)
- [Rust encounter actor context effects](./rust-encounter-actor-context-effects.md)
