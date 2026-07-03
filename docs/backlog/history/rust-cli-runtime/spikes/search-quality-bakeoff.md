# Search Quality Bakeoff Spike

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-05-06

## Question

Which retrieval pipeline actually improves PF2E search quality enough to justify runtime and artifact changes?

## Motivation

Current lexical and semantic quality is only acceptable, not excellent. The migration should not preserve the current retrieval stack by inertia if better lexical engines, embedding models, hybrid scoring, or reranking materially improve results.

## Prototype Scope

- Build a PF2E-specific eval set with real queries and expected relevant records.
- Include category-scoped, broad, rule, name-heavy, phrase-heavy, semantic-intent, and tag-style queries.
- Compare candidate pipelines:
  - current SQLite FTS5 plus MiniLM baseline
  - SQLite FTS5 plus better embedding models
  - SQLite FTS5 plus better embeddings plus reranker
  - Tantivy lexical plus vector candidates plus hybrid scorer
  - Tantivy lexical plus vector candidates plus reranker
  - LanceDB retrieval path if the LanceDB spike is feasible
- Capture quality and operational metrics.

## Do Not Mock

- real PF2E searchable text
- field weighting
- structured filters
- representative categories
- semantic inputs
- candidate merging
- final ranking

## Metrics

- recall@10
- MRR
- nDCG@10 where judgments are available
- latency
- index size
- startup cost
- implementation complexity
- packaging risk

## Outputs

- eval corpus and scoring script
- comparison report
- recommended retrieval stack
- explicit rejected alternatives and why

## Migration Dependency

The Rust migration should adopt the retrieval architecture selected by this bakeoff rather than merely porting the current TypeScript ranking behavior.
