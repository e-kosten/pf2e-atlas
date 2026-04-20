# ADR 0002: Readonly Ontology and Explicit Storage Boundary

- Status: Accepted
- Date: 2026-04-19

## Context

The editorial subsystem needs a browsable ontology model with live record counts and record samples. It also needs workflows that open the SQLite index directly for exploration, migration, and review.

Two failure modes are easy to introduce in this shape:

- shared ontology models become mutable UI state and pick up order-dependent behavior
- direct `DatabaseSync` opening spreads into feature code, making connection lifetime and caching harder to reason about

The current code already addresses both concerns:

- `src/app/ontology-service.ts` builds ontology domains from explicit builders
- `src/app/storage-service.ts` is the app-layer index-opening boundary
- the derived-tag explorer builds a reusable model from the current authored state and live index data
- offline tag/editorial tooling that opens the index directly does so through clearly named helper entrypoints

## Decision

Treat ontology browsing models as readonly published snapshots, and keep database opening behind explicit service boundaries except for justified offline editorial tooling.

In practice this means:

- `src/app/ontology-service.ts` owns ontology domain assembly
- `src/app/storage-service.ts` owns app-layer short-lived index access
- callers consume ontology models instead of mutating shared tag state in place
- tag CLIs and migration helpers may still open the index directly, but only as explicit offline workflow boundaries

## Consequences

- Ontology explorer caching is safer because callers are not expected to mutate shared nodes.
- Storage lifetime decisions remain visible and concentrated.
- The editorial TUI can reuse storage and ontology services without reaching into low-level DB setup.
- When new browsing or workflow code needs index access, the preferred fix is another facade or helper boundary, not ad hoc `DatabaseSync` construction.
