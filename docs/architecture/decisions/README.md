# Architecture Decisions

This index is the quickest way to scan the accepted architecture decision records for this repo. Use it when you need to understand the architectural commitments behind the shared runtime, the editorial surface, or the enforced boundaries around derived-tag code.

## Current ADRs

- [`0001-shared-backend-separate-surfaces.md`](./0001-shared-backend-separate-surfaces.md): one shared backend runtime serves both the read-only MCP server and the terminal/editorial surface, with each surface composed at its own boundary.
- [`0002-readonly-ontology-and-explicit-storage-boundary.md`](./0002-readonly-ontology-and-explicit-storage-boundary.md): ontology browsing models are published as readonly snapshots, and app-layer index access stays behind explicit storage helpers.
- [`0003-lint-enforced-derived-tag-boundaries.md`](./0003-lint-enforced-derived-tag-boundaries.md): stable derived-tag seams, including the split runtime/reviews/editorial owners, should be enforced with lint rules instead of remaining convention-only.
- [`0004-non-tag-ownership-import-boundaries.md`](./0004-non-tag-ownership-import-boundaries.md): non-tag code should import domain and helper modules from their owning paths instead of broad shared barrels or compatibility aliases.
- [`0005-live-search-semantics-exploration.md`](./0005-live-search-semantics-exploration.md): search semantics, ontology inspection, and query-field picking should share one live exploration surface that opens real results from concrete leaves.
- [`0006-shared-tui-interaction-contracts.md`](./0006-shared-tui-interaction-contracts.md): TUI screens should route interactions, action-target behavior, and help/footer derivation through shared contracts instead of bespoke feature-local handling.
- [`0007-render-ready-route-transitions.md`](./0007-render-ready-route-transitions.md): route screens must mount from render-ready payloads while navigation owns any preparation work and shows the shared current-screen transition loader.
- [`0008-search-filters-and-concern-specific-tag-facades.md`](./0008-search-filters-and-concern-specific-tag-facades.md): live search-filter ownership lives in `src/search/filters/`, and non-tag tag integrations go through concern-specific top-level facades.
- [`0009-shared-list-detail-presentation-layer.md`](./0009-shared-list-detail-presentation-layer.md): list/detail TUI screens should share one presentation layer for pane measurement, screen-model assembly, and routing setup while keeping domain workflows feature-owned.
- [`0010-shared-search-request-contract.md`](./0010-shared-search-request-contract.md): `SearchRequest` is the shared semantic search contract, while search-execution filters stay behind the backend/search compilation boundary.

## Reading Order

Read ADR 0001 first for the top-level surface split, then ADR 0002 for ontology and storage boundaries, ADR 0003 for tag/editorial enforcement, ADR 0004 for the non-tag ownership import rules that keep broad shared pathways from regrowing, ADR 0005 for the converged search-semantics exploration model, ADR 0006 for the shared TUI interaction contract, ADR 0007 for render-ready route preparation and navigation-owned transition loading, ADR 0009 for the shared list/detail presentation layer that sits above those interaction primitives, and ADR 0010 for the shared semantic search contract that now sits ahead of search execution.
