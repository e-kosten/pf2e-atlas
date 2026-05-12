# TypeScript Runtime Inventory And Rust Contract Mapping

Status: active contract input
Feeds: [migration-checklist.md](./migration-checklist.md) Phase 1.5
Last reviewed: 2026-05-12

This document maps the current TypeScript runtime shape to the Rust migration contract surface. It is not a promise to copy TypeScript module structure, names, table design, or type boundaries. It is the parity inventory that Phase 2 domain work, Phase 3 ingest/index work, and later lookup/search/discovery slices must either preserve deliberately, redesign with user-visible parity, or retire explicitly.

Current TypeScript behavior is the first compatibility baseline, not the final design authority. Follow-up agents should challenge inherited shapes when they look underspecified, stringly typed, over-broad, duplicated, or mismatched with Rust ownership. If the right Rust path is not obvious, the expected behavior is to present a short set of viable approaches and ask for input before hardening the contract.

## Classification Vocabulary

- `parity`: Rust should initially preserve the TypeScript behavior or artifact meaning.
- `rust redesign`: Rust should expose a stronger typed contract while preserving user-visible behavior.
- `transitional`: Rust may read or compare this during migration, but it should not become a primary long-term concept.
- `retired`: Rust should not carry this forward except for explicit parity analysis.

## Source Inventory

Primary TypeScript sources:

- `src/data/schema.ts`: SQLite schema, index version, table/index inventory.
- `src/data/indexing/build-index.ts`: index build stage order.
- `src/data/indexing/catalog-writer.ts`: metadata, pack rows, alias rows, remaster bridge rows, metric catalogs.
- `src/data/indexing/record-writer.ts`: records, traits, derived tags, side-data tables, references, FTS rows.
- `src/data/indexing/embedding-writer.ts`: reusable embedding blobs and sqlite-vec rows.
- `src/data/index-types.ts`: normalized ingest/write model.
- `src/domain/record-types.ts`: runtime record shape.
- `src/domain/search-types.ts`: categories, subcategories, lookup/search result contracts, filter discovery fields.
- `src/domain/search-request-types.ts`: canonical request and filter tree.
- `src/domain/metadata-field-types.ts` and `src/domain/search-filter-metadata.ts`: metadata fields and predicate contracts.
- `src/domain/rule-types.ts`: rule graph and rule-context contracts.
- `docs/architecture/search.md`: current search flow and owner boundaries.
- `docs/architecture/editorial.md`: current derived-tag runtime/editorial split.

## SQLite Artifact Table Map

| Current TS table | Current role | Rust classification | Rust owner | Notes |
| --- | --- | --- | --- | --- |
| `metadata` | TypeScript schema, source, and embedding identity metadata | transitional | `atlas-index` reader only during migration | Rust artifacts use `artifact_metadata`. TS `metadata` remains useful for legacy diagnostics and parity comparisons. |
| `artifact_metadata` | New Rust artifact contract metadata | parity | `atlas-domain` constants, `atlas-index` validation, `atlas-ingest` writer | First Rust-owned metadata table. Existing contract covers keys and diagnostics. |
| `packs` | Pack labels, document types, source paths, counts | parity | `atlas-index` read model, `atlas-ingest` writer | Required for pack filtering, display, schema discovery, and source parity. |
| `records` | Canonical normalized record row plus raw JSON and search flags | rust redesign | `atlas-domain` record types, `atlas-index` row loading, `atlas-ingest` writer | Preserve field meaning, but model Rust rows as typed structs rather than stringly row bags. |
| `record_aliases` | Variant/name lookup aliases | parity | `atlas-index`, `atlas-ingest`, `atlas-search` lookup | Required before production lookup parity. |
| `record_legacy_links` | Premaster-to-remaster record bridges extracted from remaster journals and migration aliases | rust redesign | `atlas-domain`, `atlas-index`, `atlas-ingest` | Preserve the domain concept, but name/model it as `remaster_links` or edition links in Rust rather than a generic legacy compatibility bucket. |
| `record_traits` | Normalized trait rows | parity | `atlas-index`, `atlas-ingest`, `atlas-search` filters/discovery | Required for trait filters and value discovery. |
| `record_derived_tags` | Normalized derived-tag rows | parity | `atlas-tags` model, `atlas-index`, `atlas-ingest`, `atlas-search` filters/discovery | Required for read-only tag filtering before editorial migration. |
| `actor_records` | Actor-specific side data | parity | `atlas-domain`, `atlas-index`, `atlas-ingest` | Required for creature/hazard discovery and filters. |
| `actor_metrics` | Actor metric predicates and discovery | parity | `atlas-domain`, `atlas-index`, `atlas-ingest`, `atlas-search` | Required for metric filters and dynamic metric discovery. |
| `item_records` | Item/equipment side data | parity | `atlas-domain`, `atlas-index`, `atlas-ingest` | Required for item metadata filters and discovery. |
| `item_metrics` | Item metric predicates and discovery | parity | `atlas-domain`, `atlas-index`, `atlas-ingest`, `atlas-search` | Required for metric filters and dynamic metric discovery. |
| `metric_key_catalog` | Precomputed metric key availability by scope | parity | `atlas-index`, `atlas-ingest`, `atlas-search` discovery | Must be written before `atlas filters list-values` can replace MCP discovery. |
| `metric_value_catalog` | Precomputed text/boolean metric values by scope | parity | `atlas-index`, `atlas-ingest`, `atlas-search` discovery | Must be part of Phase 3 or a blocking prerequisite for Phase 7. |
| `spell_records` | Spell-specific side data | parity | `atlas-domain`, `atlas-index`, `atlas-ingest` | Required for spell filters/discovery and presentation. |
| `embeddings` | Reusable vector blobs plus semantic input hashes | parity | `atlas-index`, `atlas-embedding`, `atlas-ingest` | Preserve split between reusable vector storage and sqlite-vec rows. |
| `record_embeddings` | sqlite-vec virtual table with filter partition columns | rust redesign | `atlas-index` vector access, `atlas-embedding`, `atlas-search` | Preserve vector/filter meaning. Rust may refine capability checks and row loaders. |
| `reference_edges` | Extracted exact record references and backlink source facts | parity | `atlas-domain`, `atlas-index`, `atlas-ingest`, `atlas-search`, rule graph | Required for `linksTo`, `linkedFrom`, graph, and rule context. |
| `records_fts` | SQLite FTS5 lexical index | parity | `atlas-index`, `atlas-ingest`, `atlas-search` | First Rust search baseline remains SQLite-centered. |

Required Phase 3 writer outputs are therefore broader than the current checklist's first table list. The writer plan must cover `packs`, side tables, metric catalogs, embeddings/vector linkage, and derived-tag rows, not only `records`, `records_fts`, and `reference_edges`.

## Foundational DB And Type Design Review

The TypeScript schema is a reliable inventory of current behavior, but it should not be treated as a fully scrutinized long-term database design. The Rust migration should preserve user-visible behavior while taking the chance to tighten artifact and type boundaries before Phase 3 writes durable data.

### Decisions To Reconsider Before Phase 3

| Current TS design | Risk | Rust plan decision |
| --- | --- | --- |
| Arrays stored both as JSON blobs on `records` and as side tables for selected fields, such as traits and derived tags | Duplicated truth can drift and forces row hydration to parse JSON for common filters/discovery | Prefer typed side tables for filterable multi-value fields. Keep JSON blobs only for compact record presentation or parity debugging when they are generated from the same typed source. |
| `records` is a broad catch-all row containing core identity, presentation text, classification, variant facts, search text, raw JSON, and several denormalized filter fields | Makes the central table hard to version and encourages consumers to depend on incidental columns | Split Rust domain types into identity, summary, presentation, source/provenance, variant, and search-index inputs. The SQLite table may stay wide for read performance, but Rust APIs should not expose one giant mutable shape as the primary contract. |
| `raw_json` is persisted for every row | Useful for debugging, but dangerous if runtime behavior keeps reaching into arbitrary JSON paths | Keep initially for parity/debugging. Runtime lookup/search/discovery should use typed rows. Any raw JSON dependency must be called out as an ingest gap. |
| Boolean values are stored as `INTEGER` without explicit value constraints | SQLite permits non-0/1 values unless writers are disciplined | Rust artifact schema should add `CHECK` constraints for boolean columns or validate them during artifact validation. |
| Many enum-like columns are plain text: category, subcategory, rarity, source category, variant source, document type, record type, metric value type | Current TypeScript relies on conventions and runtime normalization | Rust should model these as enums or validated newtypes at load time. SQLite can still store text, but invalid literals should fail artifact validation or row loading. |
| `RecordKey` is a string alias in TypeScript | Malformed keys can cross boundaries until late SQL/runtime failures | Rust should use a parseable `RecordKey` newtype with `pack` and `id` components, and only serialize/display as `pack:id` at boundaries. |
| Metric storage uses EAV tables with string metric keys and dynamic value types | Flexible, but weakly typed and hard to discover without catalogs | Keep EAV for first parity because actor/item metrics are open-ended. Strengthen with typed `MetricValue`, catalog validation, namespace prefix parsing, and stable metric key normalization. |
| `metric_key_catalog` and `metric_value_catalog` are derived after all record writes | Correct today, but easy for future writers to forget | Make metric catalog writing a required artifact stage with validation that catalog rows match metric rows for canonical records. |
| `record_embeddings` duplicates many filter columns already present in `records`/side tables | Useful for sqlite-vec partition filtering, but can drift from source rows | Treat vector filter columns as generated projection data. Rust writer should derive them from typed records in one function and validation should detect mismatched row counts/keys. |
| Sentinel values in sqlite-vec partition columns use empty string and `-1` | Necessary for vec metadata constraints, but semantically lossy | Hide sentinels behind `atlas-index` vector-row projection helpers. Domain/search code should never see sentinel values as real metadata. |
| `reference_edges` has generic reference rows but no explicit relationship enum beyond source/target and text | Adequate for current `linksTo`/`linkedFrom`, but may blur rules, page links, aliases, and generated references later | Add a Rust relationship/source-kind enum before expanding graph behavior. Preserve current `references` semantics for parity, but leave room for typed relationship classes. |
| Current TS `record_legacy_links` is a real table beside canonical aliases | The concept is useful for PF2E remaster navigation, but the current name makes it sound like generic compatibility | Keep the concept as `remaster_links` or edition links. Rust lookup should prefer canonical keys and aliases, while record detail can expose explicit remaster bridge relationships. |
| Search canonicality is stored as `is_search_canonical` | Necessary for variants/generated records, but easy to misuse | Model canonicality as a typed search visibility policy, not just a boolean. Decide whether generated afflictions, variants, and aliases need separate policy states. |
| FTS text is stored as `search_text` on `records` and repeated in `records_fts` | Practical for ranking/debugging, but duplicated | Keep for first parity. Rust writer should generate both from a single search-text artifact and validate row-key coverage. |
| Detail/output vocabulary differs between current TS (`minimal`, `standard`, `full`) and roadmap draft (`compact`, `full`, `answerable`) | Output contract drift can leak into CLI docs and tests; `answerable` is not a current runtime concept | Keep the current TypeScript wire values `minimal`, `standard`, and `full`. Do not add `answerable` as a generic detail level. Treat `compact` as descriptive copy only, not a wire value. |

### Rust Type Principles For The Migration

- Use newtypes for identifiers: `RecordKey`, `PackName`, `RecordId`, `MetricKey`, `SourceSignature`.
- Use enums for closed vocabularies: category, subcategory, source category, rarity when normalized, search mode, search profile, sort kind, filter node kind, text status, detail level.
- Use typed side-data structs for actor, item, spell, publication, variant, and embedding identity.
- Keep open PF2E/provider-defined values as validated strings with clear owners rather than pretending they are closed enums.
- Use `Result` at boundary decoders and row loaders. Missing or malformed runtime-required fields should be artifact errors, not silently defaulted values.
- Keep SQLite row structs separate from CLI output structs. Storage shape, domain shape, and presentation shape should not collapse into one large type.
- Make generated projections explicit: FTS rows, vector rows, metric catalogs, aliases, and derived tag rows should be derived from typed source models and have coverage checks.

### Design Review Gates

Before Phase 2 is complete:

- Keep Rust detail vocabulary aligned with the current TypeScript wire values: `minimal`, `standard`, and `full`.
- Decide which text columns are required for compact lookup versus full record output.
- Decide which TypeScript string aliases remain accepted CLI inputs.

Before Phase 3 writer work starts:

- Add the full Rust artifact table contract beyond `artifact_metadata`.
- Decide which JSON array columns are durable versus generated presentation caches.
- Decide whether boolean and enum-like columns get SQLite `CHECK` constraints, row-loader validation, or both.
- Decide the vector-row projection/sentinel contract.
- Model the current TS remaster bridge table as `remaster_links` or edition links and preserve premaster-to-remaster navigation semantics.

Before Phase 7 discovery starts:

- Validate that discovery fields are backed by typed columns, side tables, or generated catalogs, not ad hoc raw JSON reads.
- Validate metric catalog coverage against metric rows for canonical records.

## Indexing Stage Map

| Current TS stage | Current owner | Rust stage | Rust owner | Classification |
| --- | --- | --- | --- | --- |
| Write metadata | `catalog-writer.ts` | Write `artifact_metadata` and migration comparison metadata | `atlas-ingest` + `atlas-domain` constants | rust redesign |
| Load source packs and records | `source-loading.ts` | Tolerant Foundry source loading | `atlas-ingest` | parity |
| Write packs | `catalog-writer.ts` | Pack writer | `atlas-ingest` | parity |
| Normalize records | `record-normalization.ts` and helpers | Boundary parse to typed ingest model | `atlas-ingest` using `atlas-domain` value types | rust redesign |
| Assign families | `family-assignment.ts` | Family/variant assignment | `atlas-ingest` initially, possibly later `atlas-search` for reusable semantics | parity |
| Resolve references, aliases, remaster links | `reference-resolution.ts` | Reference/alias/remaster-link resolver | `atlas-ingest` | parity with Rust naming/model cleanup |
| Canonicalize records and derived afflictions | `canonicalization.ts`, `derived-afflictions.ts` | Canonical record selection and generated record policy | `atlas-ingest` | parity |
| Build writable model | `record-write-model.ts` | Typed writer model | `atlas-ingest` | rust redesign |
| Write records, side data, FTS, references | `record-writer.ts` | Table writers with typed row inputs | `atlas-ingest` | parity |
| Generate/reuse embeddings | `embedding-writer.ts` | Embedding writer and reusable vector loader | `atlas-embedding` + `atlas-ingest` | parity |
| Write alias and remaster-link rows | `catalog-writer.ts` | Alias writer and remaster-link writer | `atlas-ingest` | parity with Rust naming/model cleanup |
| Populate metric catalogs | `catalog-writer.ts` | Metric catalog writer | `atlas-ingest` | parity |

Rust implementation should keep the stage order mostly intact until parity is proven. The crate/module layout can differ, but the source-to-artifact data dependencies should remain explicit.

## Domain Contract Map

| Current TS contract | Rust contract | Owner | Classification | Notes |
| --- | --- | --- | --- | --- |
| `RecordKey = string` | `RecordKey { pack, id }` parseable/displayable newtype | `atlas-domain` | rust redesign | User-visible key syntax remains `pack:id`; Rust should reject malformed keys at the boundary. |
| `SearchCategory` and aliases | `Category` enum plus explicit input alias parser | `atlas-domain` | rust redesign | Domain serde uses Rust-owned canonical values. Alias acceptance belongs at user-facing or compatibility boundaries and must serialize back to one canonical representation. |
| `SearchSubcategory` and aliases | `Subcategory` enum plus explicit input alias parser | `atlas-domain` | rust redesign | Domain serde uses Rust-owned canonical values. Alias acceptance belongs at user-facing or compatibility boundaries and must serialize back to one canonical representation. |
| `SourceCategory` | `SourceCategory` enum | `atlas-domain` | parity | Values: `core`, `rules`, `adventure`, `unknown`. |
| `VariantSource` | `VariantSource` enum | `atlas-domain` | parity | Keep until variant parity is classified. |
| `RecordDetail` | `DetailLevel` enum | `atlas-domain` | rust redesign | Current TS values are `minimal`, `standard`, `full`, and Rust keeps those as wire values. `standard` is current full record content minus `sourcePath`; `full` adds source provenance. Do not add `answerable` as a generic detail level. |
| `NormalizedRecord` | `RecordSummary`, `RecordDetail`, side-data structs | `atlas-domain` | rust redesign | Avoid one overgrown public struct when command outputs need smaller typed envelopes. |
| `SearchRequest` | tagged enum: browse/search/lookup | `atlas-domain` | parity semantics, Rust-owned shape | Preserve mode semantics. Do not bake TypeScript camelCase request JSON into core domain serde; put TS compatibility, if needed, in a removable adapter. |
| `SearchFilterNode` | recursive enum | `atlas-domain` | parity semantics, Rust-owned shape | Exhaustive matching should force downstream search handling. Core serde uses Rust-owned canonical names; TS compatibility belongs in an adapter. |
| metadata fields and predicates | field enum by kind plus predicate enum | `atlas-domain` | parity semantics, Rust-owned shape | Preserve operator semantics by field type. Core serde uses Rust-owned canonical names; TS compatibility belongs in an adapter. |
| metric predicates | metric predicate structs/enums | `atlas-domain` | parity | Keep metric key as string initially; catalog discovery owns valid values. |
| `SearchProfile` | enum: lexical/balanced/concept | `atlas-domain` or `atlas-search` | parity | Search execution owns meaning; domain may own wire value. |
| browse/lookup sort specs | sort enum plus lookup policy enum | `atlas-domain` | parity | Preserve random seed support. |
| `LookupResult` | lookup result envelope | `atlas-domain` or `atlas-cli` | parity | Preserve safe exact-miss behavior; output can be narrower. |
| `SearchResult` | search result envelope | `atlas-domain` or `atlas-cli` | parity | Preserve total/offset/limit/hasMore semantics. |
| `RuleReferenceEdge` and graph results | graph edge/result structs | `atlas-domain` | parity | Required for rule graph and rule context. |
| derived-tag ontology/runtime types | runtime tag model and filtered row model | `atlas-tags` with shared ids in `atlas-domain` | rust redesign | Do not port high-churn editorial state before read-only runtime consumption. |

## Filter And Discovery Contract Map

`atlas filters list-values` and `atlas schema search-filters` must be able to replace current MCP discovery. Initial Rust discovery must cover:

- field vocabulary from `FILTER_VALUE_FIELDS`
- metadata field kind/operator compatibility
- category and subcategory scopes
- traits, families, derived tags, and variant axes
- spell traditions and spell kinds
- item fields such as item category, base item, usage, hands, weapon group, armor group, price, bulk, and damage types
- actor fields such as size, languages, speeds, senses, immunities, resistances, weaknesses, disable skills, and complexity
- spell fields such as range, save type, area type, duration, target, sustained, and basic save
- actor and item metric key/value discovery from `metric_key_catalog` and `metric_value_catalog`
- source, pack, category, and subcategory discovery

Discovery is blocked on Rust-owned writes for the tables and catalogs that back these values. Phase 7 should not be marked complete until the Rust artifact can answer these discovery calls without TypeScript runtime help.

## Product Surface Parity Map

The Rust CLI can use different command names, but Phase 5 through Phase 10 should map to the current product surface
instead of inventing PF2E capability. Any command that does not map to one of these rows needs a backlog or ADR decision
before it becomes part of the durable CLI contract.

| Current product surface | Current behavior | Rust parity target |
| --- | --- | --- |
| `pf2e_lookup` | Best matching record by name with optional pack/category/subcategory hints and alternatives | `atlas lookup` or equivalent exact-name lookup command. |
| `pf2e_lookup_many` | Batch exact-name resolution with compact output by default | Batch lookup command or a stable JSON input mode for `atlas lookup`. |
| `pf2e_get_record_by_key` | Exact record fetch by canonical key, including raw JSON today | Record-key lookup command; raw JSON remains debug/parity-only unless typed fields are missing. |
| `pf2e_get_records_by_key` | Batch canonical-key fetch with detail selection | Batch record-key lookup with current detail semantics. |
| `pf2e_list_packs` | Pack list, labels, document types, record counts, and startup warnings | Pack list command if CLI replaces the full MCP discovery surface. |
| `pf2e_get_pack_metadata` | Metadata for one pack by name or label | Pack metadata command if pack discovery remains user-facing. |
| `pf2e_search` | Ranked search using the canonical `mode:"search"` request branch | `atlas search` with the same query/profile/exclude/filter semantics. |
| `pf2e_list_records` | Browse/list using the canonical `mode:"browse"` request branch | `atlas browse` or `atlas list` with the same filter, sort, and pagination semantics. |
| `pf2e_get_search_semantics` | Category-first ontology, filter vocabulary, metadata semantics, derived-tag vocabulary, and ranking status | Search/schema discovery command such as `atlas schema search-filters`. |
| `pf2e_list_filter_values` | Live filter-value discovery by field, scope, category/subcategory, and metric key/prefix | Filter-value discovery command such as `atlas filters list-values`. |
| `pf2e_collect_rule_question_context` | Primary rule lookup plus outgoing support records and optional curated backlinks | Rule-context command that returns context only; it should not synthesize an answer. |
| `pf2e_get_rule_graph` | Rule graph records and edges for known canonical record keys | Graph command over record keys with outgoing/backlink controls. |
| `npm run tui` / Ink workbench | Derived-tag migration workbench | Ratatui replacement only after core lookup/search/detail flows are stable, then editorial workflows in separate slices. |
| `src/tags/cli/**` scripts | Derived-tag discovery, evaluation, migration session, review, import, lint, and queue summary workflows | Tag CLI commands should preserve existing workflow semantics first. Names such as `tags review next` are only placeholders unless mapped to an existing script or approved as a new workflow. |

## Compatibility And Retirement Decisions

| TS concept | Rust decision |
| --- | --- |
| `metadata` table | Transitional legacy diagnostic only. |
| Current TS remaster bridge table | Preserve as `remaster_links` or edition links with explicit premaster-to-remaster semantics. |
| broad raw `raw_json` storage | Keep for initial parity/debugging; do not make Rust runtime behavior depend on arbitrary JSON paths where typed rows exist. |
| legacy derived-tag matcher and seed migration paths | Transitional comparison/input only; read-only derived-tag runtime should prefer published/current authored model. |
| MCP server tool schema as primary product contract | Retired as primary; optional compatibility only after CLI plus skill is proven. |
| Ink TUI controller conventions | Retired after Ratatui replacement; do not copy implicit state patterns. |

## Crate Ownership

- `atlas-domain`: record keys, category/subcategory vocabularies, metadata field vocabulary, search request/filter contracts, rule graph contracts, shared artifact metadata constants, common output envelope primitives when shared by multiple surfaces.
- `atlas-index`: read-only artifact opening, metadata validation, table contract validation, typed row loading, prepared SQL owners, vector table capability checks.
- `atlas-ingest`: Foundry source loading, normalization, reference/alias resolution, canonicalization, table writers, metric catalog writer, source signature generation.
- `atlas-embedding`: MiniLM query/document embedding, tokenizer/model identity, reusable vector blob handling, sqlite-vec integration helpers.
- `atlas-search`: request lowering, filter SQL, FTS retrieval, vector retrieval, hybrid fusion, rerank adjustments, discovery commands over index/catalog readers.
- `atlas-tags`: read-only derived-tag runtime types, published ontology/assignment loading, tag filter support, later editorial state machines.
- `atlas-cli`: command routing, argument parsing, stable JSON output, exit code policy, human-readable presentation.
- `atlas-tui`: Ratatui state machines over Rust runtime services.
- `atlas-mcp`: optional compatibility surface only; no MCP-only backend behavior.

## First Parity Fixture Set

Each later phase should update a durable parity note with source revision, command inputs, TS behavior, Rust behavior, accepted differences, and open defects.

### Lookup And Record Presentation

- `Treat Wounds`
- `Grabbed`
- `Antidote (Lesser)`
- one variant family with aliases
- one exact miss that must not return fuzzy unrelated records
- one localized or placeholder-text record

### Rule Graph And Rule Context

- `Grab` bestiary glossary/rule-context case
- direct outgoing references for a rule
- backlinks with default suppression and explicit include
- ambiguous rule names
- localized support text

### Schema And Filter Discovery

- poison consumables discovery task
- creature trait values by category/subcategory
- actor metric key discovery with namespace prefix
- item metric key/value discovery
- spell metadata values for traditions, spell kinds, range, save, area, duration, sustained, and basic save
- derived-tag value discovery

### Search And Browse

- browse category with alphabetical, level ascending, level descending, and random seed sorts
- lexical search with `search.exclude`
- balanced hybrid search for a known concept query
- concept profile query from the search-quality bakeoff
- structured filter with `linksTo`
- structured filter with `linkedFrom`
- metadata predicate and metric predicate filters

### Derived-Tag Runtime

- read-only tag list by category
- tag filter for equipment
- tag filter for spell
- tag filter for creature
- parity sample for current explicit assignments

### CLI Output Contracts

- valid command JSON envelope
- invalid option exit code
- missing index exit code
- incompatible artifact exit code
- source-signature mismatch diagnostic when an expected signature is provided
- compact default output size check

## Phase-Gate Rules

- Phase 2 domain work must cite this map when adding each durable type.
- Phase 3 writer work must update the SQLite artifact table contract before adding broad table writers.
- Phase 4 embedding work must preserve MiniLM compatibility unless a new ADR changes the baseline.
- Phase 7 discovery work cannot introduce a new table or catalog dependency without adding it to this map and the artifact contract.
- Phase 13 retirement cannot start until each parity fixture group has a recorded pass, accepted difference, or explicit deferred defect.
- Source freshness validation should stay lightweight: compare against an expected source signature when supplied, but do not add a broad full-artifact validator that effectively reloads the source corpus.
