# Rust Encounter Condition Mechanics

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-23

## Problem

Encounter v1 stores condition annotations with value, duration, source, and notes. It does not mechanically apply condition effects to creature stats. PF2e conditions often have situational rules, so mechanical application needs a deliberately scoped projection model.

## Desired Outcome

Add condition mechanics incrementally for deterministic condition effects.

The design should answer:

- which condition records are mechanically supported first;
- how numeric condition values are interpreted;
- which actor stats are adjusted and which remain annotation-only;
- how unsupported or situational effects are displayed without implying automation;
- whether duration decrementing stays manual or becomes opt-in automation.

## Constraints

- Keep unsupported conditions as durable annotations.
- Do not auto-apply effects that require GM judgment or encounter context the model does not represent.
- Make each mechanically applied condition test-backed against representative records.
- Preserve the v1 freeform condition path for non-standard effects.

## Related

- [Runnable encounters design](../../../scratch/plans/2026-06-22-runnable-encounters-design.md)
- [Rust encounter elite and weak projection](./rust-encounter-elite-weak-projection.md)
