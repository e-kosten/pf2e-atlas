# Rust Web Filter UX Expansion

Status: in_progress
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-11

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

## Current implementation scope

Backend contracts provide a catalog-owned common Level / rank field, ordered
Kind/Level-rank/Rarity/Traits roles and readable editor metadata. Contextual
suggestions/counts use typed optional availability independently of valid search.
Explicit count-free vocabulary retains artifact/list scope and selected values.
The shared browser controls now expose those common fields, preserve complex
conditions in the advanced editor, retain invalid input as local drafts and use
searchable grouped fields with readable operators. Value controls use explicit
server-side vocabulary search/paging and retain selected values. The browser
uses its hybrid product default, with page size beside results; filter URLs
retain the canonical predicate. A narrow Ant Drawer and pane-width layout serve
search and saved lists through the same owners. Per-value include/exclude states
share one canonical predicate, while the advanced editor shows residual conditions
without duplicating common clauses. Structural boolean constants remain editable.
Unfinished drafts use local feedback. Browsing results use row activation and
native full-page links without radio selection. Browser-local remembered Kind
is a default only for fresh bare search visits; incoming URLs remain authoritative.

Common categorical fields use one compact searchable Ant picker shared by search
and saved lists. Each option cycles neutral/include/exclude with a small marker;
only active removable selections appear when closed. Menus retain row order and
keyboard focus during edits, including selected literals outside the current
vocabulary page. Scoped vocabulary remains count-free with bounded server search
and paging; complex Any/All conditions retain the maintained advanced editor.

Similar search reuses stateless search pages, the table and canonical predicate.
Its URL preserves the root seed and filters through reload, Back/Forward and
generation recovery from page1. Keyword entry replaces the seed; clearing the criterion
preserves filters. Similarity uses catalog choices and count-free vocabulary,
with contextual text counts disabled. Top matches and available pages retain
bounded-search meaning without an exhaustive total claim or a hidden same-kind
filter. The related-record preview links to this same search workflow.

Focused frontend tests and read-only real API/browser workflows are implemented.
Completion requires integrated verification, independent Astra task/visual review,
and product feedback on the common and advanced workflows. Local tests alone do
not finish this backlog item.

## Constraints

- Use app-service/filter-discovery contracts rather than duplicating discovery semantics in frontend code.
- Keep source-only concepts out of user-facing labels unless they are meaningful to users.
- Reuse the maintained structured query builder and shared typed predicates; do not add a second filter-state model or CEL/editor translator.
- Avoid making all discovered filters visible by default.

## Related

- [Architecture overview](../../architecture/overview.md)
- [ADR 0029: Local web app boundary](../../architecture/decisions/0029-local-web-app-boundary.md)
- [Rust web text-scoped filter counts](./rust-web-text-scoped-filter-counts.md)
