# Rust Record Presentation Mechanics Unification

Status: done
Priority: now
Owner: unassigned
Last reviewed: 2026-09-08

## Problem

Record detail and encounter views previously used separate presentation paths. Rich prose, structured mechanics, adjusted encounter facts, references, and runtime state could repeat or disagree, while browser code risked reconciling facts that belong to typed backend owners.

## Landed Outcome

The completed implementation:

- uses app-service `RecordSurfaceView` composition for record detail and encounter participant views;
- presents creature, hazard, and standalone-spell mechanics from their canonical typed bodies instead of rebuilding them from generic rows or prose;
- keeps authored rich content, provenance, occurrences, and resolved references attached to their owning records and components;
- resolves spell forms and cast ranks on the server and returns one effective selected definition with explicit unavailable states;
- composes encounter adjustments and mutable runtime state beside the same static record facts without duplicating browser-side mechanics rules;
- provides typed incoming and outgoing reference navigation through the shared search contract;
- renders responsive, accessible record and search surfaces with stable loading, error, selection, and keyboard behavior; and
- preserves the generated artifact and durable local-state databases as separate lifecycles.

The Creature, Hazard, and standalone Spell presentation implementation passed focused Rust and frontend validation, Linux artifact lifecycle checks, independent backend and frontend review, and human product/visual review. PR 7 remains draft through the required modeling or explicit human disposition of every other registry family. Simplification and main-promotion planning follow that broader modeling milestone rather than this item's closure.

## Preserved Constraints

- Browser code does not parse raw Foundry prose or duplicate backend mechanics rules.
- Authored rich prose and source/provenance evidence remain available.
- Canonical entities, occurrences, and runtime instances retain separate ownership.
- Generated indexes remain rebuildable source-derived data; user-authored saved lists and encounters remain in separately versioned local state.
- Later record families, including consumables, remain required modeling portfolios with their own typed owners and are not implied by this completed three-family presentation work.

## Related

- [Architecture overview](../../../architecture/overview.md)
- [ADR 0034: Canonical entities, occurrences, runtime instances, and projections](../../../architecture/decisions/0034-canonical-entities-occurrences-and-projections.md)
- [ADR 0036: Source-faithful record surfaces](../../../architecture/decisions/0036-source-faithful-record-surfaces.md)
- [Rust encounter condition mechanics](../../items/rust-encounter-condition-mechanics.md)
- [Rust encounter elite and weak projection](../../items/rust-encounter-elite-weak-projection.md)
