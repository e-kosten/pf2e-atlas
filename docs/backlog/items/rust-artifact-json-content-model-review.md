# Rust Artifact JSON Content Model Review

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-05-28

## Problem

The SQLite artifact intentionally normalizes many Foundry facts into typed columns and relation tables, but it also stores several normalized JSON projections. The most important examples are rich content fields such as `records.description_json`, `records.blurb_json`, and `record_content.content_json`.

That split is reasonable for presentation, but it creates a modeling question for reference provenance and future snippet-oriented retrieval. `reference_occurrences` identifies the source content with a logical `content_key`, but primary `description` and `blurb` are not rows in `record_content`, so the occurrence source cannot be enforced with one simple foreign key.

The broader question is whether the artifact should keep primary rich content as JSON columns on `records`, move all content units into a unified content table, or keep the current shape and treat content locators as an intentionally loose contract.

## Desired Outcome

Review the artifact JSON content model and decide whether a refactor would improve reference provenance, content hydration, snippets, or future retrieval surfaces.

The review should answer:

- which JSON columns are presentation caches, source provenance, compact list storage, or queryable facts that should be relationalized
- whether `description` and `blurb` should remain on `records` or become first-class content rows
- whether `reference_occurrences.content_key` should gain a real foreign key through a unified content model
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

One possible stronger model is to introduce a unified content-unit table for `description`, `blurb`, and supplemental content, while still storing rich content JSON as the renderable payload for each unit. That would let occurrence provenance point to a concrete content row without requiring a full document-block schema.

## Related

- [Rust artifact contract](../../architecture/artifact-contract.md)
- [Rust Foundry JSON field audit](./rust-foundry-json-field-audit.md)
- [Rust content subdocuments for journal pages and table results](./rust-content-subdocuments-journal-table-results.md)
