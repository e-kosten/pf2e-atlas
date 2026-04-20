# Architecture Decisions

This index is the quickest way to scan the accepted architecture decision records for this repo. Use it when you need to understand the architectural commitments behind the shared runtime, the editorial surface, or the enforced boundaries around derived-tag code.

## Current ADRs

- [`0001-shared-backend-separate-surfaces.md`](./0001-shared-backend-separate-surfaces.md): one shared backend runtime serves both the read-only MCP server and the terminal/editorial surface, with each surface composed at its own boundary.
- [`0002-readonly-ontology-and-explicit-storage-boundary.md`](./0002-readonly-ontology-and-explicit-storage-boundary.md): ontology browsing models are published as readonly snapshots, and app-layer index access stays behind explicit storage helpers.
- [`0003-lint-enforced-derived-tag-boundaries.md`](./0003-lint-enforced-derived-tag-boundaries.md): stable editorial and derived-tag seams should be enforced with lint rules instead of remaining convention-only.

## Reading Order

Read ADR 0001 first for the top-level surface split, then ADR 0002 for ontology and storage boundaries, and ADR 0003 for how those boundaries become mechanically enforced.
