# atlas-embedding

`atlas-embedding` owns embedding model behavior and embedding input construction.

This crate handles model catalog decisions, query and document vector generation, token budgeting, embedding text rendering, document-unit construction, and semantic input hashes. It owns how text becomes vectors, not how vectors are stored in SQLite.

## Owns

- Embedding model catalog and model runtime configuration.
- Query and document embedding generation.
- Token budgeting and truncation telemetry.
- Embedding input text rendering.
- Parent and child document unit construction.
- Embedding unit kind vocabulary.

## Should Not Own

- Foundry raw markup parsing.
- SQLite table names, DDL, or artifact metadata ownership.
- Vector BLOB storage layout for SQLite artifacts.
- Search result collapse or ranking policy.
- CLI command presentation.

## Boundary Notes

`atlas-ingest` uses this crate to prepare and generate document embeddings during artifact builds. `atlas-search` uses it to embed user queries. Stored vector byte layout belongs at the artifact boundary, even when the values originated here.
