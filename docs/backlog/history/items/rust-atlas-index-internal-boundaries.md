# Rust Atlas Index Internal Boundaries

Status: completed
Completed: 2026-06-01

## Problem

`atlas-index` now correctly owns the SQLite artifact boundary: Diesel migrations and schema, artifact validation, artifact writing, read-only artifact access, filter discovery, filter compilation, graph/reference SQL, FTS SQL, and vector SQL.

Those responsibilities belong together in one crate, but the current internal layout is still too flat. Several top-level modules act as implicit boundaries, and `SqliteIndexReader` / `SqliteIndexWriter` risk becoming broad collection points unless the crate is reshaped around artifact, read, write, and SQLite implementation facades.

## Outcome

`atlas-index` was reshaped into a storage/query engine crate with:

- a small public crate API used by `atlas-ingest`, `atlas-runtime`, `atlas-search`, and `atlas-cli`
- explicit internal module facades for artifact contract, read/query behavior, write/build behavior, and SQLite mechanics
- `SqliteIndexReader` and `SqliteIndexWriter` kept as lifecycle/orchestration shells rather than SQL or phase-logic bags
- a single source of truth for filter discovery definitions and extractor rendering
- no compatibility modules, forwarding shims, or mixed old/new paths left behind

## Constraints

- Keep all current `atlas-index` responsibilities in this crate; do not split new crates out as part of this work.
- Keep shared request/result vocabulary in `atlas-domain`.
- Keep normalized records and presentation/content policy in `atlas-record`.
- Keep Diesel migrations as the physical schema source of truth.
- Keep catalog-backed reads Diesel-native where they fit.
- Keep dynamic filter/discovery, FTS5, sqlite-vec, and validation pragmas behind index-owned raw SQL helpers.
- Do not duplicate discovery field definitions under SQLite writer internals.

## Notes

The crate now groups artifact contract/validation under `artifact`, read-side records/search/discovery/graph behavior under `read`, writer input and SQLite artifact creation under `write`, and connection-level SQLite mechanics under `sqlite`.

Diesel migrations remain the physical schema source of truth. Raw SQL remains owned by `atlas-index` for FTS5, sqlite-vec, dynamic filter/discovery relations, and validation pragmas.

## Related

- [Architecture overview](../../architecture/overview.md)
- [Runtime architecture](../../architecture/runtime.md)
- [Artifact contract](../../architecture/artifact-contract.md)
- [Rust artifact crate boundary review](../history/items/rust-artifact-crate-boundary-review.md)
