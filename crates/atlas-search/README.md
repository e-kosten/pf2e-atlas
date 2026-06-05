# atlas-search

`atlas-search` owns product-facing retrieval orchestration for the Rust runtime.

This crate coordinates read-only index handles, query embedding, vector/lexical retrieval, result assembly, ranking modes, graph context, variants, remaster links, and similar-record retrieval behind `AtlasRetrievalService`.

## Owns

- `AtlasRetrievalService` as the product-facing retrieval boundary.
- Narrow capability traits such as `RecordRetrieval`, `TextRetrieval`, `SimilarRetrieval`, `GraphRetrieval`, `VariantRetrieval`, and `RemasterRetrieval`.
- Semantic, lexical, hybrid, filter-only, graph context, similar-record, variant, and remaster retrieval orchestration.
- Query embedding composition with `atlas-embedding`.
- Vector-hit collapse and search ranking modes.
- User-facing search result DTOs.

## Should Not Own

- Opening source files or building artifacts.
- Raw SQLite schema definitions.
- Artifact validation or row-loading internals.
- Embedding model catalog definitions.
- CLI command presentation.

## Boundary Notes

Product surfaces should use this crate for retrieval instead of assembling `atlas-index` and `atlas-embedding` directly. Public APIs should expose search-owned request/result types and collapse implementation details into `SearchErrorKind` when callers need error classification. Keep lower-level index, SQL, semantic/vector, and filter compiler details private unless they are durable product boundaries.

The crate root is the ordinary product surface. Semantic-only retrieval DTOs and low-level fusion controls are available under `atlas_search::expert` for CLI diagnostics, tuning, and validation workflows, but normal callers should prefer `TextRetrieval` with default `TextSearchRequest` tuning.

Text-search diagnostics are explain-scoped. `TextSearchResult::diagnostics` and per-match diagnostics are populated only when the caller sets `TextSearchRequest::explain`; ordinary search responses carry records, pagination, retrieval mode, and resolved fusion settings without query-token or rank-evidence details.
