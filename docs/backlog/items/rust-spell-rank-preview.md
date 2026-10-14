# Rust Spell Rank Preview

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-13

## Problem

Spell reference shows authored baseline, fixed/interval heightening and form
changes. Users cannot yet preview how a selected rank changes supported spell
facts. Adding a rank selector without a computed result would misrepresent the
current reference contract.

The user also flagged creature-roster wording such as “Base rank 4 / Cast rank 5”
as confusing. Review that wording and its relationship to selected-rank preview
before implementation. Agree the presentation with the user first.

## Desired Outcome

Design a bounded Rust-owned preview operation using the exact checked root/owner
address, requested rank and optional form. Preserve parent-owned customizations.
Return supported effective facts, change explanations, and explicit unavailable
or unsupported outcomes.

Agree fixed-layer ordering, interval steps, formula support, component identity,
overflow behavior and field availability before implementation. Missing patches
mean unchanged; null patches mean clear. Retain prepared source prose without
using it as a general mechanics parser.

## Constraints

- Update ADR0047 deliberately before adding selected-rank execution.
- Keep computation in Rust rather than frontend renderers.
- Do not imply arbitrary Foundry rule execution, resource spending or encounter
  casting mutations.
- Reuse shared spell presentation across reference, preview and encounter surfaces.
- Treat the older integration's feature as requirements inspiration, not as
  mechanically validated code to copy.

## Related

- [Semantic record presentation](../../architecture/decisions/0047-semantic-record-presentation.md)
- [Rust web record detail polish](rust-web-record-detail-polish.md)

Historical discussion (local untracked plan):
`scratch/plans/archive/2026-10/2026-10-10-record-presentation-followup.md#5-separate-selected-rank-preview-design`.
