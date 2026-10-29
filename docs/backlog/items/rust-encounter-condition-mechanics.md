# Rust Encounter Condition Mechanics

Status: in_progress
Priority: next
Owner: Codex
Last reviewed: 2026-10-28

## Desired outcome

Keep standard-condition rules consistent across displayed facts, strikes, damage,
spellcasting, action availability and HP events. Atlas remains a GM table aid:
optional dice, direct table outcomes, independent condition sources and concise
contextual reminders support adjudication without a mandatory action ledger or
simulation of movement, equipment or behavioral decisions.

The current correction batch covers maintained-child and observer immunity,
source-owned Petrified mortality, durable ability-state identities, explicit
detection alongside Invisible, source range defaults, raw attribute preservation,
attribute-based damage penalties, and the reviewed condition editing/recovery
flows. Source-backed Spend now honors the same known prohibitions as its display,
without reallocating stock or restricting manual corrections. Successful editor
closure waits for the controls to re-enable, preserving native focus return.

One rules-owned RegainPlan now supplies allocation, numeric Stunned loss,
Quickened restrictions and typed missing facts; presentation adapts those facts.
Actual regain and validated explicit Stunned recovery share one consumption
reducer. With product agreement, unused source-answer/Cantrip plumbing and raw
recovery total/natural input were removed. Roll, table degrees, physical damage
totals, historical outcomes and manual Stunned reconciliation remain supported.

The shared evaluation and event boundaries are described in
[ADR0055](../../architecture/decisions/0055-encounter-condition-evaluation.md).

## Open follow-through

- Complete PR review of the current correction batch.
- Finish validation of the reopened effective-source repair in PR142. HP, recovery,
  before/after condition transitions, own-turn Stunned, targeting, senses and rest
  now use shared effective facts. Known inherited Dying/Wounded/Doomed participate
  without requiring an attachment or source-editing workflow. Saved selectors and
  historical results remain deliberately distinct. Supported Go Dark and Smoke
  Exhalation declarations have public service regressions for Hidden/Concealed.
- Verify the agreed uncertainty presentation in the updated preview, then complete
  final gates and review. Unknown relevant facts explain the affected calculation;
  independent known facts and manual correction remain usable. No new adjudication
  form or arbitrary Foundry effect runtime is introduced.

## Boundaries

Keep generated condition declarations immutable, interpret checked source facts in
app-service, and leave atomic saved-state changes with local-state. Preserve scoped
causes, independent sources, persistent precedence, table degrees, correction
versus application, and retry receipts. Do not replace these with a general Foundry
runtime or infer rules from names and prose. Cursebound remains separately deferred.

## Related

- [Cursebound encounter mechanics](./encounter-cursebound.md)
- [Rust encounter actor context effects](./rust-encounter-actor-context-effects.md)
- [Rust encounter runtime mechanics surfaces](./rust-encounter-runtime-mechanics-surfaces.md)
