# Artifact Contract

This document defines the runtime artifact boundary for PF2e Atlas. The Rust runtime opens prepared SQLite artifacts read-only for lookup and search commands. Setup and index diagnostic commands own metadata validation, vector readiness checks, and deep artifact coherence checks.

For the broader crate and data-flow architecture around this artifact contract, see [runtime architecture](./runtime.md).

Durable mutable local state is not part of this artifact contract. Saved lists and future user-authored or agent-authored local data live in the separate local-state database owned by `atlas-local-state`; artifact rebuilds do not copy, preserve, or validate those rows.

## Canonical Artifact Boundary

[ADR 0035](./decisions/0035-atomic-canonical-artifact.md) defines the source-faithful artifact boundary approved at Checkpoint B.

The C2 exhaustive-validation snapshot is outside this artifact contract. It is a
private, no-clobber, checksum- and identity-bound evidence directory that may
contain validation-produced artifacts and reports. No runtime reader accepts it,
no setup freshness path consults it, and it provides neither a fallback artifact
nor a second serializable canonical record model.

The target stores canonical entities, contextual occurrences, owned content, exact reference occurrences, and derived query projections with stable identities. Product-addressable activities, spellcasting entries, resources, content, and occurrences are relational. Nested mechanics normally consumed with one entity may use deterministic Atlas-owned typed JSON; independently queryable/filterable/joinable facts require authoritative relational projections. Complete canonical hydration belongs only to `atlas-index::read`.

B5 supplies the storage-neutral creature owned-content input to the C1 boundary:
stable composite content identity, typed owner/role/origin/visibility/provenance, authored order,
canonical `RichDocument`, content hash, duplicate-source status, diagnostics, and exact ordered
reference occurrences. Schema v2 persists and hydrates that model atomically. Migrated creature metric rows and categorical
side facts are already one-way projections from `CreatureRecord`, and current display/FTS inputs
consume those same projections; C1 persists them without reinterpreting raw source. D1 remains
responsible for later search projection and duplicate-ranking policy.

The v2 writer publishes only after the completed temporary SQLite artifact passes deep validation. `atlas-index::read` decodes the canonical typed body and exposes canonical-body and combined hydrated-record reads; no other crate owns complete hydration. Old artifacts receive explicit rebuild guidance and are never upgraded in place.

Typed visibility/provenance and product retrieval disposition are separate. Atlas has no authentication boundary, but the pinned base still uses `is_default_visible` and public-only content/reference predicates across FTS, embeddings, graph, discovery/metrics, and validation, so it is not GM-complete. Checkpoint A's target makes useful authored information eligible regardless of classification. Classification makes no current privacy/security claim. Every retained excluded record, content unit, reference, FTS row, or embedding unit needs a non-auth product rationale, owner, fixture, validation, and audit checkpoint.

## Contract Version

The first supported contract version is:

```text
pf2e-atlas-artifact/v2
```

The SQLite schema version for this active development artifact family is:

```text
2
```

These values are stored in `artifact_metadata` as `artifact_contract_version` and `schema_version`. Setup and index diagnostic commands must fail validation when they do not support either value. Normal lookup and search commands open the resolved artifact read-only and report operational failures from the requested action instead of running validation as a command preflight.

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
| `artifact_contract_version` | `pf2e-atlas-artifact/v2` |
| `schema_version` | `2` |
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
| `embedding_unit_policy_version` | `coverage-driven-rich-content/v1` |
| `fts_tokenizer` | `unicode61 remove_diacritics 2` |
| `adjacent_manifest_path` | relative path next to the SQLite artifact |

Embedding model decisions must be concentrated in an `atlas-embedding` model catalog rather than repeated as raw strings across ingest, index validation, and search. The catalog default is `bge-small-en-v1.5`, selected from the embedding comparison harness as the best quality/performance default for the Rust runtime; MiniLM remains an explicit catalog option for parity checks and older artifacts. Switching models requires rebuilding `document_embedding_cache`, `record_vector_index`, and embedding metadata, then rerunning search-quality validation; it should not require changing storage, validation, or search call sites outside the catalog/provider boundary unless the new model changes vector storage requirements such as dimensions or dtype. Runtime retrieval opens the embedding model recorded in artifact metadata; product search callers do not choose an embedding model after the artifact is built. Runtime query orchestration lives in `atlas-search`, which composes `atlas-embedding` with read-only index handles rather than loading models or opening SQLite connections in CLI code.

The Rust embedding provider uses the `ort` crate with a pinned ONNX Runtime crate version plus the real `tokenizers` tokenizer JSON from the prepared model cache. This keeps model execution in-process and avoids introducing a sidecar runtime. Distribution packaging should keep the native ONNX Runtime dependency explicit and pinned; a later product packaging pass may decide whether to vendor the native library, rely on the crate-managed artifact, or provide a platform-specific install step.

Document embedding input construction and document vector generation are owned by `atlas-embedding` so ingest, refresh, validation, and query tooling use the same renderer and provider. Rust ingest builds a `RecordPresentationDocument` for each eligible normalized record through `atlas-record`, renders it through the compact embedding text renderer in `atlas-embedding`, appends lookup aliases as a late embedding-only recall signal, and computes the semantic input hash as SHA-256 over the exact generated input text. The embedding-unit kind vocabulary is owned by `atlas-embedding`; ingest writes it, index validation parses it, and search ranks over that shared typed vocabulary instead of repeating string allowlists. The model catalog declares each provider's ONNX-safe input token limit; tokenizers are configured from that catalog so documents and queries are truncated before model execution. Document embedding text is rendered as ordered section chunks from strongest identity and user-facing semantic signals toward lower-priority relationship recall signals. Ingest analyzes the full candidate input, applies the active tokenizer's budget to only over-limit documents, drops or word-trims lower-priority chunks by section, and recomputes the semantic input hash from the budgeted text before vector reuse or generation. Ingest reports document tokenization telemetry, including observed max tokens, tokens over the configured limit, and section-level truncation counts, for comparison harness lossiness analysis.

Ingest prepares embedding units for product-relevant searchable records under an explicit retrieval disposition; typed visibility does not act as an authorization filter. Provenance-only or tooling rows may be excluded only with a documented non-auth product rationale and audit evidence. Every eligible record has exactly one `parent` unit keyed as `{record_key}#parent`. Parent units render the canonical presentation projection from eligible record-owned content but exclude copied embedded item/spell supplemental content so duplicate capabilities do not rank as ordinary parent description text. Rich content documents are split into stable coverage groups from leading text, explicit heading sections, and whole-document fallback groups. Synthetic sections created from strong-leading paragraphs or table captions remain folded into their containing text, and copied embedded capability content does not produce independent default child embedding units. Child units remain overflow-only: they are materialized only when tokenizer budgeting trims or drops a rich group from the parent and only for child-worthy rich sources. When a child is needed, it is built from the whole impacted rich group and independently budgeted. Broader child indexing requires a separate measured approval. The active unit kinds and policy version remain typed artifact metadata. Runtime semantic search continues to collapse units by parent record while retaining matched content/section targets for explanation and navigation.

Rust artifact rebuilds reuse existing `document_embedding_cache` rows by default when the output artifact already exists, embedding identity metadata matches the active model catalog, and the row's `semantic_input_hash` and dimensions match the newly prepared input. The build command exposes an explicit opt-out for full regeneration. Reuse is limited to the normal SQLite cache table; normal embedding-enabled ingest creates and populates `record_vector_index` from `document_embedding_cache` before publishing the artifact.

`atlas-runtime` resolves source, embedding-cache, and index paths for CLI and future Rust surfaces. `global` mode is the default and uses platform cache paths under a `pf2e-atlas` directory so first-run setup and later installed-CLI commands resolve the same data from any working directory. `repo` mode is an explicit contributor mode: it requires `git rev-parse --show-toplevel` to find a git root containing `Cargo.toml` plus `crates/atlas-cli/Cargo.toml`, then uses that checkout's `vendor/pf2e` and `.cache` paths.

Direct path flags are command-local overrides rather than persisted configuration. `--index` is the SQLite artifact override for commands that open or repair an artifact, including `setup`, record, search, filter discovery, validation, and inspection commands. `atlas index build` uses `--output` for the artifact it writes, while `setup --index` names the artifact target that setup should repair or build. `--source` and `--embedding-cache-path` override the source checkout and embedding model cache for the command that receives them. A future persisted config file, if added, should feed `AtlasPathOverrides` before CLI direct flags are applied; CLI direct flags should remain the highest-precedence path source.

`atlas setup` is the first-run install/repair path: it fetches or updates PF2E source by default, prepares embedding model cache files for full setup, builds or repairs the selected artifact target, and checks the final readiness state. `atlas setup clean` removes selected resolved runtime data without uninstalling the CLI: artifact cleanup removes the SQLite artifact plus WAL/SHM companions, embedding cleanup removes the embedding cache root, and source cleanup removes the PF2E source checkout. Setup emits progress events to stderr for long-running phases while preserving JSON command results on stdout. `--offline` prevents network-backed source or model preparation, `--check` reports planned or blocked actions without writing, and `--no-embeddings` selects base record-readiness rather than full semantic readiness. Setup uses the adjacent artifact manifest's source position to avoid recomputing the full source signature when the source state clearly matches the artifact; it falls back to full source analysis when the manifest is missing, inconsistent with artifact metadata, lacks a source position, or the current source position differs. `atlas-cli` owns argument parsing, output formatting, progress presentation, and exit codes rather than durable path or setup policy.

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

`atlas index check --json` returns stable JSON diagnostics for fast runtime readiness: supported metadata, required runtime tables, and vector capability for the default full target. `atlas index validate --json` runs deeper artifact diagnostics for artifact table/data contract violations. Indexes without the `artifact_metadata` table are reported as missing the artifact contract.

## Adjacent Manifest

`adjacent_manifest_path` points to the JSON artifact manifest written beside the SQLite artifact. The current manifest version is `pf2e-atlas-artifact-manifest/v2`. The manifest records artifact contract/schema identity, source kind, source root display path, full `source_signature`, source record count, artifact/generated record counts, document embedding count, selected embedding model, the SQLite artifact SHA-256, and the source position used for setup freshness. The manifest is mandatory: absence, an unsupported manifest contract, or a digest mismatch is an unavailable split or stale pair and requires a complete rebuild. A hard link or copy of SQLite without its bound adjacent manifest is not an artifact publication unit.

The source position is an invalidation hint for setup freshness, not a replacement for the full source signature. For git source checkouts, the manifest records the PF2E checkout `git_commit` and setup compares it to the current `HEAD`. For non-git source roots, the source position uses the source manifest content plus pack JSON file paths, sizes, and mtimes; when it differs from the manifest, setup falls back to full source analysis and compares the authoritative source signature.

Runtime artifact validation requires the relative `adjacent_manifest_path` metadata, a manifest-bound matching SQLite digest, and SQLite coherence. Reader acquisition takes a shared lock on the persistent pair-coordination file, opens the visible SQLite file once, hashes that open file, and materializes or verifies an adjacent immutable generation snapshot named by that digest. Every Diesel and validation/hydration SQLite connection opens that generation snapshot while the lock still protects the verified visible pair. The reader then releases the shared lock and retains the snapshot connections and file handle, so a publisher may install a new visible pair while long-lived readers remain coherent on the old generation. No reader reopens the visible pathname or combines connections from different generations. Setup owns freshness policy by reading the adjacent manifest, verifying that its source and record counts match artifact metadata, comparing the current source position, and deciding whether full source analysis is needed before planning a rebuild.

Builds stage and syncs the complete SQLite file and manifest before publication. Publishers for the same adjacent-manifest target serialize through an exclusive OS lock whose kernel ownership is released after process failure; the coordination file itself remains stable so deleting and recreating it cannot split lock domains. Exclusive acquisition has a five-second deadline and returns actionable retry guidance instead of blocking indefinitely. Under that lock, publication verifies the staged digest, recovers or cleans fixed transaction backups from an interrupted attempt, snapshots a prior matching pair, installs both files, re-verifies the final digest, and removes recovery state. Failure before commit restores both prior files or removes both first-publication files. Readers share the lock only during generation acquisition, never for the long-lived reader or service lifetime. Completed publication removes obsolete generation snapshots when no live operating-system handle prevents deletion; a final old-generation reader also removes its obsolete snapshot on drop. Orphan generation and temporary files from an interrupted attempt are cleaned by the next successful publication, and stale staging files are never selected as visible state.

## Source Signature

Rust ingest computes `source_signature` from the raw Foundry source snapshot before canonical, DTO, content, or enrichment projection can accept or reject a record and before source-backed generated records are added. The digest includes the manifest's relative path and content hash, loaded localization files such as `static/lang/en.json` by relative path and content hash, declared pack metadata and raw record counts, and every readable declared record's stable relative path and raw content hash. A raw file that cannot be read prevents a complete snapshot signature; parse or projection failures remain diagnostics but do not remove that file's raw identity from the digest.

Projection outcomes, skip classifications, diagnostic messages, absolute paths, output artifact paths, generated records, timestamps, and other machine-local build facts are excluded from signed material. Diagnostic reports may retain actionable display text, but the v1 signature never hashes arbitrary `Display` output; identical declared inputs at different source roots therefore produce the same signature.

Generated records, enrichment projections, and side-table rows are validated as artifact/data coherence rather than source freshness. This keeps the source signature stable for identical Foundry inputs while allowing Rust-owned projection policy to be checked by artifact validation and parity reports.

## Rust Record And Schema Owners

The Rust workspace keeps storage-agnostic normalized record DTOs in `atlas-record`. Ingest builds `AtlasRecord` values from Foundry source data, artifact writers serialize the durable subset into SQLite table families, and index/runtime readers deserialize SQLite rows into `AtlasRecord` and `AtlasRecordSet` values or deliberately narrower views derived from them. `atlas-record` also owns the canonical `RichDocument` model for authored rich text, including plain-text and markdown-like renderers, reference traversal, structured FTS projection, and section-tree projection. `AtlasRecord` carries an ordered set of `RecordContentDocument` values rather than stripped text fields. Foundry `@Localize[...]` macros preserve their authored localization key and optional authored label in `RichDocument`; ingest resolves English localization values from the source localization catalog and stores the parsed resolved rich content beside that macro context. Raw Foundry markup belongs to ingest parsing and raw JSON provenance, not to runtime presentation, search, reference extraction, or embedding construction.

`atlas-record` also owns the renderer-neutral `RecordPresentationDocument` contract: title, identity facts, badges, ordered section union, fact/prose/relationship blocks, and relationship targets. Family-specific projection recipes in `atlas-record` assign normalized record facts to that shared sectioned document. `atlas-embedding` owns compact embedding text rendering, token budgeting, and embedding-unit selection over presentation/content inputs. It consumes the `atlas-record` section tree for child units and does not parse raw Foundry markup. Ingest-only construction state is carried by ingest-owned source-load DTOs, not by `AtlasRecord`; content parse diagnostics, source slugs, compendium-source locators, embedded item facts, and journal page facts are loader facts, while resolved record targets are stored in `RichDocument` `FoundryLink` nodes and written as source-tagged reference edges/occurrences. Post-normalization ingest stages consume typed construction facts when building aliases, remaster links, and source-backed generated records; reference, FTS, and embedding projections consume the normalized `RichDocument` outputs produced during normalization. Persisted `raw_json` remains available for provenance, parity debugging, and deliberate future source-audit tooling, but normal ingest projection code should not parse it after normalization. `atlas-record` depends on `atlas-domain` primitives and must not own SQLite table names, column names, DDL, validation diagnostics, CLI envelopes, or source JSON parser structs.

`atlas-ingest` is organized by ingest phase. The crate root is a thin public facade. Top-level artifact build orchestration lives in `build`, source loading and normalization live under `source`, record enrichment lives under `records`, source-backed generated records live under `generated`, and ingest-side embedding execution lives under `embeddings`. Ingest does not own database-specific artifact writers; after enrichment and embedding generation it consumes ingest-only state into an owned `atlas-index::IndexBuildInput` and invokes configured `atlas-index::IndexArtifactWriter` implementations. `IndexBuildInput` is the write-side artifact package, while `AtlasRecord` remains the read-side hydration DTO for completed artifacts. Crate-internal modules should import from those owner subtrees instead of using `lib.rs` as a crate-private barrel. Embedding policy remains in `atlas-embedding`; ingest-side embedding code only decides whether to run embeddings for this artifact build, handles cache reuse, records timing, and passes prepared source data to embedding-owned APIs. The ingest public build API accepts embedding model identifiers as strings and returns ingest-owned report DTOs so `atlas-embedding` remains the only crate exposing embedding-specific public types.

Physical SQLite table and column ownership lives in `atlas-index`. Diesel migration files under `crates/atlas-index/migrations/` are the physical schema source of truth. Artifact creation embeds the migration `up.sql` rather than maintaining separate DDL, and the checked-in Diesel `schema.rs` is validated against that migration-created schema in the atlas-index test suite. Diesel schema declarations, writer DTOs, and stable reader hydration should follow those migrations. `atlas-index` also owns artifact metadata keys, expected artifact contract values, narrow validation requirements for external artifact files, runtime validation semantics, filter discovery field policy, SQLite discovery extractor rendering, and SQLite `f32` vector blob encoding. Raw SQL remains an index-owned implementation detail for SQLite facilities that do not fit Diesel's typed query model: FTS5 virtual tables, sqlite-vec virtual tables, dynamic filter/discovery relations, and SQLite validation pragmas. Raw-SQL helpers may keep stable table and column name constants, but they must not recreate a parallel schema descriptor model for Diesel-owned tables. Artifact validation report DTOs live with the validator in `atlas-index`. `atlas-domain` owns semantic vocabulary and shared request/output contracts; it must not own SQLite DDL, table inventories, artifact metadata inventories, storage DTOs, filter discovery extractor SQL, or validation-context DTOs.

Within `atlas-index`, physical layout follows ownership boundaries rather than storage technology alone. `artifact` owns artifact metadata identity, schema migration embedding, vector blob storage, validation inventory, and artifact coherence validation. `read` owns record hydration, search traits, filter compilation, FTS, vector query SQL, graph/reference reads, filter discovery, and read-side validation/inspection methods. `write` owns index build input, retrieval visibility policy, and SQLite artifact writing. `sqlite` owns connection lifecycle, read-only opening, raw SQL binding primitives, and public SQLite reader/writer facade re-exports. `SqliteIndexReader` should remain a connection shell; read behavior belongs in the `read` submodule that owns the concern. `SqliteIndexWriter` should remain an artifact-write orchestrator; table-specific writes and output publishing belong in focused `write::sqlite` children.

The normalized metric definition catalog lives in `atlas-record::metrics`. `MetricRow` remains open-ended storage-agnostic record data, while definitions describe known metric domains, keys, value types, namespaces, labels, and groups. Static definitions cover exact keys such as actor armor class and item weapon damage; pattern definitions cover data-driven families such as skills, speeds, senses, and hazard disable metrics. Foundry JSON source paths are ingest-owned source specs that reference those shared metric definitions instead of restating metric identity. Static source specs map first-valid path candidates to exact metric definitions, while dynamic source specs map closed vocabularies, object entries, or array entries to pattern definitions using definition-owned key helpers. Computed and parser-backed metrics remain imperative until they share enough shape to justify source specs. Ingest validates emitted rows against the typed catalog during metric extraction, so emitted metric keys and value types stay aligned with the shared definitions. Search, discovery, and presentation may enrich known rows with labels from the definition catalog but must preserve unknown rows with raw-key fallback labels if a future ingestion policy explicitly permits provider-defined metric rows.

## Runtime Table Families

The Rust artifact contract extends beyond `artifact_metadata`. Artifact table families are generated projections over typed ingest/domain models rather than ad hoc row bags.

Required runtime table families for Rust-written artifacts are:

| Family | Tables | Purpose |
| --- | --- | --- |
| Artifact identity | `artifact_metadata` | Runtime contract, source identity, embedding identity, tokenizer/FTS contract, and adjacent manifest pointer. |
| Source packs | `packs` | Pack labels, document type, source paths, and record counts for display, filtering, and source parity. |
| Records | `records`, `canonical_creature_records` | Canonical normalized record identity, classification, publication, source path/provenance, kind-specific facts, stable target identities, retrieval disposition, and raw JSON for parity/debugging. Authored rich content is not stored on `records`; raw JSON is not a hydration fallback. Every v2 NPC requires one deterministic canonical creature body, and non-NPC rows cannot own one. |
| Rich content | `record_content`, `record_content_exclusions` | All authored `RichDocument` rows keyed by record, canonical `content_key`, and authored order, with typed owner, role, source kind, visibility, provenance, retrieval/reference disposition, optional label, and content JSON. Authored order is part of the relational locator so approved repeated Stage B source identities remain unchanged canonically without dropping rows. Under the approved target, useful authored content is available regardless of classification; every excluded implementation/provenance source requires a non-auth rationale. |
| Canonical mechanics entities | `canonical_creature_resources`, `canonical_creature_entities`, `canonical_creature_occurrences`, `canonical_creature_relationships` | Product-addressable canonical resources and entities, contextual occurrences, and non-executing grant/item-grant/linked-weapon/prepared-spell relationships with stable IDs, source order, authored order, owners, typed mechanics, locators, lifecycle provenance, and runtime target identities. Nested rolls/damage/modes may use deterministic Atlas-owned typed JSON when they are consumed with their entity. |
| Generated source-backed records | `records`, `reference_edges` | Derived affliction canonicals and source-instance occurrences generated from staged action, consumable, and spell records. Canonical records own user-facing meaning; instances remain addressable provenance/graph context and may be excluded from ordinary ranking to prevent duplicate canonicals, not because their visibility is hidden. |
| Aliases and remaster links | `record_aliases`, `remaster_links` | Lookup aliases and explicit premaster-to-remaster record bridges extracted from remaster journals and migration aliases. `record_aliases` stores `canonical_record_key`, alias text, normalized alias text, `source_kind`, and `source_ref`; `remaster_links` stores `remaster_record_key`, `legacy_record_key`, `source_kind`, and `source_ref`. Source and presentation metadata is derived by joining to `records`. |
| Filterable row projections | `record_traits`, actor/item/spell side tables; later `record_tags` after the tagging implementation | Normalized rows for common filters, discovery, presentation, and search SQL. Multi-value filterable facts should have typed row projections instead of requiring runtime JSON parsing. Tag rows will be written during regular `atlas index build` from validated YAML catalog and assignment files. |
| Filter discovery catalogs | `filter_field_catalog`, `filter_value_catalog`, `filter_sample_catalog`, `filter_numeric_catalog` | Generated field/value discovery catalogs for unfiltered and single-record-kind filter spaces. Field rows describe filterable field ids, groups, operators, CLI flag mappings, value policies, applicability, and value-shape statistics. Value rows store complete enumerable value counts, sample rows store deterministic representative text samples, and numeric rows store exact count/null/min/percentile/mean/max summaries. Richer filter spaces are computed dynamically by `atlas-index` over the canonical eligible-record relation. |
| Metric rows and catalogs | `record_metrics`, `metric_key_catalog`, `metric_value_catalog` | Open-ended actor/item metrics in one physical row model with a `metric_domain` axis, plus generated discovery catalogs for unfiltered and single-record-kind scopes by metric domain and metric namespace. |
| Reference graph | `reference_edges`, `reference_occurrences` | Exact outgoing/backlink relationships for graph context and page navigation. Aggregate edges and per-content occurrences preserve owner/content/section identity, source kind, visibility, provenance, relation kind, label, and target. The pinned base is public-only in default graph modes; the approved target removes classification-only suppression. Copied embedded capability edges may remain a separate mode for deduplication. |
| Lexical search | `records_fts` | SQLite FTS5 index over records eligible under explicit product retrieval dispositions. The active precision path indexes title/aliases plus structured high-signal fields such as traits, taxonomy, constraints, mechanics, source terms, and metric terms. Rich prose columns remain part of the table contract for future exploration but are not populated from `RichDocument` in the default V1 projection. |
| Document embedding cache | `document_embedding_cache` | Durable reusable embedding unit vector blobs keyed by `embedding_unit_key`, with parent `record_key`, unit kind, optional label, ordinal, semantic input hashes, and vector dimensions for reuse, validation, debugging, sqlite-vec rowid mapping, and optional non-sqlite-vec scoring paths. Rows are normal SQLite and can be written before sqlite-vec is available. |
| Vector search index | `record_vector_index` | Lightweight sqlite-vec KNN table over embedding units eligible under explicit product retrieval dispositions. It stores only the embedding vector and uses rowid as the physical key matching `document_embedding_cache.rowid`; it does not own user-facing filter semantics or duplicate text metadata. Normal embedding-enabled ingest creates and populates this table from `document_embedding_cache`; runtime validation and search access it through an explicit sqlite-vec capability path. |

Rust writers may refine exact SQL column constraints, but they must preserve the runtime meaning of these table families until an artifact-contract decision records an accepted difference.

`records_fts` structured-term columns are deterministic projections from normalized typed record fields, not parsed raw JSON. `taxonomy_terms` contains categorical identity fields such as record kind, Foundry record type, taxonomy families, item category/group/base item, spell kinds, and variant labels. `constraint_terms` contains user-facing preconditions and use gates such as prerequisites, activation/action-cost phrases, and hands requirements. `mechanic_terms` contains non-metric mechanical descriptors such as level/rank phrases, rarity, spell traditions/range/targets/area/save/damage-type flags, item usage/damage types, and actor mechanics sets. `source_terms` contains publication title/family and pack label, not pack name. `metric_terms` contains metric display labels and short labels for metric rows present on the record; numeric metric values stay out of FTS and remain filter/metric query data.

## Table Design Rules

Rust artifacts should keep schema constraints explicit where doing so improves validation without changing intended behavior:

- `RecordKey` values are serialized as `pack:id`, but Rust runtime code should use a parsed `RecordKey` newtype.
- Boolean columns should either use SQLite `CHECK` constraints for `0`/`1` values or be rejected by row-loader validation.
- Closed vocabularies such as `record_kind`, publication family, default visibility policy, metric value type, variant source, and source-backed type axes should be validated by row loaders and represented as enums or validated newtypes in Rust.
- `record_kind` is an Atlas-derived product grouping from Foundry `document_type` plus record `type`. Raw source identity remains available separately as `foundry_document_type` and `foundry_record_type`.
- Generated afflictions use `record_kind = affliction`, `foundry_document_type = Item`, and generated `foundry_record_type` values `affliction` or `affliction-instance`. The pinned base incorrectly overloads default visibility as canonical-versus-source-instance role. The target persists an explicit checked role and typed host-instance-canonical relationships: canonicals are ordinary-ranked, while source instances preserve exact host occurrence/provenance and remain directly addressable and graph-reachable but outside ordinary ranking to prevent duplicate canonical results. Role, pack/count routing, and this non-auth disposition must not be inferred from visibility.
- Direct Foundry `system.*` projections use `system_*` names. Atlas-derived fields stay unprefixed, such as `price_cp`, `activation_time_*`, and `duration_*`.
- Current `is_default_visible` storage is a generated product-routing projection, not source visibility and not an authorization/security boundary. Under the Checkpoint B target, routing is GM-complete for useful authored information and every false/excluded disposition requires a non-auth product rationale, owner, fixtures, validation, and audit checkpoint. Tooling/container scaffolding may remain inspection-only when it has no addressable product meaning; its `hidden` or `internal` classification is not itself the rationale.
- Activation time is the normalized "how long it takes to do/cast/use this" field family. Spell or effect duration remains a separate normalized field family because it means how long the effect lasts after activation.
- Actor, item, and spell side tables hold stable non-metric filter and presentation data. Metrics remain in `record_metrics`; direct Foundry item projections keep `system_*` names.
- The artifact does not have a generic `subcategory` scope. Useful grouping concepts are represented as explicit metadata/filter axes when they come from source fields or collapsed signals, and as trait filters when they are entirely trait-derived.
- Open PF2E/provider-defined values such as traits, metric keys, pack names, and some metadata text values may remain strings, but their normalization owner must be explicit.
- Runtime behavior should prefer typed columns, side tables, and generated catalogs over `raw_json`. Persisted raw JSON is for parity, debugging, and future ingest analysis, not normal lookup/search/discovery execution.
- Filterable multi-value fields should have typed row projections or generated catalogs. JSON array columns can remain as compact presentation caches only when generated from the same typed source. Small display-only source arrays, such as feat prerequisite text, may live on `records` as JSON when runtime consumers only need to present the authored strings and do not need relational filtering, discovery, FTS, or semantic retrieval over their values.
- Generated projections such as `records_fts`, `record_vector_index`, metric catalogs, filter discovery catalogs, and aliases must be written from typed source models and covered by row-count/key-coverage validation. Tag rows are deferred until the accepted tagging architecture is implemented; the target table family is `record_tags`, not legacy `record_derived_tags`.
- `record_vector_index` is a query index, not a duplicated filter store. The Rust vector table should stay to rowid plus vector only. Do not add `record_key`, `embedding_unit_key`, `record_kind`, trait, level, rarity, metric, source, or other filter projection columns as part of the baseline. If performance testing later proves a vec metadata or partition column is needed, treat it as an accelerator generated from authoritative rows and add validation that it cannot drift.
- Semantic search with filters uses authoritative SQL keyset prefiltering. The shared filter compiler lives in `atlas-index` and lowers the canonical `SearchFilterNode` tree into a parameterized eligible-record relation from `records` and side tables. Browse, lexical, lookup narrowing, and vector search compose from that same eligible relation; vector search should constrain sqlite-vec units by `record_vector_index.rowid` using eligible `document_embedding_cache.rowid` values and return raw matching units to `atlas-search`. Do not implement exact filtered semantic search by joining ordinary filter tables around the vec scan alone, because joined predicates can be applied after the vec top-k result set.
- The `atlas-index` vector query boundary accepts an already-computed query vector and optional `SearchFilterNode`. Text-to-query-vector embedding remains owned by `atlas-embedding` and is called by `atlas-search` before it invokes vector-index SQL. Product surfaces should consume `atlas-search` for semantic search instead of opening SQLite connections or embedding models directly.
- `atlas-search` groups sqlite-vec unit hits by parent record before returning user-facing records. Search-time semantic modes determine which embedding units participate: `parent-only` restricts retrieval to parent units, `chunks` allows parent and child units to compete on raw vector distance, and `weighted-chunks` allows child units to recover records while applying unit-kind rank-distance adjustments before parent-record collapse. Diagnostics may expose both raw vector distance and adjusted rank distance.
- Filters that cannot compile to an authoritative SQL keyset are errors for the first Rust baseline. Approximate overfetch-and-post-filter behavior is not part of the default contract.
- sqlite-vec sentinel values for nullable filter columns should not be needed in the baseline because the vector table has no filter projection columns. If future performance accelerators add vec metadata columns, their sentinels must stay hidden behind `atlas-index` vector projection helpers. Domain and search code should not observe sentinels as real metadata.
- Premaster-to-remaster bridges should be modeled as `remaster_links` or edition links, not as generic legacy compatibility. Canonical Rust lookup should be based on `RecordKey` and aliases, while record detail can expose explicit remaster bridge relationships.

## Artifact Validation Beyond Metadata

Metadata validation must remain available without loading `sqlite-vec`. For Rust-written SQLite artifacts, `atlas index validate` also validates the lightweight runtime table contract:

- required runtime table presence
- required column presence for the current artifact schema
- exact canonical relational row sets across all authoritative resource, entity, occurrence, creature-relationship, content, reference, exclusion, and metric columns, with no missing, extra, or foreign-key-valid-but-wrong rows
- one canonical creature body for every NPC record and no creature body for non-NPC records
- `artifact_record_count` agreement with `records`
- `source_record_count` plus `generated_record_count` agreement with `artifact_record_count`
- SQLite foreign-key integrity plus explicit relationship orphan checks
- boolean integer columns constrained to `0`/`1`
- metric value columns matching `value_type`
- `records_fts` key coverage exactly matching records eligible under the approved GM-complete product retrieval disposition, with every exclusion backed by non-auth rationale
- remaster routing policy: remaster-side records are ordinary browse results while linked legacy records remain directly addressable; this is a canonical-edition product rationale, not authorization
- metric key/value catalogs matching metrics emitted for product-eligible records
- filter discovery catalog tables present with coherent field/value/stat rows for broad discovery
- non-empty `document_embedding_cache` parent-record coverage exactly matching records eligible for semantic search under the approved product disposition, with one or more embedding-unit rows per eligible record and audited non-auth rationales for exclusions
- embedding unit policy metadata matching the runtime's active document-unit generation policy
- search-readiness validation for `record_vector_index` rowid coverage exactly matching `document_embedding_cache`
- vector dimensions matching the embedding identity metadata

Setup and `atlas index check` use a fast readiness check rather than the full deep validation suite. Fast readiness validates metadata compatibility, required runtime table presence, and vector capability for semantic targets. Deep validation stays behind `atlas index validate` and covers SQLite data coherence such as row coverage, catalog consistency, foreign keys, FTS, and vector row diagnostics. Source freshness comparison belongs to callers that supply an expected source signature or equivalent already-computed value. If vector readiness is missing or stale, the supported repair path is an ingest rebuild with the selected embedding model. Recomputing source-derived assignment quality, semantic coverage, or full parity against the Foundry corpus remains outside startup validation.

## Extension Loading

Artifact metadata validation must not load vector extensions. Vector-specific validation and search commands use an explicit sqlite-vec capability boundary and report `vector_extension_unavailable` when the extension cannot be loaded or probed. Plain artifact validation remains available on systems where vector extension loading is unavailable.

Rust sqlite-vec registration is isolated in `atlas-sqlite-vec`, a small crate that owns the required SQLite FFI call. Runtime crates such as `atlas-index` and `atlas-search` use that capability boundary instead of embedding unsafe extension-registration code locally.
