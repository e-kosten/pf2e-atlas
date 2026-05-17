# Rust Workspace

This workspace is the Rust migration foundation for PF2e Atlas. The Rust side is intended to become the owner of the deterministic core path: artifact validation, canonical ingest and index building, lookup/search runtime behavior, CLI workflows, and the Ratatui workbench.

TypeScript and Python may remain useful for exploratory analysis, parity comparison, and transitional tooling, but they are not the default long-term owners for canonical runtime artifacts.

## Layout

- `crates/atlas-domain`: shared Rust contracts and vocabulary.
- `crates/atlas-record`: storage-agnostic normalized record DTOs shared by ingest, artifact writing, index loading, and future runtime surfaces.
- `crates/atlas-artifact`: shared SQLite artifact schema descriptors used by writers and validators.
- `crates/atlas-ingest`: Foundry source loading, normalized ingest records, and SQLite artifact writing.
- `crates/atlas-index`: index/artifact opening and validation.
- `crates/atlas-runtime`: shared runtime path resolution, setup readiness, and source-fetch policy.
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
cargo run -p atlas-cli -- index validate --index ../.cache/pf2e-index.sqlite --json
```

JSON output uses a shared envelope: successful command payloads are under `data`, and command failures are under `error`. Artifact validation that runs against an invalid artifact still returns `status: "ok"` with `data.valid: false` and exits with code `3`.

Current TypeScript-built indexes are expected to report a legacy-contract diagnostic until the Rust artifact contract is implemented by the index builder.

The first Rust writer behavior is:

```bash
cargo run -p atlas-cli -- setup
cargo run -p atlas-cli -- index build --json
```

The read-only source analysis behavior is:

```bash
cargo run -p atlas-cli -- index analyze --json
```

Use Cargo's release profile for ingest or query performance measurements:

```bash
cargo run --release -p atlas-cli -- index analyze --source ../vendor/pf2e --json
```

The default Cargo dev profile is useful for edit/build loops, but its runtime timings should not be compared against the TypeScript implementation.
`atlas index build` generates BGE small document embeddings in batches by default; pass `--embedding-model <model>` to compare catalog models and `--embedding-batch-size <count>` to compare batch sizes during performance work.

`atlas setup`, `atlas index build`, index path commands, and semantic search share the `atlas-runtime` path resolver. With `--path-mode auto` (the default), the resolver uses repo-local paths when `git rev-parse --show-toplevel` finds this Rust workspace, otherwise it uses platform user cache paths. Use `--path-mode repo` to require repo-local paths or `--path-mode user` to force platform user paths. Direct flags such as `--source`, `--output`, `--index`, and `--embedding-cache-path` override the resolver for that command. `atlas setup --fetch-source` calls shared runtime source-fetch policy to clone or fast-forward the PF2E source checkout, but embedding model downloads are still a separate preparation step. `atlas index build --no-embeddings` is available for fixture and contract builds that intentionally skip semantic vectors; that output is a base artifact and is not semantic-search-ready.

The current writer loads Foundry packs and records, normalizes canonical record keys and names, maps `foundry_document_type` plus `foundry_record_type` into `record_family`, preserves those Foundry type axes as explicit source projections, reports skipped records with path and reason, and writes `artifact_metadata`, `packs`, `records`, `record_aliases`, `record_traits`, `reference_edges`, `remaster_links`, unified `record_metrics`, metric catalogs, actor/item/spell side-data tables, `records_fts`, `document_embedding_cache`, and `record_vector_index`. It also extracts selected direct `system_*` paths, raw price JSON, normalized copper price, activation time, separate effect duration, exact Foundry inline reference links resolved against loaded records, source-backed lookup aliases, premaster-to-remaster bridges from remaster journals and migration rename files, and variant family metadata. Source-backed aliases and variant family metadata are separate concepts; broad variant base-name aliases should be handled as variant-aware lookup behavior rather than `record_aliases`. Index readers can deserialize the durable record table family into `atlas-record::PersistedRecord` and relationship-bearing `PersistedRecordSet` values; ingest-only construction state stays out of the persisted read shape. Derived tags are a Phase 10 redesign concern because the Rust model changes require a separate design pass for that surface.

## Artifact Validation Diagnostics

The first Rust artifact contract is `pf2e-atlas-artifact/v1` with SQLite schema version `1`. The durable contract is documented in [`docs/architecture/artifact-contract.md`](../docs/architecture/artifact-contract.md).

`atlas index validate --json` can return these validation codes:

| Code | Meaning |
| --- | --- |
| `ok` | The artifact metadata matches the supported Rust runtime contract. |
| `index_unavailable` | The SQLite file could not be opened read-only. |
| `missing_artifact_metadata` | The index does not contain the Rust `artifact_metadata` table. Current TypeScript-built indexes are expected to fail this way. |
| `missing_required_metadata` | The `artifact_metadata` table exists but omits one or more required keys. |
| `unsupported_contract_version` | `artifact_contract_version` is not supported by this runtime. |
| `unsupported_schema_version` | `schema_version` is not supported by this runtime. |
| `artifact_contract_violation` | The artifact shape violates the Rust artifact contract. |
| `invalid_source_metadata` | Source identity, source record count, or source hashing metadata is malformed or incompatible. |
| `stale_source_signature` | The source signature marks the artifact as stale. |
| `embedding_mismatch` | Embedding provider, model, tokenizer, pooling, normalization, dimensions, dtype, distance metric, or prefixes do not match the runtime baseline. |
| `fts_mismatch` | The artifact was built with an unsupported FTS tokenizer contract. |
| `manifest_mismatch` | The adjacent manifest path is not a valid relative artifact path. |
| `vector_extension_unavailable` | The runtime could not load the required SQLite vector extension. |
| `query_failed` | Metadata validation could not complete because a SQLite query failed. |

## Lint Policy

Use standard Clippy with warnings denied, plus `deny(unsafe_code)` in crates. Prefer typed errors in runtime code, but do not globally ban `unwrap` or `expect` during the scaffold phase; tests and short fixture setup often read more clearly with explicit panics. Add narrower lint denies later when a crate's runtime surface is stable enough to justify them.
