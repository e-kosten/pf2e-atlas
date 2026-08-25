# Rust Record Presentation Mechanics Unification

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-27

A7 candidate disposition: **superseded** by E1-E3/F1-F3/G1. The prior v1 adjusted sidecar plus retained creature fallback is replaced by shared canonical facts and profile projections, with fallback removal only after Checkpoint E. Checkpoint B is not yet approved.

## Problem

Encounter adjusted mechanics now render structured activity and damage facts in the encounter inspector, while normal record detail rendering still uses the rich prose-oriented presentation document. This is acceptable for the first encounter mechanics slice, but it creates two presentation paths for facts that should eventually feel like one coherent record view.

The broader record detail experience also needs to become more character-sheet-like and less like a generic rich-text dump. Static source facts, adjusted encounter facts, runtime HP/conditions, activities, rich prose, and references need one final composition point that decides what should be shown, replaced, suppressed, annotated, or linked for a given view.

## Desired Outcome

Unify or standardize record presentation so rich prose, structured mechanics, activity facts, and adjusted encounter mechanics share a deliberate presentation model.

The end state should let record detail views display structured spells, strikes, saves, damage, and prose in one readable hierarchy while still allowing encounter runtime projections to overlay adjusted values and modifier explanations.

The chosen v1 target is:

- use a shared `RecordSurfaceView`-style app contract composed by `atlas-app-service`;
- use rich list rows as the default creature search result presentation, with no search-row mutation controls or variant selector;
- preserve a future table-compatible search mode with configurable columns backed by stable surface fact keys and filter/discovery fields;
- use the structured encounter participant layout from `scratch/plans/2026-06-26-record-surface-live-implementation-plan.md` for selected encounter participants;
- render adjusted final values inline, highlight modified values, and expose base/final/modifier explanations through tooltip or popover details instead of inline `base ...` text;
- hide participant notes by default behind a note-specific encounter affordance, and never show participant notes in search;
- continue rendering the current static `RecordPresentationDocument` at the bottom of creature detail/encounter surfaces during v1 as a sanity check against information loss.

The implementation should:

- separate semantic fact extraction from final user-facing presentation composition;
- introduce a composed app presentation for contexts such as record detail and encounter participant detail;
- consume structured embedded activity, spellcasting, and resource entities from the artifact instead of treating embedded capability prose as equivalent to native description text;
- avoid rendering an adjusted runtime sidecar above a complete static record presentation when that duplicates HP, defenses, movement, activities, or other facts;
- support future character-sheet-style record layouts that organize defenses, actions, activities, traits, spells, prose, and references intentionally;
- let app-service compose source-derived facts with local/runtime encounter state before the browser renders the final shape;
- keep the browser responsible for layout components and interactions, not reconciling source facts against adjusted facts.

## Constraints

- Do not make the browser parse raw Foundry text or duplicate backend mechanics rules.
- Keep authored rich prose available; structured mechanics should supplement or organize it, not discard it.
- Preserve source/provenance links so activity facts can still open or reference the content they came from.
- Avoid rewriting the central record reader during the first encounter mechanics implementation unless the mechanics DTO shape requires it.
- Do not continue adding runtime-only presentation sidecars without revisiting whether the composed record/participant presentation should own that surface.

## Related

- [ADR 0034 candidate: Canonical entities, occurrences, runtime instances, and projections](../../architecture/decisions/0034-canonical-entities-occurrences-and-projections.md)
- [ADR 0036 candidate: Source-faithful record surfaces](../../architecture/decisions/0036-source-faithful-record-surfaces.md)
- [Rust encounter condition mechanics](./rust-encounter-condition-mechanics.md)
- [Rust encounter elite and weak projection](./rust-encounter-elite-weak-projection.md)
