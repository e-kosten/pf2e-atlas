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
- whether duration decrementing stays manual or becomes opt-in automation.

Condition automation should prefer canonical `conditionitems` record identity when `condition_key` is present, then fall back to exact supported slugs for manual/freeform rows. Fuzzy condition names remain annotation-only. The UI should eventually use a canonical condition picker for built-in condition rows while preserving a freeform path for non-standard statuses.

## Constraints

- Keep unsupported conditions as durable annotations.
- Do not auto-apply effects that require GM judgment or encounter context the model does not represent.
- Make each mechanically applied condition test-backed against representative records.
- Preserve the v1 freeform condition path for non-standard effects.
- Expose unapplied effects for modeled conditions when a rule targets attack, damage, spellcasting, formula-bearing values, or other targets that the first typed stat projection does not yet model.
- Track follow-up work for attack, damage, spellcasting, and formula-backed transitive effects instead of silently ignoring them or approximating them from display strings.

## Related

- [Runnable encounters design](../../../scratch/plans/2026-06-22-runnable-encounters-design.md)
- [Rust encounter elite and weak projection](./rust-encounter-elite-weak-projection.md)
