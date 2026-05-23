# atlas-index

`atlas-index` owns index backend access contracts plus completed-artifact read and write implementations.

This crate opens validated artifacts, loads persisted rows, validates artifact contract coherence, compiles canonical filters into backend keysets, runs lexical/vector queries, writes backend artifacts from normalized build inputs, and exposes inspection summaries. It is the storage backend boundary for both build-time artifact creation and runtime artifact reading.

## Owns

- `SqliteIndexReader` read handles.
- `LadybugIndexReader` spike read handles.
- `SqliteIndexWriter` and `LadybugIndexWriter` artifact writers.
- Backend-neutral search and build/write contracts.
- Artifact validation diagnostics and validation reports.
- Row readers and hydration into `atlas-record` models.
- Filter-to-SQL keyset compilation.
- Vector query SQL over `document_embedding_cache` and `record_vector_index`.
- Index inspection summaries.

## Should Not Own

- Ingest-time source normalization or build orchestration.
- Physical table/column inventories that belong in `atlas-artifact`.
- Query embedding generation.
- Product-level search ranking or vector-hit collapse.
- CLI presentation.

## Boundary Notes

Runtime surfaces should reach SQLite through `SqliteIndexReader`, not by opening their own connections. `atlas-index` may own SQL execution semantics, but physical schema names should come from `atlas-artifact` and product-facing retrieval behavior should compose through `atlas-search`.

Backend-specific code is grouped by backend and direction: production SQLite code lives under `sqlite/reader.rs` and `sqlite/writer/`, while the spike Ladybug backend lives under `ladybug/reader/` and `ladybug/writer/`. New backend-specific modules should follow that shape instead of adding top-level reader or writer modules.
