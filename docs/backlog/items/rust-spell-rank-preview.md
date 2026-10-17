# Rust Spell Rank Preview

Status: planned
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-16

## Problem

Spell reference shows authored baseline, fixed/interval heightening and form
changes. Users cannot yet preview how a selected rank changes supported spell
facts. Adding a rank selector without a computed result would misrepresent the
current reference contract.

Creature spell lists are being simplified to show casting rank, with base-rank
and heightening context in the opened spell. Correct-rank spell rendering is a
required separate follow-up to the encounter workflow, rather than an optional
rank-selector feature. Agree its detailed interaction before implementation.

## Desired Outcome

Design a bounded Rust-owned preview operation using the exact checked root/owner
address, requested rank and optional form. Preserve parent-owned customizations.
Return supported effective facts, change explanations, and explicit unavailable
or unsupported outcomes.

Opening an actor-owned spell must show its mechanics at the authored casting
rank where that rank is known, rather than requiring the user to discover and
apply a separate patch. Preserve the exact embedded copy and its customizations.
First establish upstream persisted-versus-derived semantics: an embedded spell
is not evidence that heightening has already been applied, and a modified formula
is not evidence for guessing whether to apply it again.

A concrete corpus example is Fabled Harrowkin's embedded Dimension Door: base
rank 4, `location.heightenedLevel` 5, stored range 120 feet, with the rank 5
heightening patch specifying 1 mile. Cover that case and interval damage in regression tests,
alongside customized embedded spells, equal/missing ranks and cantrip rules.
Keep supported area/range/damage changes coherent in the presented spell, and
make field-specific limitations visible without labelling baseline facts as
successfully calculated higher-rank mechanics.

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
