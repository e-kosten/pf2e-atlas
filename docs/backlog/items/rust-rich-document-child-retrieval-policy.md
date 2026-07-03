# Rust RichDocument Child Retrieval Policy

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-04

## Problem

The RichDocument migration should start with overflow-only child embeddings: materialize child units only when parent embedding budgeting drops or trims rich content. That keeps the first production change focused on replacing lossy content parsing and preserving current retrieval intent.

There is a separate retrieval-quality question: should Atlas also materialize selected child embeddings for records whose parent embedding already fits under the model limit?

General vector-search guidance suggests smaller structure-aware chunks can improve precision, but overly small child units can add noise, duplicate parent results, and crowd out other relevant records. Full `all_children` indexing is therefore unlikely to be a good default without additional deduplication, weighting, or reranking.

## Desired Outcome

Run a retrieval-quality spike after the RichDocument chunking baseline is available in production or a production-like artifact.

The spike should compare overflow-only against targeted expansion policies:

- `overflow_only`: V1 baseline; child embeddings only for parent-over-limit records.
- `body_children`: always materialize child embeddings for the body pack bucket, especially `blurb + description/public_notes`.
- `structured_children`: always materialize child embeddings for rich content with meaningful authored headings or section structure.
- `all_children`: control/upper-bound policy, expected to expose duplicate/noisy-result risks rather than become the default.

The decision should identify whether Atlas should keep overflow-only, add body children, add structured children, or defer broader child indexing until a reranking/deduplication layer exists.

## Evaluation Guidance

Do not decide this from unit counts alone. Use a small judged query set and inspect top results across policies.

Include queries that exercise:

- creature lore and ecology, such as drake eggs, nindoru traits, gold dragon behavior, and deity servitors
- body/blurb matching where a short blurb plus descriptive text should retrieve the right creature
- authored sections inside long records, such as spell repertoire, Earn Income, and explicit table/heading content
- noise-risk content where tiny operational or GM-facing fragments might over-rank

Compare:

- whether the correct record appears
- whether the correct child unit appears
- whether duplicate child hits from one parent crowd out other useful records
- whether tiny child units produce brittle or noisy high-confidence matches
- whether body packing improves recall without diluting relevance

## Constraints

- Keep the RichDocument production migration overflow-only for V1 unless this backlog item is explicitly promoted.
- Reuse the RichDocument parser, semantic renderer, source pack buckets, and tokenizer-gated chunking logic; do not fork parsing policy for the retrieval spike.
- Preserve parent linkage and metadata for every child unit so presentation can collapse, deduplicate, or show parent context.
- Treat `all_children` as a control policy, not the expected production default.

## Related

- [Rust rich source content model](./rust-rich-source-content-model.md)
- [Rust search quality and retrieval weight tuning](./rust-search-quality-tuning.md)
- [RichDocument production migration plan](../../../scratch/plans/2026-06-02-rich-document-production-migration.md)
- [Child embedding strategy decisions](../../../scratch/rich-content-audit/output/child-embedding-strategy-decisions.md)
