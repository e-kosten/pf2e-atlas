# 0028 Rust Tagging Model

## Status

Accepted

## Context

PF2e Atlas needs authored labels that add search and discovery axes not present in native Pathfinder or Foundry data. The old node implementation explored useful vocabulary and editorial workflows, but it was organized around category projections, broad TypeScript facades, review queues, and human-heavy assignment flows.

The Rust product model has different boundaries: record kinds are explicit, filter discovery is typed, normalized record presentation lives in `atlas-record`, artifact schema and validation live in `atlas-index`, and product retrieval routes through `atlas-search`.

## Decision

Rust tagging will be a new typed tagging subsystem, not a port of the old node derived-tag implementation.

- Tags are global concepts with typed applicability predicates.
- Node-style category projections are not semantic ownership.
- Applicability starts with `record_kind` and optional `foundry_record_type` refinements, plus small normalized fact predicates when needed.
- Coarse Foundry document type is not part of baseline tag applicability or tagging context.
- Presentation hierarchy is shallow navigation: display group, optional display subgroup, then tag.
- Tag definitions and assignments are authored as YAML under `data/tags/catalog/` and `data/tags/assignments/<record-kind>/<pack-name>.yaml`.
- Tag ids are validated dotted newtypes, while group/subgroup/kind/applicability/evidence vocabularies are closed Rust enums or validated vocabularies.
- Assignments are record-centered. An assignment entry with `tags: []` means reviewed and intentionally untagged; no assignment entry means not reviewed.
- The baseline does not model excluded tags, separate review files, or formal provenance.
- Assignment evidence is a small mechanically validatable enum; free-form reasoning belongs in notes.
- Agent assignment is automated-first. Automated acceptance requires unanimous agreement after reconsideration; unresolved disagreement escalates to a human.
- Agents may produce secondary ontology suggestions, but novel tags require human approval before catalog or assignment changes.
- Runtime filter rows use `record_tags`, written during regular `atlas index build`.
- Agent context packets reuse renderer-neutral presentation from `atlas-record`, not terminal-rendered CLI output.

## Ownership

The accepted ownership model is:

- `atlas-tags`: tag ontology, YAML parsing, applicability, assignment validation, evidence validation, ontology suggestions, and agent contract DTOs.
- `atlas-record`: storage-agnostic record context and presentation projections for tagging.
- `atlas-ingest`: consumption of validated tag data during index build.
- `atlas-index`: `record_tags` schema, row readers/writers, validation, and SQL filter/discovery behavior.
- `atlas-search`: product-facing tag worklists, applicability discovery, context assembly, reconciliation contracts, and search integration.
- `atlas-runtime`: path/setup policy and service construction.
- `atlas-cli`: command grammar and JSON/text presentation only.

`atlas-domain` remains the owner of shared primitives such as `RecordKind` and `RecordKey`; it does not own tag ontology simply because tags cross crate boundaries.

## Consequences

Implementation should add `atlas-tags` when the first tagging implementation slice lands. It should not add compatibility layers for the old node layout.

Architecture docs should refer to [Tagging architecture](../tagging.md) for the accepted model. Historical node ADRs remain context only.

The current artifact may continue to report derived-tag metadata filters as unsupported until `record_tags`, validation, and query support exist. Once implemented, tag filters must use authoritative SQL rows and normal keyset filtering rather than post-filtering.

Future UI work should compute applicable tags from the current search space and group them by presentation group/subgroup. It should not force users through old category boundaries.
