# Rust Side Data And Metric Source Fact Convergence

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-17

## Problem

Rust ingest projects some overlapping Foundry facts into both stable side tables and open metric rows. Examples include speed and sense source data, spell range and area values, and hazard disable facts. These projections serve different runtime contracts, but their extraction should not drift into separate raw JSON parsers for the same source fields.

Side tables are stable named facets and presentation axes. Metrics are open scalar values used for comparison, catalogs, and numeric filtering. Keeping both representations can be correct, but duplicated source parsing makes it easier for one projection to accept aliases, coercions, or fallback paths that the other projection misses.

## Desired Outcome

Introduce a shared source-fact or source-spec projection shape for overlapping side-data and metric families so the source field is interpreted once and then projected into the appropriate side table and metric rows.

The cleanup should preserve the different runtime roles:

- side tables remain the home for stable facets such as languages, traditions, damage types, save type, and boolean spell facts;
- metric rows remain the home for open scalar values such as modifiers, DCs, speeds, ranges, and numeric item facts;
- overlapping source families derive from a shared typed source fact or spec rather than independent raw pointer parsing.

## Candidate Areas

- actor speed types and `actor.speed.*.value` metrics
- actor sense names and `actor.sense.*.range` metrics
- hazard disable skills/text and disable metric families
- spell range/area side-data values and future spell metric families
- item damage-type side data and item damage metrics where source fields overlap

## Constraints

- Do not collapse side tables and metrics into one storage model.
- Do not make side-table projection depend on persisted metric rows unless that dependency is explicitly accepted as the source of truth.
- Prefer deriving both projections from ingest-owned source facts/specs before artifact writing.
- Keep this separate from the embedded source entity migration unless a small helper naturally falls out of that work.

## Acceptance Sketch

- Each migrated overlapping family has one source interpretation path.
- Tests prove side-data and metric projections stay aligned for fallback/alias source paths.
- Residual duplicated raw parsing is either removed or explicitly documented as non-overlapping.
- Architecture docs describe the distinction between side-table facets and metric scalars.

## Related

- [Rust Embedded Source Entities And Promotions](../../../scratch/plans/2026-05-17-rust-embedded-source-entities-and-promotions.md)
- [Rust artifact contract](../../architecture/artifact-contract.md)
