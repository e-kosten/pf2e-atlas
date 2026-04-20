# Architecture Decisions

This index is the quickest way to scan the accepted architecture decision records for this repo. Use it when you need to understand the architectural commitments behind the shared runtime, the editorial surface, or the enforced boundaries around derived-tag code.

## Current ADRs

- [`0001-shared-backend-separate-surfaces.md`](./0001-shared-backend-separate-surfaces.md): one shared backend runtime serves both the read-only MCP server and the terminal/editorial surface, with each surface composed at its own boundary.
- [`0002-readonly-ontology-and-explicit-storage-boundary.md`](./0002-readonly-ontology-and-explicit-storage-boundary.md): ontology browsing models are published as readonly snapshots, and app-layer index access stays behind explicit storage helpers.
- [`0003-lint-enforced-derived-tag-boundaries.md`](./0003-lint-enforced-derived-tag-boundaries.md): stable derived-tag seams, including the split runtime/reviews/editorial owners, should be enforced with lint rules instead of remaining convention-only.
- [`0004-non-tag-ownership-import-boundaries.md`](./0004-non-tag-ownership-import-boundaries.md): non-tag code should import domain and helper modules from their owning paths instead of broad shared barrels or compatibility aliases.
- [`0005-live-search-semantics-exploration.md`](./0005-live-search-semantics-exploration.md): search semantics, ontology inspection, and query-field picking should share one live exploration surface that opens real results from concrete leaves.
- [`0006-shared-tui-interaction-contracts.md`](./0006-shared-tui-interaction-contracts.md): TUI screens should route interactions, action-target behavior, and help/footer derivation through shared contracts instead of bespoke feature-local handling.

## Reading Order

Read ADR 0001 first for the top-level surface split, then ADR 0002 for ontology and storage boundaries, ADR 0003 for tag/editorial enforcement, ADR 0004 for the non-tag ownership import rules that keep broad shared pathways from regrowing, ADR 0005 for the converged search-semantics exploration model, and ADR 0006 for the shared TUI interaction contract.
