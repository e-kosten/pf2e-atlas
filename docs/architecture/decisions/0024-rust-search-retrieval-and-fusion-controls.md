# 0024 Rust Search Retrieval And Fusion Controls

## Status

Accepted

## Context

The Rust CLI search surface needs to move beyond Phase 5 filter-only listing and the temporary Phase 4 semantic diagnostic route. The older TypeScript product vocabulary exposes profile-like concepts such as lexical, balanced, and concept. That language reflects historical implementation choices more than the intended Rust product contract.

Rust search also has multiple retrieval mechanisms with different score spaces:

- FTS/BM25 over `records_fts`
- vector retrieval through `document_embedding_cache` and `record_vector_index`
- strict record identity resolution through the same behavior used by `atlas record resolve`

BM25 scores and vector distances are not directly comparable. The Rust runtime needs one stable default result-set command for ordinary users and agents, while still exposing enough advanced controls to evaluate and tune retrieval quality.

## Decision

`atlas search <text>` is the normal ranked text search surface. By default, text search uses hybrid retrieval over precision FTS and vector lanes and fuses record ranks with weighted reciprocal rank fusion. The default lane weights are equal:

```text
fts_weight = 1.0
vector_weight = 1.0
```

The implementation uses one weighted-RRF path so later tuning does not introduce a second fusion engine. FTS evidence also carries a precision policy. The default `demote-weak` policy keeps direct title/alias and strong lexical matches at full FTS strength, discounts medium lexical evidence, and heavily discounts weak lexical evidence so broad token overlap does not crowd out stronger semantic matches.

The CLI exposes advanced retrieval-path controls by retrieval mechanism:

```bash
atlas search "healing magic" --retrieval fts --json
atlas search "healing magic" --retrieval vector --json
atlas search "healing magic" --retrieval hybrid --json
```

`--retrieval fts` must work with record-only artifacts produced by `atlas setup --no-embeddings`. Default hybrid and `--retrieval vector` require embedding/vector readiness. If that readiness is unavailable, the CLI reports `vector_readiness_required` and suggests either rebuilding with embeddings or using `--retrieval fts`.

Fusion controls are advanced controls, not ordinary product modes. `--fusion weighted-rrf` accepts lane weights. If a user explicitly requests unweighted `--fusion rrf`, lane-weight flags are invalid.

Exact identity matches form a top tier above fused FTS/vector results. Search reuses the same strict resolution implementation as `atlas record resolve` for exact name, normalized name, verified alias, and exact full variant-name matches. Identity records are deduplicated out of the fused result list before pagination.

User query text is treated as plain text, not raw SQLite FTS5 syntax. The runtime builds safe FTS syntax from normalized non-stopword tokens, quotes each token, joins tokens with `OR`, and does not use raw user-provided FTS syntax.

The default FTS lane is intentionally precision-oriented rather than full-body lexical search. `atlas-index` queries scoped FTS lanes for:

- title/name and verified alias text
- high-signal facets such as traits and taxonomy terms

`atlas-search` classifies those FTS hits as direct-title, strong-lexical, medium-lexical, or weak-lexical evidence using title/alias token coverage and high-value record tokens. That classification can adjust effective FTS rank and FTS contribution before hybrid fusion. Semantic retrieval owns broad natural-language intent, while FTS primarily supplies deterministic precision evidence for names, aliases, traits, families, and record types. Custom synonym maps, PF2E domain token rewrites, typo correction, stemming, and broader body/reference FTS are follow-up tuning topics, not the default product path.

The Rust CLI should not expose Node-era `balanced` or `concept` profiles as normal product language. If any compatibility mapping is ever needed, it belongs at an explicit edge and must not become the Rust runtime's primary search model.

## Consequences

`atlas-search::AtlasRetrievalService` owns identity tiering, FTS/vector lane orchestration, fusion, result assembly, and search-window shaping. `atlas-index::RetrievalReadIndex` is the composite read contract used by retrieval, with focused sub-capabilities such as `RecordReadIndex`, `IdentityReadIndex`, `FilterReadIndex`, `FtsReadIndex`, `VectorReadIndex`, `ReferenceReadIndex`, `VariantReadIndex`, and `RemasterReadIndex`; `atlas-index::SqliteIndexReader` owns direct SQLite access for FTS, vector, record, and graph retrieval. `atlas-cli` owns only argument parsing, JSON/human presentation, and exit codes.

The temporary `atlas search semantic --query ...` diagnostic branch should be removed once normal `atlas search <text>` covers vector retrieval.

Search JSON remains compact by default. Rank, score, FTS lane, FTS confidence, vector-unit, and query-analysis details belong behind `--explain`, where per-result explain fields are attached to each result's `match` object.

Filter-only search remains SQL-paged. Ranked text search works from bounded candidate windows rather than materializing the whole corpus. Phase 6 should not add a CLI-local window store or public runtime window registry, but the runtime internals should remain compatible with a future Rust TUI window abstraction.

Search quality tuning should prefer measured changes to FTS confidence policy, FTS lane weights, RRF weights, candidate windows, rank constants, tokenizer/stemmer behavior, and vector unit policy before adding generic rerank adjustments.
