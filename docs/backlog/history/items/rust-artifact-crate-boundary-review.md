# Rust Artifact Crate Boundary Review

Status: completed
Completed: 2026-05-31

## Outcome

`atlas-artifact` was folded into `atlas-index` and retired as a workspace crate.

`atlas-index` now owns the SQLite artifact boundary: Diesel migrations and schema declarations for ordinary relational tables, artifact metadata constants, narrow validation requirements for external artifact files, SQLite vector blob encoding, sqlite-vec table helpers, test support, artifact writing, artifact validation, and read-only index access.

The remaining public cross-crate surface is intentionally narrow:

- `atlas_index::artifact_metadata` for artifact identity constants needed by ingest manifests.
- `atlas_index::artifact_storage` for vector blob encoding used by index/vector tests and helpers.
- `atlas_index::sqlite_vector_index` for sqlite-vec virtual table SQL helpers.
- `atlas_index::test_support` behind the `test-support` feature for CLI fixture construction.

The old descriptor-driven schema and SQL builder model was removed. Diesel migrations are the physical schema source of truth; `schema_inventory` now retains only validation requirements and stable table/column names needed by raw SQL paths.

## Follow-Up

Structured filter compilation still uses an index-owned raw SQL compiler executed through Diesel `sql_query`. That is a deliberate open design question because FTS and sqlite-vec paths need the same eligible-record relation.
