# ADR 0004: Non-Tag Ownership Import Boundaries

- Status: Accepted
- Date: 2026-04-19

## Context

Several helpers and contracts that were once reachable through broad shared barrels now have clearer owners:

- `src/domain/index.ts` is no longer an approved import path
- `src/shared/utils.ts` still hosts a few true cross-layer primitives, but app/data/search-specific helpers have been moved to owner modules
- `src/shared/fs.ts` still exposes explicit helpers, but the `fileExists` alias is only compatibility glue

Without enforcement, non-tag code can slide back toward broad shared imports and undo that ownership cleanup.

## Decision

Use lint rules and architecture docs to make the non-tag ownership paths explicit:

- non-tag code imports domain contracts from concrete `src/domain/*` modules instead of `src/domain/index.ts`
- `src/shared/` stays intentionally tiny and only keeps true cross-layer primitives
- moved non-tag helpers stay with their owner modules in `src/data/` and `src/search/`
- non-tag code uses explicit filesystem helpers such as `pathExists` and `pathIsReadable` instead of compatibility aliases

## Consequences

- Non-tag code keeps a clearer dependency graph and avoids broad shared barrels.
- Future helper extraction work has a documented default: keep the helper with its owner unless it is truly cross-layer.
- Non-tag code no longer has a fallback broad domain barrel to drift back toward.
