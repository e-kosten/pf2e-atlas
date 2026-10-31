# Broader exact neighbor reuse

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-30

## Scope and evidence

Explore exact native-neighbor reuse beyond ordinary unscoped retrieval without
changing retrieval quality, public request state or cancellation. The audit used
merged revision `07ca075467c318fbf5d134555bbea7070c7397b2` from
[PR164](https://github.com/e-kosten/pf2e-atlas/pull/164).
[ADR0062](../../architecture/decisions/0062-exact-text-neighbor-reuse.md) defines
the ownership, identity, bounds and publication contract. The shared ordinary
producer now covers unscoped TextSearch and baseline record/trait Similar.
Measurements from one consumer do not establish benefits for another.

## Remaining candidates

| Consumer | Native work and required extension |
| --- | --- |
| Trait-reference text search | Hybrid retrieval inside referenced record keys. Bind the actual eligible set, including empty scope, rather than the trait label. |
| Configured content/part Similar | Distinct 256-neighbor selections with comparison-role and identity eligibility. Requires matching selection recipe and hydration checks; candidate-vector reads and scoring remain separate. |
| Saved-list text filtering | FTS-only today; neighbor reuse removes no work. |

Confirm concrete first-party repetition before choosing another slice. Trait
Similar defaults to Baseline with separate preview/full-search requests; record
Similar defaults to Passages and benefits from ordinary retention only when
Baseline is selected. Sharing requires identical filters and other native inputs.
A preview reopen can already hit the browser cache. Do not extrapolate baseline
gains to advanced strategies.

Configured Similar can generate up to 256 distinct windows against the current
64-entry store, so repeated requests can churn it. Measure generation, vector
reads and scoring separately, plus mixed-consumer hits, evictions and memory,
before choosing admission or capacity changes.

## Default Passages profile

A descriptive profile at merged `d5191ecf` used default record Passages
(decay 0.5, candidate parts Any), source sections All and independent candidate
content Core/All. Leather Armor, Goblin Warrior and Treerazer supplied 2, 7 and
27 stored seed units, respectively. Both profiles and preview/full/page/Random
workloads were covered in all 12 representative cells.

There were 135 accepted observations: 127 successes in that representative
matrix, seven supplemental Lich successes and one Lich work-limit failure.
All 79 repeated-request complete-output hashes matched. Candidate generation
accounted for 73.9–93.5% of measured phase totals in the representative matrix.
Its timing includes validation, native selection, decoding and union insertion;
this percentage is not an established cache speedup. These are fresh-read
component measurements, with small and uneven repetitions, not browser latency
or tail estimates. Other build/test jobs remained running; interrupted operations
were rejected, and the extended repetition schedule was left incomplete.

The descriptive study identified exact retention at the configured native-selection
producer, sharing the existing finite store, as a candidate requiring further
evidence. Bind its distinct recipe, comparison role and identity eligibility
alongside complete ordinary identity components.
Hydration must recheck the configured mask; the ordinary hydrator is insufficient.
Keep union construction, candidate-vector reads, scoring, work budgets and
evidence hydration with their current owners. Add actual native-execution metrics
separately from logical candidate-call diagnostics.

Observed completed candidate-call counts were 3, 8, 28 and 44 for the four seed
sizes. If identities are distinct, interleaving 28- and 44-window requests can
exceed the 64-entry store. Measure actual unique identities, reuse distance,
hits/evictions and accounted allocations before changing capacity.

Lich/BGE/All exceeded the accumulated 65,536 candidate-vector work budget;
Lich/EmbeddingGemma/All succeeded with 61,142 candidate vectors. The raw BGE error
reports a remaining batch allowance of 2,428, not a different global limit.
Lich/BGE/Core succeeded; Lich/EmbeddingGemma/Core was not observed. Preserve these
existing profile-specific outcomes; selection retention does not remove a later
vector-work-limit failure.

## Fresh candidate generation

An exclusive optimized investigation on production `d5191ecf` completed 38
processes / 196 operations in 530.16 seconds with 2,013 clear process-inventory
checks. Default record Passages used source sections All, candidate Core/All,
both profiles and preview/full/page-two repetition; ordinary Hybrid TextSearch
was the control. Separate process/model first use, fresh selections and warm
repetition were recorded. The diagnostic overlay, source/executable hashes,
raw phase/CPU traces and review packet are retained locally under
`scratch/fresh-generation/`; no optimization was implemented.

Across representative cells, aggregate pre-filter SQL/VM used 48.1–58.1% of candidate
query-collection thread CPU and the native distance loop used 18.6–24.9%.
The pre-filter interval includes binding and eligibility RHS materialization;
it is not a pure eligibility timer. Rust construction plus statement preparation
was only 0.28–3.17 ms/request. Metadata decoding and union construction were small.
Native timing separately measured rowid-IN consumption, chunk/BLOB IO, masking,
distance and top-k; their enclosing total must not be added to its children.
A separate CPU stack sample corroborated SQLite VM/B-tree/JSON and native work.

Armor and Treerazer made 3 and 28 logical/native calls with 3 and 28 complete
unique selection identities, respectively, and one shared setup identity per
request. A 97-unit seed made 98 calls / 91 unique selections / one setup:
seven duplicates exist there, but the small/medium cases have none. Repetition
of configured requests still executes all selections on this production base.
The observed setup identity binds exact semantic-unit eligibility independently
of vector bits and k; matching identities establish an opportunity, not savings.

Actual ordinary retention avoided five of six Hybrid native executions while
real query inference remained separately charged. In each two-round mixed run,
266 logical calls produced 259 native executions, including seven ordinary hits;
configured selections never entered or evicted from the production store.
A separately labeled replay of modeled configured admission through the actual
unchanged store produced 19 hits / 247 misses / 183 evictions: configured got
14/258 hits, exactly the seven large-request duplicates twice, with no surviving
inter-request configured hits. Ordinary fell to 5/8 hits through two evictions.
The store ended at 64 entries, 8,421,208 accounted live bytes and 8,552,776 peak
bytes under the existing 16 MiB budget. Provisional keys, sequential replay,
accounted memory and modeled hits do not establish production configured latency,
hydration memory, RSS or concurrent/pinned-borrower behavior. Do not increase
capacity merely to fit these requests.

One bounded next proposal is lazy request-local reuse of the exact configured
semantic-unit eligibility relation, inside the index owner and the same admitted
snapshot. Materialize only after the first selection's existing validation and
reuse only for matching complete setup identities. Keep native traversal,
distance arithmetic, k=256 cutoff/ties, location decoding, unions, scoring and
evidence unchanged. Bound materialization memory and release it with the request; oversized
relations must fall back to the existing exact path without new search failures.
Retain every existing cancellation checkpoint, error order and work limit.
This is a proposed prototype, not a measured speedup. It must first prove exact
eligible-set parity for empty/scoped/excluded inputs, structured and relationship
predicates, Core and comparison-role/identity masks, then measure construction,
repeated consumption and peak memory. Ordinary Hybrid remains the control.
No window changes, approximation, budget increases, cache expansion or batching
are included in this proposal.

The diagnostic study passed 150 complete-output comparisons, both-profile
all-equal-distance cutoff-tie fixtures, cancellation-check parity and existing
work-limit failures. Lich/BGE/All still reported remaining allowance 2,428;
the large seed reported 41 for BGE/All and succeeded for Gemma/All. Native traces
separately counted 1,486 logical calls / 1,452 actual scans and checked exact
ordered payload bits for repeated identities. Three rotated calibration rounds, using each round's three-operation total,
put timed observation within +0.38–1.49% of the uninstrumented control for one
BGE/All cell; tiny decode phases include observer cost.

These are one-machine/corpus component observations with few repetitions, warm
OS files to an uncontrolled degree and no browser/tail or retrieval-quality
claim. Explicit filter/role variants were not timed. The earlier descriptive
profile's uneven coverage, incomplete schedule and nonexclusive natural windows
remain limitations of that earlier study; this cohort does not repair them.

## Deferred discovery experiment

The local discovery-reuse experiment (`ffc1e77d`) is retired and must not be
integrated. The browser does not request text-context facets; the CLI can repeat
counts and values in one command, but counting a ranked candidate set is a
separate product capability from ordinary filter counts. Ordinary discovery is
to use structured eligibility. Summaries of semantic retrieval candidates are
deferred until a demonstrated use case and explicit product contract exist.
No accepted latency result was obtained for the retired experiment.

## Validation for an extension

- Prove cold/hit equality of deterministic output, coverage, distance bits,
  witnesses and failures for both profiles; distinguish changing telemetry.
- Preserve native exact-k membership and cutoff ties. Filtering global top-k
  cannot produce scoped top-k. Exercise changed/reordered/duplicate/empty scopes,
  exclusions, predicates, Core/All, seed failures and role/identity masks.
- Preserve current-snapshot hydration/admission, generation/publication retirement,
  cancellation, permits, work budgets, bounded allocation and oversized-key bypass.
  Keep ranking, remaster policy and Similar scoring with their existing owners.
- Use matched release operations on the same private corpus/model snapshot and
  serialize builds and timings. Report exact commits, output parity, cold/warm
  latency, native/inference counts, mixed working sets and accounting limits.

Keep policy in shared retrieval/native-candidate owners. Reconcile dedicated
opt-in paths and ADR0062 exclusions when coverage expands; do not add one cache
per surface or retain transitional implementations.

Concurrent-miss sharing is a separate proposed optimization. Measure duplicate
in-flight work before defining leader cancellation, surviving waiters, error
fan-out, admission and permit ownership. No broader speedup is established.

## Bounded eligibility prototype

The bounded request-local exact eligibility prototype is implemented locally at
`91c39050`, based on main `b10ce08e`; coordinator integration remains separate.
[ADR0063](../../architecture/decisions/0063-request-local-configured-eligibility.md)
records its complete identity, aggregate bounds, validation/error/cancellation
order and exact capacity bypass. Original `d5191ecf` research is unchanged.

A guarded optimized cohort completed 56 processes / 184 operations in 783.95
seconds with 2,973 clear relevant-load checks, plus a separate six-process resource
cohort. All 95 paired complete outputs/errors, native work and rowid counters
matched. Successful default Passages request medians improved 20.1–43.7% across
both profiles and Core/All; selective corresponding-role cases improved
40.0–43.6%. Large BGE/All preserved its work-limit failure and improved 43.1% on
that error path. Native selection counts remained 3/28/98 for small/medium/large;
this reuses eligibility setup, not selections. No cross-request retention or
capacity change is included.

One-selection controls cost 2.1–3.7% more. Ordinary warm BGE controls were 8.9%
and 14.3% slower (+4.01/+5.67 ms), with corresponding CPU movement; their cause
is unresolved. Gemma warm controls were near baseline. Additional quiet paired
process replication is needed before claiming ordinary stability or general
non-regression. One process per arm/cell, correlated repeats, small n and
uncontrolled OS cache state limit the latency/tail evidence.

Maximum accounted owned state was 3,215,441 B under the 4 MiB policy. The separate
resource cohort observed SQLite highwater increases of 1.05–3.11 MB and whole-process
RSS increases of 1.33–6.44 MB. Reader closure and one-second cooldown returned
SQLite usage to the baseline 16 B. Implicit TEMP file observations are sampled,
not complete disk peaks; no hard global SQLite heap/TEMP policy is introduced.

Full applicable local gates passed (1,531 nextest passes / 30 skips, Clippy,
formatting, doctests, workspace/frontend builds), as did affected real HTTP/browser
checks and separate AI implementation/evidence review. No hosted CI or human
approval is claimed. Raw source/binary provenance, exact outcomes, medians/p95,
CPU/resource receipts and remaining limitations are retained locally under
`scratch/request-local-eligibility/REPORT.md`. These results support bounded
prototype handoff; publication and broad release acceptance remain separate.


### Ordinary-search replication

A separately frozen follow-up ran 48 counterbalanced independent process pairs
(96 processes, 1,056 operations: three warmups and eight measured requests per
process) in 209.47 seconds with 861 clear load checks. It compared the original
diagnostic probes and matched release probes without production diagnostic
overlays, using both profiles and Core/All. All outputs, exact profile/corpus
identity, store states and miss/hit observations matched. The original configured
cohort is unchanged and is not pooled with these results.

Median paired warm BGE request changes were +0.988 ms/+2.013% diagnostic All,
+2.210 ms/+4.845% diagnostic Core, +0.375 ms/+0.740% plain All, and
-0.616 ms/-1.309% plain Core. Both plain BGE cells and all Gemma controls met the
predeclared bounded stability rule. Diagnostic Core missed its 2 ms absolute
limit, with five slower pairs and approximately +2.288 ms process/thread CPU;
the overall frozen verdict remains mixed/unresolved. The earlier 8.9%/14.3%
effects did not recur at that size in the independent paired medians, which does
not by itself establish general non-regression.

Every measured request was a retained hit with no native scan. Source/phase
inspection finds no changed ordinary path or hit pattern; the residual is inside
the request and changes with build mode/order, but no causal mechanism is
established. Separate AI review confirmed the evidence and mixed verdict. Do not
selectively extend or relabel the cohort. Further causal work should start from
a specific hypothesis and an agreed new probe, or explicitly accept this remaining
uncertainty before publication. The 2.1–3.7% one-selection cost and unestablished
tail improvement remain. Every new process, warmup, paired CPU/millisecond effect
and source/binary hash is retained in `scratch/ordinary-replication/REPORT.md`.
