# Actor Metrics Search Orchestration

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-20

## Problem

Actor metrics appear correctly in the search-semantics explorer, but the executable search path does not yet support turning those selections into real queries in a trustworthy way.

## Desired Outcome

Users should be able to select actor metrics in the search UI and have those constraints flow cleanly into real search execution, with labels and operators that remain understandable in both the explorer and the resulting query model.

## Constraints

- Keep MCP and TUI filter semantics aligned.
- Avoid special-case query paths that bypass the shared search/filter pipeline.
- Treat this as executable search behavior, not only explorer labeling.

## Related

- [Search architecture](../../architecture/search.md)
- [TUI architecture](../../architecture/tui.md)
