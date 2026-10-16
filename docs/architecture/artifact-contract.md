# Artifact contract

The generated reference artifact is a read-only SQLite database owned by
`atlas-index`. It stores checked Foundry source snapshots and rebuildable query,
content and search projections. Mutable saved lists and encounters belong to the
separate `atlas-local-state` database. Rebuilding the artifact never rewrites
those local rows. [ADR 0046](./decisions/0046-source-backed-reference-artifact.md)
records the durable storage and retrieval decisions.

## Identity and versions

Record identity is the Foundry pack and source `_id`, serialized as `pack:id`.
Readable route slugs may accompany that identity. They do not introduce another
identity registry. Embedded documents stay in their parent snapshot, including
customized copies of canonical Items. Checked owner chains use unique source IDs
where available and snapshot-local positions otherwise. Snapshot-local owners
must not reconcile saved state across rebuilds.
Product navigation using snapshot-local positions carries the source fingerprint;
detail rejects missing or stale generation context for those explicit addresses.

The supported contract is `pf2e-atlas-source-artifact/v3`, with format and SQLite
`user_version` 3. `artifact_context` contains one typed, hashed build context.
The checked snapshot codec has its own version owned by `atlas-foundry-model`.
Unsupported artifacts require rebuilding. There is no migration, compatibility
reader, normalized-record adapter or original-JSON fallback for old reference
artifacts. Local-state migrations remain independent.

The physical relational schema is
[`atlas-index/migrations/00000000000001_create_artifact/up.sql`](../../crates/atlas-index/migrations/00000000000001_create_artifact/up.sql).
Diesel declarations and writer/readers must agree with this executable schema.
FTS5, sqlite-vec, dynamic eligibility relations and validation pragmas use focused
SQL owned by index. Artifact-supplied data never supplies executable SQL.

## Stored families

| Family | Tables and purpose |
| --- | --- |
| Identity and provenance | `packs`, `records`: canonical keys, compact source paths/hashes, lookup keys and body-free common summaries. |
| Checked source | `record_bodies`: one gzip-compressed, versioned checked DTO snapshot per root. Invalid members retained by admission remain in that snapshot. No duplicate complete original JSON payload is stored. |
| Query accelerators | `record_traits`, `actor_projection`, `actor_items`, Actor IWR/language/speed/sense rows, and named Spell/physical/weapon/armor/shield/ability/heritage/effect/condition/deity projection tables. Root and immediate Actor Item scopes use the same typed extractors. |
| Query catalog | `query_field_catalog`: serialized public definitions checked against executable Rust bindings; no artifact-defined extractor expressions. |
| Prepared content | `prepared_content`: selected field outcomes, one compressed sanitized HTML cache, narrow interaction facts and authored/preparation hashes. Plain authored fields are read from the checked source when needed. |
| Developer reports | `developer_diagnostics`: attributed admission/preparation/resolution/projection evidence. Product record responses do not expose these reports. |
| Relationships | `relationship_occurrences`: attributed structured and visible content occurrences, preserving repeated links and resolved/unverified/unresolved/blocked status. |
| Verified identity evidence | `verified_aliases`, `remaster_pairs`: explicit evidence-backed lookup names and edition relationships. |
| Precision lexical search | `lexical_units`, `lexical_fts`: root names, verified aliases, typed vocabulary, owned names, headings and structural definition labels. |
| Semantic search | `search_profiles`, `semantic_units`, `semantic_vectors`: complete profile identity, profile-scoped attributed section/chunk addresses, raw input hashes and vectors. |

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

`atlas-domain::QueryPredicate` is the shared Boolean/field/set/Exists vocabulary.
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
Discovery shares query eligibility. Contextual facet counts self-exclude the target predicate while
preserving same-child siblings. Valid conditions below OR/NOT or using negative
comparisons can report typed optional discovery unavailability; the query itself
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

Query capability version 3 includes intentional shared user-facing labels and
the root-only numeric `record.level_or_rank`
binding over existing columns. Spells use spell rank and its availability state;
other applicable level families use record level and its state. There is no
fallback from unavailable spell rank to level. Existing explicit family fields
retain their meanings. Artifacts rebuild for the new catalog identity even though
the physical tables are unchanged; readers require the exact executable catalog.

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

Preparation emits fixed action glyphs as allowlisted semantic HTML attributes
with readable text, retaining nested enrichments rather than flattening them.
Font characters are display decoration, not canonical search or terminal text.
Prepared unresolved references preserve an explicit authored label or a qualified
authored root name; opaque UUIDs display as unavailable. Their exact authored
targets and unresolved status remain in relationship evidence and diagnostics.
The content interpretation identity is `foundry-content/v6`; older prepared
artifacts require rebuilding even though the physical schema is unchanged.

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
Vector reuse requires complete model identity and verification of the old unit's
input association. Ingest reconstructs selected input from the old checked source
and prepared caches using the pinned tokenizer, and compares its hash, token count,
address and chunk ordinal before accepting a vector. Offline index validation
checks source/cache addresses, coverage, hash syntax and vector integrity; it
does not require model assets or claim tokenizer-backed input verification.
Reuse never merges distinct source attributions. Vectors contain no duplicate filter metadata.
Each stored profile independently has complete semantic identity/section/vector
coverage. An empty manifest is a lexical artifact. One shared vec0 table uses
globally unique unit IDs; KNN eligibility includes profile and eligible roots
inside the scan. Similar seeds and reuse remain selected-profile scoped.
Lexical/reference reads and stored-vector Similar do not require query weights.

Search collapses semantic units by maximum similarity per root. Hybrid uses one
rank per root per lane with RRF60; sibling appearance does not penalize a passage.
Semantic candidate windows are bounded and reported as such. Similar uses the
stored root identity seed and semantic candidates, excluding the seed. No invented
trait/graph reranking weights apply. App search pages expose the same bounded
Similar request and body-free seed summary; they do not extend the stored schema.

The full serialized `SourceArtifactBuildContext` SHA-256 identifies the immutable
artifact generation for search pages, detail envelopes and Related availability. It covers
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

Ingest fingerprints the deterministic actual relevant input set, including pack
definitions, manifests and excluded inputs. Local edits therefore change identity
even if a Git revision is unchanged. Locale, catalog and preparation/model policy
identities are separate context inputs. Ordinary reads do not rescan source files.
The typed context retains a compact map of resolved trait labels actually used
by selected vocabulary. Exact lexical terms can therefore be reconstructed
offline; there is no duplicate full localization catalog. Root alias terms must
match the verified alias relation, with no arbitrary additional FTS vocabulary.

The writer constructs a temporary database, runs the coherence checks below, then
publishes atomically. Failure preserves the prior artifact. An open reader keeps
its SQLite connections on one verified generation; it cannot combine old snapshots
with new caches by reopening the published path. Platforms that prevent replacing
an in-use artifact return a clear failure preserving the prior bytes rather than
copying or deleting around the lock.

Index retains a non-serialized physical instance identity from the successful
reader-open guard. Search exposes it narrowly for application composition. Web
startup requires every worker to have opened the same instance and complete
context; comparing context hashes alone does not establish physical identity.
A composed CLI command retains its first successfully opened reader instead of
reopening the published path between parent and embedded reads. Physical identity
stays process-local; public artifact generation remains the full context hash.

Opening a reader checks format, typed context, executable schema/catalog and active
model metadata. Explicit full validation checks SQLite integrity/foreign keys,
checked snapshots, exact typed projections, owner addresses, content input/context
hashes, cache/section coverage, sanitized markers, attributed relationships,
identity evidence, FTS postings and vector coverage/dimensions/finiteness/normalization.
Query summaries and selected cache reads do not decode bodies. Explicit detail
decodes one root and requests bounded batches of selected content; no per-marker
or per-field body reads are permitted.

Exact-locator attributed content reading is an index-owned transient operation;
it changes no stored schema or preparation identity. Requests select at most 256
fields. Delivered rich fields retain complete validated content-origin occurrence
attribution within a 1024 occurrence response budget. A whole field exceeding the
remaining budget has an explicit bounded outcome; readers do not display its HTML
with silently missing links. Non-content relationship discovery stays a separate
bounded operation. App-service owned-reading batches check one parent, select at
most 32 immediate Ability-class activities and reuse this prepared batch without
recursive detail projection or a second fact authority.
