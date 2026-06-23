# Rust Encounter Saved-List Import

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-23

## Problem

Saved lists are useful prep artifacts, but runnable encounters need participant instances rather than set-like saved-list items. The v1 encounter runner intentionally keeps saved-list import out of scope so encounter CRUD, participant state, and runner workflows can land first.

## Desired Outcome

Add an app-service workflow that creates a runnable encounter from a saved list.

The design should answer:

- whether unresolved saved-list items are skipped with an explicit report or imported as unresolved participant snapshots;
- how saved-list order maps to encounter participant order;
- whether the import UI allows quantity expansion before creation or starts with one participant per saved-list row;
- how duplicate records are named and numbered when imported;
- whether a CLI creation/import command should call the same app-service workflow.

## Constraints

- Keep saved lists set-like; do not add duplicate item semantics to saved lists to support encounters.
- Keep encounter participants instance-based with their own `participant_key`.
- Do not bypass app-service record-kind validation.
- Preserve unresolved-item behavior explicitly rather than silently dropping rows.

## Related

- [Runnable encounters design](../../../scratch/plans/2026-06-22-runnable-encounters-design.md)
- [Rust local-state import and export](./rust-local-state-import-export.md)
