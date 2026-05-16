# Tagging Tooling Reorganization

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

Most of the original command-surface reorganization is already landed:

- the root npm surface is small and focused
- deeper editorial/tagging commands are grouped under the dedicated tag CLI
- the editorial architecture doc already describes that grouped layout as the current structure

The remaining inconsistency is naming and discoverability drift. Some command names and launcher copy still imply that the terminal app is primarily the derived-tag migration workbench instead of the broader PF2E terminal app.

## Desired Outcome

Finish the remaining naming and discoverability cleanup without reopening the already-landed command-surface reorganization.

That follow-through should:

- keep the slim root command surface and grouped tag CLI as-is
- align `tui` and workbench naming with the broader terminal app
- keep editorial/tagging utilities discoverable without making them look like the whole product surface

## Constraints

- Do not reopen the grouped tag CLI architecture just to relitigate the reorganization that already landed.
- Do not make editorial tooling hard to discover for active maintainers.
- Preserve documented or scripted workflows that matter in current use.
- Keep the repo’s architecture and developer ergonomics aligned.

## Related

- [Editorial architecture](../../architecture/node/editorial.md)
- [Architecture overview](../../architecture/overview.md)
