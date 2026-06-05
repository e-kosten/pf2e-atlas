# Rust Artifact JSON Content Model Review

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-05-28

## Problem

The SQLite artifact intentionally normalizes many Foundry facts into typed columns and relation tables, but it also stores several normalized JSON projections. The most important rich-content example is `record_content.content_json`, which stores canonical `RichDocument` payloads for description, blurb, supplemental, embedded, and generated content rows.

That shape is reasonable for presentation, but it still creates modeling questions for future snippet-oriented retrieval and referential integrity. `reference_occurrences` identifies the source content with a logical `content_key` that matches `record_content.content_key`; future work should decide whether that contract is strong enough or should become a stricter foreign-key-backed content identity.

The broader question is whether the artifact should keep the current unified rich-content table with JSON payloads, add stronger relational constraints around content identity, or split additional derived projections out of JSON when product/query needs justify it.

## Desired Outcome

Review the artifact JSON content model and decide whether a refactor would improve reference provenance, content hydration, snippets, or future retrieval surfaces.

The review should answer:

- which JSON columns are presentation caches, source provenance, compact list storage, or queryable facts that should be relationalized
- whether the current `record_content` rows are sufficient as first-class content identity for descriptions, blurbs, notes, embedded content, and generated content
- whether `reference_occurrences.content_key` should gain a real foreign key to `record_content`
- whether any duplicated JSON/list fields should be replaced by joins, retained as hydration caches, or documented as intentional denormalization
- whether the current shape creates measurable query, hydration, or maintenance costs

## Constraints

- Do not block the current `reference_occurrences` slice on this review.
- Do not normalize rich document block structure into many low-level relational rows unless there is a concrete product or query need.
- Preserve efficient record hydration for CLI/detail output.
- Keep raw source JSON available for audit/debugging, but do not make product retrieval depend on scanning raw JSON.
- If the chosen model keeps JSON, document which JSON fields are intentional presentation caches versus normalized query surfaces.

## Notes

The current artifact already extracts many queryable projections from rich content: reference edges, reference occurrences, FTS text, embedding inputs, metrics, aliases, and filters. The main concern is not that JSON exists, but whether JSON-only ownership hides data that needs relational identity or referential integrity.

One possible stronger model is to keep the unified content-unit table while making occurrence provenance a concrete foreign-key relationship to `record_content`, without requiring a full document-block relational schema.

## Related

- [Rust artifact contract](../../architecture/artifact-contract.md)
- [Rust Foundry JSON field audit](./rust-foundry-json-field-audit.md)
- [Rust content subdocuments for journal pages and table results](./rust-content-subdocuments-journal-table-results.md)
