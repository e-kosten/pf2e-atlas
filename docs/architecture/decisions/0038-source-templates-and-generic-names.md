# Source templates and generic names

Status: accepted
Date: 2026-10-07

## Decision

Generate TypeScript template domains with arbitrary string interpolations as
named Rust String aliases and checked parsers. Match the exact prefix, ordered
intermediate literals and suffix. Interpolations may be empty, contain line
breaks, Unicode or further separators. A color prefix does not imply hex
validation, and a file suffix does not imply file existence. Other interpolation
domains and malformed template nodes fail generation explicitly.

Unions consisting of templates and other string alternatives share a String
carrier with the combined constraint. Overlapping string patterns describe one
value domain; they do not create ambiguous enum arms. Literal-only closed sets
retain their checked enums. Mixed unions, array/tuple guards and required object
discriminants retain template constraints while following the existing
shape-sensitive union policy. Rust aliases do not enforce these constraints on
manually constructed strings; the generated source parsers do.

Resolved generic instantiations are concrete source shapes, not Rust generic
types. Prefer the declaration name followed by simple named arguments, such as
SourceFromSchemaChoiceSetRuleSchema. For anonymous or complex arguments, use the
owning field/root context followed by the declaration name. Do not turn a printed
anonymous schema into an enormous identifier or assign traversal counters.
Equivalent shapes reuse their existing owner. Genuine residual naming collisions
remain errors. Generic union alternatives use the same naming policy.

## Evidence boundaries

Full-root emission and compilation are separate from corpus parsing, value
fidelity and Foundry admission. SourceFromSchema can describe values after
Foundry cleaning/coercion, rather than every authored pre-cleaning form. Corpus
rejections must retain source paths and be investigated before production
adoption; do not widen them silently or report a compiled root as corpus-complete.

The committed generated selection remains the existing partial Item, physical
and predicate models. Whole-portfolio outputs are diagnostic artifacts. Database,
metrics, application contracts and UI adoption remain deferred.
