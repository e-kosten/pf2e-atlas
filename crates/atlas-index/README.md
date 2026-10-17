# atlas-index

`atlas-index` owns index read/write boundaries and concrete SQLite artifact access.

This crate opens validated artifacts, loads persisted rows, validates artifact contract coherence, compiles canonical filters into SQL keysets, runs lexical/vector SQL, exposes inspection summaries, and writes completed SQLite artifacts from build inputs. It is the storage access boundary for both runtime reads and ingest-time artifact writes.

## Owns

- Concrete `SqliteIndexReader` handles tied to a validated artifact generation.
- `IndexArtifactWriter` write contract and `SqliteIndexWriter` artifact writes.
- Artifact validation diagnostics and validation reports.
- Diesel migrations, checked-in schema declarations validated against those migrations, and ordinary relational writer/reader row models.
- Bounded checked source snapshots, summaries, selected prepared HTML and attributed relationship reads.
- Filter-to-SQL keyset compilation.
- Curated lexical units and FTS5, semantic unit attribution, deduplicated input vectors, and sqlite-vec query SQL.
- Typed query catalog discovery, exact numeric projections, and shared eligibility keysets.
- Bounded old-source/cache/alias batches for ingest to verify reusable embeddings.
- Index inspection summaries.

## Should Not Own

- Ingest-time source normalization.
- Query embedding generation.
- Product-level search ranking or vector-hit collapse.
- CLI presentation.

## Boundary Notes

Runtime surfaces reach SQLite through `SqliteIndexReader`, rather than opening their own connections. Ingest writes completed artifacts through `IndexArtifactWriter`. The artifact is rebuilt from source when its contract changes; local lists and encounters live in a separate store. `atlas-index` owns the SQLite artifact contract: migration files under `migrations/` define the schema, artifact creation embeds those migrations, and tests check the Diesel declarations against them. Ordinary relational access uses Diesel; FTS5, sqlite-vec, dynamic filter relations, and validation pragmas use explicit SQL. Product-facing retrieval composes through `atlas-search`.

Offline validation checks source addresses, cache safety and coherence, semantic coverage, hash syntax, and vector shape. It does not require model assets or reproduce tokenization. Before reusing vectors, `atlas-ingest` reconstructs each old input with the pinned tokenizer and verifies its hash, token count, and attribution. The index crate supplies bounded batches for that check without depending on embedding execution.
