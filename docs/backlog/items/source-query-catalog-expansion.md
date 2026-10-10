# Source query catalog expansion

Status: deferred
Priority: after source-backed artifact adoption
Owner: unassigned
Last reviewed: 2026-10-09

The initial catalog intentionally covers representative useful shared, Actor and
Item-family filters. Extend it only for concrete retrieval needs; the generated
DTO already preserves other authored fields without making them filterable.

Candidates include frequency, skills, finer collection scopes and additional
family-specific facts. Keep different meanings and units distinct. Add each
binding, availability policy, operators, discovery, parameterized SQL and direct
DTO oracle together. Reuse stable field shapes and the maintained UI builder;
do not reintroduce an anonymous metric EAV table or hand-maintained expression
language. New numeric inputs must preserve exact supported precision.

Nested quantification, arbitrary arithmetic, field-to-field comparison and
relationship predicates require a separate query-contract decision. No generic
source-path escape hatch is implied by this backlog item. Query-aware text facets
remain their own [follow-up](./rust-web-text-scoped-filter-counts.md).

See [ADR 0046](../../architecture/decisions/0046-source-backed-reference-artifact.md)
and the [artifact contract](../../architecture/artifact-contract.md).
