# Rust Artifact Contract

This document defines the first runtime artifact boundary for the Rust migration. The Rust runtime opens a prepared SQLite index read-only and validates normal SQLite metadata before loading vector tables or runtime search state.

## Contract Version

The first supported contract version is:

```text
pf2e-atlas-artifact/v1
```

The SQLite schema version for this first Rust artifact family is:

```text
1
```

These values are stored in `artifact_metadata` as `artifact_contract_version` and `schema_version`. A runtime that does not support either value must fail validation before loading lookup, search, embedding, or vector capabilities.

## Metadata Table

Runtime artifacts must include:

```sql
CREATE TABLE artifact_metadata (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
```

The metadata table must be readable with plain SQLite. Validation of this table must not require loading `sqlite-vec` or any other extension.

Required keys:

| Key | Required value or rule |
| --- | --- |
| `artifact_contract_version` | `pf2e-atlas-artifact/v1` |
| `schema_version` | `1` |
| `source_kind` | `foundry-pf2e` |
| `source_signature` | `foundry-pf2e:<signature>` for current source snapshots |
| `source_record_count` | positive integer |
| `content_hash_algorithm` | `sha256` |
| `embedding_provider_family` | `transformers-js-minilm` for the first MiniLM baseline |
| `embedding_model_id` | `Xenova/all-MiniLM-L12-v2` |
| `embedding_model_revision` | `main` until model artifact checksums are available |
| `embedding_tokenizer_id` | `Xenova/all-MiniLM-L12-v2` |
| `embedding_pooling` | `mean` |
| `embedding_normalization` | `l2` |
| `embedding_dimensions` | `384` |
| `embedding_dtype` | `f32` |
| `embedding_distance_metric` | `cosine` |
| `embedding_document_prefix` | empty string for the MiniLM baseline |
| `embedding_query_prefix` | empty string for the MiniLM baseline |
| `fts_tokenizer` | `unicode61 remove_diacritics 2` |
| `adjacent_manifest_path` | relative path next to the SQLite artifact |

## Validation Families

Validation diagnostics are grouped by contract family:

- `contract`: artifact contract version.
- `schema`: SQLite schema version.
- `source`: Foundry source kind, source signature, record count, and content hashing.
- `embedding`: query/document vector compatibility.
- `fts`: lexical tokenizer compatibility.
- `manifest`: adjacent artifact manifest linkage.

`atlas validate-index --json` returns stable JSON diagnostics for missing or incompatible metadata. Current TypeScript-built indexes that only expose the legacy `metadata` table are intentionally reported as missing the Rust artifact contract.

## Adjacent Manifest

`adjacent_manifest_path` is reserved for a JSON manifest that can carry larger provenance, source file inventories, report locations, and non-runtime artifact references. Runtime startup validation only requires the relative path metadata during phase one; manifest schema validation lands when Rust ingest starts writing artifacts.

## Extension Loading

Artifact metadata validation must not load vector extensions. Later search commands may require `sqlite-vec`, but contract validation must remain available on systems where vector extension loading is unavailable.
