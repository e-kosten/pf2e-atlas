# Architecture overview

PF2e Atlas keeps Foundry reference data and saved lists/encounters in one persistent
SQLite workspace. Product surfaces are the `atlas` CLI, the local web service launched
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
| `atlas-embedding` | Two pinned baseline search profiles, shared segmentation/native input budgets, FastEmbed inference, retained shared query sessions and exact vector reuse identity. |
| `atlas-ingest` | Source filesystem loading/provenance, admission reporting, explicit content/reference context, index inputs and build-time embedding execution. |
| `atlas-storage` | Connection admission, bounded read/write sessions, maintenance ownership, transactional migrations and backups. No domain schema ownership. |
| `atlas-index` | Reference SQL schema, checked publication/validation, incremental projection updates, embedding checkpoints, row readers, query catalog/bindings/discovery, parameterized filter SQL and vector queries. |
| `atlas-search` | Concrete retrieval service, strict lookup, browse, lexical/semantic/hybrid root ordering, attributable witnesses, graph, verified edition preference and Suggested variants. |
| `atlas-runtime` | Paths, setup/source-fetch/cache policy, requested configuration, shared migration composition, reference readiness and bounded typed workspace operations. |
| `atlas-local-state` | Saved-domain SQL and session-bound lists/encounters, revision guards and versioned durable roll receipts. |
| `atlas-dice` | Bounded library-backed dice evaluation and structured outcomes; no source, persistence or UI policy. |
| `atlas-app-model` | Intentional app DTOs and generated TypeScript contracts for filters, search pages, record content, lists and encounters. |
| `atlas-app-service` | Cross-layer app workflows, bounded retrieval execution, catalog editor presentation, source-based product projections and local-state hydration. |
| `atlas-web` | Thin Axum transport over app-service and static frontend serving. |
| `atlas-cli` | Product grammar, terminal/JSON presentation, progress/exit behavior, web startup and agent skill installation. |
| `atlas-dev` | Private Rust source discovery/audit/artifact inspection. Builds independently of the web UI and Node. |
| `atlas-cli-support` | Shared path/progress argument and JSON/output primitives for both Rust CLIs. No runtime/ingest policy. |
| `atlas-tags` | File-owned tag definitions/assignment DTOs and structural validation. Assessment and product filtering are deferred pending demonstrated retrieval gaps (ADR0054). |
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
3. Index reconciles and validates reference changes in one transaction inside the
   existing workspace. Saved state and stable identities survive publication.
   Checked snapshots are authoritative; named relational query rows,
   HTML caches, lookup evidence and lexical/vector units are rebuildable projections.
4. Runtime resolves paths and constructs bounded search sessions. Query embeddings
   are prepared before opening the consuming database snapshot.
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

Keep reference schema and SQL in index. The tracked schema is the physical source of truth;
rusqlite executes parameterized statements on the owning session. Focused SQL is appropriate
for FTS5/sqlite-vec/dynamic eligibility/validation. Query catalog metadata must
match executable bindings, never artifact-supplied SQL. CLI CEL and the structured
UI builder lower independently to the shared predicate. Do not invent another
expression parser, filter authority, per-field CLI flag set or metric EAV store.

Creature/hazard reference projections retain authored defense qualifiers beside
numeric facts. AC, HP and save notes are explanations, not modifiers inferred
from prose. Size and meaningful hardness are reference facts; focus capacity appears
only on an authored focus casting entry, and ritual DC remains separate from
ordinary spell DC. Activity frequency presents authored maximum and interval separately
from source remaining uses or saved encounter expenditure. CLI and web readers
render these shared facts; known-empty optional notes remain quiet and malformed
source values retain their unavailable state.

Ritual reference facts preserve authored primary/secondary checks and caster counts;
counteraction is a source capability, not a calculated target DC. Feat prerequisites
retain authored alternatives without eligibility evaluation. Each prerequisite is
prepared as an attributed rich field, using its source array index as the checked
field address; CLI and browser render those fields before descriptive prose. Effect reference duration
is separate from saved encounter timing: encounter/unlimited units ignore numeric
sentinels and inapplicable turn-expiry metadata, matching the pinned system policy.
These facts are projected once in app-service for CLI and shared browser readers.

App-service owns product filter grouping and editor hints shared by CLI discovery
and the web builder, including compact collection controls and short choice lists.
Product field discovery omits source UUID references and groups unit-specific
effect duration fields into one editor control. The executable artifact catalog
retains technical bindings. Applicability uses one family inventory under the
committed filter, and Kind choices omit categories absent from the artifact.
These hints reference existing catalog fields; they do not define new predicates
or aliases. Index owns contextual option eligibility and applies matching-only
enumeration before pagination, retaining selected values. Unsupported count
contexts remain browsable through count-free vocabulary.

Keep product retrieval in search. Surfaces do not open SQLite, implement ranking,
scan source clones or load models. Summary reads are body free; explicit detail
decodes one root and batches selected cache reads. Filters and key scopes apply
before KNN/FTS ranking. Search witnesses retain actual owner/field/passage addresses.
Verified remaster preference applies only when both records match the complete
request; variants are separate derived suggestions.

Keep application workflows in app-service. It uses runtime/search/local-state,
not index imports or handcrafted SQL readers. Retain the bounded retrieval
executor rather than serializing all read-only web requests behind one lane.
Worker startup compares the actual opened artifact instances and complete build
contexts. Short-lived commands retain one lazy reader across composed operations.
Replaceable page and discovery reads carry an explicit app-owned execution context.
Their awaiting HTTP helper marks abandonment on drop; cooperative checks skip
queued work and later projection. Blocking work retains its permit until it
finishes. Durable mutations and their reference reads remain uncancellable.
Stored-vector Similar is a mode of the stateless search-page request, carrying its
root seed and canonical filters. Related availability is a bounded app-service
workflow over exact selected-owner graph reads and root-only verified suggestions,
with complete artifact-generation cache identity and truthful empty/unavailable
outcomes. It does not add a retrieval, ranking or stored-record model.

App-service owns useful catalog control presentation, search pages, semantic
family presentation and hydration. Reference reads project authored facts directly;
encounters compose the existing bounded runtime overlays into one presentation.
There is no generic section/value/profile layout interpreter or second encounter
stat-block response. App-model stays a transient DTO/export boundary; run
`cargo test -p atlas-app-model` and regenerate checked bindings intentionally
with `cargo test -p atlas-app-model export_typescript_bindings -- --ignored`.

Search-page requests include criteria, page and an optional expected artifact
generation. Mismatches require an explicit browser refresh from page1, preserving
criteria and mutable drafts. See
[ADR0049](./decisions/0049-search-lifetimes-and-pages.md).

Keep saved-domain ownership separate from reference generation within the shared
workspace. Unknown saved keys retain unavailable identity rows and
their snapshots. NPC variants/conditions are read-time participant overlays over
authored baselines. New HP uses known integral effective maxima; saved HP,
explicit overrides and edit intent survive rebuilds. Missing values stay unknown.
Encounter Damage/Heal and exact HP correction inputs are explicit revision-checked
actions interpreted by app-service; the browser sends raw integer formulas and
does not calculate persisted HP. Enter sends a signed Heal action; negative Damage
or Heal amounts reverse the operation in Rust and use its complete HP policy.
The bounded addition/subtraction shortcut permits one initial sign and is not
a general expression or dice engine. The HP editor saves its supplied corrections
in one guarded action, preserving omitted fields and explicit unknowns. Condition
manual-effect notes carry their exact condition ID; unrelated diagnostics remain
separate from condition cards.
Do not implement a Foundry world/rule/macro execution engine through presentation.

Local-state owns complete atomic user actions and coherent aggregate reads.
Participant replacements require the revision the caller read; stale writes
receive a conflict without changing state. App-service owns pure turn selection
over the local-state transaction snapshot. Turn review returns a durable encounter
revision; Continue checks it and decrements explicitly anchored round durations
with the turn in one transaction. Zero duration retains the condition for explicit
removal, and nonround durations remain table reminders. Existing-encounter mutations return
a canonical identity receipt without postcommit reference hydration; explicit
reads independently refresh presentation. See
[ADR0048](./decisions/0048-atomic-mutable-workflows.md).

Encounter detail/edit routes have one mutation owner spanning write and refresh.
Committed-write refresh failures block further edits with read-only Retry;
typed conflicts refresh current state without resubmitting the retained draft.
Save and destructive confirmation await their durable action. Background reads
preserve dirty same-entity metadata, while route changes isolate completions.

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

Responsive browser acceptance uses the developer-only `atlas-web`
`browser_fixture` example. It builds a small checked artifact with existing
retrieval test support, then serves the production router and app-service in
stored-vector mode with isolated local state. Structural fixture vectors do not
establish model inference or ranking quality; query assets remain unavailable.
Fixture-only dependencies and composition stay in the example/dev-dependency
boundary. Browser tests use normal APIs rather than test-only product endpoints
or fabricated readiness. Frontend guidance defines the device matrix and the
separate automated, Astra visual and actual-device evidence requirements.

Architectural replacements land with every call site and matching docs updated.
The persistent workspace upgrades through one composed migration ledger. Reference
updates preserve saved domains and stable record identities; expensive inference
is reused by exact model/input identity. No legacy adapter, bridge or mixed old/new
authority is retained. See ADR0051 for workspace ownership and ADR0046 for the
source-backed reference decision.

Journal presentation retains authored page order and exact checked navigation;
empty pages differ from missing/null/invalid collections. Page identity remains
visible even when authored title visibility suppresses an additional body heading.
Unsupported media is identified without fetching or introducing a viewer.
Roll-table reference rolls are stateless and may repeat, including when a source
specifies no replacement; source drawn flags do not become saved deck history.
All matching valid ranges remain visible with an incomplete-results warning when
other rows have invalid ranges; a gap is not repaired or re-rolled automatically.

Consumable presentation treats its explicit embedded spell as a semantic Spell
association, not generic contents. Inline occurrences retain their authored target
and canonical relationship. App-service may attach a separate contextual reading
destination when the embedded spell's resolved provenance uniquely agrees with
the citation in the exact selected consumable. Conflicting, unresolved or bounded
provenance leaves the association explicit. The browser uses supplied navigation;
it never matches names, interprets casting prose or rewrites graph identity.

Product navigation uses `embedded_path`; presentation child links use `embedded`.
The bounded reader is `/api/records/embedded-reading`, and CLI selection uses
`--embedded-path`. Product content addresses are `ContentLocationView`; private
source locators retain their persisted ownership vocabulary. There are no legacy
public aliases. Usage labels preserve exact source tokens for filtering and do
not infer a normalized hand-count requirement.
