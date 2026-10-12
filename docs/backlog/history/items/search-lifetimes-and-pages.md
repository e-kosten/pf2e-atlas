# Search lifetimes and stateless pages

Status: done

Web reader startup compares the physical artifact instance and complete build
context across its independent lanes. Failed startup joins started workers;
successful startup pins the reference artifact until restart. Commands lazily
retain one reader across composed operations without opening it for source-free
local-state workflows.

One generation-checked page operation replaces request-only result-window state.
Browser recovery cancels and retires affected immutable caches, preserves criteria
and mutable drafts, and restarts at page1. Contextual encounter previews reload
their actual owner and require explicit reopening with the refreshed selection.
There is no new ranking, candidate or query-embedding cache.

Replaceable page and discovery reads carry cooperative abandonment through the
HTTP waiter to queue admission, execution and projection. Queued reads can skip
expensive work; running blocking work retains its permit. Durable mutations and
receipts stay uncancellable. A real HTTP/1 full-disconnect probe supplements the
deterministic future/queue tests; it does not establish every browser-abort mode
or active native interruption.

Empty and over-budget semantic queries retain typed invalid-input errors through
text retrieval, discovery and application/HTTP output. Model/cache/inference
failures keep their distinct classification.

Current policy: [ADR0049](../../../architecture/decisions/0049-search-lifetimes-and-pages.md).
Remaining consumer work is tracked in
[downstream architecture follow-through](../../items/rust-downstream-architecture-follow-through.md).
