# ADR 0010: Shared SearchRequest Semantic Contract

- Status: Superseded in part by ADR 0011 and ADR 0013
- Date: 2026-04-21

## Context

Search meaning had started drifting across surfaces:

- MCP transport inputs carried top-level fields such as `query`, `subcategory`, `rarity`, and `actionCost`
- the TUI had moved to a structured query-part model
- ontology query carriers still exposed search behavior through legacy `kind` plus `filters` payloads
- backend and search code still treated the execution filter DTO as if it were the shared cross-surface contract

That mixed ownership made it too easy for one surface to gain a new search concept without the others following the same semantic model.

## Decision

- `src/domain/search-request-types.ts` owns `SearchRequest` as the shared semantic query contract
- `src/domain/metadata-field-types.ts` and `src/domain/metadata-filter-types.ts` own the metadata query vocabulary and AST carried by `SearchRequest`
- MCP, TUI, and ontology-origin flows may keep their own surface-local editing or transport models, but they must adapt to `SearchRequest` before crossing into shared backend code
- `src/search/request-compilation.ts` lowers `SearchRequest` into search-execution filters owned by `src/search/contracts.ts`
- execution filters are search-owned compiled output, not the shared semantic contract
- `src/data/backend/search-service.ts` is the backend control point that compiles, normalizes, validates, and executes search requests
- lint rules should prevent `src/app/**`, `src/domain/**`, `src/server/**`, and `src/tui/**` from importing the search execution DTOs or compiler directly
- ontology query carriers are `request`-only and do not preserve legacy `kind` plus `filters` fallback shapes
- domain modules must not import `src/search/**`; shared query meaning moves outward into `src/domain/`, while search keeps execution-only types

## Consequences

- new shared search concepts must be added once in `SearchRequest` and then lowered centrally instead of teaching separate surface-specific filter contracts
- TUI query parts remain the preferred local editing model, but they are no longer the owner of cross-surface search semantics
- MCP compatibility input stays at the transport edge instead of leaking execution DTO ownership back into shared callers
- metadata field names and metadata boolean-group structure can evolve once in the domain contract and then be reused by MCP schemas, ontology queries, TUI editors, and search execution
- backend search normalization and validation remain search-owned and operate on compiled execution filters rather than on transport- or UI-shaped input

## Superseded Notes

ADR 0011 replaces the older shared-contract shape assumptions that kept `intent` / `parts` as the durable public model. ADR 0013 also replaces the assumption that the TUI may keep query parts as its preferred durable local editing model.
