# Reusable encounter party definitions

Status: in_progress
Priority: now
Owner: encounter-parties task
Last reviewed: 2026-10-20

Agreed first version: multiple named parties with ordered PC names only. Manage
parties from Encounters and add a whole party from the encounter toolbar menu.
No HP, initiative, conditions, reference-backed definitions or Foundry import. Additions copy names
into independent manual encounter PCs; party edits/deletion do not alter existing
encounters. Duplicate names remain independent members.

Implementation: saved-domain parties/member schema and runtime migration,
revision-guarded aggregate edits, atomic whole-party encounter addition,
generated app contracts and thin web transport. Existing encounter mutation owner
handles the durable write and independent refresh. The user accepted the simplified
whole-party menu and separate Active/Archived views. Local implementation and
validation are complete in [PR88](https://github.com/e-kosten/pf2e-atlas/pull/88);
PR review and hosted CI remain separate from the passing local gates.

Future Foundry import is a separate capability; this version introduces no import
format or character-sheet/world model.

Related: [encounter usability](rust-web-encounter-runner-polish.md),
[local-state ownership](../../architecture/decisions/0030-local-state-database.md).
