# atlas-index

`atlas-index` owns reference-domain SQLite schema, validation, publication and
queries within the persistent Atlas workspace. `atlas-storage` owns connections,
transactions, the migration ledger and coordinated maintenance admission.

## Owns

- `SqliteIndexReader` handles bound to admitted read sessions and snapshots.
- Reference schema SQL and relational models, composed with saved-state migrations
  by runtime into one workspace.
- Incremental reference publication through `SqliteIndexWriter` on an admitted
  maintenance connection, retaining stable record identities and unchanged rows.
- Checked DTO snapshots, compact prepared content, diagnostics and relationship
  occurrences, including exact metadata for reusing source preparation.
- Typed filter catalog discovery, validation and SQL compilation.
- Curated FTS5 units, semantic attribution and profile-specific sqlite-vec tables.
- Durable exact-input embedding checkpoints and reference-update job metadata.
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
hashes and vector shape without model assets. Embedding reuse reads durable
checkpoints by exact model identity and execution-input key, checking input hash,
native token count, dimensions and vector checksum. It does not reconstruct old
embedding text from active reference rows. Checkpoints remain independent of
content/projection versions and are pruned only after successful publication.
