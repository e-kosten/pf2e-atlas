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

Install the local CLI from a clone with:

```bash
cargo install --path crates/atlas-cli --locked
```

The standard first-run setup command is:

```bash
atlas setup
```

`atlas setup` resolves the default paths, fetches or updates the PF2E source checkout, checks the configured embedding model cache, builds or repairs the SQLite artifact when needed, and validates the selected readiness target. Embeddings are required by default. Use the faster record-only path when you only need `record get`, `record resolve`, or filter-only listing:

```bash
atlas setup --no-embeddings
```

Use `--check` for a no-write readiness and planned-action report, and `--offline` to prevent network-backed source or model preparation.

After setup, record commands can run without passing index paths:

```bash
atlas record get actionspf2e:1kGNdIIhuglAjIp9
atlas record resolve "Treat Wounds" --pack-name actionspf2e
```

JSON output uses a shared envelope: successful command payloads are under `data`, and command failures are under `error`. Artifact validation that runs against an invalid artifact still returns `status: "ok"` with `data.valid: false` and exits with code `3`.

Filter discovery is available through `atlas filters`:

```bash
atlas filters fields --family spell
atlas filters values --field traits --family spell
atlas filters values --field level --family equipment
atlas filters values --field metric --family creature --metric-label save
```

Filter discovery commands always emit the standard JSON envelope. They share the normal filter flags for family, rarity, trait, level, price, publication title, pack name, pack label, and reference narrowing. Metric predicates for narrowing can be expressed with `--filter-json`; `filters values --field metric` reserves `--metric` for selecting the metric key or exact known metric label to inspect.

## Agent Skills

The CLI includes a first-party PF2e Atlas skill package for local coding agents. Inspect install readiness with:

```bash
atlas agent skills doctor --json
```

Install into the current workspace with:

```bash
atlas agent skills install --target agents --scope workspace --yes --json
```

Supported targets are `agents`, `claude`, `codex`, `copilot`, `gemini`, and `kiro`. Supported scopes are `workspace` and `global`. Without explicit `--target`, `--scope`, and `--yes`, an interactive terminal uses a small picker flow. Existing skill directories are only overwritten with confirmation in the picker or with `--force` in non-interactive mode. Installed Atlas-managed skills include an `.atlas-skill.json` manifest with the package content hash used by doctor and install.

Manual artifact build remains available for development and diagnostics:

```bash
cargo run -p atlas-cli -- index build --no-embeddings --json
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
`atlas setup` and `atlas index build` generate BGE small document embeddings in batches by default; pass `--embedding-model <model>` and `--embedding-batch-size <count>` on setup or build commands to compare catalog models and batch sizes during performance work.

`atlas setup`, `atlas index build`, index path commands, and semantic search share the `atlas-runtime` path resolver. With `--path-mode auto` (the default), the resolver uses repo-local paths when `git rev-parse --show-toplevel` finds this Rust workspace, otherwise it uses platform user cache paths. Use `--path-mode repo` to require repo-local paths or `--path-mode user` to force platform user paths. Direct flags such as `--source`, `--output`, `--index`, and `--embedding-cache-path` override the resolver for that command. `atlas setup --offline` prevents source fetch/update and embedding model preparation. `atlas setup --no-embeddings` and `atlas index build --no-embeddings` intentionally skip semantic vectors; that output is a base artifact and is not semantic-search-ready.

The current writer loads Foundry packs and records, normalizes canonical record keys and names, maps `foundry_document_type` plus `foundry_record_type` into `record_family`, preserves those Foundry type axes as explicit source projections, reports skipped records with path and reason, and writes `artifact_metadata`, `packs`, `records`, `record_aliases`, `record_traits`, `reference_edges`, `remaster_links`, unified `record_metrics`, metric catalogs, filter discovery catalogs, actor/item/spell side-data tables, `records_fts`, `document_embedding_cache`, and `record_vector_index`. It also extracts selected direct `system_*` paths, raw price JSON, normalized copper price, activation time, separate effect duration, exact Foundry inline reference links resolved against loaded records, source-backed lookup aliases, premaster-to-remaster bridges from remaster journals and migration rename files, and variant family metadata. Source-backed aliases and variant family metadata are separate concepts; broad variant base-name aliases should be handled as variant-aware lookup behavior rather than `record_aliases`. Index readers can deserialize the durable record table family into `atlas-record::PersistedRecord` and relationship-bearing `PersistedRecordSet` values; ingest-only construction state stays out of the persisted read shape. Derived tags are a Phase 10 redesign concern because the Rust model changes require a separate design pass for that surface.

## Artifact Validation Diagnostics

The first Rust artifact contract is `pf2e-atlas-artifact/v1` with SQLite schema version `1`. The durable contract is documented in [`docs/architecture/artifact-contract.md`](../docs/architecture/artifact-contract.md).

`atlas index validate --json` validates full semantic readiness by default, including vector table readiness. Use `atlas index validate --no-embeddings --json` for base artifact validation, or `atlas index validate --embeddings-only --json` for focused embedding/vector diagnostics.

Validation can return these codes:

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
