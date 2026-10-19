# Independent projection/fusion comparison audit

Audited 2026-10-18 at HEAD `1a95060ea5af1ff8fe2aed617662257a349cd17a`. Scope: `projection-fusion-followup.ts`, `resolve-matrix.ts`, `measure-matrix.ts`, `matrix-contrasts.ts`, their helper contracts and resulting research artifacts. Only this audit was written. Checks used read-only captures, source files and a read-only database connection; no production edits, tuning, builds, retrieval, inference or network access occurred. No new source relevance judgments were made.

**Disposition: pass for the bounded exploratory comparison. No blocking parity, review-coverage, pool-consistency or statistical-arithmetic defect was found.** These results support comparison of the specified lexical-policy and fusion bundles on the reused corpus. They do not establish an untouched holdout result, production latency or deployment acceptance.

## Ranking parity and comparison design

I replayed all 288 context/profile captures using the captured lexical, semantic and field-unit inputs and read-only identity/remaster relationships. All **1,440 saved full ranking lists** matched the recomputed lists, including metadata and scores, not merely top-ten keys. Every available previously captured comparison matched exactly: **1,152 full-ranking parity checks**.

| Strategy | Previous full captured lists matched |
|---|---:|
| baseline/current lexical + RRF60 | 288 |
| current lexical + minmax alpha 0.25 | 144 |
| current lexical + minmax alpha 0.5 | 288 |
| coherent25 lexical + RRF60 | 144 |
| coherent25 lexical + minmax alpha 0.25 | 288 |

The 144-list entries have previous development captures; their holdout lists are newly computed from the fixed captured lanes. The other three strategies retain all prior development and holdout lists exactly. No prior ranking keys or ranking scores changed. The independently rebuilt union of new top-ten query/record pairs exactly matches the 14-entry missing-judgments file.

The five-strategy matrix implements the intended matched comparison:

- `projection-with-rrf` compares current versus coherent25 lexical policy with the same RRF60 fusion.
- `projection-with-score` compares current versus coherent25 lexical policy with the same full-lane minmax normalization and **alpha 0.25**. It does not substitute the alpha-0.5 reference for the matched current-side comparator.
- `fusion-with-field-aware` holds coherent25 lexical input fixed while comparing RRF60 with minmax alpha 0.25. This changes the fusion policy, including rank-versus-score treatment and effective lane weighting; it does not isolate normalization alone.
- Current minmax alpha 0.5 is retained as a reference strategy, rather than being used to attribute a lexical-policy difference.

The contrast names beginning `projection` are shorthand for a **lexical-policy bundle**, not pure textual projection. The coherent lane selects the strongest weighted coherent unit (root 1, section 0.5, embedded 0.25), uses the captured field-aware BM25 evidence, and assigns exact root/alias tiers ahead of tier 3. The current lane also has an embedded-exact tier 2. Thus representation, attribution/conjunction behavior, lexical tiers and weights move together. These results cannot identify the contribution of any one of those components. Fusion still operates on full lanes before the common edition suppression.

## New source-review completeness and resolution

The review packet, added private pool and both grade files contain exactly the same **14 unique IDs**. All added IDs and query/record pairs are new relative to the original pools, with no collisions or duplicate keys. All are holdout pairs across six contexts: `hold-07-u` (3), `hold-08-u` (4), `hold-09-u` (3), `hold-11-u` (1), `hold-20-u` (2) and `hold-22-u` (1).

Both reviewer audits attest separate source-only review without access to other opinions or evaluation materials. I checked their output coverage, every added source-file SHA256, exact review/private-pool metadata correspondence, and the unchanged rubric: local `GRADING.md` is byte-identical to the revised relevance rubric. I also reran the shared validator with **node**, independently for `grades-a.json` and `grades-b.json`; both passed all 14 IDs, grades/flags and literal pointer/quote checks. This establishes output/evidence integrity and recorded independent review, not an additional relevance adjudication by this audit.

Opinion pairs are eleven 2/2, one 3/2, one 1/1 and one 3/3. Both reviewers mark every opposite-function flag false. There are no null grades/flags, grade distances of two or more, usefulness-boundary crossings or opposite-flag disagreements. The pending-adjudication file is correctly empty. The sole 3/2 difference is Mage of Many Styles for `vampiric`; both opinions are retained, with mean gain 5 and endpoint gains 3 and 7, rather than rounded into artificial agreement.

I independently rebuilt all 14 added utilities and compared every saved field. They match. The combined result preserves **all 2,925 prior resolved records and the prior unresolved array exactly**, adding 14 numeric records: **2,939 resolved plus one unresolved, covering 2,940 pooled pairs**. In particular, earlier source-followup/carry revisions and the qualitative null alternative remain intact. No unresolved value is coerced to zero.

## Common enlarged pool and metric recalculation

All five strategies use one enlarged query relevance pool, not a strategy-specific top-ten pool. Its split totals are 1,570 development pairs and 1,370 holdout pairs. Every included ranked top-ten entry joins successfully to its query pool; pool and ranking keys are unique.

The exclusions are exactly the previous exclusions in every mode: development retains 48 clusters/72 contexts, while holdout excludes the entire `hold-41` cluster and retains 47 clusters/70 contexts. Both `hold-41-u` and `hold-41-f` remain excluded for both profiles and every strategy. The same unresolved source case determines the omission; the 14 additions do not weaken it. Each saved metrics file has `hypothetical:null`.

Using a separate Python implementation, I recalculated **all 4,260 rows and 300 summaries** across mean/low/high. Every row field and summary value matched within floating tolerance `2e-12`. The checks covered pooled nDCG over the entire enlarged query pool, useful count, both success endpoints, opposite fraction, reference top-one/ranks, and context-within-cluster then equal-cluster summary weighting. Each mode has 1,420 rows and 100 summaries, with no zero-ideal/null nDCG rows.

## Why unchanged baselines have different numeric nDCG

The 14 added candidates enlarge the shared ideal-ranking denominator. For the three strategies present in both the earlier metrics and this matrix, **24 shared rows per mode** change nDCG: four contexts times two profiles times three strategies. The affected contexts are `hold-07-u`, `hold-08-u`, `hold-09-u` and `hold-11-u`. Eight of those rows per mode belong to baseline. All other shared row metrics and all reference ranks remain unchanged.

For every changed shared row I checked that its DCG numerator is unchanged and that `new nDCG = old nDCG * old IDCG / new IDCG`. Example: baseline/bge-base for `hold-07-u` keeps its entire ranking but changes from `0.7355440378079012` to `0.6536043231224955`, because IDCG rises from `15.152891950063175` to `17.05254836162926`. Baseline holdout/all mean nDCG therefore moves from `0.7251066490288628` to `0.7177989493289345` for bge-base, and from `0.814703571582378` to `0.8075501181570454` for embeddinggemma. This is a measurement-pool change, not a retrieval regression. Comparisons must use strategies scored inside the same enlarged pool.

## Contrast and bootstrap arithmetic

I independently rebuilt all **36 contrast reports**, including their context pairing, within-cluster mean deltas, wins/losses/ties (tolerance `1e-9`), and cluster bootstrap. Every contrast mean and every interval matches the stored output within `2e-12`. The bootstrap resamples whole paired clusters with replacement, using 10,000 draws and the specified seed/LCG; sorted indices 250 and 9,749 give the saved percentile endpoints. Development uses 48 clusters and holdout uses 47.

The salient holdout mean scenarios reproduce as follows. Deltas are absolute nDCG, candidate minus comparator.

| Profile | Contrast | Mean delta | Bootstrap interval | Wins/losses/ties |
|---|---|---:|---|---|
| bge-base | lexical policy with RRF | +0.006982 | [+0.001003, +0.014659] | 9/2/36 |
| bge-base | lexical policy with matched score alpha 0.25 | +0.008302 | [+0.000644, +0.017664] | 8/2/37 |
| bge-base | fusion with coherent25 lexical policy | +0.001782 | [-0.007120, +0.010589] | 6/6/35 |
| embeddinggemma | lexical policy with RRF | +0.008654 | [+0.001184, +0.017873] | 9/2/36 |
| embeddinggemma | lexical policy with matched score alpha 0.25 | +0.006553 | [-0.000835, +0.015961] | 7/2/38 |
| embeddinggemma | fusion with coherent25 lexical policy | +0.001516 | [-0.017847, +0.021984] | 7/6/34 |

These are exploratory, post-hoc paired bootstrap intervals on reused clusters, without multiplicity adjustment. The two coherent-side fusion intervals cross zero; the arithmetic does not establish a universal fusion winner. Grade-choice low/high files are endpoint scenarios, not nDCG bounds or statistical confidence limits: changing gains also changes the ideal denominator. The retained unresolved/qualitative source uncertainty is not exhausted by these numeric scenarios. This audit makes no policy-selection or production-readiness decision.

## Snapshot fingerprints

Files below are relative to `scratch/search-fusion-review/`. Later changes require checking the changed snapshot again.

| File | SHA256 |
|---|---|
| `harness/projection-fusion-followup.ts` | `b2d74199e00f136e82f1cd76e8df90a07ebe7eab114a6af80833b5f9f0555a62` |
| `harness/resolve-matrix.ts` | `d49b1a1eea1fde94f725af2cddeff7c20bda1b4406d7df302b79bfbf52ee61ba` |
| `harness/measure-matrix.ts` | `e5dbfdb562cbae5359fd87c4108616cf4f9e32133ee3d297f35a8aa62d72323f` |
| `harness/matrix-contrasts.ts` | `1cae51a5378925b455dd0c303b59274291e8c5f2fd0b75e015611c32e8ea6b02` |
| `projection-fusion-followup/PROTOCOL.md` | `0997bb7da5fa14ea9d70efb1c24f8f2baf79adef36685235106728a9fd09a676` |
| `projection-fusion-followup/GRADING.md` | `d291b2ed2008cfcf310ad9501a4cccef4a2553aed488b8bd7d1bcb7e56af9275` |
| `projection-fusion-followup/rankings.json` | `d70f0116f0e199035d025abddec1a993be6aaa1f043476afee0c2304162adc3a` |
| `projection-fusion-followup/missing-judgments-private.json` | `6e1fbe98c0d7d84b951df23d94122b07ec021e9880c52888ac5b10112907413c` |
| `projection-fusion-followup/pool-private.json` | `838e365697da2439507feed321a3b1f7e452b74396c415ef147cd8758d31b273` |
| `projection-fusion-followup/review.json` | `ec7a4f2170a578bdd404935814332aba8531e939571686d10c2e4a58c5df20b3` |
| `projection-fusion-followup/grades-a.json` | `18275908dc3bffd8118d8c753e63e8a04f1c41803656ab576495364bd45d4587` |
| `projection-fusion-followup/grades-b.json` | `194be58e425283e35e10a068c6d4c1536750f28c837c06ed2dbc5cfec996fe0d` |
| `projection-fusion-followup/pending-adjudication.json` | `4f53cda18c2baa0c0354bb5f9a3ecbe5ed12ab4d8e11ba873c2f11161202b945` |
| `projection-fusion-followup/resolved-utilities.json` | `97a9d52cef3bb9ee336e17d5c00b1911215d2fbc2b4b966b6a55f2437744de11` |
| `projection-fusion-followup/metrics-mean.json` | `3eebdfc609dedcb633b51bcab55504d4e3d05540fa44a8629b82367118e11100` |
| `projection-fusion-followup/metrics-low.json` | `473b32ee1284fda73785791486f7708648fe4b09bc5761578ac5963cb2ae55cb` |
| `projection-fusion-followup/metrics-high.json` | `d748d4cf73d01ea2440d8bbe3859fc80935cb18d3408aa0f3038733f29b9c9b3` |
| `projection-fusion-followup/contrasts.json` | `4eac47e225cc7346380b2a6b9ca374289cd5f49fdfbf09351f0e5a2e14ea1790` |
