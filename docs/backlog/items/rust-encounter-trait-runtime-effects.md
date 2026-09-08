# Rust Encounter Trait Runtime Effects

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-25

## Problem

Encounter runtime automation currently starts from participant conditions and variants. Some PF2E runtime behavior instead comes from record traits or source-applied trait-like effects. The most immediate example is the `minion` trait, which affects action economy but is not a condition and should not be modeled as one.

If trait behavior is added as a one-off beside condition handling, encounter runtime projection will split into separate condition, variant, and trait paths with duplicated precedence rules and UI explanation behavior.

## Desired Outcome

Extend the encounter runtime mechanics pipeline so traits and conditions feed a shared runtime-effect application layer.

The implementation should:

- expose canonical record traits as typed mechanics facts, not presentation badges that app-service or the browser must scrape;
- introduce a normalized runtime-effect source shape for conditions, record traits, participant-applied effects, and future encounter/global effects;
- keep conditions as one source of runtime effects rather than the only source;
- route trait effects through the same action-budget, speed, capability, note, and explanation machinery created for condition automation;
- implement `minion` as the first trait-backed runtime effect, including action-budget projection and an explanatory source note;
- make source labels explicit, such as `Minion trait`, so the UI can explain why a runtime surface changed;
- define precedence between set/override effects and additive/subtractive effects before combining `minion` with effects such as slowed or quickened;
- avoid parsing rich presentation text, raw Foundry JSON, or frontend-rendered badges to discover traits.

## Constraints

- Do not persist derived trait effects in local-state rows.
- Do not make trait automation depend on the current static `RecordPresentationDocument` shape.
- Do not build a general PF2E rules engine in this slice.
- Do not force all future trait-like behavior to come from the base record; participant-applied effects must remain possible for cases such as summoned creatures or source-specific minion behavior.
- Avoid expanding runtime presentation sidecars before the broader record/mechanics presentation unification question is addressed.

## Related

- [Rust encounter runtime mechanics surfaces](./rust-encounter-runtime-mechanics-surfaces.md)
- [Rust encounter condition mechanics](./rust-encounter-condition-mechanics.md)
- [Rust record presentation mechanics unification](../history/items/rust-record-presentation-mechanics-unification.md)
