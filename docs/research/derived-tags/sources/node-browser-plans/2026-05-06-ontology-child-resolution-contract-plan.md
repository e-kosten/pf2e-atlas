# Historical snapshot: 2026-05-06-ontology-child-resolution-contract-plan.md

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `scratch/plans/archive/2026-05/2026-05-06-ontology-child-resolution-contract-plan.md`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````markdown
# Ontology Child Resolution And Materialized Tree Contract

## Summary

Fix the scoped pack picker bug by removing the broken generic sync child-access contract. The current `getOntologyNodeChildren(...)` returns `[]` for lazy child sources, so callers can accidentally treat "not loaded yet" as "has no children." Scoped pack editing hits that bug because prepared scoped pack values are lazy, while direct-value picker assembly flattens children synchronously.

The intended end state is stricter:

- ontology nodes describe potential structure, not current loaded UI state
- async resolvers load the exact branch a workflow needs
- TUI sessions own explicitly materialized branch state
- no generic helper exposes "whatever children happen to be loaded"
- no option/result/model builder can use partial materialized state as truth

This supersedes the narrower pack-only plan in `scratch/plans/archive/2026-05/2026-05-06-filter-builder-pack-scope-prompt-fixes.md`.

## Architecture Context

- `src/app/ontology/node-helpers.ts` currently owns lazy child caching and exposes three overlapping access patterns: `getOntologyNodeChildren(...)`, `getLoadedOntologyNodeChildren(...)`, and `resolveOntologyNodeChildren(...)`.
- `src/tui/filter-explorer/browser.ts` and `src/tui/filter-explorer/controller-routing.ts` currently mix pure browser projection with lazy resolution. The controller already has the right high-level behavior: show loading, resolve children, then drill. The browser projection should consume session materialized state instead of peeking through ontology nodes.
- `src/tui/filter-explorer/search-draft-model.ts` currently uses sync child access during model assembly. That is invalid for direct-value picker assembly because direct values are a data requirement, not presentation state.
- `src/tui/search-screen/filter-explorer-model-reconciliation.ts` and `src/tui/search-screen/filter-explorer-value-sorting.ts` currently use loaded-child peeking to avoid causing async work during refresh/sort. Their valid future input is a materialized branch snapshot supplied by the explorer/session layer, not a generic ontology-node cache.
- Architecture boundaries still apply: ontology loading remains app-owned, TUI session state remains TUI-owned, and backend/search contracts do not gain terminal-specific concepts.

## Target Model

Use two explicit concepts instead of one overloaded tree helper:

- **Resolvable ontology tree:** immutable-ish `OntologyNode` objects with optional `childSource`. This is the app/domain-facing model. Children are real only after `resolveOntologyNodeChildren(node)` or a targeted branch resolver has completed.
- **Materialized explorer tree:** TUI/session-owned state keyed by node id that records branches intentionally loaded for this explorer session. Render, selection, sorting, and reconciliation read this state, not a hidden global "loaded children" accessor.

Representative TUI state shape, exact names can follow local style:

```ts
type MaterializedOntologyBranchState = {
  readonly childrenByNodeId: ReadonlyMap<string, readonly OntologyNode[]>;
  readonly loadingNodeIds: ReadonlySet<string>;
};
```

Required semantics:

- resolving a node loads only that node's immediate children
- resolving a branch/path loads only the nodes needed to reach that branch
- render/projection functions do not start async loads
- model/option builders that need true children must resolve first
- partial materialized state is never treated as canonical data outside the owning TUI session

## Implementation Changes

- Remove generic sync child access.
  - Delete the `getOntologyNodeChildren(...)` export from `src/app/ontology/node-helpers.ts`.
  - Do not keep a throwing compatibility shim.
  - Do not add a generic `peekLoadedOntologyNodeChildren(...)` replacement.
  - Keep `resolveOntologyNodeChildren(...)` as the app-level async child loader.
  - Keep internal caching behind resolution, but do not expose that cache as a general data API.

- Add explicit materialized branch ownership in the filter explorer.
  - Extend the filter explorer controller/browser state to store materialized children by node id when a branch is resolved.
  - On initial model mount, seed materialized state for static/sync root-level branches only as needed for the current visible level.
  - On drill into a node with unmaterialized children, dispatch loading state, call `resolveOntologyNodeChildren(node)`, store the returned children under that node id, then drill.
  - Browser projection should derive ancestors/current nodes from `model.rootNodes` plus materialized branch state.
  - Back navigation and selection normalization should not consult ontology-node child sources directly.

- Make direct-value model assembly resolve required branches.
  - Convert the search filter explorer model preparation path to async where `singleFieldBehavior: "directValues"` requires value children.
  - Locate the requested field node from the prepared domain, resolve that field node's immediate children, and build the direct-value root nodes from those resolved children.
  - For scoped pack, resolve only the prepared `category[:subcategory]:pack` field node and flatten its pack value children.
  - Apply the same resolved-branch path for rarity and other direct-value fields so callers do not have to know whether the source is static, sync, or lazy.
  - Keep `targetFields` normalization aligned: TUI field `pack` maps to discovery/backend field `packs`; this is boundary cleanup, not the root-cause fix.

- Move loaded-state consumers to explicit snapshots.
  - Refactor reconciliation to accept materialized branch snapshots or a materialized tree view from the session layer.
  - Refactor value sorting to sort the materialized tree/view it is given, without loading additional branches.
  - Any tests that currently call `getOntologyNodeChildren(...)` must either call `resolveOntologyNodeChildren(...)` for real data or inspect explicit materialized test fixtures.

- Preserve and fix related user-visible behavior.
  - Add count-bearing scope category/subcategory prompt labels using app-owned summary data.
  - Subcategories should remain visible from static vocabulary even when summary count is missing; show count `0`.
  - Fix prompt-local styling for `Add Clause`, `Scope`, `Subcategory Mode`, and `Specific Subcategory` so selected prompt chrome does not make those screens appear blue.

## Migration Rules

- Allowed after migration:
  - `resolveOntologyNodeChildren(node)` in app/TUI workflows that need true children.
  - materialized branch state in TUI controllers/session reducers.
  - pure functions that receive a materialized view/snapshot as an argument.

- Not allowed after migration:
  - any import or call of `getOntologyNodeChildren(...)`
  - a generic "loaded child peek" helper exported from `node-helpers.ts`
  - direct-value model assembly from partial materialized UI state
  - recursive whole-tree hydration for scoped pack or ordinary direct-value picking
  - compatibility shims preserving the old sync accessor name

## Test Plan

- Contract tests:
  - no `getOntologyNodeChildren(...)` export remains
  - no source imports of `getOntologyNodeChildren(...)` remain
  - resolving a lazy node loads only that immediate node's children and caches the result
  - resolving/direct-value preparation for one field does not resolve unrelated lazy branches

- Filter explorer session tests:
  - drilling into an unmaterialized lazy node sets loading state, resolves, stores children by node id, and then drills
  - render/projection over materialized state does not call lazy loaders
  - back/selection normalization works from explicit materialized branch state
  - refresh reconciliation preserves selected visible values from materialized branch snapshots without loading unrelated branches
  - value sorting sorts materialized children only

- Prepared explorer tests:
  - scoped `scope=creature` plus pack direct-values shows non-empty pack values
  - scoped pack direct-values resolve only the pack field branch
  - direct-values mode behaves consistently for static, sync, and lazy value children
  - `pack`/`packs` target-field normalization is covered at the prepared discovery boundary

- TUI regression tests:
  - after choosing a creature scope, adding a pack shows actual pack options rather than the empty-state screen
  - scope prompt category and subcategory rows show count-bearing labels
  - affected prompts render with the intended black panel treatment

- Code audit and validation:
  - `rg "getOntologyNodeChildren" src tests` returns no uses
  - `rg "getLoadedOntologyNodeChildren|peekLoadedOntology" src tests` does not reveal a new generic loaded-child accessor
  - `cd scripts && npm run lint`
  - `npm run build`
  - `cd scripts && npm test`
  - land from a dedicated `/tmp` worktree with `scripts/land-worktree.sh`

## Assumptions

- This is a contract-level cleanup, not a pack-specific workaround.
- Partial materialized state belongs only to a TUI session/controller, never to shared ontology/model builders as a data source.
- Any code constructing user-visible option lists, counts, filter models, or result models must resolve the exact branch it needs before presenting data.
- Recursive full-tree hydration is out of scope unless a later caller has a concrete product need for it.
- This touches the TUI/app ontology boundary; implementation should re-check `docs/architecture/tui.md` and `docs/architecture/boundaries.md`, and update architecture docs if the materialized-tree/session boundary becomes durable guidance.
````
