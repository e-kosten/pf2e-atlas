# Downstream architecture follow-through

Status: planned

## Context

The downstream architecture review examined integration/source-modeling at
e76a7e68. Atomic mutable actions, participant stale-write protection, truthful
commit receipts and frontend mutation/draft lifecycle are implemented separately
under ADR0048. Retain the existing source-backed authority and crate boundaries.

Coherent artifact lifetimes, stateless pages, cooperative read abandonment and
typed query-input errors are described in
[the completed search lifecycle slice](../history/items/search-lifetimes-and-pages.md).

## Bounded consumer follow-through

- Admit malformed URL filter JSON structurally before traversal; keep executable
  filter semantics backend-owned.
- Emit one CLI JSON envelope for multi-record outcomes, with deliberate per-key
  failure behavior.
- Make shell Refresh target the active view.
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
