# ADR 0034: Canonical Entities, Occurrences, Runtime Instances, And Projections

Status: proposed for Checkpoint B
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

Metrics, facets, FTS, embeddings, CLI records, app DTOs, and UI sections derive from canonical facts and occurrences. They are not canonical stores. `atlas-app-service` is the final static/runtime composition point, and the frontend renders generated DTOs without Foundry interpretation.

## Consequences

Night Hag and the approved fixture corpus must round-trip with stable entity/occurrence order and identity. Runtime final values replace corresponding base display values instead of adding a second unadjusted stat block.

Every H1-H11 future family plan must supply the same field-level ledger and fixture quality before approval; H12 rejects missing ledgers/fixtures and generic or catch-all substitution.

The old sparse creature mechanics, orphaned embedded-description list, fallback presentation, and mixed hydration paths are transitional residue to remove after the required audit and visual gates. No compatibility shim is part of the end state.
