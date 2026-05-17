# atlas-runtime

`atlas-runtime` owns runtime path resolution and shared setup policy.

This crate decides where source data, embedding caches, and artifacts live for repo-local and user-cache modes. It constructs runtime handles that CLI and future Rust surfaces can share without each surface reimplementing path or setup behavior.

## Owns

- Repo/user path resolution.
- Runtime setup status.
- Source fetch policy for setup commands.
- Construction of `AtlasIndex` and `AtlasRetrievalService` handles.
- Shared runtime defaults for local CLI and future Rust surfaces.

## Should Not Own

- Search semantics or ranking.
- SQLite schema or artifact validation rules.
- Foundry source normalization.
- CLI-specific output formatting.
- Embedding model execution policy beyond runtime wiring.

## Boundary Notes

Product surfaces should compose through this crate for paths and service construction. Durable search behavior still belongs in `atlas-search`, read access in `atlas-index`, and physical schema in `atlas-artifact`.
