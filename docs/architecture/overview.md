# Architecture overview

PF2e Atlas builds and queries a local SQLite reference artifact from Foundry PF2E
source data. Product surfaces are the `atlas` CLI, the local web service launched
by `atlas web`, and the first-party agent skill installed by `atlas agent skills`.

Read this document first, then the [runtime architecture](./runtime.md),
[artifact contract](./artifact-contract.md), [tagging architecture](./tagging.md)
and [ADR index](./decisions/README.md) for the relevant owner.

## Crate ownership

| Crate | Responsibility |
| --- | --- |
| `atlas-foundry-model` | Generated authored Foundry DTOs, admission/field retention and versioned checked snapshots. No ingest/runtime/SQLite dependency. |
| `atlas-record` | Minimal source-backed record, checked embedded addressing, consumer-required borrowed source views, content interpretation/selection and attributed relationship policies. Storage and presentation neutral. |
| `atlas-domain` | Shared keys, predicates, query catalog/discovery vocabulary, summary facts and passage addresses. No SQLite or source parser ownership. |
| `atlas-embedding` | One pinned BGE model, tokenizer/input budgets, library inference/segmentation, exact vector reuse identity and query/document embeddings. |
| `atlas-ingest` | Source filesystem loading/provenance, admission reporting, explicit content/reference context, index inputs and build-time embedding execution. |
| `atlas-index` | Executable SQLite schema, checked artifact publication/validation, row readers, query catalog/bindings/discovery, parameterized filter SQL and vector queries. |
| `atlas-search` | Concrete retrieval service, strict lookup, browse, lexical/semantic/hybrid root ordering, attributable witnesses, graph, verified edition preference and Suggested variants. |
| `atlas-runtime` | Paths, setup/source-fetch/cache policy, artifact readiness and retrieval service construction. |
| `atlas-local-state` | Durable mutable saved lists and encounters in a separate SQLite database, with independent migrations. |
| `atlas-app-model` | Intentional app DTOs and generated TypeScript contracts for filters, result windows, record content, lists and encounters. |
| `atlas-app-service` | Cross-layer app workflows, bounded retrieval execution, catalog editor presentation, source-based product projections and local-state hydration. |
| `atlas-web` | Thin Axum transport over app-service and static frontend serving. |
| `atlas-cli` | Product grammar, terminal/JSON presentation, progress/exit behavior, web startup and agent skill installation. |
| `atlas-dev` | Private Rust source discovery/audit/artifact inspection. Builds independently of the web UI and Node. |
| `atlas-cli-support` | Shared path/progress argument and JSON/output primitives for both Rust CLIs. No runtime/ingest policy. |
| `atlas-tags` | File-owned tag ontology, assignments, validation and agent contracts. Artifact tag search remains deferred. |
| `atlas-sqlite-vec` | sqlite-vec registration and capability probing. |

`web/atlas-ui` renders generated app contracts with React and Ant Design. Private
TypeScript compiler research/generation lives under `dev-tools`; Rust developer
commands do not dispatch those scripts. Product record identity remains Foundry
pack plus source ID. Embedded documents stay in one checked parent snapshot.

## Data flow

1. Ingest loads the source checkout into admitted authored DTOs with compact
   provenance and developer diagnostics. `SourceBackedRecord` retains a key and
   one DTO; it does not duplicate or normalize every source field.
2. Pure record policies select content and useful facts. Ingest supplies explicit
   locale, audience and reference resolution inputs, prepares sanitized HTML and
   builds query/search/relationship inputs. Embedding independently covers selected
   explanatory prose and compact identity units.
3. Index writes and validates a temporary source-backed artifact, then publishes
   atomically. Checked snapshots are authoritative; named relational query rows,
   HTML caches, lookup evidence and lexical/vector units are rebuildable projections.
4. Runtime resolves paths and constructs concrete search services over read-only
   index readers, with query embeddings only when the chosen workflow needs them.
   Ordinary reads work without the source checkout and never run ingest admission.
5. Search applies SQL eligibility before lexical/vector ranking, aggregates roots
   with attributable witnesses and reports bounded semantic coverage honestly.
6. App-service composes product views and saved-list/encounter workflows. Thin CLI
   and web transports present those views; the browser does not infer source rules
   or gameplay values from raw JSON/prose.

The artifact stores selected sanitized HTML/control caches, not RichDocument or
a second plain-text corpus. html2text formats terminal output at read time.
Precision FTS differs from semantic prose selection. GM/owner prose and DCs are
included; explicit None is excluded. Locale defaults to English at indexing and
can be overridden; changing search locale requires a rebuild. Initial display
uses the artifact locale. Macros remain developer-only; RollTables are products.

## Editing guidance

Keep entrypoints focused on module declarations, intentional re-exports and
composition. Put parsing, projection, storage and workflow logic in cohesive
named modules. Promote shared helpers only for an existing stable concern with
at least two owners; avoid generic utility modules.

Keep source DTO generation and snapshot admission in `atlas-foundry-model`.
Runtime readers decode the checked codec without admitting JSON again. Invalid
members retained by admission are explicit unavailable source values, not a
catch-all product raw-JSON fallback. Do not add a separately maintained family
schema or convert DTOs through the retired normalized AtlasRecord model.

Keep record policies storage agnostic. Add borrowed common/family views only for
actual consumers. Do not add a global embedded-node/status/content inventory to
every record. Content preparation and relationship resolution return separate
derived outputs. Authored source stays authoritative for runtime display.

Keep schema and SQL in index. The tracked schema is the physical source of truth;
Diesel declarations must stay checked against it. Focused raw SQL is appropriate
for FTS5/sqlite-vec/dynamic eligibility/validation. Query catalog metadata must
match executable bindings, never artifact-supplied SQL. CLI CEL and the structured
UI builder lower independently to the shared predicate. Do not invent another
expression parser, filter authority, per-field CLI flag set or metric EAV store.

Keep product retrieval in search. Surfaces do not open SQLite, implement ranking,
scan source clones or load models. Summary reads are body free; explicit detail
decodes one root and batches selected cache reads. Filters and key scopes apply
before KNN/FTS ranking. Search witnesses retain actual owner/field/passage addresses.
Verified remaster preference applies only when both records match the complete
request; variants are separate derived suggestions.

Keep application workflows in app-service. It uses runtime/search/local-state,
not index imports or handcrafted SQL readers. Retain the bounded retrieval
executor rather than serializing all read-only web requests behind one lane.
App-service owns useful catalog control presentation, result windows, semantic
family presentation and hydration. Reference reads project authored facts directly;
encounters compose the existing bounded runtime overlays into one presentation.
There is no generic section/value/profile layout interpreter or second encounter
stat-block response. App-model stays a transient DTO/export boundary; run
`cargo test -p atlas-app-model` and regenerate checked bindings intentionally
with `cargo test -p atlas-app-model export_typescript_bindings -- --ignored`.

Keep local state separate from artifact generation. Unknown saved keys retain
their snapshots. NPC variants/conditions are read-time participant overlays over
authored baselines. New HP uses known integral effective maxima; saved HP,
explicit overrides and edit intent survive rebuilds. Missing values stay unknown.
Do not implement a Foundry world/rule/macro execution engine through presentation.

Keep CLI/web thin and frontend semantics Rust-owned. The browser uses Ant controls
and a maintained structured query builder, generated contracts and backend catalog
operators. It renders sanitized HTML and supported narrow controls, without a
Foundry parser, mechanics inference or CEL editor translation. Follow
[`web/atlas-ui/AGENTS.md`](../../web/atlas-ui/AGENTS.md) and
[frontend guidelines](../../web/atlas-ui/docs/frontend-guidelines.md).

Reuse family presentation components across detail, preview, comparison and
encounter surfaces; feature modules compose their own controls around them.
Repeated generic interactions use shared Ant-based primitives. New components
need a clear responsibility or actual reuse need, rather than creating local
copies of the same layout. [ADR0047](./decisions/0047-semantic-record-presentation.md)
defines the transient presentation boundary.

Architectural replacements land with every call site and matching docs updated.
Generated reference artifacts rebuild directly: no legacy adapter, bridge or
mixed old/new authority. Local-state migration is a separate durable-data concern.
See ADR0046 for the source-backed artifact decision and prior ADRs for history.
