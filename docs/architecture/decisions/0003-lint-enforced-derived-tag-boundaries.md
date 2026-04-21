# ADR 0003: Lint-Enforced Derived-Tag Boundaries

- Status: Accepted
- Date: 2026-04-19

## Context

`src/tags/` is a large subsystem with many leaf modules:

- authored ontology and rules
- runtime publication, derivation, matcher, and compatibility helpers
- review registries and reviewed discovery state
- discovery and evaluation code
- editorial state, session, writeback, and review UI workflows
- grouped CLI entrypoints

Without enforcement, other parts of the repo can easily couple themselves to arbitrary leaf modules, bypass shared facades, or reimplement parsing and workflow boundaries. The project already uses ESLint restrictions to prevent that drift in several places.

## Decision

Use lint rules to make the intended derived-tag and editorial boundaries mandatory once they are stable enough to be the preferred path.

Current examples include:

- outside `src/tags`, import derived-tag functionality through `src/tags/runtime.js`, `src/tags/editorial.js`, or `src/tags/editorial-ui.js` instead of tag leaf modules
- durable reviewed discovery state lives under `src/tags/reviews/`, not ad hoc discovery-owned scratch modules
- editorial execution is split across `src/tags/editorial/state/`, `sessions/`, `writeback/`, and `ui/`
- tag CLI scope parsing goes through `src/tags/cli/shared/search-scope-args.ts`
- TUI composition for the tag workbench flows through `src/tui/app-services.ts` instead of direct editorial-service imports from arbitrary feature code
- non-tag callers should enter through the concern-specific top-level facades, while internal tag code should import the owning split directories directly instead of recreating compatibility re-export layers

## Consequences

- Architectural boundaries are reviewable and mechanically enforced instead of relying on convention alone.
- Refactors inside `src/tags/` stay cheaper because external callers depend on smaller entry surfaces.
- The split runtime, reviews, editorial, and CLI owners can keep evolving internally without reopening repo-wide imports every time a file moves.
- Adding a new shared editorial abstraction is not finished until the enforcement story is considered.
- If a rule becomes too restrictive for a legitimate new workflow, the right response is to adjust the boundary deliberately, not to bypass it ad hoc.
