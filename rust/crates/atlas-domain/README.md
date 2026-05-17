# atlas-domain

`atlas-domain` owns shared Rust vocabulary that is independent of storage, transport, and source parsing.

This crate is for lightweight request, filter, identifier, and output primitives that multiple runtime crates need to agree on. It should not become a home for physical SQLite schema, ingest DTOs, or presentation formatting.

## Owns

- Record keys, pack names, categories, and other small semantic identifiers.
- Search request and filter vocabulary.
- Shared enum-like domains used across crates.
- Lightweight output contracts that are not tied to a specific UI.

## Should Not Own

- SQLite table names, columns, DDL, or artifact metadata inventories.
- Foundry source structs or parsing rules.
- Rich record/content models.
- CLI text/JSON formatting.
- Embedding provider configuration.

## Boundary Notes

Use this crate when two or more crates need the same semantic vocabulary. If the concept has storage shape, prefer `atlas-artifact`; if it is a normalized record/content concept, prefer `atlas-record`; if it is source-specific extraction policy, prefer `atlas-ingest`.
