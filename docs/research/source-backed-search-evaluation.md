# Source-backed Search Evaluation Notes

Pinned PF2e 6.12.4 local evaluation, recorded 2026-10-13. This preserves the dated findings and evidence limits from the research backlog. Its stage descriptions and proposals are historical; current work is in the linked backlog item.

Start with the [cross-experiment synthesis](search-evaluation-synthesis.md) and
[curated grading/ranking evidence](evaluations/2026-10-search/README.md).
Open work: [backlog item](../backlog/items/rust-search-quality-tuning.md).

## Lexical and fusion decision, 2026-10-18

The subsequent full-corpus comparison supports one field-aware lexical policy:
separate identity/alias rows, no inherited parent terms in embedded rows, and
strongest-row root aggregation. The small preference between RRF and fixed
min-max score fusion reverses under judgment sensitivity analysis. Both remain
available for dogfooding, independently of embedding model; the user selected
Score-based as default after the functional preview. This is not a claim of a
universal fusion winner. Current behavior is defined by
[ADR0053](../architecture/decisions/0053-field-aware-search-fusion.md).

Local evidence: `scratch/search-fusion-review/projection-fusion-followup/FINDINGS.md`
and `AUDIT.md`, with the revised source-backed grading under
`scratch/search-fusion-review/relevance-realignment/`. The matrix audit reproduced
1,152 prior ranking comparisons and all summary arithmetic. Preserve grading,
adjudications, query sets and policy configurations for later analysis. These
holdouts are now exposed; future policy tuning needs fresh evaluation cases.

The sections below retain earlier dated research context, not current defaults.

## Historical baseline (2026-10-13)

ADR 0046 governs the source-backed artifact and search policy. Precision FTS uses
identity, verified aliases, typed vocabulary and named definitions. Semantic units
independently cover selected root and owned prose. One maximum accepted unit score
represents a root; hybrid RRF combines one root rank per lane, with specificity
tiers for exact identity/alias/owned-label evidence before fusion score. Eligibility
applies before ranking, and semantic candidate windows are bounded.

The full pinned PF2e 6.12.4 artifact has now been built and deeply validated, with
25,560 product roots and 130,566 semantic units. Local evidence is under
`scratch/full-corpus-search/`: frozen source-backed queries, raw CLI results,
independent judgments, human/JSON navigation checks and latency receipts. This
focused sample is not an exhaustive search-quality certification. Independent
evidence audit passed; policy recommendations remain local research awaiting
user review before implementation.

## Follow-up decisions

The local hierarchy comparison now covers eight policies on the same frozen 24
intents and unchanged embeddings. All baseline top 10 orders and candidate-root
counts replay exactly. Source review covers 228 unique query/root pairs across all
policy top 5 results. Lower owned semantic influence improves descriptive discovery
but damages unique owned abilities, primary journal-page content and deliberately
requested carried-spell capabilities. Removing owned FTS alone does not fix the
descriptive cases. See `scratch/full-corpus-search/hierarchy/README.md` for measured
tradeoffs and evidence limits; no experimental policy has replaced product ranking.

The subsequent research inventoried primary and associated content and compared
role-aware ranking policies. Ownership depth and compendium origin alone cannot
define those roles. Exact local definitions and capability searches constrain any
weight changes; production implementation still requires user review.

The content-role inventory now reconciles every existing lexical/semantic unit to
its pinned source node/field and checks all 25,641 source hashes. The proposal in
`scratch/full-corpus-search/content-roles/PROPOSAL.md` distinguishes root/intrinsic
explanation from named components and associated bodies. Nested spells inside root
consumables differ from the same spells inside carried items. Build-option/status
bodies and attached subitems remain explicit policy-review categories.

A local ignored review harness piloted 12 curated development query/root pairs
with three fresh Luna reviewers (36 votes). It preserves ballots, abstentions,
packet hashes and literal source citations, separating root relevance from witness
support. The pilot revealed usefulness-boundary disagreement; source adjudication
does not use numerical majority as truth or overwrite earlier grades. It is not a
policy-quality or judge-reliability estimate. Actual prepared-snippet packets and
rubric anchors are required before scaling beyond calibration.

Broader evaluation intent is 48 development/24 new held-out scenarios, stratified
by task/family and split by concept/target clusters rather than paraphrases. Freeze
source-grounded queries before retrieval, preserve non-exhaustive positives and
source-authored bias, and expose holdout results only after bounded candidate
policies are fixed. Any later tuning needs a new held-out cycle. Classification
review and this evaluation protocol precede new policy tests; production behavior
and the successful artifact remain unchanged.

The role-aware development comparison is complete: nine declared hypotheses on 48
frozen queries, with two panels covering 479 root/query pairs and 1,872 displayed
witnesses each. Source review verifies canonical spell/item discovery gains,
genuine character-owned capability admission losses under support-only treatment,
and flavor regressions caused by adding separate primary and associated ranks.
Three unchanged diagnostic policies were frozen before opening the 24-case holdout;
all 72 development/holdout baseline outputs replay exactly.

The independent annotation audit found wrong answer-kind credit, missed qualifiers
and evidence credited beyond displayed snippets, including unanimous errors.
Raw witness aggregates do not select a policy; raw root aggregates remain
exploratory. Consequential source checks and explicit uncertainty are required.
The holdout's capability-labeled cases concern standalone options, so they cannot
establish protection for genuine associated-only owner searches. Local evidence
and the source-bounded recommendation are in
`scratch/full-corpus-search/role-evaluation/FINDINGS.md`.

Do not adopt a blanket associated-content exclusion or a by-kind formula from this
comparison. A reduced-associated, single-semantic-contribution candidate requires
a new frozen experiment with real ownership positives. Changed embedding inputs
(primary prose plus associated names, or typed gameplay facts) remain untested and
need a separate input/reuse/cost design. This opened holdout is unavailable for
subsequent tuning as unseen evidence. Production implementation awaits user review.

- Agree what default discovery should return when a matching embedded spell or
  equipment entry belongs to a root that does not itself answer the request.
  Compare that with useful local afflictions, unique creature abilities and
  explicitly requested spell-carrying creatures; do not blanket-exclude embedded
  Item types. A copied spell may itself contain a useful affliction definition.
- Evaluate answer identity, ownership scope and repeated-copy diversity before
  changing RRF constants or switching models. Descriptive failures in the sample
  occur in vector-only results as well, so hybrid ordering is not a universal cause.
- Preserve ordinary kind/trait filters as explicit criteria. They improve answer
  shape in post-baseline diagnostics but do not settle unfiltered discovery or
  within-kind conceptual relevance. Do not silently infer filters from query text.
- Compare exact embedded-name specificity separately from semantic candidate-window
  diversity. The controlled comparison finds no named-example admission miss in
  this packet; balanced semantic pools add roots but do not change its top 5 under
  unchanged maximum aggregation. This is focused evidence, not proof that candidate
  crowding never affects other queries.
- Evaluate witness quality separately from root relevance: the correct owned
  lexical definition can coexist with an unrelated highest semantic passage.
- Keep fresh CLI model/artifact startup separate from warm service end-to-end
  latency. Neither wall-time measurement isolates SQL or represents release builds.

## Evaluation direction

Next proposed research scope is recorded in
`scratch/plans/2026-10-13-search-evidence-and-semantic-input-evaluation.md`.
Evaluate partial embedded-label influence without losing exact embedded-name
discovery, one semantic contribution with reduced associated influence, and a
separate self-description/intrinsic-mechanics preference hypothesis. Preserve
primary-title/alias rules and meaningful ownership answers. Include real-user
masked-spirit and duplicate-match observations as exposed development cases.
Fresh holdout must contain actual associated-only owner positives, with paraphrases
clustered rather than counted as independent scenarios.

Bundle bounded paragraph/context input probes using the pinned model, separately
from existing-vector ranking results. Target-only re-embedding cannot establish a
full-corpus gain; a broader input experiment needs a separate cost/exposure decision.
No intent controls, narrative FTS expansion, alternate model or production policy
is selected by this proposed scope.

Retain and reuse the successful full artifact and shared pinned model assets.
Build a repeatable source-backed judgment set with expected records and explanatory
passages. Include creature lore, owned afflictions, spell mechanics, long documents,
copied spells/equipment and noise-prone small sections. Later multilingual/source
upgrade experiments need their own pinned baselines. Expected positives are not
an exhaustive relevance oracle; lower unjudged results remain unjudged.

Compare changes to lexical vocabulary, section selection, context, token budgets,
overlap, FTS weights, RRF constants and candidate windows one at a time. Record
intended differences and corpus costs instead of per-query ranking exceptions.
Alternative embedding models, inference upgrades, learned rerankers and new search
backends require separate evidence and agreement. Do not restore retired weighted
fusion, weak-match heuristics or product-facing retrieval knobs as compatibility.

Maintained evaluation tooling, if needed, belongs in a separately reviewed developer
surface. The local TypeScript research harness does not add a product command or
establish a second retrieval implementation.

## References

- [Artifact contract](../architecture/artifact-contract.md)
- [ADR 0046](../architecture/decisions/0046-source-backed-reference-artifact.md)
- [FTS tokenization exploration](../backlog/items/rust-fts-tokenization-stemming.md)
- [Completed saved-list and app-wide UI consistency](../backlog/history/items/shared-ui-consistency-and-saved-lists.md)
