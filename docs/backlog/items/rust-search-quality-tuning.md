# Rust Search Quality And Retrieval Weight Tuning

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-05-17

## Problem

The Phase 6 Rust search runtime should land the retrieval architecture first: FTS, vector retrieval, default hybrid RRF, weighted RRF plumbing, and exact identity handling. Full live-index quality fixtures, parity reports, default weights, and retrieval windows should be tuned after that baseline is measurable rather than guessed during initial implementation.

Without a dedicated follow-up, tuning work can leak into ad hoc rerank rules or repeated one-off constants. That risks accumulating quality hacks instead of evaluating the Rust retrieval axes directly.

## Desired Outcome

Evaluate and choose Rust search defaults from measured search-quality runs after Phase 6 lands.

The tuning pass should cover:

- Whether the broad weighted FTS path should be removed entirely or retained only as a diagnostic/tuning primitive now that precision FTS is the product default.
- Precision FTS lane tuning, including whether kind/type/source facets should join the current title/alias and trait/taxonomy lanes.
- FTS confidence policy tuning, including the current `demote-weak` default and whether weak/medium lexical evidence should be discounted differently in hybrid fusion.
- Candidate hydration cost in ranked search. Precision FTS confidence currently hydrates bounded FTS/vector candidates before final fusion; if larger candidate windows become necessary, consider a lighter ranking-facts load instead of full record hydration.
- FTS column and BM25 weights for any retained broad weighted FTS diagnostic path.
- Top-k search-quality fixtures from the bakeoff set.
- Rust-owned embedding and ranking comparison harnesses for repeatable model, query, and fusion experiments.
- accepted-difference reports against observed task quality.
- FTS token composition policy, including OR versus AND behavior and prefix matching.
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
- Keep comparison tooling Rust-owned or language-neutral. Do not reintroduce retired scratch harnesses for search-quality runs.
- Preserve the Phase 6 product model: one opinionated default search, with advanced retrieval/fusion controls for diagnostics and evaluation.

## Notes

Phase 6 should centralize FTS weights in code so this item can tune them cleanly. Initial Phase 6 defaults should use precision FTS, weighted RRF with equal FTS/vector lane weights, and a weak-evidence demotion policy.

## Related

- [Rust Phase 6 Search Runtime Plan](../../../scratch/plans/2026-05-17-rust-phase-6-search-runtime.md)
- [Rust CLI runtime migration research](../rust-cli-runtime/README.md)
