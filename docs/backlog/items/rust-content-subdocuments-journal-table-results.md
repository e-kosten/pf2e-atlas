# Rust Content Subdocuments For Journal Pages And Table Results

Status: in progress
Priority: current modeling
Owner: H8 Journal/RollTable family
Last reviewed: 2026-09-08

H8 owns two dedicated top-level canonical families, `Journal` and `RollTable`, with ordered parent-owned `JournalPage` and `TableResult` children. Children use backend-owned opaque locators under the parent route; they are not standalone records. Their rich text contributes to the existing parent record's FTS row, while child-aware search hits and always-materialized child embeddings remain outside this slice.

The approved media disposition is metadata-only: exact image/PDF/video locators and source states remain typed provenance, with no fetch, play, render, rank, or embed behavior. Roll execution is likewise outside H8.

## Problem

The RichDocument migration models primary record descriptions, supplemental record-owned content, embedded actor/item capability text, FTS projections, semantic chunks, and reference edges from explicit rich-content sources. During planning, two rich-text source families were identified as real content but deferred from the first pass:

- journal page bodies at `pages.*.text.content`
- rollable table result prose at `results.*.text`

These fields contain useful HTML/Foundry markup and references, but they are not clearly ordinary supplemental content on the parent record. They often represent nested subdocuments with their own names, ordering, and user-facing meaning.

The H8 implementation directly replaces the former ingest-only journal-page construction facts. Canonical bodies, artifact hydration, public detail surfaces, and typed child reference context share one owner; no raw-JSON reparse or compatibility path remains.

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

## Accepted Outcome

Implement parent-owned nested content that preserves, searches, renders, and links journal pages and table results without promoting them to standalone records or flattening their identity into the parent.

The model preserves:

- stable authored child IDs where unique, and explicitly unstable parent-scoped ordinal locators otherwise
- opaque child locators that clients only round-trip under the parent record route
- exact child source context on content/reference occurrences and graph edges
- parent-record FTS hits with lower-weight embedded child content and no fabricated child winner
- the existing overflow-only semantic-unit policy rather than unconditional child embeddings

## Current contract

- `RecordKind::Journal` admits only a `JournalEntry` document with one `JournalRecord` body; `RecordKind::RollTable` admits only a `RollTable` document with one `RollTableRecord` body.
- `ContentChildLocator` carries the parent record, child kind, and either a unique authored source ID or an explicitly unstable authored ordinal. The backend encodes it as a versioned opaque value; clients only round-trip it.
- `RichLinkTarget::RecordChild` retains an exact parent-plus-child target. Graph traversal remains parent-record based, with optional source and target child locators preserving context.
- Journal page and table result content remain parent-owned `OwnedRichContent`; authored ordering, content hashes, source paths, and reference occurrences survive codec, write, and read boundaries.
- Missing, Null, known zero/false/empty values, malformed values, duplicate members, and unknown members retain distinct typed outcomes. Fixed identity or dispatch duplicates fail closed; other child failures localize to an unsupported child where the child is the smallest owner.
- Ordinary lookup, detail, CLI, app, and UI surfaces expose the two parent families. Media metadata is provenance-only and roll or script execution is absent.

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

## Acceptance

- Real-source fixtures prove Journal and RollTable source states, child identity and order, plus the Hero Point Deck cross-parent page reference.
- A mixed real SQLite fixture proves atomic canonical write, all/keyed hydration, parent-only FTS rows, child content/reference ownership, and symmetric family corruption rejection.
- Public JSON, CLI, app-model bindings, and UI tests preserve the opaque child locator and typed unsupported states without exposing a media viewer or roll executor.
- Stable IDs survive reorder; duplicate, missing, and authored fallback-like IDs cannot collide or silently select a child.

## Related

- [Rust artifact contract](../../architecture/artifact-contract.md)
- [ADR 0020: Rust Rich Documents](../../architecture/decisions/0020-rust-content-documents.md)
- [ADR 0034 candidate: Canonical entities, occurrences, runtime instances, and projections](../../architecture/decisions/0034-canonical-entities-occurrences-and-projections.md)
