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

- `Ontology browser naming friendliness`
  Replace internal-facing labels such as `derivedTags` and other machine-shaped names with natural casing and wording in the explorer UI. Status: proposed.

- `Search result readability cleanup`
  Tighten list and breadcrumb formatting where result headers and long labels are hard to scan in the current TUI layout. Status: proposed.

- `Right-navigation feedback`
  When rightward navigation cannot drill deeper, show a small transient message instead of silently behaving like a layout/focus toggle. Status: proposed.

- `Remove isUnique metadata`
  Stop exposing `isUnique` as standalone metadata where it only duplicates rarity semantics. Status: proposed.

## Soon

- [Derived-tag assignments layout](./items/derived-tag-assignments-layout.md)
  Decide a durable on-disk structure for authored assignments before the assignment corpus grows much larger. Status: proposed.

- [Derived-tag concept model implementation](./items/derived-tag-concept-model-implementation.md)
  Move the preserved canonical concept/projection design out of scratch planning and into tracked `src/tags/` ownership, including source-of-truth schema, projection metadata, and canonical concept relations. Status: proposed.

- [Structured query summary model](./items/structured-query-summary-model.md)
  Build the missing intermediate summary/document layer for the live search workspace now that canonical `query.filters.parts` ownership is already landed. Status: proposed.

- [Search semantics explorer completeness](./items/search-semantics-explorer-completeness.md)
  Most explorer depth is already landed; the remaining gap is richer numeric metric corpus exploration without breaking the current ontology/search ownership split. Status: proposed.

- [Tagging tooling reorganization](./items/tagging-tooling-reorg.md)
  The command-surface reorganization is mostly landed; remaining work is naming and discoverability cleanup so `tui` and workbench terminology match the broader terminal app. Status: proposed.

- `Incomplete search tail in nested explorer lists`
  Investigate why some deep record lists, such as `families > ghost`, advertise more results than the stepped-in detail view actually shows. Status: proposed.

- `Filter shape convergence`
  Bring MCP and TUI filter modeling back into alignment, especially around rarity and level no longer being treated as special cases. Status: proposed.

## Later

- [Category relevance script](./items/category-relevance-script.md)
  Add tooling to help tagging work happen in coherent batches without forcing one agent or reviewer to keep an entire family/tag space in active memory. Status: proposed.

- [Derived tag manifest tooling metadata](./items/derived-tag-manifest-tooling-metadata.md)
  Preserve the idea that the shared derived-tag manifest could optionally carry current owner-file metadata for tooling, while leaving open whether that is actually valuable once the future concept/projection model settles. Status: proposed.

- `Metadata predicate typing cleanup`
  Preserve the follow-up idea from the retiring `fix/metadata-filter-spec-consolidation` worktree: the shared `metadata-predicate-spec` architecture is already landed, but predicate TypeScript types may still be simplifiable so operator unions and payload shapes derive more directly from that shared spec instead of relying on more duplicated or manually expanded type definitions across domain, filter, and schema layers. Status: proposed.

- `Typed seams cleanup`
  Continue focused type-safety work around metadata registry access, prompt result narrowing, matcher adapters, and test fixtures. Status: proposed.

- `Shared UI model boundary enforcement`
  Identify which reusable TUI state/view-model helpers should become mandatory and enforce them once those pathways are stable enough. Status: proposed.

See [Backlog Done / Superseded](./history/done-and-superseded.md) for completed and retired items.
