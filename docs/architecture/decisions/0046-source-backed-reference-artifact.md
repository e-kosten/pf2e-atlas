# ADR 0046: Source-backed reference artifact and query projections

Status: Accepted.

## Decision

The generated reference artifact stores one compressed checked Foundry snapshot
per root. Canonical record identity remains the Foundry pack plus source `_id`.
Readable route slugs are decorative and do not become identity or alias history.
Embedded documents remain in their parent snapshot, including local customizations.
Navigation through snapshot-local owner positions carries the source fingerprint
and rejects a stale generation; a saved ordinal is never remapped to a new child.
Runtime decoding uses the versioned atlas-foundry-model codec without admission,
defaults, repair or an original-JSON fallback. Complete original JSON bytes are
not duplicated in the database; developer inspection reads the source checkout.

Named relational query projections accelerate selected useful filters. They are
rebuildable facts, not separately authoritative Actor/Item models. Root and
immediate embedded Item projections use the same typed extractor. The initial
query catalog covers shared traits/rarity/publication and applicable level/rank,
Actor defenses/saves/IWR/movement and representative Item-family facts.
Unselected fields remain source data. Generic metric EAV, inferred taxonomy,
guessed gameplay totals and synthetic canonical affliction records are retired.

Selected query values distinguish value, missing, null, invalid and
not-applicable. Known zero, false and empty collections remain values. Logical
composition preserves unknown; only true roots match. Exists shares one child
witness, admits one collection level, and preserves uncertainty when there is
no true witness. Only known typed IWR parents receive the upstream-backed
omission-only empty query default. Source presence is unchanged.

atlas-domain owns the shared predicate and passage-address vocabulary. atlas-index
owns executable field bindings, catalog statistics, parameterized SQL, artifact
validation and narrow readers/writers. CLI CEL and structured app filters lower
independently into the same predicate; no CEL/editor translation is required.
The maintained CEL parser owns grammar. Atlas compiles the supported AST subset
to SQL rather than interpreting expressions over source bodies. Unsupported
syntax, fields, types, nesting and excessive complexity fail before execution.
Availability uses an explicitly declared literal-key metadata map.

CEL-spec truth semantics govern compilation. The cel 0.14.5 interpreter propagates
an unavailable child's error before a later true Exists witness, contrary to the
language specification; it is not the reference oracle for that case or the
product execution path. Regression fixtures record this limitation alongside
order-independent SQL witness tests. See the
[CEL language definition](https://github.com/cel-expr/cel-spec/blob/master/doc/langdef.md#macros).

## Content and retrieval

Authored markup stays in the DTO. The artifact retains one selected sanitized
HTML cache per field with compact local interaction facts, rather than a
persisted RichDocument, general tree, terminal rendering or complete plain-text
corpus. html2text formats terminal output at read time. Browser controls use the
field's markers and sidecar; neither renderer executes Foundry scripts or rules.
Visible reference markers carry an ordinal and compact binding digest rather
than copying target metadata into HTML attributes. The digest binds the sidecar
fact, field locator and authored-field hash. Offline validation checks exact
marker/edge coverage and coherence, without reparsing localized syntax or needing
the original checkout. It detects component drift, not coordinated artifact
tampering or independently verified localized target resolution.
Interaction validation checks marker ordinals and kinds. It does not independently
bind same-kind control parameters to rendered text; this narrower check must not
be described as verification of every prepared fact against authored markup.
Initial preparation includes GM/owner prose and check DCs. Index locale defaults
to English and can be overridden at build time; changing search locale requires
rebuilding. Initial display uses the artifact locale.

Relationships retain attributed occurrences and verified target identities.
Repeated links, unresolved targets and blocked URLs remain distinct.
RollTable Pack results may resolve into the compendium artifact; Document results
use the world-document namespace and remain unresolved here, regardless of how
their collection string is spelled.
Offline validation reconstructs the existing record resolver's compact identity
and name index from checked snapshots to verify resolved structured destinations.
It does not promote unresolved occurrences, because ingest may have additional
excluded-source ambiguity evidence outside the product artifact.
Verified aliases and remaster pairs require explicit evidence. Legacy suppression
applies only when a verified replacement also matches the current request, before
pagination; a legacy-only match remains visible. Suggested variants are a bounded
same-pack/family derived view with transparent naming evidence, not identity,
aliases or result suppression. RollTables are product records; Macros are
developer-only on all product paths, including exact-key access.

Precision FTS indexes identity, verified aliases, typed vocabulary, owned names
and structurally named definitions. Semantic units independently cover selected
root and owned explanatory prose, plus one compact root identity unit. Body text
is recovered from source/prepared content; vector metadata does not retain a
second prose corpus. A typed passage address binds section ordinal and UTF-8 byte
range to source/preparation hashes, with no fabricated identity range.

Only pinned BGE-small-en-v1.5 is supported initially. FastEmbed owns inference,
CLS pooling, normalization and generic batching; text-splitter owns generic
tokenizer-aware segmentation. Atlas owns selection, context, exact input/cache
identity, source attribution and final model-budget checks. UTF-8 and the query
instruction remain intact. No silent truncation or estimator fallback is allowed.
Eligible roots constrain FTS and vector units before ranking, including explicit
key scopes. Semantic roots take their maximum accepted unit score; hybrid fuses
one rank per root per lane. Vector candidate windows remain explicitly bounded.

## Build and consumer boundary

The aggregate source fingerprint hashes the deterministic ordered relevant input
set, including manifest/pack definitions and excluded inputs. Revision alone is
insufficient for exports and local edits. Record paths and hashes remain compact
provenance; a duplicate complete manifest inventory is unnecessary. Model,
localization and preparation policy identities remain separate context inputs.
Normal product reads do not rescan source files.
The context retains only the resolved trait labels used by selected vocabulary,
not a complete localization catalog. This compact input lets offline artifact
validation reconstruct exact lexical terms and unbudgeted semantic identity
context without requiring the source checkout. Tokenizer-dependent final input
assembly is ingest-owned: vector reuse regenerates the old selected inputs and
verifies their hash, token count and attribution before accepting cached vectors.
Offline index checks do not claim tokenizer-backed input verification. Verified alias terms are checked against their
evidence relation rather than trusted as arbitrary FTS input.

Generated artifacts are rebuilt with the new writer, readers, search and consumers
together. There is no legacy artifact compatibility reader or second authority
path. Temporary writes publish only after coherence validation; failure preserves
the prior artifact. A live reader keeps its SQLite connections on one verified
artifact generation; publication cannot mix an older body with newer caches or
query units through a newly opened path. Saved lists and encounters stay in their separate local-state
database. Unresolved saved keys retain snapshots. New encounter HP uses known
effective typed maxima and one supported source/participant adjustment; saved HP,
explicit overrides and edit intent are preserved.

Missing adjustment within a known typed NPC attributes object uses an unadjusted
local participant default, with separate origin metadata. The authored DTO still
reports Missing; it does not acquire an authored Normal flag. The pinned NPC
implementation tests only explicit elite/weak values before adding an adjustment
(`npc/document.ts:45-51,100-111,134-174`). The corpus has 5,237 such missing flags
among 5,492 NPCs, all with integral authored maxima. Treating all of them as unknown
HP would contradict useful known-maximum initialization. Invalid flags and
unavailable parents remain unavailable; explicit source flags and participant
choices retain their precedence and apply once.

This decision supersedes persisted RichDocument and metric authority in ADR0020
and the older artifact/filter contracts. ADR0023's JSON envelopes and exit classes,
ADR0019's intent separation and ADR0031's app-service composition boundary remain;
their historical payload, hydration and fallback models are replaced. ADR0032's
product-purpose requirement applies to derived projections, while generated DTOs
retain the complete admitted source independently of product selection.
The single interpretation owner and thin product boundaries remain unchanged.
