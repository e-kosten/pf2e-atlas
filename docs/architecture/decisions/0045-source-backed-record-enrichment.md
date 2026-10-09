# ADR 0045: Source-backed record enrichment and query views

Status: Accepted library boundary; product artifact adoption remains open.

## Decision

`atlas-record::source_record::SourceBackedRecord` owns a validated `RecordKey`,
one complete `FoundryDocumentSource` and derived `SourceRecordEnrichment`.
atlas-record depends on atlas-foundry-model directly. Named modules own identity,
typed traversal, content selection, relationship occurrences and query/text views.
There is no second family schema, metric/EAV replacement or old-record adapter.

`atlas-ingest::enrich_loaded_source` consumes the typed loader with explicit
audience/implicit DC and optional localization context. It establishes identities
and a reference index before enriching each document. The index carries keys,
owner chains, names and small target-family facts, never copied destination
bodies. Loaded pack metadata/order and original bytes/hash/provenance remain in
ingest-owned packets beside admission diagnostics and one addressed record or
retained unavailable body. The admission's complete extra raw tree is released.

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

Owned traversal covers Actor Items, Consumable spells, declared physical
subitems, Journal pages and RollTable results. Unique valid sibling IDs provide
stable locators; every duplicate/missing/invalid ID uses snapshot-local identity.
Order is separate. Snapshot-local positions must not reconcile mutable user state
across rebuilds. Invalid whole collections remain unavailable, never shortened by
raw neighbor salvage. Owned facts do not copy child bodies or promote them into
independent product records.

Content selection uses typed fields and pinned HTMLField, sheet enrichHTML and
HTML editor-template evidence. Plain names/captions bypass macro interpretation.
Journal format1 selects HTML; format2 remains unsupported Markdown, and unavailable
format metadata is never defaulted. Macro commands, rule/patch payloads and
unknown additional strings remain source data. Whole-field and inline visibility
share one preparation traversal. Unknown biography visibility stays explicit and
emits no visible content while retaining recognized hidden references. Enrichment
records its actual audience; product audience defaults remain separate.

References preserve occurrences before graph/display/search filtering. Exact
PF2e compendium IDs and stable owned destinations resolve through the supplied
source index. The pinned pack builder's qualified authored root names also
resolve when unique; ambiguity is unresolved and names cannot invalidate an
exact ID.
Journal-page heading fragments remain in authored occurrences while resolving
the page identity; heading existence and product navigation are separate checks.
The pinned corpus's `.PAGE_ID` links resolve only sibling pages in the current
journal, using the explicit content owner. Arbitrary relative paths, world UUIDs
and external pack identities remain unresolved without supplied evidence.
Structured relationships cover declared provenance, Item grants,
actor-local casting-entry links and prepared spell slots. Casting targets must
have the correct family; repeated slots remain distinct. Null/missing fields
are retained in source, not manufactured as graph occurrences. Opaque or invalid
targets remain source/diagnostic evidence; no rule/grant evaluation or Embed
expansion occurs.

Focused query views borrow values and propagate missing/null/invalid ancestor
states separately from non-applicability. Empty sets/collections, zero and false
are known values. Numbers retain source Number semantics. Broad traits remain
strings where the source admits identifiers; generated closed vocabularies remain
closed. Authored base rank, maximum HP and defenses are not prepared gameplay
totals. Owned queries retain same-child scope. Named text sources attribute child
prose to its owner and use the same audience-filtered preparations.

## Encoding and adoption

Source bodies use the checked snapshot codec in ADR0043; SourceBackedRecord has
no unchecked Serde envelope. Bounded enrichment serialization measures the
library result. It establishes neither the eventual physical format nor a cache
policy. Its repeated locators/availability and prepared strings must be considered
when designing storage, rather than blindly persisting the whole JSON sidecar.

atlas-index owns physical storage, bindings, discovery and filter compilation;
atlas-domain owns shared query vocabulary. CLI CEL and structured UI filters may
lower independently into a shared query plan; bidirectional translation is not
required. FTS fields, semantic units, weights, pooling, budgeting, graph eligibility
and application response/rendering contracts are adoption work. Source DTOs do
not automatically become frontend DTOs.

The current product artifact/build/consumers remain active. Replace them and
remove old metric/RichDocument authority together during artifact adoption.
Unselected existing filters require explicit retain/rename/retire decisions at
that cutover; this initial library catalog does not silently remove them.

## Validation

The developer example loads the real pin, compares original typed snapshot
identities before/after enrichment, checks exact decoded equality, independently
compares selected projection values to authored fields, verifies emitted markers
and reports compact accounting/diagnostic/size/timing evidence. It is not a
published command, persisted receipt or proof of Foundry/browser/search relevance.
Fixtures cover corpus-absent families, partial fields, identity collisions,
format/visibility states, references and same-child scope. See the contributor
command and [query inventory](../../research/source-record-query-projections.md).
The [corpus evidence](../../research/source-record-enrichment.md) records measured
retention, remaining content diagnostics and physical storage implications.
