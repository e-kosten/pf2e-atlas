# Rust Rich Source Content Model

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-02

## Problem

The current Rust content model parses Foundry HTML and inline macros into `ContentDocument`. That model is useful for CLI output, FTS, references, and embedding units, but it is intentionally lossy relative to the authored rich content. It drops most HTML-native structure and attributes before any future web surface can use them.

PF2e Atlas needs a source-preserving content model that can represent browser-grade HTML structure and Foundry enrichment semantics together, without falling back to raw HTML as the runtime source of truth.

## Desired Outcome

Design and implement an Atlas-owned `RichDocument` model that becomes the canonical stored representation for authored rich content.

The model should:

- preserve HTML element structure, attributes, text order, and unknown tags safely
- parse Foundry enrichments such as UUID links, checks, damage, templates, actions, rolls, and unknown macros into typed nodes
- drive deterministic ingest/build projections for CLI text, FTS, references, section trees, and embedding units
- support future web rendering without reconstructing presentation from a lossy block model
- avoid making third-party parser DOM types or raw HTML strings the artifact contract

## Constraints

- This is a replacement-content-model refactor, not an optional alternate path.
- The implementation should not leave long-term parallel canonical models for rich text.
- Derived projections such as FTS rows, reference occurrences, and embedding units should remain build-time artifact outputs.
- Foundry markup should still be parsed once through one shared content path; embedding and reference extraction must not parse raw markup independently.
- Architecture docs and ADRs must be updated because this supersedes the current `ContentDocument` canonicality decision.

## Related

- [Rich source document model plan](../../../scratch/plans/2026-06-02-rich-source-document-model.md)
- [RichDocument production migration plan](../../../scratch/plans/2026-06-02-rich-document-production-migration.md)
- [Runtime architecture](../../architecture/runtime.md)
- [Rust artifact contract](../../architecture/artifact-contract.md)
- [Rust content documents ADR](../../architecture/decisions/0020-rust-content-documents.md)
- [Rust CLI content output formats](./rust-cli-content-output-formats.md)
