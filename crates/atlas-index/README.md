# atlas-index

`atlas-index` owns index read/write boundaries and concrete SQLite artifact access.

This crate opens validated artifacts, loads persisted rows, validates artifact contract coherence, compiles canonical filters into SQL keysets, runs lexical/vector SQL, exposes inspection summaries, and writes completed SQLite artifacts from build inputs. It is the storage access boundary for both runtime reads and ingest-time artifact writes.

## Owns

- `SearchIndex` read contract and `SqliteIndexReader` read handles.
- `IndexArtifactWriter` write contract and `SqliteIndexWriter` artifact writes.
- Artifact validation diagnostics and validation reports.
- Row readers and hydration into `atlas-record` models.
- Filter-to-SQL keyset compilation.
- Vector query SQL over `document_embedding_cache` and `record_vector_index`.
- Index inspection summaries.

## Should Not Own

- Ingest-time source normalization.
- Physical table/column inventories that belong in `atlas-artifact`.
- Query embedding generation.
- Product-level search ranking or vector-hit collapse.
- CLI presentation.

## Boundary Notes

Runtime surfaces should reach SQLite through `SqliteIndexReader` and retrieval-facing traits, not by opening their own connections. Ingest should write artifacts through `IndexArtifactWriter` implementations rather than owning database-specific writers. `atlas-index` may own SQL execution semantics, but physical schema names should come from `atlas-artifact` and product-facing retrieval behavior should compose through `atlas-search`.
