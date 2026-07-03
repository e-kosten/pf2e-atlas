# Block 1 Verification Note

Date: 2026-04-24
Worktree: `/tmp/pathfinder-mcp-worktrees/search-convergence-block1-unfinished`
Scope: Block 1 of `2026-04-24-remove-isunique-post-implementation-fix-pass-orchestration-plan.md` plus `2026-04-24-search-convergence-block-1-remediation-addendum.md`

This note records the required Block 1 spot-checks after the final remediation loop. The checks were performed by directly inspecting the live Ink frames exercised by the search-screen and terminal-app integration harnesses in this worktree, alongside a final green `npm run build` and `cd scripts && npm test`.

## Required Checks

- Centered mode picker on first entry and mode change
  Result: pass
  Notes: first entry uses the blanked centered prompt; later mode changes use the overlay centered prompt with the same shell and centered horizontal choice row.

- Browse/lookup sort visibility
  Result: pass
  Notes: browse and lookup expose sort rows in the workspace; ranked search does not expose a user sort row.

- Search query text modal shell
  Result: pass
  Notes: top-level query text and exclude text both route through the centered overlay prompt shell instead of the bottom inline text input.

- Inline flat root projection
  Result: pass
  Notes: the workspace shows the compact flat root projection inline by default and no longer prefixes each row with `Filter |`.

- `:` action flow and first-add grouping behavior
  Result: pass
  Notes: `:` uses the shared action rail on the structured editor, and first-add inserts directly into the visible root group instead of forcing an intermediate group/not picker.

- No intermediate background frame during chained prompt transitions
  Result: pass
  Notes: chained prompt flows stay within the prompt-session path and do not flash the underlying structured editor frame between steps.

- Centered numeric matcher preview/detail section
  Result: pass
  Notes: centered numeric matcher prompts for level editing show the preview/detail section inside the shared centered shell.

## Final Validation Pair

- `npm run build`
  Result: pass

- `cd scripts && npm test`
  Result: pass
  Summary: `104` test files passed, `1` skipped; `618` tests passed, `2` skipped.
