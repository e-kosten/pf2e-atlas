# Source-backed Search Quality Evaluation

Status: in_progress
Priority: soon
Owner: configurable-similarity workstream
Last reviewed: 2026-10-27

## Configurable Similar

Similar supports Passage matches with normalized geometric rank decay,
Content average, and the existing record/trait baselines over stored vectors.
Advanced Compare parts selects exact existing source sections; candidate matching
can use any prose or explicit cross-kind correspondence roles. Role eligibility
must constrain candidate retrieval before bounded KNN as well as final scoring.

The browser has one shared Search settings dialog for the model, existing hybrid
ranking choice, Similar method, decay and advanced part selection. Apply commits
one request and remembers general preferences in localStorage. Explicit search
links override saved launch defaults without saving them automatically; exact
part selections remain seed/generation-specific. CLI part discovery and the same
scoring/evidence contracts use the shared app service.

Record Similar defaults to Passage matches with Any content and decay 0.5;
trait Similar retains the Trait definition baseline. Explicit links and valid
remembered choices override those independent defaults. The
[Similar evaluation summary](../../research/similarity-evaluation.md)
preserves fixed-pool ordering, controlled-subset candidate recall, model-assisted
usefulness judgments and runtime measurements across both stored profiles.
These results do not establish full-corpus recall or a universal relevance winner.

Highlighting arbitrary text, new segmentation/models, summaries and learned
reranking remain outside this scope. The lexical/hybrid algorithms below are
unchanged by moving their existing chooser into the shared settings dialog.
Broader lexical and associated-content evaluation remains open.

## Core sections extension

Initial opt-in implementation is in progress: independent All/Core destination
scope and All/Core/Custom source selection, typed recipes including vehicles,
armies, PCs and familiars, and identity-plus-prose Core with identity-only fallback.
The metadata projection restricts existing lexical/vector units without adding
prose to FTS or generating embeddings. See
[ADR0060](../../architecture/decisions/0060-core-sections-and-search-scope.md).
The [typed Core census](../../research/core-section-census.md) records retained
source coverage and operational-probe limits separately from relevance evidence.

The broader relevance/decay evaluation is a separate remaining milestone. Compare
identity-only, Core prose-only and combined representation, independent source
and candidate restrictions, equipment's own versus intrinsic spell content, and
multiple decay values. Extend fresh cohorts to all four added actor families;
keep candidate admission, fixed-pool ordering, witness support and runtime
evidence separate. Existing exposed Similar examples and the old holdout are
development evidence. Retain current defaults until a frozen confirmation
supports changing them.

## Current Baseline

Precision FTS covers identity, verified aliases, typed vocabulary and named
owned definitions. Semantic selection covers selected root and owned explanatory
passages. Search uses one accepted semantic contribution per root and fuses root
lanes with exact primary/alias priority and selectable Score-based/Rank-based fusion. Eligibility and bounded candidate selection
follow [ADR 0046](../../architecture/decisions/0046-source-backed-reference-artifact.md).

The product's selectable baseline profiles (`bge-base` and `embeddinggemma`) use
the shared storage/input/session contract in
[ADR0050](../../architecture/decisions/0050-selectable-search-profiles.md). BGE Base
is the operational default; this does not select a semantic quality winner or
authorize a new ranking experiment. Further quality evaluation below remains open.

Local full-corpus experiments exposed tradeoffs between descriptive discovery and
useful owner capabilities. The approved field-aware lexical policy and temporary
Score-based/Rank-based comparison follow [ADR0053](../../architecture/decisions/0053-field-aware-search-fusion.md).
Score-based is the user-selected dogfooding default, not a demonstrated universal
quality winner. The [cross-experiment synthesis](../../research/search-evaluation-synthesis.md) and
[curated grading/ranking archive](../../research/evaluations/2026-10-search/README.md)
preserve the completed comparisons and rejected hypotheses. The [dated evaluation notes](../../research/source-backed-search-evaluation.md)
preserve findings, sample sizes, evidence limits and local artifact references.

## Continuing Evaluation

Evaluate FTS independently, then evaluate its fusion with semantic retrieval.
The completed lexical/fusion comparison is recorded in the evaluation notes.
For further refinement, separate candidate recall from ordering: inspect tokenization, partial
primary-title matching, verified aliases, embedded-title matches, specificity
tiers and candidate limits before attributing poor hybrid ranking to fusion.
Compare FTS-only, semantic-only and hybrid results over the same full corpus and
query/filter conditions, including both available semantic profiles. Preserve
natural-language discovery while improving short name and partial-name searches.

User observation (2026-10-17): searching `harrowkin` placed **Fabled Harrowkin**
on the first page, but below the first few results despite the primary-title
match. Record this as an exposed development case, not an established root cause
or a requirement that this particular record always rank first. Capture the
profile, filters, competing results, lexical/semantic ranks and contributing
evidence when reproducing it. Add fresh comparable partial-name queries across
record kinds so changes are not tuned to this one example. Evaluate primary-name
relevance against embedded matches and semantic relevance explicitly.

Compare the distinct roles of primary identity, the root's own explanation,
intrinsic local mechanics and associated content. Preserve useful exact embedded
name discovery and genuine queries for an owner's capabilities while reducing
noisy partial-label and associated-body influence where evidence supports it.

The proposed work should:

- Compare partial embedded-label influence separately from primary-name behavior
  and exact embedded-name specificity.
- Test reduced associated semantic influence with one contribution per root,
  using matched candidate budgets and explicit source attribution.
- Evaluate self-description and intrinsic mechanics as separate evidence classes;
  their usefulness depends on the query, not an implicit product intent setting.
- Probe paragraph/context embedding inputs separately from ranking changes over
  existing vectors. Define source coverage, reuse identity and inference cost
  before generating inputs.
- Include masked-spirit paraphrases, repeated-match observations and actual
  associated-only owner positives. Keep exposed development cases distinct from
  a fresh source-grounded holdout and cluster related scenarios.

Freeze candidate policies and evaluation rules before opening the holdout. Judge
root relevance and witness support separately; use source evidence to resolve
consequential disagreements. A small target-only embedding probe establishes
input behavior, not full-corpus ranking quality. Preserve exact lookup and broad
semantic discovery while evaluating changes.

## Execution Boundaries

Reuse the pinned artifact and model where input identity permits. Agree resource
budgets and any full-corpus rebuild separately. New models, learned rerankers,
search backends, product intent controls and narrative FTS expansion require their
own evidence and design decision. Keep ordinary filters explicit user criteria.

Maintained evaluation tooling, if needed, belongs in a reviewed developer surface.
The current local plan is under scratch/plans, indexed by its local README;
implementation and publication require authorization for that scope.

Related: [tokenization](rust-fts-tokenization-stemming.md),
[query spelling](rust-cli-typo-tolerant-discovery.md),
[completed shared UI consistency](../history/items/shared-ui-consistency-and-saved-lists.md).
