# Rust Encounter Turn Lifecycle Mutations

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-26

## Problem

Encounter runtime projection can now show action budgets, speeds, condition effects, and runtime notes, but turn progression is still mostly navigation. Several PF2E effects need mutation at specific encounter lifecycle points rather than passive projection.

Examples include condition durations that tick down on turn passage, stunned-with-a-value consuming regained or remaining actions, future slowed/quickened integration with turn progression, and persistent damage or recovery-check workflows. If these are added ad hoc from UI button handlers, the encounter runner will split rule timing across browser state, app-service projection, and local-state rows.

## Desired Outcome

Design and implement backend-owned lifecycle mutation hooks for encounter turn events.

The implementation should:

- define explicit encounter events such as start turn, advance turn, set active participant, end turn, and complete round;
- apply deterministic lifecycle mutations in app-service before persisting updated local-state rows;
- support condition duration policies without forcing every condition to auto-decrement;
- model stunned value consumption at action-regain timing and preserve the special own-turn stunned case as a deliberate mutation path;
- leave projected-only reminders, such as current slowed action-budget display, separate from mutation until the timing rules are explicitly implemented;
- keep browser controls thin: the UI requests lifecycle actions and renders the resulting encounter view;
- make mutations idempotent or otherwise guarded so refresh/retry behavior cannot double-apply a turn event.

## Constraints

- Do not put PF2E lifecycle mutation rules in frontend-only code.
- Do not mutate source-derived record mechanics or generated artifact rows.
- Do not silently decrement ambiguous condition durations; automation should be condition-aware or opt-in.
- Keep persistent damage as its own follow-up model unless this work explicitly designs damage-over-time resolution.
- Preserve manual GM override paths for condition values, durations, active participant, and round number.

## Related

- [Rust encounter runtime mechanics surfaces](./rust-encounter-runtime-mechanics-surfaces.md)
- [Rust encounter condition mechanics](./rust-encounter-condition-mechanics.md)
- [Rust encounter actor context effects](./rust-encounter-actor-context-effects.md)
