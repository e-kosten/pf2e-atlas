# ADR 0035: Generated Source Values Before Defaults

Status: accepted
Date: 2026-10-06

## Context

PF2e source declarations describe several shapes after migrations or schema
defaults. Authored packs omit required description/publication members, retain
legacy fields forbidden by current declarations, and contain trait identifiers
outside current family vocabularies. Source models must preserve these facts
without duplicating family implementations or conflating them with product DTOs.

## Decision

Generated object fields retain `SourcePresence` before defaults. Requiredness,
nullability and forbidden-member evidence remain in saved input. A forbidden
member has no typed owner in this selected slice and remains ordered additional
data when persisted, including null/empty or repeated values. Unselected members
also remain additional data. These parsers provide typed slices, not full Foundry
admission validation.

Broad trait value arrays use an explicit `openTraitArrays` manifest policy keyed
by the exact array node identity. Before generation, each policy must still refer
to an array of declared string values. Original vocabulary nodes stay unchanged
in snapshots, and the extracted trait catalog retains authored metadata. This
policy does not widen unrelated literals or all string arrays. Current finite
other-tag vocabularies, rarity, publication license and grant deletion behavior
remain checked sets. An array of `never` admits only an empty array.

Ordinary arrays produce typed vectors. Pure string-indexed objects produce
`SourceMap<T>` with typed values and authored key order. Repeated modeled map
keys reject with contextual paths; repeated additional members remain intact.
Named-plus-indexed objects and explicit upstream open domains follow
[ADR 0037](./0037-open-and-indexed-source-values.md). Indexed intersection
constraints, explicit nullable collection unions and unsupported selected
constructs stop generation until their semantics are modeled. Do not substitute
arbitrary JSON for unsupported selected values.

Compiler extraction records `impossible: true` for intersections proven assignable
to the compiler's `never` type, retaining their original constituents and source
locations. Selection can discard these impossible union alternatives. An empty
resolved member list alone is insufficient evidence.

`atlas-ingest::source_model` exposes callable source slices and primitives; its
public `generated` namespace exposes generated value types, aliases and enums.
Parser implementation functions remain private to source-model composition.
The production build pipeline and public product/API/frontend models remain under
their existing owners.

## Consequences

Value-shape sharing follows pre-default parsing semantics while distinct family
declaration facts stay in snapshots. Common Item description/publication, core
traits and item grants can be modeled across all registered Item families without
requiring each family to own the same parser. Fixtures supplement families and
states absent from the pinned corpus. Typed-value fidelity and additional-member
preservation do not establish complete family or whole-portfolio coverage.
