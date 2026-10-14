# Shared Record and Encounter Presentation

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-10-13

## Current Baseline

App-service composes semantic family presentation from checked source facts and,
for encounters, bounded participant overlays. Browser detail, preview, comparison
and encounters reuse family components. CLI rendering uses the same meaning with
its own layout. [ADR 0047](../../architecture/decisions/0047-semantic-record-presentation.md)
defines this contract.

## Remaining Work

Evaluate where family layouts and information hierarchy need refinement as
additional consumers and record kinds become useful. Preserve introductory prose,
readable mechanics, exact embedded navigation and explanations of adjusted facts.
Keep participant state and authored source context distinguishable without
unnecessary repeated information.

A future table view with useful configurable columns remains a presentation
option to evaluate against actual scan/comparison workflows. Agree its required
facts and summary-read cost before extending the search response.

Coordinate specific work through [shared UI consistency](shared-ui-consistency-and-saved-lists.md),
[record reading](rust-web-record-detail-polish.md),
[encounters](rust-web-encounter-runner-polish.md) and
[spell previews](rust-spell-rank-preview.md). These tasks refine the shared model;
this item does not propose another presentation contract.

Keep mechanics and semantic composition Rust-owned, presentation layouts
consumer-owned, and references attributable to their checked source addresses.
