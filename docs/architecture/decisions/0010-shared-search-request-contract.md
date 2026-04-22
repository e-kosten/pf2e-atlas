# ADR 0010: Shared SearchRequest Semantic Contract

- Status: Accepted
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
- MCP, TUI, and ontology-origin flows may keep their own surface-local editing or transport models, but they must adapt to `SearchRequest` before crossing into shared backend code
- `src/search/request-compilation.ts` lowers `SearchRequest` into search-execution filters owned by `src/search/contracts.ts`
- execution filters are search-owned compiled output, not the shared semantic contract
- `src/data/backend/search-service.ts` is the backend control point that compiles, normalizes, validates, and executes search requests
- lint rules should prevent `src/app/**`, `src/domain/**`, `src/server/**`, and `src/tui/**` from importing the search execution DTOs or compiler directly

## Consequences

- new shared search concepts must be added once in `SearchRequest` and then lowered centrally instead of teaching separate surface-specific filter contracts
- TUI query parts remain the preferred local editing model, but they are no longer the owner of cross-surface search semantics
- MCP compatibility input stays at the transport edge instead of leaking execution DTO ownership back into shared callers
- backend search normalization and validation remain search-owned and operate on compiled execution filters rather than on transport- or UI-shaped input
