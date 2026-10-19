# Historical snapshot: 2026-10-09-concept-labeling-and-ontology-design.md

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `scratch/plans/2026-10-09-concept-labeling-and-ontology-design.md`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````markdown
# Concept labeling and ontology development

Status: deferred
Updated: 2026-10-13.
Next action: resume design discussion after search/refactor priorities; implementation remains unapproved.
Created: 2026-10-09.
Backlog anchor: [Rust derived-tag redesign](../../docs/backlog/items/rust-derived-tag-redesign.md).

## 1. Authorization and purpose

Preserve the current discussion, research, design directions, and unresolved
choices for developing Atlas labels after the broad source-backed refactor is
finished. This document is a fresh planning surface, not an execution-ready plan
or authorization to implement the feature.

Current authorization permits creating and iterating on this plan document only.
Preserve the shared checkout and all worktrees. Do not edit code, existing plans,
backlog, architecture docs, skills, or data; run builds or assignment jobs; create
worktrees; or commit/push as part of this planning task. Planning stays on the
existing branch. The user's scope overrides broader repo-plan skill defaults
about planning on main, backlog edits, and producing an execution-ready plan.

The immediate deliverable is this document. Future implementation requires the
refactor to finish, re-authentication of the resulting architecture, resolution
of the open choices, and user authorization for the concrete implementation plan.

## 2. Goals and current discussion directions

Give users predictable filters for meaningful questions that native Foundry DTO
filters cannot express clearly. Agents prepare ontology proposals, calibration
examples, and assignments; the human approves definitions and examples and
resolves important ambiguities through a separate developer review surface.

Directions supported by the discussion:

- Stable concept identity and meaning are separate from browse organization.
  One concept may appear in several useful browse collections.
- Existing Node vocabulary and later Rust experiments are research inputs.
  Their storage formats, taxonomy, workflow, and models are open to redesign.
- Retire the Node semantic rule machinery and over-fit text heuristics. Keep
  deterministic source access, evidence checks, coverage accounting, and any
  specifically approved applicability policies.
- Every record is considered for every potentially applicable approved concept.
  Text search, embeddings, names, and keyword signals must not suppress required
  LLM assessments. They can help exploration, ordering, and exemplar discovery.
- Prefer extra LLM passes over speculative exclusions that create false negatives.
- Calibration sets are agent-prepared and human-approved. The human is an
  approver and reviewer, not responsible for assembling datasets or editing files.
- Positive assignments have inspectable evidence and concise explanations useful
  both in developer review and a future product “why this matched” affordance.
- Ontology iteration and large-scale record assessment are distinct, connected
  activities. Assignment findings can propose ontology changes; approved ontology
  changes create explicit reassessment work.
- Applicability exclusion justification exists once per policy rule. Routine
  negative record assessments do not require long explanations.
- Distinguish weak negatives from strong unrelated negatives to support review
  priority and potentially selective reassessment. Safe carry-forward semantics
  are still an open design choice, not approved automatic pruning.
- Trial Luna assignment workers and Sol independent audits/reviews. Reviewer count,
  composition, acceptance policy, and reconsideration procedure need measurement.

The proposed technical mechanisms below remain hypotheses unless identified as
a user requirement. In particular, YAML, a formal concept graph, exact schema,
developer UI hosting, and unanimous versus majority acceptance are not frozen.

## 3. Architecture context and prior evidence

Relevant current docs:

- [Overview](../../docs/architecture/overview.md),
  [runtime](../../docs/architecture/runtime.md), and
  [artifact contract](../../docs/architecture/artifact-contract.md).
- [Tagging architecture](../../docs/architecture/tagging.md) and
  [ADR 0028](../../docs/architecture/decisions/0028-rust-tagging-model.md).
- [Active cutover plan](archive/2026-10/2026-10-09-source-artifact-cutover-contract.md) and
  [execution ledger](archive/2026-10/2026-10-09-source-artifact-execution-ledger.md).

As inspected on 2026-10-09, the primary checkout is
`feat/source-discovery-command`; the active cutover is in
`.worktrees/source-artifact-cutover`, branch `refactor/source-backed-artifact`,
with implementation/validation still in progress. Its intended end state uses
checked Foundry source DTOs, borrowed views, prepared content, owned-item and
passage identities, and typed query projections. It replaces normalized record,
metric EAV, and persisted RichDocument authority. Re-read the final docs and
interfaces after landing; today's worktree details are not a frozen dependency.

The existing tagging ADR describes YAML, one primary display path, minimal review
state/provenance, and unanimous agent acceptance. This discussion explicitly
reopens those choices. This plan does not supersede that ADR or change runtime
behavior; approved changes must reconcile the docs during future implementation.
`docs/architecture/boundaries.md`, named by the planning skill, was not present
in the inspected primary checkout; use the existing overview and focused docs.

Prior artifacts worth revisiting:

- [Node concept model](../derived-tag-ontology/derived-tag-concept-model.md),
  [mapping summary](../derived-tag-ontology/derived-tag-concept-mapping-summary.md),
  and [mapping CSV](../derived-tag-ontology/derived-tag-concept-mapping.csv).
  The inspected mapping has 515 category-tag entries, 425 distinct surviving
  proposed concept IDs, and 200 medium-confidence rows. These are historical
  mapping statistics, not validated assignment coverage or semantic accuracy.
- The former Node reference at `.worktrees/node-main-reference/src/tags` contained
  canonical definitions/projections, assignments, old rules, and editorial skills.
  That checkout is unavailable at cleanup; recover the relevant Git revision
  before using it as implementation evidence.
  Preserve useful exemplar and boundary knowledge through deliberate review.
- [Later ontology proposal](../../.worktrees/tag-corpus-loader/docs/architecture/tag-ontology.md)
  and [seed catalog](../../.worktrees/tag-corpus-loader/data/tags/catalog) contain
  104 tags across ten proposed groups. UsagePattern has registry metadata but no
  seed catalog entries. This is an incomplete experiment, not a selected taxonomy.

Concrete historical faults and strengths:

- Keep useful problem-response concepts: source detection/neutralization,
  contamination cleanup, restraint escape, ritual appeasement, and diagnosis.
- Preserve encounter behavior, setting association, motifs, and teaching examples
  with meaningful nearby negatives. “Beneficial” was appropriately identified
  as too vague for a durable retrieval concept.
- The old mapping gives both a counteracting spell and a hazard resolved through
  counteracting the same countermagic concept. Preserve the distinct claim directions.
- Later “Curse Removal” guidance includes identifying/revealing curses, and
  “Disease Treatment” includes diagnosis. Labels and actual membership boundaries
  need to agree.
- Review overlaps such as exploration stealth versus build stealth. Distinct
  use cases may justify separate claims, but browse lenses alone do not.
- Theme association, operating capability, and support for an environment have
  different meanings. A swim Speed alone does not establish aquatic habitat.

Provisional ownership, to verify against the finished refactor:

| Concern | Candidate owner/boundary |
| --- | --- |
| Concept definitions, applicability, assessment contracts and validation | `atlas-tags` |
| Storage-agnostic record evidence/context and content identity | `atlas-record` over source DTOs |
| Generated product label rows, validation, SQL filters/discovery | `atlas-index` |
| Consumption of accepted label data during artifact generation | `atlas-ingest` |
| Product retrieval and label discovery | `atlas-search` |
| Paths, readiness and service construction | `atlas-runtime` |
| Product workflows and exported contracts | app-service/app-model |
| Developer review workflow, durable runs and feedback storage | Open; investigate `atlas-dev` and a separate developer service |
| CLI/HTTP/UI presentation | Thin surfaces over owning services |

Developer decisions and run history need durability independent of regenerated
product artifacts. Their exact storage owner is unresolved. Product filtering
should use accepted authoritative rows through normal SQL eligibility, with no
query-time LLM inference or approximate post-filtering.

## 4. Target model: labels express defined claims

A label can be a concise presentation of a claim with deeper semantic meaning.
Introduce structure where it makes useful queries and evidence boundaries clear;
determine whether other concepts need only definitions and exemplars.

| Example claim | Candidate label | Meaning distinction |
| --- | --- | --- |
| Provides a way to counteract active magic | Counteracts magic | Supplies a solution |
| Hazard has an explicit counteracting resolution path | Resolved by countermagic | Accepts a solution |
| Characteristically attacks from concealment | Ambusher | Actor behavior |
| Strong authored association with underground environments | Underground setting | Setting association |
| RollTable meaningfully evokes folk-hero stories | Folk hero theme | Theme can cross source families |

Potential small relationship vocabulary: provides, inflicts, protects against,
resolved by, associated with, contains/generates. These are examples for evaluation,
not a chosen enum or an obligation to build a complete formal ontology.

Explore two sources of user-facing labels: named native DTO filter expressions
and authored interpretive concepts. If a label merely makes a native filter
easier to discover, prefer improving presentation over duplicating its truth in
AI assignments. Composite concepts require a concrete additional user question.

Concept admission should establish the user question, native-filter overlap,
positive evidence, nearby negatives, conditional behavior, and scope. For actors,
separate possessing a capability from a characteristic tactical role. An option's
prerequisites, target limits, or optional activation remain part of its explanation.

Browse collections organize discovery by useful questions. They can overlap,
have stable identities of their own, and change membership without changing
concept identity or meaning. If selectable, a collection needs explicit filter
semantics; display placement does not imply semantic inheritance.

Provisional coverage checklist: setting; encounter/social roles; problem response;
exploration/logistics; combat effects; themes; scene challenges; character support;
usage patterns; status/affliction behavior. Review actions, feats, character options,
deities, rules content, and RollTables as well as the historical five categories.
This checklist is not an approved set of groups or assignable labels.

## 5. Two connected development loops

### Ontology development

Agents mine prior knowledge and real records, formulate useful claims, compare
them with native filters, prepare positive/negative examples, and propose meaning
and applicability. A coordinator assembles a concrete review packet; the human
approves, corrects, or requests further research. An approved revision releases
assessment work. The catalog can evolve without first describing every possible
concept in a complete formal ontology.

### Record assessment

Workers receive an approved concept revision and sufficient source context,
return independent decisions and evidence, undergo validation and reconciliation,
and publish accepted assignments through the chosen artifact workflow. Recurring
ambiguity, missing concepts, or family exclusions produce secondary ontology
proposals. Workers cannot silently change the approved catalog during assessment.

Ontology research can inspect any useful record; it is not required to await a
whole-corpus assessment. Broad assessment begins only after its definitions and
procedure meet the chosen calibration gate. Evolving one concept need not halt
unrelated approved work.

## 6. Applicability, negative assessments and versioning

Applicability exclusions concern what a claim requires, rather than the families
where its first examples were found. Themes generally have broad eligibility;
actor-behavior claims may require an actor subject. Unknown facts and uncertain
family combinations stay eligible for assessment. Each exclusion has one concise,
versioned policy justification and is open to challenge and audit.

Distinguish applicability exclusions from per-record semantic negatives. Candidate
assessment states are positive, weak negative, strong negative, and insufficient
context; approved structural exclusions and unfinished work are separately tracked.
These names are illustrative, and positive assignments can still be stored sparsely.

Strong versus weak negative is a proposed measure of support for a negative,
not simply the model's self-reported confidence. A compact basis code or criterion
reference may be useful. Routine negatives need no prose essay. Preserve detailed
negative evidence where it teaches a boundary, supports an audit, or resolves a
dispute. Invalid or missing context cannot produce a strong negative.

For an initial definition refinement, prioritize weak negatives, disputed cases,
and prior positives. Strong negatives may be cheaper to handle when the change
provably leaves their basis intact. However, negative strength alone cannot exempt
a pair from reassessment: broadening “performs X” to “contains or generates X” can
make a formerly unrelated RollTable relevant. Family applicability expansion must
schedule newly eligible records.

Until selective carry-forward is calibrated, reassess all potentially applicable
pairs for decision-affecting changes. If later allowed, carry-forward must record
the old assessment, new revision, impact-analysis justification and acceptance;
it must never pretend the LLM evaluated the new definition. Random audits must
include strong negatives and carried-forward decisions. This remains an open
optimization, not a replacement semantic rule engine.

Version concept semantics/applicability, catalog membership, browse organization,
record context, assignment procedure, and publication independently. A catalog
snapshot identifies exact revisions. Material meaning changes can require new IDs
or a split/retirement rather than silently reinterpreting old assignments.

| Change | Conservative work implication |
| --- | --- |
| Browse placement or wording correction without meaning change | Presentation only |
| New concept | All potentially applicable records |
| Definition, applicability, or decision-guidance change | All potentially applicable pairs until selective reuse is proven |
| Concept split | Evaluate replacements and explicitly retire predecessor |
| Record evidence changes | Invalidate affected assessments; whole-record invalidation is a safe first policy |
| Procedure/model changes | Calibration and fresh audits; decide broader reassessment from measured impact |

Retain versioned inputs or durable retrievable references, not hashes alone.
An empty positive set means no accepted positives within an explicitly completed
scope/revision, not permanent completion. Product exclusion filters must define
how unassessed/uncertain coverage is handled. Proposed publication uses coherent
accepted editions; mixed-version partial publishing is an unresolved alternative.

## 7. Exhaustive assessment and data economy

Construct an exact work inventory from product-eligible records and approved
concept applicability. Determine whether independently navigable owned/prose
definitions also receive assignments, and how their claims relate to root records.
An owned ability can supply evidence for a creature claim without inheriting all
of a referenced record's labels. Product audience/visibility must govern context
and explanations so excluded evidence is not exposed through labels.

Evaluate bounded packets of adjacent concepts independently of browse membership.
Every required pair receives a disposition. Missing, failed, malformed, or
truncated output is unfinished. Mechanically completed coverage does not prove
good judgment: calibration and random negative audits test actual recall.

For illustration, 25,000 records x 300 concepts is 7.5 million pairs. At 20 concepts
per request and two reviewers, this is 750,000 requests before exclusions, retries,
and reconsideration. At one byte per disposition, coverage is about 7.5 MB per
reviewer per edition before metadata/evidence/indexes. These are arithmetic
examples, not catalog size decisions, performance results, or cost forecasts.

Measure packet size, complete-context length and cost in a pilot. Prefer bounded
independent requests, reusable context, shared static prompt prefixes, resumable
jobs, validated reuse and incremental changes over candidate pruning. Context
must include relevant authored content, labelled facts, important owned abilities,
conditions and references. Model summaries alone cannot establish completeness.

Retain compact dispositions/completion manifests, detailed positives with source
locators, disputes, human feedback, and teaching negatives. Raw model responses
and older context snapshots require a separate retention/retrieval decision.
Exact structural constraints, status codes, hashes and serialization remain open.

## 8. Calibration, reviewers and research

Agents prepare representative and boundary-focused sets. Human review approves
the verdict, evidence and teaching value. Representative sampling should span
source families, lengths, richness and rare labels; include weak/strong negatives,
conditional effects, misleading mentions, empty positive sets and unusual family
combinations. Candidate sets must not consist only of model-selected easy cases.

Separate teaching examples from held-out evaluations and keep related variants
together when splitting. A rejected or ambiguous proposed case is not a negative
gold label by default. Repeated ambiguity may require concept refinement.
Distinguish a reusable development evaluation set from an untouched final
evaluation set. Once evaluation failures inform definitions, prompts or reviewer
policy, those cases become development evidence; they no longer establish
generalization independently. Reserve fresh cases for final evaluation.

Pilot two-, three-, and five-reviewer configurations plus a stronger single-model
control on the same held-out cases. Preserve independent first judgments before
sharing evidence. Fresh contexts provide procedural independence, not a guarantee
of statistically independent errors. Reviewer count, model composition and
aggregation/acceptance thresholds are separate variables.

Tentative production hypothesis: two initial Luna assessments, Sol for selected
disputes and independent audits of agreement, negatives and empty sets. Compare
that with routine Sol participation for difficult concepts. Measure additional
human-confirmed errors caught, per-concept precision/recall, accepted-case errors,
shared omissions, escalation/human time, evidence correctness and cost.

The old ADR's unanimity-after-reconsideration is a historical baseline for
comparison, not a renewed commitment. More reviewers under unanimity can increase
escalations; majority voting can lose correct minority findings. Preserve initial
positive nominations through an evidence-based disposition. Bound reconsideration
and retain original votes; post-discussion convergence is not fresh independent
evidence. Unresolved boundary questions may require human ontology review.

Research reviewed during this discussion:

| Source | Finding and relevance | Limit |
| --- | --- | --- |
| [W3C SKOS Primer](https://www.w3.org/TR/skos-primer/) | Labels, scope notes, examples, collections and semantic relationships offer useful vocabulary practices. | Adopting these practices does not select RDF or a full formal ontology. |
| [Fonseca and Cohen, 2024: concept annotation guidelines](https://aclanthology.org/2024.findings-acl.478/) | Explicit definitions helped annotation; adherence varied across tested models. | No benchmark of Atlas or current Luna/Sol. |
| [Verga et al., 2024: Replacing Judges with Juries](https://arxiv.org/html/2404.18796v1) | Three diverse models provided stronger human alignment than a single large judge in studied settings. | Different aggregation by task; does not establish optimal panel count. |
| [Nahum et al.: Are LLMs Better than Reported?](https://arxiv.org/html/2410.18889v2) | Ensemble-size experiments improved labeling and error detection on factual-consistency and summarization datasets. | Adjacent tasks, not PF2e multi-label assignment. |
| [Kim et al., ICML 2025: Correlated Errors](https://proceedings.mlr.press/v267/kim25e.html) | Errors correlate across models, including across providers. | Agreement cannot be translated into independent-vote confidence without measurement. |
| [Kohli, 2026 preprint: Nine Judges, Two Effective Votes](https://arxiv.org/html/2605.29800v1) | Nine judges from seven families yielded roughly 2-2.5 effective votes in studied tasks; gains diminished beyond five. | Preprint; task/model-specific analysis, not a universal limit. |
| [Elumar et al., NeurIPS 2025: Cost-aware annotation](https://arxiv.org/abs/2505.15101) | Adaptive reviewer selection can reduce annotation cost on studied datasets. | Routing reviewer effort is distinct from skipping required record-concept assessment. |
| [OpenAI model selection](https://developers.openai.com/api/docs/guides/model-selection) and [evaluation practices](https://developers.openai.com/api/docs/guides/evaluation-best-practices) | Compare on identical inputs and calibrate automated judgments against human decisions. | Product guidance; no claim that a particular reviewer mix is optimal for Atlas. |

No reviewed source establishes a universal best reviewer count. The pilot must
measure marginal useful evidence and error detection on this corpus. Research
summaries reflect retrieved versions as of 2026-10-09; reverify before implementation.

## 9. Developer review and coordinator handoff

A separate developer UI, possibly launched by `atlas-dev`, prepares meaningful
review rather than asking the human to assemble data. Hosting, frontend package,
service ownership and storage are unresolved. It may reuse maintained record
rendering/components while separating developer actions from product workflows.

| View/action | Human input and retained outcome |
| --- | --- |
| Concept review | Approve/correct meaning, boundaries, applicability and prepared exemplars |
| Record assessment | Approve or correct verdict, evidence, qualifications and explanation |
| Disagreement | Inspect competing evidence; settle a case or identify a definition problem |
| Applicability challenge | Mark an exclusion/family restriction as unjustified and attach a case |
| Missing concept | State a useful user question and attach its triggering record |
| Calibration curation | Approve positive/near-negative teaching or held-out use; reject unsuitable cases |
| Negative review | Correct strength/basis, flag an omission or request fresh context |
| Revision preview | Inspect changed meaning, examples, affected records and proposed reuse/rerun scope |
| Coverage/drift | Inspect pending/stale assessments, audits and recurring mistakes |
| Defer | Capture missing information or an unsettled decision without forcing a label |

Make correction scope explicit: this record only, a general instruction issue,
or a concept-definition/applicability issue. A case correction does not silently
become a broad rule. Optional short notes, source selections and wording edits
should supplement prepared agent proposals.

Persist review-session identity, exact concept/context/procedure revisions,
the reviewed proposal, human action, selected evidence/edits, scope and unresolved
questions. A “finish review” interaction provides a durable session ID, a readable
digest and machine-readable feedback. The coordinator can read it through a
future developer command/API without screenshots or manual copy/paste.

Coordinator follow-through: apply authorized case decisions; group systemic
findings into concrete proposals; prepare examples and impact previews; obtain
approval for ontology changes; create reassessment work; run calibration/audits;
report corrected, pending and published states separately. Prevent duplicate
imports and flag feedback on stale revisions for explicit resolution.

Accepted positive evidence should also support product explanations: concise
claims with important prerequisites/limits and navigation to the actual source
field/owned ability. Developer proposals and rejected explanations remain separate
from published evidence. Explanation and source attribution require validation.

## 10. Future workstreams and validation gates

These workstreams describe dependencies for later planning, not current execution.
The main agent coordinates ontology and integration choices. Read-only research
and independent validation can be delegated; future edits need scoped ownership.

1. **Rebase the design on the finished refactor.** Authenticate checkout/source
   identity, native filter coverage, final content/owned identities, runtime
   ownership and developer-tool architecture. Output: an updated boundary and
   native-filter overlap inventory. Review: confirm no retired model dependencies.
2. **Ontology and applicability pilot.** Agents prepare a small useful set such
   as countermeasures, diagnosis, restraint escape, actor roles and a cross-family
   theme. Human approves definitions/examples. Review: native overlap, direction,
   family assumptions, conditions and multi-home collection behavior.
3. **Assessment and revision contract.** Specify states, weak/strong negative
   basis, coverage, revision invalidation/reuse, source evidence, feedback and
   retention. Output: concrete typed contracts and examples. Independent review:
   no missing work counted as negative, no confidence-only reuse, no fake fresh
   evaluations, no hidden pruning or source visibility leakage.
4. **Developer review prototype.** Establish human approval and durable
   coordinator handoff. Output: proposed interaction flows followed by an
   authorized prototype. Validate case-versus-general scope, evidence navigation,
   stale feedback, resumption, duplicate imports and review-session retrieval.
5. **Calibration and panel experiment.** Human approves prepared cases; run
   controlled reviewer/packet experiments. Output: measured error/cost/recall
   report and chosen policy. Review: held-out integrity, representative cases,
   negative audits, uncertainty and genuinely additional errors discovered.
6. **Whole-corpus assessment.** Freeze an edition, run resumable exhaustive
   work, reconcile and audit. Output: exact coverage and accepted assignments.
   Review: all eligible pairs, failures pending, correct edition/context,
   no unapproved concepts, source-bound evidence and calibrated quality.
7. **Product integration and publication.** Build accepted rows, discovery,
   filtering and explanations through final runtime boundaries. Validate composed
   native/label filters, defined collection semantics, owned evidence, incomplete
   coverage behavior, publication coherence and source changes.

For future code changes, define owning-crate behavior tests, the complete Rust
gate (`just verify` or the then-current tracked equivalent), relevant frontend
typecheck/build/tests and real UI routes. Validate against this plan and final
agreed deliverables before claiming completion. Measure raw-archive/ledger sizes
and runtime costs; arithmetic estimates are not performance evidence.

Future docs work must update tagging architecture, relevant overview/runtime/
artifact docs, backlog intent and ADRs for approved changes. Reconcile old
unanimity, provenance/review-state, single-home presentation and YAML commitments
explicitly. Update contributor/agent guidance to implemented commands only.

Retire old semantic rules and superseded paths through direct replacement;
independently scan for heuristic candidate pruning, shims, duplicate taxonomies,
retired context models and incomplete consumer migrations. Use isolated worktree
implementation if the final scope is large or parallel; invoke the worktree skill
only when implementation is authorized. Commit, push, PR and merge are distinct
future milestones with no authorization from this document.

## 11. Open decisions for continued discussion

1. How much formal claim structure earns its cost compared with defined labels?
2. Which concepts deserve new authored membership versus a named native filter?
3. What makes a negative strong, and which definition changes can safely preserve it?
4. Can selective reuse be validated without rebuilding semantic heuristic machinery?
5. What are the assignment subjects: roots, owned items, prose definitions, or a
   selected combination? How are contained/generated concepts exposed?
6. How broad is applicability by default, and who approves exceptional restrictions?
7. What acceptance policy, reviewer mix, packet size and reconsideration budget
   meet measured quality/recall and human-effort goals?
8. How should product filters handle incomplete/uncertain coverage and exclude-label queries?
9. What is the minimum useful developer review UI and its service/storage owner?
10. Which storage/authoring format best supports review, durable versions, agent
    import, inspection and publication? YAML is one candidate, not a requirement.
11. What run evidence and source snapshots must be retained, and for how long?
12. Which initial concepts best test cross-family semantics and useful native-filter gaps?

Next discussion: settle the two-loop boundary and negative-strength/reassessment
policy with concrete examples, then choose a small concept-review packet. Keep
implementation deferred until the broad refactor finishes.
````
