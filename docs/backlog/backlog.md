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

- Verify what (if anything) still uses the bottom aligned modal. Because it's ugly and should probably get deleted or strongly changed. Same for the command pallete because the action picker is probably better.

## Soon

- [Derived-tag index service layer](./items/derived-tag-index-service-layer.md)
  Replace the raw `openIndex()` / `DatabaseSync` escape hatch in editorial, discovery, evaluation, and derived-tag ontology cache flows with a real shared derived-tag backend service family. Status: proposed.

- [Derived-tag concept model implementation](./items/derived-tag-concept-model-implementation.md)
  Move the preserved canonical concept/projection design out of scratch planning and into tracked `src/tags/` ownership, including source-of-truth schema, projection metadata, and canonical concept relations. Status: proposed.

- [Search semantics explorer completeness](./items/search-semantics-explorer-completeness.md)
  Most explorer depth is already landed; the remaining gap is richer numeric metric corpus exploration without breaking the current ontology/search ownership split. Status: proposed.

- [Search semantics surface convergence](./items/search-semantics-surface-convergence.md)
  Preserve the architectural opportunity to converge ontology/search semantics internals and mode contracts without collapsing the distinct broad-browse and prepared-query surfaces. Status: proposed.

- [Structured editor metric availability ownership](./items/structured-editor-metric-availability-ownership.md)
  Clarify whether metric-family availability filtering belongs in prompt-local filtering, the route catalog, or the shared metric explorer owner. Status: proposed.

- [Tagging tooling reorganization](./items/tagging-tooling-reorg.md)
  The command-surface reorganization is mostly landed; remaining work is naming and discoverability cleanup so `tui` and workbench terminology match the broader terminal app. Status: proposed.

- Move bottom aligned "modals" into the floating modal model. Catalog the current uses, and decide wether things should stay as they are (ex. the action palette is a good candidate to stay where it is) where as other stuff like level pickers and the exemplar filter menus should be floating. The action menu though is a good candidate to stay in the footer.

- Extend the action palette with "quick select". Basically once in the action menu vim motion shoudl still navigate, but each option should get a home-row friendly key binding option for select. It should be consistent per page. Need to think how this maps into our shared key binding rules. Could be that `: + <keys>` is considered special and allowed to use a predefined set of options that are fine to use at the same time as `:`. Ex. h/j/k/l, q, and esc shouldn't be allowed, but many other are find in this specific context. But the "fine" list should still be shared.

## Later

- [Casting time filter gap](./items/casting-time-filter-gap.md)
  Track the missing searchable concept for non-action casting times such as `1 minute` or `10 minutes`. Status: proposed.

- [Query editor shortcuts and simplification](./items/query-editor-shortcuts-and-simplification.md)
  Track follow-up shortcut and simplification affordances after the verbose canonical query-tree editor lands and real friction points are visible. Status: proposed.

- [Reference page navigation history](./items/reference-page-navigation-history.md)
  Make concrete record-reference navigation open full-screen entity pages with a usable back stack instead of incidental one-result search tabs. Status: proposed.

- [Search page navigation shaping](./items/search-page-navigation-shaping.md)
  Shape the intended search-page navigation model before making deeper changes to an interaction flow that works but does not yet feel right. Status: proposed.

- [LinkedFrom query editor follow-through](./items/linked-from-query-editor-follow-through.md)
  Preserve the optional query-editor and TUI presentation follow-through for `linked_from` after the canonical contract work lands for entity-page backlinks. Status: proposed.

- [Reference edge extraction expansion](./items/reference-edge-extraction-expansion.md)
  Preserve the follow-up to widen `reference_edges` beyond the current UUID-centered extraction path without blocking the active entity-page work. Status: proposed.

- [Shared result grouping and presentation modes](./items/shared-result-grouping-and-presentation-modes.md)
  Explore shared grouping keys and user-cyclable presentation modes for list/detail search and explorer surfaces without regressing back into per-screen experiments. Status: deferred.

- [Category relevance script](./items/category-relevance-script.md)
  Add tooling to help tagging work happen in coherent batches without forcing one agent or reviewer to keep an entire family/tag space in active memory. Status: proposed.

- [Derived tag manifest tooling metadata](./items/derived-tag-manifest-tooling-metadata.md)
  Preserve the idea that the shared derived-tag manifest could optionally carry current owner-file metadata for tooling, while leaving open whether that is actually valuable once the future concept/projection model settles. Status: proposed.

- [Rust CLI runtime migration research](./rust-cli-runtime/README.md)
  Design-in-progress migration roadmap and decision spikes for a Rust-centered local runtime, CLI plus skills as the primary agent surface, improved retrieval quality, offline prep boundaries, and optional MCP compatibility. Status: in_progress.

- [Rust content subdocuments for journal pages and table results](./items/rust-content-subdocuments-journal-table-results.md)
  Preserve the deferred design question for journal-page and rollable-table-result rich text as child content rather than flattening broad parent records during the Rust content-document refactor. Status: deferred.

- [Rust CLI content output formats](./items/rust-cli-content-output-formats.md)
  Decide which optional non-markdown content formats the Rust CLI should expose for record JSON output after the Phase 5 markdown projection lands. Status: deferred.

- [Rust Foundry JSON field audit](./items/rust-foundry-json-field-audit.md)
  Add an explicit offline audit that inventories Foundry source JSON fields against Rust ingest coverage so meaningful uncaptured source data can be found without adding partial raw-JSON scans to default ingest. Status: proposed.

- [Rust side data and metric source fact convergence](./items/rust-side-data-metric-source-fact-convergence.md)
  Track the later cleanup for overlapping Rust side-table and metric projections so shared source facts/specs interpret speed, sense, range, disable, and similar source families once while preserving distinct runtime storage roles. Status: proposed.

- [Rust search quality and retrieval weight tuning](./items/rust-search-quality-tuning.md)
  Tune Rust FTS weights, RRF windows, rank constants, and weighted-fusion defaults after the Phase 6 retrieval baseline is measurable. Status: proposed.

- [Rust FTS tokenization and stemming exploration](./items/rust-fts-tokenization-stemming.md)
  Evaluate SQLite FTS tokenizer/stemming choices for inflection handling after the Phase 6 retrieval baseline lands, without adding brittle custom synonym maps. Status: proposed.

- `Metadata predicate typing cleanup`
  Preserve the follow-up idea from the retiring `fix/metadata-filter-spec-consolidation` worktree: the shared `metadata-predicate-spec` architecture is already landed, but predicate TypeScript types may still be simplifiable so operator unions and payload shapes derive more directly from that shared spec instead of relying on more duplicated or manually expanded type definitions across domain, filter, and schema layers. Status: proposed.

- [Typed seams cleanup](./items/typed-seams-cleanup.md)
  Continue focused type-safety work around metadata field catalog access, prompt result narrowing, matcher adapters, and test fixtures. Status: proposed.

- [Shared UI model boundary enforcement](./items/shared-ui-model-boundary-enforcement.md)
  Identify which reusable TUI state and view-model helpers should become mandatory and enforce them once those pathways are stable enough. Status: proposed.

- [Shared contract string vocabulary enforcement](./items/shared-contract-string-vocabulary-enforcement.md)
  Centralize durable shared contract strings such as prompt discriminants, operator ids, and shared action vocabulary, then add targeted lint enforcement once those seams are stable. Status: proposed.

- [Shared menu/editor behavior contracts](./items/shared-menu-editor-behavior-contracts.md)
  Track the higher-level editor/workspace interaction leak where shared mechanics exist but behavior policy still lives partly in feature-local code. Status: proposed.

- [Shared TUI interaction family contracts](./items/shared-tui-interaction-family-contracts.md)
  Capture the intended higher-level interaction families and the split between shared behavior contracts and feature-local workflow ownership across list/detail, menu/editor/workspace, and command/action-target surfaces. Status: proposed.

- [Search result list viewport behavior](./items/search-result-list-viewport-behavior.md)
  Revisit whether the left search result pane should stay cursor-driven or gain an independently scrollable viewport. Status: proposed.

- [Pointer target hit testing](./items/pointer-target-hit-testing.md)
  Add shared pointer target hit testing for list rows, page sections, and section-local targets after pane-level pointer routing lands. Status: proposed.

- [Mouse text selection for copy/paste](./items/mouse-text-selection-copy-paste.md)
  Add mouse-based text selection for copy/paste workflows with explicit pane, modal, footer, and structured-region boundaries. Status: proposed.

- [Derived-tag assignments layout](./items/derived-tag-assignments-layout.md)
  Decide a durable on-disk structure for authored assignments before the assignment corpus grows much larger. Status: proposed. Deferred for now, don't prioritize.

- Evaluate the SQL data model now that the search infrastructure, filter shape, etc. have evolved

See [Backlog Done / Superseded](./history/done-and-superseded.md) for completed and retired items.
