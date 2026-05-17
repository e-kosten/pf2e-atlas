# 0022. Rust Artifact Policy Ownership

## Status

Accepted

## Context

The Rust runtime needs shared ownership rules for artifact shape, metric meaning, vector storage, and reference graph defaults. Without explicit owners, the same facts tend to appear as raw strings in ingest extraction, SQLite writers, row readers, search filters, presentation, and tests.

## Decision

`atlas-artifact` owns physical SQLite artifact shape and storage encodings. Ordered table/column descriptors drive production insert and select SQL for artifact table shapes, and the crate owns `f32` vector blob encoding/decoding. Runtime crates may own query predicates, row mapping, validation reports, and search semantics, but they should not duplicate full table column inventories or SQLite byte-layout helpers.

`atlas-record` owns storage-agnostic normalized metric definitions. `MetricRow` remains open-ended record data, while typed static and pattern definitions describe known metric keys, value types, labels, namespaces, and groups. Ingest uses definition-owned key helpers for known metrics and audits emitted rows against the catalog. Unknown emitted metrics are build warnings and diagnostics, not artifact validation failures.

`atlas-record` also owns the semantic default reference graph policy. The default graph is public non-embedded reference edges. Public embedded edges and non-public edges remain stored with source metadata and require expanded graph modes. `atlas-index` lowers this policy to SQL predicates over `reference_edges`; the artifact does not store an `is_default_graph_edge` projection.

## Consequences

Adding or renaming normalized metrics should happen in `atlas-record::metrics`, with ingest extraction and presentation/search consumers referring to the definition or pattern helper rather than repeating anonymous strings.

Vector storage code belongs at the artifact boundary even though vector values are produced by `atlas-embedding` and queried by `atlas-index`.

Reference graph behavior can evolve through a named policy and SQL lowering rather than adding a derived boolean column that could drift from the semantic definition.

Descriptor-owned SQL is not a general query engine. Filtering, ranking, vector KNN query shape, pagination, and user-facing search semantics remain owned by `atlas-index` and `atlas-search`.
