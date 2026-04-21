# Tagging Tooling Reorganization

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-20

## Problem

The root npm command surface still exposes too many editorial/tagging utilities directly, and some older naming still implies that the TUI is primarily the derived-tag workbench rather than the broader PF2E terminal app.

## Desired Outcome

Reorganize editorial tooling so that:

- root-level developer commands stay focused and readable
- deeper tagging utilities remain discoverable without cluttering the main surface
- app naming better reflects the current product shape

## Constraints

- Do not make editorial tooling hard to discover for active maintainers.
- Preserve documented or scripted workflows that matter in current use.
- Keep the repo’s architecture and developer ergonomics aligned.

## Related

- [Editorial architecture](../../architecture/editorial.md)
- [Architecture overview](../../architecture/overview.md)
