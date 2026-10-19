# Spell Form Preview

Status: deferred
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-17

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
- Return supported effective facts and explicit unsupported outcomes consistently
  across CLI, reference views and encounters.
- Keep form preview read-only and separate from resource expenditure.

## Related

- [Semantic record presentation](../../architecture/decisions/0047-semantic-record-presentation.md)
- [Rust web record detail polish](rust-web-record-detail-polish.md)
