# Source-backed search quality evaluation

Status: proposed
Priority: after the artifact cutover
Owner: unassigned
Last reviewed: 2026-10-09

## Current baseline

ADR 0046 replaces the old weighted-fusion, weak-match demotion and overflow-only
RichDocument policies. Precision FTS uses identity, verified aliases, typed
vocabulary and named definitions. Semantic units independently cover selected root
and owned prose. One maximum accepted unit score represents a root; hybrid RRF
combines one root rank per lane. Eligibility is applied before ranking, and
semantic candidate windows are bounded.

The cutover requires a judged real-model sample and attributable witnesses.
That acceptance check does not settle every future relevance tradeoff.

## Follow-up

Build a repeatable source-backed evaluation set with expected records and
explanatory passages. Include creature lore, owned afflictions, spell mechanics,
long documents, copied embedded spells, multilingual labels and noise-prone small
sections. Measure recall, rank, winning-witness quality, duplicate-root pressure
and latency separately.

Evaluate changes to lexical vocabulary, section selection, context, token
budgets, overlap, FTS weights, RRF constants and candidate windows one at a time.
Compare proposed changes against the current pinned model and artifact policies.
Record intended differences and corpus costs rather than adding one-query
reranking exceptions.

Alternative embedding models, inference upgrades, learned rerankers and new search
backends require their own evidence and decision. Do not restore old user-facing
fusion knobs or heuristics merely because they existed in the prior pipeline.

## References

- [Artifact contract](../../architecture/artifact-contract.md)
- [ADR 0046](../../architecture/decisions/0046-source-backed-reference-artifact.md)
- [FTS tokenization exploration](./rust-fts-tokenization-stemming.md)
