# Search Text Exclusion

Status: deferred
Priority: later
Owner: unassigned
Last reviewed: 2026-10-27

Recover the useful pre-refactor ability to exclude unwanted indexed text from
search results. Current Boolean trait/reference filters do not replace arbitrary
negative text. Define term, phrase and field scope before implementation, and
apply the exclusion consistently before ranking and pagination across lexical,
semantic and hybrid retrieval. Keep it shared across CLI and browser requests;
do not restore the retired search pipeline or scan source files at query time.

Validate exclusions against complete eligible roots, embedded match evidence,
combined structured filters, counts and later pages. Unknown/unsupported syntax
must be an actionable input error rather than silently changing the query.

Related: [search quality](rust-search-quality-tuning.md),
[capability recovery](rust-integration-recovery.md).
