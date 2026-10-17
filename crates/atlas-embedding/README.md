# atlas-embedding

This crate owns the two pinned search profiles, `bge-base` and `embeddinggemma`,
their token budgets, source-faithful passage splitting, exact inference reuse
keys and query/document vectors. Fresh setup uses BGE Base; runtime resolves
omitted launch defaults from the stored manifest. Both emit
768-dimensional L2-normalized vectors: BGE uses CLS pooling, while Gemma uses its
exported projected sentence output. FastEmbed owns inference, batching, generic
pooling and normalization; text-splitter owns generic passage boundaries.

Five BGE assets and six Gemma assets are checksum-verified, including Gemma's
external weights, configuration and special-token files. Setup downloads only
pinned files into model-ID/revision directories, repairing selected missing or
checksum-mismatched files through verified temporary download and rename. Old
revisions remain untouched; incomplete packages are retryable and unusable.
Inference never invokes Hub downloads. Central rendering applies
each model's official query/document prefix exactly once. Selected native
tokenization guards complete formatted inputs: BGE512 and Gemma2048, with native
special tokens and no silent truncation.

Callers supply selected identity or HTML/plain-text sections and keep root,
owner and source-field attribution. Prepared inputs carry shared typed passage
addresses, exact input hashes and execution-bound reuse keys. Prose splitting
covers all source bytes using the shared BGE tokenizer: up to32 overlap tokens,
at most256 body tokens,64 explicitly shortened context tokens,480 identity tokens
and512 baseline assembled tokens. Gemma's native capacity does not expand or
resplit that baseline. Gemma-only builds need the small shared BGE tokenizer
asset, without BGE model weights. Persisted profile identities and document reuse
keys bind its full SHA-256 separately from the selected native tokenizer digest.
Raw input hashes identify the unprefixed selected strings.

`SharedQueryInference` retains one session per loaded profile behind one global
native mutex, with four inference threads. Runtime composes this owner across
SQL workers and bounds admission; the mutex releases before SQL ranking.

Source selection and HTML interpretation belong to atlas-record. Artifact rows,
SQLite vectors and persisted byte layout belong to atlas-index. Root aggregation,
filter eligibility, ranking and remaster preference belong to atlas-search.
This crate retains no RichDocument, replacement source DTO or presentation tree.

Run focused tests with `cargo test -p atlas-embedding`. Ignored execution tests
require the pinned local model cache and are run explicitly with `-- --ignored`.
Full-corpus preparation and frozen-vector comparison evidence remain separate
from ordinary tests and from native build or retrieval-quality validation.
