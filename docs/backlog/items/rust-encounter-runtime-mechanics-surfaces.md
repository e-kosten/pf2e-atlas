# Rust Encounter Runtime Mechanics Surfaces

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-10-09

## Problem

Encounters already expose authored movement speeds and bounded condition-derived
action counts/capabilities. Additional effect targets and richer explanations
need explicit source and rule evidence. Traits such as minion affect action
economy independently of conditions; turn-timed effects require a lifecycle model.

## Desired Outcome

Extend movement/action effect attribution and carefully scoped runtime targets.

The implementation should:

- derive speed facts from borrowed authored DTO views;
- derive action budget in app-service encounter projection;
- model action counts separately from action/reaction capability, so effects such as stunned do not become misleading reaction-count math;
- support chained modeled condition effects, so a condition such as encumbered can reuse the canonical clumsy rule while also applying its own runtime speed effect;
- keep local-state limited to participant state and condition rows;
- render speed/action surfaces in the encounter UI;
- apply only deterministic condition effects;
- keep ambiguous or actor-vs-actor effects as tracked/reference notes.

## Constraints

- Do not parse raw Foundry JSON in `atlas-app-service` or `web/atlas-ui`.
- Do not persist derived speed/action values in local-state rows.
- Keep browser condition logic presentation-only.
- Do not fold actor-vs-actor visibility or targeting into this work.
- Keep persistent damage as tracked/freeform until it has a dedicated damage-over-time model.
- Speed penalties with floors, such as encumbered's 5-foot minimum, must be represented explicitly rather than approximated with ordinary additive stat modifiers.
- Treat own-turn stunned application as later runtime mutation work: after the current action/activity finishes, remaining actions should be lost immediately to reduce the stunned value.

## Related

- Encounter runtime surfaces plan (historical untracked reference: `../../../scratch/plans/2026-06-25-encounter-runtime-surfaces-plan.md`)
- [Rust encounter condition mechanics](rust-encounter-condition-mechanics.md)
- [Rust encounter actor context effects](rust-encounter-actor-context-effects.md)
- [Rust Foundry type mechanics parsers](../history/items/rust-foundry-type-mechanics-parsers.md)
