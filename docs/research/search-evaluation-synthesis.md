# Search evaluation: findings and product requirements

This summarizes the October 2026 full-corpus research, including negative results
that should constrain future experiments. It is not a model leaderboard or a claim
that search is finished. The [curated evidence](evaluations/2026-10-search/README.md)
contains queries, individual opinions, adjudications, rankings, metrics and method
contracts. [Earlier notes](source-backed-search-evaluation.md) retain the dated
history; the [archive map](search-evaluation-handoff.md) locates full local evidence.

## What the search should answer

Exact primary names and verified legacy aliases should locate their record first.
Ordinary search is broader than strict identity resolution: useful alternatives
and records about the subject should follow. For `leather armor`, Studded Leather
Armor is generally more useful than an unrelated creature merely wearing Leather
Armor. For `harrowkin`, family abilities and creatures can both be direct answers.
Explicit reference/owner searches are a different workflow; do not assume every
name query is asking for entities that possess the named object.

Primary identity, root explanation and embedded evidence have different roles,
but storage depth alone does not determine relevance. A hazard's actual mechanism
may be an embedded reaction. A creature's embedded spell may establish a requested
capability, or merely distract from the creature's own identity. Distinguish
possession from a usable spellcasting association when judging the source.

Flavor (`masked spirit`), functional discovery (`walk through walls` with a spell
filter), and composite capabilities (`wizard with fire magic`) all matter. These
examples are exposed development cases, not the only acceptable answers or grounds
for record-specific boosts. Preserve explicit kinds/filters rather than inferring
hidden user restrictions. Theme labels may eventually be useful filter axes, but
should not substitute for effective search or become query-specific annotations.
User-facing intent selectors require meaningful demonstrated benefit; no experiment
here justifies adding them. Known-good candidates entered shuffled review pools
without being announced as the expected answer or forced into retrieval results.

## Ownership and reranking experiments

Blanket penalties for embedded content, excluding associated documents, or giving
prose universal priority produced mixed results: descriptive discovery could
improve while useful intrinsic abilities and owner-capability answers suffered.
Multiple matches should provide inspectable evidence, not automatically accumulate
rank influence. Do not repeat the same ownership/role weight sweep without a new,
falsifiable hypothesis and suitable owner-capability controls.

Cross-encoder reranking can evaluate the query and richer record context together.
Joint context improved some cases substantially: in one bounded development run,
Dalgyal Gwishin reached first for `masked spirit`, while the baseline ranked it
69th with a creature filter and 168th without. That run admitted 200 candidates;
these numbers do not prove a production top-50 reranker would retrieve it.

Longer inputs did not reliably fix opposite-function, qualifier or composed-effect
errors. The fresh review retained important regressions and measurable inference
cost. Reranking was not adopted. It cannot recover candidates outside its input
pool; distinguish ingestion/eligibility loss, retrieval-window exclusion and a
rerank cutoff before choosing a remedy. `cross language barrier` and `repair broken
objects` exposed direct spells present in retrieval but below the reranker cutoff.
A later reranker study needs fixed evidence, an explicit candidate budget, actual
end-to-end timing and fresh queries—not just larger contexts for familiar cases.

## Four models × two input representations

The pinned PF2e 6.12.4 study covered 25,560 semantic roots and 130,566 units, using
all eligible units, maximum passage score per root, and semantic-only ordering.
No FTS, exact-name promotion or fusion influenced quality. Model/native-tokenizer
contracts were pinned; this compares those exported packages and input recipes,
not each model's best possible settings or every production retrieval constraint.

The parent-context recipe prepended a bounded literal root-description excerpt to
eligible embedded passages, preserving child content. Embedded headers omitted pack
labels in both representations. It was not a full-document embedding or a learned
summary. The product currently uses the baseline recipe without parent excerpts.

Fresh results averaged paired scopes within each of 24 concepts:

| Configuration | nDCG@10 | Useful results at 5 |
| --- | ---: | ---: |
| BGE Small / baseline | .598 | 27.1% |
| BGE Small / parent context | .571 | 27.5% |
| BGE Base / baseline | .621 | 35.0% |
| BGE Base / parent context | .575 | 32.1% |
| EmbeddingGemma / baseline | .637 | 30.8% |
| EmbeddingGemma / parent context | .639 | 33.3% |
| Qwen / baseline | .576 | 28.3% |
| Qwen / parent context | .561 | 26.7% |

BGE Base baseline had the strongest useful-result density; Gemma had the best
nDCG point estimates. Larger-model positive nDCG contrasts against BGE Small all
included zero after the prespecified seven-comparison correction. Parent context
hurt Small nDCG even after correction; it did not provide a consistent improvement
across models. Qwen's tested export showed no aggregate fresh quality advantage.
These results supported trying Base and Gemma in the product, not declaring a
universal winner or automatically adding parent context.

Useful means grade 2 or 3; gain is 0/1/3/7 for grades 0/1/2/3. A related opposite
function can receive grade 1 and a separate contradiction flag. It is not equated
to unrelated noise, nor counted as a useful answer. Two fresh pools had no known
useful answer, so nDCG must be read alongside usefulness. Pooled nDCG measures
ordering within judged candidates, not exhaustive recall. Review included expanded
independent overlap and source adjudication; agreement alone did not guarantee
correct interpretation. Uncertainty and provenance incidents remain in the data.

Costs were analyzed after quality finalization. Qwen's long build and every model's
startup/query timings are separate observations, not grading inputs. Research
all-passage scans and model-only timing are not production end-to-end latency.
See the saved performance definitions before comparing hardware or runtimes.

## Lexical policy and fusion

The final exploratory matrix reused captures/vectors, two semantic profiles and
one expanded relevance pool. The revised rubric credited useful subject alternatives
without treating incidental possession as a direct answer. Original opinions remain
preserved; their raw averages must not be combined with the corrected rubric.
One unresolved source cluster was excluded uniformly, leaving 47 holdout clusters.

| Holdout configuration | BGE Base nDCG@10 | Gemma nDCG@10 |
| --- | ---: | ---: |
| Earlier lexical policy + RRF | .717799 | .807550 |
| Earlier lexical policy + score (.25 lexical weight) | .718261 | .811166 |
| Field-aware policy + RRF | .724781 | .816204 |
| Field-aware policy + score (.25 lexical weight) | .726563 | .817719 |

Field-aware lexical policy was the clearer improvement. This was a bundle of
separate identity/alias rows, no inherited parent terms in embedded rows,
strongest-row weighting and exact-match tier changes—not a causal isolation of
text projection alone. Conceptual results in this comparison were unchanged.

Score versus RRF within that policy was a much smaller tradeoff. Bootstrap
intervals included zero, and high-grade judgment sensitivity reversed the small
aggregate preference. The matrix was extended after results were seen; its
intervals do not account for post-hoc selection or rubric revision. Both methods
remain available for product use; the user chose Score-based as default.
[ADR0053](../architecture/decisions/0053-field-aware-search-fusion.md) owns current
behavior. Long term, converge to one fusion strategy. Semantic model selection is
independent; switching fusion requires no new embeddings.

## Future work and evidence boundaries

- Gather new natural queries while using the product; freeze independent holdouts
  before comparing new policies. Previously opened holdouts cannot be made fresh.
- Investigate candidate admission and root ranking separately, then witness quality.
  A good result can have a poorly chosen supporting excerpt.
- Query spelling, stemming and reference-scoped search remain distinct follow-ups;
  do not assume a more permissive lexical match should dominate semantic discovery.
- Grade corrections must retain originals and be applied uniformly to every
  compared configuration. Average gain/usefulness across opinions where the
  protocol specifies it; exponentiating an average grade changes the metric.
- Reuse exact vectors and ranking captures when the hypothesis permits. Avoid new
  inference merely to change fusion, inspect a slice, or correct a judgment.

The [search backlog](../backlog/items/rust-search-quality-tuning.md) tracks open
work. Architecture owns product policy; the research archive records evidence and
its limitations. Historical proposals are not authorization for new implementation.
