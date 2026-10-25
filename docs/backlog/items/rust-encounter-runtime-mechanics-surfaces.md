# Rust Encounter Runtime Mechanics Surfaces

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-10-24

## Problem

Encounters already expose authored movement speeds and bounded condition-derived
action counts/capabilities. Additional effect targets and richer explanations
need explicit source and rule evidence. Traits such as minion affect action
economy independently of conditions. The current lifecycle records actual regained
allowances, explicit manual expenditure, separate reaction stock/permission and
Stunned consumption (ADR0048/0055; PR #105 remains under review). Source-owned
conditions/grants and source-specific reaction restrictions still need reviewed
projection support; do not guess these from actor names or silently treat Slowed
as a no-reaction rule.

## Desired Outcome

Extend movement/action effect attribution and carefully scoped runtime targets.

The implementation should:

- derive speed facts from borrowed authored DTO views;
- derive action budget in app-service encounter projection;
- model action counts separately from action/reaction capability, so effects such as stunned do not become misleading reaction-count math;
- support chained modeled condition effects, so a condition such as encumbered can reuse the canonical clumsy rule while also applying its own runtime speed effect;
- keep local-state limited to saved participant state, conditions and actual turn resources;
- render speed/action surfaces in the encounter UI;
- apply only deterministic condition effects;
- keep ambiguous or actor-vs-actor effects as tracked/reference notes.

## Constraints

- Do not parse raw Foundry JSON in `atlas-app-service` or `web/atlas-ui`.
- Do not persist passive speed/action projections as caches in local-state; the actual
  regained allowance and explicit expenditure are durable turn state, not a live forecast.
- Keep browser condition logic presentation-only.
- Do not fold actor-vs-actor visibility or targeting into this work.
- Use the existing dedicated persistent-damage/recovery model rather than adding a parallel path.
- Speed penalties with floors, such as encumbered's 5-foot minimum, must be represented explicitly rather than approximated with ordinary additive stat modifiers.
- Preserve explicit own-turn Stunned resolution after the current action/activity
  finishes; use recorded remaining actions, never a new three-action forecast.

## Related

- Encounter runtime surfaces plan (historical untracked reference: `../../../scratch/plans/archive/2026-06/2026-06-25-encounter-runtime-surfaces-plan.md`)
- [Rust encounter condition mechanics](rust-encounter-condition-mechanics.md)
- [Rust encounter actor context effects](rust-encounter-actor-context-effects.md)
- [Rust Foundry type mechanics parsers](../history/items/rust-foundry-type-mechanics-parsers.md)
