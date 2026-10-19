# Search experiment for user review

Status: in_progress
Updated: 2026-10-14

The user approved this revised strategy and reviewer guidance. Research execution
is authorized. Fresh queries remain sealed while revised inputs and tools are
verified; no primary evaluation, fresh grading or quality comparison has run.

## The questions

1. Does replacing BGE-small improve useful discovery over the full corpus?
2. Does adding descriptive parent context to embedded passages improve it?
3. Does the benefit of that context depend on the model?

Compare every cell, rather than choosing a model first and then optimizing its
representation:

| Model | Baseline without parent description | Parent-description context |
|---|---|---|
| BGE-small-en-v1.5 | Baseline | Context change alone |
| BGE-base-en-v1.5 | Model change alone | Combined change |
| EmbeddingGemma300M | Model change alone | Combined change |
| Qwen3-Embedding-0.6B | Model change alone | Combined change |

The full pinned PF2e6.12.4 artifact contains25,641 records,25,560 with semantic
units. Include all130,566 existing units, deduplicated to130,184 current strings.
Apply the same product filters/visibility to every configuration. This is a full
corpus comparison with a sampled set of search intents, not exhaustive evaluation
of every possible query.

## Exactly what changes in the representations

Current body passages have bounded root-name, pack, leaf-owner and heading context.
Omit the pack-label component from embedded-passage headers in BOTH variants and
for ALL four models; keep pack as metadata/filtering information. Regenerate
through public preparation, preserving authored prose and body chunk boundaries.
Root identity embeddings retain their existing vocabulary. This revised semantic
baseline is not an exact replay of current product search.

The proposed variant adds a literal excerpt from the root's first nonempty
Description, Notes or Biography field in public source-selection order, using
its first nonempty section and first existing text-splitter chunk of at most128
BGE tokens. It does not summarize or select text using the search query. Its shape:

```text
Parent description:
<bounded literal excerpt>

Embedded content:
<baseline child passage, with pack label omitted from its header>
```

Root/identity units stay unchanged. Children without qualifying parent prose use
the same pack-free baseline header in both variants. All6,898 roots with embedded
content were checked:5,253 have
qualifying prose and1,645 do not. This changes49,023 unit occurrences into48,785
new distinct strings in the PREVIOUS pack-bearing preparation. Distinct text
counts, reuse and token checks must be recomputed after header changes; preserve
old receipts and reuse only exact unchanged inputs. It adds no ancestor
or casting-chain facts, extra traits, inferred themes or family-specific classifier.

This tests one bounded parent-description recipe, not every possible parent
representation or a whole-record embedding. It cannot establish that all parent
context strategies work or fail. Kind-filtered scopes containing no changed units
must have identical same-model results between representations; code checks this.

Use each model's documented query/document formatting, pooling and native token
limit. All receive the same underlying source characters for a given representation.
Different tokenizers count that text differently: current maxima are286 BGE,
1,074 Gemma and1,121 Qwen tokens. Context maxima are407,714 and724 respectively.
No child clipping or longer-source-context variant is included.

## Semantic-only comparison

User agreed to isolate semantic retrieval. Apply the same explicit filters and
visibility eligibility to every configuration. No FTS/lexical lane, exact-name or
alias promotion, RRF/fusion, reranker, graph expansion, labels or intent selector.
Hybrid policy and production integration are separate follow-ups.

Approved measurement: score every eligible passage, take each record's best
passage similarity, then sort all records with a deterministic tie break. Each
record gets one score; matching more passages adds no bonus. More passages still
create more chances for a high match, which is worth examining in the results.

Current product retrieval takes the nearest1,024 passages, widening to2,048/4,096
while seeking200 distinct records. Full aggregation instead produces complete
record ranks, including known references outside the first ten. Pure best-passage
semantic top-ten results will usually agree with a window that already contains
enough distinct records. This is a measurement choice, not evidence that production
windows necessarily lose useful top-ten results.

Validate common raw distances and aggregation against native semantic results
where inputs are unchanged. Hybrid top-ten replay is no longer the acceptance
check. Test production candidate limits and fusion separately from model quality.

## Queries and blindness

Primary comparison:24 independently authored fresh concepts,48 open/kind-filtered
requests, four concepts in each of six categories:

- Creature identity/flavor.
- Composed creature capabilities, including caster-style discovery.
- Functional spell discovery.
- Hazard discovery.
- Equipment, rules and lore.
- Exact/partial-name controls.

Fresh wording remains unopened. Freeze the complete model/representation/search/
grading choices before opening it once. Keep requests unchanged; do not remove
awkward cases or tune a model/prompt after seeing outcomes.

Also replay the exposed24-concept/43-request development cohort as diagnostics.
Existing examples include `masked spirit`, `wizard with fire magic`,
`walk through walls`, `repair broken objects`, `lethal music`, and staff lookups.
Show known source-example ranks and failures without calling those examples
exhaustive recall or treating exposed queries as independent confirmation.

Pool all eight top-ten lists AND available source-checked reference candidates,
even references absent from every top ten. For fresh queries, a source-discovery
pass can propose references before viewing model outputs. References remain
challengeable, nonexhaustive examples. They enter only the judgment pool, never
retrieval results or ranking. Shuffle unique query/record
judgments and hide configuration, model, rank, score, retrieval channel and model
membership and reference-candidate status. Reuse a source judgment across compatible filtered/open observations
while retaining their memberships. Pooling avoids separately grading the same
answer eight times. Include adjudicated references in the common ideal and report
their actual ranks/coverage. Reference coverage is not exhaustive corpus recall.

## Reviewers and checks

Use three Luna evaluators in parallel, with primary work partitioned by concept
and balanced across categories. Each reads the full relevant source, not a search
snippet. The coordinator stores source fingerprints once in the input manifest.
Reviewers return assigned ID, grade/flag, rationale, pointers and literal quotes;
they need not calculate/repeat hashes or source filenames. Validate citations
against that ID's bound source. This proves provenance, not semantic interpretation.

Assign a deterministic category/scope-stratified10% overlap to a different Luna
before any grades arrive. The coordinator reviews source evidence for disputes
without looking at model membership or joined quality scores. Preserve original
labels and any adjudication separately. Do not resolve truth by majority vote.

Adjudicate **all grade disagreements in the overlap**, including2-versus3, as
well as opposite-function disagreements. This requirement is approved and the
evaluation tools must enforce it before final conclusions.
Unresolved material cases must leave conclusions provisional or carry explicit
label sensitivity, rather than silently trusting the primary label.

The approved [reviewer guidance](evaluation/GRADING.md) includes
the rubric and concrete interpretation issues found in calibration. A12-case
development calibration has already run: all three judges agreed on the useful
versus not-useful boundary, but three cases differed in0/1 grades and one differed
on the opposite-function flag. They all gave a cold-cantrip caster2 where an old
review gave3. These are reasons to align the rubric, not claims of reviewer accuracy.
One agent initially emitted invalid evidence paths; a preserved correction passes
the coordinator's exact validator. No calibration opinions were accepted unchecked.

Prepare a user packet containing a deterministically selected concept from each
category, consequential gains/losses and every unresolved material dispute, with
source excerpts and complete linked details. Keep user corrections versioned.

## What will be measured

- Primary: concept-balanced useful-result proportion in the first five and pooled
  graded nDCG@10. Average paired scopes within each concept, then concepts equally.
- Also show any useful answer@5, category/open/filtered results, name diagnostics,
  opposite-function results@3, known-reference ranks/coverage and alternatives.
  Show semantic-discovery intents separately from name controls; those controls
  do not test the later lexical lookup strategy.
- Report all eight configurations and each model's context delta; show interactions
  rather than only the best observed cell.
- Show paired concept-bootstrap uncertainty, with ordinary95% intervals and wider
  seven-comparison intervals for alternatives versus small/current. Four concepts
  per category support descriptive examples, not strong category-superiority claims.
- Treat unknown labels as unknown; report coverage and conservative bounds. An
  all-irrelevant pool has no defined nDCG ideal and is reported separately.
- Measure startup, warm query embedding, full document-build cost, reference scan
  time, memory and storage. Query embedding time is not end-to-end product latency.

No zero-error veto, significance-only gate or automatic default-model switch.
The decision should weigh practical usefulness, size/diversity of gains, specific
regressions and cost. A higher average can be worthwhile despite a few regressions;
a favorable scalar alone is insufficient evidence. Product integration is separate.

## Discussion after scoring

Preserve raw ranks/scores, query/record judgments, source evidence and cost
measurements. Start with the coordinator's analysis, then examine categories,
filtered/open scopes, model/context interactions, reference ranks, concrete
successes/failures, grade sensitivity and cost in response to user feedback.
Source-backed grading corrections stay possible: preserve originals, record the
correction and recompute all configurations consistently. Label newly chosen
outcome-based slices exploratory. Changes to models/inputs/ranking after results
make these queries development data; a new independent evaluation is needed for
another fresh-evaluation claim.

## Already done and remaining cost

Completed: verified corpus reconstruction and all model/context input checks;
model assets and runtime smoke checks; all BGE-small/current vectors reused;
BGE-base/current completed using32,184 reused and98,000 newly inferred vectors.
The new base run took20.3 minutes. Published-example numerical checks and longest
input smoke tests passed for Qwen/Gemma.

Stopped: BGE-small context run was terminated on user request, retaining its
partial output and failed/interrupted receipt. The sequential runner then exited;
no other queued document runs started. Only fully receipted and independently
verified vectors may be reused after approval. Do not discard and recompute them
merely to resume a clean-looking run.

Remaining: revise the prototype for semantic-only ranking, pack-free embedded
headers, reference candidates, revised grade1 and simpler reviewer output. Recheck
inputs/reuse before finishing vector runs. Then freeze outputs, open the
fresh queries, retrieve results, run blind reviews/adjudication and produce findings.
Expect several hours of sequential CPU work; this is a smoke-based estimate.
Research-only disk remains capped at6GiB and sampled per-model RAM at8GiB. No new
Cargo targets, worktrees, product changes, commits or PRs. Preserve existing artifact,
source, model cache, preview and unrelated checkout changes.

Detailed proposed statistics: [protocol](evaluation-design/protocol.md).
Preparation evidence: [context review](evaluation/context-method-review.md),
[method review](ranking/evaluation-method-review.md), and
[calibration report](calibration/review.json).
