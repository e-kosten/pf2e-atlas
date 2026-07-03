# ADR 0008: Search Filters And Concern-Specific Tag Facades

- Status: Accepted
- Date: 2026-04-20

## Context

Two parts of the repo had grown ambiguous:

- live search-filter behavior was split across `src/domain/`, backend data modules, runtime search modules, and a search-layer SQL helper
- non-tag callers had one broad `src/tags/index.ts` facade that mixed runtime, editorial orchestration, and UI exports

That ambiguity raised change cost, encouraged broad imports, and made the documented layer boundaries less trustworthy.

## Decision

- centralize live metadata-filter normalization and record-level matching ownership under `src/search/filters/`
- keep SQL construction separate from normalization and record-level matching
- remove the broad `src/tags/index.ts` facade
- expose three concern-specific top-level tag facades instead:
  - `src/tags/runtime.ts`
  - `src/tags/editorial.ts`
  - `src/tags/editorial-ui.ts`

## Consequences

- filter normalization and record-level matching changes now route through one search-owned surface, while physical SQL construction stays with data
- domain code no longer has an approved broad `src/domain/index.ts` barrel
- non-tag callers choose the tag facade that matches their concern instead of importing one catch-all barrel
- lint rules and architecture docs can enforce narrower, more durable integration paths
