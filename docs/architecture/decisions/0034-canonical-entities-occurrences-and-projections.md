# ADR 0034: Canonical Entities, Occurrences, Runtime Instances, And Projections

Status: accepted at Checkpoint B; implementation is dependency-ordered
Date: 2026-08-24

## Context

The current creature path mixes sparse mechanics, embedded descriptions, presentation documents, and encounter sidecars. Repeated embedded capabilities can lose their context, runtime adjustments can duplicate base facts, and downstream surfaces may compensate for missing canonical structure.

## Decision

`atlas-record` owns the storage-neutral canonical semantic model. Four layers remain distinct:

- Canonical entities own intrinsic identity, classification, publication, source-backed facts, storage-neutral mechanics, content links, and provenance.
- Occurrences own parent/owner identity, authored order, stable occurrence identity, slot/rank/location/grouping, contextual labels, source overrides, and content/reference origin.
- Runtime instances own mutable participant-local state such as HP, conditions, initiative, side, turn/action/resource state, notes, variant, and modifier ledgers. Durable mutable state lives in `atlas-local-state`; `atlas-app-service` composes it with canonical facts.
- Presentation profiles select density, order, disclosure, and interactions from the same semantic model. `search_compact`, `record_detail`, and `encounter_participant` are projections, not alternate records.

Embedded spells, items, and equipment resolve canonical target identity by precedence: a valid stable compendium/source locator resolves to the existing canonical `RecordKey`; another explicit verified stable source identity may resolve next; otherwise a typed actor-owned fallback entity is created rather than name-deduplicating globally. Stable nested occurrence source ID takes precedence for occurrence identity. Authored order remains occurrence data and does not change that identity; owner plus typed family plus ordinal/range is used only as a diagnosed explicitly unstable fallback when the source has no stable nested ID.

Repeated occurrences do not collapse solely because they target the same canonical entity. Parent/entry/group/rank/location/slot/use/label/content context and typed `Missing | Null | Value` overrides/deltas remain occurrence-owned, preserve provenance, and never mutate or duplicate canonical facts. Stable identity must not use content or semantic hashes.

Generated afflictions follow the same ownership rule. The pinned `is_default_visible` construction boolean is replaced by an explicit `canonical | source_instance` role and typed host-instance-canonical relationships. The canonical owns deduplicated intrinsic/user-facing meaning. A source instance owns exact host occurrence/provenance, remains direct/graph reachable, and is excluded only from ordinary ranking to avoid duplicating the canonical result; visibility is not its role or rationale. B4 must preserve one canonical, one source instance, and three relationships in the generated Ghoul fixture, with C1 round-trip validation and D1 retrieval tests.

Creature field families have explicit source/canonical/occurrence/runtime/profile dispositions and fixtures. They include identity/publication, awareness/languages/skills, defenses/IWR, movement, strikes/abilities, spellcasting, resources/gear, owned content, and references. Fireball canonical reuse, repeated occurrences of one spell/item target, stable identity under reordering, and unresolved actor-owned fallbacks are binding fixtures. Unsupported mechanics remain typed and explicit; no owner infers deterministic rules from prose.

B3 implements the durable creature core on `atlas-record::CreatureRecord`. `atlas-ingest` constructs that subtype only from the versioned serialized-source DTO, preserving `Missing | Null | Value`, stable component IDs, authored array/item order, exact record/source provenance, and diagnosed unsupported open or legacy shapes. The core includes source adjustment and alliance, legacy ability facts, source initiative selection, hardness and shield facts, and resource drift alongside the other approved core families. Source alliance remains an intrinsic non-runtime fact; it never selects encounter side. Serialized shield HP and resource values remain provenance-only, and legacy resource `maxx` is retained as typed unsupported drift with a structured diagnostic rather than silently replacing `max`. The complete versioned NPC Source envelope remains available as ingest-only construction state so a narrow canonical projection issue never erases valid siblings. A valid open source value that violates component-ID syntax receives a reversible record-scoped source fallback; absent or null identity uses the diagnosed owner/family/ordinal fallback already defined above. Portable diagnostics record stable code, relative record/source/field identity, observed value, disposition, and owner, and neither fallback fabricates a canonical target or discards the parent. Standard skill facts and Lore skill facts are core awareness data; the embedded contract owns the corresponding Lore entity/occurrence model along with strikes, actions, spells, and equipment. The pre-C1 sparse actor artifact values are derived one-way from the canonical core and are never an alternate canonical or raw-source fallback path.

B4 implements the shared embedded entity/occurrence contract on that same subtype. Typed capabilities retain action economy, frequencies, rolls and DCs, damage and healing kinds, spell entries and prepared slots, ranks and locations, actor ritual DC context, equipment and spell uses, Lore modifiers, and explicit unsupported notes even when an entity has no damage. Explicit item sort determines authored order with stable-ID and source-ordinal tie breaking, while occurrence identity remains independent of order. Serialized scalar fields use a reusable typed-or-unsupported contract: supported values remain typed, while an unexpected individual shape retains its exact JSON value and receives a portable field diagnostic instead of coercing the value or rejecting the parent and valid siblings. Exact compendium/source locators resolve only through verified pack-and-ID identity with `_stats.compendiumSource` before `flags.core.sourceId`; names and slugs are never global deduplication keys. Grant, item-grant, prepared-spell, and linked-weapon IDs are typed provenance relationships whose lifecycle values are retained but never executed. Repeated canonical targets remain separate occurrences, while actor-local labels and source-state deltas remain occurrence-owned and do not alter canonical target facts. Later intrinsic item families retain typed creature occurrence shells and exact unsupported local facts without creating premature standalone product models. Rich-content attachment remains a separate owned slice.

Creature and embedded-item `img` strings are provenance-only Foundry source locators because reuse licensing is not established. Atlas does not copy, fetch, embed, or display them; that policy can be revisited if licensing changes. Embedded-item `folder` is non-addressable Foundry container provenance because canonical item identity, authored order, ownership, and typed relationships are represented independently. `prototypeToken.name` is a token-instance presentation label rather than creature identity; the actor `name` remains authoritative. Grant and item-grant `onDelete` values are Foundry lifecycle-cascade configuration retained for provenance without execution, while their endpoint IDs remain consumed typed relationships.

B5 implements that separate content slice on `CreatureRecord.content`. Record-authored lore and
notes attach to the creature record. Entity descriptions attach to actor-owned fallback entities,
while prose copied from or locally overriding a resolved canonical spell/item attaches to the
specific B4 occurrence and records the canonical target as duplicate-source metadata. Authored
order remains separate from stable source identity. Every content link becomes an ordered typed
reference occurrence with owner/content/provenance parity; safe unknown markup and unresolved
targets remain diagnosed. Embedded `system.description.gm` content follows the same entity or
occurrence ownership for every typed B4 family, carries a distinct embedded GM source kind and
`gm_only` visibility, and remains GM-complete data rather than an authentication filter. Ordinary
creature description projection contains only record-owned content, so embedded capability
descriptions are neither orphaned nor duplicated there.

Metrics, facets, FTS, embeddings, CLI records, app DTOs, and UI sections derive from canonical facts and occurrences. They are not canonical stores. `atlas-app-service` is the final static/runtime composition point, and the frontend renders generated DTOs without Foundry interpretation.

Encounter runtime composition matches `MechanicTarget` variants only while constructing the typed result. The public result has named level, defenses, saves, awareness, abilities, vitals, and action budget plus typed repeated skills, speeds, resources, spellcasting, activities, and conditions. The outer `level` property is a `RuntimeNumberView`; its base/final distinction remains in `base_value` and `adjusted_value` with applied/suppressed modifiers and provenance. User-relevant automation gaps use stable typed limitation codes and typed participant/condition/activity/spellcasting placement targets; their messages are display-only. Malformed, duplicate, unsupported, unmapped, raw-path, publication, null, and source-noise facts remain internal projection diagnostics, while critical missing or unsafe facts fail closed or leave the affected value unavailable. Base/final values and applied/suppressed adjustments carry tagged provenance; encoded canonical target identity is retained only as provenance or internal diagnostics and is never a downstream lookup key.

Record-bearing retrieval carries the canonical body beside the narrow record projection in the `atlas-record`-owned `RetrievedRecord` aggregate. This transports the same canonical semantic model rather than defining a second record truth. The index hydrates it once, search retains it without changing selection semantics, app profiles may consume only its `record` member, and the CLI creature contract projects its `RecordBody::Creature` member directly.

For the migrated creature core, `atlas-record` projects metric rows and categorical actor facts together from `CreatureRecord`. Display and FTS consume those shared rows and categories. Ingest may retain raw extraction only for an explicitly non-migrated fact family whose canonical owner is not yet present; it does not use raw values or prepared-data aliases as a fallback for migrated creature facts.

## Consequences

Night Hag and the approved fixture corpus must round-trip with stable entity/occurrence order and identity. Runtime final values replace corresponding base display values instead of adding a second unadjusted stat block.

Every H1-H11 future family plan must supply the same field-level ledger and fixture quality before approval; H12 rejects missing ledgers/fixtures and generic or catch-all substitution.

The old sparse creature mechanics, orphaned embedded-description list, fallback presentation, and mixed hydration paths are transitional residue to remove after the required audit and visual gates. No compatibility shim is part of the end state.
