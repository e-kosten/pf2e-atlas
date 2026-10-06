# ADR 0038: Parser-Only Source Modeling

Status: accepted
Date: 2026-10-05

## Context

Source discovery exposes common structures across families that have only partial
typed source models. Modeling those structures before further family product
work lets storage and metrics decisions use a broader understanding of the input.
The parser boundary must be independently usable while those decisions remain
deferred.

## Decision

`atlas-ingest` owns source types and parsers for the pinned PF2e/Foundry serialized
contract. The bounded portfolio consists of Actor, Item, JournalEntry, RollTable
and Macro, their declared families, and reachable embedded source components.
Declarations include zero-count families; the observed corpus alone does not
define the complete portfolio.

`atlas_ingest::item_source::parse_common_item_source` parses Item bytes with pinned
version metadata, source identity and optional actor occurrence context. Root and
actor-embedded documents use one common parser. Actor-specific admission policy
and family system refinements are separate from common structural parsing.
The API constructs no canonical record, database artifact, metric, embedding or
presentation model. The current ingest pipeline remains on its existing typed
conversion paths until the later integration phase; this API is not a fallback
or compatibility wrapper around them.

`atlas_ingest::physical_item_source::parse_physical_item_source` composes that
common tree parser with one shared PhysicalSystemSource parser for armor,
backpack, book, consumable, equipment, shield, treasure and weapon. Nested price,
equipment state, identification, material, apex and activation structures are
typed; physical subitems recurse through the same structural parser. Only
physical child discriminators are admitted. Equipment, backpack, book and
treasure have named family refinements; armor, consumable, shield and weapon
remain explicitly pending. Shared fields have one typed
owner rather than being copied into each family or retained as pending JSON.
The same refinements apply to recursive children. Common level, physical bulk
and legacy numeric bonuses share ItemNumberValueSource. Broad trait and usage
vocabularies remain strings rather than closed enums. Book categories and
treasure stack groups use their complete pinned literal vocabularies.

Family applicability rejects declared forbidden fields even when null or empty:
backpack/book subitems and treasure subitems/apex/usage. Treasure's supplied
traits must be empty. Prepared-Data restrictions do not narrow persisted Source
(for example treasure equipped.invested). Defaultable fields continue to retain
missing/null source evidence. Observed legacy equipment stowing, weapon-like
fields, slot and null deletion markers have typed source owners without changing
the authored family or applying gameplay interpretation.

Common fields retain pre-default `Missing | Null | Value`; an explicit null is
source evidence even where a prepared/schema model expects a defaulted value.
Malformed non-null typed values and repeated scalar/structural members produce
contextual diagnostics. Ordered keyed collections preserve repeated members.
Pending family systems, rules and effects retain their source values, with their
typed modeling status explicitly pending. Open flag extensions stay open.
Additional fields on otherwise structured objects remain inspection evidence,
not a claim that all such fields are valid extensions or have been modeled.

The existing ordered serialized source tree is available through this source API
for inspecting pending payloads and open extensions. Product/runtime consumers
continue to use typed canonical records, not tree lookups as semantic fallbacks.
Reuse neutral domain/record types when their semantics and source fidelity match;
introduce different payloads only for actual presence, context, resolution or
authored-versus-derived differences. No new crate is required.

Source-parser completion is measured against the pin and bounded portfolio.
Recognition, typed implementation and parser exercise are distinct states.
Known structured fields cannot count as modeled merely because raw JSON was
retained. Declaration-based and adversarial fixtures supplement corpus evidence.
This phase adds no field-owner ledger, receipts or new artifact readiness gate.
Schema/value discovery remains governed by ADR 0037.

## Consequences

The common Item and shared physical source layers are completed components, not
completion of all 24 item systems. Four physical system refinements are modeled;
the other four family bodies, built-in rule schemas, nested effects, all
actor bodies and other document classes remain tracked work. Canonical conversion,
database/storage design, metrics and UI expansion follow the source-modeling
phase. Pipeline adoption must later finish the direct replacement and retire old
source paths; parser-only completion does not certify migration completion.
