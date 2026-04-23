# Backlog

This is the tracked backlog for open durable future work.

## Status Vocabulary

- `proposed`
- `planned`
- `in_progress`
- `blocked`
- `deferred`

Completed and retired items are tracked separately in [history/done-and-superseded.md](./history/done-and-superseded.md).

## Now

- [Search interaction cleanup](./items/search-interaction-cleanup.md)
  Unify how `/`, query text, and action menus work in the search editor and long selection lists. Status: proposed.

- [View pages and detail presentation](./items/view-pages-and-details.md)
  Make record detail views easier to scan, more dynamic by record type, and better suited for linked navigation. Status: proposed.

- [Remove isUnique metadata](./items/remove-isunique-metadata.md)
  Stop exposing `isUnique` as standalone metadata where it only duplicates rarity semantics. Status: proposed.

## Soon

- [Derived-tag concept model implementation](./items/derived-tag-concept-model-implementation.md)
  Move the preserved canonical concept/projection design out of scratch planning and into tracked `src/tags/` ownership, including source-of-truth schema, projection metadata, and canonical concept relations. Status: proposed.

- [Search semantics explorer completeness](./items/search-semantics-explorer-completeness.md)
  Most explorer depth is already landed; the remaining gap is richer numeric metric corpus exploration without breaking the current ontology/search ownership split. Status: proposed.

- [Tagging tooling reorganization](./items/tagging-tooling-reorg.md)
  The command-surface reorganization is mostly landed; remaining work is naming and discoverability cleanup so `tui` and workbench terminology match the broader terminal app. Status: proposed.

- [Filter shape convergence](./items/filter-shape-convergence.md)
  Bring MCP and TUI filter modeling back into alignment, especially around rarity and level no longer being treated as special cases. Status: proposed.

## Later

- [Shared result grouping and presentation modes](./items/shared-result-grouping-and-presentation-modes.md)
  Explore shared grouping keys and user-cyclable presentation modes for list/detail search and explorer surfaces without regressing back into per-screen experiments. Status: deferred.

- [Category relevance script](./items/category-relevance-script.md)
  Add tooling to help tagging work happen in coherent batches without forcing one agent or reviewer to keep an entire family/tag space in active memory. Status: proposed.

- [Derived tag manifest tooling metadata](./items/derived-tag-manifest-tooling-metadata.md)
  Preserve the idea that the shared derived-tag manifest could optionally carry current owner-file metadata for tooling, while leaving open whether that is actually valuable once the future concept/projection model settles. Status: proposed.

- `Metadata predicate typing cleanup`
  Preserve the follow-up idea from the retiring `fix/metadata-filter-spec-consolidation` worktree: the shared `metadata-predicate-spec` architecture is already landed, but predicate TypeScript types may still be simplifiable so operator unions and payload shapes derive more directly from that shared spec instead of relying on more duplicated or manually expanded type definitions across domain, filter, and schema layers. Status: proposed.

- [Typed seams cleanup](./items/typed-seams-cleanup.md)
  Continue focused type-safety work around metadata registry access, prompt result narrowing, matcher adapters, and test fixtures. Status: proposed.

- [Shared UI model boundary enforcement](./items/shared-ui-model-boundary-enforcement.md)
  Identify which reusable TUI state and view-model helpers should become mandatory and enforce them once those pathways are stable enough. Status: proposed.

- [Shared menu/editor behavior contracts](./items/shared-menu-editor-behavior-contracts.md)
  Track the higher-level editor/workspace interaction leak where shared mechanics exist but behavior policy still lives partly in feature-local code. Status: proposed.

- [Shared TUI interaction family contracts](./items/shared-tui-interaction-family-contracts.md)
  Capture the intended higher-level interaction families and the split between shared behavior contracts and feature-local workflow ownership across list/detail, menu/editor/workspace, and command/action-target surfaces. Status: proposed.

- [Derived-tag assignments layout](./items/derived-tag-assignments-layout.md)
  Decide a durable on-disk structure for authored assignments before the assignment corpus grows much larger. Status: proposed. Deferred for now, don't prioritize.

See [Backlog Done / Superseded](./history/done-and-superseded.md) for completed and retired items.
