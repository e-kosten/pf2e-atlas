# atlas-artifact

`atlas-artifact` owns the physical SQLite artifact contract for the Rust runtime.

This crate is the place for table names, column names, artifact metadata keys, schema descriptors, DDL helpers, insert/select SQL helpers, and storage-format primitives such as vector BLOB encoding. It describes what a completed artifact looks like on disk; it does not decide what PF2E source data means or how users search it.

## Owns

- SQLite table and column descriptors.
- Artifact metadata key names and expected contract values.
- Schema creation SQL and descriptor-backed SQL helpers.
- Physical storage formats that ingest and index must share.
- Contract constants needed by both writers and readers.

## Should Not Own

- Foundry source parsing or normalization.
- Record presentation, content semantics, or metric meaning.
- Artifact writing policy beyond physical SQL shapes.
- Row hydration into runtime records.
- Search ranking, lookup behavior, or CLI output.

## Boundary Notes

`atlas-index` writes, validates, and reads artifacts using this crate's physical contract. `atlas-ingest` produces the normalized build input but should not touch database-specific artifact schema or connections. New artifact tables, columns, or storage encodings should start here so writer and reader code cannot drift.
