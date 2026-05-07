# Core architecture convergence

Status: planned

## Intent

Land the long-term architecture cleanup identified by the May 2026 codebase architecture review outside the rough legacy/derived-tag internals.

The target end state is:

- `src/data/` owns SQLite/index SQL and physical retrieval.
- `src/search/` owns storage-agnostic search execution, ranking, query analysis, and scoring.
- TUI search editing uses canonical `SearchRequest` / `SearchFilterNode` as the only semantic query model.
- Index rebuild code is split into typed data-pipeline stages under `src/data/indexing/`.
- Search discovery and ontology search-semantics assembly are split into focused app/ontology owners.

## Active Plan

- `scratch/plans/2026-05-07-core-architecture-convergence-plan.md`

## Notes

This is architecture-impacting refactor work. It should land as finished end-state changes with no compatibility layers, no mixed old/new ownership, and architecture docs updated in the same task.
