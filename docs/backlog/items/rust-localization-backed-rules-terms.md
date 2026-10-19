# Rust Localization-Backed Rules Terms

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-10-13

## Problem

PF2e source localization contains reusable rules vocabulary such as trait labels,
trait descriptions and NPC ability glossary entries. Source-backed preparation
resolves `@Localize[...]`; the artifact context retains only verified trait labels
used by selected vocabulary. Labels do not create canonical product terms,
description hover cards or executable mechanics.

That means web and future TUI surfaces can render localized text correctly, but they still cannot offer AoN-style trait or glossary hover cards, detail pages, or explicit relationships from records to reusable rules terms.

## Desired Outcome

Design and implement a canonical rules-term layer for high-value localization-backed game concepts.

The implementation should support:

- canonical term identity such as `trait:halfling` or `npc-ability-glossary:negative-healing`
- term kind, slug, label, localization keys and selected prepared HTML description content
- relationships from normal records to terms through trait membership, localized macros, rule-option labels, or authored references
- web/TUI-ready lookup and hover-card data without making runtime consumers resolve raw localization keys

## Source discovery prerequisite

The completed [source-contract generation and catalog work](../history/items/rust-source-contract-generation.md)
proposes extracting upstream trait and other-tag identifiers, authoritative labels,
localization keys, authored description text and catalog/family membership first.
That source-only catalog can be inspected independently of storage or app models.
Catalog extraction does not complete this item's product lookup, hover content or
record-to-term relationships.

Use the explicit upstream trait-description mappings, including parameterized
variants, rather than assuming every label key has a mechanically matching
description key. Preserve missing descriptions. Foundry otherTags can have
family-specific vocabularies and remain distinct from traits and Atlas authored
tags. Description prose does not itself supply structured implication edges or
executable mechanics. AoN-style citations/links require a separate source review.

## Follow-up: rules references within prose

The user flagged Create Demiplane's Gravity feature: AoN links to useful
[gravity rules](https://2e.aonprd.com/Rules.aspx?ID=3017&Redirected=1), while Atlas
offers no definition or navigation at that mention. Investigate this alongside
trait/glossary definitions, keeping actual record trait membership distinct from
rules concepts mentioned in prose.

An initial check of the pinned PF2e 6.12.4 export (source commit `4cbdaa37`)
found Gravity as a plain feature heading in Create Demiplane, with no authored
reference. The ritual's structured traits are Mythic and Teleportation. Its English
catalog has a Subjective Gravity label but no matching definition; the journal
pack did not contain the gravity-rules section. This is a source-coverage question,
not evidence that the renderer dropped an authored Gravity link.

Review the earlier trait/tag catalog extraction, explicit description mappings,
localization, rule journals and any authored references. Distinguish labels alone,
usable definitions and resolvable rule destinations, and verify the pinned source
before relying on newer Foundry content. Determine what can be supported directly
from Foundry and what would require a separately reviewed external mapping or
additional rules source. Discuss the presentation and association policy before
implementation; do not auto-link arbitrary prose from label matches or invent
relationships where source evidence is absent.

## Initial Scope

Start with high-confidence namespaces only:

- `PF2E.Trait*`
- `PF2E.TraitDescription*`
- `PF2E.NPC.Abilities.Glossary.*`

Do not persist every localization key as product content. UI strings, settings labels, implementation messages, and one-off rule-element labels should remain ordinary localization data unless a later review identifies durable game meaning.

## Possible Artifact Shape

Potential table families:

- `rules_terms`
- `rules_term_content`
- `record_term_occurrences`

Keep catalog-bound trait projections as the filter surface. A rules-term layer
should answer "what is this trait or glossary entry?" without replacing the
authored trait identifiers or duplicating record identity.

## Constraints

- Preserve relevant authored localization and source provenance; do not introduce a persisted document tree for term rendering.
- Keep generic localization loading ingest-owned.
- Avoid turning broad localization catalogs into runtime product data.
- Add validation for term identity, content coverage, and occurrence references if artifact tables are added.

## Related

- [Rust artifact contract](../../architecture/artifact-contract.md)
- [Rust artifact JSON content model review](../history/items/rust-artifact-json-content-model-review.md)
