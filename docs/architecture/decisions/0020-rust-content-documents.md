# 0020 Rust Content Documents

## Status

Accepted

## Context

The Rust ingest path originally normalized Foundry description markup into stripped plain text. That was sufficient for early lexical search and simple presentation, but it loses information that future Rust runtime surfaces need:

- headings, lists, tables, action glyphs, and other authored structure
- inline Foundry references that should become navigable presentation targets
- source context for reference edges and backlinks
- section structure needed for semantic chunking
- markdown-like CLI output and future TUI rendering

The early Rust implementation also let multiple owners reinterpret the same source markup. Ingest stripped description text and extracted references, while embedding code separately parsed raw markup into its own hierarchy for child units. Reference extraction scanned raw JSON broadly, which captured useful links but also pulled in implementation text and other strings that should not silently become ordinary page relations.

This is the same class of ownership problem the Rust migration is meant to avoid: a weak source model forces each downstream consumer to invent its own projection and policy.

## Decision

Rust records will preserve authored rich text as `ContentDocument` values. `ContentDocument` is the canonical Rust model for Foundry-authored rich text, not just descriptions.

`NormalizedRecord` keeps first-class content fields for core record text:

- `description: Option<ContentDocument>`
- `blurb: Option<ContentDocument>`
- `supplemental_content: Vec<SupplementalContentDocument>`

Supplemental content represents named rich-text sources such as hazard disable/routine/reset text, public/GM/private notes, stealth details, and embedded actor/item capability descriptions. It carries explicit source kind, visibility, and search/reference participation policy.

Raw source markup is not a runtime source of truth. It may be retained for ingest, provenance, diagnostics, or debug workflows, but runtime presentation, FTS, semantic chunks, and reference extraction derive from `ContentDocument`.

All plain-text and retrieval documents are projections:

- CLI/TUI text is rendered from `ContentDocument` or presentation documents that preserve rich content.
- FTS documents are weighted lexical projections from record presentation/content, including a lower-weight embedded-content field.
- Semantic embedding units consume presentation/content plus a shared section-tree projection over `ContentDocument`.
- Reference edges derive from resolved inline references in explicit content sources, plus explicit generated or special relationship producers.

Reference edges must persist source kind and visibility so graph consumers can distinguish public primary content, embedded capability content, GM/private content, internal implementation sources, and generated relationship facts. Default backlink and public graph views use public primary content unless a caller explicitly asks for expanded visibility or embedded-source edges.

`ContentDocument` is owned by `atlas-record`. `atlas-ingest` parses Foundry source fields into content documents, resolves references, assigns content visibility/source policy, and prepares build input rows. `atlas-embedding` owns embedding-specific chunk selection, token budgeting, model-facing rendering, unit metadata, and semantic input hashes, but it does not parse raw Foundry markup. `atlas-artifact` owns the physical content and reference-edge storage contract. `atlas-index` writes, validates, and reads that artifact contract.

Journal pages and rollable table results are recognized as rich content, but they are deferred to a separate child/subdocument design. They should not be flattened into broad parent records or recovered through raw JSON scanning.

## Consequences

The Rust content refactor must remove `description_text`, `blurb_text`, and `search_text_projection` as canonical normalized fields. Plain text may be added later only as a deliberate projection, not as the source model.

The recursive raw JSON reference scan is retired. If a JSON field should contribute content or references, it must be promoted into an explicit content source or explicit relationship producer.

Duplicate markup handling in ingest and embedding must be removed. Foundry markup is parsed once into the central content model, and downstream consumers derive their views from that model.

The SQLite artifact contract must change to persist content JSON, FTS projection fields, and reference-edge source/visibility metadata. Artifact validation should reject malformed content JSON and incoherent reference-edge source or visibility values.

Search and presentation quality should improve because structure is preserved:

- title/name/alias/trait/fact/body/reference/embedded-content lexical fields can be weighted independently
- semantic child units can use the shared section tree
- future TUI pages can render headings, lists, tables, action glyphs, and navigable inline references
- CLI output can use markdown-like rendering without exposing raw Foundry markup

The implementation should land as a finished replacement. Compatibility shims that preserve old stripped-text or raw-markup parsing paths are migration debt and should not remain after the refactor is reported complete.
