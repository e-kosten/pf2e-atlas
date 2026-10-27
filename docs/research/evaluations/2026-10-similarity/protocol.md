# Stored-vector similarity protocol v1

Status: inputs and rules frozen before ranked-result inspection. The frozen
files are identified by `freeze.json`; corrections create a new version rather
than silently editing an opened experiment.

## Questions and scope

Measure numerical correctness, candidate recall, ranking usefulness, explanation
support and runtime cost separately. Compare the unchanged legacy baseline,
Passage matches and both Content average recipes over the same artifact and
stored embeddings. No new inference, source re-ingest, models, chunk boundaries,
role changes, scoring cutoff, summaries, rerankers or candidate-budget tuning is
part of this protocol. UI hiding of weights rounding to 0.0% is display-only.

Source interpretation remains in the checked record policy; SQL remains in
index, scoring in search, coherent workflows in app-service and developer
experiment orchestration in atlas-dev. Use exact stored selectors and source
addresses; never derive a new field selection from prose in the evaluation tool.

## Frozen inputs and exposure

The manifest has six scenarios each for hazards, creatures, spells and traits.
Each family has three development and three holdout scenarios. There are 22
unique seeds: Brown Mold and Lich each support two deliberately related tasks.
Those pairs are one cluster each, not independent extra samples.

Development includes every scenario using the previously discussed fire, cold
fungus, arcane undead and restoration examples. Holdout has three disjoint theme
clusters: aquatic terrain, grasping plants and sonic pressure. Each holds one
hazard, creature, spell and trait task. Related entity variants and themes are
never split between development and holdout. No old rankings or judgments were
used to select this holdout. Familiarity with the game/source is not treated as
proof that the entities are globally unseen.

Source inspection before freeze grounds task intent and selection; it does not
open retrieval results. The source-bound inventory includes all stored part
addresses, hashes, roles and vector availability for both profiles. Holdout is
opened only after this protocol, variants, corpus keys, selections, source
bindings, failure rules and metric rules have passed integrity review. Once
opened, the holdout is exposed: subsequent tuning requires new independent cases
for a confirmatory quality claim.

`trait:amp` is a known catalog label with no stored definition and an explicit
stored label-only vector in both profiles. Its scenario is diagnostic-only.
Do not invent a definition or force semantic grades from exact trait membership.
Its usefulness remains unknown unless an independently sourced intent is
established in a separately versioned experiment. It stays in the six-trait
execution stratum but is excluded from semantic quality means uniformly.

## Variant matrix and comparison strata

Both `bge-base` and `embeddinggemma` run:

- legacy baseline;
- Content average with Any content and, for record seeds, Corresponding content;
- Passage matches at decay 0, 0.25, 0.5, 0.75 and 1 with Any content and, for record
  seeds, Corresponding content.

This gives 552 applicable scenario/profile/variant cells: 18 record tasks × two
profiles × 13 variants, plus six trait tasks × two profiles × seven variants.
Trait Corresponding variants are inapplicable and must be reported as skipped,
not silently replaced with Any content or counted as failed quality trials.
Repeated calls with a single trait part remain distinct declared configurations,
although equivalent scores can be reported as an equivalence observation.

Report three strata separately:

1. Whole-record/default comparison: 12 scenarios. The baseline's identity seed is
   intentionally compared with whole selected prose as alternative representations
   for the same whole-record discovery task.
2. Whole-trait/default comparison: six execution scenarios, five quality-eligible
   scenarios; record and trait default decisions are independent.
3. Section-focused advanced comparison: six scenarios. Only the new methods can
   honor exact selections. Baseline runs are contextual legacy references and
   cannot be penalized as if they honored those selected sections. Do not include
   them in a matched-selection advanced-method quality contrast.

The 24 candidate filters are null, preserving cross-kind discovery. Corresponding
content constrains semantic roles, not record kind. Do not add an implicit kind
filter during grading. A strong cross-kind analogy can be useful when supported
by the task, while matching one shared trait is never automatically sufficient.

## Candidate universe and experiments

The frozen manageable oracle is 278 explicit product roots. Select 60 each from
hazards, creatures, spells and other product kinds using Atlas's identity-only
browse with `--sort random --seed 20261026 --limit 60`. The other stratum uses
`kind != "hazard" && kind != "creature" && kind != "spell"`. Add the 16 unique
record scenario seeds and 22 independently source-nominated roots before ranking.
The union contains 278 keys. This universe is deliberately mixed, reproducible
and enriched for meaningful comparisons; it is not a random representative sample
of all 29,813 product roots. Nominations are neither gold judgments nor guaranteed
positive matches. A record seed itself remains in the manifest universe for
other tasks but is excluded from its own eligible candidates.

Run two complementary comparisons without changing the declared variants:

- Fixed pool: union each applicable variant/profile's top 20 bounded results per
  scenario, deduplicate exact root keys, then directly score every eligible key
  in that same pool under each method. This isolates ordering from retrieval.
  Record the union identity and method failures before grading; do not fill
  failed variants from another method or fabricate absent scores.
- Controlled oracle: directly score every eligible frozen corpus key, bypassing
  KNN admission. Compare it with the normal bounded retrieval pipeline constrained
  to the identical corpus and filters. Both controlled arms use
  `prefer_remaster=false`, so eligible old/new editions remain explicit keys in
  a stable universe. Report that controlled eligibility separately from the
  production full-corpus pipeline, which retains its current remaster behavior.

Full-corpus captures and controlled-oracle results have distinct scope labels.
The controlled experiment establishes recall only on the declared 278-key
universe. Filtering KNN to those keys does not itself establish an exhaustive
oracle; the maintained direct-scoring path is required. Match existing baseline
and new-method algorithms, including positive-evidence handling, role masks,
missing-role zeros, overlap weights, source ownership and profile isolation.
Do not substitute another score because a centroid or seed is unavailable.

Candidate generation is fixed across decay values. Scoring preparation may be
shared across decay variants when mathematically equivalent; record reuse and
do not report prepared-scorer timing as a fresh complete retrieval request.
Explanation limits and UI visibility never affect ranking.

## Relevance packets and judging

Construct a randomized, deduplicated union of every compared top-ten result for
each scenario. Assign stable experiment-local blind IDs and freeze the pool,
randomization seed, source bindings and packets before judging. Give each judge:

- the task intent, exact selected seed content, source availability and whether
  it is a whole or section-focused task;
- the candidate's identity and sufficient source content/owner context to assess
  its intrinsic concept and supported capabilities;
- exact source addresses and hashes so supporting passages can be checked.

Hide configuration names, model names, scores, ranks, method provenance and the
other judge's opinions. Candidate titles are visible because root identity is
part of source-grounded usefulness. Raw source excerpts stay in ignored local
packets. Curated tracked opinions retain paraphrased rationale, pointers and
quote digests instead of republishing source bodies.

Use two independent model judges. Record judge model/version, task instructions,
packet digest and each unmodified opinion. These are model-assisted opinions,
not human gold labels. Human UX acceptance remains a separate evidence state.
Do not let judges see retrieval identities before submitting their decisions.

Root usefulness grades are ordinal:

| Grade | Meaning |
| --- | --- |
| 3 | Strongly useful: the candidate's own concept or supported capability closely satisfies the requested mechanism/purpose and its important consequences. |
| 2 | Useful: an authored, supported analogy satisfies a salient aspect, with meaningful differences or incomplete coverage. |
| 1 | Weak: superficial wording, a minor secondary association or an associated-only overlap gives limited help for this task. |
| 0 | Irrelevant: available authored evidence supports no useful connection or contradicts the requested mechanism. |
| unknown | Source or intent is insufficient to judge; never convert missing evidence into grade 0. |

Separately record evidence support as supported, partial, unsupported or unknown,
and connection class as intrinsic, owned-capability, associated-only, incidental,
none or unknown. Source ownership and task intent matter: an owned spell can be
a real capability when the task asks for it. Conversely, copied spell/item text
alone is not evidence that a root shares the lich's intrinsic undeath mechanism.
A carried item's trait or label does not prove the owner can use every capability
mentioned in that item's catalog description. Keep root usefulness and supporting
excerpt fidelity distinct.

Each opinion includes scenario ID, blind candidate ID, grade or unknown,
evidence-support status, connection class, uncertainty, concise rationale and
source pointers. Keep disagreement visible. A two-grade difference, supported
versus unsupported evidence, or a material intrinsic/associated-only dispute
requires independent source-based adjudication. Preserve both opinions and the
adjudication; do not average consequential disagreements away. Unresolved cases
stay unknown and enter the uniform missingness rules below.

## Metrics and decision rules

Compute nDCG@10 using gains 0, 1, 3 and 7 for grades 0 through 3; useful@5 counts
grades at least 2. Also report method overlap/diversity and pairwise preferences
from matched source-grounded opinions. The ideal ordering for pooled nDCG is the
judged pool, so label it pool-relative rather than corpus-wide optimal usefulness.

Report the number of judged/unknown candidates and affected top-ten positions per
scenario and configuration. Unknown is never zero. Use one consistent complete
case set across compared configurations for the primary contrast, or report
explicit grade-range sensitivity that treats unknown grades uniformly at both
bounds. Do not drop unknown candidates for only one method. A scenario with no
positive gain has undefined nDCG; report it and its count instead of silently
assigning 0 or 1.

Candidate-admission recall@10 is the fraction of exhaustive oracle top-ten roots
present in the bounded pipeline’s complete admitted root-key union before
reranking, under the same method/profile/selection/scope. Its denominator is the
number of usable oracle roots up to ten. The provisional gate is at least 95%
candidate-admission recall@10 on the defined subset. Separately report the final
bounded returned-top-ten agreement with the oracle, which also reflects ordering
and result presentation. Never substitute final returned keys for admitted keys.
The legacy baseline does not currently expose its complete admitted union, so
its admission metric is unavailable; report only its explicitly labeled returned
top-ten agreement. Report every admission miss by scenario class, including
exact score-boundary ties. Preserve existing arbitrary KNN cutoff ties;
record deterministic output tie order and distinguish tie-boundary alternatives
from strict higher-score misses. Do not claim physical-layout-independent KNN
membership. Empty usable oracles, classified failures and unsupported cells have
separate denominators, never automatic perfect recall.

Report per scenario, kind, comparison stratum and theme cluster. For the primary
whole-record default contrast, average related tasks within each kind/theme
cluster, average those clusters within kind, then give hazard, creature and spell
kinds equal weight. The whole-record stratum is not kind-balanced: it contains
three hazard, three creature and six spell tasks (holdout: two, one and three).
Publish those counts and per-kind results, plus equal-scenario descriptive means;
never present the overall 24-case family balance as balance within this stratum.
Trait default and advanced selected-section contrasts remain separate. Related
Brown Mold/Lich tasks do not add independent votes. Holdout has only three theme
clusters and 12 scenarios; any
cluster-resampling interval is exploratory and must not imply a large independent
sample or definitive generalization. Keep model judge disagreements and low/high
unknown sensitivity separate from sampling intervals.

A new default requires a documented holdout usefulness advantage without a
material regression in an agreed scenario class, acceptable measured latency,
and separate human UX acceptance. Choose record whole-seed and trait defaults
independently; never choose a record default by pooling advanced selected-section
or trait tasks. If evidence is inconclusive, keep the baseline as the shipping
reference and retain the new methods as experiments.

## Performance and failure handling

Freeze the declared repeated subset before results: Brown Mold whole, Striding
Fire whole, Lich whole, Heal whole and the Fire trait. For both profiles, measure
baseline, Passage Any at decay 0.5 and Content average Any. Additionally measure
Lich Passage Corresponding at decay 0.25 to cover the user's current complex
workflow even if Any content hits a resource ceiling.

A cold application-context sample is the first complete request in a fresh
service process/context; OS file caches are uncontrolled and never flushed.
Warm samples are 20 repeated complete requests in the same service process and
context, including normal candidate retrieval, vector reads, scoring and evidence
hydration. Do not label a new CLI process per sample as warm, or a reused prepared
scorer as complete warm retrieval. Report scorer preparation reuse separately.

Capture phase timings and counts for part inventory, vector reads, candidate
queries, pair comparisons, evidence hydration and end-to-end latency; capture
process peak RSS and relevant cache/vector counts. Report samples, p50/p95
convention, resource caps and the exact process/context lifecycle. Measure one
active request plus declared cancellation/concurrency probes separately from the
primary single-request timing distribution.

Development targets frozen before result inspection are warm full-pipeline p95
at most 5 seconds for common seeds, at most 20 seconds for the complex 38-section
lich and process peak RSS at most 512 MiB. These are provisional development
budgets, not shipping SLOs. Report an existing target miss rather than moving the
gate after looking at the holdout or silently reducing input scope.

Structural/option/seed-unavailable/resource-limit failures are explicit classified
outcomes. After a deterministic cold failure, do not repeat it 20 times to produce
a meaningless successful-looking percentile. Report failure count and no warm
p95 for that cell. Never replace the difficult scenario, silently truncate a
selected section, fall back to another model/method, grade missing results as
irrelevant, or report a failed query's short duration as favorable latency.

No product optimization begins from holdout observations. Tuning uses development
cases; changes require a new app revision/run identity and uniformly recomputed
comparisons. The frozen holdout cannot become new development while retaining a
confirmatory label.
