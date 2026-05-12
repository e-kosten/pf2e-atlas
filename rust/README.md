# Rust Workspace

This workspace is the Rust migration foundation for PF2e Atlas. The Rust side is intended to become the owner of the deterministic core path: artifact validation, canonical ingest and index building, lookup/search runtime behavior, CLI workflows, and the Ratatui workbench.

TypeScript and Python may remain useful for exploratory analysis, parity comparison, and transitional tooling, but they are not the default long-term owners for canonical runtime artifacts.

## Layout

- `crates/atlas-domain`: shared Rust contracts and vocabulary.
- `crates/atlas-ingest`: Foundry source loading, normalized ingest records, and SQLite artifact writing.
- `crates/atlas-index`: index/artifact opening and validation.
- `crates/atlas-cli`: the local `atlas` command surface.

Add future crates only when their first real implementation slice lands.

## Validation

Run from this directory:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

The first CLI behavior is:

```bash
cargo run -p atlas-cli -- validate-index --index ../.cache/pf2e-index.sqlite --json
```

Current TypeScript-built indexes are expected to report a legacy-contract diagnostic until the Rust artifact contract is implemented by the index builder.

The first Rust writer behavior is:

```bash
cargo run -p atlas-cli -- build-index --source ../vendor/pf2e --output ../.cache/pf2e-rust-index.sqlite --json
```

The current writer is a minimal Phase 3 slice. It loads Foundry packs and records, normalizes canonical record keys and names, maps `foundry_document_type` plus `foundry_record_type` into `record_family`, preserves those Foundry type axes as explicit source projections, reports skipped records with path and reason, and writes `artifact_metadata`, `packs`, `records`, `record_aliases`, `record_traits`, `reference_edges`, `remaster_links`, unified `record_metrics`, metric catalogs, actor/item/spell side-data tables, and `records_fts`. It also extracts selected direct `system_*` paths, raw price JSON, normalized copper price, activation time, separate effect duration, exact Foundry inline reference links resolved against loaded records, source-backed lookup aliases, and premaster-to-remaster bridges from remaster journals and migration rename files. Full corpus parity, variant-name aliases, embeddings, and manifest schema validation remain later ingest/index slices. Derived tags are intentionally deferred until late in the migration because the Rust model changes require a separate design pass for that surface.

## Artifact Validation Diagnostics

The first Rust artifact contract is `pf2e-atlas-artifact/v1` with SQLite schema version `1`. The durable contract is documented in [`docs/architecture/rust-artifact-contract.md`](../docs/architecture/rust-artifact-contract.md).

`atlas validate-index --json` can return these validation codes:

| Code | Meaning |
| --- | --- |
| `OK` | The artifact metadata matches the supported Rust runtime contract. |
| `INDEX_UNAVAILABLE` | The SQLite file could not be opened read-only. |
| `MISSING_ARTIFACT_METADATA` | The index does not contain the Rust `artifact_metadata` table. Current TypeScript-built indexes are expected to fail this way. |
| `MISSING_REQUIRED_METADATA` | The `artifact_metadata` table exists but omits one or more required keys. |
| `UNSUPPORTED_CONTRACT_VERSION` | `artifact_contract_version` is not supported by this runtime. |
| `UNSUPPORTED_SCHEMA_VERSION` | `schema_version` is not supported by this runtime. |
| `INVALID_SOURCE_METADATA` | Source identity, source record count, or source hashing metadata is malformed or incompatible. |
| `STALE_SOURCE_SIGNATURE` | The source signature marks the artifact as stale. |
| `EMBEDDING_MISMATCH` | Embedding provider, model, tokenizer, pooling, normalization, dimensions, dtype, distance metric, or prefixes do not match the runtime baseline. |
| `FTS_MISMATCH` | The artifact was built with an unsupported FTS tokenizer contract. |
| `MANIFEST_MISMATCH` | The adjacent manifest path is not a valid relative artifact path. |
| `QUERY_FAILED` | Metadata validation could not complete because a SQLite query failed. |

## Lint Policy

Use standard Clippy with warnings denied, plus `deny(unsafe_code)` in crates. Prefer typed errors in runtime code, but do not globally ban `unwrap` or `expect` during the scaffold phase; tests and short fixture setup often read more clearly with explicit panics. Add narrower lint denies later when a crate's runtime surface is stable enough to justify them.
