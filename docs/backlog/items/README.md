# Backlog Items

Create one file per substantial backlog item when the item needs durable context beyond a short note in [`../backlog.md`](../backlog.md).

## When To Create An Item File

Create `items/<slug>.md` when the item:

- spans multiple subsystems
- has important constraints or tradeoffs
- needs links to related architecture or design docs
- is likely to survive across multiple work sessions

Keep small notes in the top-level backlog index instead of creating a file per idea.

## Item Contents

Use the status vocabulary from the backlog, plus priority, owner and review date
where useful. Describe the problem, desired outcome, material constraints and
relevant evidence. Choose sections to fit the item; a short task may need only
a few paragraphs.

Link current architecture and related work. Keep implementation progress in its
working plan and dated findings in research or history. Revise the item around
what remains open as its baseline changes.

## File Naming

- use short descriptive slugs such as `ontology-browser-naming.md`
- prefer one durable topic per file
- rename only when the item meaning changes materially
