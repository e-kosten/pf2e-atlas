# Rust Web Filter UX Expansion

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-06-07

## Problem

The initial Atlas web filter surface supports standard filters, optional discovered filters, and dynamic counts, but the product experience still needs a deliberate pass over filter grouping, field priority, labels, and progressive disclosure.

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
- how the basic filter surface remains compatible with a future complex filter-tree editor.

## Constraints

- Use app-service/filter-discovery contracts rather than duplicating discovery semantics in frontend code.
- Keep source-only concepts out of user-facing labels unless they are meaningful to users.
- Preserve the V1 basic-filter model; do not implement the full canonical filter-tree editor as part of this item.
- Avoid making all discovered filters visible by default.

## Related

- [Architecture overview](../../architecture/overview.md)
- [ADR 0029: Local web app boundary](../../architecture/decisions/0029-local-web-app-boundary.md)
- [Rust web text-scoped filter counts](./rust-web-text-scoped-filter-counts.md)
