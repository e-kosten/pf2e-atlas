# Foundry inline content fidelity

Status: proposed
Priority: before extending supported inline mechanics
Owner: unassigned
Last reviewed: 2026-10-23

Extend the upstream-backed comparison of Foundry rich-text syntax when changing
the source pin or supporting additional content. ADR 0044 already established
shared parsing and prepared HTML; this is follow-through on remaining differences,
not a replacement content model or a claim that the existing comparison is absent.

## Research checklist

- Compare the pinned PF2e enricher registrations and handlers with Atlas's shared
  parser: Check, Damage, Template, Localize, inline act commands, UUID references,
  embeds and inline rolls. Distinguish PF2e implementations from Foundry core
  behavior for which runtime evidence may be unavailable.
- Check parameter parsing, nested delimiters, aliases, defaults, labels,
  localization, visibility and malformed-input handling. Classify differences as
  supported, intentionally unsupported or lacking evidence.
- Separate literal source values from expressions requiring actor, item, roll or
  user context. Retain unsupported syntax and diagnostics without inventing an
  interpretation from display text.
- Use representative corpus examples and adversarial fixtures for accepted
  corrections. Record the exact source revision; older 6.12.4 findings are
  historical evidence, not proof of parity with the current pin.

## Remaining comparison scope

Atlas's reading composition now expands supported paragraph-isolated inline Item
inclusions from checked prepared fields, with bounded traversal and exact source
attribution. Stored preparation keeps the embed marker and occurrence; read-time
composition supplies the included prose. The Fighter class feature and reaction
remain distinct records and must never be deduplicated by visible name.

The broader comparison remains open: revisit unobserved options, non-Item inserts,
other insertion positions, localization and contextual expressions against the
pinned upstream behavior before extending support. Existing unsupported or bounded
cases retain a local explanation and the checked destination when available.
Do not infer complete Foundry runtime parity from the supported corpus examples.
See [the read-time composition boundary](../../architecture/decisions/0044-shared-source-content-interpretation.md).

English `@Localize` preparation composes the manifest-declared catalogs, including
`action-en.json`, `re-en.json` and `kingmaker-en.json` in the PF2e 8.6.1 source.
Catalog composition is covered separately from translating the product or
ingesting translation-module overlays. Remaining localization comparison covers
unobserved options, malformed input and runtime-specific behavior, rather than
the previously omitted English catalog files.

PF2e action-font spans recognize lowercase `f` and `r` as aliases of `F` and `R`,
matching the pinned font. Preparation emits the existing free-action/reaction
attributes and readable labels; unknown values retain their diagnostics.

Literal typed damage and supported mixed-type pools have readable preparation
and source-checked reference dice controls. Actor/item expressions, nested damage
flavors and unrecognized options remain outside that execution subset. Known
traits/domains/roll options stay metadata beside literal base-formula rolls. Authored
formulas/options remain evidence; this does not establish full Foundry roll parity.

## Completion and boundaries

Produce a bounded discrepancy report and regression fixtures for any accepted
parser changes. Source DTO completeness alone does not establish markup fidelity.
Use the current pin in `dev-tools/source-contracts/source-pin.json`.

Pure parsing and preparation belong to atlas-record. Ingest supplies localization
and reference-resolution context. Preserve markup in checked DTOs and derive
sanitized HTML and interaction facts; do not restore a persisted RichDocument
tree. Gameplay execution requires separate, explicitly scoped runtime behavior.

See [shared content interpretation](../../architecture/decisions/0044-shared-source-content-interpretation.md)
and [Foundry-inspired product actions](foundry-inspired-product-actions.md).
