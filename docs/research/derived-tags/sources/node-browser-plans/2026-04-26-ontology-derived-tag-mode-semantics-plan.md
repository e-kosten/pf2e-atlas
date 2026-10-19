# Historical snapshot: 2026-04-26-ontology-derived-tag-mode-semantics-plan.md

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `scratch/plans/archive/2026-04/2026-04-26-ontology-derived-tag-mode-semantics-plan.md`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````markdown
# Ontology Derived-Tag Matching/Catalog Semantics

## Summary

This plan covers a focused follow-up slice in the search-semantics area: make the ontology browser's derived-tag families and tags behave meaningfully when the user switches between `Matching` and `Catalog`.

The intended end state is:

- ontology derived-tag family counts and child lists actually change by discovery mode
- `Matching` means:
  - family count = number of nonzero tags
  - children = only nonzero tags
- `Catalog` means:
  - family count = full family size
  - children = all tags, including zero-count tags
- zero-count catalog tags remain searchable/actionable
- the implementation converges touched broad/prepared derived-tag builder seams where practical rather than deepening duplication

This plan does **not** solve the broader search-side empty derived-tag explorer bug. That issue remains explicitly deferred, though the refactor should avoid making it harder to investigate later.

Backlog anchor:

- `docs/backlog/items/search-semantics-surface-convergence.md`

## Architecture Context

Relevant architecture and ADR sources for this slice:

- `docs/architecture/overview.md`
- `docs/architecture/boundaries.md`
- `docs/architecture/tui.md`
- `docs/architecture/decisions/0005-live-search-semantics-exploration.md`

Key current ownership:

- `src/app/ontology-service.ts` is the app-layer ontology facade and cache owner
- `src/app/ontology/search-semantics-domain.ts` owns broad ontology and prepared search-scoped search-semantics model construction
- `src/tui/pf2e-navigation.ts` owns ontology-route preparation and discovery-mode wiring for the ontology browser
- `src/tui/filter-explorer/*` and `src/tui/ontology-explorer/inspect-screen.tsx` own the shared explorer shell and ontology inspect host, not the derived-tag count semantics themselves

The architecture docs already distinguish:

- a broad ontology browser model from `loadSearchSemanticsDomain()` / `buildSearchSemanticsDomain(...)`
- a prepared search-scoped model from `loadSearchFilterExplorerDomain({ request, discoveryMode })` / `buildPreparedSearchFilterExplorerDomain(...)`

This plan preserves that public split. The convergence target is the touched **internal assembly helpers and mode semantics**, not the public entrypoints.

## Target Model

### Ontology broad browse derived-tag semantics

For ontology derived-tag family nodes only:

- `matching`
  - family node remains visible only if it has at least one nonzero tag
  - family count is the number of nonzero child tags
  - children include only the nonzero child tags
- `catalog`
  - family node remains visible if the family has any authored tags in scope
  - family count is the full authored tag count for the family
  - children include all authored tags in scope, including zero-count tags

For derived-tag leaf nodes:

- the leaf row count is still the discovered count, including `0` in catalog mode
- zero-count catalog tags remain actionable/searchable so the explorer keeps one consistent query/open pathway rather than introducing a special-case disabled leaf model

### Broad vs prepared surface rule

- The ontology browser should become mode-aware enough that its derived-tag family counts and child visibility reflect the chosen mode.
- The prepared search-scoped explorer keeps its own surface-specific semantics for now.
- Touched broad/prepared derived-tag builder logic should be refactored toward shared internal helpers where that improves clarity and reduces duplicated policy decisions.

## Resolved Planning Decisions

1. The ontology browser should use the same underlying `matching` / `catalog` concept rather than inventing a separate zero-count toggle concept.
2. In ontology derived-tag families, the practical mode distinction is exactly:
   - `Matching`: nonzero tags only
   - `Catalog`: full family, including zero-count tags
3. Zero-count catalog tags should still be searchable/actionable.
4. `0` is an acceptable visible count rendering for catalog-only entries.
5. The implementation should refactor touched broad/prepared derived-tag assembly seams where practical rather than stacking another parallel near-copy.
6. The broader search-side empty derived-tag issue remains out of scope for this slice except for avoiding owner drift or making it harder to diagnose later.

## Change Envelope

This plan is intentionally focused. It should change:

- ontology broad-model loading so discovery mode has semantic effect for derived-tag families
- derived-tag family/tag assembly helpers in `src/app/ontology/search-semantics-domain.ts`
- ontology-route/service plumbing needed to request a mode-aware broad model
- related screen/app tests
- relevant TUI architecture docs if the documented mode contract materially changes

This plan should **not**:

- collapse the broad ontology and prepared search-scoped public builders into one public entrypoint
- redesign prepared search-scoped matching/catalog semantics beyond local helper cleanup required by touched code
- solve the search-side `scope=creature` derived-tag emptiness bug
- reopen unrelated explorer or picker behavior

## Execution Model

Implementation should happen from a dedicated `/tmp/pathfinder-mcp-worktrees/` worktree on a new branch from current `main`.

This is a medium architectural slice with a contained refactor component. The primary agent should orchestrate and delegate by slice so the implementation and validation contexts stay small.

Recommended validators:

- Validator A: plan coverage, mode semantics, and broad/prepared owner discipline
- Validator B: regression risk, helper convergence quality, and runnable coverage

## Implementation Slices

### Slice 1: Service And Route Mode-Aware Broad Loading

Owners:

- `src/app/ontology-service.ts`
- `src/tui/pf2e-navigation.ts`
- any adjacent route/type owners needed for the ontology payload

Deliverables:

- add a mode-aware broad ontology loader path, for example `loadSearchSemanticsDomain({ discoveryMode })`, or an equivalent typed contract that does not hide the new semantic input
- keep ontology-service as the public facade and cache owner
- wire ontology route preparation and `loadModelForDiscoveryMode(...)` through the new broad-mode-aware loader
- keep initial ontology broad entry on `matching`

Constraints:

- do not add a compatibility shim that preserves two long-lived broad ontology loader contracts unless there is a concrete blocker
- if caching changes, keep cache keys explicit and mode-aware

Validation:

- focused tests for ontology-service mode-aware loading/caching
- focused route test proving ontology discovery-mode switch reloads through the mode-aware broad loader rather than the old mode-agnostic path

### Slice 2: Broad Derived-Tag Family/Leaf Semantics

Owners:

- `src/app/ontology/search-semantics-domain.ts`

Deliverables:

- make broad ontology derived-tag family nodes mode-aware
- in `matching`, derive family counts and child visibility from nonzero live counts only
- in `catalog`, preserve full family counts and include zero-count child tags
- keep scoped family filtering rules intact so subcategory scoping still works
- preserve actionable queries for zero-count catalog leaves

Validation:

- domain-level tests for broad ontology derived-tag family nodes in `matching`
- domain-level tests for broad ontology derived-tag family nodes in `catalog`
- tests proving zero-count tags render as child nodes in catalog mode and remain queryable

### Slice 3: Shared Helper Convergence In Touched Builder Seams

Owners:

- `src/app/ontology/search-semantics-domain.ts`

Deliverables:

- identify and extract the smallest useful shared helper seam for touched derived-tag family/tag assembly logic across the broad and prepared builders
- centralize repeated policy decisions that are currently duplicated, such as:
  - family-node id construction
  - tag-node id construction
  - scoped subcategory resolution for derived-tag queries
  - consistent derived-tag family/tag label construction
- do **not** force identical broad/prepared semantics where the surfaces legitimately differ

Constraints:

- this is a direct-replacement refactor in the touched area, not a compatibility-layer exercise
- no new third parallel helper family for “ontology-mode-specific derived tags”

Validation:

- validator review explicitly checks for reduced duplication rather than another added near-copy
- repo search and code review pass to confirm no leftover touched old-path helper remains in mixed ownership for the refactored seam

### Slice 4: Docs And Runnable Coverage Follow-Through

Owners:

- `docs/architecture/tui.md`
- relevant tests under `tests/app/` and `tests/tui/`

Deliverables:

- update `docs/architecture/tui.md` so the ontology-browser `matching` / `catalog` behavior is described accurately for derived-tag families if the current docs would otherwise imply a no-op or incomplete semantics
- ensure the runnable test suite captures any user-visible semantics found during validation
- do not leave the behavior proven only by manual inspection

Validation:

- docs/code agreement review
- test review proving the new ontology-mode semantics are encoded in runnable coverage

## Docs And ADR Updates

Required docs follow-through:

- update `docs/architecture/tui.md` if the ontology broad-browse mode semantics are no longer accurately described

ADR expectation:

- no new ADR is required unless the implementation materially changes the broader documented contract between broad ontology and prepared search-scoped models beyond the scope already captured in the architecture docs and backlog item

## Validation Plan

Targeted validation should include:

- broad ontology derived-tag family/domain tests for both `matching` and `catalog`
- route/service tests proving ontology mode switching uses the mode-aware broad loader
- TUI or screen-level coverage proving the ontology explorer action-rail mode switch actually changes visible family/tag counts
- verification that zero-count catalog tags still open/search through the standard path

Final validation commands after implementation:

- `npm run build`
- `cd scripts && npm test`

Plan-completion validation must also confirm:

- no compatibility-layer broad loader was left behind unintentionally
- broad and prepared public entrypoints remain distinct
- touched derived-tag helper duplication was reduced rather than increased
- docs and code agree about ontology derived-tag matching/catalog behavior

## Landing Workflow

1. Create a dedicated worktree under `/tmp/pathfinder-mcp-worktrees/`.
2. Implement and validate slices there.
3. Rebase the worktree branch onto current `main`.
4. Rerun `npm run build` and `cd scripts && npm test` in the rebased worktree.
5. Merge back to clean `main`.
6. Rerun `npm run build` and `cd scripts && npm test` on `main`.
7. Remove the temporary worktree.

## Assumptions, Blockers, And Open Questions

Assumptions:

- the ontology broad browser should stay usable without a live `SearchRequest`
- zero-count catalog leaves are acceptable as ordinary actionable search seeds
- the user wants semantic effect in ontology mode switching now, while deferring the broader prepared search-side empty-derived-tag investigation

Current out-of-scope but related open issue:

- the search-side derived-tag surface can still appear empty for broad scoped cases such as only `scope=creature`

Possible blocker to surface during implementation:

- if the cleanest helper convergence path would force a larger-than-expected rewrite of broad/prepared builder ownership, stop and check in rather than quietly broadening this slice into a larger architecture migration
````
