# ADR 0003: Lint-Enforced Derived-Tag Boundaries

- Status: Accepted
- Date: 2026-04-19

## Context

`src/tags/` is a large subsystem with many leaf modules:

- authored ontology and rules
- runtime derivation helpers
- discovery and evaluation code
- migration and review workflows
- CLI entrypoints

Without enforcement, other parts of the repo can easily couple themselves to arbitrary leaf modules, bypass shared facades, or reimplement parsing and workflow boundaries. The project already uses ESLint restrictions to prevent that drift in several places.

## Decision

Use lint rules to make the intended derived-tag and editorial boundaries mandatory once they are stable enough to be the preferred path.

Current examples include:

- outside `src/tags`, import derived-tag functionality through `src/tags/index.js` or another approved facade instead of tag leaf modules
- tag CLI scope parsing goes through `src/tags/cli/search-scope-args.ts`
- TUI composition for the tag workbench flows through `src/tui/app-services.ts` instead of direct migration-service imports from arbitrary feature code

## Consequences

- Architectural boundaries are reviewable and mechanically enforced instead of relying on convention alone.
- Refactors inside `src/tags/` stay cheaper because external callers depend on smaller entry surfaces.
- Adding a new shared editorial abstraction is not finished until the enforcement story is considered.
- If a rule becomes too restrictive for a legitimate new workflow, the right response is to adjust the boundary deliberately, not to bypass it ad hoc.
