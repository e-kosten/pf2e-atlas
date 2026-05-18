# atlas-search

`atlas-search` owns product-facing retrieval orchestration for the Rust runtime.

This crate coordinates read-only index handles, query embedding, vector/lexical retrieval, result assembly, ranking modes, and future retrieval patterns behind `AtlasRetrievalService`.

## Owns

- `AtlasRetrievalService` as the product-facing retrieval boundary.
- Semantic, lexical, hybrid, and filter-only orchestration as those modes land.
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

Future Rust CLI, TUI, and MCP surfaces should use this crate for retrieval instead of assembling `atlas-index` and `atlas-embedding` directly. Keep lower-level semantic/vector helpers private unless they are durable product boundaries.
