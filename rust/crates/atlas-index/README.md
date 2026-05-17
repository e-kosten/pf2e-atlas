# atlas-index

`atlas-index` owns read-only access to completed Rust SQLite artifacts.

This crate opens validated artifacts, loads persisted rows, validates artifact contract coherence, compiles canonical filters into SQL keysets, runs lexical/vector SQL, and exposes inspection summaries. It is the runtime storage access boundary.

## Owns

- `AtlasIndex` read handles.
- Artifact validation diagnostics and validation reports.
- Row readers and hydration into `atlas-record` models.
- Filter-to-SQL keyset compilation.
- Vector query SQL over `document_embedding_cache` and `record_vector_index`.
- Index inspection summaries.

## Should Not Own

- Ingest-time source normalization or artifact mutation.
- Physical table/column inventories that belong in `atlas-artifact`.
- Query embedding generation.
- Product-level search ranking or vector-hit collapse.
- CLI presentation.

## Boundary Notes

Runtime surfaces should reach SQLite through `AtlasIndex`, not by opening their own connections. `atlas-index` may own SQL execution semantics, but physical schema names should come from `atlas-artifact` and product-facing retrieval behavior should compose through `atlas-search`.
