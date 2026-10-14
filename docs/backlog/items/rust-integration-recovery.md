# Selective Integration Feature Review

Status: planned
Priority: product requirements review
Owner: unassigned
Last reviewed: 2026-10-13

## Purpose

Use integration/record-refactor as inspiration for useful product requirements,
especially family-specific reference layouts and encounter interactions. Compare
those requirements with the source-backed workstream's actual capabilities before
selecting implementation work.

The earlier source-model and storage comparisons are preserved in the
[dated recovery inventory](../../research/integration-recovery-inventory.md).
Current source authority and application ownership follow the
[architecture overview](../../architecture/overview.md) and
[ADR 0046](../../architecture/decisions/0046-source-backed-reference-artifact.md).

## Open Work

- Compare reference, spellcasting, equipment and encounter workflows for useful
  behavior missing from the current product.
- Review layout, grouping, preview/drill-down and contextual controls as product
  interactions with the user before implementation.
- Verify required facts and mechanics against checked source data and the current
  Rust contracts. Select implementation and library reuse on that evidence.
- Record distinct agreed work in its owning backlog item, with an acceptance
  example and validation appropriate to the behavior.

Choose branches and PR bases using the current contributor workflow. Feature
inspiration does not establish implementation correctness or authorize adopting
that branch's data/storage model.
