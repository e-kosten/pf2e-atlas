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
[ADR 0037](./0037-open-and-indexed-source-values.md). Unsupported selected
constructs stop generation until their semantics are modeled. Do not substitute
arbitrary JSON for unsupported selected values.

Nullable collection entries use inline `Option<T>` and preserve JSON nulls,
positions, key order and the remaining value parser's constraints. Null-only
entries use `()` with a parser that accepts only null; serde serializes this
unit as JSON null. Array/fixed-tuple element unions containing undefined also
allow persisted null, because ordinary JSON serialization writes undefined or
sparse positions as null. No undefined runtime value or array hole is recreated.
The declaration graph retains the original null/undefined evidence.
In object index unions, undefined permits omitted keys and does not admit a
present null; explicit null is required for that representation. Undefined-only
index values still stop generation because they have no persisted value arm.
Named object fields retain the existing SourcePresence policy.

Null-only named values use `SourcePresence<()>`: missing/null states retain the
same policy and present non-null values fail. A null/undefined-only union has the
same persisted null value type. Literal string enums retain exact serde tokens;
empty, digit-leading and punctuation-only tokens receive valid Rust variant names
(`Empty`, `Value0`, `ValueU2D`). Collisions still fail explicitly rather than
silently merging vocabulary entries.

Structural signatures intern child shapes and memoize resolved nodes instead of
expanding repeated descendants into strings. Recursive anchors validate each
reachable shape before allocation; nominal identity still separates recursive
owners. This keeps large shared declaration graphs bounded without weakening
unsupported-shape diagnostics or adding family-specific generation paths.

Rust fields preserve valid leading underscores, map punctuation to underscores,
prefix digit-leading names with an underscore, and use raw identifiers for Rust
keywords. self/super/crate receive a trailing underscore. Converted names must
remain valid and unique, including the generated additional_fields/indexed_fields
slots. Collisions fail generation. Parsers always address the original source key;
new punctuation/reserved/digit mappings carry an explicit serde rename. Existing
camelCase-to-snake_case model serialization retains its current convention.

Compiler extraction records `impossible: true` for intersections proven assignable
to the compiler's `never` type, retaining their original constituents and source
locations. Selection can discard these impossible union alternatives. An empty
resolved member list alone is insufficient evidence.

`atlas-ingest::source_model` exposes callable source slices and primitives; its
public `generated` namespace exposes generated value types, aliases and enums.
Selected root parsers are public in `generated`, taking an ordered `SourceValue`
and contextual path. Recursive implementation functions remain private to
source-model composition. Handwritten document entry points accept JSON bytes;
specific rules use a generated keyed `RuleSource` dispatcher. Item.rules retains
the generic upstream RuleElementSource and does not invoke specific rule parsers.
The production build pipeline and public product/API/frontend models remain under
their existing owners.

## Consequences

Value-shape sharing follows pre-default parsing semantics while distinct family
declaration facts stay in snapshots. Common Item description/publication, core
traits and item grants can be modeled across all registered Item families without
requiring each family to own the same parser. Fixtures supplement families and
states absent from the pinned corpus. Typed-value fidelity and additional-member
preservation do not establish complete family or whole-portfolio coverage.
