# LadybugDB Graph Database Spike

## Goal

Evaluate whether LadybugDB is a viable graph-native search artifact for PF2e Atlas.

The central question is whether modeling records, references, aliases, traits, metrics, remaster links, and embedding units as a property graph improves retrieval and graph context enough to justify moving away from the current relational SQLite artifact contract.

## Background

PF2e Atlas already has graph-shaped data:

- `reference_edges` for links, backlinks, and graph context
- aliases and remaster links
- generated source-backed records and provenance relationships
- traits, packs, metrics, and taxonomy axes
- embedding units that collapse back to parent records
- future derived tags and TUI exploration workflows

This spike originally targeted Kuzu directly, but `kuzudb/kuzu` was archived on October 10, 2025. The spike now targets LadybugDB, the maintained Kuzu fork, through the Rust crate `lbug`. Any recommendation must treat the archived upstream as a key dependency-risk discovery and evaluate LadybugDB's governance, release cadence, Rust bindings, packaging, and compatibility as part of the core technical result.

LadybugDB is interesting because graph traversal, structured predicates, full-text search, and vector search may live in one embedded on-disk database. It should be evaluated as an architectural shift, not as a drop-in SQLite replacement.

The product question is not only "can LadybugDB replace SQLite?" It is whether PF2e Atlas should become a local rules graph with search entry points. A successful LadybugDB design should make relationship-aware retrieval, graph context, concept browsing, and future TUI exploration materially better. If the prototype mostly recreates relational tables in Cypher, the architecture shift is probably not worthwhile.

## Before Starting

Refresh this worktree from the repository's local `main` branch before doing spike work:

```bash
git merge main
```

Do this from this worktree, not from the shared main checkout. The local `main` branch may contain active SQLite FTS spike work that should be used as current reference material when evaluating LadybugDB.

## Current Prototype

The local prototype crate is `crates/atlas-ladybug-spike`. It uses the LadybugDB Rust crate:

```toml
lbug = "0.16.1"
```

Installation was also tested in a clean minimal Rust project at `/private/tmp/pathfinder-mcp-worktrees/ladybug-install-repro`. That repro only depends on `lbug = "0.16.1"` and creates/query one in-memory node.

Observed install/build paths on macOS arm64 with Rust 1.95.0:

- Plain documented Rust dependency failed:
  - command: `cargo run`
  - result: compile reached final binary link and failed with missing symbols from bundled native dependencies, including `simsimd` and `yyjson`
  - interpretation: the default prebuilt static archive path is not currently clean on this machine
- Forced Rust source build passed:
  - command: `LBUG_RUST_BUILD_FROM_SOURCE=1 cargo run`
  - result: passed and printed the expected query result
  - required local tool: `cmake`
  - interpretation: Rust support is real, but the viable clean path may be source-build rather than prebuilt static linking
- Forced Rust source build plus extension-capable binary passed:
  - command: `scripts/run-ladybug-spike.sh`
  - crate requirement: `crates/atlas-ladybug-spike/build.rs` emits `cargo:rustc-link-arg=-rdynamic`
  - result: loaded `FTS`, `VECTOR`, and `ALGO`; ran FTS, vector, and projected filtered-vector checks
  - interpretation: source-build is viable for spike development, and extension loading works once the binary exports the symbols documented by `lbug`
- Documented Homebrew dynamic path failed:
  - command: `LBUG_SHARED=1 LBUG_LIBRARY_DIR=/opt/homebrew/lib LBUG_INCLUDE_DIR=/opt/homebrew/include cargo run`
  - result: C++ bridge compile failed because `common/enums/statement_type.h` was not found
  - interpretation: Homebrew `ladybug` installs `lbug.h` and `lbug.hpp`, but not the internal headers needed by the Rust CXX wrapper
- Homebrew dynamic path plus source include workaround passed:
  - command: `LBUG_SHARED=1 LBUG_LIBRARY_DIR=/opt/homebrew/lib LBUG_INCLUDE_DIR=/opt/homebrew/include CXXFLAGS=-I.../lbug-0.16.1/lbug-src/src/include cargo run`
  - result: passed
  - interpretation: dynamic linking can work, but this include-path workaround is too brittle for a product install story

The least surprising spike command is currently the source-build path:

```bash
LBUG_RUST_BUILD_FROM_SOURCE=1 cargo run -p atlas-ladybug-spike
```

Use the checked-in helper for normal spike runs:

```bash
scripts/run-ladybug-spike.sh
```

The helper sets `LBUG_RUST_BUILD_FROM_SOURCE=1` by default and then runs
`cargo run -p atlas-ladybug-spike`. This environment variable affects the build
path only: it forces the Rust crate to compile the bundled Ladybug native engine
from source instead of linking the default prebuilt archive. It does not imply
that end users must compile Ladybug from source if Atlas ships prebuilt release
binaries. The portability cost is paid by developers, CI, and release builders:
they need a native C++ toolchain and CMake, first builds are slower, and
cross-compilation becomes harder. This is acceptable for the spike and likely
acceptable for production only if Atlas's release pipeline owns per-platform
binary builds.

The spike crate also has a `build.rs` that emits `cargo:rustc-link-arg=-rdynamic`.
The `lbug` crate documents this as required for binaries or tests that load
Ladybug extensions on non-Windows platforms. Without it, the binary can build
and core graph queries can run, but extension loading fails with undefined
native symbols when `FTS`, `VECTOR`, or `ALGO` are loaded.

The runtime extension path is a separate packaging question. The current probe
loads `FTS`, `VECTOR`, and `ALGO`; if they are not installed, Ladybug attempts
to install them under the user's Ladybug extension cache. The spike must decide
whether a production Atlas build pre-installs, vendors, or deterministically
manages those extensions.

The dynamic-link workaround that was verified in this repository is:

```bash
LBUG_SHARED=1 \
LBUG_LIBRARY_DIR=/opt/homebrew/lib \
LBUG_INCLUDE_DIR=/opt/homebrew/include \
CXXFLAGS=-I/Users/ekosten/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lbug-0.16.1/lbug-src/src/include \
cargo run -p atlas-ladybug-spike
```

This is intentionally awkward and must be treated as a packaging question for the spike. Dynamic linking against the Homebrew `ladybug` formula worked only when the Rust crate also saw the full source include tree.

The probe currently builds a tiny Pathfinder-like graph with:

- `Record` nodes
- `Trait` nodes
- `Concept` nodes
- `EmbeddingUnit` nodes
- `HAS_TRAIT`, `REFERS_TO`, `APPLIES_CONCEPT`, and `HAS_EMBEDDING` relationships

Observed successful behaviors:

- direct record-key lookup
- structured record counts
- grouped facet counts under a partial filter
- graph neighborhood queries
- path explanations from a record to a referenced rule/concept
- FTS over multiple `Record` text properties with a structured `WHERE` filter on returned nodes
- vector search over `EmbeddingUnit` nodes and collapse back to parent records
- vector search over a projected filtered graph, proving the basic shape needed to avoid global top-k then post-filter behavior

## Initial PF2e Graph Model

This is the first long-term-oriented LadybugDB model to try. It is intentionally close enough to implement from the current SQLite artifact, but it should not copy SQLite tables one-for-one. The graph should make durable PF2e relationships first-class while keeping record hydration and filter semantics deterministic.

### Modeling Principles

- `Record` remains the user-facing result unit. Search, lookup, browse, graph context, and semantic parent collapse all return records unless diagnostics explicitly ask for lower-level units.
- Graph structure should represent reusable PF2e concepts, not just storage normalization. Traits, publications, packs, metrics, aliases, references, variants, remaster links, semantic units, and future derived concepts should be traversable where they produce product value.
- Multi-value filterable fields should become relationship edges or typed fact nodes, not JSON blobs. JSON should remain only for presentation payloads or parity debugging.
- FTS projection text and embedding units are search-entry nodes/properties connected back to records. They are allowed to be denormalized enough for search quality, but they should not own canonical structured filter truth.
- Structured filters must compile to a canonical eligible-record graph pattern or projected graph. FTS, vector, browse, counts, and graph traversal should consume that same eligible set rather than duplicate filter logic in separate search tables.
- Hidden/provenance records remain in the graph for direct lookup and relationship explanation, but default search and discovery start from `Record.is_default_visible = true`.

### Core Nodes

`Record`

- Primary key: `record_key`
- Suggested properties: `id`, `name`, `normalized_name`, `record_family`, `foundry_document_type`, `foundry_record_type`, `level`, `rarity`, `is_default_visible`, `source_path`, `publication_family`, `publication_remaster`
- Purpose: canonical Atlas result unit and direct lookup target
- Keep on node when scalar, commonly filtered, and record-owned: family, level/rank, rarity, visibility, normalized name, Foundry type
- Do not use `Record` as a dumping ground for every side-table field; use concept/fact nodes for reusable axes

`SearchDocument`

- Primary key: `search_doc_key`, likely `{record_key}#fts`
- Suggested properties: `record_key`, `title`, `aliases`, `traits`, `taxonomy_terms`, `constraint_terms`, `mechanic_terms`, `source_terms`, `metric_terms`, `headings`, `body`, `facts`, `reference_terms`, `embedded_content`
- Relationship: `(Record)-[:HAS_SEARCH_DOCUMENT]->(SearchDocument)`
- Purpose: preserve the existing FTS projection shape and field-weighting intent without pretending those projections are canonical structured facts
- Open question: Ladybug FTS may only index node-table string properties and may not provide enough field weighting. If so, split into multiple `SearchDocument` fields or multiple document-unit nodes, but keep the logical projection grouped around the record

`EmbeddingUnit`

- Primary key: `embedding_unit_key`
- Suggested properties: `record_key`, `unit_kind`, `label`, `ordinal`, `semantic_input_hash`, `dimensions`, `embedding`
- Relationship: `(Record)-[:HAS_EMBEDDING_UNIT]->(EmbeddingUnit)`
- Purpose: semantic retrieval unit, matching the current document embedding cache and parent-record collapse behavior
- Important test: filtered semantic search should query eligible `EmbeddingUnit` nodes through their parent `Record` eligibility, not require duplicated filter properties on every unit except for proven performance accelerators

`ContentUnit`

- Primary key: `content_unit_key`
- Suggested properties: `record_key`, `ordinal`, `source_kind`, `visibility`, `contributes_to_search`, `contributes_to_references`, `label`, `content_json`
- Relationship: `(Record)-[:HAS_CONTENT_UNIT]->(ContentUnit)`
- Purpose: model supplemental/default content as addressable graph context and provenance for references/search units
- Spike scope: hydrate only if needed for reference evidence, graph explanations, or richer search snippets

`Trait`

- Primary key: `name`
- Relationship: `(Record)-[:HAS_TRAIT]->(Trait)`
- Purpose: canonical trait filtering, facets, traversal, and related-record discovery
- Product value: trait neighborhoods become searchable/browsable graph surfaces rather than flat filters only

`Publication`

- Primary key: normalized publication title or stable publication key
- Suggested properties: `title`, `family`, `remaster`
- Relationship: `(Record)-[:PUBLISHED_IN]->(Publication)`
- Purpose: source/publication filtering, source facets, remaster-era exploration

`Pack`

- Primary key: `pack_name`
- Suggested properties: `pack_label`, `document_type`, `source_path`
- Relationship: `(Record)-[:FROM_PACK]->(Pack)`
- Purpose: source parity, diagnostics, pack filtering, and provenance

`Alias`

- Primary key: deterministic alias key, likely `{canonical_record_key}#alias#{normalized_alias}`
- Suggested properties: `alias_text`, `normalized_alias`, `source_kind`, `source_ref`
- Relationship: `(Record)-[:HAS_ALIAS]->(Alias)`
- Purpose: strict resolution, lookup expansion, and "why did this resolve?" explanations
- Alternative to test: make aliases searchable properties on `SearchDocument` while retaining `Alias` nodes for lookup provenance

`Metric`

- Primary key: `{metric_domain}:{metric_key}`
- Suggested properties: `metric_domain`, `metric_key`, `namespace_prefix`, `value_type`, display labels if available
- Relationship: `(Record)-[:HAS_METRIC {number_value, text_value, bool_value}]->(Metric)`
- Purpose: preserve open-ended actor/item metrics without creating a table-shaped node per row
- Product value: metric keys become browsable axes, and records sharing mechanical measurements can be discovered through the graph

`Concept`

- Primary key: stable concept key, e.g. `condition:frightened`, `mechanic:demoralize`, `damage:mental`, future derived tag keys
- Suggested properties: `name`, `kind`, `source`, `confidence`, `derived`
- Relationships: `(Record)-[:APPLIES_CONCEPT]->(Concept)`, `(Record)-[:MENTIONS_CONCEPT]->(Concept)`, `(Concept)-[:RELATED_TO]->(Concept)`
- Purpose: this is where Ladybug can become more than SQLite. Concepts should model mechanics and inferred/derived relationships that are useful for graph-native search.
- Spike caution: seed only a tiny concept vocabulary at first, using obvious traits/conditions/actions/references; do not invent a large derived ontology before core search works

`VariantGroup`

- Primary key: `variant_group_key`
- Suggested properties: `base_name`, `source`, `confidence`
- Relationships: `(Record)-[:IN_VARIANT_GROUP {variant_label, variant_axes_json}]->(VariantGroup)`
- Purpose: variant navigation and search grouping without burying variants as record-only JSON

### Core Relationships

`REFERENCES`

- Shape: `(Record)-[:REFERENCES {display_text, reference_text, source_kind, visibility}]->(Record)`
- Source: current `reference_edges`
- Purpose: authored links, backlinks, graph context, "records related by rule text", and path explanations
- Default policy: public non-embedded edges for ordinary graph context; broader visibility only when explicitly requested

`HAS_TRAIT`, `PUBLISHED_IN`, `FROM_PACK`, `HAS_ALIAS`, `HAS_METRIC`, `HAS_SEARCH_DOCUMENT`, `HAS_EMBEDDING_UNIT`, `HAS_CONTENT_UNIT`

- These provide the canonical graph patterns for structured search and facets.
- They should be the inputs for filter counts and discovery, not separate duplicated search metadata.

`REMASTERED_BY`

- Shape: `(legacy:Record)-[:REMASTERED_BY {source_kind, source_ref}]->(remaster:Record)`
- Source: current `remaster_links`
- Purpose: direct remaster navigation and lookup explanation
- Policy: remaster-side records are normally default-visible; legacy records may remain direct-lookup/provenance nodes

`GENERATED_FROM`

- Shape: `(generated:Record)-[:GENERATED_FROM {source_kind, source_ref}]->(source:Record)`
- Purpose: generated affliction canonicals/instances and future generated records
- Product value: explain why generated records exist and navigate back to source hosts

`APPLIES_CONCEPT`, `MENTIONS_CONCEPT`, `RELATED_TO`

- Purpose: graph-native product experiments. These should support queries like "find records connected to frightened through spell effects, traits, references, or generated concepts" without relying only on lexical match.

### Structured Filter Shape

The spike should implement filters as graph patterns over `Record` plus canonical relationship axes:

- scalar record filters: `Record.record_family`, `Record.level`, `Record.rarity`, `Record.is_default_visible`, `Record.foundry_record_type`
- trait filters: `(Record)-[:HAS_TRAIT]->(Trait {name})`
- source filters: `(Record)-[:PUBLISHED_IN]->(Publication)` and/or `(Record)-[:FROM_PACK]->(Pack)`
- metric filters: `(Record)-[rel:HAS_METRIC]->(Metric {metric_domain, metric_key})` with value predicates on `rel`
- reference filters: `(Record)-[:REFERENCES]->(target:Record)` and inverse backlink patterns
- alias/name resolution: `Record.normalized_name` plus `(Record)-[:HAS_ALIAS]->(Alias {normalized_alias})`
- concept filters: `(Record)-[:APPLIES_CONCEPT|MENTIONS_CONCEPT]->(Concept)`

This is the graph equivalent of SQLite's eligible-record CTE. The critical spike question is whether this eligible-record pattern can be reused by:

- browse/list queries
- count queries
- facet queries
- FTS result pruning
- vector projected graphs
- graph expansion around candidate records

If any path requires its own independent filter copy, document that as the same class of problem seen in the Tantivy and LanceDB spikes.

### FTS Shape

Start with `SearchDocument` attached to `Record` because the current SQLite/Tantivy direction already has a carefully curated FTS projection. The graph model should reuse that projection intent:

- identity fields: title, aliases
- categorical fields: traits, taxonomy terms, source terms
- mechanical fields: constraint terms, mechanic terms, metric terms
- content fields: headings, body, facts, embedded content
- relationship recall fields: reference terms

The first implementation can create one `SearchDocument` per default-visible record. If Ladybug FTS cannot express adequate field weighting across many properties, test either:

- multiple indexed properties queried together with manual score combination
- multiple search-document nodes per record, such as title/alias/mechanics/body/reference units
- retaining a specialist lexical index while Ladybug owns graph/vector/filtering

The long-term model should not collapse everything into one opaque `search_text` unless the spike proves Ladybug lacks usable field controls and the quality tradeoff is acceptable.

### Vector Shape

Use `EmbeddingUnit` nodes as the canonical semantic search target:

- parent unit: one per default-visible record
- child units: heading sections and titled options
- collapse: query returns embedding units, then groups by parent `Record`
- filtering: eligible records should determine eligible embedding units through `(Record)-[:HAS_EMBEDDING_UNIT]->(EmbeddingUnit)`

The preferred filtered-vector shape is a Ladybug projected graph over eligible `EmbeddingUnit` nodes. The spike must measure whether creating/reusing projected graphs is viable for interactive filters. If projection is too expensive, test exact search or bounded overfetch only as a clearly documented fallback, not as the default semantics.

### Discovery And Counts

The graph model should preserve old Node/current Rust affordances:

- count records matching a filter set
- count trait/source/family/rarity/level values under a partial filter set
- count metric keys and metric values under a partial filter set
- expose applicable filter axes for a record family
- distinguish unfiltered catalogs from live narrowed counts

Initial implementation should compute these live from graph patterns over canonical relationships. If that is too slow, add cached `FilterField`/`FilterValue`/`MetricCatalog` nodes later as generated accelerators with validation against canonical graph facts. Do not start with duplicated catalog nodes as the source of truth.

### Ingest Write Path

The primary spike implementation should build LadybugDB from the existing normalized build payload, not by exporting the completed SQLite artifact.

Add a parallel Ladybug writer under `atlas-index`, beside the SQLite index writer, and feed it from the same `IndexBuildInput` used by SQLite: normalized records, FTS projections, side-data facts, reference edges, aliases, metrics, content documents, generated records, and prepared embedding-unit metadata. This is the closest spike shape to a possible production path because it tests whether the graph model fits Atlas's canonical data before it has been flattened into SQLite rows.

Current spike status:

- `atlas index build --ladybug-output <path>` writes a LadybugDB graph artifact from the ingest pipeline.
- The writer emits `Record`, `SearchDocument`, `EmbeddingUnit`, `ContentUnit`, `Trait`, `Publication`, `Pack`, `Alias`, `Metric`, `VariantGroup`, and placeholder `Concept` tables plus the core graph relationships from ingest-owned data.
- The produced graph artifact reopens successfully when it stores graph data without persistent Ladybug FTS/vector indexes.
- Persistent search index creation is currently opt-in with `ATLAS_LADYBUG_CREATE_SEARCH_INDEXES=1`. The first test showed that creating Ladybug FTS/vector indexes during artifact write produced a database that failed to reopen with a Ladybug hash-index storage assertion. Treat persistent extension-index creation as the next hard blocker to isolate before claiming Ladybug FTS/vector artifact readiness.
- A toy on-disk probe in `crates/atlas-ladybug-spike` shows basic extension persistence works:
  - `scripts/run-ladybug-spike.sh disk-fts` creates an on-disk DB, creates an FTS index, closes, reopens, and queries FTS successfully.
  - `scripts/run-ladybug-spike.sh disk-vector` creates an on-disk DB, creates a vector index, closes, reopens, and queries vector successfully.
  - `scripts/run-ladybug-spike.sh disk-both` creates both indexes and both query successfully after reopen.
  - `scripts/run-ladybug-spike.sh theory-searchdoc-fts` and `theory-searchdoc-empty-fts` show `SearchDocument` FTS and empty-heavy multi-field FTS reopen successfully.
  - `scripts/run-ladybug-spike.sh theory-vector-384` and `theory-many-vector-384` show 384-dimensional vector indexes reopen successfully with non-empty vector rows.
  - `scripts/run-ladybug-spike.sh theory-ingest-shape-indexes` shows a small ingest-shaped graph with FTS and vector indexes reopens successfully.
  - `scripts/run-ladybug-spike.sh theory-empty-vector` reproduces the ingest failure: creating a vector index on an empty `EmbeddingUnit` table produces a DB that fails to reopen with `hashIndexStorageInfo.overflowHeaderPage == INVALID_PAGE_IDX`.
  - Interpretation: the current failing case is specifically empty persistent vector indexes. The ingest writer now skips Ladybug vector index creation when there are no document embeddings.

Embedding reuse is now backend-owned through the shared cache-reader trait:

- `atlas-index` exposes `DocumentEmbeddingCacheReader`.
- SQLite implements it from `document_embedding_cache`.
- Ladybug implements it from `EmbeddingUnit` nodes plus `ArtifactMetadata` embedding-identity rows.
- `atlas-ingest` asks configured existing artifacts for reusable document embeddings before generating new vectors.
- Treat SQLite or Ladybug only as embedding cache sources during reuse, not as the source of record, filter, FTS, graph, alias, metric, or discovery truth.

The Ladybug writer should map ingest data directly:

- normalized records become `Record`
- FTS projection output becomes `SearchDocument`
- prepared embedding-unit metadata plus reused vectors becomes `EmbeddingUnit`
- normalized traits become `Trait` and `HAS_TRAIT`
- resolved content references become `REFERENCES`
- lookup aliases become `Alias` and `HAS_ALIAS`
- remaster data becomes `REMASTERED_BY`
- metric rows become `Metric` and `HAS_METRIC`
- pack/source/publication facts become `Pack`, `Publication`, `FROM_PACK`, and `PUBLISHED_IN`
- generated-record provenance becomes `GENERATED_FROM`
- obvious graph-native concepts can be seeded from traits, conditions, references, and generated facts only where the mapping is clear

Avoid creating an isomorphic Ladybug copy of every SQLite table. If a field does not support filtering, retrieval, graph explanation, or product navigation, it can remain in record presentation JSON for now.

Extension behavior is important: `FTS`, `VECTOR`, and `ALGO` are runtime extensions. They require both access to the Ladybug extension cache and an extension-capable binary. Without `-rdynamic`, the probe built and core graph queries worked, but the installed extensions failed to load with undefined native symbols. With `crates/atlas-ladybug-spike/build.rs` emitting `cargo:rustc-link-arg=-rdynamic`, the helper loaded all three extensions and ran the FTS, vector, and projected filtered-vector checks. The production path must decide whether Atlas pre-installs these extensions, vendors them, uses a deterministic extension cache, or avoids a runtime install step.

## Next Open Questions

Performance tuning is not the primary next target. The bulk-load path proved that Ladybug can ingest the corpus quickly enough for a serious spike. The next work should decide whether Ladybug is actually useful for the product.

### 1. Graph-Native Product Value

Determine whether Ladybug unlocks genuinely useful search or exploration behavior that is hard, awkward, or less natural in the current SQLite artifact.

Explore questions such as:

- what records most directly or indirectly depend on a condition, trait, rule, action, spell, item, or source concept?
- what bridge records connect two mechanics, such as `frightened` and `demoralize`, or `fire` and `persistent damage`?
- can search results be explained through paths, shared traits, references, variant groups, remaster links, aliases, and semantic units?
- can a graph neighborhood become part of search ranking or result presentation, rather than only a separate `graph get` command?
- can concept-centric browsing over conditions, traits, mechanics, source concepts, and future derived tags become materially better than SQL edge-table browsing?
- can impact analysis answer "if this rule changes, what records should we inspect?"

The output should be concrete product-worthy query examples, not only proof that Cypher can express equivalent SQL joins. If the graph model mostly recreates current SQL edge-table behavior, that is a weak result.

### 2. FTS And Semantic Search Capability Comparison

Compare Ladybug against SQLite as a search backend on capability and result quality before doing more performance work.

For FTS, compare:

- field weighting and whether the current weighted projection can be represented without quality loss
- tokenizer, stemming, stopword, phrase, prefix, typo/fuzzy, and PF2e vocabulary behavior
- later follow-up: evaluate field-sensitive normalization choices, especially whether names/aliases/mechanical labels should avoid stemming/default stop words while body/rules text benefits from English stemming and stopword removal
- ability to explain why a record matched
- filtered FTS semantics: exact pre-filtering, overfetch/post-filtering, or some projected/indexed alternative
- result quality for known query sets against SQLite FTS and the Tantivy spike
- tuning options and whether ranking can be made product-acceptable

For semantic search, compare:

- parent/child embedding-unit modeling
- result quality using real query vectors, not synthetic vectors
- weighted chunk collapse back to parent records
- filtered vector search and whether projected graphs can preserve the exact filter behavior SQLite currently provides
- whether any duplicated parent filter projections on `EmbeddingUnit` are required and how they would be validated

The output should classify each path as one of:

- can replace SQLite behavior
- can supplement SQLite behavior
- cannot preserve product requirements without unacceptable tradeoffs

### 3. Shared Filtering Model

SQLite's strongest property is that structured filters, FTS, vector search, browse counts, and facet discovery can all compose through one authoritative filter tree and record-key keyset. Ladybug must either preserve this or unlock enough graph-native value to justify a different architecture.

Explore whether one filter model can constrain:

- browse/count queries
- facet counts and discovery
- FTS
- semantic/vector search
- graph traversal and graph-context retrieval

Specifically prove or reject:

- exact pre-filtered vector search using Ladybug projected graphs
- exact pre-filtered FTS, or a clear explanation that Ladybug FTS is post-filtered
- filter predicates that depend on relationships, metrics, traits, publication/source, visibility, and parent-record fields
- whether `SearchDocument`, `EmbeddingUnit`, and relationship tables need duplicated parent filter fields
- how duplicated filter projections would be validated against authoritative graph facts

If Ladybug requires duplicating core filter fields into multiple search nodes, record whether that is acceptable as a validated accelerator or whether it recreates the same multi-index drift risk we were trying to avoid.

## Explore

- Confirm that LadybugDB is a credible maintained successor to archived Kuzu for our use case, not merely a renamed frozen fork.
- Confirm current LadybugDB Rust support, local on-disk packaging, extension loading, FTS, vector indexes, and filtered vector search behavior.
- Design a minimal PF2e property graph:
  - `Record`
  - `Pack`
  - `Trait`
  - `Metric`
  - `Alias`
  - `EmbeddingUnit`
  - `Reference` / relationship edges with visibility and source metadata
- Test record-key lookup, alias lookup, structured filters, graph context, FTS retrieval, vector retrieval, and hybrid composition.
- Verify whether LadybugDB can perform vector search over a filtered candidate graph rather than overfetching and post-filtering.
- Evaluate how to represent weighted FTS fields currently stored in `records_fts`.
- Evaluate parent-record collapse from embedding-unit hits.
- Exercise one-hop and shallow multi-hop graph context queries with visibility policy.
- Check how filter discovery catalogs, metric catalogs, and validation diagnostics would translate from SQL tables to graph schema.
- Verify old-Node-style UI affordances, especially "given this filter set, how many records match?" and facet counts for traits, family, source, rank/level, metrics, and relationship-derived axes.
- Measure artifact size, ingest/build time, query latency, and query explainability on representative PF2e slices.
- Check schema migration, deterministic validation, native dependencies, binary size, licensing, and cross-platform CLI packaging.
- Compare LadybugDB's Rust crate (`lbug`) packaging against the archived Kuzu Rust crate; the archived crate exposed CMake/linking problems during the initial probe.

## Product Capabilities To Prove

LadybugDB should be treated as promising only if it unlocks product behavior that is hard or awkward in the current SQLite artifact:

- relationship-aware search where references, traits, remaster links, aliases, generated-record provenance, and semantic units participate in ranking or explanations
- graph context as part of search answers rather than only as a separate command
- concept-centric browsing for conditions, traits, mechanics, source concepts, and future derived tags
- impact analysis such as "what depends on this rule, trait, action, spell, or condition?"
- future TUI exploration over visible neighborhoods with edge-type, visibility, family, source, and metric filters
- result explanations based on paths, shared traits, backlinks, edge types, semantic units, and lexical matches
- derived tags or concepts modeled as nodes/edges with evidence rather than only flat row labels

If these behaviors are not meaningfully easier or better in LadybugDB, prefer staying with a SQL-shaped artifact or using a separate search specialist.

## FTS Tradeoffs To Test

Do not assume LadybugDB FTS is a drop-in replacement for SQLite FTS5 or Tantivy.

LadybugDB FTS currently indexes string properties on node tables and uses BM25. The spike must test whether that is enough for PF2e Atlas's weighted lexical projection. Current SQLite FTS has separate weighted fields such as title, aliases, traits, taxonomy terms, constraints, mechanics, source terms, metric terms, headings, body, facts, reference terms, and embedded content.

Evaluate at least these shapes:

- one denormalized `Record.search_text` property
- several indexed `Record` text properties such as title, mechanics, body, facts, and references
- separate searchable nodes connected back to `Record`
- LadybugDB for graph/vector with Tantivy, SQLite FTS, or another specialist retained for lexical search

The spike must report:

- whether LadybugDB supports adequate field weighting or whether field importance has to be baked into duplicated text
- tokenizer, stemming, stopword, and PF2e vocabulary behavior
- field-sensitive stemming/stopword policy: test conservative title/name/alias/mechanical-label indexes against more aggressively normalized body/rules indexes, and document whether one Ladybug FTS config per index is sufficient or whether separate projection/indexes are needed
- phrase/prefix/fuzzy/typo-tolerance support or gaps
- snippets/highlighting/debuggability for "why did this match?"
- whether lexical ranking can be fused cleanly with graph and vector scores
- whether LadybugDB FTS is good enough as the only lexical path, or better treated as a graph-entry aid

Reject a LadybugDB-only search design if it materially weakens lexical quality, ranking control, or match explanations compared with the SQLite FTS and Tantivy spike baselines.

## Vector Tradeoffs To Test

LadybugDB vector search is a better conceptual fit than FTS because PF2e Atlas already searches embedding units and collapses them to parent records. The expected graph model is `EmbeddingUnit` nodes connected to `Record` nodes.

The hard question is filtered vector search. The spike must prove that LadybugDB can search only the eligible semantic unit set for a filter, not run top-k globally and post-filter afterward.

Test selective filters including:

- record-family filters
- rank/level ranges
- trait filters through relationships
- source/publication filters
- visibility filters
- metric filters
- relationship-derived filters
- filters on parent records while vectors live on child `EmbeddingUnit` nodes

The spike must report:

- how projected graphs are created, reused, scoped, and cleaned up
- whether projected-graph creation is compatible with interactive search latency
- how many projected graphs would be needed for common UI workflows
- whether multiple embedding units per record and weighted chunk collapse remain straightforward
- whether HNSW approximate search recall is acceptable for small and selective candidate sets
- whether exact vector fallback is needed for very selective filters

Reject or defer LadybugDB vector search if exact PF2e filters require brittle overfetch/post-filter behavior or if projected-graph management is too expensive for interactive use.

## Filter Counts And Discovery

The spike must preserve product affordances from the old Node implementation and the current Rust discovery model:

- count records matching the current filter set
- count facet values under a partial filter set
- discover available values for traits, source, family, rank/level, rarity, metrics, and future derived concepts
- support numeric summaries where useful, such as min/max/count for level, rank, metric values, or prices
- distinguish default-visible search records from direct-lookup-only records

LadybugDB can express `count` and grouped counts in Cypher, but the spike must determine whether these should be computed live, precomputed as catalog nodes/properties, cached by the runtime, or retained in a SQL companion artifact.

Report the implementation shape for:

- "how many records match this filter set?"
- "for this current filter set, what trait values are available and how many records would each add or narrow to?"
- "which filters are applicable for this record family?"
- "which metrics are searchable for this family?"
- "what diagnostics prove the discovery counts are complete and not drifting from stored records?"

## Prototype Shape

- Keep the tiny hand-authored graph fixture only as an installation and extension smoke test.
- Build the meaningful prototype through a parallel Ladybug writer in `atlas-ingest`.
- Start with a bounded representative PF2e slice emitted from the ingest pipeline.
- Reuse vectors from an existing compatible embedding cache artifact during the spike to avoid embedding recomputation.
- Keep SQLite results available as a comparison baseline, but do not use SQLite tables as the primary source of Ladybug graph facts.
- Keep persistent Ladybug FTS/vector index creation separate from graph artifact writing until the reopen failure with extension indexes is isolated.
- Do not flatten the graph back into generic JSON blobs just to make ingest easier; the spike should test whether graph modeling itself helps.

## Decision Criteria

Treat LadybugDB as a serious candidate only if it can:

- model PF2e relationships more naturally than the current SQL edge tables
- support exact structured filtering for search
- combine graph predicates with FTS and vector retrieval in a product-usable way
- preserve simple key lookup and deterministic record hydration
- make graph context or relationship-aware search materially better
- preserve filter counts, facet counts, and discovery workflows needed by UI and CLI consumers
- identify a credible lexical-search strategy, whether LadybugDB-only or LadybugDB plus a specialist index
- prove filtered vector search without default overfetch/post-filter semantics
- support local offline artifact distribution and validation

Prefer SQLite or another SQL-shaped backend if LadybugDB mainly recreates relational tables in graph syntax, if FTS/vector behavior is too limited, or if validation and packaging become materially harder.

## Outputs

- Proposed PF2e graph schema.
- Minimal reproduction commands and fixture data.
- Notes on FTS, vector, filter, and graph-query behavior.
- Explicit product-capability assessment: what LadybugDB unlocks, what it weakens, and what must stay in another artifact if anything.
- Filter-count and discovery assessment for old-Node-style UI workflows.
- Comparison against the current SQLite artifact contract.
- Recommendation: graph-native replacement candidate, companion graph/search artifact, or reject for now.

## Current Findings: Bulk Writes And Query Probe

The initial one-query-at-a-time Ladybug writer is not viable for full-corpus evaluation. A full build with row-by-row Cypher writes took about 86 minutes for the Ladybug output, with graph relationships alone taking about 57 minutes.

The writer now uses Parquet staging files and Ladybug `COPY FROM` for all modeled node and relationship tables. On the same full corpus, Ladybug output completed in about 83 seconds:

- embedding collection from an existing cache artifact: about 0.8s
- schema creation: about 0.1s
- Parquet staging file generation: about 8.7s
- Parquet `COPY FROM` for all node and relationship tables: about 25.2s
- FTS/vector index creation plus checkpoint: remainder of the 83s output phase

This means bulk loading is mandatory for any credible Ladybug implementation, but the bulk path is promising. The spike toy confirmed that Parquet can load scalar node tables, relationship tables, nullable relationship properties, large text, and fixed-size vector columns. The full ingest writer then confirmed the same shape against 29,674 records, 82,382 content units, 73,239 reference edges, and 26,402 copied legacy embedding units.

The `atlas-ladybug-spike query-artifact` mode probes the full artifact. Current full-artifact measurements from `.cache/ladybug-spike/bulk-scratch.lbug`:

- record count: 29,674 records, about 0.7ms execution
- default-visible count: 28,525 records, about 2ms execution
- key lookup for Fear: about 1.2ms execution
- structured filter count for common rank <= 3 spells: 438 records, about 7.3ms execution
- facet count for traits under that spell filter: about 13.3ms execution
- metric range filter for AC >= 30: 1,977 records, about 23.7ms execution
- graph out-neighborhood for Fear references: about 5.3ms execution
- backlinks to Frightened: about 38.2ms execution
- FTS `frightened condition`: about 98ms execution
- FTS with structured spell/rank filter: about 61ms execution
- vector query over copied legacy embeddings: about 260ms execution
- vector query with creature structured filter after top-k vector search: about 112ms execution

Important caveat: the current vector probe uses a synthetic one-hot query vector. It proves vector index plumbing, graph hydration from `EmbeddingUnit` to `Record`, and post-vector structured filtering mechanics, but it does not prove semantic ranking quality. The next vector probe should either reuse a real query vector from `atlas-embedding` or copy a known document vector and use it as a self-query.

## Current Findings: Product Search Read Path

The first product-facing Ladybug CLI comparisons were distorted by a reader-path bug, not by Ladybug FTS/vector query cost. `AtlasRetrievalService::text_search` always calls identity resolution before ranked FTS/vector/hybrid search. The initial Ladybug `load_record_set` implementation hydrated the entire corpus, including record content, metrics, supplemental content, aliases, references, and remaster links, just to test exact-name/alias matches.

An env-gated trace helper (`ATLAS_SEARCH_TRACE=1`) showed the issue clearly on `fireball --retrieval fts --family spell`:

- before direct identity lookup: `resolve_record` took about 37.3s, while `query_fts_index` took about 62ms
- after direct Ladybug identity lookup: `resolve_record` took about 438ms, `query_fts_index` took about 64ms, and `text_search_total` took about 742ms

The spike now adds a `SearchIndex::resolve_record_matches` hook. SQLite keeps the default existing path; Ladybug implements direct graph lookups for exact name, normalized name, alias, and variant name. This preserves the product service boundary while avoiding full graph hydration for ordinary ranked searches.

Current product-facing comparisons against `.cache/ladybug-spike/with-embeddings.*` after that fix:

- FTS `fireball` filtered to spells:
  - SQLite: about 0.7s `text_search_total`
  - Ladybug: about 0.75s `text_search_total`
  - Result set parity is good; all five records match, with only minor ranked-order differences below the exact identity result.
- Vector `spell that makes enemies afraid` filtered to spells:
  - SQLite: about 0.2s `text_search_total`
  - Ladybug: about 1.1s `text_search_total`
  - Top 10 result quality is currently identical in the observed run.
- Hybrid `monster with grab and constrict` filtered to creatures:
  - SQLite: about 2.6s `text_search_total`, total 385
  - Ladybug: about 1.7s `text_search_total`, total 316
  - Top result is the same, and many top results are plausible, but total and ranking differences need investigation before claiming parity.
- FTS `frightened demoralize`:
  - SQLite with `--fts-top-k 200`: about 2.2s `text_search_total`; `query_fts_index` took about 1.4s
  - Ladybug with `--fts-top-k 200`: about 1.0s `text_search_total`; `query_fts_index` took about 0.09s
  - Ladybug with `--fts-top-k 1000`: about 2.8s `text_search_total`; `query_fts_index` still took only about 0.10s, but `load_ranked_records` grew to about 2.5s because the current read path hydrates the larger candidate window
  - The ranking still looks weaker than SQLite/Tantivy expectations: broad creature/equipment hits can outrank `Demoralize`. This is an FTS quality/tuning issue, not the earlier 38s performance bug.
- FTS prefiltering:
  - A direct probe attempted to mirror the vector projected-graph path by creating a projected `SearchDocument` graph with `PROJECT_GRAPH_CYPHER` and then calling `QUERY_FTS_INDEX('eligible_spell_search_docs', 'search_document_fts', ...)`.
  - The query failed with `Table eligible_spell_search_docs does not exist`.
  - Source-confirmed interpretation: unlike `QUERY_VECTOR_INDEX`, Ladybug `QUERY_FTS_INDEX` targets concrete node tables/FTS indexes, not projected graphs. In `ladybug/extension/vector/src/function/query_hnsw_index.cpp`, the vector binder accepts a table or projected graph name; for projected graphs it binds the graph query as a filter statement and the planner pushes it into a semi-mask before HNSW search. In `ladybug/extension/fts/src/function/query_fts_index.cpp`, the FTS binder immediately calls `FTSIndexUtils::bindNodeTable`, and `ladybug/extension/fts/src/function/fts_index_utils.cpp` validates that target as a real node table with an existing FTS index.
  - FTS internally constructs its own native graph over the extension-managed `terms`, `docs`, and `appears_in` tables, then scores docs from that internal graph. There is no user projected-graph input equivalent to vector's `filterStatement`.
  - A materialized filtered node table can be indexed and queried with FTS, so coarse per-scope FTS indexes are possible. Arbitrary UI filter trees would still require either many materialized indexes, query-time overfetch/post-filtering, or upstream Ladybug FTS work to add a projected-graph/filter-statement path comparable to vector.
  - The current Ladybug FTS path is therefore FTS top-k followed by graph/record filtering unless a per-scope materialized index strategy or upstream extension change is adopted.
  - This is a real semantic gap from SQLite FTS, where the structured filter is part of the candidate query.

Important embedded-mode caveat: concurrent Ladybug CLI reads against the same `.lbug` path failed with a file-lock error. The spike should explicitly determine whether this is expected embedded Ladybug behavior, whether read-only/concurrent-open configuration exists, or whether a production Atlas shape would need a process-local shared handle, serialized access, copied per-query artifacts, or a sidecar/server mode.

## Current Findings: Graph Opportunity Probe

The `atlas-ladybug-spike graph-opportunities` mode probes whether the full Ladybug graph unlocks product behavior that is meaningfully better than rewriting the current SQLite edge tables in Cypher.

Current command:

```bash
LBUG_RUST_BUILD_FROM_SOURCE=1 cargo run -p atlas-ladybug-spike -- graph-opportunities .cache/ladybug-spike/bulk-scratch.lbug
```

The strongest product opportunities so far are:

- Mechanic hub and impact maps. The most-referenced default-visible records are rules and conditions such as Off-Guard, Slowed, Concealed, Frightened, Stunned, Prone, Sickened, Enfeebled, Stupefied, Grabbed, Invisible, and Clumsy. This is possible in SQLite, but the graph model makes "show me the rules surface around this mechanic" a first-class product shape instead of a reporting query over an edge table.
- Impact analysis by source family and edge kind. For Frightened, the graph can quickly show that most references come from creature embedded item descriptions, then feats, equipment, hazards, spells, and rules. This supports product experiences like "what would be affected if this condition or rule changed?" and "where does this mechanic actually appear?"
- Mechanic bridge search. Querying for records that connect Demoralize and Frightened produced meaningful mixed-family results: pregenerated characters, creatures, equipment, and related abilities. SQLite can express this through repeated joins over `reference_edges`, but graph traversal matches the product language more directly and should be easier to extend to path explanations.
- Related-mechanic browsing. Co-referenced records around Frightened surfaced adjacent conditions such as Fleeing, Stunned, Slowed, Concealed, Enfeebled, Invisible, Prone, Stupefied, Off-Guard, Sickened, Confused, Hidden, Immobilized, Blinded, Grabbed, and Drained. This looks useful for concept-centric browsing, but raw co-reference also includes noise from broad rule/effect references.
- Relationship-based "more like this". Raw shared-reference similarity for Fear was noisy because common referenced targets dominate. Combining shared traits with shared referenced targets produced much better results, including Horrific Visage, Invoke Spirits, Phantasmal Killer, Vision of Death, Weird, Roar of the Dragon, Clear Mind, Frightful Attrition, Grim Swagger, Grudge, Meditate on This!, Owl Screech Egg, Panic the Dead, and Scare to Death. This is the clearest graph-native search opportunity found so far: ranking records by multiple relationship signals and explaining the result through shared paths.
- Graph diagnostics for model tuning. Aggregating embedded-item reference edges showed that creature-to-rule and creature-to-spell edges dominate the graph. This is useful product and engineering feedback: default graph search probably needs edge-source weighting, visibility modes, and maybe "expanded graph" toggles so embedded content does not overwhelm authored top-level relationships.

The main negative finding is that graph value is not automatic. Single-hop backlinks, simple hub counts, and many facet-like queries are already possible and faster in SQLite. Ladybug becomes interesting when queries combine multiple relationship signals, path evidence, edge metadata, and concept neighborhoods. Raw graph counts are often noisy, so a credible product path likely needs:

- edge weights by `source_kind`, visibility, and record family
- hub downranking for extremely common targets
- clearer concept nodes for conditions, actions, damage types, and future derived tags
- query templates that return both results and explanation paths
- side-by-side result-quality comparison against SQLite FTS, semantic search, and current graph context

The next useful graph-product step is a small relevance harness with 5-10 curated product questions, expected good answers, and query variants. The goal should be to decide whether graph ranking and graph explanations produce product-worthy results, not just whether Cypher can produce rows.

## Current Findings: Graph Relevance Harness

The spike now has a narrower relevance harness for product-shaped graph questions:

```bash
LBUG_RUST_BUILD_FROM_SOURCE=1 cargo run -p atlas-ladybug-spike -- graph-relevance .cache/ladybug-spike/bulk-scratch.lbug
```

Initial full-corpus observations:

- "Records mechanically similar to Fear" using shared traits plus shared referenced rules produced strong top results: Horrific Visage, Invoke Spirits, Phantasmal Killer, Vision of Death, Weird, Roar of the Dragon, Clear Mind, Frightful Attrition, Grim Swagger, Grudge, Meditate on This!, Owl Screech Egg, Panic the Dead, and related fear spells. This is the strongest evidence so far that multi-signal graph ranking can produce product-worthy "more like this" results.
- The explanation queries are useful because they can show exactly which traits and referenced rules connect a candidate back to Fear. This is a meaningful product difference from opaque lexical or vector ranking.
- The same explanation query also exposed noise: broad traits such as `concentrate`, `manipulate`, `emotion`, and generic common references can pull in records that are technically connected but not necessarily product-relevant. A serious graph relevance model needs trait weighting, edge-source weighting, and hub downranking.
- "Records bridging Demoralize and Frightened" produced useful mixed-family results across iconics, pregens, creatures, equipment, feats, and spells. This is a concrete example of a graph query that could become a product-facing mechanic explorer.
- "Conditions most associated with Frightened" surfaced plausible neighboring conditions such as Fleeing, Stunned, Slowed, Concealed, Enfeebled, Invisible, Stupefied, Prone, Off-Guard, Sickened, Confused, Hidden, Immobilized, Blinded, Grabbed, and Drained. It also surfaced non-condition rule records such as Spell Effect: Shield and Escape because the current query scopes to `record_family = 'rule'`, not to a true condition concept. This supports adding explicit concept/type nodes rather than relying on broad record families.
- "Frightened impact by record family" confirmed that creature embedded-item descriptions dominate references, followed by feat descriptions, equipment descriptions, hazard embedded descriptions, spells, and rules. This is useful for impact analysis but means default graph search must decide how much embedded content should influence ranking.
- "Fear trait plus Frightened reference" produced a rich mechanic set across afflictions, equipment, feats, and spells. This looks promising for concept pages like "fear effects that inflict Frightened", but it needs product-level grouping and family filters to avoid overwhelming users.

The early conclusion is that Ladybug's graph value is real only above the one-hop level. Simple backlinks and counts are not enough. The promising shape is a graph relevance layer that combines multiple relationship signals and returns explanation paths, with explicit weighting to avoid common-trait and common-reference noise.

Additional graph-product probes broadened the opportunity set beyond "records like this one":

- Variant and progression browsing looks immediately useful. Querying `VariantGroup` nodes surfaced families with broad level spreads such as Aeon Stone, Elixir of Life, Life Shot, Numbing Tonic, Draugr, healing potions, locks, manacles, wands, and ammunition families. A focused Dread Ampoule query returned Lesser, Moderate, Greater, and Major as a clean level-ordered progression. This is not impossible in SQL, but the graph model makes "show the progression/family this record belongs to" a natural navigation primitive rather than a separate grouping feature.
- Remaster relationship analysis is a distinct product opportunity. Remaster pairs such as Djinni -> Jaathoom, Efreeti -> Ifrit, Fire Mephit -> Fire Scamp, Faerie Fire -> Revealing Light, and Dragging Strike -> Sleek Reposition can be compared through shared referenced mechanics. This points toward "what changed across remaster?" and "show me mechanically equivalent/renamed content" experiences.
- Alias ambiguity can become an explicit resolver UX. Grouping aliases by normalized text found ambiguous names such as attack of opportunity, abundant step, empty body, inspire competence, ki strike, planar binding, quivering palm, stance savant, wild empathy, and wild shape. This is useful for lookup disambiguation and "why did this resolve?" explanations. The first detail query using `shield` showed duplicate alias rows for Steel Shield rather than meaningful ambiguity, so this needs de-duplication and better example selection.
- Source concentration maps are promising. For Frightened, the query surfaced books/sources that concentrate the mechanic, including Lost Omens Draconic Codex, Monster Core 2, Monster Core, NPC Core, Kingmaker, Book of the Dead, Treasure Vault, Battlecry!, Player Core 2, and Season of Ghosts. This could support "where does this mechanic live?" browsing and source-pack impact analysis.
- Mechanic ecology over creatures is promising but needs modeling care. For creatures with Frightened abilities, trait and AC summaries showed clusters such as evil, humanoid, unholy, chaotic, human, undead, dragon, fiend, lawful, incorporeal, aberration, spirit, fey, occult, beast, and demon with observed AC ranges. This suggests product surfaces like "what creature families tend to use this mechanic?" or encounter-building aids, but broad alignment traits may need weighting or family-specific filters.
- Content-unit evidence is powerful but currently too noisy. Asking for records connected through content units showed exactly which embedded item labels carried a relationship, but character records produced huge equipment/feat lists. This confirms that content units can support great explanations, but the model needs evidence edges from specific content units to targets, not only record-level references plus all content units on the record.

These probes suggest several graph-native product directions:

- "More like this" ranked by relationship evidence, not just text or vector similarity.
- Mechanic pages with related mechanics, source concentration, record-family distribution, and evidence paths.
- Variant/progression navigation for items, creatures, spells, and generated variants.
- Remaster comparison and migration views.
- Lookup disambiguation based on alias nodes plus relationship context.
- Encounter/design ecology views such as "creatures that use this condition and what traits/levels/metrics they cluster around."
- Evidence-backed explanations that show which embedded item, content unit, trait, reference, or source path caused a result.

The strongest modeling lesson from this round is that record-level graph edges are not enough for the richest explanations. To make evidence-backed search excellent, references should probably be attributable to `ContentUnit` or another evidence node, so the UI can say "this matched because this specific embedded ability references Frightened" instead of "this character record has many embedded things and one of them references Frightened."

## Current Findings: Content-Unit Evidence Edges

The Ladybug writer now emits `CONTENT_REFERENCES` edges from `ContentUnit` nodes to referenced `Record` nodes for supplemental content whose resolved content references already exist during ingest. This does not yet cover primary description/blurb text because those are not modeled as `ContentUnit` nodes in the current graph, but it proves the embedded/supplemental evidence shape.

The new edge shape is:

```text
Record -[:HAS_CONTENT_UNIT]-> ContentUnit -[:CONTENT_REFERENCES]-> Record
```

The full-corpus no-embedding build succeeded with the new table:

```bash
LBUG_RUST_BUILD_FROM_SOURCE=1 cargo run -p atlas-cli -- index build --progress always \
  --source /Users/ekosten/projects/pathfinder-mcp/pathfinder-2e-foundry-mcp/vendor/pf2e \
  --output .cache/ladybug-spike/evidence-scratch.sqlite \
  --ladybug-output .cache/ladybug-spike/evidence-scratch.lbug \
  --no-embeddings
```

Observed build details:

- `CONTENT_REFERENCES` copied from Parquet in about 630ms.
- Ladybug Parquet staging generation took about 7.8s.
- Ladybug table copy and checkpoint took about 19.9s after staging began.
- Total no-embedding build, including SQLite artifact work, took about 101s.

The relevance harness now runs against the evidence artifact:

```bash
LBUG_RUST_BUILD_FROM_SOURCE=1 cargo run -p atlas-ladybug-spike -- graph-relevance .cache/ladybug-spike/evidence-scratch.lbug
```

The direct evidence query is a clear improvement over record-level references. Instead of saying only that a character or creature references Frightened somewhere, it can return the specific evidence unit:

- Amiri -> Demoralize -> Frightened 2
- Lem -> Dread Ampoule / Fear -> Frightened 1
- Ulka -> Commander's Banner -> Frightened 1
- 287'S Powerful Ghost -> Frightful Moan -> Frightened 2
- Abrikandilu -> Fear -> Frightened 1
- Adamantine Dragon -> Mask of Terror -> Frightened 2
- Adrivallo -> Dirge of Doom / Fear -> Frightened 1

Evidence counts for Frightened also became more precise:

- creature embedded item descriptions: 1,416 evidence units across 1,134 records
- hazard embedded item descriptions: 84 evidence units across 83 records
- character embedded item descriptions: 40 evidence units across 32 records
- hazard routines: 24 evidence units across 24 records

Most importantly, a same-content-unit bridge query for Demoralize plus Frightened reduced the broad record-level result set to concrete evidence units:

- Zathri -> Antagonize
- Cyclops Bully -> Terrorizing Swing
- Hu Ban-Niang -> Want to Try Me?

This is a materially stronger product shape than "records that reference both Demoralize and Frightened." It answers "which specific ability or embedded rule connects these mechanics?" and gives the UI a displayable explanation path.

Open modeling follow-up:

- Model primary description and blurb as content/evidence units too, or add a parallel evidence-node shape for primary text.
- Decide whether record-level `REFERENCES` should be retained as a derived accelerator from evidence edges, or whether graph search should start from evidence edges and aggregate to records.
- Add source-kind and evidence-kind weighting so embedded equipment on pregenerated characters does not dominate ordinary graph relevance.
- Consider whether FTS/snippet generation should point at `ContentUnit` evidence rather than only `SearchDocument`.

## Current Findings: EvidenceUnit Layer

The graph now has an explicit `EvidenceUnit` layer. This is the candidate long-term provenance unit for graph explanations, evidence-level FTS, and optional embedding linkage:

```text
Record -[:HAS_EVIDENCE_UNIT]-> EvidenceUnit
EvidenceUnit -[:EVIDENCE_REFERENCES]-> Record
EvidenceUnit -[:HAS_EVIDENCE_EMBEDDING]-> EmbeddingUnit
```

This is different from the earlier `ContentUnit` experiment. `ContentUnit` only covered supplemental/default content that was already modeled as a content unit. `EvidenceUnit` covers primary record description, blurb, supplemental content, and embedded content through one query surface.

Current implementation shape:

- Primary description and blurb become `EvidenceUnit` nodes.
- Supplemental embedded content becomes one `EvidenceUnit` per content unit, preserving the useful "this specific ability/item/spell caused the match" behavior.
- Non-embedded supplemental content and primary text are split by explicit headings where possible, using the same section-tree machinery used by embedding preparation. When a heading section corresponds to an embedding child unit, the evidence key uses the same key shape so it can link to the embedding later.
- If content is not split or is too small for embeddings, it still becomes evidence. This is important because graph explanations and FTS snippets should not disappear just because semantic embeddings are disabled or skipped.
- `HAS_EVIDENCE_EMBEDDING` is optional. The no-embedding build intentionally produced zero embedding links while still producing the evidence graph.
- `EvidenceUnit` has `label`, `search_text`, `source_kind`, `visibility`, and `unit_kind`, so it is a natural future target for evidence-level FTS and snippet generation.

The full-corpus no-embedding build succeeded with this model:

```bash
LBUG_RUST_BUILD_FROM_SOURCE=1 cargo run -p atlas-cli -- index build --progress always \
  --source /Users/ekosten/projects/pathfinder-mcp/pathfinder-2e-foundry-mcp/vendor/pf2e \
  --output .cache/ladybug-spike/evidence-unit-scratch.sqlite \
  --ladybug-output .cache/ladybug-spike/evidence-unit-scratch.lbug \
  --no-embeddings
```

Observed build details:

- Parquet staging generation, including `EvidenceUnit` and relationships, took about 16.1s.
- `EvidenceUnit` copied from Parquet in about 3.3s.
- `HAS_EVIDENCE_UNIT` copied in about 570ms.
- `EVIDENCE_REFERENCES` copied in about 934ms.
- The Ladybug Parquet copy/checkpoint phase took about 33.1s total.
- Total no-embedding build, including SQLite artifact work, took about 114.6s.

The relevance harness now runs against the evidence-unit artifact:

```bash
LBUG_RUST_BUILD_FROM_SOURCE=1 cargo run -p atlas-ladybug-spike -- graph-relevance .cache/ladybug-spike/evidence-unit-scratch.lbug
```

Evidence-unit queries confirmed the model covers both embedded content and top-level record text:

- `Frightened` evidence came from creature embedded item descriptions, feats, equipment, hazards, spells, rules, character embedded item descriptions, afflictions, and primary description documents.
- The source mix query found 1,416 creature embedded-item evidence units across 1,134 records, 100 feat description evidence units across 100 records, 97 equipment description evidence units across 97 records, 84 hazard embedded-item evidence units across 83 records, and 63 spell description evidence units across 63 records.
- A same-evidence-unit Demoralize plus Frightened query returned both embedded-content matches and top-level text matches. Embedded examples stayed precise: Zathri -> Antagonize, Cyclops Bully -> Terrorizing Swing, and Hu Ban-Niang -> Want to Try Me?. Top-level text examples included Draconic Verge, Lion's Armor, Antagonize, Brandish Authority, Fear of God, Gorilla Pound, Tut-Tut, and Belittling Boast.
- Optional embedding linkage was correctly empty in the no-embedding build. The harness counted evidence units by source kind and confirmed all `embedded_units` counts were `0`, which is the expected result for this build mode.

This looks useful for FTS too. The current SQLite/Tantivy projection already has a concept of "search document prep", but the UI often needs a smaller displayable reason than a whole record-level search document. `EvidenceUnit` can become the common addressable chunk for:

- graph references and path explanations
- semantic embedding provenance
- FTS indexed text and snippets
- "why did this match?" UI rows
- evidence-level weighting by source kind, visibility, record family, and unit kind

The practical benefit is that FTS, semantic search, and graph traversal can all converge on the same evidence boundary even when they use different indexes internally. A search result can still collapse to `Record`, but the explanation can point to an `EvidenceUnit` instead of saying only that the record matched somewhere.

Open modeling follow-up:

- Decide whether `ContentUnit` remains as raw supplemental-content storage or whether it should collapse into `EvidenceUnit` after the spike.
- Decide whether `SearchDocument` remains a record-level projection, becomes a derived aggregate from `EvidenceUnit`, or is replaced by evidence-level FTS plus record collapse.
- Run the same evidence-unit build with embeddings enabled or copied from SQLite so `HAS_EVIDENCE_EMBEDDING` can be validated with real rows.
- Add source-kind weighting in graph relevance queries so embedded character inventories and very common rule references do not dominate.
- Compare evidence-level FTS quality against current SQLite FTS projection fields before deciding whether this is a real replacement path.

## Current Findings: General Graph Product Evaluation

The spike now has a more opinionated general-purpose graph product evaluation mode:

```bash
LBUG_RUST_BUILD_FROM_SOURCE=1 cargo run -p atlas-ladybug-spike -- graph-product-eval .cache/ladybug-spike/evidence-unit-scratch.lbug
```

This is not a performance benchmark. It is a product-value harness that labels each question as `strong`, `mixed`, or `weak` depending on whether the graph is doing work that looks product-novel rather than merely rewriting a SQLite query in Cypher.

Current result: the graph is clearly useful as a relationship-relevance and explanation layer, but it has not proven that it should replace SQLite for the whole core artifact.

Strong graph cases:

- Relationship-ranked "more like this" for Fear. Combining shared traits and shared referenced rules surfaced strong fear-adjacent results such as Horrific Visage, Invoke Spirits, Phantasmal Killer, Vision of Death, Weird, Roar of the Dragon, Clear Mind, Frightful Attrition, Grim Swagger, Grudge, Meditate on This!, Owl Screech Egg, Panic the Dead, and Agonizing Despair. This is product-shaped because the result can be explained through graph paths, not just ranked by opaque text/vector similarity.
- Same-evidence mechanic bridge queries. Asking for evidence units that reference both Demoralize and Frightened returned specific abilities or text units, including Zathri -> Antagonize, Cyclops Bully -> Terrorizing Swing, Hu Ban-Niang -> Want to Try Me?, plus top-level description matches such as Draconic Verge, Lion's Armor, Antagonize, Brandish Authority, Fear of God, Gorilla Pound, Tut-Tut, and Belittling Boast. This is the clearest evidence-unit win because the UI can answer "which specific thing connects these mechanics?"
- Variant progression navigation. The Dread Ampoule progression query returned Lesser, Moderate, Greater, and Major in level order. This is not impossible in SQL, but it is naturally graph-shaped product navigation: from this item, walk to its family/progression.

Mixed graph cases:

- Mechanic impact maps. Frightened evidence-source counts are useful for product pages and impact analysis, but they are count/facet-shaped. SQLite can express the same basic question well.
- Mechanic neighborhoods. Co-referenced rules around Frightened produced plausible adjacent conditions such as Fleeing, Stunned, Slowed, Concealed, Enfeebled, Invisible, Stupefied, Prone, Off-Guard, and Sickened. The shape is promising, but common references and broad record families need weighting or explicit `Concept` nodes before the output is reliably product-grade.
- Remaster relationship comparison. Shared referenced mechanics across remaster pairs produced meaningful examples such as Djinni -> Jaathoom, Efreeti -> Ifrit, Fire Mephit -> Fire Scamp, Faerie Fire -> Revealing Light, and Dragging Strike -> Sleek Reposition. This could become useful, but the model likely needs diff-specific facts to become more than relationship context.
- Alias ambiguity resolver candidates. Alias nodes expose ambiguity candidates such as attack of opportunity, abundant step, empty body, inspire competence, ki strike, planar binding, and quivering palm. The graph shape is useful, but the current data needs de-duplication and resolver-specific ranking.
- Creature mechanic ecology. Traits and AC ranges around creatures that reference Frightened produced plausible clusters such as humanoid, undead, dragon, fiend, incorporeal, aberration, spirit, fey, and occult, but broad alignment traits dominated. This needs weighting or curated concept taxonomy.

Weak graph cases:

- Plain structured counts, direct lookup, and flat facets. The harness included a visible low-level spell count and it worked, but there is nothing graph-novel about it. SQLite remains the better baseline fit for these questions.

Current product read:

- The graph is most compelling when the user asks a relationship-first question and the UI needs an explanation path.
- `EvidenceUnit` materially improves the product case because it lets graph search collapse to records while explaining the exact text/ability/section that matched.
- The graph is not yet compelling as a general replacement for SQLite's core strengths: direct lookup, structured filtering, result counts, and ordinary facets.
- The strongest architecture path may be graph-assisted search, graph-native product surfaces, or a graph artifact alongside SQLite unless future FTS/vector/filter tests show Ladybug can also match the current core search ergonomics.

## Current Findings: Baseline Parity Evaluation

The spike now separates "is graph novel?" from "can Ladybug preserve boring-but-critical SQLite behavior?" with a baseline parity harness:

```bash
LBUG_RUST_BUILD_FROM_SOURCE=1 cargo run -p atlas-ladybug-spike -- baseline-parity .cache/ladybug-spike/evidence-unit-scratch.lbug
```

The harness classifies cases as:

- `parity-clean`: Ladybug expresses the current SQLite-shaped need naturally.
- `parity-awkward`: possible, but the query/model is more awkward than SQLite.
- `parity-blocked`: not proven with this artifact or currently failing.
- `potentially-better`: preserves baseline behavior while improving evidence/provenance shape.

Current result: baseline structured behavior looks mostly clean. The important weak/non-differentiating cases from the graph product evaluation are not Ladybug failures; they are simply not graph-specific reasons to choose Ladybug.

Parity-clean cases:

- Direct record-key lookup worked naturally:
  - `Fear` resolved by `record_key` with family `spell`, level `1`, rarity `common`.
- Scalar structured filters worked naturally:
  - visible common rank 0-3 spells returned `438`.
- Facet counts under partial filters worked naturally:
  - for visible common rank 0-3 spells, top traits included manipulate, concentrate, cantrip, mental, illusion, attack, earth, fire, water, wood, air, emotion, auditory, and visual.
- Relationship-backed filters worked naturally:
  - `HAS_TRAIT` queries over `fear` plus family/level filters returned equipment, feats, and spells without duplicating trait values onto search nodes.
- Open-ended metric range filters worked naturally:
  - creatures with `ac.value` between 30 and 35 returned `814` distinct records, proving the current metric side-table idea maps cleanly to graph fact relationships.
- SearchDocument FTS with structured filters worked:
  - the harness loaded the FTS extension, created/reused `search_document_fts`, queried `frightened condition`, joined FTS candidates back to parent `Record`, and applied family/level/default-visibility filters.

Potentially-better cases:

- Evidence/reference search with structured filters worked:
  - the query kept structured filters on `Record` while traversing `Record -> EvidenceUnit -> referenced Record`.
  - This preserves the SQLite-style shared filter behavior while adding evidence provenance.
- Evidence-hit collapse back to distinct records worked:
  - evidence units referencing Frightened collapsed to records with evidence hit counts, which can feed ranking and explanations.
- Facet counts under an evidence/reference filter worked:
  - the same eligible set can feed trait facets, which is important for "given this filtered search, how many records match this facet?" UI behavior.
- EvidenceUnit FTS with structured filters and record collapse worked:
  - the harness loaded/created `evidence_unit_fts`, queried `frightened demoralize`, joined hits back to parent `Record`, applied record-family/default-visibility filters, and returned match provenance such as Gendarme -> Stop in the Name of the Law!, Fear of God -> Description, Hu Ban-Niang -> Want to Try Me?, Antagonize -> Description, and Tut-Tut -> Description.

Parity-blocked cases:

- Semantic/vector parity is not evaluated by this artifact:
  - the current evidence-unit artifact was built with `--no-embeddings`;
  - the harness correctly reported `0` records and `0` embedding units.
  - This is not a Ladybug vector failure; it means the next parity artifact needs embeddings copied or generated before vector search can be judged.

No `parity-awkward` cases were confirmed in this run, but that does not mean none exist. The likely awkwardness risks are still:

- expressing the full Atlas filter tree ergonomically in Cypher;
- preserving current UI count/facet behavior across every filter operator;
- keeping FTS/vector candidate generation and record-level filtering in one maintainable query shape;
- matching or beating SQLite performance once the full query set is exercised.

Current baseline read:

- Ladybug appears capable of preserving the shared structured-filter idea that made SQLite attractive: search units can be joined back to `Record`, and structured filters can remain on canonical record/relationship facts.
- FTS is viable in the basic shape, including both record-level `SearchDocument` FTS and evidence-level `EvidenceUnit` FTS.
- Evidence-level FTS is particularly interesting because it can return snippets/provenance while still collapsing to records.
- Vector/semantic parity remains the next necessary proof point.

### Parallel Backend Shape

The spike now has the first cut of the architectural shape needed to test Ladybug through product-like paths without making the CLI or runtime responsible for comparison logic.

Ingest side:

- `atlas-ingest` has a narrow `ArtifactOutputWriter` sink boundary.
- The current SQLite writer is wrapped as `SqliteArtifactOutput`.
- The Ladybug spike writer is wrapped as `LadybugArtifactOutput`.
- `atlas-ingest::build` prepares the normalized source and embeddings once, then fans out to the configured artifact outputs.
- This intentionally stops at the final artifact-write boundary. It does not try to invent a generic table/graph writer because SQLite and Ladybug should remain free to use different durable models.

Search side:

- `atlas-index` owns the `SearchIndex` capability trait for the operations the product retrieval service actually needs.
- `atlas_index::SqliteIndexReader` implements the trait, so existing SQLite behavior still routes through the same retrieval service.
- `atlas_index::LadybugIndexReader` implements the same trait for the Ladybug spike artifact, keeping Cypher, graph projection, FTS, and vector-index details out of CLI/runtime code while avoiding a separate asymmetric read crate.
- `AtlasRetrievalService` now stores a boxed `SearchIndex`, with compatibility constructors for the current SQLite path and explicit constructors for future backend implementations.
- `atlas search` can select the read backend with `--index-backend sqlite|ladybug`; `--ladybug-index` overrides the Ladybug artifact path.
- `atlas record get` and `atlas record resolve` can now select the read backend with `--index-backend sqlite|ladybug`; `--ladybug-index` overrides the Ladybug artifact path. Graph context remains SQLite-owned for now.
- The first Ladybug read path supports core spike cases: record hydration from graph node properties, filter-only record listing, record-family/level/rarity/pack/scalar text/scalar boolean/trait/repeated fact/simple metric filters, FTS through `SearchDocument`, vector search through `EmbeddingUnit`, and prefiltered vector search through a projected eligible embedding graph.
- Record hydration is now being pushed toward full parity rather than a deliberately minimal summary path. The Ladybug `Record` node stores the durable presentation fields needed for search results, including traits, prerequisites, system fields, pricing/timing, publication metadata, content JSON, taxonomy/variant data, raw JSON, and actor/item/spell side-data projections. The reader also hydrates reference edges, remaster links, supplemental content units, and metric rows from graph relationships.
- Remaining parity gaps should be treated as design questions to prove, then strip back only if the graph model makes them unnecessary. The spike still needs to decide which rich record fields should remain `Record` properties, which should become graph facts, and which can stay presentation JSON.
- Unsupported filter shapes return explicit errors rather than falling back to silent post-filter behavior.

Comparison should remain outside runtime as a harness over stable JSON command output. Runtime should select and open one backend; it should not become the place where SQLite and Ladybug are benchmarked against each other.

### Filter Discovery / UI Count Parity

The spike now routes `atlas filters fields` and `atlas filters values` through the same `atlas-search` backend boundary as search. The CLI can select the discovery backend with `--index-backend sqlite|ladybug` and `--ladybug-index`.

For fair dynamic-vs-dynamic comparison, the SQLite CLI path can force live discovery with:

```bash
target/debug/atlas filters values \
  --index .cache/ladybug-spike/with-embeddings.sqlite \
  --disable-discovery-catalog \
  --field traits \
  --family spell \
  --level 1..3 \
  --json \
  --progress never
```

This matters because UI filter panels need non-search structural questions, not just ranked search:

- "given this filter set, how many records remain?"
- "which filter fields apply under this filter set?"
- "which values are available for this field, and how many records would each value represent?"
- "what numeric ranges/statistics exist for this field or metric under the current filter?"

Current Ladybug implementation:

- Uses graph facts dynamically rather than SQLite's precomputed discovery catalogs.
- Supports the first graph-backed discovery slice:
  - scalar `Record` properties such as family, level, rarity, publication fields, action/cost/range/duration fields, variant fields, and boolean fields;
  - relationship-backed fields for `pack_name`, `pack_label`, and `traits`;
  - repeated graph facts through `FilterValue(field, value)` nodes and `HAS_FILTER_VALUE` relationships for `taxonomy_families`, `traditions`, `spell_kinds`, `damage_types`, `languages`, `speed_types`, `senses`, `immunities`, `resistances`, `weaknesses`, `disable_skills`, and `variant_axes`;
  - metric key lookup and metric numeric/boolean/text value payloads through `Record -[:HAS_METRIC]-> Metric` relationships.
- Returns `execution = dynamic` and `catalog_available = false` for Ladybug field discovery.
- The old row-by-row Cypher writer was removed from the spike implementation. Ladybug writes now go through Parquet staging and `COPY FROM`, including repeated filter facts.

Spot-check parity against `.cache/ladybug-spike/with-embeddings.*`:

- Filter-only search for `family=spell AND level=1..3`:
  - SQLite total: `944`;
  - Ladybug total: `944`;
  - both returned `500 Toads` as the first alphabetical result.
- Trait values for `family=spell`:
  - SQLite and Ladybug both returned `1,795` matching records;
  - top counts matched, including `concentrate=1453`, `manipulate=1435`, `focus=457`, `mental=279`;
  - `null_count=68` matched.
- Trait values for `family=spell AND level=1..3`:
  - SQLite and Ladybug both returned `944` matching records;
  - top counts matched, including `concentrate=755`, `manipulate=754`, `focus=268`, `mental=137`;
  - `null_count=31` matched.
- Numeric stats for `level` under `family=spell`:
  - SQLite and Ladybug both returned `count=1795`, `min=1`, `p50=3`, `mean=3.652924791086351`, `p95=9`, `max=10`.
- Metric stats for `ac.value` under `family=creature`:
  - SQLite and Ladybug both returned `count=6161`, `min=0`, `p50=24`, `mean=25.51290374939133`, `p95=43`, `max=54`.
- Rebuilt no-embeddings parity artifact after adding graph facts:
  - command shape: `target/debug/atlas index build --source vendor/pf2e --output .cache/ladybug-spike/parity-noemb.sqlite --ladybug-output .cache/ladybug-spike/parity-noemb.lbug --no-embeddings`;
  - total build time: about `3m 11s`;
  - Ladybug Parquet staging plus copy/checkpoint: about `1m 50s`;
  - `traditions` values for `family=spell` returned `arcane=788`, `occult=636`, `primal=595`, `divine=446`, `null_count=654`;
  - `family=spell AND traditions includes arcane` returned `788` records;
  - `family=spell AND (trait fire OR trait healing)` returned `141` records, proving the corrected same-field `AnyOf` relationship filter shape.

Rough local timings through the CLI binary, before disabling SQLite catalogs:

- Filter-only search `family=spell AND level=1..3 LIMIT 1`:
  - SQLite: about `0.06s`;
  - Ladybug: about `0.78s`.
- Trait facet for `family=spell AND level=1..3`:
  - SQLite: about `0.09s`;
  - Ladybug: about `0.35s`.
- Numeric level stats for `family=spell`:
  - Ladybug: about `0.31s`;
  - SQLite timings varied during local runs, but remained catalog-backed and generally lower-cost.
- Metric `ac.value` stats for `family=creature`:
  - SQLite: about `0.60s` in one local run;
  - Ladybug: about `0.36s` after warmup.

Rough local timings with SQLite discovery catalogs disabled:

- `filters fields --family spell`:
  - SQLite dynamic: about `0.87s`;
  - Ladybug dynamic: about `0.44s` in an earlier run.
- Trait facet for `family=spell AND level=1..3`:
  - SQLite dynamic: about `3.31s`;
  - Ladybug dynamic: about `0.35s`.
- Numeric level stats for `family=spell`:
  - SQLite dynamic: about `0.09s`;
  - Ladybug dynamic: about `0.31s`.
- Metric `ac.value` stats for `family=creature`:
  - SQLite dynamic: about `0.60s`;
  - Ladybug dynamic: about `0.36s`.

These are CLI wall-clock timings from one local machine, not benchmark-grade numbers. They are still directionally useful: the SQLite catalog path is very strong for the production UI, but the live SQLite dynamic path is not uniformly faster than Ladybug. Relationship-heavy facets such as `traits` can be notably expensive when SQLite has to recompute them dynamically through the generic eligible-record CTE shape.

Current read:

- Correctness is promising for graph-backed structural fields: Ladybug can answer the UI count/facet/stat questions through the same reader boundary, without post-filtering in application memory.
- SQLite remains a stronger default for catalog-style discovery because it has precomputed discovery tables and richer metric metadata.
- When catalogs are disabled, the comparison is more mixed: Ladybug dynamic graph traversal looks competitive or faster for some relationship-backed UI facets, while SQLite remains very fast for simple scalar numeric stats.
- Ladybug's dynamic graph discovery may be good enough for many UI cases, but needs a broader benchmark across real filter trees before claiming parity.
- Ladybug `Metric` nodes are now self-describing: ingest writes `namespace_prefix`, `label`, `short_label`, `group_name`, and `known` from the shared `atlas-record` metric definitions. This preserves the existing SQLite metric path-collapse semantics while making metric browsing graph-native.
- If Ladybug remains interesting, the next modeling task is to continue promoting currently JSON-only repeated fields into graph facts where the UI expects facet/filter behavior.
- Embedded-mode concurrency is still an operational caveat: parallel Ladybug reads can hit file locking, so benchmark harnesses should run Ladybug commands serially or evaluate a sidecar/server strategy.

### With-Embeddings Parity Follow-Up

The same baseline harness was rerun against a full embedding artifact:

```bash
LBUG_RUST_BUILD_FROM_SOURCE=1 ATLAS_LADYBUG_CREATE_SEARCH_INDEXES=1 \
  cargo run -p atlas-cli -- index build --progress always \
  --source /Users/ekosten/projects/pathfinder-mcp/pathfinder-2e-foundry-mcp/vendor/pf2e \
  --output .cache/ladybug-spike/with-embeddings.sqlite \
  --ladybug-output .cache/ladybug-spike/with-embeddings.lbug

LBUG_RUST_BUILD_FROM_SOURCE=1 cargo run -p atlas-ladybug-spike -- baseline-parity .cache/ladybug-spike/with-embeddings.lbug
```

Observed build details:

- SQLite/Ladybug build wrote `29,674` records.
- Embedding generation produced `29,379` document embeddings.
- `584` embedding inputs were truncated at `max_tokens=512`; the maximum observed token count was `1,736`.
- Ladybug checkpoint completed in about `122.7s` after the Ladybug write phase.
- Total embedding build duration was about `1,084.8s`.

Semantic/vector parity results:

- `EmbeddingUnit` rows were present:
  - `28,525` records with embeddings;
  - `29,379` embedding units;
  - dimensions were consistently `384`.
- `EvidenceUnit -> EmbeddingUnit` links existed but were sparse:
  - `854` evidence units linked to `854` embedding units.
  - This is expected for the current key-matching implementation: exact evidence-to-embedding provenance only exists where the evidence key intentionally matches an embedding child unit key. It proves the link shape, not full coverage.
- Vector extension and vector index were usable:
  - the harness loaded `VECTOR`;
  - queried `embedding_hnsw`;
  - returned vector candidates and collapsed them back to parent records through `HAS_EMBEDDING_UNIT`.
- Vector search with post-filtered structured filters worked:
  - querying top-k `EmbeddingUnit` candidates and then filtering parent `Record.record_family = 'creature'` returned creature records.
- Prefiltered vector search through a projected graph worked:
  - the harness used `PROJECT_GRAPH_CYPHER` to create `eligible_creature_embeddings` from this Cypher:
    - `MATCH (record:Record)-[:HAS_EMBEDDING_UNIT]->(embedding:EmbeddingUnit) WHERE record.is_default_visible AND record.record_family = 'creature' RETURN embedding`
  - it then called `QUERY_VECTOR_INDEX('eligible_creature_embeddings', 'embedding_hnsw', ...)`.
  - all returned rows were creature records, and the nearest results differed from the post-filtered top-k query, proving the filter was applied before vector top-k rather than after it.
  - example top results included Festering Gnasher, Grioth Cultist, Cassisian, Glitterspore, Ovinnik, Stony Goat, Gogiteth, Graelar the Whisper, Gnokesh, Gylou, Stony Bat, Totenmaske, Gnagrif, Grodair, and Golem.

Important limitation:

- Global vector top-k plus parent-record filtering remains `parity-awkward`, because it can miss good filtered candidates.
- `PROJECT_GRAPH_CYPHER` prefiltered vector search is now `parity-clean` for this simple structured filter.
- The next proof point is not basic feasibility; it is whether projected-graph creation/reuse remains fast and manageable for realistic Atlas filter trees and interactive UI workflows.

Updated baseline read:

- FTS parity is now stronger: both `SearchDocument` FTS and `EvidenceUnit` FTS work with parent-record structured filters.
- Semantic/vector write, index creation, query, and record collapse are proven on the full corpus.
- Semantic/vector structured filtering is proven for one important shape: `Record`-level structured filter -> eligible `EmbeddingUnit` projected graph -> vector top-k.
- This is the closest Ladybug result so far to SQLite's shared record-key pruning behavior for semantic search.
- Remaining risk is operational: projection lifecycle, projection latency, cache strategy, and coverage of the full Atlas filter tree.

### Next Parity Evaluation Targets

Two evaluation tracks should run before drawing a stronger conclusion about Ladybug feature parity:

1. Vector parity for realistic filter trees:
   - `family + level + trait`;
   - `family + metric range`;
   - publication/remaster filters;
   - reference/graph-derived filters.
   The important measurement is whether `PROJECT_GRAPH_CYPHER` creation/reuse lets Ladybug apply the structured filter before vector `top_k`, and whether projection lifecycle cost is acceptable for interactive search.

2. Record hydration audit:
   - compare SQLite JSON output to Ladybug JSON output for representative records across `spell`, `feat`, `equipment`, `creature`, `hazard`, `rule`, and generated `affliction`;
   - look for missing side data, content sections, metric fields, aliases, remaster links, references, and presentation fields;
   - keep this as a harness outside runtime/product code so it validates the CLI-facing reader boundary without baking comparison logic into production retrieval.

The spike harness now has a hydration audit mode:

```bash
LBUG_RUST_BUILD_FROM_SOURCE=1 cargo run -p atlas-ladybug-spike -- hydration-audit \
  .cache/ladybug-spike/parity-noemb.sqlite \
  .cache/ladybug-spike/parity-noemb.lbug \
  target/debug/atlas
```

The harness runs `atlas record get --detail full --json` against both backends and reports structural JSON differences per representative record. Ladybug reads should remain serial in this harness because embedded-mode concurrent artifact access can hit file locks.

Initial hydration audit result against `.cache/ladybug-spike/parity-noemb.*`:

- `spell`: parity-clean;
- `feat`: parity-clean;
- `equipment`: parity-clean;
- `creature`: parity-clean;
- `hazard`: parity-clean;
- `rule`: parity-clean;
- generated `affliction`: parity-clean.

This means the current Ladybug record retrieval path can reproduce the full CLI-facing JSON projection for representative records. It does not prove every corner case, but it clears the main risk that the Ladybug read path is missing ordinary content sections, details, presentation fields, or generated-record hydration.

The spike harness also has a vector filter parity mode:

```bash
LBUG_RUST_BUILD_FROM_SOURCE=1 cargo run -p atlas-ladybug-spike -- vector-filter-parity \
  .cache/ladybug-spike/with-embeddings.lbug
```

Initial vector filter parity result against `.cache/ladybug-spike/with-embeddings.lbug`:

- `family + level + trait`: projected eligible embedding graph in about `19.3ms`; vector query in about `78.8ms`;
- `family + metric range`: projected eligible embedding graph in about `5.4ms`; vector query in about `138.0ms`;
- `publication/remaster filter`: projected eligible embedding graph in about `7.9ms`; vector query in about `75.9ms`;
- `reference-derived filter`: projected eligible embedding graph in about `18.6ms`; vector query in about `197.8ms`.

These numbers are still single-run local timings, but they materially improve the vector parity read: realistic structured filters can be applied before vector `top_k` by projecting the eligible `EmbeddingUnit` graph first. This makes Ladybug much closer to SQLite's shared record-key pruning behavior for semantic search than the earlier post-filtered vector probe suggested.

Remaining vector questions:

- whether projection names should be cached by canonical filter tree hash or recreated per request;
- whether concurrent projected-graph creation works in embedded mode or needs serialization;
- whether projection reuse invalidation is simple enough for rebuilt immutable artifacts;
- whether the same approach remains acceptable for larger `top_k`, higher `efs`, and broad multi-family filters.

The spike harness now also has a search parity and quality mode:

```bash
LBUG_RUST_BUILD_FROM_SOURCE=1 cargo run -p atlas-ladybug-spike -- search-eval \
  .cache/ladybug-spike/with-embeddings.sqlite \
  .cache/ladybug-spike/with-embeddings.lbug \
  target/debug/atlas
```

This harness drives the real `atlas search` CLI against SQLite and Ladybug, using the same query, retrieval mode, filters, JSON output, and result hydration path for each backend. Timings are end-to-end CLI wall time, including process startup and embedding model load for vector/hybrid queries, so they are useful for direction and regression detection but are not benchmark-grade backend timings.

Initial search parity result against `.cache/ladybug-spike/with-embeddings.*`:

- Direct lexical title quality is preserved. `battle medicine` returned `Battle Medicine` first on both backends, with `8/10` top-result overlap. Ladybug was faster in this single CLI run, but the main conclusion is that direct title behavior survived.
- Filtered lexical spell quality is preserved for the direct hit. `fear --family spell --max-level 3` returned `Fear` first on both backends. Result totals differed materially: SQLite reported `26`, Ladybug reported `6`. Treat this as an FTS/filter semantics question, not only a speed question.
- Semantic/vector concept search is the strongest parity result. `low level spell that makes enemies afraid --family spell --max-level 3 --retrieval vector` returned the same top five and `10/10` top-result overlap on both backends. `Dirge of Doom` and `Fear` appeared at positions `2` and `3` in both.
- Realistic structured vector-only search is now parity-clean. `healing spell --family spell --max-level 3 --trait healing --retrieval vector`, `high armor creature --family creature --metric ac.value>=25 --retrieval vector`, `fear --family spell --publication-title "Pathfinder Player Core" --retrieval vector`, and `frightened --family spell --references conditionitems:TBSHQspnbcqxsmjL --retrieval vector` all produced `10/10` top-result overlap between SQLite and Ladybug. The metric case had minor near-tie ordering differences, but the candidate set matched.
- Hybrid concept search is not yet parity-clean. The same query under `--retrieval hybrid` had only `1/10` top-result overlap. SQLite kept `Menacing Lament`, `Dirge of Doom`, and related fear spells near the top; Ladybug put weak lexical-looking results such as `Shadow Projectile`, `Curse of Recoil`, `Biting Words`, and `Shielded Arm` above the strong semantic results. This reinforces the FTS/RRF risk: Ladybug hybrid quality depends on lexical confidence gating, score fusion tuning, or query-intent-aware FTS weighting.
- Actual CLI hybrid search now works for relationship-backed structured vector filters after changing the Ladybug reader to build a single coherent projected-graph pattern for `Record -> EmbeddingUnit` plus relationship filters. `healing spell --family spell --max-level 3 --trait healing --retrieval hybrid` and `high armor creature --family creature --metric ac.value>=25 --retrieval hybrid` both return results instead of failing in the vector lane. The remaining issue is ranking/result parity, not basic execution.
- Publication-filtered lexical search preserves the direct hit. `fear --family spell --publication-title "Pathfinder Player Core"` returned `Fear` first on both backends, but SQLite reported `13` total results and Ladybug reported `3`.
- Reference-derived FTS filtering is promising. `frightened --family spell --references conditionitems:TBSHQspnbcqxsmjL` produced `9/10` top-result overlap, with different ordering and totals (`63` SQLite vs `34` Ladybug).

Current search-quality read:

- Ladybug vector search is parity-clean across the realistic filter cases tested so far. This is the strongest evidence that Ladybug can preserve SQLite's shared structured-filter-before-semantic-top-k behavior.
- Ladybug FTS is good enough for direct/title-ish entry points in these cases, but result totals and ranking semantics differ from SQLite enough that it needs explicit quality tuning before it can be the only lexical backend.
- Ladybug hybrid search is currently the weakest search surface. The issue is not just performance; weak lexical candidates can dominate or distort hybrid ranking unless the FTS lane is treated as precision-oriented evidence.
- The next blocker is search quality, especially hybrid ranking and FTS candidate semantics. Vector-only results are strong; the major remaining divergence appears when the lexical lane participates.

The spike harness now also has a broad non-FTS CLI parity mode:

```bash
LBUG_RUST_BUILD_FROM_SOURCE=1 cargo run -p atlas-ladybug-spike -- cli-parity \
  .cache/ladybug-spike/metric-metadata.sqlite \
  .cache/ladybug-spike/metric-metadata.lbug \
  target/debug/atlas
```

This mode intentionally avoids FTS quality questions and drives the real CLI surfaces for direct record lookup, strict record resolution, filter-only search, filter discovery, metric discovery, concrete metric stats, and vector-only search. The current case matrix covers:

- record get for spell, equipment, creature, and hazard records across full/preview/standard/summary detail levels;
- strict `record resolve` for a filtered exact spell name;
- filter-only search for family/level, pagination, required trait, any-trait OR, numeric metric range, price range plus price sort, outgoing reference filters, and reverse reference filters;
- filter field discovery for spell and creature families;
- enumerable filter value discovery for traits, rarity, and high-cardinality publication titles;
- numeric filter stats for level and price;
- boolean filter counts for sustained spells;
- metric-key discovery and concrete `ac.value` numeric stats;
- vector-only search with scalar and reference-derived structured filters when an embedding artifact is supplied.

Because the current local artifacts are split by age, the cleanest run is currently two-part:

- `.cache/ladybug-spike/metric-metadata.*` has current graph schema and no embeddings. On that artifact, all non-vector cases above are parity-clean against SQLite. Vector cases are blocked only because this artifact intentionally lacks embedding indexes.
- `.cache/ladybug-spike/with-embeddings.*` has embedding indexes but was built before the latest graph discovery schema. On that artifact, record lookup, resolve, filter-only search, enum/numeric/boolean discovery that does not need the newer graph tables, and vector-only structured searches are parity-clean. Discovery cases that need newer `FilterValue` and metric metadata nodes are blocked by artifact staleness rather than by the current code.

Parity gaps found by this mode are now fixed in the Ladybug reader:

- metric-key discovery now reports SQLite-compatible `namespace_prefix` values and includes numeric stats for number metrics. This matters for UI filter panels because metric discovery is not just a key list; SQLite exposes count/min/percentile/max data in the same response, and Ladybug should preserve that contract.
- price sorting is now implemented for filter-only search, matching SQLite's non-null-first `price_cp`, normalized-name, record-key ordering.

### Metric Metadata Parity Follow-Up

SQLite and Ladybug both receive metrics after the shared ingest metric extractor has collapsed multiple raw Foundry paths into canonical metric keys. Examples include multiple perception modifier paths collapsing to `actor.perception.mod`, and multiple shield HP paths collapsing to `item.shield.hp`.

The Ladybug graph now stores the shared definition metadata directly on `Metric` nodes:

- `namespace_prefix`;
- `label`;
- `short_label`;
- `group_name`;
- `known`.

This makes the graph self-describing without changing the source of truth: values are still derived from `atlas-record::metrics` definitions during ingest. A fresh no-embedding artifact verified that Ladybug metric discovery returns `actor.ac.value` with `label = "Armor Class"`, `short_label = "AC"`, `group = "defense"`, and `known = true`, and that `--metric AC` resolves to the canonical `actor.ac.value` metric.

### FTS OR/RRF Risk Follow-Up

The spike harness now has an FTS RRF risk mode:

```bash
LBUG_RUST_BUILD_FROM_SOURCE=1 cargo run -p atlas-ladybug-spike -- fts-rrf-risk \
  .cache/ladybug-spike/metric-metadata.lbug
```

The purpose is to inspect whether broad OR-style FTS candidates would contribute weak lexical votes during RRF. It compares broad OR queries to space-separated token queries and reports token coverage for the top candidates.

Initial result:

- `treat wounds` broad OR had `5/10` top results with only one apparent token match, while the space-separated query had `0/10`.
- `frightened condition` broad OR had `9/10` top results with only one apparent token match, while the space-separated query had `0/10`.
- `fire damage` broad OR had `7/10` top results with only one apparent token match, while the space-separated query had `0/10`.
- `armor class` broad OR had `6/10` top results with only one apparent token match, while the space-separated query had `0/10`.
- `low level fear spell` broad OR had `6/10` top results with only one apparent token match. The space-separated query improved coverage but did not behave as a hard all-token AND for longer natural-language queries.

Important interpretation caveat: the harness counts exact normalized token overlap in the returned document text, while Ladybug FTS applies its analyzer/stemmer. Some apparent zero-token or one-token matches are likely analyzer matches such as `treat` matching `treated`. The direction is still useful: broad OR can admit weak lexical candidates high enough that they could affect RRF.

Current recommendation:

- Treat broad OR as a recall fallback, not the primary FTS lane for hybrid search.
- Prefer strict or near-strict FTS candidates for RRF contribution.
- If Ladybug does not expose a reliable all-token query mode, add an application-side token coverage gate before contributing the FTS lane to RRF:
  - 2-token queries should generally require both tokens unless a high-value title/alias match exists.
  - 3+ token queries should require at least 2 tokens or a high-value title/alias match.
  - Semantic/vector search should carry broad conceptual recall instead of letting weak OR FTS candidates vote in RRF.

### Score-Fusion Follow-Up

The search path now has an experimental `--fusion min-max-score` mode. It keeps the same FTS and vector candidate windows as `weighted-rrf`, but it combines normalized lane scores instead of rank positions:

- FTS scores are normalized within the returned FTS lane. The implementation infers whether the backend's first result is higher-is-better or lower-is-better from the first and last returned score, so it can handle SQLite's lower-is-better BM25-style rank and Ladybug's higher-is-better FTS score.
- Vector `rank_distance` is normalized as lower-is-better.
- Flat multi-result lanes contribute `0` rather than amplifying meaningless tiny differences; a single-result lane contributes `1`.
- Existing `--fts-weight` and `--vector-weight` apply to normalized lane scores. `--rank-constant` remains accepted for CLI compatibility but only affects fallback scoring if a lane score cannot be normalized.

Initial smoke checks on `.cache/ladybug-spike/with-embeddings.*`:

```bash
target/debug/atlas --progress never search "treat wounds" \
  --retrieval hybrid --fusion min-max-score \
  --index-backend ladybug \
  --index .cache/ladybug-spike/with-embeddings.sqlite \
  --ladybug-index .cache/ladybug-spike/with-embeddings.lbug \
  --limit 5 --fts-top-k 50 --vector-top-k 50 --json --explain
```

For `treat wounds`, min-max score fusion promoted the top semantic/lexical overlap (`Effect: Treat Wounds Immunity`) above pure FTS-only noise, but a high-scoring FTS-only result still entered the top results. For `low level fear spell --family spell`, equal-weight min-max still allowed a bad top FTS-only hit (`Shielded Arm`) to rank second because its FTS score was the lane maximum. Reducing FTS weight (`--fts-weight 0.25 --vector-weight 1.0`) produced a more semantic result set with `Fear`, `Fearful Feast`, and other fear-related spells at the top.

Interpretation: raw score fusion is useful for testing and exposes confidence gaps that RRF hides, but it does not solve weak OR FTS by itself. If broad OR stays in the FTS lane, the better product shape is likely:

- FTS is a precision lane with token-coverage or title/alias gates.
- Semantic search owns broad natural-language recall.
- Score fusion is tuned with lower FTS weight for conceptual queries, or FTS contributes only when lexical confidence passes a query-analysis threshold.

The search path now also has an experimental FTS confidence gate:

```bash
--fts-fusion-policy all|demote-weak|strong-only
```

The classifier currently emits:

- `direct-title`: query tokens directly cover the record title, or the title is fully contained in the query.
- `strong-lexical`: all or most query tokens match high-value record fields such as title, traits, taxonomy, prerequisites, category, or group.
- `medium-lexical`: partial high-value lexical overlap.
- `weak-lexical`: broad OR/body-style lexical evidence only.

Policy behavior:

- `all`: preserves existing behavior.
- `demote-weak`: keeps weak candidates but reduces weak FTS contribution and medium FTS contribution.
- `strong-only`: only lets direct/strong FTS evidence contribute; weak or medium FTS-only rows with zero contribution are removed from the fused pool. Rows that also have vector evidence can still rank through the vector lane.

Current live-query read from both Ladybug and SQLite:

- Direct/near-direct queries are protected by identity and direct-title evidence. `treat wounds`, `battle medicine`, and `fear --family spell` keep the direct record at the top.
- `weighted-rrf + demote-weak` is the steadier conservative setting for title-ish and lexical queries. It cleans up obvious broad-OR noise without over-preferring semantic-only candidates.
- `min-max-score + demote-weak` is better for natural-language concept queries. It improved `spell that makes enemies afraid --family spell`, `low level fear spell --family spell`, and `healing feat --family feat` by letting vector evidence outrank weak lexical rows.
- `min-max-score + strong-only` is useful as a diagnostic upper bound for treating FTS as precision-only, but it can suppress medium lexical evidence too aggressively. It should not become the default without a better query-intent classifier.

Most promising next shape:

- Default hybrid should probably be `weighted-rrf + demote-weak`, or a tuned score fusion once query intent is understood.
- Conceptual/natural-language queries should lean toward `min-max-score + demote-weak`.
- Long-term, the retrieval layer likely needs query intent detection: exact/direct lexical queries should preserve strong FTS influence, while broad natural-language queries should demote FTS unless title/alias/high-value coverage is strong.

### Architecture Alignment Before Further Feature Work

The Ladybug spike has crossed the threshold where continued feature work on large scratch-shaped files is risky. Current alignment target:

- Keep `atlas-search` as the product retrieval orchestrator, but split reusable search policy into focused modules:
  - query analysis/tokenization;
  - FTS/vector fusion and confidence policy;
  - semantic vector collapse/ranking.
- Keep Ladybug read behavior behind the same `SearchIndex` boundary used by SQLite. The Ladybug index crate should be split by read concern:
  - connection/query helpers;
  - filter compilation;
  - record hydration;
  - FTS;
  - vector;
  - graph/reference reads;
  - discovery/facet reads.
- Keep Ladybug ingest as an ingest-owned artifact writer path, but split the current writer by build phase:
  - schema creation;
  - Parquet staging DTOs and writers;
  - node staging;
  - relationship staging;
  - FTS/vector index creation;
  - publication/checkpoint lifecycle and instrumentation.
- Keep `atlas-ladybug-spike` as a harness for experiments only. It should be split by probe family rather than becoming a product runtime boundary:
  - install/capability;
  - bulk load;
  - FTS;
  - vector/filter;
  - graph novelty;
  - fusion/ranking comparison.

Completed alignment slices:

- `atlas-search` fusion, query analysis, and semantic collapse were extracted from `lib.rs` without changing behavior. This makes future ranking work easier to reason about and keeps the service facade closer to the intended architecture.
- `atlas-index::ladybug::reader` was split by read concern:
  - `filter.rs` owns graph filter/projection lowering;
  - `row.rs` owns Ladybug row decoding and hydration helpers;
  - `search.rs` owns FTS and vector query execution;
  - `graph.rs` owns graph/reference/remaster reads;
  - `discovery.rs` owns filter discovery and facet/stat queries;
  - `reader.rs` remains the backend read facade, `SearchIndex` implementation, connection lifecycle, and record orchestration.
- The separate `atlas-ladybug-index` crate was folded into `atlas-index` so read-side backend ownership is symmetric: `atlas-index` now contains the shared retrieval contract plus both SQLite and Ladybug backend implementations. `atlas-search` remains the product retrieval orchestrator and re-exports the shared types for compatibility.
- Embedding cache hydration now follows that same backend boundary: `atlas-index` exposes `DocumentEmbeddingCacheReader`, `SqliteIndexReader` implements it for `document_embedding_cache`, and `atlas-ingest` consumes the trait while retaining the embedding reuse/generation policy.
- The write-side handoff is now explicit: `atlas-index` exposes `IndexBuildInput`/`IndexBuildPack` as the backend-neutral normalized payload for index writers, while `atlas-ingest` converts from ingest-only `SourceLoad` after enrichment and embedding generation. SQLite and Ladybug writers now live in `atlas-index` and consume that handoff instead of reaching into ingest construction state.
- Ladybug embedding reuse no longer has a legacy SQLite escape hatch in the writer. Reuse goes through the same `DocumentEmbeddingCacheReader` boundary as SQLite, so a previously built Ladybug artifact can act as the embedding cache source when its embedding metadata matches the active model and embedding-unit policy.
- `atlas-index::ladybug::writer` owns the Ladybug writer split:
  - `embeddings.rs` owns Ladybug embedding-unit collection from the backend-neutral index build input;
  - `evidence.rs` owns graph evidence-unit construction from content documents and embedding-section policy;
  - `facts.rs` owns shared graph fact/key helpers used by node and relationship staging;
  - `nodes.rs` owns Parquet staging for graph node tables;
  - `output.rs` owns progress, output publication, temp path, cleanup, and move lifecycle;
  - `parquet.rs` owns generic Arrow/Parquet staging helpers, staging directory setup, and `COPY FROM` execution;
  - `relationships.rs` owns Parquet staging for graph relationship tables;
  - `schema.rs` owns Ladybug schema and search/vector index creation;
  - `orchestrator.rs` is now the high-level Ladybug write orchestrator: collect embeddings, create schema, emit staging files, bulk-copy them, build optional search indexes, checkpoint, and publish.
