# Rust Foundry Type Mechanics Parsers

Status: superseded
Priority: retired
Owner: unassigned
Last reviewed: 2026-09-05

The proposed peer-parser initiative is superseded by the source-faithful family contract in ADRs 0033-0036. Creature, hazard, and standalone-spell facts now have distinct canonical family bodies and projections rather than sibling variants of a generic Foundry-type mechanics model.

Families without a canonical body may retain bounded generic mechanics and query metrics. Any future family migration requires its own typed source-fidelity plan and direct cutover; it must not revive `SpellMechanics`, a generic type-parser union, or generic mechanics as a second hydration source.

Scalar query projections remain valid where filtering, discovery, or aggregation requires them. They are derived from the family owner and do not reconstruct canonical bodies.

## Related

- [ADR 0033: Source fidelity and exhaustive coverage](../../../architecture/decisions/0033-source-fidelity-and-exhaustive-coverage.md)
- [ADR 0034: Canonical entities, occurrences, runtime instances, and projections](../../../architecture/decisions/0034-canonical-entities-occurrences-and-projections.md)
- [ADR 0035: Atomic canonical artifact](../../../architecture/decisions/0035-atomic-canonical-artifact.md)
- [ADR 0036: Source-faithful record surfaces](../../../architecture/decisions/0036-source-faithful-record-surfaces.md)
