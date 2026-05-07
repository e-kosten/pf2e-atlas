# Tantivy Lexical Search Spike

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-05-06

## Question

Does Tantivy provide enough lexical relevance improvement over SQLite FTS5 to justify a separate lexical search artifact?

## Motivation

PF2E search has many lexical-heavy needs: exact names, phrases, rules terms, traits, actions, source labels, and short mechanical text. SQLite FTS5 is simple and local, but Tantivy may provide better BM25 behavior, phrase handling, tokenization control, field weighting, and facets.

## Prototype Scope

- Build a Tantivy index over a representative PF2E record slice and, if practical, the full corpus.
- Index separate fields for name, category, traits/tags, source, summary, rules text, and full searchable text.
- Test field boosts, phrase queries, tokenization, stemming or no-stemming, facets, and structured filter equivalents.
- Compare results against SQLite FTS5 for the search-quality bakeoff query set.

## Do Not Mock

- real PF2E text
- category/subcategory filters
- trait/tag filters
- phrase search
- field weighting
- index build time
- index size
- query latency

## Outputs

- Tantivy index schema recommendation
- lexical quality comparison against FTS5
- artifact sync complexity notes
- recommendation: SQLite-only, SQLite plus Tantivy, or defer

## Migration Dependency

If Tantivy wins, the runtime architecture should treat SQLite as the canonical store and Tantivy as a lexical retrieval artifact unless a later design proves a stronger single-store option.
