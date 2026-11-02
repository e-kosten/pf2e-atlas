# atlas-index

`atlas-index` owns reference-domain SQLite schema, validation, publication and
queries within the persistent Atlas workspace. `atlas-storage` owns connections,
transactions, the migration ledger and coordinated maintenance admission.

## Owns

- `SqliteIndexReader` handles bound to admitted read sessions and snapshots.
- Reference schema SQL and relational models, composed with saved-state migrations
  by runtime into one workspace.
- One owned `PreparedPublication` binding captured metadata, the expected
  generation and numerical selection, retaining stable identities and unchanged rows.
- Checked DTO snapshots, compact prepared content, diagnostics and relationship
  occurrences, including exact metadata for reusing source preparation.
- Typed filter catalog discovery, validation and SQL compilation.
- Curated FTS5 units, semantic attribution and profile-specific sqlite-vec tables.
- Input-owned native vectors and a bounded recovery cache for unpublished inference.
- Independent requested source/model configuration, component-version inspection
  and automatic query-projection backfills.

## Boundaries

Runtime composes the workspace; product retrieval goes through `atlas-search`.
Ingest owns source admission, content preparation and model execution. This crate
does not generate embeddings, decide product ranking, or present CLI output.
Saved lists and encounters belong to `atlas-local-state` in the same database.

A reference publication validates its transaction before advancing the reference
revision. It updates reference-domain rows without replacing the database or
clearing saved state. Schema upgrades use the shared migration coordinator;
component upgrades rebuild only affected derived outputs from retained inputs.

Offline validation checks source addresses, content coherence, semantic coverage,
hashes and vector shape without model assets. Reuse binds complete model/input,
token and checksum identity. Selected unpublished bytes are build-owned before
cache eviction; successful complete publication clears recovery values atomically.
Current-schema fixtures use this same publication owner; historical fixtures seed
their old representation and exercise the real migration callbacks. See
[ADR0065](../../docs/architecture/decisions/0065-input-owned-reference-publication.md)
for numerical ownership and recovery lifetimes.
