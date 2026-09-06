# Rust Encounter Runtime Mechanics Surfaces

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-25

A7 candidate disposition: **subsumed** by E1-E3 and F2 for typed movement/action/resource targets, deterministic effect projection, composed runtime DTOs, and feature-owned controls. Checkpoint B is not yet approved.

## Problem

Encounter condition automation currently targets typed stat rows such as AC, saves, skills, and activity rolls. Several useful PF2e encounter effects target runtime surfaces that are not currently represented as first-class projections, especially movement speeds and action economy.

Without these surfaces, conditions such as slowed, quickened, stunned, immobilized, grabbed, restrained, prone, and encumbered can only be tracked as notes or overfit into unrelated stat-modifier rows. Traits such as minion also affect action economy but are not conditions, which suggests the encounter runner needs a broader runtime-effect projection instead of condition-only stat mutation.

## Desired Outcome

Add typed encounter runtime surfaces for movement speeds and action budget, then let deterministic conditions and future runtime effect sources modify those surfaces with clear explanations.

The implementation should:

- derive speed facts from record-owned mechanics or metrics;
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

- [ADR 0034 candidate: Canonical entities, occurrences, runtime instances, and projections](../../architecture/decisions/0034-canonical-entities-occurrences-and-projections.md)
- [ADR 0036 candidate: Source-faithful record surfaces](../../architecture/decisions/0036-source-faithful-record-surfaces.md)
- [Rust encounter condition mechanics](./rust-encounter-condition-mechanics.md)
- [Rust encounter actor context effects](./rust-encounter-actor-context-effects.md)
- [Rust Foundry type mechanics parsers](../history/items/rust-foundry-type-mechanics-parsers.md)
