# Source-backed Search Quality Evaluation

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-17

## Current Baseline

Precision FTS covers identity, verified aliases, typed vocabulary and named
owned definitions. Semantic selection covers selected root and owned explanatory
passages. Search uses one accepted semantic contribution per root and fuses root
ranks with lexical specificity tiers. Eligibility and bounded candidate selection
follow [ADR 0046](../../architecture/decisions/0046-source-backed-reference-artifact.md).

The product's selectable baseline profiles (`bge-base` and `embeddinggemma`) use
the shared storage/input/session contract in
[ADR0050](../../architecture/decisions/0050-selectable-search-profiles.md). BGE Base
is the operational default; this does not select a semantic quality winner or
authorize a new ranking experiment. Further quality evaluation below remains open.

Local full-corpus experiments exposed tradeoffs between descriptive discovery and
useful owner capabilities. No experimental ranking policy has been selected for
production. The [dated evaluation notes](../../research/source-backed-search-evaluation.md)
preserve findings, sample sizes, evidence limits and local artifact references.

## Next Evaluation

Evaluate FTS independently, then evaluate its fusion with semantic retrieval.
Lexical retrieval has not received the same depth of evaluation as the semantic
models. Separate candidate recall from ordering: inspect tokenization, partial
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
