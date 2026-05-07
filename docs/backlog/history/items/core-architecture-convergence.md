# Core architecture convergence

Status: done

## Outcome

The long-term architecture cleanup identified by the May 2026 codebase architecture review is landed outside the rough legacy/derived-tag internals.

The landed state is:

- `src/data/` owns SQLite/index SQL and physical retrieval.
- `src/search/` owns storage-agnostic search execution, ranking, query analysis, and scoring.
- TUI search editing uses canonical `SearchRequest` / `SearchFilterNode` as the only semantic query model.
- Index rebuild code is split into typed data-pipeline stages under `src/data/indexing/`.
- Search discovery and ontology search-semantics assembly are split into focused app/ontology owners.

## Notes

This architecture-impacting refactor landed as direct replacement work:

- physical search retrieval moved to data-owned backend modules behind a storage-agnostic search retrieval port
- the TUI-local metadata filter draft AST was removed in favor of canonical search request mutation/projection helpers
- index rebuild flow now passes typed stage artifacts through `src/data/indexing/`
- app-facing search discovery and ontology search-semantics assembly now have focused owner modules
- architecture docs and ADRs describe the current intended ownership

No compatibility migration path remains for the retired broad helper files or old semantic models.
