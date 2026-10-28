# Spell Casting Variant Preview

Status: deferred
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-27

## Missing capability

The abandoned record-refactor pathway supported selecting a casting variant
(a Foundry spell form/overlay) together with a casting rank and reading the
resulting effective spell. Current readers support rank preview and readable
variant differences, but cannot select a variant as the effective spell.

For example, Heal can offer its one-action touch, two-action living/undead target
and three-action emanation variants. Selecting the living-target variant at rank
3 should present that variant's effective casting, targeting and healing facts
together. Listing its changes separately does not provide the same workflow.

This capability remains deferred backlog work. Tracking it does not authorize
implementation or adoption of the abandoned implementation. The earlier working
interaction is reference evidence in
[PR #7's spell reader](https://github.com/e-kosten/pf2e-atlas/blob/66d47fa9608c5f8c0b77e44efc85faade78d2b89/web/atlas-ui/src/shared/records/SpellRecordSurface.test.tsx#L573-L644).

## Remaining work

Add interactive selection of persisted alternate spell forms using the same
checked source and shared Rust presentation as rank preview. Form selection must
respect Foundry overlay/layer ordering, preserve embedded customizations and
compose with the selected rank. Agree the interaction before implementation.
Forms remain readable reference information today; they are not executed by the
rank selector. Do not infer a form from its title or parse prose into mechanics.

## Existing boundary

Rank preview supports fixed mechanics/trait patches and interval damage/area,
with automatic contextual rank for owned spells where the source is valid.
Changed description patches and patches that modify heightening rules report an
unsupported preview rather than presenting incomplete changes as effective facts.
If supporting these, reuse prepared content and reference resolution; do not
introduce an alternate runtime HTML/parser pipeline.

## Acceptance

- Establish persisted-versus-derived semantics and form/heightening ordering
  from the pinned source, including clears and repeated component identities.
- Preserve exact customized embedded copies and original-record navigation.
- Compose the selected variant and rank in one effective presentation, including
  supported prepared content, rather than changing only the displayed mechanics.
- Preserve a valid selected rank when switching variants; make any required
  rank change explicit and keep unavailable combinations honest.
- Return supported effective facts and explicit unsupported outcomes consistently
  across CLI, reference views and encounters.
- Keep preview read-only: selection does not mutate source, cast the spell or
  expend a spell slot or other resource.

## Related

- [Semantic record presentation](../../architecture/decisions/0047-semantic-record-presentation.md)
- [Rust web record detail polish](rust-web-record-detail-polish.md)
