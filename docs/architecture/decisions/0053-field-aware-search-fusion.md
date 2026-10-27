# Field-aware lexical retrieval and comparison of hybrid ranking

Status: Accepted
Date: 2026-10-18

## Decision

Use one lexical projection: separate root names and verified aliases with root
vocabulary, embedded names with their own vocabulary, and section labels alone.
Keep exact primary/alias priority. Aggregate each root by its strongest weighted
lexical row, with root/alias weight 1, root-section .5 and embedded .25. Multiple
supporting matches do not accumulate influence.

Keep two bounded, request-level hybrid strategies for dogfooding: RRF60
and min-max score fusion (default) with lexical weight .25 and semantic weight .75. Constant
nonempty lanes receive their lane weight. Embedding profile remains independent.
Candidate admission, semantic maximum-per-root scoring, filters and edition
preference retain their existing contracts. Similar-record search is unaffected.

`atlas-domain` owns the choice vocabulary, `atlas-search` owns fusion, and
`atlas-index` owns lexical rows and their offline maintenance upgrade. CLI and web
only transport the selection. No user-adjustable weights or old lexical policy
are retained. Switching ranking requires no rebuild or document inference.

Attribute rules and seeded Random order are later presentation orders over the complete admitted root
set, after the existing edition preference and before paging. They neither change
fusion inputs nor apply exact-name/alias relevance tiers ahead of the chosen
attribute. Score/RRF continues to determine relevance order independently; the
admitted roots, coverage and match witnesses remain available under either order.
Relevance, Random and attribute sequences are alternative ordering modes; neither
Random nor an attribute tie breaker adds a retrieval lane or blending weight.

## Rationale and consequences

Full-corpus evaluation favored the field-aware lexical policy across both embedding
models. The smaller difference between fusion strategies changed direction under
judgment sensitivity analysis, so personal product use will inform the final choice.
The intended end state is one lexical policy and one fusion strategy. The temporary
alternative is a small explicit enum and shared lane-scoring function, not a general
ranking plugin framework.

Lexical version changes rebuild only lexical rows from retained source/content.
Schema migration adds separate alias rows without changing prior migration checksums.
Saved state, semantic inputs, vector bytes and model checkpoints are preserved.

[ADR0060](0060-core-sections-and-search-scope.md) restricts these existing rows in
Core scope. It preserves their vocabulary, postings, BM25 weights and shared
corpus statistics; descriptive prose remains outside precision FTS.
