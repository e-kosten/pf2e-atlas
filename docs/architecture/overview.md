# Architecture Overview

This document describes the current architectural shape of the project. It is intended to help future human and AI editors understand where behavior lives, which modules own which responsibilities, and which seams are intended to stay stable as the codebase changes.

## Goals

The repository currently serves two related but distinct purposes:

- a read-only MCP server over a prepared local PF2E SQLite index
- a terminal/editorial toolset for search exploration and derived-tag maintenance

Those two surfaces share the same normalized data model and backend services, but they do not share the same presentation layer. The main architectural goal is to keep retrieval, storage, and domain logic reusable while letting the MCP server and TUI evolve independently.

## Top-Level Module Map

### `src/index.ts`

The stdio server entrypoint. It loads the application runtime and registers the MCP tool surfaces from `src/server/`.

### `src/app/`

Application composition and app-level services.

- `runtime.ts` loads configuration, ranking config, and the shared `Pf2eDataService`.
- `storage-service.ts` is the app-layer storage boundary for index-backed helper workflows that need direct `DatabaseSync` access.
- `ontology-service.ts` assembles the ontology browsing domains exposed to the TUI.
- `ontology/` contains the domain builders and helper modules for ontology browsing.

This layer exists so higher-level surfaces do not need to know how to compose storage, search vocabulary, ontology models, or runtime wiring themselves.

### `src/data/`

Index-backed backend logic and normalized record access.

- `service.ts` exposes `Pf2eDataService`, the main backend facade used by the server and TUI.
- `backend/` contains the lower-level catalog, search, and rule-graph services used by `Pf2eDataService`.
- the rest of the folder contains normalization, SQL decoding, schema, indexing, and shared row/query helpers.

This is the main boundary between the rest of the application and low-level SQLite-backed record access.

### `src/search/`

Shared ranked-search runtime and scoring logic.

- query analysis and normalization
- SQL-side filter coordination
- ranking and fusion logic
- runtime search execution used by backend search flows

This layer holds search mechanics that should stay reusable across server and TUI use cases rather than leaking into transport-specific handlers.

### `src/server/`

MCP tool registration and response presentation.

- `register-search-tools.ts`
- `register-lookup-tools.ts`
- `register-rule-tools.ts`
- `presenters.ts`
- `tool-schemas.ts`

This layer should remain thin. It should translate MCP inputs into calls on `Pf2eDataService` and shape outputs for the wire protocol, not own indexing or ranking logic itself.

### `src/tui/`

Terminal application composition and workflows.

- `app-services.ts` assembles the user-facing and dev-facing service bundle for the terminal app.
- `search-service.ts` is the TUI-facing facade over the split search submodules in `src/tui/search/`.
- `ontology-explorer/` contains ontology navigation UI logic.
- `framework/` contains lower-level terminal framework helpers.

The TUI layer should consume application or backend services through explicit facades instead of reaching directly into low-level storage/query internals.

### `src/domain/`

Shared contracts and domain vocabularies.

- categories and subcategories
- ontology node types
- metadata semantics and predicate specs
- record/search/rule type definitions

This is the lowest-level shared layer. It should stay dependency-light and free of transport or UI concerns.

### `src/tags/`

Derived-tag authoring and editorial tooling.

This area includes runtime assignments, authored rules, discovery tools, migration flows, evaluation utilities, and review workflows. It is large because it supports an internal editorial workflow in addition to the public MCP retrieval surface.

The tag tree itself is intentionally out of scope for this document. What matters architecturally is that non-tag code should normally reach tag functionality through the tag facade, not through arbitrary leaf internals.

## Runtime Composition

There are two main composition roots.

### MCP Server Runtime

The server boot path is:

1. `src/index.ts`
2. `src/app/runtime.ts`
3. `Pf2eDataService.load(...)`
4. MCP tool registration from `src/server/`

At startup, the runtime loads configuration, opens the prepared SQLite-backed data runtime, and creates a long-lived `Pf2eDataService`. Tool registration is layered on top of that backend service.

### TUI Runtime

The TUI boot path goes through:

1. `src/tui/app-services.ts`
2. `src/app/runtime.ts`
3. `src/app/storage-service.ts`
4. `src/app/ontology-service.ts`
5. `src/tui/search-service.ts` and the relevant workflows/screens

The TUI reuses the same backend runtime, then adds user-facing services:

- ontology browsing
- terminal search session orchestration
- derived-tag workbench flows

That split is deliberate. The TUI should be able to evolve without forcing the MCP server entrypoint to absorb terminal-specific concerns.

## Core Service Hierarchy

The important service stack looks like this:

1. `src/domain/`: contracts and stable vocabularies
2. `src/data/`: normalized data access and backend services
3. `src/search/`: shared ranking/query execution machinery
4. `src/app/`: application composition and app-level facades
5. `src/server/` and `src/tui/`: transport and UI surfaces

When adding new behavior, prefer to put it in the lowest layer that can own it without depending on a higher-level concern.

Examples:

- new MCP-only presentation logic belongs in `src/server/`
- reusable search execution logic belongs in `src/search/` or `src/data/backend/`
- ontology assembly belongs in `src/app/ontology/`
- shared search/category type definitions belong in `src/domain/`

## Important Data Flows

### Search

The current search path is intentionally centralized:

1. callers build `SearchFilters`
2. `Pf2eDataService` forwards to `Pf2eSearchBackendService`
3. backend search uses the shared runtime search logic in `src/search/runtime-search.ts`
4. ranked or browse results flow back through either MCP presenters or TUI adapters

The important point is that ranked runtime execution is no longer supposed to splinter into separate ad hoc paths for different callers.

### Ontology Browsing

Ontology browsing is assembled in `src/app/ontology-service.ts` from three domains:

- derived tags
- catalog categories
- search semantics

Those builders live under `src/app/ontology/`. They return readonly ontology node models intended for browsing, not mutable shared state for UI features to rewrite in place.

### Storage Access

Long-lived backend storage is owned by `Pf2eDataService`. Shorter-lived app workflows that need direct SQLite access go through `src/app/storage-service.ts`.

This keeps direct `DatabaseSync` construction from spreading through feature code and gives the codebase a smaller number of places where storage-opening behavior can change.

## Design Intent Behind Recent Refactors

Several recent changes established architectural direction that future work should continue:

- application storage access was centralized behind `src/app/storage-service.ts`
- ontology service logic was decomposed into smaller domain modules under `src/app/ontology/`
- TUI search service logic was split into focused helper modules under `src/tui/search/`
- ontology browsing contracts were tightened to readonly models instead of mutable shared node graphs
- lint rules were expanded so shared abstractions are enforceable boundaries rather than conventions

These changes matter more than their exact file names. The architectural intent is:

- smaller focused modules instead of large multi-purpose service files
- explicit facades between layers
- readonly/shared models when callers should not mutate them
- lint-enforced boundaries where direct access would otherwise regress the structure

## How To Navigate The Codebase

For a new editor, the fastest way to understand the codebase is usually:

1. read `README.md`
2. read this document and `docs/architecture/boundaries.md`
3. inspect the relevant composition root:
   - `src/index.ts` for MCP work
   - `src/tui/app-services.ts` for TUI work
4. find the service facade for the feature area:
   - `src/data/service.ts`
   - `src/app/ontology-service.ts`
   - `src/tui/search-service.ts`
5. only then drill into backend/helper modules

That order keeps editors aligned with the intended layering instead of starting from leaf files and reconstructing the architecture from accident.

## Validation Expectations

The repository standard validation gate is:

```bash
npm run verify
```

That runs lint, format check, build, test typecheck, and the Vitest suite. Architectural changes should also update or add lint rules when a new shared boundary is meant to become mandatory rather than optional.
