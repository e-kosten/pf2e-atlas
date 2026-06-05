# Rust Content Subdocuments For Journal Pages And Table Results

Status: deferred  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-16

## Problem

The RichDocument migration models primary record descriptions, supplemental record-owned content, embedded actor/item capability text, FTS projections, semantic chunks, and reference edges from explicit rich-content sources. During planning, two rich-text source families were identified as real content but deferred from the first pass:

- journal page bodies at `pages.*.text.content`
- rollable table result prose at `results.*.description`

These fields contain useful HTML/Foundry markup and references, but they are not clearly ordinary supplemental content on the parent record. They often represent nested subdocuments with their own names, ordering, and user-facing meaning.

Rust ingest now extracts journal page text into ingest-only `JournalPageFact` values so remaster alias/link extraction can consume typed page facts instead of reparsing persisted raw JSON. Those facts intentionally remain construction state; they are not persisted as child content, FTS rows, semantic units, or reference-edge sources. This backlog item owns that later runtime child-content design.

If they are flattened into the parent record, search and backlinks may point to overly broad parents such as `GM Screen`, `Archetypes`, or `Madcap Top Effect` when the useful content is a specific page or table result. If they are ignored entirely, Rust loses meaningful text and links that may be important for future TUI/CLI navigation and retrieval.

## Evidence

Representative journal containers from the vendored PF2E data:

- `vendor/pf2e/packs/pf2e/journals/archetypes.json`
  - `Archetypes`
  - 256 pages, 255 with text content
- `vendor/pf2e/packs/pf2e/journals/gm-screen.json`
  - `GM Screen`
  - 60 pages with text content
- `vendor/pf2e/packs/pf2e/journals/hero-point-deck.json`
  - `Hero Point Deck`
  - 53 pages with text content
- `vendor/pf2e/packs/pf2e/journals/remaster-changes.json`
  - `Remaster Changes`
  - 7 pages with text content

Representative journal page examples:

- `GM Screen / Basic Actions`
  - table of actions with `@UUID[...]` links such as Aid, Arrest a Fall, Avert Gaze, and Burrow
  - content is table-shaped and reads like a reference page, not a paragraph on the parent journal
- `GM Screen / Falling`
  - prose, heading structure, checks, and a link to the Prone condition
- `GM Screen / Conditions`
  - table of linked conditions and summary text
- `Remaster Changes / Feats`, `Spells`, `Bestiaries`
  - large mapping tables with old names and linked new records

Representative table-result containers:

- `vendor/pf2e/packs/pf2e/rollable-tables/hero-point-deck.json`
  - 52 results, all with descriptions
  - results link to individual journal pages for hero point card effects
- `vendor/pf2e/packs/pf2e/rollable-tables/madcap-top-effect.json`
  - result descriptions include links to Stunned, Confused, Slow, Shrink, Illusory Disguise, Immobilized, Mind Reading, Laughing Fit, and Translocate
- `vendor/pf2e/packs/pf2e/rollable-tables/rod-of-wonder.json`
  - 29 results, many with rich descriptions
- item-level random treasure tables such as `10th-level-permanent-items.json`
  - many result rows, but their description semantics may be weaker than effect/result tables

## Desired Outcome

Design and implement a child-content model for nested rich text that can preserve, search, render, and link journal pages and table results without flattening them into broad parent records.

The model should answer:

- whether journal pages and table results are child documents, child records, or supplemental content with child identities
- how child documents are addressed by stable keys
- how CLI/TUI search results identify both the parent and child target
- how backlinks show source context without over-counting broad parent records
- how FTS and semantic search weight nested content relative to parent record descriptions and embedded capability text

## Candidate Shape

One likely shape is a child document model:

```rust
pub struct ContentSubdocument {
    pub key: ContentSubdocumentKey,
    pub parent_record_key: RecordKey,
    pub kind: ContentSubdocumentKind,
    pub title: String,
    pub ordinal: i64,
    pub document: RichDocument,
    pub visibility: ContentVisibility,
}

pub enum ContentSubdocumentKind {
    JournalPage,
    TableResult,
}
```

Example identities:

- `journals:gm-screen#page:Falling`
- `journals:gm-screen#page:BasicActions`
- `rollable-tables:madcap-top-effect#result:2`

Reference edges could either:

- store `from_record_key` as the parent plus `source_subdocument_key`, or
- promote subdocuments into addressable records and allow `from_record_key` to point directly at the child identity if the domain model supports it.

The first option is less disruptive to existing record-key assumptions. The second may produce cleaner search/detail navigation if journal pages become first-class user-visible targets.

## Constraints

- Do not reintroduce recursive raw JSON reference extraction.
- Do not flatten large journal containers into one parent content document by default.
- Do not make broad parent records overrank because a child page or table result mentions a common condition/spell/action.
- Keep journal/table-result source kind and visibility explicit if references are emitted.
- Preserve the distinction between:
  - parent record identity
  - child content identity
  - generated/special graph facts
  - ordinary content-derived references

## Acceptance Sketch

- Journal pages can be parsed into `RichDocument` values with page title and order preserved.
- Journal page skip reasons from ingest construction facts can feed coverage/audit reporting for unsupported or empty page shapes.
- Table results can be parsed into `RichDocument` values with result range/order preserved.
- Search results can expose a child target label such as `GM Screen > Falling` or `Madcap Top Effect > Result 2`.
- Reference/backlink output can show the child source context rather than only the parent record.
- FTS and semantic retrieval can downweight or separately rank child content so exact searches for canonical records are not dominated by broad containers.
- Artifact validation checks child content JSON and source-key coherence if child content is persisted.

## Related

- [Rust Content Document And Reference Refactor Plan](../../../scratch/plans/2026-05-16-rust-content-document-reference-refactor.md)
- [Reference Edge Extraction Expansion](./reference-edge-extraction-expansion.md)
- [View pages and detail presentation](./view-pages-and-details.md)
- [Rust artifact contract](../../architecture/artifact-contract.md)
