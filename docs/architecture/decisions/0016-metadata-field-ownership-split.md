# 0016 Metadata Field Ownership Split

## Status

Accepted

## Context

Metadata fields are shared search vocabulary. MCP search semantics, ontology browsing, TUI filter editing, backend filter execution, normalized record hydration, and MCP response presentation all need metadata-field facts, but they need different kinds of facts.

The architecture keeps public search meaning separate from execution and storage concerns. A module that mixes field meaning, SQL expressions, row aliases, record hydration, and presentation policy becomes a cross-layer dependency that surfaces have legitimate reasons to import.

## Decision

Metadata field ownership is split by concern:

- `src/domain/metadata-field-catalog.ts` owns public metadata field meaning: field identity, field type, category and subcategory applicability, discoverability, operators, value ordering, examples, and metric discovery guidance.
- `src/search/filters/metadata-execution.ts` owns search execution behavior for metadata fields: value normalization, execution specs, and record-value access used by search matching.
- `src/data/metadata-row-projection.ts` owns metadata row selection and hydration mapping for `NormalizedRecord`.
- `src/data/backend/metadata-search-sql.ts` owns metadata SQL expression placement, filter-value SQL sources, and physical predicate query construction.
- `src/server/metadata-presentation.ts` owns MCP summary/detail metadata projection.

Surface and app layers use the domain catalog for search-space meaning. Data uses the data row projection. Search uses the execution helpers. Server presentation uses the server presentation projection.

## Consequences

`src/search/filters/` no longer owns public metadata semantics or physical SQL construction. App, domain, server, and TUI modules must not import search-owned metadata execution helpers for field meaning or row hydration. Data-owned physical retrieval may import search-owned normalization helpers when lowering normalized filters into SQL, but row hydration remains owned by `src/data/metadata-row-projection.ts`.

Adding a metadata field requires updating the public catalog and then adding the concern-specific execution, hydration, or presentation pieces that apply to that field. Tests should check that executable metadata fields have execution specs and that public semantics remain available to MCP, ontology, and TUI callers.
