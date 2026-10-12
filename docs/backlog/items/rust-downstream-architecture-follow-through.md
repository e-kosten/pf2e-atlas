# Downstream architecture follow-through

Status: in_progress

## Context

The downstream architecture review examined integration/source-modeling at
e76a7e68. Atomic mutable actions, participant stale-write protection, truthful
commit receipts and frontend mutation/draft lifecycle are implemented separately
under ADR0048. Retain the existing source-backed authority and crate boundaries.

## Current implementation slice

- Pin one immutable artifact generation across a web reader pool and a multi-step
  CLI operation, retaining bounded parallel reads.
- Replace request-only result windows with stateless generation-checked search
  pages and deliberate browser recovery from page1. Preserve retrieval behavior;
  add no candidate or query-inference cache.
- Skip abandoned queued work; evaluate existing Tokio/rusqlite and inference
  cancellation before designing active-work cancellation.
- Preserve typed oversized-query errors as invalid input rather than model
  unavailability through application and HTTP output boundaries.

## Bounded consumer follow-through

- Admit malformed URL filter JSON structurally before traversal; keep executable
  filter semantics backend-owned.
- Emit one CLI JSON envelope for multi-record outcomes, with deliberate per-key
  failure behavior.
- Reload contextual encounter previews through their actual query owner, and
  make shell Refresh target the active view.
- Explain partially supported condition effects on casting attack/DC without
  expanding gameplay automation implicitly.
- Shrink the private encounter arithmetic workspace to fields actually used by
  bounded mechanics. Evaluate removal of duplicate SQLite read connections or
  the private mirrored web-service trait only for demonstrated consumer benefit.

## Non-goals

No new persisted normalized record model, generic layout interpreter, service
facade, global browser state framework, or ingestion redesign. Optional visual
polish remains in its existing backlog. The approved retrieval contract is recorded
in [ADR0049](../../architecture/decisions/0049-search-lifetimes-and-pages.md).
This backlog item records remaining work rather than independently authorizing it.
