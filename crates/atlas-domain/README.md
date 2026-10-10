# atlas-domain

`atlas-domain` owns shared Rust vocabulary that is independent of storage, transport, and source parsing.

This crate is for lightweight request, filter, identifier, and output primitives that multiple runtime crates need to agree on. It should not become a home for physical SQLite schema, ingest DTOs, or presentation formatting.

## Owns

- Record keys, pack names, categories, and other small semantic identifiers.
- Typed query predicates, field states, catalog descriptors, and discovery requests.
- Source passage and owned-node addresses, summaries, and exact-name normalization.
- Shared enum-like domains used across crates.
- Lightweight output contracts that are not tied to a specific UI.

## Should Not Own

- SQLite table names, columns, DDL, or artifact metadata inventories.
- Foundry source structs or parsing rules.
- Rich record/content models.
- CLI text/JSON formatting.
- Embedding provider configuration.

## Boundary Notes

Use this crate when two or more crates need the same semantic vocabulary. SQLite artifact storage belongs in `atlas-index`; checked source-backed views and content preparation belong in `atlas-record`; source loading and build policy belong in `atlas-ingest`. Foundry DTOs belong in `atlas-foundry-model`. CEL parsing and SQL compilation remain in `atlas-index`, separate from the shared typed query request.
