# Product filter paths and semantics

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-10-17

Review the public filter vocabulary independently of Foundry's Actor/Item source
organization. Spells, feats and actions are Items; hazards and vehicles are Actors.
Those source categories need not define how users understand product filters.

The intermediate direction is shared product-oriented discovery groups for CLI
and UI, with existing CEL identifiers retained. Reuse app-service editor metadata;
do not maintain separate group registries or add aliases during presentation work.

This follow-up should evaluate which concepts can have common paths across record
kinds, where family-specific paths remain meaningful, and what each path promises
about applicability, unknown values, collection ownership and same-entry matching.
Include movement, defenses, traits, level/rank, languages and embedded records.
Consider CLI CEL, structured UI queries, discovery JSON and saved URL predicates
together. Decide migration requirements explicitly rather than accumulating aliases.

Acceptance: an agreed public catalog proposal with representative cross-kind and
collection queries, evidence that shared fields have shared semantics, and a
bounded implementation plan. Keep the executable catalog and index as the filter
authority; presentation grouping must not create a second query model. No source
DTO restructuring or query grammar replacement is implied.
