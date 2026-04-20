# ADR 0001: Shared Backend, Separate MCP and Editorial Surfaces

- Status: Accepted
- Date: 2026-04-19

## Context

The repository serves two different user-facing surfaces:

- a read-only MCP server for lookup, search, and rules questions
- a terminal/editorial surface for ontology browsing and derived-tag maintenance

Both surfaces need the same prepared SQLite index, normalized record model, and backend services. The editorial workflows also need additional storage, ontology, and migration composition that the MCP server should not depend on.

If each surface owned its own backend stack, retrieval behavior would diverge and indexing or ranking changes would have to be reimplemented twice.

## Decision

Keep one shared backend and compose distinct surfaces on top of it:

- `src/data/` and `src/search/` own the reusable backend runtime
- `src/app/runtime.ts` composes the shared application runtime
- `src/index.ts` and `src/server/` stay thin MCP registration and presentation layers
- `src/tui/app-services.ts` composes the terminal/editorial surface by adding ontology, storage, and tag-workbench services

The editorial workflows remain first-class, but they are layered over the same backend runtime rather than implementing a separate stack.

## Consequences

- Search, lookup, and record-access behavior stays centralized.
- MCP handlers remain read-only transport code rather than accumulating editorial state concerns.
- The TUI can add editorial workflows without forcing terminal-specific dependencies into the MCP server.
- Derived-tag architecture documentation should treat the workbench as a surface over shared services, not as an independent backend.
