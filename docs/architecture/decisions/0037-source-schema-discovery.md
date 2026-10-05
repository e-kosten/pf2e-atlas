# ADR 0037: Source Schema Discovery

Status: accepted
Date: 2026-10-05

## Context

The source audit needs to reveal the shape of Foundry input data and changes in that shape. The source-leaf framework grew into a second implementation of parser, projection, and persistence behavior, with hand-authored ownership ledgers and thousands of lines of receipt machinery. Requiring a model owner or omission decision for every observed field adds maintenance without establishing product value.

## Decision

`atlas-ingest::audit_source_paths` owns offline schema discovery. `atlas index audit-source-paths` emits a versioned, portable snapshot grouped by Foundry document type, root record discriminator, and normalized JSON path. Paths include containers and scalar leaves, JSON types, document and occurrence counts, duplicate-member counts, and bounded source examples. Arrays use `[]`; the explicitly keyed `damageRolls`, `damage`, `overlays`, `itemGrants`, and `system.items` maps use `*`. Other keys stay literal, with JSON string escaping for punctuation. The duplicate-preserving serialized source tree is reused so repeated object members are counted with their corresponding types and values.

Discovery scans the source manifest's declared pack documents, including embedded data. It shares ingest's pack resolution and excludes `_folders.json` control files. Missing selected packs, malformed JSON, and ambiguous root identity/discriminator values are errors. The snapshot is an observation of this corpus, not a declaration of every possible upstream schema or registration-only type. Nested members are grouped under their root document family; the report does not infer a separate schema for every embedded discriminator.

Snapshots carry a digest of manifest bytes plus selected source paths and bytes, independent of absolute checkout location. That digest identifies the observed input; it is not a claim that the checkout matches the pinned source. The checked-in baseline identifies the upstream commit from which it was generated. Counts and examples assist inspection but do not turn ordinary value changes into schema changes.

`--baseline` compares added/removed paths, changed JSON type sets, and the appearance/disappearance of duplicate object members. `--strict` requires a baseline and exits with code 3 when those changes exist. Comparison uses the full inventory before display filtering. A truncated report is explicitly incomplete and cannot be a baseline; source-selection filters must match. Baselines may be either the standard CLI JSON envelope or its bare `data` object.

`atlas-ingest::discover_source_values` and `atlas index source-values --path …` inspect one exact normalized inventory path through the same offline scanner and duplicate-preserving traversal. The versioned `pf2e-source-values/v1` report separates root document families, missing path occurrences from explicit null, per-document counts from repeated occurrences, and JSON type counts from distinct serialized values. Each distinct value retains bounded concrete source references, so rare variants are visible alongside common values. Complete compact JSON in `value_json` preserves duplicate object members and authored array/member ordering; ordering differences remain separate values. References include RFC 6901 pointers into concrete source documents; duplicate keys share a pointer and require inspecting source bytes. Value limits apply after full counting and explicitly mark incomplete output. Source-value reports are inspection tools, not schema baselines, modeling gates, or runtime product inputs.

No path ownership registry, omission ledger, receipt capture, or per-field model-completeness gate remains. Schema drift prompts inspection and a reviewed snapshot refresh. Parser and canonical artifact tests protect product behavior, including presence, identity, unsupported values, ordering, and round-trip fidelity, through their production owners. Artifact storage validation and ordinary runtime readiness retain their existing responsibilities. Discovery does not parse raw JSON during runtime lookup, search, presentation, or encounter use.

This decision replaces the exhaustive source-leaf and zero-unassigned registry requirements in ADRs 0032-0034. It preserves the typed serialized-source boundary, canonical ownership, GM-complete product intent, and product-driven promotion of useful source facts.

## Consequences

The diagnostic implementation stays small and independent of the number of modeled fields. Corpus snapshots can grow as data grows without adding executable validation logic. A stable snapshot establishes the observed input shape; it does not certify that Atlas models every useful field. New modeling work is prioritized through product review and focused behavior tests.
