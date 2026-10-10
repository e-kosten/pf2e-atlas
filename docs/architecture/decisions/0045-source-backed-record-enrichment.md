# ADR 0045: Source-backed records and explicit preparation outputs

Status: Accepted library boundary; product artifact adoption remains open.

## Decision

`atlas-record::source_record::SourceBackedRecord` owns a validated `RecordKey`
and one complete `FoundryDocumentSource`. `SourceBackedRecord::new(pack, source)`
derives and validates the key from that body. Private fields and borrowed `key()` /
`source()` accessors prevent replacing either independently after construction.
Failed construction returns the identity error and unchanged body for retention.
Corpus-wide collision exclusion remains ingest-owned. Preparation never changes
that source.
Content and relationship interpretation return explicit derived outputs rather
than state on every record. There is no retained embedded-node inventory,
collection-status list, second family schema, metric/EAV replacement or old-record
adapter. atlas-record depends directly on atlas-foundry-model; named modules own
identity, internal typed traversal, content selection, relationship occurrences
and the existing focused query/text views.

This boundary separates source authority from operations over it. Embedded
identity and collection availability are useful capabilities, but their existence
does not require another stored description of the DTO's structure. The shared
internal traversal serves reference indexing, nested prose selection, structured
relationship extraction and developer counts. Broader borrowed domain views and
public traversal/lookup APIs wait for demonstrated consumer requirements; the
generated source enums already discriminate document and family types.

`atlas-ingest::enrich_loaded_source` consumes the typed loader with explicit
audience/implicit DC and optional localization context. It establishes identities
and a reference index before executing preparation. The index carries keys,
owner chains, names and small target-family facts, never copied destination
bodies. An addressed ingest outcome contains `record`, `content` and
`relationships` as separate fields. Loaded pack metadata/order and original
bytes/hash/provenance remain in ingest-owned packets beside admission diagnostics
and one addressed record or retained unavailable body. The admission's complete
extra raw tree is released. Loading and preparation packets share ingest-owned
`SourceMetadata` and `SourcePackMetadata`; unchanged metadata moves between stages
without a second field definition. Their document payloads, quarantines and
discovery failures remain explicit stage outcomes. Developer loading reports
flatten source metadata into their existing JSON shape.

## Identity and embedded access

Addressing requires a nonempty whitespace-free pack without a colon and a
16-character ASCII alphanumeric authored root ID. It does not alter general
RecordId or source admission. Every colliding root is unavailable; no filename,
content hash or first-wins key is invented. Before releasing admission.raw,
ingest reserves all authored envelope `_id` candidates, including unsupported
or ambiguous roots, solely to exclude identity collisions. This does not produce
a record or extract product fields from opaque data. Unaddressable roots also
exclude their authored name aliases so an available namesake cannot falsely
appear unique; exact ID resolution stays independent. Unaddressable typed bodies
and unsupported roots' bytes/diagnostics remain retained.

Internal owned traversal covers Actor Items, Consumable spells, declared physical
subitems, Journal pages and RollTable results. Unique valid sibling IDs provide
stable locators; every duplicate/missing/invalid ID uses snapshot-local identity.
Traversal preserves authored order independently of ID validity. Snapshot-local
positions must not reconcile mutable user state
across rebuilds. Invalid whole collections remain unavailable, never shortened by
raw neighbor salvage. Child bodies and local overrides stay in their authored
DTO; traversal does not promote them into independent product records.
The private visitor emits nodes incrementally, borrowing bodies and reusing the
current owner chain. Its active ancestor frames retain sibling ID counts needed
for ambiguity decisions; no complete temporary node inventory is constructed.
Consumers copy owner addresses only when retaining an occurrence or index entry.

Existing focused query views read collection availability directly from source.
Developer reporting calculates owned-document counts through the same traversal;
it does not require retaining a per-record node or container inventory.

## Preparation and relationship outputs

`prepare_record_content` borrows the record and accepts explicit audience,
localization and reference context. It returns `SourceContentOutcome` values for
actual selected rich-text fields across root and embedded documents. Typed
selection uses pinned HTMLField, sheet enrichHTML and HTML editor-template
evidence. Journal format1 selects HTML; format2 remains unsupported Markdown,
and unavailable format metadata is never defaulted. Macro commands, rule/patch
payloads and unknown additional strings remain source data.

Present empty and hidden rich text have prepared results. Missing/null/invalid
text remains inspectable through DTO availability and admission diagnostics; it
does not acquire an unavailable preparation row. For an actual selected rich-text
value, unavailable format metadata, unsupported format and preparation failure
remain explicit outcomes. The outcome's source locator belongs to its payload
and is accessible through `locator()` rather than repeated outside and inside it.
Every eligible present rich-text selection receives a prepared result or an
explicit problem; absence alone is never a known-empty/source-availability answer.

Plain names, captions, labels and declared plain biography
values are borrowed directly. `record.text_sources(&content, audience, pack_label)`
combines them with compatible prepared rich text, retaining owner/field identity
and visibility. Plain values are not copied into HTML/text preparation; display
consumers escape them at their boundary.

Whole-field and inline visibility share one preparation traversal. Unknown
biography visibility stays explicit and emits no visible content while retaining
recognized hidden references. Prepared output is specific to the originating
source snapshot, audience/implicit DC, interpreter, localization and resolver
inputs. The caller must keep that association; RecordKey equality alone does not
prove compatibility. These are library results with source evidence, not
audience-safe application responses or automatically reusable caches.

References preserve occurrences before graph/display/search filtering. Exact
PF2e compendium IDs and stable owned destinations resolve through the supplied
source index. The pinned pack builder's qualified authored root names also
resolve when unique; ambiguity is unresolved and names cannot invalidate an
exact ID. Journal-page heading fragments remain in authored occurrences while
resolving the page identity; heading existence and product navigation are separate
checks. The pinned corpus's `.PAGE_ID` links resolve only sibling pages in the
current journal, using the explicit content owner. Arbitrary relative paths,
world UUIDs and external pack identities remain unresolved without supplied
evidence.

`resolve_source_relationships` independently returns occurrences for declared
provenance, Item grants, actor-local casting-entry links and prepared spell slots.
Casting targets must have the correct family; repeated slots remain distinct.
Recognized prose references stay with their prepared content rather than copied
into this structured-link result. Null/missing fields are retained in source,
not manufactured as graph occurrences. Opaque or invalid targets remain
source/diagnostic evidence; no rule/grant evaluation or Embed expansion occurs.

## Query and reporting boundaries

Focused `SourceNodeView`, `ItemSourceView` and `ActorQueryView` accessors borrow
values and propagate missing/null/invalid ancestor states separately from
non-applicability. Empty sets/collections, zero and false are known values. Numbers
retain source Number semantics. Broad traits remain strings where the source
admits identifiers; authored trait access is applicable to Actor as well as Item
families. Generated closed vocabularies remain closed. Authored base rank, maximum
HP and defenses are not prepared gameplay totals. Owned queries retain same-child
scope. New domain views, such as casting or targeting, follow concrete consumers
rather than a complete handwritten mirror of the generated family hierarchy.
Query projections and audience-sensitive text extraction live in separate modules.
Private typed projection dispatch shares identical expressions with explicit
applicable and excluded Item families; a new variant requires an applicability
decision. This adds no public family schema, query DSL or field registry.

Ingest developer reports retain meaningful admission/preparation problems with
record/file/field attribution, grouped counts and their consequence for typed use.
Optional counts are calculated from source traversal. Routine optional-field
absence is not an error or a required report row; raw evidence remains retained
under the existing admission contract. Normal CLI/UI record responses use usable
data and explicit availability where a feature needs it, without carrying ingest
diagnostic payloads. Operational product errors remain product errors.

## Encoding, locale and adoption

Source bodies use the checked snapshot codec in ADR0043; SourceBackedRecord has
no unchecked Serde envelope. Bounded serialization of preparation/relationship
outputs measures library cost. It establishes neither the eventual physical
format nor a cache policy; persist selected projections for concrete consumers
rather than blindly serializing an ingest result packet.

Preparation runs during ingest/build for the initial adoption path. The agreed
indexing policy defaults to English with a user override, recorded with relevant
localization identity in artifact metadata at adoption. FTS and document embedding
inputs use that indexing context; changing search locale requires re-indexing and
rebuilding or invalidating affected projections/vector cache identities. Initial
display uses the artifact locale. The retained markup and explicit preparation
context permit future display-localization work without changing record authority
or search implicitly. Locale resolves available catalog entries rather than
translating all authored prose. No new CLI option, UI locale feature, multi-locale
index or runtime preparation framework is implemented by this library.

atlas-index owns physical storage, persisted lookup indexes, bindings, discovery
and filter compilation; atlas-domain owns shared query vocabulary. Pure in-memory
source/reference lookup belongs to atlas-record and executes with ingest-supplied
context. CLI CEL and structured UI filters may lower independently into a shared
query plan; bidirectional translation is not required. FTS fields, semantic units,
weights, pooling, budgeting, graph eligibility, prepared-output caching and
application response/rendering contracts are adoption work. Source DTOs do not
automatically become frontend DTOs.

The current product artifact/build/consumers remain active. Replace them and
remove old metric/RichDocument authority together during artifact adoption.
Unselected existing filters require explicit retain/rename/retire decisions at
that cutover; this initial library catalog does not silently remove them.

## Validation

The developer example loads the real pin, compares original typed snapshot
identities before/after preparation, checks exact decoded equality, independently
compares selected projection values to authored fields, verifies emitted markers
and reports compact accounting/diagnostic/size/timing evidence. It is not a
published command, persisted receipt or proof of Foundry/browser/search relevance.
Fixtures cover corpus-absent families, partial fields, identity collisions,
format/visibility states, references, sparse content output and same-child scope.
See the contributor command and
[query inventory](../../research/source-record-query-projections.md).
The [corpus evidence](../../research/source-record-enrichment.md) records measured
retention, remaining content diagnostics and physical storage implications.
