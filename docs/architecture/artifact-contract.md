# Artifact contract

Reference data and saved state share one persistent SQLite workspace.
`atlas-index` owns checked source snapshots and derived reference tables;
`atlas-local-state` owns lists, encounters and their durable actions.
`atlas-storage` owns admission, transactions and migration execution, while
runtime composes one schema history. Normal reference publication preserves the
workspace file and saved rows. [ADR0051](./decisions/0051-persistent-workspace.md)
defines these ownership and lifetime boundaries.

Developer baseline snapshots use `atlas-storage` admission and SQLite backup,
including committed WAL data. Seed installation accepts only an absent workspace
without orphan SQLite sidecars, verifies a staged clone/copy, and publishes without
replacing an existing path. It preserves workspace identity and all saved state.
`atlas-dev` owns manifests and provenance receipts, not SQL or migration policy.

## Identity and versions

Record identity is the Foundry pack and source `_id`, serialized as `pack:id`.
Readable route slugs may accompany that identity. They do not introduce another
public identity scheme. Internal `record_identity` rows survive source removal;
saved foreign keys reference that stable registry with restrictive deletion. Embedded documents stay in their parent snapshot, including
customized copies of canonical Items. Checked owner chains use unique source IDs
where available and snapshot-local positions otherwise. Snapshot-local owners
must not reconcile saved state across rebuilds.
Product navigation using snapshot-local positions carries the source fingerprint;
detail rejects missing or stale generation context for those explicit addresses.

The reference format, checked snapshot codec, preparation policies and query
catalog have separate version identities. `artifact_context` contains the typed,
hashed active build context. SQLite `user_version` instead tracks the shared
physical migration ledger composed by runtime.
The source artifact format is not the workspace schema version.
Workspace migration 11 expands encounter boundary claims to admit receipt-only
persistent-damage/recovery preparation. It copies every existing claim into the
replacement constrained table. Reference rows, saved conditions, HP and prior
receipts remain unchanged; preparation outcomes use the existing versioned receipt
envelope.
Workspace migration 12 adds nullable versioned participant turn-resource state.
Existing allocations remain unknown; it does not synthesize past actions or reaction
expenditure. Saved HP, conditions, parties, reference rows and prior receipts remain
unchanged. Old outgoing snapshots omit this field and decode as an unknown allocation.
App-service owns allocation/spending policy; local-state persists it with the same
atomic revision, receipt and boundary protections as other encounter events.
Workspace migration 13 allows a condition application marker to omit its source
row ID when the application is consumed before insertion. It preserves existing
markers, including those whose source was removed, and all saved state and receipts.
The application identity remains claimed without creating an inactive condition.

Workspace migration 16 adds participant-owned ability-state rows for authored
switches, finite choices and limited-use expenditure. Values use the existing
versioned saved-state codec; exact owner/rule/source identity prevents applying
one ability's state to another. Derived conditions remain reads, rather than
materialized saved condition rows. Existing reference data, HP, saved conditions,
action stock and receipts are preserved.

Pre-release separate-file formats have no compatibility reader or conversion
shim. Future workspace upgrades use ordered migrations with recorded checksums,
a consistent backup before structural changes, and transaction-scoped validation.
Saved-domain and reference migrations share the same ledger and transaction.

Reference DDL lives in
[the index baseline SQL](../../crates/atlas-index/migrations/00000000000001_create_artifact/up.sql);
saved-domain DDL lives in
[local-state schema.sql](../../crates/atlas-local-state/src/schema.sql).
Both execute through rusqlite on admitted sessions. FTS5, sqlite-vec, dynamic
eligibility relations and validation pragmas remain index-owned SQL. Stored data
never supplies executable SQL.

## Stored families

The shared header contains `workspace` and `schema_migrations`. Durable intent
lives in `requested_reference_config`; resumable inference uses
`reference_update_jobs` and `pending_embedding_values`, joined to exact
`embedding_models`/`embedding_inputs` metadata. Pending vector checksums are
independent of published checksums, so forced-rebuild progress cannot invalidate
current readers. Successful publication removes pending numerical values. Saved-domain tables include a seeded local principal
and non-null ownership, with owner-scoped slug uniqueness.

| Family | Tables and purpose |
| --- | --- |
| Identity and provenance | `record_identity`, `packs`, `records`: canonical keys, compact source paths/hashes, lookup keys and body-free common summaries. |
| Checked source | `record_bodies`: one gzip-compressed, versioned checked DTO snapshot per root. Invalid members retained by admission remain in that snapshot. No duplicate complete original JSON payload is stored. |
| Query accelerators | `record_traits`, `actor_projection`, `actor_items`, Actor IWR/language/speed/sense rows, and named Spell/physical/weapon/armor/shield/ability/heritage/effect/condition/deity projection tables. Explicit immediate-child `embedded.*` collections use the same typed extractors as the corresponding standalone families. |
| Query catalog | `query_field_catalog`: serialized public definitions checked against executable Rust bindings; no artifact-defined extractor expressions. |
| Prepared content | `prepared_content`: selected field outcomes, one compressed sanitized HTML cache, narrow interaction facts and authored/preparation hashes. Plain authored fields are read from the checked source when needed. |
| Developer reports | `developer_diagnostics`: attributed admission/preparation/resolution/projection evidence. Product record responses do not expose these reports. |
| Relationships | `relationship_occurrences`: attributed structured and visible content occurrences, preserving repeated links and resolved/unverified/unresolved/blocked status. |
| Verified identity evidence | `verified_aliases`, `remaster_pairs`: explicit evidence-backed lookup names and edition relationships. |
| Precision lexical search | `lexical_units`, `lexical_fts`: root names, verified aliases, typed vocabulary, owned names, headings and structural definition labels. |
| Semantic search | `search_profiles`, `semantic_units`, profile-specific `semantic_vectors_<identity hash>` tables: complete profile identity, profile-scoped attributed section/chunk addresses, raw input hashes and vectors. |

There is no generic metric EAV store, persisted RichDocument, synthetic canonical
affliction table, copied embedded Item authority or complete plain-text/model-input
corpus. Developer original-source inspection reads the configured Foundry checkout
using compact provenance. Normal product reads do not require that checkout.

Visible prepared reference markers and stored content occurrences must correspond
one to one within each field. A compact marker digest binds the attributed
reference fact to its locator and checked authored-field hash. Validation rejects
missing/duplicate markers, missing edges and changed destinations, alongside the
interaction-marker ordinal and kind checks. Interaction parameters are not
independently rebound to marker text: same-kind parameter mutations are outside
this offline check. These checks establish the specified derived-cache coherence;
they do not
authenticate coordinated changes to an artifact or independently resolve a
localized target without the build-time locale catalog.

Resolved structured destinations are checked with the record-owned resolver over
a compact identity/name index reconstructed from checked snapshots. Destination
existence alone is insufficient. Unresolved occurrences are not promoted during
validation: ingest may have excluded ambiguous names using unaddressable source
evidence that is intentionally absent from the product artifact.

## Query facts and eligibility

Query fields distinguish `value`, `missing`, `null`, `invalid` and
`not_applicable`. Known zero, false and empty sets are values. Numeric storage
preserves integers and finite real numbers without silently narrowing integers
to f64 or coercing quoted numeric strings. Unsupported representations produce
explicit invalid query facts while the checked source remains retained.

`atlas-domain::QueryPredicate` is the shared Boolean/field/set/Exists/reference
vocabulary. Relationship leaves apply to candidate roots and are rejected inside
authored Exists scopes. Neutral owner-address primitives live in atlas-domain;
source selection policy remains atlas-record-owned.
Index validates fields, applicability, operators, literals and bounded complexity
against its catalog, then compiles parameterized SQL. Only true roots match;
unknown remains unknown under NOT. Exists uses one child witness for its complete
predicate and allows one collection level. Missing IWR arrays have an explicit
upstream-backed empty query default only when their typed parent is known.

CLI CEL uses the maintained parser and a documented supported subset. The UI
builder submits structured predicates independently. There is no translation
between CEL and editor state. Field/value discovery shares executable bindings
and user-facing catalog labels. Labels name the useful property; source-family
applicability, stable query IDs and collection scopes remain separate metadata.
CLI numeric discovery uses range/state statistics rather than enumerating values;
collection discovery uses states and directs callers to scoped member descriptors.
Discovery shares query eligibility. Root facets self-exclude all selections of
the edited field through conjunctions, including individual negative selections.
Scoped facets replace the target clause while preserving same-child siblings.
Targets inside OR or compound NOT, and unsupported scoped negatives, can report
typed optional discovery unavailability; the query itself
remains executable. The index validates the complete predicate, target field,
scope, clause identity, value type and request limits before that classification.
Malformed requests and service failures remain errors. Search obtains availability
from the same index analysis before query embeddings or ranking.

Explicit count-free vocabulary discovery has no search predicate or retrieval
context. It enumerates known applicable values in the product artifact or explicit
saved-list key scope before edition preference, retaining server text search,
bounded paging and separately validated selections. An empty key scope stays empty.
Vocabulary totals describe available matching values, exclude absent selections,
and never masquerade as displayed-root counts. Closed choices remain in the
executable catalog even when absent from the corpus.

Public filters use domain namespaces, including root `kind`, `traits`,
`level_or_rank` and explicit `embedded.spells`/`embedded.equipment` collections.
Embedded predicates match immediate children only, irrespective of root-kind
filters. Root-or-child queries require explicit OR. Retired Actor/Item paths fail
rather than receiving an implicit translation. Supported query-catalog/projection
upgrades regenerate rows from stored DTOs in one maintenance transaction; they
neither require source admission nor invalidate unchanged embedding inputs.

Filters, explicit key scopes and product-family eligibility constrain lexical and
vector units before ranking. Macros remain available for developer inspection and
are excluded from every product path, including exact lookup, facets, graph
neighbors and similar seeds. RollTables are normal product records.

## Content and references

The DTO retains authored markup. `atlas-record::source_content` owns interpretation
and sanitization; ingest supplies localization, audience and reference resolution
context. Prepared caches bind to the exact authored field hash and preparation
context hash, which excludes the embedding profile manifest. Controls refer to
markers local to that field. The browser renders sanitized HTML with those narrow
facts; html2text formats terminal output at read time. Neither renderer executes
Foundry scripts or reconstructs a general document AST. Terminal formatting is not
FTS text.

Selected reading can compose checked inline Item fields into transient HTML with
separate occurrence binding namespaces. This changes neither persisted prepared
HTML nor interpretation identity: stored embed targets/options remain source
occurrences, and indexed lexical/semantic inputs do not acquire transitive text.

Preparation emits fixed action glyphs as allowlisted semantic HTML attributes
with readable text, retaining nested enrichments rather than flattening them.
Font characters are display decoration, not canonical search or terminal text.
The pinned font aliases lowercase `f`/`r` to uppercase `F`/`R`; both forms produce
the same free-action/reaction attributes and readable labels. Unknown glyphs
remain visible with their source diagnostic.
Prepared unresolved references preserve an explicit authored label or a qualified
authored root name; opaque UUIDs display as unavailable. Their exact authored
targets and unresolved status remain in relationship evidence and diagnostics.
The content interpretation identity is `foundry-content/v22`; older prepared
content requires re-preparation even when the physical schema is unchanged.
Exact compatible embedding checkpoints remain reusable.

Static PF2e damage-instance annotations are interpreted in atlas-record, using
atlas-dice's pure preparation to validate bounded arithmetic without rolling.
Preparation marks a formula boundary for inline dice placement.
Readable text retains each known type/category and authored labels; an adjacent
authored damage noun is preserved without duplication. Raw formulas/options remain
in the existing interaction sidecar. Unsupported expressions retain their source.
Dice preview trees are transient app projections, never persisted content ASTs.
Literal `/r`, `/gmr` and `/br` arithmetic uses the same bounded engine. Captions
and authored command modes remain source data; modes do not change audience in
Atlas's local reference roller. Flavored and unsupported inline-roll formulas stay readable without an executable offer.

The pinned SystemActions registry supplies generated literal `/act` identities,
localization keys, costs and variant names. English catalog lookup resolves those
keys during preparation; action costs follow the displayed label. Navigation resolves the registry's exact English name
through the action pack's unique admitted-name index, allowing ASCII case-only
differences for these derived action references; missing or ambiguous
destinations have no usable link. The sidecar retains the original command and
options, and reference facts identify the registry slug and derived target.
No Foundry action code executes. Action reference boundaries contain the label;
costs and option details follow outside the link.

Checks and area templates retain their source options and field-local interaction
identities but render as plain text. Preparation supplies complete check names,
basic/secret qualifiers, opposing DCs and authored area widths; exact custom labels
and audience-filtered DCs remain authoritative. Literal DC adjustments can be
composed without actor context. Authored actor/item formulas remain in the interaction sidecar.
Renderers add no check or area execution control or parameter dialog. Literal
reference dice and verified document links keep their separate capabilities.

Contextual formulas use a bounded transient arithmetic interpreter in atlas-record:
integer/decimal constants, arithmetic, dice, floor/ceil/min/max and resolve, with
an explicit variable allowlist. Canonical preparation supplies source-kind labels
only and emits symbolic prose; neither FTS nor embeddings acquire runtime values.
Symbolic halving and rounding retain compound operand grouping, so their prose
preserves the authored arithmetic order.
The sidecar records generated-label provenance, authored healing units and adjacent
check nouns/secret qualifiers, not an expression AST. An immediately adjacent clause is absorbed
only when it exactly repeats the generated symbolic DC; custom labels stay intact.
Unknown syntax/variables retain authored expressions. Area-template runtime
geometry, general predicates, actor flags and Foundry scripts remain unsupported.

App-service binds selected spell rank (including the spell-only item.level alias),
checked item level and the owning actor's effective source level. Standalone feat
levels never stand in for actor level; generic condition/effect badge defaults
never stand in for a live value. Encounter references can supply their current
participant level or an addressed condition's effective value. Missing context
remains readable prose with no input prompt or parameter dialog. Each field is
resolved before composing inline inclusions, which retain their own source context.
Canonical search passage hashes are validated against the stored field before
read-time substitution. Resolved damage and inline rolls acquire the existing bounded dice capability
only if the entire expression is supported. Roll requests carry source identity,
selected rank and, when needed, encounter/participant revision and condition
identity. The server rereads those facts and validates the complete current offer
before randomness. Results remain stateless; no roll history is written.

Localization preparation uses `selected-source-catalog/english-fallback/v2`.
Its English catalog digest covers all manifest-declared English catalog paths
and bytes in declaration order. Version 1 preparation requires source-backed
re-preparation so unresolved passages are not retained through cache reuse;
unchanged native embedding inputs remain reusable.

App-service derives transient semantic family presentation from these checked
snapshots and selected caches. Those response DTOs are not stored family bodies
or another authored authority. Presentation changes alone require neither an
artifact rebuild nor a schema migration; see [ADR0047](./decisions/0047-semantic-record-presentation.md).

Initial indexing includes GM/owner prose and check DCs, excludes explicit None,
and defaults to English with a build-time locale override. Search uses the indexed
locale until rebuild; initial display also uses it. This does not implement a
Foundry permission engine or runtime localization engine. Local/unavailable image
paths produce safe unavailable-asset placeholders. Safe HTTP(S) assets are
optional; no asset fetch subsystem is part of artifact readiness.

Relationship occurrences retain root, owner chain, field and occurrence ordinal.
Resolved root/owned identities differ from unverified external URLs and unresolved
or blocked targets. Hidden content contributes no product edge. Macro targets do
not become product graph neighbors or interactive tooling links. Authored labels
may still display as plain text. Only verified aliases affect strict name lookup.

Relationship predicates read these same occurrences directly and compose with
ordinary predicates through the shared AND/OR/NOT compiler. Exact destination owner
paths remain distinct from source subtree scope; descendants never include siblings.
No separate relationship eligibility array, stored model or capped neighbor projection
is used. Shared retrieval admission validates all targets/owners/fingerprints even
under negation; missing targets are input errors. This model change does not change
the stored occurrence schema or require artifact re-preparation.

Full result pages count distinct eligible roots before paging and hydrate bounded
positive proof evidence separately. Proof attribution reuses compiled SQL truth;
negative absence and unsuccessful Boolean branches do not fabricate evidence.
Clause IDs associate occurrence pages with exact predicate leaves. Preview truncation
is not an eligibility or root-count limit. Contextual facets, values and statistics
retain the entire expression and optional saved-list keys, applying existing
self-exclusion and unsupported-context availability rules. Count-free vocabulary
remains independent. See [ADR0057](./decisions/0057-relationship-scoped-search.md).

## Lexical and semantic policy

FTS uses `unicode61 remove_diacritics 2` with separate identity, alias, structured
and definition columns. Long explanatory prose and presentation styles are not
precision FTS input. Semantic selection independently covers selected explanatory
root and owned prose plus one compact root identity unit. Embedded/prose affliction
definitions remain attributed to their actual source; they do not become generated
canonical records.

The canonical manifest supports `bge-base` and `embeddinggemma` with exact pinned
assets, tokenizer/configuration, output/pooling, prefixes and baseline recipe.
Each emits 768-dimensional L2-normalized vectors. FastEmbed owns inference;
text-splitter owns generic segmentation. Atlas owns selection, attribution, context,
exact reuse and final native guards. Both profiles use BGE baseline segmentation:
body 256, context 64, overlap at most 32, identity at most 480, assembled baseline
input at most 512 including special tokens. Only owned headers omit pack labels.
The selected native final prefixed limit is 512 for BGE and 2048 for Gemma; Gemma
does not expand passages. No silent truncation or token-estimator fallback is allowed.

Passage metadata addresses a selected section and UTF-8 byte range, with source or
prepared HTML/canonical-text hashes and selection version. Identity units have no
fabricated prose range. Readers recover text from checked source/prepared content.
Vector reuse requires exact model/input identity, token count, dimensions and
verified vector bytes. Durable checkpoints survive interrupted builds and content
policy changes; changed prepared text obtains a different input identity. Input
reuse does not merge source attributions. Offline validation checks addresses,
coverage and vector integrity without requiring model assets.
Each stored profile has complete semantic coverage. An empty profile manifest is
lexical-only. Each exact profile identity owns a dimensioned vec0 table; KNN
eligibility remains constrained to eligible roots within the selected profile.
`semantic_locations` holds complete source addresses once across profiles;
`semantic_units` retains each attributable occurrence and references its location
and native-input metadata. Compact digests index full checked addresses; validation
recomputes them and key conflicts compare the complete metadata. Each document
vector is stored only in its model-owned vec0 table. Rebuild reuse verifies those
bytes against the native input's published checksum. `trait_seed_chunks` retains
selected input/coverage evidence and references native inputs; `trait_native_values`
holds only values not supplied by document occurrences. Trait centroids use binary
f32 bytes with separate checksums. No permanent checkpoint vector corpus remains.
Final publication reconciles and validates active units and clears pending values
and unused metadata in the same transaction, never before a complete replacement
is available. Pending values have priority for resuming unfinished builds while
published readers retain the old generation.
Lexical/reference reads and stored-vector Similar do not require query weights.

Lexical rows keep each root name and verified alias separate with root vocabulary;
embedded names use only their own vocabulary, and section rows contain their label.
No row inherits its parent's identity. The strongest weighted matching row determines
each root's lexical strength (root/alias 1, root section .5, embedded .25).
Exact primary names and verified aliases retain priority ahead of fused results.
Lexical-policy upgrades rebuild these rows from retained records and content in a
maintenance transaction without replacing semantic vectors or saved state.

Text search collapses semantic units by maximum similarity per root. Hybrid offers
one rank per root per lane with RRF60 as a dogfooding alternative. The default
`score` strategy min-max normalizes each candidate lane and combines lexical .25 with semantic .75;
a constant nonempty lane receives its full lane weight. Neither strategy adds
weight for sibling matches. The request-level choice is independent of embedding
profile and requires no persisted projection changes. See ADR0053.
Semantic candidate windows are bounded and reported as such. Selection among
exact equal-distance hits at a KNN cutoff is arbitrary and may change with physical
layout. Existing outer ordering sorts the selected hits, not excluded ties. Do not
enumerate boundary ties or expand windows to enforce reproducibility. Current
baseline Similar uses the stored root identity seed and semantic candidates,
excluding the seed; trait baseline Similar uses its stored definition centroid.
No invented trait/graph reranking weights apply.

Configurable Similar also supports passage matching and content averages over
selected existing source sections. Schema 17 adds similarity_parts metadata with
typed comparison roles and exact source/preparation-bound selectors. Its backfill
reads retained checked snapshots and prepared content, without new embeddings.
Publication refreshes this metadata with its owning record. Corresponding-role
eligibility constrains KNN before bounded candidates are selected.

Section inventory and comparison evidence project reader destinations from the
checked lexical and semantic-location metadata. The index combines full-section
addresses and contiguous chunk coverage without loading vectors or source bodies.
App-service preserves authored section headings and supplies field-title fallbacks
such as Description for NPC public notes. These display labels do not rewrite
stored selectors or comparison metadata. Reader destinations retain the exact
owner and field, canonical passage when available, and snapshot-local fingerprint
where required. A section without a canonical passage can open its containing
field; it does not receive an invented section address. The source reader checks
the same artifact generation as the section inventory.

Passage matching averages best allowed chunk matches within each source section
using overlap-adjusted coverage, then combines sorted section scores with
normalized geometric rank decay. Content average either compares global
centroids or keeps separate corresponding-role centroids. Missing counterparts
remain explicit zero contributions under the new positive-cosine score recipe.
Baseline scores remain unchanged. Aggregate scores are ranking values, not
probabilities. App search pages acknowledge effective options, recipes, profile,
generation and diagnostics and project bounded paired source evidence. Evidence
limits do not alter ranking. All methods use stored vectors without query-model
weights. See [ADR0059](./decisions/0059-configurable-similarity.md).

Attribute ordering uses the existing root name, level/rank and per-item-price
projections. Index shares one comparison contract between complete browse SQL
and ordering an accepted candidate-key set. Keys are bound as data, without a
browse-page cap or independently ordered chunks. Known values precede unavailable
states in both directions for every ordered rule. Equal or mutually unavailable
values proceed to the next rule; final ties use normalized name ascending then
canonical key ascending. Each rule reverses only its value comparison.
Spell rank retains the catalog's level/rank dispatch, and price comparison uses
native stored copper without another currency conversion. Typed batch reads of
visible-root prices support transient labels; these labels are not stored facts.
Random order uses SHA-256 over the ASCII domain
`pf2e-atlas/result-order/random/v1` followed by a zero byte, the u32 seed in
big-endian order, the canonical key's UTF-8 byte length as a big-endian u64, and
the key bytes. Digest bytes compare ascending, with canonical-key ties. This
query-time order needs no stored random column or artifact change. Browse reads
all eligible keys body-free before Random ordering and page hydration; ranked
search orders the unchanged complete accepted set. No cap, chunk-local order,
per-page randomization or unspecified runtime hash participates.
Ordering does not require new schema, source preparation or embedding work.

The full serialized `SourceArtifactBuildContext` SHA-256 identifies the active reference
generation for search pages, detail envelopes and Related availability. It covers
locale, shared preparation policies and the complete sorted profile manifest as
well as source identity. Shared HTML/content preparation hashes exclude the
profile manifest, including its model and passage recipe. Raw input hashes identify
unprefixed selected strings; profile execution/reuse keys bind final prefixed inputs,
native tokenizer assets and the full shared BGE segmentation-tokenizer SHA-256.
Each profile persists that segmentation digest separately from its native tokenizer
digest, including Gemma-only manifests.
Snapshot-local navigation continues to use the source fingerprint; these identities
serve different purposes and are not interchangeable.

Stateless app requests carry criteria, page and an optional expected generation.
Generation mismatch is checked before expensive retrieval and requires deliberate
recovery from page1. This neither changes the artifact schema nor adds a result
cache. A running web reader pool remains on its opened instance until restart;
new commands observe later publication on their first reference read.

Legacy suppression occurs only when its verified remaster also matches the complete
current request, before paging. A legacy-only match remains visible, and exact key
lookup remains available. Suggested variants are bounded evidence-backed same-pack,
family and compatible-category candidates. Verification distinguishes a complete
empty result from unavailable seed/candidate compatibility or an incomplete
bounded check; nonempty results retain ambiguity and truncation. These suggestions
never change identity or suppression.

## Publication, readers and validation

Trait vocabulary publication retains exact typed trait identity, source catalog
and localization-key evidence, selected localized definitions and prepared HTML.
Membership retains the actual source owner; explicit references remain separate
from membership and arbitrary label matches. Stored trait seeds bind their
complete selected input, versioned recipe and profile. Trait Similar uses the
existing normal-record candidate ranking and eligibility. None of these rows
create fabricated Foundry records or replace authored trait filter values. Checked
publication and deep validation require a seed for every published trait definition
in every configured profile, including definitions with label-only input. An empty
profile manifest permits metadata-only publication; unknown identifiers without a
published definition do not require seeds. See
[ADR0056](./decisions/0056-trait-reference-entities.md).

Ingest fingerprints the deterministic actual relevant input set, including pack
definitions, manifests and excluded inputs. Local edits therefore change identity
even if a Git revision is unchanged. Locale, catalog and preparation/model policy
identities are separate context inputs. Ordinary reads do not rescan source files.
The typed context retains a compact map of resolved trait labels actually used
by selected vocabulary. Exact lexical terms can therefore be reconstructed
offline; there is no duplicate full localization catalog. Root alias terms must
match the verified alias relation, with no arbitrary additional FTS vocabulary.

The writer reconciles changed reference rows inside a maintenance transaction in
the existing workspace. Unchanged rows retain their identity; removed sources lose
current projections and become unavailable identities without deleting saved
references. Reference publication and profile activation advance the workspace's
reference revision; user-state writes do not. Failed publication rolls back the
whole delta. Durable maintenance remains set until committed data is validated
and admission can safely resume.

Readers own bounded deferred transactions. One composed operation shares its
snapshot across reference and saved-domain readers; idle CLI/web owners retain no
SQLite session. Writes use immediate transactions and recheck preparation revisions
before applying source-derived changes. Local-state actions use savepoints inside
the outer transaction, so caught action errors cannot leak partial writes into a
later commit. Durable receipt recovery precedes stale-source checks. Receipt JSON
uses explicit storage versions and domain facts rather than application DTOs.

The workspace identity is durable; request generation checks additionally bind the
complete reference context. A later operation can observe a completed update, while
an already admitted read remains coherent. Application-managed sidecar locks
coordinate cooperating processes; external SQLite tools do not participate.

Opening a reader checks format, typed context, executable schema/catalog and active
model metadata. Explicit full validation checks SQLite integrity/foreign keys,
checked snapshots, exact typed projections, owner addresses, content input/context
hashes, cache/section coverage, sanitized markers, attributed relationships,
identity evidence, FTS postings and vector coverage/dimensions/finiteness/normalization.
Within a validation pass, each root's checked prepared content feeds search
selection directly. Its bytes and sidecars are validated once in the caller's
transaction and discarded after that root; no cross-transaction validation cache
or weaker publication check is introduced.
Complete vector validation scans unit IDs in storage order rather than following
the input-hash reuse index. Workspace sessions use an 8 MiB demand-filled SQLite
page-cache target so sequential vec0 chunks fit without repeated eviction; this
does not change the stored vector format or request-level retrieval policy.
After reconciliation, the writer flushes dirty SQLite pages before validation so
freshly written vector chunks do not compete with dirty pages for the cache.
This is a mid-transaction cache flush, not a commit or early activation; any
flush or validation failure still rolls back the complete publication.
Pending vector inserts borrow prepared vectors and encode one blob at a time;
reconciliation does not retain another full corpus of encoded vectors.
Query summaries and selected cache reads do not decode bodies. Explicit detail
decodes one root and requests bounded batches of selected content; no per-marker
or per-field body reads are permitted.

Exact-locator attributed content reading is an index-owned transient operation;
it changes no stored schema or preparation identity. Requests select at most 256
fields. Delivered rich fields retain complete validated content-origin occurrence
attribution within a 1024 occurrence response budget. A whole field exceeding the
remaining budget has an explicit bounded outcome; readers do not display its HTML
with silently missing links. Non-content relationship discovery stays a separate
bounded operation. App-service embedded-reading batches check one parent, select at
most 32 immediate Ability-class activities or table results and reuse this prepared batch without
recursive detail projection or a second fact authority.

Prepared-content views may carry an optional transient section-rendering HTML
sidecar. `atlas-record` derives record annotations from the same canonical section
partition as search (`structural-source-sections/v4`); a trait definition
(`trait-definition-centroid/v1`) and a plain field each retain their one logical
section. Reserved authored annotation attributes are stripped from the rendering
copy before canonical annotations are applied. The original prepared HTML, passage
hashes and persisted content remain unchanged. The sidecar preserves container, list and table topology, and
annotates block nodes by ordinal, wrapping substantive bare text only when needed.
App-service reuses each already-computed inline-inclusion insertion map for both
renderings; included foreign content retains its own namespaces without inheriting
source annotations. This adds one bounded, constant-factor rendering string and
linear parsing per selected field, without extra source lookups or stored metadata.
Consumers require the supported selection version and matching seed, generation,
owner and field identity before applying draft-only source emphasis.

Structured ability/feat self-effect UUIDs use the same attributed relationship
preparation and exact identity resolution as other authored links. The relationship
policy version invalidates old preparation when these occurrences are added; no
SQLite schema migration is required. App-service attaches verified navigation to
the shared activity projection with one bounded relationship read per selected
record. Missing targets remain readable references, never inferred name matches.
Strike-effect names remain authored labels without speculative cross-record links.

Feat prerequisite entries are selected as individually addressed rich fields under
`/system/prerequisites/value/<index>/value`, following pinned feat embed semantics.
Selection identity `structural-source-sections/v4` invalidates prepared/search
projections when these fields are added; setup re-prepares from the configured
compatible source before readers admit the new artifact. This requires no schema or snapshot migration. Array positions and
unavailable entries remain explicit; renderers never parse or fall back to raw markup.

Item provenance includes the declared open `flags.core.sourceId` namespace and
`_stats.compendiumSource`. Duplicate or non-string core source identities do not
supply a resolved target. Relationship policy `typed-authored-occurrences/v6`
also retains literal compendium targets from atomically interpreted, non-ignored
GrantItem rules and ancestry/class/background feature lists. These DeclaredGrant
occurrences retain exact owner and source-field attribution; they are references,
not evidence of active or applied grants. Injected targets are not resolved or
expanded. Predicates, alterations, level gates and grant lifecycle behavior are
not evaluated. Actual owned ItemGrant/GrantedBy occurrences remain distinct.
Older relationship preparation requires normal source-backed republication;
the schema and source snapshot codec are unchanged.
Contextual consumable reading is a transient app projection:
`target` preserves the authored citation while optional `contextual_target` points
to the checked embedded spell. It does not replace the stored graph edge.

Consumable query kind/type live on the shared physical projection for root and
immediate actor equipment. Workspace migration 4 adds their state/value columns;
authored-query/v3 and catalog v6 rebuild accelerators from retained typed snapshots.
The migration preserves saved state and does not change the source snapshot codec.

### Encounter status migration

Workspace migration 5 adds saved-domain `hazard_disabled`, defaulting existing
participants to false and restricting true to hazards. This local-state-owned
migration preserves HP, conditions, resources and reference data. It is composed
by runtime after the immutable earlier migrations.

Workspace migration 6 converts defeated hazards to disabled and prevents hazards
from storing the creature defeated marker. It preserves HP and creature status.
Migration 5 remains immutable because it has been applied to preview workspaces.

### Encounter condition system

Workspace migration 9 applies the condition system directly after the published
party (7) and trait vocabulary (8) migrations. It adds scoped condition inputs,
extra-action restrictions, immunity overrides, typed parameters and independent
application origins, plus
participant dying-tracking, mortality, level overrides and death thresholds.
Everyone and selected participants remain distinct scopes; unspecified,
unrestricted and restricted extra actions remain distinct states. Notes are
optional guidance, never executable rule input. No intermediate development
schemas or conversion of prototype fields are part of this migration.

Saved `max_hp` remains pre-condition capacity. Derived condition statistics never
charge persistent consequences on reads. App-service prepares explicit bounded
events; local-state applies their condition/HP changes, pending decisions and
versioned receipts atomically. Application identities stay consumed after removal,
so a retry cannot grant the same consequence again. Unconscious-created Prone
remains after waking. Persistent-damage comparisons retain exact episode/source
membership; boundary claims prevent replaying completed turn processing.

Outgoing turn receipts store request identities and typed pre-advance saved-state
snapshots. Advancement commits the boundary and next turn together. Only the latest
completed boundary is actionable; closing the notification changes no saved state.
There is no acknowledgment queue or persisted action-attempt/relationship workflow.
Spell reminders and optional strike context are read-only projections. Spell
resource writes and retained dice use their existing guarded mutation owners.

This saved-domain migration preserves reference artifacts, parties, lists and
existing encounters. Published migrations 1–8 remain unchanged.

### PF2e 8.6.1 reference preparation

The current source contract includes ammunition as an equipment family. NPC skills
are keyed authored entries; character movement is runtime-derived and has no
source-backed movement projection. Table result content uses `/description` and
structured references use `/documentUuid`, resolving only explicit compendium UUIDs.
The query catalog is version 6, query projections use `authored-query/v3`, content
selection uses `structural-source-sections/v4` and relationships use
`typed-authored-occurrences/v5`. These rebuild derived data without a physical
schema migration. The exact preceding 6.12.4 model can be replaced from compatible
source while preserving saved state; its snapshots are never decoded by the new
model (ADR0052).

### Native embedding storage

Workspace migration 10 replaces the permanent numerical checkpoint cache, wide
per-profile passage keys and JSON trait-vector arrays in one transactional typed
conversion. It retains exact f32 bytes, model contracts, per-profile token counts,
source addresses, unit IDs, saved-domain rows and pending inference from preparing
jobs. Published and pending values may legitimately differ for a forced rebuild.
The conversion validates retained numerical evidence without decoding source DTOs
that might require a separate source upgrade. Existing migrations remain immutable.

Vector maintenance reuses compatible slots and packs indexes only when at least
two blocks and 10% of allocation can be reclaimed. It uses supported vec0 public
DDL/inserts and file-backed temporary staging, not extension shadow-table writes.
Free SQLite pages are reclaimed separately through explicit setup/build maintenance;
this changes neither logical reference revision nor workspace identity.
