# Recursive source values and union identity

Status: accepted
Date: 2026-10-07

## Decision

The existing source generator emits anchored recursive values, mixed unions and
fixed tuples. Recursive nominal owners retain upstream identity; acyclic value
shapes can share structural owners. Validate reachable shapes before assigning
recursive identities. Allocate owners before descending. Arrays/maps already
provide layout indirection. Box inline recursive union payloads, then detect
remaining object/tuple cycles after removing those boxed edges.

Also box object union payloads with a conservative inline footprint estimate
above 256 bytes. The estimate accounts for presence discriminants and nested
inline objects, treats collections as heap-backed handles, caps large estimates
and terminates recursive edges. It is a representation heuristic, not an ABI
size promise. Small payloads remain inline; serialization and source acceptance
are unaffected. The keyed RuleSource dispatcher boxes every rule payload so its
layout stays bounded as rule schemas expand.

For object alternatives with a shared required, non-null field whose finite
literal domains are pairwise disjoint, that field identifies the arm. Choose a
discriminator deterministically from the declarations. Validate its uniqueness
and value without requiring other fields supplied by Foundry defaults. The
selected payload retains missing/null/value states, and checks present values;
declaration-forbidden members remain additional data. This supports Actor/Item
family tags without requiring exported packs to contain runtime-populated fields.

Other source union identity uses value kind, required key presence, required
literal constraints and declaration-forbidden key absence. Forbidden keys exclude
these arms even when their value is null or false; ordinary standalone object
parsing still retains these members as additional source data. Overlapping or
optional tags do not qualify for discriminator-only selection.

Match anchored alternatives before broad fallbacks: open JSON domains and
optional-only objects are fallback alternatives when their value kinds overlap
an anchored arm. Use them only if no anchored shape matches. Count all candidates
within the selected group before accepting a result; competing anchored shapes
remain errors. For the single matching arm, verify required member uniqueness
and nullable state, then retain the original nested parse diagnostic. An invalid
anchored payload must not fall through to an open domain. Competing fallback
shapes remain ambiguous; do not choose the first successful parser. Union owner
signatures retain these selection facts even when ordinary payload structs share
pre-default fields.

This boundary is distinct from ordinary object parsing, which preserves
missing/null/value through SourcePresence. Additional and declaration-forbidden
members remain ordered source data. Union identity does not enforce Foundry
runtime constraints such as nonempty atomic strings or exact total object key
counts. Predicate execution remains future work.

Fixed tuples preserve array arity and position types. Empty tuples use `[(); 0]`
so serialization retains `[]`; single tuples retain their trailing comma. Keyword
field names use Rust raw identifiers. Nullable collection entries follow
[ADR 0035](./0035-source-value-generation-policy.md), including null-aware tuple
and scalar-array union guards. Optional/rest tuples, nullable value roots and
recursive aliases without a nominal anchor fail explicitly until their
representations are supported.

Anonymous unions of complete persisted scalar types use member-derived names in
String, Number, Boolean order, such as StringOrNumber or NumberOrBoolean. A complete
true/false pair represents Boolean; a restricted boolean literal remains restricted.
When allocating a value owner, declared names take priority over generated names.
Equivalent shapes retain their existing shared owner. Name collisions fail explicitly.
Concrete generic instantiations and template-string domains follow
[ADR 0038](./0038-source-templates-and-generic-names.md).
Anonymous tuples appear inline and share a private parser; declared or explicitly
selected root names retain aliases. Defer type rendering until recursive owners are
allocated, and import an inline tuple's element types rather than inventing a type
name for the tuple. These source representations do not define product/runtime DTOs.

Value roots use `valueRef` in the existing modular input. `sourceRef`, serialized
array `rawRef` and constructor field `declaredRef` retain original provenance in
the saved closure. These evidence nodes do not become unselected parser fields.
PredicateStatement, Predicate arrays and ChoiceSet's existing constructor-input
projection share one generated owner graph under `rules/predicate`.

## Validation and consequences

Synthetic graph regressions compile generated output against the actual Rust
source primitives. Their saved generated files have TypeScript freshness checks.
They exercise layouts and selection states absent from the pinned corpus.
Fidelity probes compare typed projections in Rust without converting numbers to
strings. Known upstream conflicts remain reported failures; neither parser nor
probe has a record allowlist. Corpus field discovery and full-root emission are
bounded evidence, separate from complete family admission, compiled portfolio
coverage, pipeline adoption and runtime behavior.
