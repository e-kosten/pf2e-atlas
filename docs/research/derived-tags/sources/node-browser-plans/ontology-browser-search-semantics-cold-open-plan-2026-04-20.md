# Historical snapshot: ontology-browser-search-semantics-cold-open-plan-2026-04-20.md

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `scratch/plans/archive/2026-04/ontology-browser-search-semantics-cold-open-plan-2026-04-20.md`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````markdown
# Ontology Browser Search-Semantics Cold-Open Cleanup

## Summary

The `Ontology Browser` area now lands directly in `searchSemantics`, but the first open still stutters because the app synchronously builds the full `searchSemantics` domain on cache miss.

This pass should do two things together:

1. remove the retired generic ontology-domain surface so no dead chooser-era code remains
2. make the surviving `searchSemantics` entry path cheap enough that opening `Ontology Browser` and related small menus does not visibly block

The end state is:

- `Ontology Browser` means the shared `searchSemantics` explorer only
- there is no public generic ontology-domain chooser API left behind
- category-picker and other small menus no longer route through the full `getSearchVocabulary()` path
- `searchSemantics` bootstrap stays lazy and summary-driven rather than eagerly expanding child value spaces

## Architectural Decisions

### 1. Retire the generic ontology-domain surface

- Remove the public generic ontology service shape that exposes `listDomains()` plus `loadDomain(id)`.
- Replace it with an explicit search-semantics-oriented ontology API such as `loadSearchSemanticsDomain()`.
- Remove `catalogCategories` from the public ontology-facing type surface and summaries.
- Delete dead runtime code, tests, and docs that only existed for the old chooser flow.
- Do not keep a compatibility shell for `catalogCategories`; the goal is to eliminate the stale path, not preserve it.

### 2. Split small summary reads from full vocabulary reads

- Add a lightweight cached backend/app summary surface for the data needed to bootstrap `searchSemantics` and populate small category menus.
- That summary must include:
  - category counts
  - subcategory counts by category
  - common traits by category
  - common derived tags by category
  - lightweight derived-tag family/tag catalog data needed for `searchSemantics`
- Category-picker and similar small menu reads must stop calling full `getSearchVocabulary()`.
- Full vocabulary reads may remain only where they are still truly needed.

### 3. Keep `searchSemantics` bootstrap lazy

- Rebuild `searchSemantics` bootstrap assembly around the new lightweight summary data.
- Do not eagerly enumerate full field-value children just to build shortcut groups.
- Build common-trait and common-derived-tag shortcut groups directly from summary data instead of forcing child loaders to run during root assembly.
- Keep field values, metric values, and record children behind existing lazy child-loading paths.
- Preserve current explorer behavior and query semantics; this is a performance and cleanup pass, not a product-model redesign.

## Sub-Agent Rollout

This task should be implemented as a coordinated large-task rollout with the primary agent acting as orchestrator/integrator, not as the owner of a major implementation slice.

### Worker 1: Ontology API and legacy-surface retirement

Ownership:

- app-layer ontology service
- ontology-facing shared types
- docs/tests describing the retired public domain surface

Responsibilities:

- remove `listDomains()` from the public app ontology service
- replace generic domain loading with an explicit search-semantics load path
- retire `catalogCategories` from ontology-facing type unions and summaries
- delete dead chooser-era runtime code
- update architecture/docs to describe direct entry into `searchSemantics`

Validation:

- targeted ontology-service and typing tests
- no remaining runtime or doc references to a live generic ontology-domain chooser

### Worker 2: Backend summary caching and small-menu data split

Ownership:

- data/catalog summary reads and caches
- TUI search-service reads that currently depend on full `getSearchVocabulary()` for small menu surfaces

Responsibilities:

- add lightweight cached summary readers for category and `searchSemantics` bootstrap data
- stop routing category-picker style reads through full vocabulary assembly
- update TUI search service menu helpers to consume the new summary path

Validation:

- targeted data/search-service tests proving repeated reads are cached
- simple timing checks showing the new summary path is materially cheaper than the current full vocabulary path

### Worker 3: `searchSemantics` domain assembly slimming

Ownership:

- `src/app/ontology/search-semantics-domain.ts`
- related helper seams used for bootstrap assembly

Responsibilities:

- refactor `searchSemantics` bootstrap assembly to use Worker 2’s summary surface
- remove eager shortcut construction that forces full child-value loading at domain build time
- preserve current explorer structure and query behavior while keeping bootstrap cheap

Validation:

- targeted ontology-domain tests
- measured cold/warm `searchSemantics` load timings before and after

### Worker 4: TUI integration and regression coverage

Ownership:

- TUI integration points for ontology-browser entry and related small menus
- terminal-app and search-screen/search-service regression tests

Responsibilities:

- adapt TUI code to the explicit single-surface ontology API
- remove remaining test fixtures or assumptions that rely on the retired generic domain surface
- add or update coverage for direct `Ontology Browser` entry, snapshot restoration, and non-heavy category-menu reads

Validation:

- focused TUI tests for terminal app and search service/screen flows
- no stale UI copy or tests referencing the retired chooser surface

### Orchestrator responsibilities

- create the worktree and assign disjoint write scopes
- keep Worker 1 and Worker 4 aligned on the public ontology API change
- keep Worker 2 and Worker 3 aligned on the exact lightweight summary interface before implementation starts
- review each worker’s changed files, validation output, and remaining risks
- integrate all validated slices
- run final full-repo validation and merge only after the integrated branch is green

## Public Interface Changes

- remove public `listDomains()` from the app ontology service
- replace generic domain-loading usage with an explicit `searchSemantics` load method
- remove `catalogCategories` from public ontology-facing type unions and summaries
- add a lightweight cached summary interface in the backend/app layers for category and `searchSemantics` bootstrap data

## Test and Validation Plan

### Functional coverage

- entering `Ontology Browser` loads `searchSemantics` directly
- returning from ontology-launched search restores the exact explorer snapshot
- category-picker and similar small menus still show correct options/counts
- `searchSemantics` still exposes the same intended category, shortcut, field, derived-tag, and metric surfaces

### Performance coverage

- measure cold and warm timings for:
  - `Ontology Browser` entry
  - direct `searchSemantics` domain load
  - category-picker open
- confirm repeated opens do not rebuild heavyweight summary data

### Final repo validation

- `npm run build`
- `npm test`

## Assumptions

- `searchSemantics` is the only remaining public ontology-browsing surface the TUI should target.
- `catalogCategories` should be removed rather than deprecated.
- This pass should preserve current explorer/query behavior except for removing the retired generic ontology-domain surface and reducing cold-open/menu stutter.
````
