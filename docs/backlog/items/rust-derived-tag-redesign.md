# Rust Derived-Tag Runtime And Editorial Redesign

Status: planned
Priority: later
Owner: unassigned
Last reviewed: 2026-10-13

## Problem

Derived tags describe concepts that complement record kinds, source axes, typed
traits and query facts. Their ontology, assessment workflow and product integration
need a deliberate design grounded in those source-backed contracts.

## Desired Outcome

Define and implement a Rust-owned derived-tag runtime and editorial workflow after the core CLI/search path is stable.

The redesign should decide:

- which tag concepts remain runtime filter axes
- which concepts become authored taxonomy or editorial-only metadata
- which concepts retire because record kind, traits, typed query facts or graph context cover the use case better
- how global tag definitions, applicability predicates, shallow display group/subgroup navigation, and search-space tag discovery replace the old common-tag plus category-projection model
- what Rust struct, enum, trait, and file layout should exist before any real tag taxonomy is implemented
- which fields are closed typed vocabularies versus open authored strings, so group/subgroup/kind/applicability values do not drift through string proliferation
- how record assignment sets, reviewed-empty records, exemplars, and generated candidates should be represented on disk
- which CLI and future TUI workflows are needed for review, assignment, evaluation, and discovery
- how agent workflows should support automated batch assignment, tag audit, exemplar review, seed migration, and rule maintenance
- how CLI endpoints should expose untagged worklists, applicable tag definitions, record context, proposal validation, reconciliation, and import/writeback so agents can assign tags by quorum before escalating only unresolved cases to a human
- how agents can make secondary ontology-expansion suggestions without allowing unapproved novel tags to affect assignments or search
- how disagreement evidence should be routed back through agents for reconsideration instead of letting a coordinator make semantic assignment decisions alone, with unanimous agreement required for baseline automated acceptance
- how retained tag concepts appear in search, filter discovery, and record presentation

## Constraints

- Use the existing `atlas-tags` owner for definitions and validation; agree runtime
  and editorial integration before expanding its product responsibilities.
- Do not depend on retired implementation files as the runtime model.
- Keep high-churn candidate discovery and clustering behind stable runtime tag consumption.
- Preserve useful authored knowledge only through a deliberate migration or seed format.
- Preserve useful workflow intent from retired tag-maintenance skills, but rebuild it around Rust CLI commands, Rust-owned storage, and typed machine-readable JSON contracts.
- Treat the old node worktree as vocabulary and workflow inspiration only; do not treat its folder layout, review queues, or TypeScript facades as the target architecture.
- Do not recreate node-style category projections as semantic tag ownership. Rust tags should be global concepts with applicability over record kind, optional Foundry record type refinements, and other small normalized fact predicates. Do not include coarse Foundry document type (`Actor`, `Item`, etc.) in the baseline tag applicability or tagging context model; revisit only if a concrete future use case proves it is needed.
- If presentation needs hierarchy, use shallow display group/subgroup navigation only; do not make display hierarchy part of tag identity, applicability, assignment validity, or filter semantics.
- Pin down model and module boundaries before adding concrete tag families beyond fixtures needed to validate the shape.
- Do not make excluded tags or formal provenance part of the baseline assignment model. If needed, keep near-miss reasoning in notes or external review artifacts.
- Preserve the difference between a missing review and a reviewed record with no justified tags.
- Novel tag suggestions are useful, but they must remain secondary outputs requiring human approval before changing the catalog or authored assignments.
- Agent disagreement should be resolved through reconsideration with shared evidence until quorum is reached or the case escalates to a human.
- The catalog and assignment corpus should be authored as YAML under `data/tags/catalog/` and `data/tags/assignments/<record-kind>/<pack-name>.yaml`, not Rust source constants. Rust owns the typed representation, parsing, validation, and runtime/artifact consumption.
- Runtime tag rows should use `record_tags`, not legacy/product-confusing `record_derived_tags`, and authored tags/assignments should be consumed during regular `atlas index build`.
- Ontology suggestions should include the triggering record and rationale; broader exemplar research can be delegated as follow-up instead of inflating assignment-agent context.

## Notes

The accepted model is captured in [Tagging architecture](../../architecture/tagging.md)
and [ADR 0028](../../architecture/decisions/0028-rust-tagging-model.md). These remain
the authority while further ontology/assessment design is provisional. An approved
revision must reconcile those documents with its implementation; a working proposal
alone does not replace their policy.

## Related

- [Runtime architecture](../../architecture/runtime.md)
- [Artifact contract](../../architecture/artifact-contract.md)
- [ADR 0026: Rust CLI product surface](../../architecture/decisions/0026-rust-cli-product-surface.md)
