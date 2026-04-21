# ADR 0008: Search Filters And Concern-Specific Tag Facades

- Status: Accepted
- Date: 2026-04-20

## Context

Two parts of the repo had grown ambiguous:

- live search-filter behavior was split across `src/domain/`, `src/data/backend/`, `src/search/`, and `src/search/sql.ts`
- non-tag callers had one broad `src/tags/index.ts` facade that mixed runtime, editorial orchestration, and UI exports

That ambiguity raised change cost, encouraged broad imports, and made the documented layer boundaries less trustworthy.

## Decision

- centralize live metadata-filter and search-filter ownership under `src/search/filters/`
- keep `src/search/sql.ts` focused on SQL construction rather than also owning normalization and record-level matching
- remove the broad `src/tags/index.ts` facade
- expose three concern-specific top-level tag facades instead:
  - `src/tags/runtime.ts`
  - `src/tags/editorial.ts`
  - `src/tags/editorial-ui.ts`

## Consequences

- filter changes now route through one search-owned surface instead of drifting across backend and SQL helpers
- domain code no longer has an approved broad `src/domain/index.ts` barrel
- non-tag callers choose the tag facade that matches their concern instead of importing one catch-all barrel
- lint rules and architecture docs can enforce narrower, more durable integration paths
