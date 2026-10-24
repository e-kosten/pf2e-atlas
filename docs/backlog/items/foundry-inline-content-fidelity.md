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

## Concrete embed gap

At pin `28d9d156f252d084371d994dc359509b49eadf18`, the Fighter page
`journals:kzxu2dI7tFxv6Ix6` / page `HZbrRRVzW7w17L2W` authors a Reactive
Strike heading, an inline `@Embed` of class feature `hmShTfPOcTaKgbf4`, an
`@UUID` action link with reaction glyph, and an inline `@Embed` of action
`KAVf7AmRnbCAHrkT`. Atlas preserves both resolved embed destinations as links
and emits `EmbedNotExpanded`; the resulting four adjacent names do not provide
the intended embedded feature/action prose. This is distinct from inline
expression evaluation. Do not deduplicate by visible name: the class feature
and action are different records, and an embed and a reference have different
roles. Any future expansion must define bounded depth/work, cycle handling,
source attribution, generation ownership and the relationship between an inline
body and its exact native full-record destination. The Ant presentation work
records this gap without introducing recursive expansion or changing preparation.

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
