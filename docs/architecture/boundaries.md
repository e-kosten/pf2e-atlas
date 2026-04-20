# Architectural Boundaries

This document records the most important boundaries in the current codebase. Some are enforced directly by ESLint rules; others are design rules that should be preserved even when they are not yet mechanically enforced everywhere.

The main principle is simple: shared abstractions should become the normal path through the system, and once they are stable enough they should be enforced instead of left as convention-only.

## Composition Roots

There are two primary composition roots:

- `src/index.ts` for the MCP server
- `src/tui/app-services.ts` for the terminal app

Feature modules should not recreate runtime assembly on their own. If a new surface needs access to backend services, add it to the appropriate composition root or to a shared app-layer service.

## Storage Boundary

Direct `DatabaseSync` construction is intentionally constrained.

The intended split is:

- `src/data/` owns the long-lived backend database runtime through `Pf2eDataService`
- `src/app/storage-service.ts` owns app-layer short-lived index access for workflows that need direct DB access
- tests and offline CLI/editorial tooling may still have their own justified entrypoints

Feature modules in `src/app/`, `src/tui/`, `src/server/`, and most of the rest of `src/` should not open SQLite connections directly. They should depend on `Pf2eDataService`, `Pf2eApplicationStorageService`, or another approved facade.

Why this exists:

- storage-opening behavior stays easy to change
- connection lifetime decisions stay centralized
- UI and server code do not grow ad hoc SQL access paths

## Search Boundary

Search execution should flow through the backend service and shared runtime search modules rather than through one-off ranking paths.

The intended path is:

1. build or normalize `SearchFilters`
2. call `Pf2eDataService`
3. let `Pf2eSearchBackendService` and `src/search/runtime-search.ts` handle ranked execution

Server handlers and TUI features should not reach into low-level SQL helpers to assemble their own competing search stack.

Related enforced rule:

- server tool registration must not import low-level SQL/query internals directly
- search modules must not bypass the higher-level backend/storage facades by reaching directly into storage leaf modules when a facade should own the behavior

## Ontology Boundary

Ontology browsing is an app-layer concern assembled in `src/app/ontology-service.ts` from focused domain builders in `src/app/ontology/`.

Important expectations:

- ontology domains are loaded through `createPf2eApplicationOntologyService`
- ontology nodes are treated as readonly browsing models
- UI code should not mutate shared ontology node graphs in place
- helper caches should live alongside ontology helpers, not as hidden UI-side mutations of shared nodes

The readonly contract matters because ontology models are shared across browsing flows. Once callers start mutating them, caching and reuse become fragile and order-dependent.

## TUI Boundary

The terminal app should consume explicit service facades instead of reaching directly into random internals.

Examples of intended facades:

- `src/tui/search-service.ts` for TUI search behavior
- `src/app/ontology-service.ts` for ontology browsing
- `src/tags/migration/workbench-controller.ts` and related workbench services for tag-review workflows

The TUI also has a second boundary inside itself:

- `src/tui/framework/` and a few shared helpers own low-level terminal runtime mechanics
- feature screens and workflows should use those helpers instead of importing raw terminal primitives directly

This keeps terminal interaction behavior consistent and limits how much of the app depends on framework details.

## Domain Boundary

`src/domain/` should stay transport-agnostic and UI-agnostic.

It is the right place for:

- stable type definitions
- category/subcategory vocabularies
- metadata field and predicate semantics
- ontology contracts

It is not the right place for:

- MCP-specific presentation logic
- TUI workflows
- storage lifecycle management

When in doubt, ask whether the code defines the business/domain vocabulary of the system or whether it performs application behavior. If it performs behavior, it probably belongs above `src/domain/`.

## Tag Boundary

Outside `src/tags/`, tag functionality should normally be consumed through the tag facade rather than by importing arbitrary tag leaf modules.

Why this exists:

- the tag/editorial subsystem is large and still evolving
- a facade lets callers use stable entrypoints while the internal structure continues to change
- it reduces cross-cutting coupling from non-tag code into editorial internals

The detailed tag tree is intentionally documented elsewhere. This document only captures the architectural rule that the rest of the app should not take dependencies on tag internals casually.

## Lint-Enforced Boundaries

`eslint.config.js` currently encodes several architectural rules. The most important categories are:

- no direct `JSON.parse` outside approved decoder boundaries
- no direct `DatabaseSync` construction outside approved entry modules
- no direct low-level terminal event routing from TUI feature code
- no direct low-level storage/query imports from server registration modules
- no direct derived-tag leaf imports outside approved tag-facing entrypoints
- terminology and prompt-routing restrictions around the TUI search editor

These rules are not just style preferences. They encode architecture that already proved easy to regress when left implicit.

## When To Add A New Boundary Rule

Add or extend a lint rule when all of the following are true:

- there is now a preferred shared abstraction
- bypassing it would create duplication or architectural drift
- the abstraction is stable enough that future code should not treat direct access as acceptable

Do not add lint rules prematurely for abstractions that are still exploratory. But once a path is clearly the long-term boundary, prefer enforcement over repeated review comments.

## Editing Guidance

Before adding a new module or moving code across layers:

1. identify the owning layer
2. check whether a facade already exists
3. route through that facade if it does
4. if it does not, decide whether the right fix is a new facade or a local helper
5. if the new path is meant to be mandatory, update lint rules and tests accordingly

That discipline is what keeps the project from regressing back into large flat folders and inconsistent abstractions.
