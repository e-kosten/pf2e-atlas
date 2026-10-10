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

The supported contract is `pf2e-atlas-source-artifact/v2`, with format and SQLite
`user_version` 2. `artifact_context` contains one typed, hashed build context.
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
| Semantic search | `semantic_models`, `semantic_units`, `semantic_vectors`: exact model identity, attributed section/chunk addresses, input hashes and vectors. |

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
and eligibility. Contextual facet counts self-exclude the target predicate while
preserving same-child siblings; unsupported OR/NOT facet contexts return errors.

Filters, explicit key scopes and product-family eligibility constrain lexical and
vector units before ranking. Macros remain available for developer inspection and
are excluded from every product path, including exact lookup, facets, graph
neighbors and similar seeds. RollTables are normal product records.

## Content and references

The DTO retains authored markup. `atlas-record::source_content` owns interpretation
and sanitization; ingest supplies localization, audience and reference resolution
context. Prepared caches bind to the exact authored field hash and complete build
context hash. Controls refer to markers local to that field. The browser renders
sanitized HTML with those narrow facts; html2text formats terminal output at read
time. Neither renderer executes Foundry scripts or reconstructs a general document
AST. Terminal formatting is not FTS text.

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

Only the pinned `BAAI/bge-small-en-v1.5` model is supported. Its immutable revision,
asset checksums, tokenizer, CLS pooling, L2 normalization, cosine distance and input
policy are part of the context. FastEmbed owns inference; text-splitter owns generic
segmentation. Atlas owns selection, attribution, context, exact reuse and final
tokenizer-budget checks. Body target is 256 tokens, context 64, overlap at most 32,
identity at most 480, and final assembled input at most 512 including special tokens.
No silent truncation or token-estimator fallback is allowed.

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
An artifact has either complete semantic coverage or no semantic model. Lexical
reads remain available without embeddings; full web readiness requires them.

Search collapses semantic units by maximum similarity per root. Hybrid uses one
rank per root per lane with RRF60; sibling appearance does not penalize a passage.
Semantic candidate windows are bounded and reported as such. Similar uses the
stored root identity seed and semantic candidates, excluding the seed. No invented
trait/graph reranking weights apply.

Legacy suppression occurs only when its verified remaster also matches the complete
current request, before paging. A legacy-only match remains visible, and exact key
lookup remains available. Suggested variants are bounded evidence-backed same-pack,
family and compatible-category candidates; they never change identity or suppression.

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

Opening a reader checks format, typed context, executable schema/catalog and active
model metadata. Explicit full validation checks SQLite integrity/foreign keys,
checked snapshots, exact typed projections, owner addresses, content input/context
hashes, cache/section coverage, sanitized markers, attributed relationships,
identity evidence, FTS postings and vector coverage/dimensions/finiteness/normalization.
Query summaries and selected cache reads do not decode bodies. Explicit detail
decodes one root and requests bounded batches of selected content; no per-marker
or per-field body reads are permitted.
