# Rust Web Filter UX Expansion

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-09

## Problem

The source-backed filter surface uses a maintained Ant query builder and the
backend catalog. A later UX pass can improve field priority, grouping, labels
and progressive disclosure without replacing its predicate semantics.

PF2e records expose many useful facets. Showing too many at once overwhelms the search workflow, while hiding important detail filters makes the web UI less useful than product-search style references such as Archives of Nethys or catalog search experiences.

## Desired Outcome

Design and implement a stronger web filter UX for browse and search workflows.

The pass should decide:

- which filters are standard and visible by default;
- which filters are optional but easy to add;
- how optional filters are grouped and ordered for users rather than source-schema owners;
- which labels should differ from internal app or artifact names;
- how selected optional filters remain visible and removable;
- how counts and disabled/unavailable options behave as filters compose;
- how common simple queries remain easy within the structured group editor.

## Constraints

- Use app-service/filter-discovery contracts rather than duplicating discovery semantics in frontend code.
- Keep source-only concepts out of user-facing labels unless they are meaningful to users.
- Reuse the maintained structured query builder and shared typed predicates; do not add a second filter-state model or CEL/editor translator.
- Avoid making all discovered filters visible by default.

## Related

- [Architecture overview](../../architecture/overview.md)
- [ADR 0029: Local web app boundary](../../architecture/decisions/0029-local-web-app-boundary.md)
- [Rust web text-scoped filter counts](./rust-web-text-scoped-filter-counts.md)
