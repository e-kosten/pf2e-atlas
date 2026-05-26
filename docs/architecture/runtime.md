# Runtime Architecture

This document describes the Rust workspace architecture for deterministic ingest, artifact validation, local CLI workflows, lexical and semantic search, graph context retrieval, first-party agent skill workflows, and future Rust TUI/tagging surfaces.

The Rust architecture is deliberately crate-oriented. Crates should expose only the public API needed by adjacent owners, and ingest/build-time policy should not leak into runtime query or presentation crates.

## System Shape

```mermaid
flowchart TD
    source["Foundry PF2E JSON<br/>vendor/pf2e"] --> ingest["atlas-ingest<br/>source load, normalization,<br/>enrichment, artifact build"]
    ingest --> artifact["SQLite artifact<br/>records, content, FTS,<br/>relationships, embeddings,<br/>vector index,<br/>GraphQLite projection"]

    skill["PF2e Atlas agent skill"] --> cli["atlas-cli<br/>commands, JSON/text output,<br/>exit codes,<br/>agent skill installation"]
    cli --> runtime["atlas-runtime<br/>path and setup policy"]
    runtime --> search["atlas-search<br/>AtlasRetrievalService"]
    runtime --> index["atlas-index<br/>SqliteIndexReader read handle"]

    search --> index
    search --> embedding["atlas-embedding<br/>query vectors, document units,<br/>model catalog"]
    index --> artifact
    ingest --> sqliteVec["atlas-sqlite-vec<br/>sqlite-vec capability"]
    index --> sqliteVec
    sqliteVec --> artifact

    subgraph SharedRustModels["Shared Rust models"]
      domain["atlas-domain<br/>request/filter/output vocabulary"]
      discovery["atlas-discovery<br/>filter discovery field policy"]
      record["atlas-record<br/>normalized records, ContentDocument,<br/>presentation and projections"]
      artifactSchema["atlas-artifact<br/>SQLite schema descriptors<br/>and contract constants"]
    end

    ingest --> domain
    ingest --> discovery
    ingest --> record
    ingest --> artifactSchema
    index --> domain
    index --> discovery
    index --> record
    index --> artifactSchema
    search --> domain
    search --> embedding
    cli --> domain
```

## Crate Ownership

| Crate | Owns | Should not own |
| --- | --- | --- |
| `atlas-domain` | Shared request/filter/output vocabulary and lightweight semantic primitives. | SQLite DDL, ingest source structs, artifact metadata inventories, CLI formatting, embedding provider config. |
| `atlas-discovery` | Shared filter discovery field policy: field ids, groups, value policies, operators, CLI flag mappings, applicability, default value ordering, and source SQL projections for generated catalogs. | Physical SQLite schema descriptors, artifact metadata, row hydration, CLI presentation, runtime query execution. |
| `atlas-record` | Storage-agnostic normalized records, typed metric definitions and labels, `ContentDocument`, rich-content renderers, reference graph policy, reference traversal, section-tree projection, FTS projection, and `RecordPresentationDocument`. | Foundry HTML/macro parsing, SQLite names, validation diagnostics, CLI envelopes, embedding model execution. |
| `atlas-artifact` | Physical SQLite table/column descriptors, descriptor-owned schema DDL, artifact metadata keys, schema SQL helpers, table contract constants, and SQLite vector-blob encoding. | Record normalization, writer policy, row hydration, user-facing search behavior, embedding model behavior. |
| `atlas-ingest` | Source loading, Foundry-specific parsing, normalization, Foundry metric source specs and metric extraction with definition validation, generated records, aliases/remaster links, reference resolution, retrieval visibility, embedding reuse policy and embedding execution during builds, and conversion from ingest state to backend-neutral index build input. | Public embedding-specific API, runtime query orchestration, CLI presentation, broad crate-root behavior, metric-definition ownership, direct database access, shared backend contracts. |
| `atlas-index` | Index backend contracts plus read and write implementations. The production SQLite backend owns completed-artifact access through `SqliteIndexReader`, artifact construction through `SqliteIndexWriter`, fast artifact readiness checks, deep artifact validation, row readers, internal filter-to-SQL keyset compilation, reference-policy SQL lowering, vector query SQL, reusable embedding-cache reads, inspection summaries, the embedded GraphQLite graph projection, and backend-neutral build handoff types. The spike Ladybug backend owns graph reads through `LadybugIndexReader` and graph artifact writes through `LadybugIndexWriter`. | Query embedding, CLI command presentation, ingest-time normalization policy, metric-definition ownership, product retrieval orchestration. |
| `atlas-embedding` | Model catalog, query/document embedding generation, token budgeting, embedding text rendering, document-unit construction, semantic input hashes, and embedding-specific public types. | Foundry raw markup parsing, artifact schema ownership, SQLite vector byte layout, search result collapse policy. |
| `atlas-search` | Product-facing retrieval orchestration through `AtlasRetrievalService`, lexical/semantic composition, vector-hit collapse, and search ranking modes over read-only index handles. | Opening source files, building artifacts, loading models in CLI code, SQLite schema definitions, preflight artifact validation. |
| `atlas-runtime` | Repo/global path resolution, setup policy, setup readiness and repair orchestration, and construction of runtime index/retrieval handles shared by CLI and future Rust surfaces. | Search semantics, artifact schema, source normalization, CLI JSON projection, deep artifact diagnostics. |
| `atlas-cli` | Argument parsing, command routing, terminal/JSON presentation, progress output, exit codes, completions, and agent skill installation. | Durable retrieval semantics, SQLite access policy, embedding provider ownership. |
| `atlas-sqlite-vec` | Unsafe sqlite-vec extension registration and capability boundary. | Domain/search logic or artifact metadata interpretation. |

## Ingest And Artifact Flow

```mermaid
flowchart LR
    raw["Foundry source records<br/>raw JSON + manifest packs"] --> load["atlas-ingest::source<br/>load packs and source signature"]
    load --> normalize["normalize<br/>RecordKey, family, traits,<br/>metrics, side tables"]
    normalize --> content["Foundry content parser<br/>HTML/macros -> ContentDocument"]
    content --> enrich["atlas-ingest::records<br/>aliases, variants, taxonomy,<br/>reference resolution, visibility"]
    enrich --> generated["atlas-ingest::generated<br/>source-backed generated afflictions"]
    generated --> embedPrep["atlas-ingest::embeddings<br/>prepare/run embedding-owned units"]
    embedPrep --> handoff["atlas-index::IndexBuildInput<br/>normalized build handoff"]
    handoff --> writer["atlas-index writers<br/>SQLite + GraphQLite projection<br/>+ spike Ladybug outputs"]
    writer --> sqlite["Rust SQLite artifact"]

    record["atlas-record<br/>NormalizedRecord + ContentDocument"] -. model .-> normalize
    discovery["atlas-discovery<br/>filter discovery field policy"] -. catalog policy .-> writer
    artifact["atlas-artifact<br/>table descriptors + insert SQL"] -. schema .-> writer
    embedding["atlas-embedding<br/>document units + vectors"] -. owns .-> embedPrep
    sqliteVec["atlas-sqlite-vec<br/>vector table capability"] -. capability .-> writer
```

`atlas-ingest/src/lib.rs` is a thin facade. New ingest behavior belongs under the phase that owns it: `source`, `records`, `generated`, `embeddings`, or `artifact`.

Source normalization emits ingest-only construction facts beside each normalized record. These facts carry source identity such as slugs and compendium-source locators, embedded item identity/provenance/content references, and journal page content parsed from Foundry source JSON. Later ingest phases use those facts for aliases, remaster links, and source-backed generated records instead of reparsing `NormalizedRecord.raw_json`; reference, FTS, and embedding projections continue to consume the normalized `ContentDocument` and supplemental-content outputs produced during normalization. Persisted raw JSON remains provenance/debug input and a future analysis substrate, not the normal construction API between ingest phases.

## Content, Search, And Reference Projections

```mermaid
flowchart TD
    markup["Known Foundry rich-text fields<br/>description, notes, hazard text,<br/>embedded item/spell descriptions"] --> parser["atlas-ingest parser<br/>Foundry HTML/macros"]
    parser --> doc["atlas-record::ContentDocument<br/>blocks, inlines, references,<br/>visibility/source policy"]

    doc --> presentation["RecordPresentationDocument<br/>CLI/TUI-ready rich structure"]
    doc --> fts["RecordFtsProjection<br/>title, aliases, traits,<br/>taxonomy, constraints, mechanics,<br/>source, metrics, headings,<br/>body, facts, references,<br/>embedded_content"]
    doc --> tree["Content section tree<br/>explicit headings,<br/>synthetic run-in labels,<br/>table captions"]
    doc --> refs["Resolved ContentReference nodes"]

    presentation --> parentEmbedding["Embedding parent unit<br/>primary/default content;<br/>embedded capability content excluded"]
    tree --> childEmbedding["Embedding child units<br/>coverage-driven rich groups;<br/>only when parent budget omits content;<br/>unpromoted embedded content excluded"]
    refs --> edges["reference_edges<br/>source_kind + visibility"]
    fts --> recordsFts["records_fts<br/>weighted lexical search"]
```

The durable source of authored rich text is `ContentDocument`, not stripped text and not raw Foundry markup. Plain text, markdown-like CLI output, FTS rows, semantic chunks, and reference edges are projections from content and presentation models.

Default public graph and backlink behavior uses the named reference graph policy in `atlas-record`: public non-embedded reference edges are in the default graph, public embedded edges require an expanded mode, and GM/private/internal edges remain excluded unless a caller explicitly asks for broader visibility. `atlas-index` lowers that policy into SQL predicates over `reference_edges`; the database does not store a separate default-edge boolean.

## Runtime Query Flow

```mermaid
flowchart TD
    skill["PF2e Atlas agent skill"] --> command["atlas-cli command<br/>search, record, graph, index"]
    command --> runtime["atlas-runtime<br/>resolved paths + handles"]
    runtime --> search["atlas-search<br/>AtlasRetrievalService"]
    runtime --> index["atlas-index<br/>SqliteIndexReader"]
    search --> filters["atlas-index internal filter compiler<br/>SearchFilterNode -> eligible records"]
    filters --> sqlite["SQLite artifact"]

    search --> graph["atlas-index reference-edge queries<br/>default graph policy"]
    graph --> sqlite

    search --> lexical["atlas-index lexical SQL<br/>records_fts weighted columns"]
    lexical --> sqlite

    search --> queryVec["atlas-embedding<br/>query text -> vector"]
    queryVec --> vectorSql["atlas-index vector query<br/>eligible document_embedding_cache rowids"]
    vectorSql --> sqliteVec["atlas-sqlite-vec capability"]
    sqliteVec --> sqlite

    graph --> collapse["atlas-search result assembly"]
    lexical --> collapse
    vectorSql --> collapse
    collapse --> output["atlas-cli presentation<br/>JSON or terminal text"]
```

Filters compile to an authoritative SQL keyset before lexical or vector search. SQLite lexical search keeps that keyset in the same query as `records_fts`, applies weighted BM25, then runs index-owned deterministic reranking for strict-token matches, phrase/name containment, type intent, token coverage, and generated-effect downranking. The vector table stays rowid plus vector; filtering metadata remains in normal SQLite tables and is reached through `document_embedding_cache.rowid`.

Graph context retrieval is key-based and one-hop in the V1 Rust CLI. `atlas graph links <record>` routes through `AtlasRetrievalService`, resolves names above the index backend when needed, loads the seed record through the normal record path, asks the graph-product index trait for policy-visible `reference_edges`, applies deterministic edge ordering and unique-neighbor limits, then hydrates only retained neighbor records. `atlas graph uses <record>` is the backlinks-only product surface over the same graph context path. Search relationship flags such as `--referenced-by` remain result-set filters; graph context retrieval returns a local context bundle with edge evidence, counts, and truncation metadata. Relational-friendly graph product commands such as `atlas graph variants <record-or-base-name>` and `atlas graph remaster <record>` use the same graph-product index trait rather than the ranked search trait. Variant lookup first uses normal record-reference resolution and then, only for the variants command, falls back to exact `variant_base_name` group resolution so base names such as `Dread Ampoule` can return their group without becoming global record aliases.

Runtime index access goes through `atlas-index` backend handles. Production SQLite reads go through `SqliteIndexReader`; the Ladybug spike read path goes through `LadybugIndexReader` behind the same `SearchIndex` contract. Construction-time writes also go through `atlas-index`: ingest converts final ingest state into `IndexBuildInput`, chooses configured `IndexArtifactWriter` implementations, and lets each backend writer create, validate, checkpoint, and publish its artifact. The SQLite writer always embeds the current GraphQLite graph projection in the SQLite artifact after the authoritative SQLite tables are committed. Product surfaces route retrieval through `atlas-runtime` and `AtlasRetrievalService`; they do not open SQLite/Ladybug artifacts or assemble retrieval dependencies directly.

Embedding reuse follows the same boundary direction. `atlas-index` exposes backend-specific cached-vector readers such as `DocumentEmbeddingCacheReader` on `SqliteIndexReader`; `atlas-ingest` decides whether reuse is enabled, validates the target build context through the embedding model spec, merges reusable vectors with newly generated embeddings, and passes the resulting generated embeddings through `IndexBuildInput` to index writers.

`atlas-runtime` owns path resolution for source checkouts, embedding model caches, and SQLite artifacts. The default `global` path mode resolves to platform cache install paths; `repo` requires checkout-local contributor paths. CLI path flags are command-local overrides passed into runtime resolution, not persisted configuration. `--index` selects the SQLite artifact for commands that open or repair an artifact, while `atlas index build` uses `--output` for the artifact it writes. If persisted configuration is added later, it should feed runtime path overrides below direct CLI flags rather than changing the meaning of direct path flags.

## Artifact Families

The Rust SQLite artifact is the runtime contract between ingest and search. The authoritative table-family definitions live in [artifact contract](./artifact-contract.md). The current families are:

- artifact identity: `artifact_metadata`
- source packs: `packs`
- canonical records: `records`
- supplemental content: `record_content`
- aliases and remaster links: `record_aliases`, `remaster_links`
- filterable projections: `record_traits`, actor/item/spell side tables
- discovery catalogs: `filter_field_catalog`, `filter_value_catalog`, `filter_sample_catalog`, `filter_numeric_catalog`
- open metrics and catalogs: `record_metrics`, `metric_key_catalog`, `metric_value_catalog`
- reference graph: `reference_edges`
- lexical search: `records_fts`
- semantic cache and vector index: `document_embedding_cache`, `record_vector_index`
- embedded graph projection: GraphQLite-owned node/edge tables plus `graphqlite_*` metadata rows

## Current Gaps And Deferred Shapes

- The future Ratatui workbench is a runtime consumer. It should compose through `atlas-search`, `atlas-index`, `atlas-runtime`, and `atlas-record` rather than opening SQLite or embedding models directly.
- Journal pages and table results are recognized as rich content but are deferred to [Rust content subdocuments for journal pages and table results](../backlog/items/rust-content-subdocuments-journal-table-results.md).
- Derived-tag rows are intentionally deferred until the Rust artifact model has a dedicated derived-tag design.
- Search quality tuning and broader full-corpus parity remain follow-up validation work, not reasons to reintroduce raw JSON scanning or duplicate markup parsing.
