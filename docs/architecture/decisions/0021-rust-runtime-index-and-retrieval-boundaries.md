# 0021 Rust Runtime, Index, And Retrieval Boundaries

## Status

Accepted

## Context

The Rust runtime has separate crates for runtime composition, SQLite artifact access, ingest, embedding, search, and CLI presentation. That split is useful, but early migration code allowed some operation helpers to accept SQLite artifact paths and open their own database connections internally.

Path-based operation helpers are convenient during CLI-first migration, but they weaken the runtime boundary:

- callers can bypass the runtime object that owns artifact location policy
- multiple crates or surfaces can grow their own default-path and override handling
- database connection ownership becomes per-helper instead of per artifact handle
- validation, inspection, record loading, vector maintenance, and search can drift into separate SQLite access paths

The retrieval layer has a related boundary risk. The current `SemanticSearchService` is useful for validating the embedding and sqlite-vec loop, but semantic search is only one retrieval pattern. The durable product surface needs record get, strict record resolution, filter-only listing, lexical search, semantic search, and hybrid search to compose through one retrieval boundary rather than a set of peer services that each own part of the database/search interaction.

Rust should keep the writer lifecycle separate from runtime artifact reads. `atlas-ingest` owns build orchestration and prepares the build input for replacement artifacts. `atlas-index` owns writable SQLite construction and publishing through `IndexArtifactWriter` implementations such as `SqliteIndexWriter`. Runtime surfaces should consume an existing artifact through the runtime/index/retrieval boundaries.

## Decision

`atlas-runtime` owns runtime artifact location policy and surface composition. It is the single owner for deriving the default source checkout, SQLite artifact path, embedding model cache path, and any optional path overrides. CLI flags and future surface-specific configuration may supply overrides, but default resolution and readiness checks route through runtime types.

`atlas-index` owns runtime index read/write boundaries. Runtime read interaction goes through focused read traits such as `RecordReadIndex`, `IdentityReadIndex`, `FilterReadIndex`, `FtsReadIndex`, `VectorReadIndex`, `ReferenceReadIndex`, `VariantReadIndex`, and `RemasterReadIndex`, with the composite `RetrievalReadIndex` bundle for consumers that legitimately need the full retrieval read surface. `SqliteIndexReader` is the SQLite implementation. Artifact construction goes through `IndexArtifactWriter`, with `SqliteIndexWriter` as the SQLite implementation. Runtime artifact operations such as base validation, inspection, record loading, record lookup primitives, filter-backed listing, lexical query primitives, vector validation, and vector query execution should remain behind the index read boundary rather than being opened directly by CLI or search code.

Public path-based runtime operation helpers are not durable API. Passing a path to a concrete index reader constructor such as `SqliteIndexReader::open_*` is the correct boundary for opening an artifact. Passing paths to validation, inspection, search, or record-loading helpers that then open their own connections is a migration convenience that should be removed as the index boundary is consolidated.

`atlas-search::AtlasRetrievalService` is the durable product-facing retrieval boundary. CLI and future TUI surfaces should route retrieval use cases through this service rather than constructing separate lookup, list, lexical, semantic, or hybrid services as peer public surfaces. The service owns retrieval orchestration and result assembly over the index-owned `RetrievalReadIndex` capability bundle and embedding components, while direct SQLite interaction remains inside `SqliteIndexReader`.

Consumers should depend on narrow `atlas-search` capability traits when they only need a subset of retrieval behavior: `RecordRetrieval`, `TextRetrieval`, `SimilarRetrieval`, `GraphRetrieval`, `VariantRetrieval`, and `RemasterRetrieval`. These traits expose search-owned request/result DTOs and keep index-owned rows, SQL details, vector errors, and filter compiler internals out of product callers. Semantic-only retrieval and low-level fusion controls are expert/debug APIs rather than ordinary product entrypoints.

`SemanticSearchService` is an internal migration component for the semantic-only embedding/sqlite-vec validation loop. It may remain as an implementation detail under `AtlasRetrievalService`, but it is not the long-term public retrieval boundary.

`atlas-ingest` remains the owner of artifact build orchestration, embedding preparation, and the owned `IndexBuildInput` handoff. Writable SQLite usage belongs in `atlas-index` writer implementations and is separate from runtime artifact consumption. Normal embedding-enabled builds hand document embedding cache rows and sqlite-vec `record_vector_index` inputs to the index writer before the completed artifact is published.

Base artifact validation should use plain SQLite and validate the structural artifact contract without requiring sqlite-vec. Search-readiness validation should layer on embedding model, sqlite-vec, vector-table, and vector-count checks where the caller is preparing the full retrieval runtime.

## Consequences

Runtime-facing code should be refactored so path-based operation helpers become index reader methods or private CLI wrappers over runtime/index handles. The end state should not leave public helper APIs that accept a database path and open SQLite independently for validation, inspection, record reads, vector maintenance, or search.

Runtime vector rebuild is not part of the target architecture. If semantic vector readiness is missing or stale, the artifact should be rebuilt through ingest with the selected embedding model. Swapping embedding models is an ingest rebuild, not a vector-table rebuild over an existing completed artifact.

CLI and future runtime surfaces should request artifact paths and readiness through `atlas-runtime`. Surface code should not independently derive default artifact locations or require users to provide path overrides for normal operation.

The search crate should consolidate public retrieval behavior behind `AtlasRetrievalService` plus narrow capability traits implemented by that service. Internal modules and smaller components are still appropriate for lexical execution, semantic execution, record resolution, filter compilation, ranking, or result shaping, but they should not become separate product-facing service entrypoints unless a future ADR changes this boundary.

Architecture diagrams should show:

- `atlas-runtime` owning runtime artifact locations and composition
- `SqliteIndexReader` owning direct SQLite artifact reads behind focused read traits and the `RetrievalReadIndex` bundle
- `AtlasRetrievalService` owning retrieval orchestration
- `atlas-embedding` owning embedding model behavior and vector unit preparation, not direct runtime database ownership
- `atlas-ingest` owning build orchestration/build-input handoff
- `atlas-index` owning artifact writer lifecycles through `IndexArtifactWriter` implementations such as `SqliteIndexWriter`

This decision supports the CLI shape from ADR 0019: `record get`, `record resolve`, and `search` remain product commands, while internal retrieval modes remain behind a common service boundary.
