# Reusable encounter party definitions

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-10-13

Users currently re-enter each player character when constructing an encounter.
Provide a place to define a party once and add its members to later encounters.
Discuss this workflow alongside repeated creature addition and compact rosters.

Design the minimum useful party/member fields and their availability before
implementation. Decide whether members are manual definitions, reference-backed
participants or both, and how quantities, duplicates, missing references and
member selection work. Distinguish reusable definitions from live encounter state:
agree whether additions copy saved defaults and how later edits affect either
surface. Do not silently share HP, conditions or initiative across encounters.

Keep durable user data separate from the generated reference artifact. Any storage
and workflow design must follow atlas-local-state and app-service ownership; do
not introduce a character-sheet or Foundry-world model implicitly. Review whether
existing saved-list import primitives can help without changing lists into an
unreviewed party model. UI and CLI should use the same product workflow if both
expose it. No persistence schema or UI implementation is selected by this item.

Related: [encounter usability](rust-web-encounter-runner-polish.md),
[saved-list import](rust-encounter-saved-list-import.md),
[local-state ownership](../../architecture/decisions/0030-local-state-database.md).
