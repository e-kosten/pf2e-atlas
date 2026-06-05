# Architecture Decisions

This index is the quickest way to scan accepted architecture decision records for PF2e Atlas.

## Current ADRs

- [`0018-rust-default-embedding-model.md`](./0018-rust-default-embedding-model.md): BGE small is the default Rust embedding model with weighted chunk retrieval.
- [`0019-rust-cli-search-record-surface.md`](./0019-rust-cli-search-record-surface.md): the CLI separates record identification from result-set search.
- [`0020-rust-content-documents.md`](./0020-rust-content-documents.md): records preserve authored rich text as `RichDocument`, with presentation, structured FTS, semantic chunks, and reference edges derived from that canonical content model.
- [`0021-rust-runtime-index-and-retrieval-boundaries.md`](./0021-rust-runtime-index-and-retrieval-boundaries.md): runtime path policy belongs to `atlas-runtime`, SQLite read access belongs to `SqliteIndexReader` behind focused read traits and the `RetrievalReadIndex` bundle, artifact writes go through `IndexArtifactWriter`, and product retrieval routes through `AtlasRetrievalService`.
- [`0022-rust-artifact-policy-ownership.md`](./0022-rust-artifact-policy-ownership.md): artifact shape/storage, typed metric definitions, and default reference graph policy have explicit owning crates.
- [`0023-rust-cli-json-contract.md`](./0023-rust-cli-json-contract.md): CLI JSON uses one envelope, shared record/result DTOs, stable detail hydration levels, and standard exit-code classes.
- [`0024-rust-search-retrieval-and-fusion-controls.md`](./0024-rust-search-retrieval-and-fusion-controls.md): text search uses one default hybrid surface with advanced FTS/vector retrieval controls, weighted-RRF fusion, and exact identity tiering.
- [`0025-rust-graph-context-retrieval.md`](./0025-rust-graph-context-retrieval.md): graph context retrieval is key-based, one-hop, and separate from search relationship filters.
- [`0026-rust-cli-product-surface.md`](./0026-rust-cli-product-surface.md): PF2e Atlas is a Rust CLI plus first-party skill product; future TUI and derived tags are Rust-owned follow-ups.
- [`0027-rust-runtime-lint-policy.md`](./0027-rust-runtime-lint-policy.md): runtime Rust code denies panic-oriented Clippy lints outside tests, while tests keep assertion-oriented unwrap/expect ergonomics.

## Historical ADRs

ADRs 0001-0017 preserve design history for earlier architecture work and migration sequencing. They are retained as context, but current implementation guidance lives in the Rust architecture docs and current ADRs above. ADR 0026 supersedes ADR 0017 for product surface and workspace layout.
