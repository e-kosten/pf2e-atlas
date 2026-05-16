# Backlog Items

Create one file per substantial backlog item when the item needs durable context beyond a short note in [`../backlog.md`](../backlog.md).

## When To Create An Item File

Create `items/<slug>.md` when the item:

- spans multiple subsystems
- has important constraints or tradeoffs
- needs links to related architecture or design docs
- is likely to survive across multiple work sessions

Keep small notes in the top-level backlog index instead of creating a file per idea.

## Recommended Template

```md
# Item Title

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-04-20

## Problem
Short description of what is wrong or missing.

## Desired Outcome
What done looks like from the user or repo perspective.

## Constraints
Important boundaries, risks, or explicit non-goals.

## Notes
Extra implementation notes, links, or related context.

## Related
- [Relevant architecture doc](../../architecture/node/tui.md)
- [Temporary working plan](../../scratch/plans/example-plan.md)
```

## File Naming

- use short descriptive slugs such as `ontology-browser-naming.md`
- prefer one durable topic per file
- rename only when the item meaning changes materially
