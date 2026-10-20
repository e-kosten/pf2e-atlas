# Reusable encounter party definitions

Status: in_progress
Priority: now
Owner: encounter-parties task
Last reviewed: 2026-10-19

Agreed first version: multiple named parties with ordered PC names only. Manage
parties from Encounters and add all or selected members together. No HP, initiative,
conditions, reference-backed definitions or Foundry import. Additions copy names
into independent manual encounter PCs; party edits/deletion do not alter existing
encounters. Duplicate names remain independent members.

Implementation: saved-domain parties/member schema and runtime migration,
revision-guarded aggregate edits, atomic selected-member encounter addition,
generated app contracts and thin web transport. Existing encounter mutation owner
handles the durable write and independent refresh. Awaiting early functional UX
review, then final Rust/frontend/browser validation and PR review.

Future Foundry import is a separate capability; this version introduces no import
format or character-sheet/world model.

Related: [encounter usability](rust-web-encounter-runner-polish.md),
[local-state ownership](../../architecture/decisions/0030-local-state-database.md).
