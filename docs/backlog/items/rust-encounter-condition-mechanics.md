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
flows. The shared evaluation and event boundaries are described in
[ADR0055](../../architecture/decisions/0055-encounter-condition-evaluation.md).

## Open follow-through

- Finish validation and PR review of the current correction batch.
- Audit effective source-owned Dying, Wounded and Doomed arithmetic in ordinary
  HP lifecycle events. Petrified parity does not establish parity for these valued
  conditions: some death-threshold and recovery calculations still read saved
  rows only. Separate effective mathematical facts from selectors that edit saved
  rows; source-owned conditions remain read-only. The audit has not established
  an affected pinned creature for each case.
- Evaluate a typed internal action-regain plan. Durable allocation currently
  consumes an action-budget presentation DTO, including its first displayed note.
  The rules owner should supply typed allocation facts and unavailable reasons,
  with presentation projected separately.
- Decide whether raw recovery-roll entry, manual Stunned consumption, source-specific
  action judgments and historical-only recovery variants justify their remaining
  write surfaces. Code reachability alone is not usage evidence. Any removal needs
  a concrete end state, consumer/receipt audit and product agreement.

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
