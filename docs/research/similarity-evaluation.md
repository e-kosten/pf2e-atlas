# Stored-vector Similar evaluation

Date: October 27, 2026. Historical experiment at implementation `03b674c6167a4da99d7618372b1c48c53fc39175`.
These findings informed configurable Similar; they do not evaluate the later
identity-inclusive All preset or Core sections.

## Decisions

Keep **Passage matches, Any content, decay 0.5** as the record default.
That configuration was chosen before opening the holdout, rather than selected
as the best point in the measured decay grid. Keep **Trait definition** as the
trait default: trait results were mixed across profiles.

| Held-out whole-record results | Name and identity | Passage Any, decay 0.5 |
| --- | ---: | ---: |
| BGE | 0.359 / 3.06 | 0.866 / 4.67 |
| EmbeddingGemma | 0.391 / 3.50 | 0.826 / 4.67 |

Values are pool-relative nDCG@10 / useful results among the first five.
Each of the six held-out whole-record tasks improved over Name and identity in
both profiles, both independent model-opinion panels and the adjudicated view.
This supports the chosen baseline on these tasks, not a universal winner.

Content average did not offer a large latency advantage for complex records:
candidate generation still used multiple seed-vector queries. Choose it for its
ranking behavior rather than assuming a single cheap lookup. Corresponding
sections and other decay values remain useful alternatives; this study does not
establish a universally optimal decay or justify per-kind presets.

## Method and limits

The frozen study covered 24 scenarios, 22 distinct seeds, both stored profiles and
552 applicable case/profile/variant cells. Methods included Name and identity,
Passage matches with decay 0, 0.25, 0.5, 0.75 and 1 under Any/Corresponding, and
Content average. Whole-record, selected-section and trait tasks were analyzed
separately. The holdout contained only three theme clusters; related tasks were
not treated as independent samples.

The blinded judging pool contained 916 task/candidate pairs. Two model panels
supplied 1,832 opinions; a third review adjudicated 119 disputes. Definition-less
Amp was diagnostic-only, with 35 unresolved pairs excluded uniformly from
semantic quality means. Exact provider model builds were unavailable. These are
model-assisted judgments, not human gold labels.

Grades were 0–3, nDCG gains were 0/1/3/7 and useful@5 counted grades of at least 2.
The ideal ranking came from the judged pool, not the full corpus. Means gave
hazard, creature and spell groups equal weight. Section-focused baseline results
were context-only because Name and identity cannot honor section selection.

A controlled, source-nominated 278-root oracle found all exact top-ten winners
admitted in all 504 content-method cells. All 552 cells, including baseline,
returned the oracle's top ten. Baseline admission itself was unavailable.
This establishes bounded subset behavior, **not full-corpus recall**.

A separate exploratory displayed-text audit inspected 12 explanations containing
21 items. Eleven explanations were supported overall and one partly supported;
15 items were supported, four partial and two unsupported. Distractions included
generic carried-item activation text and an associated prerequisite. Correct
source navigation alone did not establish semantic usefulness. This small audit
did not tune the frozen study or establish corpus-wide explanation quality.

## Operational observations

Thirty-two fresh developer processes each ran one cold and 20 warm full searches:
672 requests succeeded. The largest warm p95 was 17.089 seconds for Lich; maximum
single-cell process peak RSS was 189.69 MiB. Common-case and complex-case
provisional budgets were 5 and 20 seconds, with 512 MiB process peak RSS.

Lich/BGE warm p95 was 17.09 seconds for Passage Any/0.5 and 15.96 seconds for
Content average Any. Brown Mold/BGE measured 2.88 and 2.89 seconds respectively.
The legacy baseline took roughly 0.3 seconds. OS caches and background load were
uncontrolled; these measurements are development observations, not shipping SLOs.
Multi-method research-batch memory is not single-request memory.

A subsequent integrated build preserved top-20 ordering in eight representative
CLI comparisons, with all 140 exposed scores equal. This was bounded regression
evidence, not a rerun of relevance or timings on later product versions.

## Local evidence and follow-up

The measured source pin was `28d9d156f252d084371d994dc359509b49eadf18`,
with artifact generation `a9474928b73e8cf89e84c5ed6990a838:6`.
Complete protocols, frozen inputs, rankings, judgments, analysis/replay tools and
receipts live in ignored `scratch/similarity-evaluation/study-2026-10/`,
alongside the original experiment tree. See its `LOCAL-REPLAY.md` and the
[research handoff](search-evaluation-handoff.md). They are not shipped or required
by repository tests/CI. Reproduction needs those local files and the matching
artifact; a fresh clone contains only this summary.

All opened holdouts are now exposed. Evaluate Core presets, the separate identity
component, equipment variants and decay with fresh cases before proposing further
default changes. Current policy lives in
[ADR 0059](../architecture/decisions/0059-configurable-similarity.md) and
[ADR 0060](../architecture/decisions/0060-core-sections-and-search-scope.md);
remaining work is tracked in the
[search-quality backlog](../backlog/items/rust-search-quality-tuning.md).
