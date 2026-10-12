# Active worktree build footprint

Status: proposed

## Intent

Measure and reduce active-worktree disk use after the mutable-workflow follow-up
is reviewed. Worktree retirement and preservation guidance are a separate change
on `chore/worktree-lifecycle`. The user explicitly chose to finish the active
implementation before changing compiler settings or build tooling.

## Investigation

- Establish one clean-build baseline at a pinned revision, toolchain, dependency
  lock, feature set, and frontend build. Separate dependency archives, linked
  executables, debug information, compiler metadata, incremental state, and
  generated data; report actual disk usage alongside logical file sizes.
- Compare full, limited, and line-table debug information for development and
  tests. Keep useful backtraces and measure build time and correctness, rather
  than assuming the smallest output is the best developer default.
- Measure incremental compilation across a representative edit/rebuild cycle.
  Distinguish its speed benefit from accumulated obsolete configurations.
- Inspect generated Foundry-model and native inference contributions. Measure
  code and debug sections independently before changing generated types or
  dependency boundaries.
- Review integration-test executable grouping only where repeated linking has
  measurable cost; retain behavioral coverage and useful test isolation.
- Evaluate a bounded compiler cache without sharing checkout-specific Cargo
  targets or reusing test binaries with embedded fixture paths.
- Correct new-branch pre-push path detection: docs-only branches currently trigger
  full application verification when the remote ref does not yet exist. Preserve
  conservative validation for unknown or executable changes and test the hook.

## Acceptance

Present measured alternatives and their tradeoffs before implementation. Select
consistent defaults, document retention/budget policy, preserve normal debugging
and all required checks, and verify clean/repeated builds from linked worktrees.
Do not solve disk use by silently skipping validation or discarding user state.
