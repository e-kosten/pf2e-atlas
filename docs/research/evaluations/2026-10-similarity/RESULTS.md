# Stored-vector Similar evaluation results

## Default decisions

Keep **Passage matches, Any content, decay 0.5** as the record default. This was the preview choice specified before the holdout was opened, rather than the best-looking point selected from the decay grid. Retain the legacy **Trait definition** (`baseline`) default for trait seeds. Explicit URLs and valid saved browser choices keep their authority.

The record choice is supported by a large usefulness improvement in this small model-assisted experiment: every one of the six held-out whole-record tasks improves over Name and identity in both stored profiles, in each independent opinion panel and the adjudicated view. The same six development tasks also improve in the adjudicated view. Trait results are mixed across profiles and do not justify changing that default.

| Held-out whole-record production | Name and identity | Passage Any, decay 0.5 |
| --- | ---: | ---: |
| bge-base | 0.359 / 3.06 | 0.866 / 4.67 |
| embeddinggemma | 0.391 / 3.50 | 0.826 / 4.67 |

Cells above and in the appendix show **pool-relative nDCG@10 / useful results among the first five**. These are kind-macro means: tasks are averaged within kind/theme cluster, clusters within kind, then hazard, creature and spell receive equal weight. The six held-out whole-record tasks contain two hazards, one creature and three spells; two profiles do not double the number of independent tasks.

## Evidence and limits

- The frozen input set contains 24 scenarios, 22 unique seeds and 552 applicable case/profile/variant cells. The holdout has only three theme clusters. This is exploratory evidence about these tasks, not a universal relevance guarantee. No algorithm, candidate budget, role or chunking was tuned after opening it.
- The blinded union contains 916 scenario–candidate pairs. Two isolated model-opinion panels produced 1,832 original opinions; a third blinded review resolved 119 quality-eligible disputes. All 35 unresolved pairs belong to the definition-less Amp diagnostic and are uniformly excluded from semantic quality means. Original opinions and adjudications are preserved.
- Some panel batches ran in separate isolated sessions. Exact provider model builds were not exposed; recorded identifiers disclose that limitation. These are model-assisted judgments, not human gold labels.
- nDCG uses gains 0, 1, 3, 7 and an ideal ranking over the entire judged union pool, not the full corpus. Useful@5 counts grades at least 2. Derived pairwise preferences are differences in graded nDCG; they were not independently elicited human preferences. Root overlap measures result diversity, not usefulness.
- Section-focused tasks are a separate advanced-use stratum. Baseline cannot honor selected sections and is context-only for those tasks; it is excluded from matched-selection primary comparisons. Exact trait membership is separate from semantic usefulness.
- Evidence-support opinions ask whether the checked source supports the candidate connection. They do not independently grade every method-specific displayed excerpt. Canonical source-address/navigation correctness and semantic excerpt support are different claims.

## Displayed-text support: exploratory follow-up

A separately declared, purposive audit examined the actual displayed explanation text for the first two product results in six declared cells: 12 candidate explanations and 21 displayed items. Cells were declared before capture; rubric and packet packaging were specified after capture but before an independent fresh model reviewer saw them. Candidate identity was visible; method, profile, rank, scores, weights and method-specific notices were hidden. Passage counts and shapes remained visible, so blinding was imperfect. This follow-up uses the historical `03b674c6` product CLI (`0ab4633b7a975c219e3dfda81dfb5c3b917163df7806efae41b214d1e9b49fe0`) and generation 6. It does not amend the frozen holdout, contribute to its metrics, tune defaults or establish corpus-wide semantic fidelity.

The reviewer assessed whether the text actually shown supported the declared task and selected scope, using canonical source context to distinguish intrinsic properties, owned capabilities and equipment associations. Context was explicitly separate from displayed evidence. Eleven candidate explanations were supported overall and one was partial; among individual items, 15 were supported, four partial and two unsupported. None was unknown. An overall supported result can contain distracting or unsupported items; the overall assessment was not a vote over items.

| Declared cell | Selection and configuration | Two candidate outcomes after unblinding |
| --- | --- | --- |
| cell-01 / Brown Mold | Whole record; BGE, Passage Any, decay 0.5 | Enhanced Brown Mold and Mutated Brown Mold: both supported, all six items supported |
| cell-02 / Brown Mold | Reset only; Gemma, Content average Corresponding | Mutated Brown Mold and Enhanced Brown Mold: both supported, both items supported |
| cell-03 / Lich | Whole record; BGE, Passage Corresponding, decay 0.25 | Mythic Lich and Lich Legion: both supported overall; two items supported, three partial, one unsupported |
| cell-04 / Lich | Description and Rejuvenation; Gemma, Content average Any | (Lich) Rejuvenation and Lich Soul Cage: both supported, both items supported |
| cell-05 / Fire trait | Trait definition; BGE, Baseline | Burning Spell: supported overall, one supported and one unsupported item. Effect: Kindling: partial overall and partial item |
| cell-06 / Hydraulic Push | Whole record; Gemma, Passage Any, decay 0.5 | Water Elemental Vessel and Undine Hydromancer: both supported through owned spell capabilities, both items supported |

The limits are substantive despite the successful citation checks:

- Mythic Lich's Siphon Life supplied a salient intrinsic connection. Its Howling Blizzard excerpt only partly supported the requested undead identity and powers; the carried teleport scroll's generic activation instructions were unsupported for that purpose. Lich Legion's two vampiric spell excerpts partly supported shared owned powers, while Void Healing supplied the intrinsic connection. None of those displayed excerpts established immortality or soul anchoring.
- Burning Spell's effect described fire behavior, but its separate philosophy prerequisite supplied only an association and was unsupported as an explanation of that behavior. Effect: Kindling was the single partial candidate: it modified damage from fire-classified spells without displaying the actual burning, creation or manipulation mechanism or stating that the added damage was fire damage.
- The water-blast matches showed actual owned damage-and-push spells; the reviewer did not infer those capabilities merely from a water-themed name. Selected reset and soul-anchor comparisons stayed within the explicitly selected seed scope.

Separately, the preparer recorded 39 exact source-navigation/text checks: all navigation requests were accepted and all diagnostic text prefixes were present, with zero misses or identity-only entries. These are address/fidelity checks, not semantic grades. The two unsupported items are semantic relevance limits, not demonstrated citation or codec bugs. Neither this reviewer nor the original full-source opinion panels graded whether excerpts causally explain numeric vector scores or contribution weights; the original panels instead assessed candidate connections using fuller checked source material.

[The retained audit metadata](results/displayed-text-audit/summary.json) preserves every original paraphrased item/candidate opinion, including bad items, original instructions, index, active v2 freeze, predeclaration, capture/command receipts and source-address/hash bindings. Before any judging, v2 corrected a prose-only note that had miscalled a baseline prerequisite excerpt identity-only; the actual DTO packets, instruction and index were unchanged, and v1 remains retained. Packet/source bodies remain ignored. Offline replay verifies their recorded hashes, opinion/item identities, grade enums, bindings and counts; it cannot re-judge or re-verify the omitted text. This small one-reviewer audit is exploratory, not human gold or a comparative method-performance estimate.

## Candidate recall

The controlled oracle directly scored all 278 declared roots for each method/profile, excluding a record seed from its own eligible set (277 eligible roots for record tasks; 278 for traits). This mixed, source-nominated universe is not representative of all 29,813 product roots. Both oracle and bounded arms disable remaster preference; production captures retain it.

All 504 content-method cells admitted every oracle top-ten winner: candidate-admission recall@10 is 1.0 in each cell, above the provisional 95% subset gate. All 552 cells, including baseline, returned the same top ten as the oracle. Baseline admission is unavailable because the legacy pipeline does not expose its complete admitted union. There were no admission misses or failed applicable cells. These results do **not** establish full-corpus recall.

Candidate limits remain unchanged:256 KNN hits per query, 128 seed units, 4,096 candidate roots, 65,536 candidate units and 8,000,000 comparisons in a product request. Explanation caps and hiding weights that round to 0.0% remain presentation limits only.

## Runtime

The primary measurement uses 32 fresh instrumented developer processes. Each runs one complete cold request and 20 complete warm requests in the same service/context; all 672 requests succeeded. Percentiles use nearest rank (`ceil(p*N)-1`). OS page caches were neither controlled nor flushed. The 16-core host began with a load average around 22; background activity and the retained preview were uncontrolled. These are provisional development budgets, not shipping SLOs.

All 32 cells meet warm p95 budgets of 5 seconds for common seeds and 20 seconds for the lich, plus 512 MiB process peak RSS. The largest warm p95 is 17.089 seconds; the largest single-cell process peak is 189.69 MiB. The developer executable enables partial read instrumentation; these counters are not a full SQLite trace.

Warm **p50 / p95 in seconds**:

| Seed / profile | Baseline | Passage Any, 0.5 | Content average Any |
| --- | ---: | ---: | ---: |
| Brown Mold / bge-base | 0.29 / 0.30 | 2.83 / 2.88 | 2.83 / 2.89 |
| Brown Mold / embeddinggemma | 0.30 / 0.31 | 2.85 / 2.92 | 2.81 / 2.87 |
| Striding Fire / bge-base | 0.31 / 0.31 | 1.53 / 1.56 | 1.52 / 1.59 |
| Striding Fire / embeddinggemma | 0.30 / 0.31 | 1.48 / 1.51 | 1.47 / 1.50 |
| Lich / bge-base | 0.30 / 0.31 | 16.50 / 17.09 | 15.54 / 15.96 |
| Lich / embeddinggemma | 0.29 / 0.30 | 16.45 / 16.77 | 15.54 / 15.79 |
| Heal / bge-base | 0.29 / 0.31 | 0.77 / 0.79 | 0.77 / 0.79 |
| Heal / embeddinggemma | 0.29 / 0.30 | 0.77 / 0.78 | 0.78 / 0.84 |
| Fire trait / bge-base | 0.30 / 0.31 | 0.72 / 0.75 | 0.72 / 0.74 |
| Fire trait / embeddinggemma | 0.30 / 0.31 | 0.67 / 0.68 | 0.67 / 0.68 |

The additional Lich Corresponding, decay 0.25 cells measured the current complex workflow:

| Profile | Warm p50 / p95 seconds | Process peak MiB |
| --- | ---: | ---: |
| bge-base | 15.89 / 15.97 | 138.08 |
| embeddinggemma | 16.00 / 16.17 | 141.17 |

Content average has little latency advantage for the lich in this recipe because candidate generation still protects recall with multiple seed-vector queries. It should be chosen for its ranking behavior, not assumed to be a single cheap query.

Mean phase costs for the Lich/BGE warm requests (milliseconds; inventory endpoint measured separately below):

| Method | KNN queries | Candidate vectors | Pair comparisons | Seed prep | Candidate generation | Vector reads | Scoring | Evidence hydration |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| passages-any-0p5 | 43 | 62730 | 2634660 | 1.14 | 13686.99 | 1762.37 | 1093.64 | 11.99 |
| average-any | 43 | 62730 | 207534 | 1.06 | 13648.96 | 1742.57 | 261.07 | 4.98 |
| passages-corresponding-0p25 | 44 | 56944 | 1503425 | 1.10 | 13981.95 | 1242.86 | 637.66 | 11.45 |

The separate section-inventory probe made 210 successful HTTP requests across 10 fresh product-server cells (one cold plus 20 warm per cell). Cold endpoint latency was 3.45–12.78 ms; maximum warm p95 was 2.81 ms. Startup is excluded from endpoint latency. The maximum whole-server-lifetime peak was 21.52 MiB, not a per-request allocation measure.

The uninstrumented product cross-check contains six fresh CLI samples and eight successful HTTP requests. A baseline request completed in 0.381 seconds while a complex request ran for 16.248 seconds. Following a complete HTTP disconnect, baseline and recovery requests completed in 0.322 seconds. CPU-idle observations are consistent with abandoned work stopping; they do not replace internal cancellation instrumentation or prove an exact cancellation deadline.

Memory scopes stay separate: the mixed HTTP probe server peaked at 156.64 MiB; the full multi-method research batch peaked at 580.23 MiB (608,419,840 bytes). The latter retains multiple evaluation censuses and is not a single product-request memory measurement. No inference, source rebuild or installed-workspace mutation was part of these measurements.

## Reproduction and provenance

The experiment reports declare and observe runtime revision `03b674c6167a4da99d7618372b1c48c53fc39175`, artifact generation `a9474928b73e8cf89e84c5ed6990a838:6` and source revision `28d9d156f252d084371d994dc359509b49eadf18`. The instrumented binary SHA256 is `3b1eee95c682d6df830fa26f3caaa071dd6ead1c6631a74cde1326d6d23aa55c`. The original build receipt records its pre-commit dirty tree and partial source census. A later [clean rebuild receipt](results/build-proofs/clean-03b-rebuild-proof.json), with its [build log](results/build-proofs/clean-03b-build.log), records a clean Git checkout of that exact revision in the same workspace, using pinned toolchain `1.95.0-aarch64-apple-darwin` and the same release command. The rebuilt executable is byte-identical to the measured binary, resolving the earlier source-census uncertainty. Both receipts are retained and hash-bound; recorded-data replay checks their identities and bindings without rebuilding the executable.

The historical study remains on the original generation-6 artifact. Later integration with main `153ae618` at feature revision `0ed3dd72` preserves main migrations 1–16 and assigns the similarity migration 17, with graph policy v6. Selector rediscovery, native-vector equivalence and representative parity checks for that integrated artifact are separate evidence and are pending at this addendum; these timings and quality judgments are not relabeled as measurements of the integrated head. Raw research packets, reports, source bindings and the original study database remain protected from cleanup until their retained-evidence handoff is reviewed.

[The compact result manifest](results/manifest.json) binds retained metadata, original prompt/opinion/adjudication bytes, cited source addresses/hashes, raw timing samples and score censuses. Dense generated tables use lossless `.jsonl.gz`; no game-source bodies are included. All 11,040 production top-twenty score values join to the exact fixed-pool matrix; all 4,320 actual capture values match that join exactly. Production order is preserved. Canonical remaster suppression only removes rows and does not transfer keys or scores.

Run the offline research gate:

```sh
node --experimental-strip-types docs/research/evaluations/2026-10-similarity/verify.ts
```

This runs the unchanged frozen-input verifier, canonical analysis and synthetic audit regression tests, and recorded-result replay. Replay checks byte/digest bindings, 1832 original opinions, 119 adjudications, score/rank joins, 552 controlled oracle cells, 672 full-search samples and 210 inventory samples, plus the separate 12-explanation/21-item displayed-text audit. It replays recorded data; omitted source bodies and vector math require the pinned source/artifact and are not re-verified. See [RUN.md](RUN.md) for fresh execution.

## Complete variant grids

Each cell is kind-macro nDCG@10 / useful@5. The primary view uses agreement or independent adjudication. Individual-panel aggregates, per-case metrics, derived paired preferences and within-profile root overlap remain in the compact results. A dash means the mode is inapplicable or the baseline selection is unmatched; it is not a failed trial or zero relevance.

### development: whole-record

| Variant | Production BGE | Production Gemma | Fixed-pool BGE | Fixed-pool Gemma |
| --- | ---: | ---: | ---: | ---: |
| baseline | 0.492 / 3.33 | 0.468 / 2.67 | 0.492 / 3.33 | 0.468 / 2.67 |
| average-any | 0.822 / 4.17 | 0.804 / 4.08 | 0.813 / 4.17 | 0.797 / 4.08 |
| average-corresponding | 0.882 / 5.00 | 0.804 / 4.58 | 0.874 / 5.00 | 0.797 / 4.58 |
| passages-any-0 | 0.797 / 4.33 | 0.870 / 4.33 | 0.797 / 4.33 | 0.870 / 4.33 |
| passages-any-0p25 | 0.820 / 4.33 | 0.867 / 4.33 | 0.820 / 4.33 | 0.867 / 4.33 |
| passages-any-0p5 | 0.836 / 4.33 | 0.872 / 4.33 | 0.836 / 4.33 | 0.872 / 4.33 |
| passages-any-0p75 | 0.869 / 4.33 | 0.888 / 4.33 | 0.869 / 4.33 | 0.888 / 4.33 |
| passages-any-1 | 0.880 / 4.33 | 0.896 / 4.33 | 0.880 / 4.33 | 0.896 / 4.33 |
| passages-corresponding-0 | 0.797 / 4.33 | 0.870 / 4.33 | 0.797 / 4.33 | 0.870 / 4.33 |
| passages-corresponding-0p25 | 0.820 / 4.33 | 0.867 / 4.33 | 0.820 / 4.33 | 0.867 / 4.33 |
| passages-corresponding-0p5 | 0.853 / 4.33 | 0.880 / 4.33 | 0.853 / 4.33 | 0.880 / 4.33 |
| passages-corresponding-0p75 | 0.885 / 4.67 | 0.877 / 4.67 | 0.885 / 4.67 | 0.877 / 4.67 |
| passages-corresponding-1 | 0.904 / 5.00 | 0.917 / 4.67 | 0.904 / 5.00 | 0.917 / 4.67 |

### development: trait-default

| Variant | Production BGE | Production Gemma | Fixed-pool BGE | Fixed-pool Gemma |
| --- | ---: | ---: | ---: | ---: |
| baseline | 0.917 / 5.00 | 0.809 / 4.50 | 0.917 / 5.00 | 0.809 / 4.50 |
| average-any | 0.916 / 5.00 | 0.844 / 4.50 | 0.917 / 5.00 | 0.844 / 4.50 |
| average-corresponding | — | — | — | — |
| passages-any-0 | 0.917 / 5.00 | 0.809 / 4.50 | 0.917 / 5.00 | 0.809 / 4.50 |
| passages-any-0p25 | 0.917 / 5.00 | 0.809 / 4.50 | 0.917 / 5.00 | 0.809 / 4.50 |
| passages-any-0p5 | 0.917 / 5.00 | 0.809 / 4.50 | 0.917 / 5.00 | 0.809 / 4.50 |
| passages-any-0p75 | 0.917 / 5.00 | 0.809 / 4.50 | 0.917 / 5.00 | 0.809 / 4.50 |
| passages-any-1 | 0.917 / 5.00 | 0.809 / 4.50 | 0.917 / 5.00 | 0.809 / 4.50 |
| passages-corresponding-0 | — | — | — | — |
| passages-corresponding-0p25 | — | — | — | — |
| passages-corresponding-0p5 | — | — | — | — |
| passages-corresponding-0p75 | — | — | — | — |
| passages-corresponding-1 | — | — | — | — |

### development: section-focused

| Variant | Production BGE | Production Gemma | Fixed-pool BGE | Fixed-pool Gemma |
| --- | ---: | ---: | ---: | ---: |
| baseline | — | — | — | — |
| average-any | 0.745 / 3.50 | 0.783 / 3.75 | 0.755 / 3.75 | 0.783 / 3.75 |
| average-corresponding | 0.863 / 5.00 | 0.764 / 4.50 | 0.863 / 5.00 | 0.764 / 4.50 |
| passages-any-0 | 0.890 / 4.75 | 0.859 / 4.50 | 0.890 / 4.75 | 0.859 / 4.50 |
| passages-any-0p25 | 0.890 / 4.75 | 0.859 / 4.50 | 0.890 / 4.75 | 0.859 / 4.50 |
| passages-any-0p5 | 0.890 / 4.75 | 0.841 / 4.50 | 0.890 / 4.75 | 0.841 / 4.50 |
| passages-any-0p75 | 0.890 / 4.75 | 0.841 / 4.50 | 0.890 / 4.75 | 0.841 / 4.50 |
| passages-any-1 | 0.890 / 4.75 | 0.841 / 4.50 | 0.890 / 4.75 | 0.841 / 4.50 |
| passages-corresponding-0 | 0.943 / 5.00 | 0.930 / 5.00 | 0.943 / 5.00 | 0.930 / 5.00 |
| passages-corresponding-0p25 | 0.943 / 5.00 | 0.911 / 5.00 | 0.943 / 5.00 | 0.911 / 5.00 |
| passages-corresponding-0p5 | 0.943 / 5.00 | 0.911 / 5.00 | 0.943 / 5.00 | 0.911 / 5.00 |
| passages-corresponding-0p75 | 0.925 / 5.00 | 0.911 / 5.00 | 0.925 / 5.00 | 0.911 / 5.00 |
| passages-corresponding-1 | 0.943 / 5.00 | 0.911 / 5.00 | 0.943 / 5.00 | 0.911 / 5.00 |

### holdout: whole-record

| Variant | Production BGE | Production Gemma | Fixed-pool BGE | Fixed-pool Gemma |
| --- | ---: | ---: | ---: | ---: |
| baseline | 0.359 / 3.06 | 0.391 / 3.50 | 0.359 / 3.06 | 0.391 / 3.50 |
| average-any | 0.741 / 4.83 | 0.711 / 4.83 | 0.730 / 4.83 | 0.711 / 4.83 |
| average-corresponding | 0.842 / 4.83 | 0.759 / 4.83 | 0.842 / 4.83 | 0.759 / 4.83 |
| passages-any-0 | 0.860 / 4.83 | 0.805 / 4.83 | 0.860 / 4.83 | 0.805 / 4.83 |
| passages-any-0p25 | 0.866 / 4.83 | 0.817 / 4.83 | 0.866 / 4.83 | 0.817 / 4.83 |
| passages-any-0p5 | 0.866 / 4.67 | 0.826 / 4.67 | 0.866 / 4.67 | 0.826 / 4.67 |
| passages-any-0p75 | 0.853 / 4.83 | 0.840 / 4.67 | 0.853 / 4.83 | 0.840 / 4.67 |
| passages-any-1 | 0.847 / 4.83 | 0.854 / 4.83 | 0.847 / 4.83 | 0.854 / 4.83 |
| passages-corresponding-0 | 0.842 / 4.83 | 0.806 / 4.83 | 0.842 / 4.83 | 0.806 / 4.83 |
| passages-corresponding-0p25 | 0.830 / 4.50 | 0.822 / 4.67 | 0.830 / 4.50 | 0.822 / 4.67 |
| passages-corresponding-0p5 | 0.859 / 4.67 | 0.849 / 4.67 | 0.859 / 4.67 | 0.849 / 4.67 |
| passages-corresponding-0p75 | 0.858 / 4.83 | 0.845 / 4.67 | 0.858 / 4.83 | 0.845 / 4.67 |
| passages-corresponding-1 | 0.872 / 4.83 | 0.865 / 4.83 | 0.872 / 4.83 | 0.865 / 4.83 |

### holdout: trait-default

| Variant | Production BGE | Production Gemma | Fixed-pool BGE | Fixed-pool Gemma |
| --- | ---: | ---: | ---: | ---: |
| baseline | 0.767 / 5.00 | 0.689 / 4.00 | 0.767 / 5.00 | 0.689 / 4.00 |
| average-any | 0.739 / 4.33 | 0.751 / 4.00 | 0.739 / 4.33 | 0.751 / 4.00 |
| average-corresponding | — | — | — | — |
| passages-any-0 | 0.767 / 5.00 | 0.680 / 4.00 | 0.767 / 5.00 | 0.680 / 4.00 |
| passages-any-0p25 | 0.767 / 5.00 | 0.680 / 4.00 | 0.767 / 5.00 | 0.680 / 4.00 |
| passages-any-0p5 | 0.767 / 5.00 | 0.680 / 4.00 | 0.767 / 5.00 | 0.680 / 4.00 |
| passages-any-0p75 | 0.767 / 5.00 | 0.680 / 4.00 | 0.767 / 5.00 | 0.680 / 4.00 |
| passages-any-1 | 0.767 / 5.00 | 0.680 / 4.00 | 0.767 / 5.00 | 0.680 / 4.00 |
| passages-corresponding-0 | — | — | — | — |
| passages-corresponding-0p25 | — | — | — | — |
| passages-corresponding-0p5 | — | — | — | — |
| passages-corresponding-0p75 | — | — | — | — |
| passages-corresponding-1 | — | — | — | — |

### holdout: section-focused

| Variant | Production BGE | Production Gemma | Fixed-pool BGE | Fixed-pool Gemma |
| --- | ---: | ---: | ---: | ---: |
| baseline | — | — | — | — |
| average-any | 0.600 / 3.75 | 0.791 / 5.00 | 0.600 / 3.75 | 0.791 / 5.00 |
| average-corresponding | 0.400 / 2.25 | 0.553 / 3.75 | 0.400 / 2.25 | 0.553 / 3.75 |
| passages-any-0 | 0.913 / 5.00 | 0.957 / 5.00 | 0.913 / 5.00 | 0.957 / 5.00 |
| passages-any-0p25 | 0.882 / 5.00 | 0.864 / 5.00 | 0.882 / 5.00 | 0.864 / 5.00 |
| passages-any-0p5 | 0.852 / 5.00 | 0.823 / 4.00 | 0.852 / 5.00 | 0.823 / 4.00 |
| passages-any-0p75 | 0.852 / 5.00 | 0.786 / 4.00 | 0.852 / 5.00 | 0.786 / 4.00 |
| passages-any-1 | 0.834 / 5.00 | 0.740 / 4.00 | 0.834 / 5.00 | 0.740 / 4.00 |
| passages-corresponding-0 | 0.866 / 5.00 | 0.870 / 5.00 | 0.866 / 5.00 | 0.870 / 5.00 |
| passages-corresponding-0p25 | 0.571 / 3.50 | 0.694 / 4.00 | 0.571 / 3.50 | 0.694 / 4.00 |
| passages-corresponding-0p5 | 0.536 / 3.00 | 0.677 / 4.00 | 0.536 / 3.00 | 0.677 / 4.00 |
| passages-corresponding-0p75 | 0.538 / 3.00 | 0.651 / 4.00 | 0.538 / 3.00 | 0.651 / 4.00 |
| passages-corresponding-1 | 0.512 / 3.00 | 0.647 / 4.00 | 0.512 / 3.00 | 0.647 / 4.00 |
