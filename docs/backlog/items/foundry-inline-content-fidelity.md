# Foundry inline content fidelity

Status: proposed
Priority: before extending supported inline mechanics
Owner: unassigned
Last reviewed: 2026-10-27

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
and source-checked reference dice controls. Unsupported actor/item expressions,
nested damage flavors and unrecognized options remain authored data. Known
traits/domains/roll options stay metadata beside literal base-formula rolls. Authored
formulas/options remain evidence; this does not establish full Foundry roll parity.

The registered PF2e `/act` commands now use source-backed English labels, trailing
action costs, variants and verified unique action references. References back to
the containing action remain readable without reopening that action. The action
registry's ASCII case-only naming differences are handled separately from exact
authored UUID/name references. These controls do not execute actor actions.

Literal `/r`, `/gmr` and `/br` commands share source-checked transient reference
rolling, retaining their original modes and captions. Atlas has no Foundry chat
recipient routing or persisted roll history. Unsupported dice syntax, flavored
inline formulas remain outside this subset; supported contextual formulas use
the checked values described below.
Checks and area templates render as plain text, with complete labels, opposing
DCs, basic/secret qualifiers, statistic choices and explicit area widths. Source
options, custom labels and DC visibility are retained without generic parameter
popups.

Contextual reading resolves the bounded arithmetic subset from selected spell rank,
checked item/owning-actor levels and addressed encounter context. Missing values
render symbolic prose; generic badge defaults remain unknown. Other actor data,
kingdom state, unsupported formulas, flavored command rolls and runtime template
geometry remain outside this subset. See ADR0044 and the artifact contract.

## Potential follow-ups

These are candidates for later investigation, not agreed implementation slices or
confirmed defects in every record. Start with a ranked inventory of remaining
visible raw syntax or obscured rules text in admitted product records. For each
candidate, record the pinned source identity, current rendered output, proposed
readable output, available context and benefit to a GM. Prioritize reading gaps
and deterministic values already present in checked records. Recheck these
PF2e 8.6.1 examples against the current source pin before implementation:

- **Additional item and actor fields.** Drake Rifle (Fire) references
  `@item.system.damage.dice` and `@item.system.damage.damageType` for persistent
  and splash damage. Evaluate binding specific checked fields, including any
  relevant runtime adjustments, rather than broadly accepting arbitrary paths.
  Preserve its useful authored labels when a value cannot be established.
- **Choices made during play.** Elemental Attack determines damage type through
  `@item.flags.system.rulesSelections.elementalAttack.damage`. Establish where the
  actual selected element would come from before resolving it; the generic
  compendium record cannot establish that choice. Keep readable symbolic or
  authored wording when the choice is unavailable.
- **Kingdom context.** Relocate Capital uses `@kingdom.control.dc + 5`.
  Numeric resolution would require an authoritative kingdom value and a concrete
  campaign workflow. Assess whether the existing symbolic rule is sufficient
  before adding kingdom state or context inputs.
- **Flavored inline rolls.** Tidal Crash uses a flavored `/r` expression for
  2 bludgeoning splash damage. Investigate sharing the existing damage
  interpretation with slash rolls while preserving nested annotations and
  captions. Its authored label is already readable, so record the practical
  benefit rather than assuming every unsupported roll needs a control.
- **Advanced dice and nested damage expressions.** Inventory concrete occurrences
  of unsupported modifiers, pools and nested flavors before expanding the bounded
  arithmetic subset. This is a syntax boundary to investigate, not an established
  list of important broken records. Verify semantics and regression examples
  before making additional forms executable.
- **Additional embed forms and positions.** Evaluate non-Item targets, additional
  options and inserts mixed into surrounding prose only when corpus examples
  demonstrate a reading benefit. Define selected fields, attribution, visibility
  and valid HTML composition. Preserve checked navigation and the local
  explanation when expansion is unavailable; do not insert block content into a
  sentence or imply general Foundry document rendering.

These candidates do not require full Foundry runtime parity. Existing choices
remain: checks and area templates are plain text, action references do not execute
actor actions, and reference rolls have transient results without saved history.

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
