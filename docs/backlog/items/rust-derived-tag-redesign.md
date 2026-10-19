# Derived-tag runtime and editorial research

Status: deferred
Priority: later
Owner: unassigned
Last reviewed: 2026-10-19

## Current intent

Improved semantic search may make derived record labels unnecessary. Preserve
the accumulated knowledge and use the product before committing to ontology,
classification or editorial infrastructure. This is not a promised implementation.

The [durable research package](../../research/derived-tags/README.md) consolidates
the Node matcher/editorial work, canonical concepts and 515-row mapping, Rust
model/seed experiments, October assessment discussion and reviewer research.
It includes unique local authored snapshots, provenance and cleanup boundaries.
[ADR0054](../../architecture/decisions/0054-derived-tag-deferral.md) records deferral.

## Reopening condition

Bring concrete user questions, actual queries/filters, expected and observed
records, source evidence and current profile/reference identity. Compare with
native DTO-backed filters, better field discovery/presets and current search.
Distinguish source/admission, eligibility, candidate-window and ranking problems
from a stable interpretive concept that earns authored membership.

If a meaningful gap remains, review a small concept/example packet before
proposing implementation or whole-corpus assessment. The preserved
[design discussion](../../research/derived-tags/design-if-revisited.md) covers
two connected ontology/assessment loops, broad applicability, revision/coverage
tracking, weak/strong negatives, evidence and developer review. Its open choices
include format, grouping, subjects, storage, reviewer policy and publication.

## Existing capability and boundaries

`atlas-tags` and `atlas tags validate` implement file types, strict parsing and
structural corpus validation. They do not establish record existence, evidence
truth, semantic applicability or classification coverage. Product runtime tag
filtering is unimplemented. Existing file contracts stay unchanged by deferral.

Earlier YAML, closed group/subgroup, single-home, minimal provenance and unanimous
acceptance choices are historical context, not binding requirements for a revived
design. Use [tagging architecture](../../architecture/tagging.md) and the status of
[ADR0028](../../architecture/decisions/0028-rust-tagging-model.md) to distinguish the
implemented surface from conditional future integration. Preserve useful examples
without restoring Node category ownership or heuristic derivation.

Removing existing code, deleting worktrees, runtime implementation and publication
remain separate tasks; this item does not authorize them.
