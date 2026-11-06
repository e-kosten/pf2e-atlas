# Contextual glossary references

Status: in_progress
Priority: soon
Owner: unassigned
Last reviewed: 2026-11-05

## Remaining outcome

Extend contextual help and Advanced Filter discovery to senses, languages, domains,
groups and reusable ability definitions. Reuse existing glossary records and their
prepared content wherever available. Standalone definitions extend the trait
storage owner in place. Open contextual help in the shared Drawer; discovery comes
from record context and filters. A dedicated glossary browser is outside this scope.

The initial implementation slice covers generated source evidence, canonical
definition storage/publication, offline migration and explicit setup freshness.
Remaining retrieval and presentation work must provide readable language names with
communication notes, sense qualifiers, sourced definition content and exact related-
record filters for every included entry. Preserve the distinction between having
a property and explicitly referencing a definition. Labels without source
definitions stay useful filter values; do not invent rules prose or name-based links.

Trait pages, exact membership/reference navigation, stored-vector Similar, CLI
lookup and shared document chrome were delivered in
[PR93](https://github.com/e-kosten/pf2e-atlas/pull/93); see the
[completed trait milestone](../history/items/trait-reference-pages.md).

Do not reopen trait identity or introduce a parallel generic term store merely
to extend the namespaces. [ADR0056](../../architecture/decisions/0056-trait-reference-entities.md)
records the existing typed identity, publication and ownership boundaries.

## Source investigation

The pinned inventory and selected source evidence determine namespace coverage. For
`PF2E.NPC.Abilities.Glossary.*`, distinguish reusable definitions from labels,
one-off rule-element strings and UI text. Establish exact source identity,
prepared content and attributable references. Only traits and existing record
destinations retain their established Similar capabilities; new vocabulary does
not automatically require vectors. Preserve absent definitions and source removal
behavior. Brief language speaker context and missing weapon-group prose use a
small approved, versioned subset with content-specific notices and an explicit
maintainer update path.

Foundry otherTags, rarity and Atlas authored tags retain separate semantics.
Localization alone supplies neither a trait implication graph nor executable
mechanics. Bibliographic citations or external rules destinations require their
own verified source evidence.

## Rules references within prose

The user flagged Create Demiplane's Gravity feature: AoN links to useful
[gravity rules](https://2e.aonprd.com/Rules.aspx?ID=3017&Redirected=1), while Atlas
has no destination at that mention. This is separate from authored trait
membership and from following an explicit UUID/localization reference.

Historical research against PF2e6.12.4 at `4cbdaa37` found Gravity as a plain
feature heading, with Mythic and Teleportation as the ritual's structured traits.
The English catalog had a Subjective Gravity label without a matching definition,
and the journal pack lacked the gravity-rules section. Recheck those observations
at the current pin; they do not prove current source coverage or a dropped link.

Review localization mappings, rule journals and explicit authored links. State
which destinations are supported directly by Foundry and which need a separately
reviewed external mapping or rules source. Discuss presentation and association
policy before implementation. Do not auto-link arbitrary prose from label matches.

## Acceptance

- Select a bounded namespace/source gap and define the exact supported identity.
- Keep locale resolution and source loading ingest-owned; do not persist or
  expose the entire localization catalog as product content.
- Reuse prepared reading, generation checks, bounded retrieval and exact owner
  navigation, without fabricated Foundry records or transitional facades.
- Validate source-free lookup, missing content, attribution, reference resolution
  and any Similar contract introduced for the new namespace.
- Preserve authored trait filters, saved state and the delivered trait behavior.

Description permission/attribution decisions remain in the separate
[content-rights item](trait-description-content-rights.md). Trait-driven encounter
mechanics remain in [trait runtime effects](rust-encounter-trait-runtime-effects.md).
Neither is implied by adding contextual glossary references.
