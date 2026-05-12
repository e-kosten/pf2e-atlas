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

`adjacent_manifest_path` is reserved for a JSON manifest that can carry larger provenance, source file inventories, report locations, and non-runtime artifact references. Runtime startup validation requires only the relative path metadata. Manifest schema validation belongs to a later ingest slice that writes and validates the adjacent manifest as a real artifact.

## Runtime Table Families

The Rust artifact contract extends beyond `artifact_metadata` once Rust ingest starts writing artifacts. The current TypeScript index is the parity inventory, but the Rust artifact should treat table families as generated projections over typed ingest/domain models rather than as ad hoc row bags.

Required runtime table families for Rust-written artifacts are:

| Family | Tables | Purpose |
| --- | --- | --- |
| Artifact identity | `artifact_metadata` | Runtime contract, source identity, embedding identity, tokenizer/FTS contract, and adjacent manifest pointer. |
| Source packs | `packs` | Pack labels, document type, source paths, and record counts for display, filtering, and source parity. |
| Records | `records` | Canonical normalized record identity, derived `record_family`, presentation text, source facts, selected direct `system_*` projections, normalized price/time facts, variant facts, search visibility, generated search text projection, and raw JSON for parity/debugging. |
| Aliases and remaster links | `record_aliases`, `remaster_links` | Lookup aliases and explicit premaster-to-remaster record bridges extracted from remaster journals and migration aliases. |
| Filterable row projections | `record_traits`, actor/item/spell side tables; later `record_derived_tags` after the derived-tag redesign | Normalized rows for common filters, discovery, presentation, and search SQL. Multi-value filterable facts should have typed row projections instead of requiring runtime JSON parsing. |
| Metric rows and catalogs | `record_metrics`, `metric_key_catalog`, `metric_value_catalog` | Open-ended actor/item metrics in one physical row model with a `metric_domain` axis, plus generated discovery catalogs by record family, metric domain, and metric namespace. |
| Reference graph | `reference_edges` | Exact outgoing/backlink relationships for `linksTo`, `linkedFrom`, rule graph, rule context, and page navigation. |
| Lexical search | `records_fts` | SQLite FTS5 index over canonical record name and search text. |
| Embeddings and vector search | `embeddings`, `record_embeddings` | Reusable vector blobs with semantic input hashes plus sqlite-vec rows and vector-side filter projections. |

Rust writers may refine exact SQL column constraints from the current TypeScript schema, but they must preserve the runtime meaning of these table families until a parity report records an accepted difference.

## Table Design Rules

Rust artifacts should tighten the current TypeScript schema where doing so improves validation without changing intended behavior:

- `RecordKey` values are serialized as `pack:id`, but Rust runtime code should use a parsed `RecordKey` newtype.
- Boolean columns should either use SQLite `CHECK` constraints for `0`/`1` values or be rejected by row-loader validation.
- Closed vocabularies such as `record_family`, publication family, search visibility policy, metric value type, variant source, and source-backed type axes should be validated by row loaders and represented as enums or validated newtypes in Rust.
- `record_family` is an Atlas-derived product grouping from Foundry `document_type` plus record `type`. Raw source identity remains available separately as `foundry_document_type` and `foundry_record_type`.
- Direct Foundry `system.*` projections use `system_*` names. Atlas-derived fields stay unprefixed, such as `price_cp`, `activation_time_*`, and `duration_*`.
- Activation time is the normalized "how long it takes to do/cast/use this" field family. Spell or effect duration remains a separate normalized field family because it means how long the effect lasts after activation.
- Actor, item, and spell side tables hold stable non-metric filter and presentation data. Metrics remain in `record_metrics`; direct Foundry item projections keep `system_*` names rather than recreating TypeScript's split weapon/armor group fields.
- The Rust artifact does not have a generic `subcategory` scope. Former TypeScript subcategory use cases are represented as explicit metadata/filter axes when they come from source fields or useful collapsed signals, and as trait filters when they are entirely trait-derived.
- Open PF2E/provider-defined values such as traits, metric keys, pack names, and some metadata text values may remain strings, but their normalization owner must be explicit.
- Runtime behavior should prefer typed columns, side tables, and generated catalogs over `raw_json`. Persisted raw JSON is for parity, debugging, and future ingest analysis, not normal lookup/search/discovery execution.
- Filterable multi-value fields should have typed row projections or generated catalogs. JSON array columns can remain as compact presentation caches only when generated from the same typed source.
- Generated projections such as `records_fts`, `record_embeddings`, metric catalogs, and aliases must be written from typed source models and covered by row-count/key-coverage validation. Derived-tag rows are intentionally deferred until a later design pass.
- sqlite-vec sentinel values for nullable filter columns must stay hidden behind `atlas-index` vector projection helpers. Domain and search code should not observe sentinels as real metadata.
- Premaster-to-remaster bridges should be modeled as `remaster_links` or edition links, not as generic legacy compatibility. Canonical Rust lookup should be based on `RecordKey` and aliases, while record detail can expose explicit remaster bridge relationships.

## Artifact Validation Beyond Metadata

Metadata validation must remain available without loading `sqlite-vec`. Once Rust writes full runtime artifacts, startup validation should add a second validation layer for table contracts:

- required table presence by family
- vector table capability checks only when a command needs vector search
- source-signature comparison only when the caller supplies an expected source signature or equivalent already-computed value

Do not add a broad full-artifact validator by default. Recomputing source freshness, row-domain coverage, generated catalog coverage, and vector coverage can approach the cost and complexity of rebuilding or reloading the source corpus. Prefer lightweight startup validation plus targeted writer/parity tests for generated projections such as FTS rows, vector rows, metric catalogs, aliases, and remaster links.

## Extension Loading

Artifact metadata validation must not load vector extensions. Later search commands may require `sqlite-vec`, but contract validation must remain available on systems where vector extension loading is unavailable.
