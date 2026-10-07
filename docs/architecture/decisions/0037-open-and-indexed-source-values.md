# Open and indexed source values

Status: accepted
Date: 2026-10-07

## Decision

The source emitter supports explicit upstream open domains through SourceValue
and domain-specific parsing primitives. any and unknown preserve every persisted
JSON value. TypeScript object accepts arrays and objects, excluding primitives
and null. An explicitly empty TypeScript shape (`{}`) accepts non-null JSON,
including primitives. The Rust carrier is deliberately broader than these last
two domains; their parsers enforce the declaration's JSON kind constraint.
No runtime JavaScript values, functions or undefined JSON values are synthesized.
Structured open domains, unresolved types and unsupported constructs still stop
generation; open values are not a fallback for extraction or generation gaps.

Anonymous atomic unions use canonical member-derived names, extending the scalar
order with Object, NonNullish, Unknown and Any. For example, the Item rule selection
value becomes StringOrNumberOrObject. Object includes arrays as required by
TypeScript. Declared names and structural sharing retain the existing policies.
Overlapping broad and specific union alternatives remain reported ambiguities.

A single string index signature may coexist with named fields. Generate an
ordinary struct with SourcePresence named fields, ordered SourceMap indexed_fields,
and ordered SourceObject additional_fields for declaration-forbidden members.
Every non-null named value must satisfy both its field parser and the index value
parser. Named missing/null states retain the ordinary pre-default source policy,
even when a current declaration would reject them after defaults/admission.
Dynamic values use the index parser directly, including its null constraints.
Every modeled key occurs at most once; declaration-forbidden repeated members
remain additional data. Repeated names inside an explicit open value remain intact.

Pure maps remain SourceMap. Undefined in an index value union permits absent keys;
persisted entries still use its remaining declared value types. Explicit nullable
collection unions, multiple/non-string index signatures and indexed intersection
constraints remain unsupported until their representations are defined.
Indexed object identity includes the dynamic value constraint and forbidden-member
set. Named indexed structs can anchor recursion; their map entries provide layout
indirection, while inline recursive named fields retain the existing boxing policy.

## Application and validation

The shared Item source slice uses the generated complete ItemSourceFlagsPF2e
declaration under items/flags. Its pf2e namespace models itemGrants, grantedBy and
rulesSelections plus open entries; other namespaces are ordered maps of unknown
values. The previous partial handwritten flags parser and grant slice are removed.

These source models support later ingest interpretation. Preserved open data does
not automatically become a stored field, metric, API contract or product behavior.
Production ingest continues through its existing parsers.

Synthetic generated fixtures compile against the actual parsing primitives and
exercise domain boundaries, indexed constraints, escaped diagnostics, duplicate
members, recursive indexed owners, cross-module imports and overlapping unions.
The Item corpus probe independently projects the expanded flags from ordered raw
values. Whole-portfolio emission reports remain distinct from compilation,
admission and production adoption.
