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

The common Item source layer is the first completed component, not completion of
all 24 item systems. Family bodies, built-in rule schemas, nested effects, all
actor bodies and other document classes remain tracked work. Canonical conversion,
database/storage design, metrics and UI expansion follow the source-modeling
phase. Pipeline adoption must later finish the direct replacement and retire old
source paths; parser-only completion does not certify migration completion.
