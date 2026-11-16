# Semantic-search heap benchmarks

See [scope, exactness and the decision to stop](README.md) for corpus and build pins. All estimates are historical, paired and descriptive. Each arm median is shown separately. A paired percentage uses the stock latency in that pair, and the table reports the median of those percentages. OS file caching was uncontrolled.

## Stock phase profile

| Profile | Fireball Hybrid/All stock wall | Diagnostic selector | Share of diagnostic wall | Median conservative share |
|---|---:|---:|---:|---:|
| BGE Base | 358.21 ms | 112.75 ms | 31.8% | 30.4% |
| EmbeddingGemma | 379.54 ms | 113.29 ms | 30.2% | 27.2% |

The screen required a median of at least 25 ms and 20% of the complete request in the same cell for both profiles. At least three of four pairs per profile had to meet both thresholds under the lower sensitivity calculation. Fireball cleared the screen in four of four pairs on both profiles. Stock selection time bounds how much a change confined to the selector could save on this fixture.

## Two-profile release A/B

The main gate required at least 15% median paired wall improvement on Fireball, with all six pairs faster per profile. Each other scored cell required median heap-minus-stock wall time after tolerance at or below zero and at least five of six pairs within tolerance. Tolerance was the larger of 2 ms and 5% of the paired stock wall time. Warmup was reported separately. The selective Gemma miss was the only unresolved cell. The gate result was **hold**.

`name` is Fireball Hybrid/All, `broad` is `fire` Hybrid/Core, and `filtered` is `fire` Hybrid/All with `kind=spell`. `baseline` is the explicit Baseline Similar method for Fireball. Default Similar seeds were Fireball (small) and Ogre Warrior (medium). The lexical controls were the Fireball All and `fire` Core counterparts. Repeat wall and CPU times were reduced to a median per process before pairing.

| Profile | Cell | Stock ms | Heap ms | Paired Δ ms | Paired Δ % | CPU Δ ms | Passing pairs | Verdict |
|---|---|---:|---:|---:|---:|---:|---:|---|
| bge-base | baseline-hit | 20.90 | 20.26 | +0.13 | +0.51% | +0.10 | 6/6 | pass |
| bge-base | baseline-miss | 326.39 | 221.91 | -104.77 | -32.03% | -104.75 | 6/6 | pass |
| bge-base | broad-hit | 50.34 | 46.48 | -3.33 | -6.51% | -3.25 | 6/6 | pass |
| bge-base | broad-miss | 236.00 | 179.34 | -57.48 | -24.71% | -56.81 | 6/6 | pass |
| bge-base | filtered-hit | 33.92 | 33.84 | -0.00 | -0.00% | -0.08 | 6/6 | pass |
| bge-base | filtered-miss | 80.88 | 76.34 | -5.11 | -6.03% | -4.34 | 6/6 | pass |
| bge-base | lexical-all | 9.43 | 9.24 | +0.14 | +1.25% | +0.15 | 5/6 | pass |
| bge-base | lexical-core | 13.37 | 13.17 | -0.31 | -2.28% | -0.30 | 6/6 | pass |
| bge-base | medium-all | 1988.09 | 1789.13 | -195.38 | -9.85% | -192.51 | 6/6 | pass |
| bge-base | medium-core | 1210.96 | 1017.99 | -194.20 | -16.02% | -194.67 | 6/6 | pass |
| bge-base | name-hit | 43.65 | 41.85 | -1.49 | -3.40% | -0.18 | 6/6 | pass |
| bge-base | name-miss | 360.27 | 251.36 | -110.87 | -30.79% | -109.78 | 6/6 | pass |
| bge-base | small-all | 769.90 | 697.82 | -70.87 | -9.23% | -70.58 | 6/6 | pass |
| bge-base | small-core | 517.98 | 439.86 | -77.03 | -14.87% | -76.92 | 6/6 | pass |
| bge-base | warmup | 1323.38 | 1187.21 | -121.53 | -9.17% | -119.38 | 6/6 | separate |
| embeddinggemma | baseline-hit | 18.67 | 18.80 | +0.10 | +0.52% | +0.10 | 6/6 | pass |
| embeddinggemma | baseline-miss | 327.04 | 222.40 | -106.89 | -32.63% | -106.97 | 6/6 | pass |
| embeddinggemma | broad-hit | 57.18 | 58.19 | +0.19 | +0.35% | +1.67 | 5/6 | pass |
| embeddinggemma | broad-miss | 251.42 | 195.67 | -54.26 | -21.81% | -52.72 | 6/6 | pass |
| embeddinggemma | filtered-hit | 46.95 | 46.01 | -0.48 | -1.03% | -0.83 | 6/6 | pass |
| embeddinggemma | filtered-miss | 94.52 | 96.78 | +1.16 | +1.17% | +11.52 | 4/6 | unresolved |
| embeddinggemma | lexical-all | 9.57 | 8.26 | -0.57 | -4.75% | -0.58 | 6/6 | pass |
| embeddinggemma | lexical-core | 13.99 | 13.21 | -0.09 | -0.62% | -0.13 | 6/6 | pass |
| embeddinggemma | medium-all | 1948.89 | 1815.83 | -149.95 | -7.63% | -148.61 | 6/6 | pass |
| embeddinggemma | medium-core | 1172.81 | 974.36 | -193.93 | -16.60% | -194.15 | 6/6 | pass |
| embeddinggemma | name-hit | 56.93 | 55.74 | -1.68 | -2.88% | -0.71 | 5/6 | pass |
| embeddinggemma | name-miss | 379.80 | 265.83 | -111.29 | -29.54% | -109.88 | 6/6 | pass |
| embeddinggemma | small-all | 752.55 | 730.59 | -16.56 | -2.24% | -17.42 | 6/6 | pass |
| embeddinggemma | small-core | 515.88 | 446.10 | -71.02 | -13.89% | -70.85 | 6/6 | pass |
| embeddinggemma | warmup | 5899.27 | 5804.23 | +12.18 | +0.23% | -58.85 | 6/6 | separate |

## Selective EmbeddingGemma follow-up

| Sequence before the selective miss | Stock wall | Heap wall | Paired wall change | Paired process CPU change | Within tolerance |
|---|---:|---:|---:|---:|---:|
| Warmup | 99.74 ms | 95.82 ms | −3.40 ms (−3.43%) | −2.16 ms | 6/6 |
| Warmup and four Similar requests | 99.72 ms | 98.38 ms | −1.63 ms (−1.64%) | −2.25 ms | 4/6 |

Both conditions used the same wall guard as above. The median excess after Similar was −6.63 ms, so the two exceedances left that condition unresolved. The repeatable-lead thresholds were +10 ms for process CPU and +512 MiB for current or peak RSS, with at least five of six positive paired deltas at the named checkpoint. The CPU interaction was +1.97 ms (range −4.83 to +32.99 ms, positive in 4/6). The lexical control changed by −0.28 ms (warmup only) and −0.27 ms (after Similar). The two retained repeats changed by +0.17 to +0.54 ms across the four condition/repeat medians.

| Checkpoint in the processes assigned to Similar | Paired current RSS change | Paired peak RSS change |
|---|---:|---:|
| Before warmup | +0.14 MiB | +0.11 MiB |
| Immediately after warmup | +5,548.18 MiB | +580.05 MiB |
| After Ogre Warrior Core | +5,549.61 MiB | +631.99 MiB |
| After Ogre Warrior All | −2,081.65 MiB | +657.54 MiB |
| After Fireball Core | −2,078.17 MiB | +657.54 MiB |
| After Fireball All | −2,075.04 MiB | +657.54 MiB |
| After the selective miss | −2,075.12 MiB | +657.54 MiB |
| Closeout | −2,075.91 MiB | +657.54 MiB |

Both differences immediately after warmup were positive in all six pairs in the processes assigned to Similar. The corresponding warmup-only medians were −34.14 MiB current and −23.13 MiB peak. The peak RSS interaction was +581.73 MiB (5/6 positive) immediately after warmup and +658.97 MiB (6/6 positive) after the selective miss. The median change in that interaction from post-warmup to the miss was +75.66 MiB. SQLite requested-memory peak changed by +0.27 MiB at the selective miss after Similar. Darwin resident size and high-water encompass the whole process and do not establish allocation ownership.
