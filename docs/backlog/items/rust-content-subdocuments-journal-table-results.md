# Rust Content Subdocuments For Journal Pages And Table Results

Status: in progress
Priority: current modeling
Owner: H8 Journal/RollTable family
Last reviewed: 2026-09-08

H8 owns two dedicated top-level canonical families, `Journal` and `RollTable`, with ordered parent-owned `JournalPage` and `TableResult` children. Children use backend-owned opaque locators under the parent route; they are not standalone records. Their rich text contributes to the existing parent record's FTS row, while child-aware search hits and always-materialized child embeddings remain outside this slice.

The approved media disposition is metadata-only: exact image/PDF/video locators and source states remain typed provenance, with no fetch, play, render, rank, or embed behavior. The current family landing does not yet execute table rolls; the human-requested bounded roll contract is a separate pending H8 design/implementation milestone and does not authorize general macro execution or `drawn` mutation.

## Problem

The RichDocument migration models primary record descriptions, supplemental record-owned content, embedded actor/item capability text, FTS projections, semantic chunks, and reference edges from explicit rich-content sources. During planning, two rich-text source families were identified as real content but deferred from the first pass:

- journal page bodies at `pages.*.text.content`
- rollable table result prose at `results.*.text`

These fields contain useful HTML/Foundry markup and references, but they are not clearly ordinary supplemental content on the parent record. They often represent nested subdocuments with their own names, ordering, and user-facing meaning.

The H8 implementation directly replaces the former ingest-only journal-page construction facts. Canonical bodies, artifact hydration, public detail surfaces, and typed child reference context share one owner; no raw-JSON reparse or compatibility path remains.

If they are flattened into the parent record, search and backlinks may point to overly broad parents such as `GM Screen`, `Archetypes`, or `Madcap Top Effect` when the useful content is a specific page or table result. If they are ignored entirely, Rust loses meaningful text and links that may be important for future TUI/CLI navigation and retrieval.

## Evidence

Representative journal containers from the pinned PF2E source:

- `packs/journals/gm-screen.json`
  - `GM Screen` (`S55aqwWIzpQRFhcq`)
  - 57 ordered pages at the accepted source pin
- `packs/journals/hero-point-deck.json`
  - `Hero Point Deck` (`BSp4LUSaOmUyjBko`)
  - includes `Ancestral Might` page `quxPxuMub8k6abzN`

Representative journal page examples:

- `GM Screen / Basic Actions`
  - table of actions with `@UUID[...]` links such as Aid, Arrest a Fall, Avert Gaze, and Burrow
  - content is table-shaped and reads like a reference page, not a paragraph on the parent journal
- `GM Screen / Falling`
  - prose, heading structure, checks, and a link to the Prone condition
- `GM Screen / Conditions`
  - table of linked conditions and summary text
Representative table-result containers:

- `packs/rollable-tables/hero-point-deck.json`
  - 52 ordered results
  - exact result `jDDTMCTfeJotdH7k` links to `Ancestral Might` in the parent journal above
- `packs/rollable-tables/madcap-top-effect.json`
  - result descriptions include links to Stunned, Confused, Slow, Shrink, Illusory Disguise, Immobilized, Mind Reading, Laughing Fit, and Translocate
- `packs/rollable-tables/10th-level-consumables-items.json`
  - result `jyhJwR67lpDdKiDD` retains the exact `pf2e.equipment-srd` / `XWkeL34yJK6t5qUE` target pair, weight 6, range `[1,6]`, and `drawn: false`

## Accepted Outcome

Implement parent-owned nested content that preserves, searches, renders, and links journal pages and table results without promoting them to standalone records or flattening their identity into the parent.

The model preserves:

- stable authored child IDs where unique, and explicitly unstable parent-scoped ordinal locators otherwise
- full parent-scoped canonical child locators plus shorter parent-relative app-route selectors that clients only round-trip
- exact child source context on content/reference occurrences and graph edges
- parent-record FTS hits with lower-weight embedded child content and no fabricated child winner
- the existing overflow-only semantic-unit policy rather than unconditional child embeddings

## Current contract

- `RecordKind::Journal` admits only a `JournalEntry` document with one `JournalRecord` body; `RecordKind::RollTable` admits only a `RollTable` document with one `RollTableRecord` body.
- `ContentChildLocator` carries the parent record, child kind, and either a unique authored source ID or an explicitly unstable authored ordinal. Canonical and machine projections encode that complete locator. App DTOs and routes receive a shorter versioned selector that omits the parent already present in the route; app-service reconstructs the complete locator from the requested parent and rejects selectors not owned by that canonical body. Clients only round-trip backend-provided selectors.
- `RichLinkTarget::RecordChild` retains an exact parent-plus-child target. Graph traversal remains parent-record based, with optional source and target child locators preserving context.
- Journal page and table result content remain parent-owned `OwnedRichContent`; authored ordering, content hashes, source paths, and reference occurrences survive codec, write, and read boundaries.
- Missing, Null, known zero/false/empty values, malformed values, duplicate members, and unknown members retain distinct typed outcomes. Fixed identity or dispatch duplicates fail closed; other child failures localize to an unsupported child where the child is the smallest owner.
- Ordinary lookup, detail, CLI, app, and UI surfaces expose the two parent families. Media metadata is provenance-only. Table roll execution remains a separately bounded pending milestone; arbitrary scripts and `drawn` mutation remain unsupported.

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

### Source coverage evidence layout

The original H8 plan described separate parent and child machine ledgers. The
implemented evidence uses one complete machine ledger for each actual Foundry
selector instead: `journal-entry.yaml` owns the JournalEntry parent fields and
all supported scalar `pages[]` descendants, while `roll-table.yaml` owns the
RollTable parent fields and all supported scalar `results[]` descendants.
This keeps selector identity honest without extending the shared ledger schema
or admitting partial ledgers.

The two exact terminal object paths that the scalar ledger cannot represent,
`$.pages[].image` and `$.pages[].system`, are accounted for in
`journal-entry-child-dispositions.md`. That companion names their canonical
four-state owners and the source-to-DTO-to-canonical and codec/hydration tests
that are sensitive to removal, malformed shapes, and duplicate evidence. The
RollTable result-local policies and zero-observed variants are recorded in
`roll-table-result-dispositions.md`.

At the pinned source boundary, the two machine ledgers declare 33 scalar
identities and exactly equal the accepted v1-admissible parent-selector
prevalence partition. The Journal companion separately binds two independently
observed exact-empty container states to typed structural evidence; those
states are not prevalence entries, machine leaves, or receipt counts. Machine
and companion totals are reported separately rather than combined into a
fabricated leaf-coverage total.

## Related

- [Rust artifact contract](../../architecture/artifact-contract.md)
- [ADR 0020: Rust Rich Documents](../../architecture/decisions/0020-rust-content-documents.md)
- [ADR 0034 candidate: Canonical entities, occurrences, runtime instances, and projections](../../architecture/decisions/0034-canonical-entities-occurrences-and-projections.md)
