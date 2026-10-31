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
- Evaluate removal of duplicate SQLite read connections only for demonstrated
  consumer benefit. Encounter read preparation, arithmetic-field ownership and
  the concrete web-to-app-service boundary follow the current
  [runtime boundaries](../../architecture/runtime.md#retrieval-and-application-workflows).

## Conditional architecture questions

The completed [whole-system review](../history/items/architecture-simplification-review.md)
retains the overall decomposition. These questions remain deferred and unassigned;
they are not confirmed product defects or prerequisites for program closure:

- **Pure/native embedding boundary:** identify an actual native-free consumer and
  the needed profile/input/identity/vector API first. Sketch a feature/export
  closure preserving native token validation, exact reuse identities and default
  behavior. Only then compare matched build/resource effects if those motivate
  isolation. Index/search consolidation alone would not remove native dependencies.
- **Fixture composition:** inventory index/search fixture callers and distinguish
  reference-only tests from full-history and saved-state preservation tests.
  Compare a smaller no-cycle topology with full-workspace integration composition
  above schema owners. Lower fixtures intentionally stop before the current global
  ledger and runtime upgrades app-service fixtures; this is maintenance duplication,
  not demonstrated missing product migration coverage. Do not copy the newest
  ledger down or introduce another migration crate without a smaller complete design.

Small tag-validator placement or concrete-writer cleanup may be considered when
editing those owners for a real task; no independent migration is assigned.

## Non-goals

No new persisted normalized record model, generic layout interpreter, service
facade, global browser state framework, or ingestion redesign. Optional visual
polish remains in its existing backlog. The approved retrieval contract is recorded
in [ADR0049](../../architecture/decisions/0049-search-lifetimes-and-pages.md).
This backlog item records remaining work rather than independently authorizing it.
