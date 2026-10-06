# 0020 Rust Rich Documents

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

Rust records preserve authored rich text as `RichDocument` values. `RichDocument` is the canonical Rust model for Foundry-authored rich text, not just descriptions. It preserves HTML elements, attributes, text nodes, Foundry links, and other Foundry enrichment nodes together so CLI, embedding, and future web rendering can derive their own projections from one source-preserving tree.

`AtlasRecord` stores content as an ordered `RecordContent` document set:

- `ContentSourceKind::Description` for primary descriptions
- `ContentSourceKind::Blurb` for short summary text
- additional `RecordContentDocument` entries for details, notes, embedded capability descriptions, generated affliction content, and other special content sources

Each content document carries explicit owner, role, source kind, label, visibility, provenance, order, and search/reference participation policy.

The creature implementation stores those documents in `CreatureRecord.content`, keyed by the
composite `(parent_record_key, content_key)`. Stable singleton field keys and embedded source IDs
are allocated during ingest; a missing embedded source ID uses the diagnosed authored-ordinal
fallback only. The canonical content locator never contains `content_hash` or
`semantic_input_hash`. Content hashes cover the canonical parsed tree and may change after a text
or structure edit without changing the source content key.

Actor lore/details/notes are record-owned. An embedded capability without a resolved canonical
target is owned by its typed actor-owned entity. When an embedded spell/item resolves to an
existing canonical record, its actor-local copied or overridden prose is occurrence-owned and
records the canonical target as duplicate-source metadata. It never becomes parent-owned creature
content and never mutates or duplicates canonical target facts. Embedded `system.description.gm`
uses a stable source key, the embedded entity or occurrence owner, the embedded GM source kind,
and `gm_only` visibility. That visibility is retained classification data, not current
authentication or a reason to discard the document. A later intrinsic item family still has a
current typed creature occurrence shell, so family deferral alone cannot orphan or exclude its
content.

All authored rich content is stored in `record_content`; `records` does not carry special `description_json` or `blurb_json` columns. Each stored content row carries typed owner/role/source/visibility/provenance and a stable owner-relative content key. `content_hash` and `semantic_input_hash` are change/cache identities, never locators.

Raw source markup is not a runtime source of truth. It may be retained for ingest, provenance, diagnostics, or debug workflows, but runtime presentation, FTS, semantic chunks, and reference extraction derive from `RichDocument`.

`atlas-record` also owns the one structured presentation projection from `RichDocument`. That projection preserves authored heading, paragraph, list, table, and divider blocks; paragraph content preserves typed text, emphasis, strong text, code, references, line breaks, and Foundry `Check` nodes. A public `Check` inline carries its display fallback plus optional statistic and numeric difficulty class, while plain and markdown projections continue to render the same display text. For example, the authored Abyssal Plague check remains a typed Fortitude check with DC 28 and display `Fortitude DC 28`; consumers do not recover the statistic or DC by parsing prose or raw markup.

All plain-text and retrieval documents are projections:

- CLI/TUI text is rendered from `RichDocument` or presentation documents that preserve rich content.
- App record surfaces consume the structured presentation projection and retain its authored block order instead of flattening content into one text value.
- FTS documents are precision lexical projections from normalized record identity and structured high-signal fields. Rich prose is intentionally not indexed in the default V1 FTS projection.
- Semantic embedding units consume presentation/content plus a shared section-tree projection over `RichDocument`.
- Reference edges derive from resolved `FoundryLink` nodes in explicit content sources, plus explicit generated or special relationship producers.

Reference edges and reference occurrences must persist owner/content/section identity, source kind, visibility, provenance, and relation kind so graph consumers can distinguish primary content, embedded capability content, source-classified GM/private content, implementation provenance, generated facts, ordinary references, and Foundry embeds. Atlas currently has no authentication boundary, but pinned-base default graph modes are public-only; Checkpoint A's target makes useful authored edges available regardless of visibility classification. Embedded copied capability edges may require an expanded capability mode to prevent duplicate/noisy results, and implementation-only provenance may remain inspection-only because it has no authored product meaning, but every target exclusion requires a non-auth product rationale and audit evidence. This typed metadata makes no current security claim and remains available for a future separately approved auth feature.

`RichDocument` is owned by `atlas-record`. `atlas-ingest` parses Foundry source fields into rich documents, resolves record references into stored `RecordKey` and target-name data, assigns content visibility/source policy, and prepares build input rows. `atlas-embedding` owns embedding-specific chunk selection, token budgeting, model-facing rendering, unit metadata, and semantic input hashes, but it does not parse raw Foundry markup. `atlas-index` owns, writes, validates, and reads the physical content and reference-edge storage contract.

Journal pages and rollable table results are recognized as addressable child rich content and remain assigned to the separate H8 family plan. They retain child identity, parent context, order, typed visibility/provenance, and navigable targets; they are not flattened into broad parent records or recovered through raw JSON scanning.

## Consequences

The Rust content refactor removes `description_text`, `blurb_text`, `search_text_projection`, `records.description_json`, and `records.blurb_json` as canonical normalized fields. Plain text may be added later only as a deliberate projection, not as the source model.

The recursive raw JSON reference scan is retired. If a JSON field should contribute content or references, it must be promoted into an explicit content source or explicit relationship producer.

Duplicate markup handling in ingest and embedding must be removed. Foundry markup is parsed once into the central rich content model, and downstream consumers derive their views from that model.

The ingest implementation also uses the parsed tree for remaster-journal alias/reference
conversion; it does not retain a second raw markup scanner. Unknown macros remain safe
`UnknownFoundry` nodes. Meaningful unsupported tags/attributes and unresolved links produce typed
diagnostics, while reference occurrences preserve canonical traversal order and source-content
metadata parity.

The SQLite artifact contract persists all authored rich content as `record_content.content_json`, keeps structured FTS projection fields, and stores reference-edge source/visibility/relation metadata. Artifact validation should reject malformed rich content JSON and incoherent reference-edge source, visibility, or relation values.

Search and presentation quality should improve because structure is preserved:

- title/name/alias/trait and other structured lexical fields can be weighted independently, while rich prose remains available for future FTS experiments from the stored rich model
- semantic child units can use the shared section tree
- future TUI pages can render headings, lists, tables, action glyphs, and navigable inline references
- CLI output can use markdown-like rendering without exposing raw Foundry markup

The implementation should land as a finished replacement. Compatibility shims that preserve old stripped-text or raw-markup parsing paths are migration debt and should not remain after the refactor is reported complete.
