# 0022. Rust Artifact Policy Ownership

## Status

Accepted

## Context

The Rust runtime needs shared ownership rules for artifact shape, metric meaning, vector storage, and reference graph defaults. Without explicit owners, the same facts tend to appear as raw strings in ingest extraction, SQLite writers, row readers, search filters, presentation, and tests.

## Decision

`atlas-index` owns physical SQLite artifact shape and storage encodings. Diesel migrations and schema declarations drive ordinary relational table creation, writer rows, and stable reader hydration. `atlas-index` also owns artifact metadata constants, narrow validation requirements for external artifact files, and `f32` vector blob encoding/decoding. Raw SQL remains appropriate inside `atlas-index` for SQLite facilities that Diesel does not model well: FTS5, sqlite-vec virtual tables, dynamic filter/discovery relations, and validation pragmas. Raw-SQL helpers may keep stable table and column names, but not a parallel schema descriptor model for Diesel-owned tables. The former `atlas-artifact` crate is retired.

`atlas-record` owns storage-agnostic normalized metric definitions. `MetricRow` remains open-ended record data, while typed static and pattern definitions describe known metric keys, value types, labels, namespaces, and groups. Ingest uses definition-owned key helpers for known metrics and validates emitted rows against the catalog during metric extraction. Emitted metric keys and value types are ingest invariants; broader source-field coverage analysis belongs to explicit offline audit tooling rather than default artifact builds.

`atlas-record` also owns the semantic default reference graph policy. The default graph includes authored non-embedded reference edges across typed visibility classifications. Embedded-description edges remain stored with source metadata and require the expanded graph mode because copied embedded prose would otherwise duplicate canonical targets and add noisy context. `atlas-index` lowers this policy to SQL predicates over `reference_edges`; the artifact does not store an `is_default_graph_edge` projection.

`atlas-record` owns the storage-neutral product retrieval disposition and canonical search projection. The retrieval policy produces the typed role, disposition, and rationale for source records, generated canonicals and instances, linked legacy records, and tooling records. Ingest applies that single decision to embedding eligibility, while the index writer persists it and derives the checked legacy boolean from it; neither layer maintains an independent policy table. For canonical creatures, FTS mechanics terms and the embedding parent presentation consume `project_creature_mechanics` directly rather than treating legacy `AtlasRecord.mechanics` as a second search source.

## Consequences

Adding or renaming normalized metrics should happen in `atlas-record::metrics`, with ingest extraction and presentation/search consumers referring to the definition or pattern helper rather than repeating anonymous strings.

Vector storage code belongs at the index artifact boundary even though vector values are produced by `atlas-embedding` and queried by `atlas-index`.

Reference graph behavior can evolve through a named policy and SQL lowering rather than adding a derived boolean column that could drift from the semantic definition.

Retrieval eligibility changes must be made in the `atlas-record` policy and validated through both ingest embedding selection and index persistence. Canonical mechanics search changes must preserve embedding policy-version and semantic-input-hash coherence; a changed unit policy requires a new artifact metadata identity so stale cache rows cannot be reused.

Diesel schema ownership is not a general query-engine mandate. Filtering, ranking, vector KNN query shape, FTS, and SQL paging execution remain owned by `atlas-index` and `atlas-search`, using raw SQL where the database feature is inherently dynamic or SQLite-specific. Product-facing pagination and user-facing search semantics belong above direct SQL details in `atlas-search`.
