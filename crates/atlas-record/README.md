# atlas-record

`atlas-record` owns storage-agnostic Atlas record and content models.

This crate defines what an interpreted PF2E Atlas record is after ingest has interpreted source data, and it owns projections from those models into presentation, rich content traversal, FTS text, section trees, and reference-bearing content.

## Owns

- `AtlasRecord` and its identity, classification, Foundry, provenance, publication, mechanics, content, variant, and visibility sections.
- `RichDocument` and rich-content traversal/rendering.
- Content source and visibility semantics.
- Record presentation documents.
- FTS and section-tree projections from normalized content.
- Stable typed record-side concepts that are not storage-specific.

## Should Not Own

- Foundry raw source parser structs or HTML/macro parsing policy.
- SQLite table names, columns, or DDL.
- Artifact validation diagnostics.
- Embedding provider execution.
- CLI envelopes or terminal formatting.

## Boundary Notes

Use this crate for semantic record/content shape. `atlas-ingest` constructs these models from source data, `atlas-index` hydrates them from SQLite rows, `atlas-embedding` consumes their presentation/content projections, and `atlas-search` uses them for user-facing retrieval results.
