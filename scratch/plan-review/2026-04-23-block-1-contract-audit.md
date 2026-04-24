# Block 1 Contract Audit Note

Plan: `scratch/plans/2026-04-22-remove-isunique-filter-shape-convergence-plan.md`

Scope audited for Block 1:

- `Slice A`
- `Slice B`
- `Slice C`
- `Slice D`
- `Slice J`

## Final Discriminated Union Shape

Canonical shared request shape after Block 1:

- `BrowseRequest`
  - `mode: "browse"`
  - `filter?: SearchFilterNode`
  - `offset?: number`
  - `limit?: number`
  - `sort?: BrowseSortSpec`
- `SearchModeRequest`
  - `mode: "search"`
  - `search.query: string`
  - `search.exclude?: string`
  - `search.profile?: "lexical" | "balanced" | "concept"`
  - `explain?: boolean`
  - `filter?: SearchFilterNode`
  - `offset?: number`
  - `limit?: number`
- `LookupRequest`
  - `mode: "lookup"`
  - `search.query: string`
  - `filter?: SearchFilterNode`
  - `offset?: number`
  - `limit?: number`
  - `sort?: LookupSortSpec`

Primary owner:

- `src/domain/search-request-types.ts`

## Final Atomic Clause Inventory

Canonical `SearchFilterNode` inventory after Block 1:

- `pack`
- `scope`
- `level`
- `price`
- `rarity`
- `actionCost`
- `linksTo`
- `metadataPredicate`
- `metric`
- `metricCompare`
- `anyOf`
- `allOf`
- `not`

Primary owners:

- `src/domain/search-request-types.ts`
- `src/domain/search-filter-metadata.ts`
- `src/domain/search-filter-operators.ts`
- `src/domain/search-field-domains.ts`

## Deleted Old Names And Shapes

Removed from shared/public search contract surfaces:

- top-level `intent`
- top-level `parts`
- top-level `filters`
- flat root filter fields on public MCP search/list requests:
  - `category`
  - `subcategory`
  - `scopes`
  - `pack`
  - `levelMin`
  - `levelMax`
  - `priceMin`
  - `priceMax`
  - `rarity`
  - `actionCost`
  - `linksTo`
  - `linksToMode`
  - `excludeLinksTo`
  - `metadata`
- canonical filter mini-languages and overlapping wrappers:
  - `SearchFilterPolicy`
  - canonical `excludeLinksTo`
  - canonical embedded `exclude`
  - canonical `includesAny`
  - canonical `includesAll`
  - canonical `values` payloads in public/shared filter leaves
  - canonical clause-level `any` / `all` wrappers
- searchable/filterable metadata vocabulary entry:
  - `isUnique`

Still present by design after Block 1:

- TUI-local query-builder state that still uses `queryText` and root `parts` as internal editor sugar
  - current owners:
    - `src/tui/search/service-types.ts`
    - `src/tui/search/query-state.ts`
    - `src/tui/search/query-parts.ts`
  - this is deferred to `Slice G: TUI Literal Adoption Of The Shared Union`
  - Block 1 removed the old names from the shared/public request contract, MCP transport contract, adapters, and runtime lowering, but it did not complete the later TUI workspace migration

## Serialized Shapes Rewritten Directly

Rewritten directly in Block 1:

- shared/public `SearchRequest` and `SearchFilterNode` payloads crossing:
  - server adapters
  - search runtime compilation
  - ontology query request payloads
  - filter-explorer inspect/open request payloads
  - search test fixtures

Not rewritten in Block 1:

- no durable persisted search-state artifact was found that required migration
- TUI-local draft/query-builder state remains runtime-only and is deferred to `Slice G`

Representative rewritten owners:

- `src/server/search-request-adapter.ts`
- `src/server/tool-schemas.ts`
- `src/search/request-compilation.ts`
- `src/app/ontology/search-semantics-domain.ts`
- `src/app/ontology/inspect-and-open-explorer.ts`
- `src/tui/filter-explorer/controller-inspect.ts`
- `tests/helpers/search-request-fixture.ts`

## Audit Evidence

Code-search targets used for the Block 1 owner audit:

- `rg -n "\\bintent\\b|\\bparts\\b|intent:|parts:" src tests`
- `rg -n "isUnique" src`
- `rg -n "excludeLinksTo|levelMin|levelMax|linksToMode|metadata" src/server/register-search-tools.ts`
- `rg -n "mode: \\\"browse\\\"|mode: \\\"search\\\"|mode: \\\"lookup\\\"" src tests`

Audit conclusion:

- shared/public contract owners no longer expose `intent` or `parts`
- public MCP list/search inputs are canonical `filter` payloads, not flat root filter fields
- `isUnique` is removed from search/filter vocabulary while remaining available on runtime record shapes for ranking and other non-filter logic
- remaining local TUI query sugar is an explicit later-slice migration target rather than an untracked compatibility helper in the shared/public contract
