# atlas-ingest

`atlas-ingest` loads and admits authored Foundry source, preserves developer diagnostics, resolves source references, and prepares a source-backed artifact for `atlas-index` to validate and publish atomically.

The build consumes the checked DTOs directly. It prepares sanitized HTML and interaction caches under an explicit indexing locale with source-English fallback, selects attributed lexical/semantic units using `atlas-record`, and optionally runs the pinned local embedding model through `atlas-embedding`. A lexical build does not activate native vector tables. Before reusing a vector, ingest reconstructs its old input from the checked snapshot, prepared cache, pack label and verified aliases using the pinned tokenizer and the same input assembler. Hash, token count, passage address and chunk ordinal must match exactly. Old roots are reconstructed once through bounded batched reads; current attribution is always regenerated. Offline index validation remains independent of the tokenizer.

Every relevant pack file, manifest binding, locale catalog and migration declaration participates in the aggregate content fingerprint, including invalid or unaddressable files excluded from product records. Source bytes stay in the source checkout and ingest developer outcomes; the database stores the checked compressed typed snapshot, per-record path/hash, derived query projections and compact prepared caches.

`analyze_foundry_source` reports typed admission and selection counts. `audit_source_paths` samples raw source paths for developer discovery, without claiming those paths have a product consumer. Database schema, validation, SQL and atomic publication belong to `atlas-index`; runtime paths, downloading and CLI presentation belong to their respective owners.
