# Rust Search Quality And Retrieval Weight Tuning

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-05-17

## Problem

The Phase 6 Rust search runtime should land the retrieval architecture first: FTS, vector retrieval, default hybrid RRF, weighted RRF plumbing, exact identity handling, and quality fixtures. The actual default weights and retrieval windows should be tuned after that baseline is measurable rather than guessed during initial implementation.

Without a dedicated follow-up, tuning work can leak into Phase 6 as ad hoc rerank rules or repeated one-off constants. That risks reproducing the Node runtime's quality hacks instead of evaluating the Rust retrieval axes directly.

## Desired Outcome

Evaluate and choose Rust search defaults from measured search-quality runs after Phase 6 lands.

The tuning pass should cover:

- FTS column and BM25 weights.
- RRF rank constant.
- FTS and vector candidate windows.
- Weighted RRF lane weights.
- Vector unit participation and collapse policy, if fixture failures point there.
- Whether any generic rerank adjustment is still justified after retrieval weights are tuned.

The outcome should be documented defaults plus the smallest stable configuration surface needed for future tuning.

## Constraints

- Do not add Tantivy, LanceDB, learned rerankers, or non-default embedding model switches as part of this item unless a separate quality decision justifies expanding scope.
- Do not expose individual FTS component weights as stable user-facing CLI flags before they have proven durable.
- Prefer fixture-backed changes over subjective one-query adjustments.
- Preserve the Phase 6 product model: one opinionated default search, with advanced retrieval/fusion controls for diagnostics and evaluation.

## Notes

Phase 6 should centralize FTS weights in code so this item can tune them cleanly. Initial Phase 6 defaults should use plain RRF through the same implementation as weighted RRF with equal lane weights.

## Related

- [Rust Phase 6 Search Runtime Plan](../../../scratch/plans/2026-05-17-rust-phase-6-search-runtime.md)
- [Rust CLI runtime migration research](../rust-cli-runtime/README.md)
