# Reference Edge Extraction Expansion

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-03

## Problem

The current `reference_edges` table is the best available shared source for native record-to-record link references, but the indexer currently fills it from only part of the link context present in the Foundry data.

Today the broad durable edge surface is centered on resolved `@UUID[...]` prose references. The vendor data also contains other record-link carriers, such as structured compendium UUID fields, rule-element UUID targets, and journal-page UUID references, that are not yet part of the general `reference_edges` extraction path.

## Desired Outcome

Expand the durable link-extraction pipeline so `reference_edges` covers the intended set of native internal record references the repo wants entity pages, backlink drills, and other shared consumers to rely on.

That follow-up should:

- keep `reference_edges` as the authoritative shared source for native record-to-record links
- add extraction for additional real record-link carriers where the source data identifies another compendium record cleanly
- decide whether some link-like sources belong in `reference_edges` directly or in a separate relation class that the page-relations owner can combine intentionally
- make the expanded edge surface available without forcing page/detail consumers to invent parallel relation sources

## Constraints

- Do not block the current entity-page and reference-navigation work on this expansion. The existing UUID-based `reference_edges` surface is the current implementation baseline.
- Do not use the curated rule-graph backlink view as the generic replacement for `reference_edges`; that path has narrower product semantics.
- Preserve the distinction between native record links and search/ontology pivots such as traits, families, tags, category, subcategory, or pack.
- Be careful with relation semantics from `compendiumSource` and similar structured provenance fields; not every source-level relationship is equivalent to an authored page reference.

## Notes

Current candidate expansion sources from the Foundry data include:

- structured UUID fields such as `system.selfEffect.uuid`
- rule-element UUID targets such as `GrantItem` and nested aura/effect UUID fields
- journal-page UUID references with `JournalEntry` / `JournalEntryPage` shapes that the current prose extractor does not fully capture
- `@Embed[Compendium....]` references in journal content if journals become part of the durable page/reference experience
- selected `compendiumSource` relationships where the repo decides they represent a real shared page relation rather than only provenance or embedded-item sourcing

This item exists so the repo can defer that expansion cleanly while keeping track of the gap discovered during entity-page planning review.

## Related

- [View pages and detail presentation](./view-pages-and-details.md)
- [TUI architecture](../../architecture/node/tui.md)
- [Search runtime architecture](../../architecture/node/search.md)
