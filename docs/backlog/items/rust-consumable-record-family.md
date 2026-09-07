# Rust Standalone and Embedded Consumable Record Family

Status: deferred
Priority: later
Owner: unassigned
Last reviewed: 2026-09-06

## Problem

Consumables do not yet have one source-faithful canonical family across standalone Item records and consumables embedded in Actors. Hazard ingest therefore retains Purple Worm Venom from False Door Trap as a safe-content, non-executable unsupported child. Its useful description and references survive, but its inventory, economy, durability, usage, and charge facts cannot be treated as supported consumable behavior.

The hazard remediation deliberately types only common child metadata such as rarity and lineage. It must not become a partial consumable implementation or allow standalone and embedded consumables to drift into separate semantic models.

## Desired Outcome

Implement one schema-grounded consumable family contract that:

- admits standalone and embedded consumables through the same intrinsic canonical definition while retaining parent-local occurrence context
- preserves stable source identity, authored order, repeated occurrences, parent association, and diagnosed fallbacks without using names or slugs as identity
- gives typed owners to supported inventory, economy, durability, material, usage, and charge facts, preserving `Missing | Null | Value | Unsupported`, false, zero, known-empty values, and exact malformed evidence
- keeps embedded content and reference occurrences attached once to their natural owner and available independently of executable inventory behavior
- defines explicit runtime/use eligibility before any consumable is executable; ingest shape retention alone does not authorize consumption, charge mutation, damage application, or inventory automation
- covers standalone/embedded parity, artifact round-trip, query projection, presentation, and runtime boundaries without raw-JSON or path-based semantic recovery

## Minimum Regression Contract

False Door Trap's embedded Purple Worm Venom is the minimum no-loss fixture. The current hazard boundary retains 20 populated/common source facts: typed child rarity plus 19 exact unsupported leaves. The 20 source paths are:

- `system.baseItem`
- `system.bulk.value`
- `system.category`
- `system.containerId`
- `system.damage`
- `system.equipped.carryType`
- `system.hardness`
- `system.hp.max`
- `system.hp.value`
- `system.level.value`
- `system.material.grade`
- `system.material.type`
- `system.price.value.gp`
- `system.quantity`
- `system.size`
- `system.traits.rarity`
- `system.usage.value`
- `system.uses.autoDestroy`
- `system.uses.max`
- `system.uses.value`

The future cutover must prove that all 20 remain represented exactly once across source DTO, canonical ownership, artifact hydration, and applicable product projections. Typing a leaf may move it from unsupported evidence to a reviewed semantic owner, but must not reduce the regression inventory, change identity/order, or erase the existing content and references.

## Constraints

- This item records future work only; the hazard record-presentation remediation does not implement consumables.
- Standalone and embedded forms must not gain competing canonical definitions or compatibility adapters.
- `_stats.compendiumSource` remains lineage provenance unless a separate authenticated identity decision verifies a canonical target; it is not automatically a record relationship.
- Unsupported or unclassified populated values remain conspicuous human limitations until explicitly modeled.
- Condition/Effect shells and hazard runtime behavior are outside this consumable-family scope.
- Any canonical codec, physical schema, manifest, application DTO, CLI machine-contract, or runtime change requires its owning version/contract disposition and focused regression evidence.

## Related

- [Source-Faithful Contract State](../backlog.md#source-faithful-contract-state)
- [Rust record presentation mechanics unification](./rust-record-presentation-mechanics-unification.md)
- [Canonical entities, occurrences, and projections](../../architecture/decisions/0034-canonical-entities-occurrences-and-projections.md)
- [Atomic canonical artifact](../../architecture/decisions/0035-atomic-canonical-artifact.md)
- [Source-faithful record surfaces](../../architecture/decisions/0036-source-faithful-record-surfaces.md)
