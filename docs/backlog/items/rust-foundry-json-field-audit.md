# Rust Foundry JSON Field Audit

Status: active
Priority: later
Owner: unassigned
Last reviewed: 2026-06-26

A7 candidate disposition: **subsumed** by B1-B2 source DTO and real-owner coverage enforcement. The current heuristic audit remains evidence/tooling until direct replacement; Checkpoint B is not yet approved.

## Problem

Rust ingest intentionally projects Foundry source JSON into typed records, content documents, metrics, references, aliases, generated records, FTS rows, and embedding units. The current pipeline validates the projections it emits, but it does not have a comprehensive way to ask which source JSON fields appear across the corpus and whether those fields carry meaningful information that the Rust ingest model does not yet capture.

Ad hoc audits for a single projection family are easy to overfit. For example, a metric-specific "candidate source field was not emitted" check can become a second partial metric extractor with subtly different parsing rules from the real extractor. That makes default ingest noisier without providing a trustworthy whole-corpus coverage picture.

## Desired Outcome

Add an offline JSON field audit that inventories all Foundry source fields across the vendored corpus and compares them against explicit Rust ingest coverage.

Current Rust ingest already exposes some ingest-only construction facts, including source slugs, compendium-source locators, embedded item facts, embedded content refs, journal page facts, and journal page skip reasons. The audit should treat those source fact extractors as coverage declarations rather than reimplementing their parsing logic.

The audit should help answer:

- which JSON pointer paths exist by document type, record type, pack, and frequency
- which paths are consumed by normalization, side-data extraction, content parsing, metric extraction, reference/alias/remaster extraction, generated-record logic, FTS, and embedding preparation
- which unconsumed paths have non-empty, non-trivial values and representative examples
- which ignored paths are known provenance, presentation-only caches, implementation details, redundant fields, or intentionally deferred source families
- which candidate fields deserve new typed projections, supplemental content extraction, metrics, references, aliases, generated records, or artifact schema work

## Constraints

- Keep this audit as an explicit offline/reporting command, not an always-on ingest gate.
- Do not duplicate extraction policy inside default ingest just to report missing candidates.
- Prefer instrumenting or declaring coverage at the ingest owner that consumes each source region.
- Report examples and frequencies so reviewers can decide whether a field is meaningful rather than treating every unconsumed path as a bug.
- Keep raw JSON provenance available for audit/debugging, but do not make runtime search or presentation depend on broad raw JSON scans.

## Acceptance Sketch

- The audit can scan the full Foundry source tree and emit a stable JSON report. Initial support exists as `atlas index audit-source-paths`.
- The report groups source paths by document/record type, record type, or pack filter and includes counts plus representative record keys or source paths.
- The report distinguishes consumed, ignored, deferred, and unknown paths.
- Existing ingest owners can declare covered JSON pointer paths or path families without centralizing all source policy in one file.
- Ingest-owned source fact extractors can report covered and skipped nested source regions, including embedded items and journal pages.
- The output identifies a small actionable list of high-signal unknown fields rather than overwhelming reviewers with every mechanical or empty source key.

The first implementation uses an ingest-owned path-family map to mark `consumed`, `partial`, and `uncovered` paths. Follow-up work should replace or supplement those heuristics with coverage declarations from the real extractors that consume each source region.

## Related

- [Rust artifact contract](../../architecture/artifact-contract.md)
- [Runtime architecture](../../architecture/runtime.md)
- [Rust content subdocuments for journal pages and table results](./rust-content-subdocuments-journal-table-results.md)
