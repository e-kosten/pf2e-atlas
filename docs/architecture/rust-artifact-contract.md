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
| `source_signature` | `foundry-pf2e:sha256:<digest>` for current source snapshots |
| `source_record_count` | positive integer count of loaded Foundry source records before Rust-generated records are added |
| `artifact_record_count` | positive integer count of rows in `records`, including generated records |
| `generated_record_count` | non-negative integer count of Rust-generated records included in `records` |
| `content_hash_algorithm` | `sha256` |
| `embedding_provider_family` | `onnx-mean-pooling` for the default BGE small embedding model |
| `embedding_model_id` | `BAAI/bge-small-en-v1.5` |
| `embedding_model_revision` | `main` until model artifact checksums are available |
| `embedding_tokenizer_id` | `BAAI/bge-small-en-v1.5` |
| `embedding_pooling` | `mean` |
| `embedding_normalization` | `l2` |
| `embedding_dimensions` | `384` |
| `embedding_dtype` | `f32` |
| `embedding_distance_metric` | `cosine` |
| `embedding_document_prefix` | empty string for BGE small |
| `embedding_query_prefix` | `Represent this sentence for searching relevant passages: ` |
| `fts_tokenizer` | `unicode61 remove_diacritics 2` |
| `adjacent_manifest_path` | relative path next to the SQLite artifact |

Embedding model decisions must be concentrated in an `atlas-embedding` model catalog rather than repeated as raw strings across ingest, index validation, and search. The catalog default is `bge-small-en-v1.5`, selected from the embedding comparison harness as the best quality/performance default for the Rust runtime; MiniLM remains an explicit catalog option for parity checks and older artifacts. Switching models requires rebuilding `document_embedding_cache`, `record_vector_index`, and embedding metadata, then rerunning search-quality validation; it should not require changing storage, validation, or search call sites outside the catalog/provider boundary unless the new model changes vector storage requirements such as dimensions or dtype. Runtime query orchestration lives in `atlas-search`, which composes `atlas-embedding` with validated index handles rather than loading models or opening SQLite connections in CLI code.

The Rust embedding provider uses the `ort` crate with a pinned ONNX Runtime crate version plus the real `tokenizers` tokenizer JSON from the prepared model cache. This keeps model execution in-process and avoids introducing a sidecar runtime. Distribution packaging should keep the native ONNX Runtime dependency explicit and pinned; a later product packaging pass may decide whether to vendor the native library, rely on the crate-managed artifact, or provide a platform-specific install step.

Document embedding input construction and document vector generation are owned by `atlas-embedding` so ingest, refresh, validation, and query tooling use the same renderer and provider. Rust ingest builds a `RecordPresentationDocument` for each eligible normalized record through `atlas-record`, renders it through the compact embedding text renderer in `atlas-embedding`, appends lookup aliases as a late embedding-only recall signal, and computes the semantic input hash as SHA-256 over the exact generated input text. The model catalog declares each provider's ONNX-safe input token limit; tokenizers are configured from that catalog so documents and queries are truncated before model execution. Document embedding text is rendered as ordered section chunks from strongest identity and user-facing semantic signals toward lower-priority relationship recall signals. Ingest analyzes the full candidate input, applies the active tokenizer's budget to only over-limit documents, drops or word-trims lower-priority chunks by section, and recomputes the semantic input hash from the budgeted text before vector reuse or generation. Ingest reports document tokenization telemetry, including observed max tokens, tokens over the configured limit, and section-level truncation counts, for comparison harness lossiness analysis.

Ingest prepares embedding units only for default-visible searchable records; hidden provenance rows and remaster-hidden legacy rows are excluded from vector work. Every eligible record has exactly one `parent` unit keyed as `{record_key}#parent`. Raw Foundry description markup may also produce deterministic child units for long documents with useful structure. The splitter first keeps compact descriptions parent-only, then chooses the shallowest useful `h1`/`h2`/`h3` section level for long documents, and only falls through to repeated titled options in lists or paragraphs when an overlong structural leaf still needs deeper segmentation. The active splitter does not emit activation-specific child units, does not infer summaries, does not split on bare strong-label mechanics, and renders table content conservatively by retaining title/header text while dropping body cells. Generated vectors carry `embedding_unit_key`, parent `record_key`, unit kind, optional unit label, ordinal, semantic input hash, vector dimensions, and the vector payload into `document_embedding_cache`. The sqlite-vec `record_vector_index` stores only the vector payload keyed by rowid; its rowid matches the corresponding `document_embedding_cache` rowid so normal SQLite metadata remains the authoritative embedding-unit mapping. Runtime semantic search defaults to `weighted-chunks`: child units may recover records from long-document sections, but unit-kind weighting is applied before collapsing hits by parent `record_key`. User-facing results remain records while diagnostics may expose the matched unit.

Rust artifact rebuilds reuse existing `document_embedding_cache` rows by default when the output artifact already exists, embedding identity metadata matches the active model catalog, and the row's `semantic_input_hash` and dimensions match the newly prepared input. The build command exposes an explicit opt-out for full regeneration. Reuse is limited to the normal SQLite cache table; `record_vector_index` is still rebuilt from `document_embedding_cache` through the vector capability path.

The Rust CLI resolves source, embedding-cache, and index paths through one shared path resolver. In `auto` mode, it uses repo-local paths only when `git rev-parse --show-toplevel` succeeds and the git root contains `rust/Cargo.toml` plus `rust/crates/atlas-cli/Cargo.toml`; otherwise it uses platform user cache paths under a `pf2e-atlas` directory. `repo` mode requires that git-backed workspace detection, and `user` mode always uses platform cache paths. Direct path flags override the resolver for the command that receives them. `atlas setup` reports the resolved paths and can clone or fast-forward the PF2E source with `--fetch-source`; embedding model downloads remain a separate model-registry follow-up.

Long-running Rust CLI index operations initialize `tracing` and emit progress events to stderr. JSON command output remains reserved for the command result on stdout, so automation can parse reports without stripping progress text.

## Validation Families

Validation diagnostics are grouped by contract family:

- `contract`: artifact contract version.
- `schema`: SQLite schema version.
- `source`: Foundry source kind, source signature, record count, and content hashing.
- `data`: required row relationships, generated catalogs, and side-table coherence.
- `embedding`: query/document vector compatibility.
- `fts`: lexical tokenizer compatibility.
- `manifest`: adjacent artifact manifest linkage.

`atlas index validate --json` returns stable JSON diagnostics for missing or incompatible metadata and for Rust artifact table/data contract violations. Current TypeScript-built indexes that only expose the legacy `metadata` table are intentionally reported as missing the Rust artifact contract.

## Adjacent Manifest

`adjacent_manifest_path` is reserved for a JSON manifest that can carry larger provenance, source file inventories, report locations, and non-runtime artifact references. Runtime startup validation requires only the relative path metadata. Manifest schema validation belongs to a later ingest slice that writes and validates the adjacent manifest as a real artifact.

## Source Signature

Rust ingest computes `source_signature` from the loaded Foundry source inputs before source-backed generated records are added. The digest includes the manifest path and content hash, loaded pack declarations and record counts, each loaded source record's relative source path and raw JSON content hash, and skipped source record paths with skip reasons. It excludes absolute local paths, output artifact paths, generated records, timestamps, and other machine-local build facts.

Generated records, enrichment projections, and side-table rows are validated as artifact/data coherence rather than source freshness. This keeps the source signature stable for identical Foundry inputs while allowing Rust-owned projection policy to be checked by artifact validation and parity reports.

## Rust Record And Schema Owners

The Rust workspace keeps storage-agnostic normalized record DTOs in `atlas-record`. Ingest builds `NormalizedRecord` values from Foundry source data, artifact writers serialize the durable subset into SQLite table families, and index/runtime readers deserialize SQLite rows into `PersistedRecord` and `PersistedRecordSet` values or deliberately narrower views derived from them. `atlas-record` also owns the renderer-neutral `RecordPresentationDocument` contract: title, identity facts, badges, ordered section union, fact/prose/relationship blocks, and relationship targets. Family-specific projection recipes in `atlas-record` assign normalized record facts to that shared sectioned document. `atlas-embedding` owns the compact embedding text renderer over `RecordPresentationDocument`, including section priority and omission policy, while future TUI renderers should build on the same presentation document rather than inventing separate record-page shapes. Ingest-only construction state, such as unresolved reference candidates, does not belong to the persisted read DTO. `atlas-record` depends on `atlas-domain` primitives and must not own SQLite table names, column names, DDL, validation diagnostics, CLI envelopes, or source JSON parser structs.

Physical SQLite table and column ownership lives in `atlas-artifact`. Artifact metadata keys and expected artifact contract values also live in `atlas-artifact`. Record table write and read SQL must be built from the ordered column descriptors in `atlas-artifact`; ingest and index code own directional DTO mapping, not independent record column inventories. Artifact validation report DTOs live with the validator in `atlas-index`. `atlas-domain` owns semantic vocabulary and shared request/output contracts; it must not own SQLite DDL, table inventories, artifact metadata inventories, storage DTOs, or validation-context DTOs. Writers and validators import the shared artifact schema descriptors from `atlas-artifact` instead of maintaining independent table or column lists.

## Runtime Table Families

The Rust artifact contract extends beyond `artifact_metadata` once Rust ingest starts writing artifacts. The current TypeScript index is the parity inventory, but the Rust artifact should treat table families as generated projections over typed ingest/domain models rather than as ad hoc row bags.

Required runtime table families for Rust-written artifacts are:

| Family | Tables | Purpose |
| --- | --- | --- |
| Artifact identity | `artifact_metadata` | Runtime contract, source identity, embedding identity, tokenizer/FTS contract, and adjacent manifest pointer. |
| Source packs | `packs` | Pack labels, document type, source paths, and record counts for display, filtering, and source parity. |
| Records | `records` | Canonical normalized record identity, derived `record_family`, presentation text, source facts, selected direct `system_*` projections, normalized price/time facts, variant facts, default retrieval visibility, generated search text projection, and raw JSON for parity/debugging. |
| Generated source-backed records | `records`, `reference_edges` | Derived affliction canonicals and hidden affliction instances generated from staged action, consumable, and spell records. Canonical records are default-visible; instance records are retained for provenance and graph navigation. |
| Aliases and remaster links | `record_aliases`, `remaster_links` | Lookup aliases and explicit premaster-to-remaster record bridges extracted from remaster journals and migration aliases. `record_aliases` stores `canonical_record_key`, alias text, normalized alias text, `source_kind`, and `source_ref`; `remaster_links` stores `remaster_record_key`, `legacy_record_key`, `source_kind`, and `source_ref`. Source and presentation metadata is derived by joining to `records`. |
| Filterable row projections | `record_traits`, actor/item/spell side tables; later `record_derived_tags` after the derived-tag redesign | Normalized rows for common filters, discovery, presentation, and search SQL. Multi-value filterable facts should have typed row projections instead of requiring runtime JSON parsing. |
| Metric rows and catalogs | `record_metrics`, `metric_key_catalog`, `metric_value_catalog` | Open-ended actor/item metrics in one physical row model with a `metric_domain` axis, plus generated discovery catalogs by record family, metric domain, and metric namespace. |
| Reference graph | `reference_edges` | Exact outgoing/backlink relationships for `linksTo`, `linkedFrom`, rule graph, rule context, and page navigation. Edges store `from_record_key`, `to_record_key`, optional authored link `display_text`, and the exact `reference_text`; pack/type/source metadata is derived by joining to `records`. |
| Lexical search | `records_fts` | SQLite FTS5 index over default-visible canonical record name and search text. |
| Document embedding cache | `document_embedding_cache` | Durable reusable embedding unit vector blobs keyed by `embedding_unit_key`, with parent `record_key`, unit kind, optional label, ordinal, semantic input hashes, and vector dimensions for reuse, validation, debugging, sqlite-vec rowid mapping, and optional non-sqlite-vec scoring paths. Rows are normal SQLite and can be written before sqlite-vec is available. |
| Vector search index | `record_vector_index` | Lightweight sqlite-vec KNN table over default-visible embedding units. It stores only the embedding vector and uses rowid as the physical key matching `document_embedding_cache.rowid`; it does not own user-facing filter semantics or duplicate text metadata. This table is created, populated from `document_embedding_cache`, and validated through an explicit sqlite-vec capability path, not through the base plain-SQLite schema. |

Rust writers may refine exact SQL column constraints from the current TypeScript schema, but they must preserve the runtime meaning of these table families until a parity report records an accepted difference.

## Table Design Rules

Rust artifacts should tighten the current TypeScript schema where doing so improves validation without changing intended behavior:

- `RecordKey` values are serialized as `pack:id`, but Rust runtime code should use a parsed `RecordKey` newtype.
- Boolean columns should either use SQLite `CHECK` constraints for `0`/`1` values or be rejected by row-loader validation.
- Closed vocabularies such as `record_family`, publication family, default visibility policy, metric value type, variant source, and source-backed type axes should be validated by row loaders and represented as enums or validated newtypes in Rust.
- `record_family` is an Atlas-derived product grouping from Foundry `document_type` plus record `type`. Raw source identity remains available separately as `foundry_document_type` and `foundry_record_type`.
- Generated afflictions use `record_family = affliction`, `foundry_document_type = Item`, and generated `foundry_record_type` values `affliction` or `affliction-instance`. Instances are not default-visible and exist to preserve the source host relationship.
- Direct Foundry `system.*` projections use `system_*` names. Atlas-derived fields stay unprefixed, such as `price_cp`, `activation_time_*`, and `duration_*`.
- `is_default_visible` is a generated retrieval policy projection. It controls default user-facing search, browse, and list surfaces; records with false remain present for direct key lookup, references, remaster links, and inspection. It is not a source-authored fact.
- Activation time is the normalized "how long it takes to do/cast/use this" field family. Spell or effect duration remains a separate normalized field family because it means how long the effect lasts after activation.
- Actor, item, and spell side tables hold stable non-metric filter and presentation data. Metrics remain in `record_metrics`; direct Foundry item projections keep `system_*` names rather than recreating TypeScript's split weapon/armor group fields.
- The Rust artifact does not have a generic `subcategory` scope. Former TypeScript subcategory use cases are represented as explicit metadata/filter axes when they come from source fields or useful collapsed signals, and as trait filters when they are entirely trait-derived.
- Open PF2E/provider-defined values such as traits, metric keys, pack names, and some metadata text values may remain strings, but their normalization owner must be explicit.
- Runtime behavior should prefer typed columns, side tables, and generated catalogs over `raw_json`. Persisted raw JSON is for parity, debugging, and future ingest analysis, not normal lookup/search/discovery execution.
- Filterable multi-value fields should have typed row projections or generated catalogs. JSON array columns can remain as compact presentation caches only when generated from the same typed source.
- Generated projections such as `records_fts`, `record_vector_index`, metric catalogs, and aliases must be written from typed source models and covered by row-count/key-coverage validation. Derived-tag rows are intentionally deferred until a later design pass.
- `record_vector_index` is a query index, not a duplicated filter store. The Rust vector table should stay to rowid plus vector only. Do not add `record_key`, `embedding_unit_key`, `record_family`, trait, level, rarity, metric, source, or other filter projection columns as part of the baseline. If performance testing later proves a vec metadata or partition column is needed, treat it as an accelerator generated from authoritative rows and add validation that it cannot drift.
- Semantic search with filters uses authoritative SQL keyset prefiltering. The shared filter compiler lives in `atlas-index` and lowers the canonical `SearchFilterNode` tree into a parameterized eligible-record relation from `records` and side tables. Browse, lexical, lookup narrowing, and vector search compose from that same eligible relation; vector search should constrain sqlite-vec units by `record_vector_index.rowid` using eligible `document_embedding_cache.rowid` values and collapse matching units by `record_key`. Do not implement exact filtered semantic search by joining ordinary filter tables around the vec scan alone, because joined predicates can be applied after the vec top-k result set.
- The `atlas-index` vector query boundary accepts an already-computed query vector and optional `SearchFilterNode`. Text-to-query-vector embedding remains owned by `atlas-embedding` and is called by `atlas-search` before it invokes vector-index SQL. CLI, future TUI, and future MCP surfaces should consume `atlas-search` for semantic search instead of opening SQLite connections or embedding models directly.
- Runtime semantic search groups sqlite-vec unit hits by parent record before returning user-facing records. Search-time semantic modes determine which embedding units participate: `parent-only` restricts retrieval to parent units, `chunks` allows parent and child units to compete on raw vector distance, and `weighted-chunks` allows child units to recover records while applying unit-kind rank-distance adjustments before parent-record collapse. Diagnostics may expose both raw vector distance and adjusted rank distance.
- Filters that cannot compile to an authoritative SQL keyset are errors for the first Rust baseline. Approximate overfetch-and-post-filter behavior is not part of the default contract.
- sqlite-vec sentinel values for nullable filter columns should not be needed in the baseline because the vector table has no filter projection columns. If future performance accelerators add vec metadata columns, their sentinels must stay hidden behind `atlas-index` vector projection helpers. Domain and search code should not observe sentinels as real metadata.
- Premaster-to-remaster bridges should be modeled as `remaster_links` or edition links, not as generic legacy compatibility. Canonical Rust lookup should be based on `RecordKey` and aliases, while record detail can expose explicit remaster bridge relationships.

## Artifact Validation Beyond Metadata

Metadata validation must remain available without loading `sqlite-vec`. For Rust-written SQLite artifacts, `atlas index validate` also validates the lightweight runtime table contract:

- required runtime table presence
- required column presence for the current artifact schema
- `artifact_record_count` agreement with `records`
- `source_record_count` plus `generated_record_count` agreement with `artifact_record_count`
- SQLite foreign-key integrity plus explicit relationship orphan checks
- boolean integer columns constrained to `0`/`1`
- metric value columns matching `value_type`
- `records_fts` key coverage exactly matching default-visible records
- remaster link visibility policy: remaster-side records are default-visible and legacy-side records are not
- metric key/value catalogs matching metrics emitted for default-visible records
- non-empty `document_embedding_cache` parent-record coverage exactly matching default-visible records that are eligible for semantic search, with one or more embedding-unit rows per eligible record
- `record_vector_index` rowid coverage exactly matching `document_embedding_cache` once the sqlite-vec index is present
- vector dimensions matching the embedding identity metadata

Validation should stay bounded to SQLite runtime coherence. Source freshness comparison belongs to callers that supply an expected source signature or equivalent already-computed value. Vector table capability checks belong to commands that need vector search. Recomputing source-derived assignment quality, semantic coverage, or full parity against the Foundry corpus remains outside startup validation.

## Extension Loading

Artifact metadata validation must not load vector extensions. Vector-specific validation and search commands use an explicit sqlite-vec capability boundary and report `vector_extension_unavailable` when the extension cannot be loaded or probed. Plain artifact validation remains available on systems where vector extension loading is unavailable.

Rust sqlite-vec registration is isolated in `atlas-sqlite-vec`, a small crate that owns the required SQLite FFI call. Runtime crates such as `atlas-index`, `atlas-search`, and `atlas-cli` use that capability boundary instead of embedding unsafe extension-registration code locally.
