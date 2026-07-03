# 0018 Rust Default Embedding Model

## Status

Accepted

## Context

The Rust runtime initially used MiniLM as the embedding baseline because it matched the TypeScript provider and gave a low-risk parity target. Once the Rust embedding catalog, deterministic ingest, semantic modes, and evaluation harness existed, the runtime needed a product default chosen from measured search quality and runtime cost rather than migration convenience.

The embedding comparison harness evaluated candidate models across the PF2E corpus with blind rank-order scoring. The final comparison used the same indexed corpus and scored each query across semantic modes (`parent-only`, `chunks`, and `weighted-chunks`) without exposing model identities to the reviewer. The run artifacts remain local scratch outputs, but the durable findings are:

- `all-mpnet-base-v2` had the strongest quality score, with average rank about `1.47`, but required about `14m 37s` to build and produced a larger artifact.
- `nomic-embed-text-v1.5` scored close behind, with average rank about `1.53`, but had the highest build cost at about `19m 24s`.
- `bge-small-en-v1.5` had average rank about `1.64`, built in about `6m 47s`, used a smaller artifact footprint than the 768-dimensional candidates, and remained top-3 for nearly every scored query.
- `bge-base-en-v1.5` did not justify its higher build/query cost in this corpus.
- `weighted-chunks` tied `parent-only` on aggregate rank while preserving child-unit recovery for long documents and avoiding the small regression seen with unweighted `chunks`.

The evaluation still supports keeping higher-quality alternates available for periodic comparison, but the built-in default should optimize the normal local runtime path, not the quality ceiling alone.

## Decision

Use `bge-small-en-v1.5` as the Rust runtime's built-in default embedding model.

Use `weighted-chunks` as the built-in semantic search mode where the CLI or runtime needs a default semantic mode.

Keep model identity centralized in `atlas-embedding`:

- `DEFAULT_EMBEDDING_MODEL` is `EmbeddingModelId::BgeSmallEnV15`.
- The `default` model alias resolves through `DEFAULT_EMBEDDING_MODEL`.
- Explicit MiniLM aliases remain supported for parity checks and older artifacts.

The default embedding metadata written into Rust artifacts is:

- provider family: `onnx-mean-pooling`
- model id: `BAAI/bge-small-en-v1.5`
- tokenizer id: `BAAI/bge-small-en-v1.5`
- dimensions: `384`
- pooling: `mean`
- normalization: `l2`
- distance metric: `cosine`
- document prefix: empty
- query prefix: `Represent this sentence for searching relevant passages: `

## Consequences

Default artifact rebuilds require the BGE small model cache to be installed under the configured embedding cache root. Existing MiniLM artifacts remain valid only when the runtime is explicitly pointed at MiniLM or when their metadata is validated against the matching catalog entry.

Changing the default model changes `document_embedding_cache`, `record_vector_index`, embedding metadata, semantic input hashes where prefixes differ, and search ranking behavior. Treat default-model changes as search-quality changes that require a fresh index build and evaluation pass.

`all-mpnet-base-v2` remains the quality-ceiling comparison candidate. It should not become the default unless later scoring shows a larger quality gap or runtime constraints change enough that the larger build/query cost is acceptable.

Raw `chunks` should remain available for diagnostics, but `weighted-chunks` is the production-oriented default because it allows long-document child units to recover records without letting child units dominate parent-level retrieval as aggressively.
