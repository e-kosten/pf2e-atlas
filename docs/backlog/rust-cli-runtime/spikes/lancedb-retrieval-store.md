# LanceDB Retrieval Store Spike

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-05-06

## Question

Is LanceDB a better retrieval artifact than SQLite vector tables for PF2E semantic or hybrid search?

## Motivation

LanceDB is vector-first and has Rust, Python, and TypeScript access paths. It may fit cross-language offline prep plus Rust runtime retrieval better than SQLite vector extensions if semantic search quality, metadata filtering, or hybrid retrieval becomes central.

## Prototype Scope

- Build a LanceDB prototype with PF2E record embeddings, metadata filters, and text fields.
- Test vector retrieval, metadata constraints, and any available full-text or hybrid retrieval behavior.
- Compare against SQLite vector search and any Tantivy-backed hybrid pipeline.
- Exercise both tiny fixtures and a representative PF2E slice.

## Do Not Mock

- vector storage
- metadata filters
- category/subcategory filters
- embedding dimensions
- index build time
- query latency
- local artifact size
- Rust runtime access

## Outputs

- retrieval quality notes
- operational and packaging notes
- artifact layout recommendation
- decision: serious candidate, companion only, or reject for now

## Migration Dependency

The migration should not replace SQLite vector search with LanceDB unless this spike demonstrates better quality or operational simplicity for the actual PF2E workload.
