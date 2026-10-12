# Downstream architecture follow-through

Status: proposed

## Context

The downstream architecture review examined integration/source-modeling at
e76a7e68. Atomic mutable actions, participant stale-write protection, truthful
commit receipts and frontend mutation/draft lifecycle are implemented separately
under ADR0048. Retain the existing source-backed authority and crate boundaries.

## Next design slice

- Pin one immutable artifact generation across a web reader pool and a multi-step
  CLI operation, retaining bounded parallel reads.
- Decide whether result windows should be stateless requests or useful bounded
  generation-bound results. Current handles retain requests and rerun retrieval;
  they do not stabilize candidates or reuse query inference. Measure only what
  is needed to choose the lifetime rather than adding another cache by default.
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
polish remains in its existing backlog. Plan and discuss the chosen retrieval
contract before implementation; this item records work rather than authorizing it.
