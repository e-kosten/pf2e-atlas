# Rust FTS Tokenization And Stemming Exploration

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-05-17

## Problem

The Rust Phase 6 search baseline should not add custom synonym maps or domain token rewrites for query terms such as `breathes` versus `breath`. Those brittle transformations belong neither in CLI code nor in ad hoc query analysis. At the same time, inflection handling is a real FTS quality concern that can affect natural-language and rules-heavy searches.

SQLite FTS5 supports tokenizer choices such as `unicode61` and the `porter` tokenizer wrapper. Other full-text systems commonly use stemming, lemmatization, dictionaries, and stop-word handling during both document indexing and query analysis. The Rust runtime needs an explicit evaluation of whether and where standard tokenization/stemming improves PF2E search quality.

## Desired Outcome

Evaluate FTS tokenizer and normalization options for the Rust artifact and pick a measured default or confirm that the Phase 6 baseline should stay unchanged.

The exploration should compare at least:

- current Rust `records_fts` tokenizer behavior
- `unicode61` configuration details, including diacritic handling
- `porter unicode61` or equivalent SQLite FTS5 stemming configuration
- stop-word handling for FTS query construction
- OR versus AND token composition and prefix-token behavior where they interact with tokenizer choices
- exact-name/title behavior under any stemming configuration
- natural-language query behavior for inflectional variants such as `breath`, `breathes`, `breathing`, and `breathed`

## Constraints

- Do not add custom PF2E synonym maps or hand-written token rewrites as the first solution.
- Apply any tokenizer/stemmer decision consistently to indexed documents and query construction.
- Preserve exact identity behavior through the resolver/top-tier search path rather than relying on FTS stemming for record names.
- Treat tokenizer changes as artifact-contract relevant if they change `records_fts` creation or validation metadata.
- Use fixture-backed quality results, not isolated anecdotal queries.

## Notes

This item is intentionally separate from Phase 6 implementation. Phase 6 should build safe plain-text FTS query construction and hybrid retrieval plumbing first. This follow-up can then test whether standard stemming improves the FTS lane enough to update artifact schema, validation metadata, and search-quality defaults.

## Related

- [Rust Phase 6 Search Runtime Plan](../../../scratch/plans/2026-05-17-rust-phase-6-search-runtime.md)
- [Rust search quality and retrieval weight tuning](./rust-search-quality-tuning.md)
- [Rust CLI runtime migration research](../rust-cli-runtime/README.md)
