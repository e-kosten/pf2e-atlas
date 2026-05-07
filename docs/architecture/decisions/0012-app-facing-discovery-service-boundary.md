# ADR 0012: Shared App-Facing Discovery Service Boundary

- Status: Accepted
- Date: 2026-04-23

## Context

Discovery behavior for search editing and ontology/search-semantics browsing had become fragmented:

- TUI search code assembled scoped field and value discovery directly in `src/tui/search/`
- ontology/search-semantics loaders performed their own cached filter-value loading and discovery shaping in `src/app/ontology/`
- low-level helpers such as `listFilterValues(...)` were available, but they were backend primitives rather than a cross-surface discovery abstraction

The next search-contract pass introduces shared `Matching` / `Catalog` discovery semantics, pack label-to-name resolution, and broader convergence between TUI pickers and ontology exploration. Those behaviors need one stable owner so surfaces do not reinterpret applicability, ordering, or counts independently.

## Decision

Introduce one shared app-facing discovery service boundary for cross-surface discovery behavior.

That boundary is defined as follows:

- `src/domain/` owns the discovery vocabulary and contracts
  - discovery mode
  - applicability context
  - discovery target identity
  - returned option/count/ordering model
- a new app-facing service under `src/app/` owns cross-surface discovery orchestration
- search/data owners remain responsible for the underlying value/count computation and low-level retrieval primitives
- low-level helpers such as `listFilterValues(...)` remain backend primitives and must not be treated as the durable cross-surface abstraction once the new service exists
- TUI and ontology consumers use the same app-facing discovery boundary
- for TUI callers, that service is wired through `src/tui/app-services.ts` rather than imported ad hoc from feature modules

The shared discovery service is the durable owner for:

- scoped field discovery and applicability rules
- value discovery with counts under shared `Matching` / `Catalog` semantics
- metric-key discovery for shared consumers
- pack label/name resolution before canonical request construction
- shared option ordering and any justified runtime caching

The app-facing implementation keeps those responsibilities split under `src/app/search-discovery/`:

- `service.ts` exposes the stable app facade
- `applicability.ts`, `query-builders.ts`, and `cache-keys.ts` own shared request shape and cache identity helpers
- `metadata-fields.ts`, `metric-discovery.ts`, and `value-discovery.ts` own metadata-field, metric, and value discovery respectively
- `readers.ts` prepares search-semantics discovery readers for ontology and explorer consumers

The service is not the owner for:

- picker shell/container choice
- editor-view mapping
- local dialog draft state
- TUI-only selection helpers
- ontology-only navigation and presentation structure

Ontology search-semantics browsing consumes the discovery service from focused builders under `src/app/ontology/search-semantics/`. That folder owns ontology domain composition, field nodes, metric nodes, pack nodes, value nodes, child sources, query construction, derived-tag nodes, and labels as separate concerns so discovery semantics are not duplicated in presentation assembly.

## Consequences

- TUI and ontology/search-semantics consumers must stop growing separate discovery semantics, caches, and ordering logic where the shared service should own them.
- Cross-surface discovery behavior can evolve once at the app/domain boundary instead of drifting between picker flows and ontology loaders.
- Search/data code keeps low-level retrieval ownership, but app-layer callers should depend on the discovery service rather than reassembling discovery behavior from raw backend helpers.
- Once both surfaces have adopted the new boundary and it is stable enough to be mandatory, lint rules should enforce that path instead of leaving it convention-only.
