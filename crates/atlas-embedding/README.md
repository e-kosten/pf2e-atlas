# atlas-embedding

This crate owns the verified BGE-small-en-v1.5 execution contract, token budgets,
source-faithful passage splitting, exact inference reuse keys and query/document
vector generation. FastEmbed owns generic inference/batching/CLS pooling/L2
normalization; text-splitter owns generic passage boundaries.

Only the pinned current BGE model is supported. Four immutable local assets are
checksum-verified before use. Asset setup downloads only those pinned files;
inference never invokes Hub downloads. Query text receives BGE's documented
prefix, and document text remains faithful UTF8. Inputs over the actual tokenizer
limit fail before inference; neither library silently clips them.

Callers supply selected identity or HTML/plain-text sections and keep root,
owner and source-field attribution. Prepared inputs carry shared typed passage
addresses, exact input hashes and execution-bound reuse keys. Prose splitting
covers all source bytes with up to32 overlap tokens, at most256 body tokens,
64 explicitly shortened context tokens and512 actual final tokens. Identity
synopses fit480 body tokens and never manufacture source ranges.

Source selection and HTML interpretation belong to atlas-record. Artifact rows,
SQLite vectors and persisted byte layout belong to atlas-index. Root aggregation,
filter eligibility, ranking and remaster preference belong to atlas-search.
This crate retains no RichDocument, replacement source DTO or presentation tree.

Run focused tests with `cargo test -p atlas-embedding`. Ignored execution tests
require the pinned local model cache and are run explicitly with `-- --ignored`.
The selected-corpus proof additionally needs the repository-local immutable
research corpus; it tests preparation, not the production record selector.
