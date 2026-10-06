# Rust Localization-Backed Rules Terms

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-10-05

## Problem

PF2e source localization contains reusable rules vocabulary such as trait labels, trait descriptions, and NPC ability glossary entries. Atlas now resolves `@Localize[...]` macros during ingest and preserves localization key context inside `RichDocument`, but it does not persist selected localization-backed vocabulary as canonical product concepts.

That means web and future TUI surfaces can render localized text correctly, but they still cannot offer AoN-style trait or glossary hover cards, detail pages, or explicit relationships from records to reusable rules terms.

## Desired Outcome

Design and implement a canonical rules-term layer for high-value localization-backed game concepts.

The implementation should support:

- canonical term identity such as `trait:halfling` or `npc-ability-glossary:negative-healing`
- term kind, slug, label, localization keys, and parsed `RichDocument` description content
- relationships from normal records to terms through trait membership, localized macros, rule-option labels, or authored references
- web/TUI-ready lookup and hover-card data without making runtime consumers resolve raw localization keys

## Source discovery prerequisite

The [source-contract generation and catalog experiment](./rust-source-contract-generation.md)
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

Keep `record_traits` as the filterable trait facet table. `rules_terms` should answer "what is this trait or glossary entry?", while `record_traits` should answer "which records have this trait?"

## Constraints

- Preserve `RichDocument` macro context for `@Localize[...]`; do not replace localized macro provenance with plain text.
- Keep generic localization loading ingest-owned.
- Avoid turning broad localization catalogs into runtime product data.
- Add validation for term identity, content coverage, and occurrence references if artifact tables are added.

## Related

- [Rust artifact contract](../../architecture/artifact-contract.md)
- [Rust artifact JSON content model review](./rust-artifact-json-content-model-review.md)
