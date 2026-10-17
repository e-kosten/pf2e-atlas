# Rust Encounter Condition Mechanics

Status: planned
Priority: later
Owner: unassigned
Last reviewed: 2026-10-16

## Problem

Encounters track conditions, round durations, expiry confirmation and persistent
damage reminders. Mechanical projection is incomplete: a condition can adjust
some displayed values while leaving attacks, damage or spellcasting unchanged.
Condition-specific manual notices make that limitation visible but do not finish
the automation.

## Desired Outcome

Complete mechanical automation for standard conditions. Collect the typed context
needed for each effect and apply it consistently across creature facts, attacks,
damage, spellcasting and action availability. Conditions should not remain
partially implemented merely because their effects cross presentation groups.

Prioritize the current gaps, including Enfeebled Strength-based attacks/damage,
Clumsy Dexterity-based attacks, Stupefied spellcasting, conditional AC effects,
restricted extra actions and condition-specific turn transitions. Use Foundry
types/rules and authoritative PF2e rules to establish applicability; do not infer
attack ability or rule identity from names or formatted text.

The initial implementation supports these conditions through the shared typed target and modifier-resolution machinery:

- frightened;
- sickened;
- off-guard;
- clumsy;
- enfeebled;
- stupefied.

The follow-up must establish:

- how numeric condition values are interpreted;
- which actor stats are adjusted and which remain annotation-only;
- how unsupported or situational effects are displayed without implying automation;
- complete coverage of the standard condition catalog, including effects that
  need explicit target/context choices rather than an unconditional modifier;
- source-grounded tests for complete effects and interactions, including typed
  modifier stacking, representative creatures and turn boundaries.

Done means standard conditions have complete supported mechanical behavior;
genuine table decisions expose the necessary user interaction. No known mechanical
effect is silently skipped or replaced indefinitely by a generic unsupported
notice. Custom/unknown effects can remain durable annotations. Automatic dice,
recovery adjudication and unconfirmed expiry removal remain separate product
decisions; completing numerical automation does not authorize them.

The approved [encounter workflow revision](rust-web-encounter-runner-polish.md)
owns round-duration countdowns, explicit expiry confirmation and actionable
persistent-damage reminders. Non-round durations remain reminders. Configurable
automatic dice/recovery and additional condition-specific turn changes need their
own rules and interaction design; duration countdown does not imply automatic
condition-value reduction.

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

- Runnable encounters design (historical local plan: `scratch/plans/archive/2026-06/2026-06-22-runnable-encounters-design.md`)
- Encounter mechanics activity model (historical local plan: `scratch/plans/archive/2026-06/2026-06-24-encounter-mechanics-activity-model.md`)
- Encounter runtime surfaces plan (historical local plan: `scratch/plans/archive/2026-06/2026-06-25-encounter-runtime-surfaces-plan.md`)
- [Rust encounter elite and weak projection](./rust-encounter-elite-weak-projection.md)
- [Rust encounter actor context effects](./rust-encounter-actor-context-effects.md)
- [Rust encounter runtime mechanics surfaces](./rust-encounter-runtime-mechanics-surfaces.md)
