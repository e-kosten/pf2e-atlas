# Rust Web Record Detail Polish

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-24

## Problem

The web UI renders semantic family facts and prepared HTML through shared Ant
document layouts. Spells, rituals, equipment, character options, journals, roll
tables, creatures and hazards have dedicated presentation. Remaining coverage
gaps concern source fields and consumer projections identified below, rather
than another general layout pass.

Further detail work should improve source-backed semantic contracts only when a
consumer needs another fact, and reuse shared family presentation across detail,
preview, comparison and encounters.

## Current Baseline

Reference, preview, comparison and encounter surfaces reuse semantic family
components and selected prepared HTML. Exact owner addresses govern navigation,
Related availability and bounded owned reading. Similar discovery uses the shared
search browser with explicit seed/filter state. These contracts are documented in
[ADR 0047](../../architecture/decisions/0047-semantic-record-presentation.md) and
[frontend guidelines](../../../web/atlas-ui/docs/frontend-guidelines.md).

Spell rank preview, casting resources, authored tables and dice controls use
shared owners. Document widths, padding, identity, source attribution, inline
reference reading and creature/encounter ordering follow the frontend guidelines.
Further changes must preserve useful content, availability states and source
meaning across hosts.

## Desired Outcome

Address the specific source-backed coverage gaps below using the existing family
layouts and navigation. Additional open consumer work includes:

- meaningful casting-entry context for owned spells, resolved from their actual
  source owner through the existing service rather than displaying a raw local
  Foundry identifier or resolving it in the browser.
- [alternate spell-form preview](rust-spell-rank-preview.md), which remains deferred
  separately from the implemented rank preview.

## Constraints

- Creature spell rosters already use cast-rank presentation and embedded previews;
  standalone and embedded spell rank controls share the same reference model.
  Reproduce any new issue against the current renderer before redesigning it.

- Extend narrow app-service family facts when current consumer data is insufficient;
  keep checked source and prepared HTML authoritative under ADR 0047.
- Do not duplicate rich document parsing or record-kind semantics in TypeScript.
- Keep browser-specific layout code separate from shared Rust presentation contracts.
- Reuse family components and shared Ant primitives rather than duplicating
  layouts or creating visually inconsistent feature-local designs.

## Separately scoped source-modeling findings

The equipment DTO audit at PF2e source pin
`28d9d156f252d084371d994dc359509b49eadf18` found weapon `bonusDamage` and `staff`,
and kit `equippedBulk`, outside the generated typed members. Review their source
meaning and typed admission before proposing a consumer projection. They are
separate from presenting already typed weapon dice, armor facts or container
capacity; do not read retained extras or interpret prose in the browser to fill
these gaps. Preserve complete authored rules while this support is deferred.

Separate reference-scan candidates already have typed source support: weapon
attack bonus (especially alchemical bombs), item grade on two commercial weapons,
ammunition's alternate crafting kinds and apex attribute. Review their source
meaning and useful presentation individually; item grade is distinct from material
grade, and base damage plus runes is not a complete calculated attack. Integrated
shield attack/rune details likewise require a qualified presentation rather than
an inferred effective attack. These are explicit remaining coverage decisions,
not claims that every nested equipment DTO member is now displayed.

## Separately scoped creature coverage findings

The creature audit at the same source pin distinguishes supplied facts from
calculated character/familiar support. NPC movement qualifications, supplied
character languages and owned strike range facts can use existing typed source
support. Further bounded projection/content work remains:

- Hazard `details.isComplex` is structured, but `details.routine` is rich prose,
  including any action allotment. The generic creature action/reaction defaults
  are not a source-backed hazard budget. A structured hazard allotment and its
  encounter calculation policy require separate modeling; retain the complete
  routine and activation rules in the meantime.
- NPC `details.blurb` already has a plain attributed Notes selection alongside
  permitted public/private notes (1,620 nonempty records). Review its labeling
  and overlap before adding a second summary; no new source selection is needed.
  Do not inject raw HTML or remove repeated prose by string matching.
- `NPCSpecialSkillSource.predicate` is retained but not represented in conditional
  skill facts. Preserve supplied label/modifier and unavailable/blank qualifiers;
  define an honest readable predicate projection before interpreting it or
  evaluating whether a bonus applies.
- Owned effect badges (for example Prismhydra's Hydra Heads counter) are retained
  but lack a presentation projection. Distinguish an authored counter, formula
  and runtime reevaluation before exposing values as current encounter state.
- Character build/proficiency inputs and familiar/master references do not supply
  complete effective NPC-style combat statistics. Treat character identity and
  calculated sheet support as separate projections; preserve biography visibility.
- Legacy retained action requirements/trigger fields and casting-entry resource
  snapshots need source-policy review before adding generic rows or workflows.
  The observed owned action trigger/requirements objects have empty values; their
  presence alone does not establish omitted rules prose.

The audit traces root groups and immediate owned melee/action/feat/effect/lore,
casting, spell and physical-item families. It is not certification that every
nested DTO member is rendered. Rule elements, source snapshots and platform
configuration are not automatically user-facing facts or permission to extend
encounter mechanics. Unresolved glossary attack effects remain under the
[localization-backed rules work](rust-localization-backed-rules-terms.md); exact
unique owned-slug links do not establish a general glossary resolver.

## Related

- [Architecture overview](../../architecture/overview.md)
- [ADR 0029: Local web app boundary](../../architecture/decisions/0029-local-web-app-boundary.md)
