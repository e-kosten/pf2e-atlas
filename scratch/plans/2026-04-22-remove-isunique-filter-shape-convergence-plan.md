# Remove `isUnique` Metadata + Filter Shape Convergence

## 1. Summary

This plan covers one coordinated search-contract cleanup pass anchored to these backlog items:

- `docs/backlog/items/remove-isunique-metadata.md`
- `docs/backlog/items/filter-shape-convergence.md`

The end state is:

- `isUnique` is not exposed as searchable/filterable metadata
- MCP and TUI use the same canonical search request model
- the canonical request model uses `mode` / `filter`, not `intent` / `parts`
- the canonical filter model uses one boolean composition mechanism (`anyOf` / `allOf` / `not`) rather than multiple overlapping `and` / `or` / `any` / `all` / `exclude` mechanisms
- browse, search, and lookup become genuinely different request shapes instead of one loose object with mode-like flags
- the TUI adopts that same discriminated union directly rather than maintaining a parallel query-state model

This is planning only. No tracked implementation edits should happen on `main`; implementation belongs in a dedicated `/tmp` worktree.

## 1.1 Target Data Model

The intended long-term shared search shape for both MCP and TUI is a discriminated union with a small shared base. It is not a flat MCP root filter shape, and it is not a public/shared field named `parts`.

Representative target shape:

```ts
type SearchRequestBase = {
  filter?: SearchFilterNode;
  offset?: number;
  limit?: number;
};

type BrowseSortSpec =
  | { kind: "alphabetical" | "levelAsc" | "levelDesc" }
  | { kind: "random"; seed?: number };

type LookupSortSpec = {
  kind: "alphabetical" | "levelAsc" | "levelDesc";
  policy?: "tiered" | "global";
};

type BrowseRequest = SearchRequestBase & {
  mode: "browse";
  sort?: BrowseSortSpec;
};

type SearchModeRequest = SearchRequestBase & {
  mode: "search";
  search: {
    query: string;
    exclude?: string;
    profile?: "lexical" | "balanced" | "concept";
  };
  explain?: boolean;
};

type LookupRequest = SearchRequestBase & {
  mode: "lookup";
  search: {
    query: string;
  };
  sort?: LookupSortSpec;
};

type SearchRequest = BrowseRequest | SearchModeRequest | LookupRequest;

type SearchFilterNode =
  | { kind: "pack"; value: string }
  | {
      kind: "scope";
      category: SearchCategoryInput;
      subcategory:
        | { kind: "any" }
        | { kind: "eq"; value: SearchSubcategoryInput }
        | { kind: "isNull" }
        | { kind: "isNotNull" };
    }
  | {
      kind: "level";
      match:
        | { kind: "eq"; value: number }
        | { kind: "gte"; value: number }
        | { kind: "lte"; value: number }
        | { kind: "between"; min: number; max: number };
    }
  | {
      kind: "price";
      match:
        | { kind: "eq"; value: number }
        | { kind: "gte"; value: number }
        | { kind: "lte"; value: number }
        | { kind: "between"; min: number; max: number };
    }
  | {
      kind: "rarity";
      match:
        | { kind: "eq"; value: string }
        | { kind: "isNull" }
        | { kind: "isNotNull" };
    }
  | {
      kind: "actionCost";
      match:
        | { kind: "eq"; value: number }
        | { kind: "gte"; value: number }
        | { kind: "lte"; value: number }
        | { kind: "between"; min: number; max: number }
        | { kind: "isNull" }
        | { kind: "isNotNull" };
    }
  | { kind: "linksTo"; target: string }
  | { kind: "metadataPredicate"; predicate: MetadataAtomicPredicate }
  | { kind: "metric"; metric: string; op: MetricOp; value: string | number | boolean }
  | { kind: "metricCompare"; leftMetric: string; op: NumericMetricOp; rightMetric: string }
  | { kind: "anyOf"; children: SearchFilterNode[] }
  | { kind: "allOf"; children: SearchFilterNode[] }
  | { kind: "not"; child: SearchFilterNode };

type EqualityOperator = "eq" | "notEq";
type OrderingOperator = "gt" | "gte" | "lt" | "lte";
type SetOperator = "includes";
type RangeOperator = "between";
type NullOperator = "isNull" | "isNotNull";

type ScalarOperator = EqualityOperator | OrderingOperator | RangeOperator | NullOperator;
type CollectionOperator = SetOperator | NullOperator;
type MetricOp = ScalarOperator;
type NumericMetricOp = EqualityOperator | OrderingOperator;

type MetadataAtomicPredicate =
  // Representative shape, not the full final exhaustive union.
  // The important rule is that predicates are atomic and do not
  // embed boolean composition inside the payload.
  | { field: "traits"; op: CollectionOperator; value?: string }
  | { field: "rarity"; op: ScalarOperator; value?: string }
  | { field: "level"; op: ScalarOperator; value?: number; min?: number; max?: number };

type ValueOrderingSpec =
  | { kind: "alpha" }
  | { kind: "countDescThenAlpha" }
  | { kind: "numericAsc" }
  | { kind: "declared" };

type ValueDomainSpec =
  | {
      kind: "closedEnum";
      values: readonly string[];
      normalization: "lowercaseTrim" | "normalizedText" | "custom";
      ordering?: ValueOrderingSpec;
      unknownPolicy: "reject" | "drop" | "preserve";
    }
  | {
      kind: "boundedNumber";
      values?: readonly number[];
      ordering?: ValueOrderingSpec;
      unknownPolicy: "reject" | "drop" | "preserve";
    }
  | {
      kind: "openString";
      normalization: "lowercaseTrim" | "normalizedText" | "derivedTag" | "custom";
      ordering?: ValueOrderingSpec;
    }
  | {
      kind: "freeText";
      normalization: "normalizedText" | "custom";
      ordering?: ValueOrderingSpec;
    };

type SearchFieldDomainRegistry = {
  rarity: {
    filterKind: "rarity";
    valueDomain: {
      kind: "closedEnum";
      values: ["common", "uncommon", "rare", "unique"];
      normalization: "lowercaseTrim";
      ordering: { kind: "declared" };
      unknownPolicy: "reject";
    };
  };
  actionCost: {
    filterKind: "actionCost";
    valueDomain: {
      kind: "boundedNumber";
      values: [0, 1, 2, 3];
      ordering: { kind: "declared" };
      unknownPolicy: "reject";
    };
  };
};
```

## 1.2 Data Modeling Decisions

These decisions are part of the implementation target, not optional follow-up:

- `mode` is the top-level discriminant. `intent` should be removed everywhere.
- `filter` is the canonical shared/public field name once the canonical model becomes a tree. `parts` should be removed everywhere.
- The TUI should adopt this discriminated union literally rather than maintaining a separate UI-only query-state shape.
- `browse` does not carry a `search` object at all.
- `search` requires `search.query` and may carry `search.exclude` plus `search.profile`.
- `lookup` requires `search.query` and does not carry `profile`.
- `search.exclude` remains part of text-retrieval behavior. It is not boolean filter negation.
- `explain` is valid only on `mode: "search"`.
- `sort` is not a universally shared field.
  - `browse` allows explicit user sort.
  - `search` does not expose user sort; ranked retrieval order is the product behavior.
  - `lookup` may expose lookup-specific sort.
- All result-set constraints live in the root `filter` tree, including pack, scope, links, ranges, metadata, and other narrowing logic.
- `category`, `subcategory`, and `scopes` collapse into one atomic `scope` clause kind in the canonical model.
  - Use one leaf per category/subcategory pair.
  - Full-category selection and explicit null-subcategory selection must be distinguishable.
  - Multi-scope and multi-subcategory behavior should be expressed through tree composition or lowered UI sugar, not separate root fields.
- The canonical model should have one boolean composition mechanism:
  - `anyOf`
  - `allOf`
  - `not`
- `anyOf` / `allOf` children do not need to be the same clause kind. Heterogeneous boolean composition is allowed and required.
- The canonical model should not keep overlapping embedded boolean mini-languages such as:
  - `SearchFilterPolicy`
  - embedded `exclude` fields
  - `excludeLinksTo`
  - `includesAny` / `includesAll`
  - `any` / `all` payloads
  - implicit sibling-AND list semantics
- Presentation-layer sugar is acceptable, but it must lower into the canonical atomic-plus-tree model before crossing the shared contract boundary.
- Metadata predicates should be mixed directly into the same top-level filter tree as the other clauses.
  - Keep one generic `anyOf` / `allOf` / `not`, not separate metadata-only boolean structures.
- Canonical operator tokens should have one shared owner.
  - Define shared operator domains such as equality, ordering, range, collection, and null operators in one place.
  - Metadata predicates and metric predicates should reference those shared operator types rather than defining parallel operator vocabularies.
  - Surface sugar like `>=` should normalize once into canonical operator tokens rather than being redefined per predicate family.
- Operators such as `eq`, `gte`, `lte`, `between`, and `includes` belong inside predicate leaves, not in the boolean tree itself.
- Keep a real boundary between ordinary metadata fields and keyed metric predicates.
  - Ordinary typed metadata fields should stay under a generic `metadataPredicate`.
  - Keyed metrics should use dedicated `metric` and `metricCompare` leaf kinds rather than being folded into the ordinary metadata-field registry.
  - Do not split metric leaves by actor versus item in the canonical public shape; the metric key space and scoped corpus are sufficient.
- Canonical metadata predicates should stay atomic and avoid plural payloads or ad hoc wrappers.
  - Scalar predicates should use `value`.
  - Collection membership predicates should also use singular `value`, with multiplicity expressed through `anyOf` / `allOf`.
  - Range predicates should use `min` / `max`.
  - Null operators should carry no value payload.
  - Mixed `value` / `values` payload styles are acceptable only as edge sugar that lowers into the canonical atomic tree before persistence or compilation.
- First-class non-metadata leaves may use small field-specific matcher objects where the field needs more than a single literal payload.
  - This is required for null/presence semantics on fields like `scope` subcategory and `actionCost`.
  - This is also the better long-term shape for numeric first-class filters like `level`, `price`, and `actionCost`, because equality and bounded comparison are all meaningful.
- Promoted first-class fields should declare explicit shared value domains when they have stable expected value sets or stable non-default ordering.
  - `rarity` should be modeled as a closed declared domain rather than a loose string with only ordering hints.
  - `actionCost` should be modeled as a bounded declared numeric domain for the currently supported numeric action-count cases.
  - The declared expected domain and the currently present corpus values are separate concerns.
- `pack` should have a clear identity/presentation split.
  - The canonical filter value should be the stable pack machine name, not the display label.
  - TUI rendering and pack-selection affordances should use pack labels for human-facing presentation.
  - MCP/TUI edge parsing may accept labels as identity sugar only when a caller is plausibly starting from a discovered or rendered display value.
  - Identity sugar means:
    - accept a display label only for fields with a separately exposed canonical identifier and display label
    - resolve the display label immediately to the canonical identifier before the shared filter model is built
    - never persist, compile, or echo the ambiguous display value as if it were the canonical stored filter value
  - Identity sugar does not mean:
    - alternate request shapes
    - alias object structures
    - duplicate boolean/filter mechanisms
    - a second long-term public contract beside the canonical model
  - Saved/shared query state should not preserve ambiguous raw pack strings that might be either label or name.
- Ordering belongs in the shared field-domain/value-domain model rather than being scattered through local registries.
  - `freeText` may still declare ordering when discovery or presentation needs it.
  - The point is not that every `freeText` field must use ordering, only that the capability belongs in the same model.
- Filters do not need to be meaningful across every member of a scope union. When scope and metadata must stay coupled, use explicit composed branches.
- Null/presence filtering is a real canonical requirement.
  - Metadata predicates should support explicit null operators where meaningful.
  - Promoted first-class leaves should support explicit null/presence matching where the field meaningfully needs it.
  - For this pass, `scope` subcategory, `rarity`, and `actionCost` should support explicit null/presence handling.
  - `level` and `price` should move to numeric matcher leaves now because equality and bounded comparison are both meaningful.
  - For this pass, `level` and `price` should not gain null/presence matching.
- Arbitrary filters are allowed in `lookup`.
- Lookup results should expose match strength explicitly in the output model, such as:
  - `exact`
  - `normalized_exact`
  - `fuzzy`
- Lookup sorting may exist, but it should be lookup-specific rather than pretending to be the same as browse sorting.
  - Lookup sorting should explicitly support `policy: "tiered" | "global"` because the distinction affects server-side ordering, not just client rendering.
- Saved queries, snapshots, route payloads, and similar serialized search shapes touched by this refactor should be hard-migrated.
  - Do not add compatibility readers.
  - Rebuild indexes or other derived artifacts if the new model requires it.
- Introduce one shared field-domain registry that drives:
  - parse/index normalization for promoted fields
  - request/filter validation
  - filter-value discovery ordering
  - MCP semantics for expected value sets
  - TUI value ordering and presentation
- MCP should stay close to the canonical data model.
  - Do not add request-shape sugar for MCP beyond narrowly scoped identity resolution where a discovery surface returns display values alongside canonical identifiers.
  - If an MCP affordance is added, it should be justified as identifier resolution, not general ergonomics.

## 2. Architecture Context

Read and preserve:

- `docs/architecture/overview.md`
- `docs/architecture/boundaries.md`
- `docs/architecture/search.md`
- `docs/architecture/tui.md`

Relevant current owners and pressure points:

- `src/domain/search-request-types.ts`
  - currently exposes `intent` plus `parts`
- `src/server/search-request-adapter.ts`
  - still lowers a flat MCP root shape into `SearchRequestPart` variants
- `src/server/register-search-tools.ts`
  - still exposes flat root fields such as `category`, `subcategory`, `levelMin`, `rarity`, `metadata`, `linksTo`, `excludeLinksTo`, and `searchProfile`
- `src/search/request-compilation.ts`
  - still assumes the current `parts`/policy lowering pathway
- `src/tui/search/query-state.ts`
  - still carries a TUI-local query shape with browse-mode `queryText`
- `src/tui/search/filter-building.ts`
  - still drops browse-mode text during lowering
- `src/tui/search/service.ts`
  - still works through the current local query model rather than the target discriminated union directly
- `src/domain/metadata-field-types.ts`
  - still exposes `isUnique` as metadata
- `src/domain/search-types.ts`
  - still lists `isUnique` in `FILTER_VALUE_FIELDS`
- `src/search/filters/registry.ts`
  - still registers `isUnique` as a searchable metadata field
- `src/search/filters/semantics.ts`
  - still surfaces metadata vocabulary from that registry

Boundary expectations to preserve:

- keep one canonical shared search contract across MCP and TUI
- keep `SearchExecutionFilters` internal to search/data execution owners
- prefer direct replacement over compatibility support
- do not remove record-level `isUnique` storage/runtime usage if it still serves non-filter logic

## 3. Execution Model

Treat this as one coordinated refactor with direct replacement goals, not two loosely related fixes.

- Planning stays on `main` and remains uncommitted.
- Implementation happens in a dedicated `/tmp` worktree from current `main`.
- The implementation pass should include both code and docs updates in the same branch.
- Validation should cover contract types, schemas, TUI behavior, MCP surface behavior, lookup output, and runtime lowering.
- Milestone commits inside the implementation worktree are allowed even when the full end-to-end refactor is not yet build-green, as long as they are clearly partial checkpoints and are not presented as merge-ready completion.
- Partial-checkpoint commits must not be used as justification for compatibility shims or temporary fallback pathways whose only purpose is to keep an in-progress branch building.
- The implementation branch must not be merged back into `main` until the full planned workset is complete, the branch is cleanly rebased, the full validation suite passes, and the user explicitly confirms they want the completed branch landed.
- Until that explicit user confirmation, treat the worktree branch as in-progress even if it contains one or more local milestone commits.

Recommended sub-agent shape during implementation:

- one bounded slice for contract/schema/runtime changes
- one bounded slice for the shared app-facing discovery service boundary
- one bounded slice for the shared result-view presentation capability
- one bounded slice for TUI adoption and TUI tests
- one bounded slice for validation and serialized-shape migration checks

Primary slice dependencies:

- `Slice B` and `Slice D` should establish the canonical shared contract and filter-tree lowering before most of `Slice G` lands
- `Slice C` should establish shared field domains, operator ownership, and discovery contracts before `Slice D` finalizes canonical filter normalization around those owners
- `Slice C1` should establish the shared app-facing discovery service boundary before TUI and ontology discovery adoption work moves onto it
- `Slice E` and `Slice F` should land before the corresponding picker/dialog/editor flows in `Slice G2`
- `Slice H` should establish the shared result-view presentation capability before `Slice I` adopts it for lookup-specific behavior
- `Slice J` should land with or immediately after the field-vocabulary/filter-contract work in `Slice C` and `Slice D`, not as an unrelated late cleanup
- if implementation uncovers a concrete database/index schema requirement that is outside the current planned scope, stop and check with the user instead of silently expanding the refactor

Orchestration blocks for implementation:

- [ ] Block 1: Canonical contract and filter foundations
  - orchestrate `Slice A`, `Slice B`, `Slice C`, `Slice D`, and `Slice J`
  - goal: finish the shared request/filter model, field/operator ownership, canonical lowering, and `isUnique` removal as one coherent backend/domain pass
  - expected check-in point: canonical types, schemas, lowering, and searchable-field vocabulary are settled before any major surface adoption proceeds
- [ ] Block 2: Shared boundaries and reusable cross-surface primitives
  - orchestrate `Slice C1`, `Slice E`, `Slice F`, and `Slice H`
  - goal: land the shared discovery boundary, centered dialog primitive, generic picker filtering, and shared result-view presentation seam before feature-level adoption
  - expected check-in point: shared service and shared TUI/result primitives exist as stable owners that later slices consume rather than recreate
- [ ] Block 3: Main TUI adoption
  - orchestrate `Slice G1`, `Slice G2`, and `Slice G3`
  - goal: move the search workspace onto canonical `SearchRequest`, land focused editors and picker-driven creation flows, and complete tree/move/presentation adoption
  - expected check-in point: the TUI search workflow runs on the new model end-to-end without preserving the old semantic query shape
- [ ] Block 4: Lookup adoption, public-surface follow-through, and full finish validation
  - orchestrate `Slice I` and `Slice K`
  - goal: adopt the shared result-view seam for lookup, complete MCP/docs/ADR/backlog follow-through, and close validation against the full plan
  - expected check-in point: all remaining surface adaptations and documentation updates are complete and the branch is ready for final landing review

Orchestration rule:

- the top-level orchestrator should run exactly one block at a time
- after each block, stop, validate that block’s end state, and check back with the user before starting the next block
- do not queue or begin later blocks speculatively; this plan is intentionally structured so the user can restart with `/new` between orchestration blocks to keep agent context bounded

## 3.1 Change Envelope

Broad-strokes change rules for this refactor:

- allowed to change:
  - the shared search request/filter contracts
  - search compilation, validation, and canonical filter normalization
  - shared field-domain/operator/discovery contracts
  - the new app-facing discovery service boundary and search-related app-service wiring
  - the TUI search workspace, focused editors, picker/dialog reuse, and shared result-view presentation extensions
  - search-related docs, ADRs, backlog wording, and lint follow-through tied to the new stable boundaries
- preserve as stable unless directly required by the refactor and explicitly called out in this plan:
  - global shared TUI interaction/keybinding/help/footer contracts
  - shared list/detail behavior contracts outside the new result-presentation extension points introduced for search results
  - ontology navigation and render-ready route-preparation model
  - prepared index/schema, ranking/runtime behavior, and unrelated persistence/storage behavior
  - derived-tag/editorial subsystems and unrelated MCP tools

Specific stability rules:

- shared TUI interaction/keybinding layer:
  - extend shared interaction primitives only as needed for the planned search/editor flows
  - do not redesign global keybinding semantics, shared help/footer derivation, command-palette behavior, or low-level interaction routing in this pass
- ontology browser and result handoff:
  - preserve the ontology inspect/open/result-reader workflow contract and the render-ready navigation contract
  - allowed changes are limited to the internal query/discovery payloads, shared-service wiring, and other search-driven structural updates needed to consume the new canonical query model and shared discovery service
  - this refactor should not replace the ontology area’s top-level workflow shape or turn ontology navigation into a separate result-navigation architecture
- shared list/detail and result-view seams:
  - `Slice H` may extend shared result-view presentation with grouping axes and lightweight result metadata hooks
  - do not otherwise redesign pane-focus semantics, dead-end behavior, or route-readiness/list-detail contracts in this pass
- backend/runtime/storage envelope:
  - treat database/index/schema changes, ranking/runtime changes, and unrelated persistence changes as out of scope by default
  - they are a hard no unless the user explicitly approves a broadened scope after a concrete check-in

Required user check-in triggers:

- any database/index/schema change or new rebuild requirement beyond what is already scoped here
- any broad redesign of shared TUI interaction/keybinding/help/footer contracts
- any change to the ontology navigation/workflow contract rather than just its internal search/discovery wiring
- any new persistence or migration surface beyond the already scoped search-contract work

## 4. Implementation Slices

### Slice A: Contract Audit And Final Clause Inventory

Goals:

- audit every current place where `isUnique`, flat root filter fields, `intent`, `parts`, `queryText`, and local TUI query helpers are used
- audit current serialized search-adjacent shapes:
  - saved queries
  - search sessions
  - screen routes
  - snapshots
  - TUI draft/query-builder state
  - route payloads
  - other search-state artifacts
- identify which fields belong in the shared base versus a specific `mode` branch
- inventory all current embedded any/all/exclude mechanisms that must be normalized away

Expected artifact:

- an implementation note in the worktree or final task notes listing:
  - the final discriminated union shape
  - the final atomic clause inventory
  - all deleted old names and old field shapes
  - all serialized shapes that were rewritten directly

Validation:

- code search proving the caller/owner audit was done before broad edits

### Slice B: Canonical Contract Rename And Direct Replacement

Goals:

- replace `intent` with `mode`
- replace `parts` with `filter`
- remove old names everywhere rather than aliasing them
- update domain contracts, MCP schemas, request adapters, and runtime lowering to the new shared shape
- remove any compatibility upgrade helpers for the old model

Likely file set:

- `src/domain/search-request-types.ts`
- `src/domain/search-types.ts`
- `src/server/search-request-adapter.ts`
- `src/server/register-search-tools.ts`
- `src/server/tool-schemas.ts`
- `src/search/request-compilation.ts`
- any tests or fixtures still building `intent` / `parts`

Validation:

- no shared/public contract field named `intent`
- no shared/public contract field named `parts`
- no shared/public contract field named `filters`
- no alias fields, alias types, alias schemas, or upgrade helpers preserving the old names

### Slice C: Shared Field Domains And Value Normalization

Goals:

- introduce a shared field-domain/value-domain registry for promoted fields and any metadata fields that need declared normalization or ordering
- move existing rarity ordering into that shared registry rather than leaving it as a local registry hint
- introduce one shared owner for canonical operator types, matcher/value vocabulary, and declared field-value semantics
- keep raw surface sugar parsing out of `src/domain/**`
  - server/TUI adapter layers may accept friendly spellings such as `>=`, `<=`, `1-5`, or display-label identity sugar where that edge input is justified
  - those adapters must lower raw input once into canonical operator tokens and canonical value shapes before crossing the shared contract boundary
- define the first explicit promoted domains:
  - `rarity` as a closed declared domain
  - `actionCost` as a bounded declared numeric domain for the currently modeled numeric action-count values
- separate declared expected values from live present corpus values in discovery flows
- use one shared owner for normalization, validation, and ordering instead of duplicating these concerns across parse code, filter semantics, and TUI helpers
- define one shared discovery-mode contract for picker-driven value selection and ontology-style browsing
  - `Matching` = values/counts from the currently matching result set
  - `Catalog` = all known valid values within the current applicability slice, even when the current matching set is zero
- make that discovery model a shared domain/app contract rather than a TUI-local or ontology-local convenience layer
  - `src/domain/` owns the discovery vocabulary and contracts:
    - discovery mode
    - applicability context
    - discovery target/field identity
    - returned option/count/ordering model
  - app/data/search owners will later compute discovery results from canonical query meaning and scoped applicability through the shared app-facing service introduced in `Slice C1`
  - this slice defines the shared semantics and contracts that `Slice C1` must expose, not the final cross-surface service wiring itself
- keep the applicability slice for `Catalog` narrow and explicit
  - include current mode
  - include current scope union
  - include current pack constraints
  - exclude search text and narrower value predicates
- make these discovery-mode semantics reusable across later TUI and ontology consumer slices rather than redefining them per surface

Implementation shape:

- add a new domain owner for field-domain specs, likely under `src/domain/`
- add one shared discovery contract that answers:
  - given canonical query context
  - plus a discovery target/field family
  - plus discovery mode (`Matching` or `Catalog`)
  - return ordered options with counts, stable identifiers, ordering data, and other presentation-neutral shared semantics
- define reusable types for:
  - value domain kind
  - normalization strategy
  - ordering strategy
  - unknown-value policy
- define reusable operator domains for:
  - equality
  - ordering
  - range
  - collection membership
  - null/presence
- keep the ownership split explicit:
  - `src/domain/**` owns canonical operator domains, canonical matcher/value shapes, and declared field/value semantics
  - adapter-layer normalization owns raw input parsing and alias resolution for editor/transport sugar before canonical request construction
  - adapter layer here means MCP request adapters, TUI editor-confirmation/adaptation seams, or a narrowly owned edge-normalization helper consumed by those adapters
  - `src/domain/**` must not become the owner of raw user-facing shorthand spellings or transport-specific parser syntax
- attach declared domains to promoted fields such as `rarity` and `actionCost`
- route parse/index-time normalization for those fields through that registry rather than ad hoc per-field parsing
- route request-time validation for those fields through that registry rather than accepting arbitrary strings/numbers
- route `listFilterValues`, semantics, and TUI ordering through that same registry
- route operator alias normalization through the adapter layer using the shared domain vocabulary rather than per-family ad hoc parsing
- preserve one-lowering semantics for sugar:
  - friendly input may be parsed only at the edge
  - canonical requests, compiled search inputs, and persisted/shared semantic shapes should carry canonical operator/value tokens only
- make pack identity explicit in the same shared contract boundary:
  - canonical stored filter values use pack names
  - user-facing discovery and rendering use pack labels
  - edge normalization resolves labels to canonical names before persistence or compilation
- keep MCP edge sugar narrow and explicit:
  - acceptable: resolving a discovered display label to its canonical identifier before the canonical request is built
  - not acceptable: alternate MCP request shapes, duplicate field vocabularies, or parallel “friendly” filter structures
- keep discovery contracts live but mode-aware:
  - `Matching` should evaluate against the current canonical query meaning
  - `Catalog` should evaluate against the applicability slice only, including unions of currently allowed scopes
  - exact-valued/scoped fields should remain picker-backed rather than accepting arbitrary free-text values
  - picker filtering text should narrow visible options but should not itself become a committed raw field value
- define the shared discovery/context model that later service and UI slices must consume

Likely file set:

- `src/domain/` value-domain definitions and registry
- `src/data/record-normalization.ts`
- `src/data/index-types.ts`
- `src/search/filters/registry.ts`
- `src/search/filters/semantics.ts`
- `src/search/filters/metadata.ts`
- `src/search/request-compilation.ts`
- tests covering shared domain contracts for field domains, operators, and discovery-mode semantics

Validation:

- rarity normalization and validation are explicit and shared
- the canonical expected set for rarity is declared in one place
- action-cost normalization and validation are explicit and shared
- operator ownership is centralized rather than duplicated across metadata and metric families
- pack filters have a canonical identity/presentation split in the shared contract
- the shared `Matching` / `Catalog` discovery semantics are defined once in reusable domain/app contracts
- `Catalog` remains constrained only by mode, scope union, and pack context rather than collapsing back into full query matching
- later service and UI slices consume one shared discovery-mode contract instead of redefining discovery semantics per surface
- ordering no longer relies on one-off local hints for promoted fields

### Slice C1: Shared App-Facing Discovery Service Boundary

Goals:

- introduce one app-facing discovery service boundary that both TUI and ontology/search-semantics consumers use
- keep discovery semantics and applicability rules out of TUI-local helpers and ontology-local loaders
- consolidate currently fragmented discovery responsibilities behind one shared service contract
- make the service answer discovery questions from canonical query meaning rather than from surface-specific draft shapes
- adopt the shared field-domain/operator/discovery contracts established in `Slice C` rather than redefining them locally
- keep the ownership boundary explicit:
  - the shared seam lives at the app/search service layer
  - it does not become a domain-level UI contract
  - it does not become a TUI-first helper that ontology later has to mirror or adapt awkwardly

Responsibilities that should move behind this service:

- scoped field discovery and applicability rules
- live value discovery with counts under shared `Matching` / `Catalog` semantics
- metric-key discovery grouped and ordered for shared consumers
- pack identity resolution between canonical names and display labels
- shared ordering and caching of discovered options where caching is appropriate at the app/runtime layer
- async discovery execution for any surface that needs live `Matching` refresh from current canonical query meaning

Consumer model for this boundary:

- keep one shared async discovery service boundary for both ontology and search consumers
- do not require one surface to adopt the other surface's interaction contract
- ontology/search-semantics browse should use async model/session construction at route or scope-load boundaries, then hold a concrete in-memory model for the active scope
- search-editor picker/explorer flows should consume the same discovery boundary in a query-aware way and remain free to refresh asynchronously as the canonical `SearchRequest` changes
- do not force the ontology tree contract itself to become a continuously reloading live-query UI just to support search-editor `Matching` mode
- do not fork discovery semantics per surface; the difference is consumer behavior, not separate meaning for `Matching` / `Catalog`

Count-path rule for this boundary:

- ontology-browser live counts, picker/explorer value counts, and other discovery/value-count surfaces should come from the shared discovery service
- whole-query total-match counts for the intermediate search state and similar search-workspace summaries should use the existing count-only search path (`countRecords(...)` / `SearchCountResult`)
- do not call `listRecords(...)` just to obtain a total unless implementation uncovers a concrete mode-specific requirement and the user agrees to that scope expansion

Responsibilities that should remain outside this service:

- picker shell/container choice
- editor-view mapping
- local dialog draft/builder state
- TUI-only selection/partition helpers
- ontology-only navigation/presentation structure
- surface copy and presentation details
  - picker rows and row text
  - footer/help/action-menu copy
  - badges, subtitles, and section headings
  - terminal or ontology-browser interaction affordances

Implementation shape:

- create a new app-facing service under `src/app/` rather than stretching an existing owner
- use a concrete app-facing owner path rather than leaving the service nameless during implementation
  - provisional target: `src/app/search-discovery-service.ts`
- `src/domain/` owns the discovery contracts and vocabulary
- the new app service owns cross-surface discovery orchestration and exposes a stable API to TUI and ontology consumers
- the app service delegates to search/data owners for actual value/count computation
- low-level helpers such as `listFilterValues(...)` remain backend primitives, not the cross-surface discovery abstraction
- callers should not treat `Pf2eDataService` passthrough discovery helpers as the durable boundary once the new app service exists
- TUI consumers should receive this boundary through `src/tui/app-services.ts` rather than importing the app service ad hoc from feature code
- the app service becomes the durable home for:
  - field discovery by canonical query context
  - value discovery by canonical query context plus discovery target
  - metric-key discovery by scope/category context
  - pack label-to-name resolution before canonical request construction
  - shared option ordering and any justified runtime caching
- prefer async model/session construction at the consumer boundary over turning the domain model itself into an always-async tree API
  - acceptable: ontology or search hosts await a prepared model/session, then render a concrete in-memory structure for the active scope
  - acceptable: search-editor matching-mode consumers debounce, cancel, and ignore stale async refreshes while the user edits the canonical request
  - not acceptable: pushing picker/session/footer concerns into the service just because the service is async
  - not acceptable: requiring every ontology node expansion to become an arbitrary async fetch unless a later slice proves that shape is necessary
- keep the service transport-neutral and presentation-light
  - the service returns structured discovery results, ordering data, counts, stable identifiers, and other shared semantics
  - rows, panes, badges, picker mode, footer copy, help text, and dialog flow stay in TUI/ontology consumers
  - do not encode picker labels, section headers, footer strings, or action-menu wording into the service or into `src/domain/**`
- do not allow this boundary to collapse into a TUI-shaped contract
  - ontology and TUI should both consume the same service semantics
  - neither surface should become the de facto owner of discovery semantics that the other later has to translate back out of a UI-shaped model
- once the new service boundary is stable and both surfaces have adopted it, add or tighten lint rules so cross-surface discovery callers do not reopen direct low-level discovery wiring
  - prefer enforcing the stable owner after adoption rather than pre-emptively blocking work before the boundary is proven
  - the intended enforcement target is direct surface-level dependence on low-level discovery primitives when the shared app-facing service should own that path

Likely current fragmentation to collapse:

- `src/tui/search/service.ts`
  - current facet/query-field option assembly and direct `listFilterValues(...)` orchestration
- `src/tui/search/discoverable-fields.ts`
  - current scoped-field discovery and discoverable-field selection logic that is really about shared applicability
- `src/app/ontology/search-semantics-domain.ts`
  - current ontology-local cached filter-value loading and field/value discovery assembly
- adjacent ontology search-semantics helpers that currently decide live discovery loading shape

Likely file set:

- new app-facing discovery service owner(s) under `src/app/`
- `src/domain/` discovery contracts/vocabulary
- `src/tui/app-services.ts`
- `src/tui/search/service.ts`
- `src/tui/search/discoverable-fields.ts`
- `src/app/ontology/search-semantics-domain.ts`
- related ontology helper owners that currently perform direct value discovery

Validation:

- TUI and ontology/search-semantics consumers both use the same app-facing discovery service boundary
- scoped field discovery/applicability rules are no longer split across TUI-local and ontology-local helpers
- `Matching` / `Catalog` value discovery semantics are computed in one shared place
- metric-key discovery is exposed through the same service boundary rather than separate surface-local loaders
- pack label/name resolution happens before canonical request construction and is not reimplemented per surface
- ontology browse loads a prepared model/session asynchronously at route or scope boundaries, then navigates a concrete in-memory model for the active scope
- search-editor `Matching` consumers can refresh asynchronously from canonical query changes without redefining discovery semantics in TUI-local code
- presentation-only concerns remain outside the service
- the service contract is not a TUI picker model or ontology-browser presentation model in disguise
- no picker labels, footer/help copy, action-menu wording, or other surface presentation strings are pushed into `src/domain/**` or the app-facing discovery service
- no new surface-specific discovery semantics are introduced in TUI or ontology code during the refactor
- no TUI feature-level code bypasses `src/tui/app-services.ts` to wire discovery behavior directly from feature modules
- no ontology-local duplicate discovery ordering/cache logic remains where the shared app-facing service should own that behavior
- lint follow-through is completed once the boundary is stable enough to be mandatory
  - add or tighten lint rules so surfaces do not reopen direct low-level discovery wiring
  - validate that the enforced path matches the landed app-facing service boundary rather than a temporary implementation detail

### Slice D: Canonical Filter Normalization

Goals:

- replace the flat MCP filter surface with canonical `filter`
- collapse `category` / `subcategory` / `scopes` into canonical scope clauses
- remove embedded boolean policy wrappers in the canonical model
- add general `anyOf`, `allOf`, and `not`
- normalize links to atomic positive clauses plus boolean composition
- normalize metadata predicates toward atomic leaves plus generic boolean composition
- keep ordinary metadata predicates under one generic metadata leaf while moving keyed metrics to dedicated `metric` and `metricCompare` leaves
- convert first-class numeric filters from range-only leaves into numeric matcher leaves where equality and bounded comparison are both meaningful
- add explicit null/presence handling where required by the canonical model
- do not decide final MCP/TUI sugar affordances beyond what is already required for deterministic canonical lowering

Concrete target requirements:

- no canonical `excludeLinksTo`
- no canonical `SearchFilterPolicy`
- no canonical embedded `exclude`
- no canonical `includesAny` / `includesAll`
- no canonical `any` / `all` boolean wrappers in clause payloads
- no actor-vs-item split in the canonical public metric leaf kinds
- no canonical plural metadata payloads such as `values`; use atomic leaves plus tree composition instead

Likely file set:

- `src/domain/search-request-types.ts`
- `src/server/tool-schemas.ts`
- `src/server/search-request-adapter.ts`
- `src/search/request-compilation.ts`
- metadata filter/domain predicate owners touched by the atomic-predicate cutover
- tests for request adaptation, compilation, and schemas

Validation:

- general boolean composition works across heterogeneous clause kinds
- disjoint numeric queries are representable via `anyOf`
- scope-plus-metadata coupled branches are representable via `allOf` within `anyOf`
- full-category scope selection and null-subcategory scope selection are both representable without ambiguity
- numeric first-class leaves support both equality and bounded comparisons
- ordinary metadata predicates and keyed metric predicates have explicit, non-overlapping leaf shapes
- no canonical flat MCP filter shape remains

### Slice E: Floating Dialog Primitive For TUI Search Workflows

Goals:

- introduce a reusable centered floating dialog primitive for the TUI
- use that primitive immediately for initial mode selection
- make the primitive strong enough for near-term reuse by focused search editors such as numeric matcher entry and other picker-style interactions
- keep the first implementation intentionally narrow rather than trying to solve every future overlay need

Implementation shape:

- extend the existing shared TUI modal/prompt framework with a centered floating dialog capability rather than introducing a second modal system beside it
- if the current framework needs new surface area, add that surface area to the shared owners under `src/tui/framework/**`
  - acceptable shapes include a new `TerminalModalState` kind, a new shared modal presentation/layout mode, or a new prompt-adapter entrypoint routed through the existing provider/host/planning path
  - unacceptable shape: search-screen-owned modal state, layout, rendering, or input-routing that bypasses the shared provider/modal host/prompt adapters
- keep feature code consuming the existing prompt/modal seam rather than owning overlay lifecycle directly
- define the minimum supported capabilities for the first version:
  - title and optional description
  - selectable list or action rows
  - confirm/cancel and dismiss behavior
  - explicit focus handling and keyboard navigation
  - bounded sizing and centered layout distinct from the current bottom-aligned modal presentation
- for the initial mode picker built on this primitive:
  - render the three search modes as a horizontally browsed choice set inside the centered modal rather than as a plain vertical menu
  - use left/right as the primary visible navigation for mode selection
  - mirror up/down to the same previous/next selection behavior silently as ergonomic aliases
  - keep the shared description in one stable area beneath the choice set, updating as the active mode changes
  - do not advertise the up/down aliases in the help copy for this first pass
- wire the search entry flow to use this primitive for mode selection immediately after the abstraction lands
- preserve a clear ownership boundary so later search editors can reuse the primitive without copying mode-picker-specific logic into new overlays

Representative mode-picker example:

```text
Choose Search Mode

[ Browse ]   Search   Lookup

Explore records without text search.
Use filters and sorting over the scoped corpus.

← / → change mode
Enter confirm
Esc cancel
```

Delegation expectation:

- this slice is large enough to justify its own sub-agent ownership during implementation
- the delegated slice should own both the shared framework extension and the first search-mode-picker integration, with explicit validation

Likely file set:

- shared TUI dialog/modal owners under `src/tui/framework/**` plus the existing prompt-adapter seam
- `src/tui/search-screen/**`
- related focus/input helpers
- TUI tests covering centered dialog behavior and mode-picker integration

Validation:

- the centered floating dialog exists as a reusable extension of the shared modal framework rather than a search-only one-off or parallel modal stack
- initial search entry uses the primitive for mode selection
- cancel and dismiss behavior leave the underlying screen state unchanged
- selecting an option confirms through the shared prompt/modal path and hands control back to the search workspace as expected

### Slice F: Generic Picker Filtering Across TUI Menus

Goals:

- make `/`-driven filtering work consistently across picker-style menus in the TUI, not only command palettes
- establish one shared picker-filter interaction model that search-editor pickers can depend on
- treat this as a reusable dependency for clause-kind selection and other picker-driven editors rather than a one-off search feature

Implementation shape:

- extend the existing shared picker/select prompt family so it supports filter text entry using the same footer/status pattern already proven in the ontology browser rather than introducing a body-level search row
- keep picker filtering inside the shared modal/prompt owners under `src/tui/framework/**` plus the existing prompt-adapter seam
  - acceptable shape: shared filtering state, rendering, and input routing added to the current select/multiselect/policy prompt path
  - unacceptable shape: a search-screen-owned picker implementation, editor-local picker host, or feature-local filtering loop that bypasses the shared prompt family
- make clause-kind selection, focused search editors, and later picker-style consumers depend on that shared capability rather than forking their own picker behavior
- keep the interaction model consistent across picker-style menus:
  - `/` enters filtering mode
  - typed text narrows the visible entries
  - backspace edits the filter text
  - clearing the filter restores the full picker list
  - dismissing the picker abandons the filter and the selection together
- when filter mode is active:
  - keep the picker body layout stable rather than inserting a transient search field into the list body
- render the live filter state through the shared footer/status seam in the ontology-browser style, e.g. `Search /<text>`
- preserve one stable secondary help/description area in the picker body for the currently focused option
- treat the current ontology-browser footer-based live filtering as the model to generalize into the shared implementation
- preserve one shared filtering implementation rather than re-implementing search/filter logic per picker screen
- make clause-kind selection depend on this shared picker filtering instead of special-case menu hierarchy
- do not add a separate search-editor picker abstraction as an intermediate step
- do not land feature code that duplicates shared picker filtering behavior with the intent to “generalize it later”

Representative numeric-matcher editor examples:

Valid `>=` matcher:

```text
Level Filter

Enter a matcher for level.
Examples: 1, >=5, <=10, 3-7

Value: >=5
Preview: level >= 5

Enter confirm
Esc cancel
```

Incomplete matcher, no preview yet:

```text
Level Filter

Enter a matcher for level.
Examples: 1, >=5, <=10, 3-7

Value: >=
Example formats: 1, >=5, <=10, 3-7

Enter confirm disabled
Esc cancel
```

Valid range matcher:

```text
Level Filter

Enter a matcher for level.
Examples: 1, >=5, <=10, 3-7

Value: 1-5
Preview: level between 1 and 5

Enter confirm
Esc cancel
```

Delegation expectation:

- this slice is large enough to justify its own bounded implementation/validation ownership during execution
- it should land as a reusable TUI capability before or alongside the search-editor picker work that depends on it

Likely file set:

- `src/tui/framework/modal-host.tsx`
- `src/tui/framework/modal-helpers.ts`
- `src/tui/framework/modal-prompt-bodies.tsx`
- `src/tui/framework/provider.tsx`
- `src/tui/framework/types.ts`
- `src/tui/interaction-context-adapters.ts`
- related picker/modal framework owners under `src/tui/framework/**`
- TUI tests covering picker filtering behavior

Validation:

- `/` filtering works in picker-style menus, not only command palettes
- picker filtering behavior is shared and consistent across the relevant picker families
- picker filtering uses the shared footer/status presentation pattern rather than ad hoc body-level search rows
- clause-kind selection can rely on generic picker filtering without requiring special promoted-filter buckets
- no search-screen-owned picker filtering path remains or is introduced for the same interaction model
- focused search editors consume the shared picker-filtering capability through the existing prompt/modal seam

### Slice G: TUI Literal Adoption Of The Shared Union

Execution sub-slices within `Slice G`:

- `Slice G1`: canonical TUI state, workspace shell, mode flow, and tree/query-summary projection
- `Slice G2`: focused editors, picker-driven node creation, metric-key discovery flow, and picker/dialog integration
- `Slice G3`: tree restructuring, move semantics, and presentation-alias rendering

These sub-slices are intentionally ordered. `G1` establishes the canonical workspace shape, `G2` adds focused editing flows on top of that shape, and `G3` adds structural manipulation and presentation-layer refinement on top of the landed editor/tree model.

Goals:

- make the TUI use the same discriminated `mode` union directly
- enter the search flow through a centered floating mode picker rather than assuming an in-workspace default mode
- use the canonical shared search request as the workspace source of truth rather than maintaining a long-lived parallel UI-only query model
- remove browse-mode text state and the current silent-drop behavior
- ensure browse renders no text-search input
- ensure search renders `query`, optional `exclude`, optional `profile`
- ensure lookup renders `query` with lookup-specific behavior
- remove local TUI query-shape fields that duplicate or drift from the shared model
- keep mode selection available inside the workspace after entry
  - reopening mode selection should reuse the same floating picker or an intentionally equivalent condensed dialog
  - confirming a different mode should reset the canonical query state for that workspace and refresh the list/results view around the newly selected mode
  - dismissing the picker or reselecting the current mode should leave query state and list/results behavior unchanged

Representative workspace mode-switch reopen examples:

```text
Change Search Mode

 Browse    [ Search ]   Lookup

Ranked text search with optional filters.

Changing mode resets the current workspace query.
Reselecting the current mode leaves everything unchanged.

← / → change mode
Enter confirm
Esc cancel
```

```text
Change Search Mode

 Browse    [ Search ]   Lookup

Ranked text search with optional filters.

Current mode selected.
Workspace will remain unchanged.

← / → change mode
Enter confirm
Esc cancel
```

```text
Change Search Mode

[ Browse ]   Search   Lookup

Explore records without text search.
Use filters and sorting over the scoped corpus.

This will reset the current workspace query.

← / → change mode
Enter confirm
Esc cancel
```

- use the floating mode picker as the first centered dialog primitive that future focused editors can build on, without overcommitting the initial implementation to every future modal use case
- make the filter node, not the whole query or a staged policy object, the primary editing/navigation unit inside the workspace
- keep TUI restart behavior as a fresh boot rather than introducing restart restoration of search/editor state in this pass
- use a hybrid editing model:
  - keep the canonical filter structure visible and navigable
  - use focused add/edit flows that create or update canonical nodes
  - do not force users to hand-author boolean structure for every common operation
  - on the shared search page, render only the root group as a flat immediate-child projection for the common case
  - when nested boolean structure is created or opened, transition into a dedicated filter-builder view that shows the full tree with full context
- in the TUI, always render and edit the filter tree under a group root
  - default the initial visible root group to `allOf`
  - allow the user to change the root group type to `anyOf`
  - this TUI-root-group convention is an editor-shape decision, not a reason to change the canonical shared data model to require grouped roots everywhere
  - a single-leaf or negated-leaf query should still be reachable without extra ceremony by placing that leaf under the default root group
- on the shared search page, nested groups should not be expanded inline
  - show multi-child groups as compact placeholder rows such as `allOf(2 filters)` or `anyOf(3 filters)`
  - inline `not` directly on the shared search page rather than treating it as a multi-child placeholder
  - those rows should communicate that additional nested structure exists without inlining the subtree contents
  - entering nested structure should move the user into the dedicated full-tree filter builder rather than creating a drill-in view that loses overall query context
- insertion/editing flow should be:
  - choose insertion point in the tree
  - choose clause kind
  - open a focused editor for that clause
  - treat insertion slots as first-class cursor targets in the tree renderer
  - show insertion slots inside the current group structure rather than inventing multiple roots
  - allow “flat add within current group” behavior so repeated additions to the same group do not require re-navigating the whole tree between each selected value
  - multi-select pickers may emit multiple sibling leaves directly into the selected group level when that is the natural user intent
- use only short-lived dialog-local draft state for incomplete edits
  - incomplete or invalid edits should not live in canonical workspace state
  - dismissing an editor should discard that local draft rather than leaving half-built nodes in the tree
  - drafts should be builder-style editor state, not `Partial<SearchFilterNode>`
  - avoid a mirrored full draft AST for the whole query tree
  - prefer a small set of shared draft matcher/building types reused by focused editors
  - each focused editor should finalize its local draft into a valid canonical node on confirm
- keep one shared workspace shell across modes, with mode-specific sections rather than three unrelated search UIs
- make the long-lived TUI ownership split explicit:
  - canonical workspace state is the shared `SearchRequest`
  - focused editor builders are short-lived dialog-local draft state only
  - the filter-tree model with ephemeral node IDs and insertion slots is a derived presentation projection of canonical `SearchRequest`, not a second semantic query model
  - the search-screen summary/document model is likewise a derived presentation projection, not a second semantic query model
  - no TUI-owned canonical query shape should survive alongside `SearchRequest` once this refactor lands
  - the TUI must still expose one explicit search-facing facade through `services.user.search`
  - `search-screen/**` controllers, reducers, and workflows should consume that facade rather than absorbing service-owned workflow logic directly
  - the current facade owner is `src/tui/search/service.ts`; the implementation behind that facade may be narrowed or split into richer TUI search owners during this refactor
  - acceptable split examples include:
    - query-adaptation or workspace-normalization helpers
    - ontology-origin query handoff adapters
    - result-window or session-lifecycle owners
    - mode/sort execution-policy owners
  - unacceptable end state: feature-local `search-screen/**` code becomes the de facto owner of query/session orchestration, result-window lifecycle, count-path behavior, ontology-origin handoff, or mode/sort execution policy
  - if the refactor intentionally replaces `src/tui/search/service.ts` as the durable facade owner rather than narrowing/splitting its internals, the implementation must say so explicitly and update `docs/architecture/tui.md` in the same pass
- hard migration rule for this slice:
  - `SearchRequest` is the only durable semantic query source of truth in the TUI after migration
  - any successor to `src/tui/search/query-state.ts` must either disappear or narrow to canonical normalization plus derived/editor helpers
  - neither the filter-tree projection nor the summary/document layer may become a serialized or long-lived semantic source parallel to `SearchRequest`
- keep route/session handoff coarse and canonical while the TUI is running
  - route transitions may carry canonical query state, prepared result sessions, and other intentional runtime context
  - route transitions should not carry editor-local clutter such as open modal state, picker filter text, move mode, or draft builders
- make the query summary a navigable structured outline of the canonical query tree rather than a passive recap
  - in the dedicated filter-builder view, the full tree is the primary summary
  - on the shared search page, the summary should be the root-level flat projection plus compact nested-group placeholders rather than an expanded full tree
  - the shared search page should also expose a selectable `Filters >` entry path that opens the dedicated filter-builder directly even when nested structure is not yet present
- render canonical nodes in a friendlier presentation vocabulary than the raw object shape
  - for example, show `trait includes fire` or a similarly compact humanized form rather than raw typed payload syntax
  - presentation-friendly rendering must not create a second canonical state shape
- expose `metadataPredicate`, `metric`, and `metricCompare` as top-level add/edit choices rather than hiding them behind a single advanced umbrella
- use friendlier user-facing labels for those top-level choices
  - `Field filter`
  - `Metric filter`
  - `Metric comparison`
- build the first version around the verbose/core editing workflow rather than shortcuts or automatic convenience flows
- clause-kind selection does not need special “promoted filter” buckets in the first pass
  - show the full node-kind list in one picker
  - order entries by expected/common usage where helpful
  - rely on generic picker filtering rather than demoting less common node kinds behind extra hierarchy
  - treat this full filtered picker as the intended replacement for special promoted-filter buckets rather than leaving bucketed clause menus as an implementation choice

Representative clause-kind picker:

```text
Add Filter

> Field filter
  Metric filter
  Metric comparison
  Scope
  Pack

Filter on a metadata field such as traits or rarity.

Esc cancel
Enter choose
```

Representative clause-kind picker while filtering:

```text
Add Filter

> Metric filter
  Metric comparison
  Scope

Filter on one discovered metric key.

Filter: met
Enter choose
Backspace edit
Esc clear and cancel
```

- start tree restructuring with keyboard/context actions on the selected node rather than drag-style editing or a separate structure-editing mode
  - prefer the existing `: focus actions` footer/action model for common node actions rather than hidden command discovery alone
  - the initial action set should cover:
    - wrap selected node in `not`
    - create a new `anyOf` group
    - create a new `allOf` group
    - change the selected group type, including the root group
    - move a node by selecting from valid insertion slots
    - unwrap a group
    - lift a child out of a group
  - do not add sibling-wrapping shortcuts such as “wrap selected siblings in `anyOf` / `allOf`” in the first pass
    - users should create groups explicitly and move nodes into them
    - treat “create group -> move nodes into it -> adjust group type if needed” as the intended first-pass grouping workflow
    - preserve the simpler action surface first, then revisit shortcut grouping only after real usage
  - `not` should be able to wrap any selected node, including groups
  - when the selected node is already a `not` group, the corresponding action should explicitly unwrap that `not` layer rather than nesting another `not`
  - while in move mode, `j` / `k` should move between valid insertion slots, not arbitrary rows
  - treat this as the starting interaction model and expect to refine it after real usage rather than overdesigning the first pass

Likely file set:

- `src/tui/search/query-state.ts`
- `src/tui/search/filter-building.ts`
- `src/tui/search/service.ts`
- `src/tui/search/service-types.ts`
- `src/tui/search-screen/**`
- TUI tests covering mode-specific UI and request behavior

Implementation shape:

- keep canonical workspace state typed directly as the shared `SearchRequest`
- model focused editor state separately from canonical query state
- project canonical filter state into a TUI tree model that carries:
  - ephemeral node IDs for selection/focus
  - insertion slots as explicit cursor targets
  - a visible root-group framing even when the canonical filter is structurally simple
  - keep that identity strictly UI-local; canonical query state itself must remain free of persistent node IDs
- render the filter tree as a terminal tree with selectable nodes and selectable insertion slots, for example:

Shared search page, flat root example:

```text
Search
Mode: Search
Query: fire resistance
Exclude: cold
Profile: balanced
Sort: relevance

Filters >
allOf
├─ trait includes fire
├─ rarity is uncommon
├─ level >= 3
└─ [+ add filter]
```

Shared search page, nested-placeholder example:

```text
Search
Mode: Search
Query: blast
Profile: balanced

Filters >
allOf
├─ scope: spell / any
├─ anyOf(2 filters)
├─ not action cost = 2
└─ [+ add filter]

Hint: Nested filter groups present. Open Filters for full structure.
```

Dedicated filter-builder example:

```text
Filters                                    Details
Query Context                              Selected: anyOf
Mode: Search                               Type: Any Of
Query: blast                               Children: 2
Profile: balanced                          Actions:
Sort: relevance                            - add filter
                                           - change group type
Filter Tree                                - move node
allOf                                      - wrap in not
├─ scope: spell / any                      - unwrap group
├─ anyOf
│  ├─ trait includes fire
│  ├─ trait includes cold
│  └─ [+ add here]
├─ not
│  └─ action cost = 2
└─ [+ add here]
```

- when a node is selected, the tree should make both the selected node and the nearby insertion structure obvious, for example:

```text
Filter
└─ allOf
   ├─ scope: spell / any
   ├─ anyOf
   │  ├─ > trait includes fire
   │  ├─   trait includes cold
   │  └─   [+ add here]
   └─ [+ add here]
```

- when move mode is active, the selected node should remain visually anchored while insertion slots become move targets, for example:

```text
Filters                                    Details
Query Context                              Move: trait includes fire
Mode: Search                               Select destination with j/k
Query: blast                               Enter confirm, Esc cancel

Filter Tree
allOf
├─ scope: spell / any
├─ anyOf
│  ├─ trait includes fire
│  ├─ trait includes cold
│  └─ [move here]
├─ not
│  └─ action cost = 2
└─ [move here]
```

- define move mode in terms of valid insertion slots, not arbitrary row motion
  - entering move mode captures the selected canonical node as the moving payload
  - the UI computes the ordered list of valid insertion slots for that payload
  - `j` / `k` move between those valid insertion slots only
  - confirm inserts the payload at the selected slot
  - cancel returns to normal selection without changing the tree
  - invalid/self-recursive targets must not appear as move slots
  - both add slots and move slots should appear only at the bottom of groups rather than between siblings
- define insertion-slot ordering explicitly so cross-level motion is predictable
  - use one stable traversal order through the visible tree
  - group-local slots and ancestor/descendant boundary slots should appear in that traversal order
  - crossing tree levels during move mode is therefore just moving to the next or previous valid slot in the ordered slot list, not a separate navigation rule
- define a small reusable set of editor-local draft/building types, for example:
  - draft scalar match builders
  - draft numeric match builders
  - draft scope/subcategory selection builders
- use those shared builder pieces inside focused editors such as:
  - scope editor
  - level editor
  - price editor
  - rarity editor
  - action-cost editor
  - metadata predicate editor
  - metric editor
  - metric-compare editor
- require each focused editor to:
  - own its incomplete local draft state
  - validate/finalize that draft on confirm
  - emit a valid canonical `SearchFilterNode`
  - discard the draft on cancel
- do not make incomplete draft types assignable to canonical workspace tree nodes
- keep picker/edit experiences aligned by interaction family:
  - categorical/selectable values such as rarity, traits, and families should reuse the same picker family even when list size differs
  - numeric matchers should use the centered floating dialog family
- standardize interaction models more strongly than shell choice
  - floating dialogs are one reusable shell, not the mandatory shell for every editor
  - taxonomy/catalog explorers are another valid reusable shell for editors that need browsing, grouping, and descriptions
  - choose shell/container by interaction complexity and contextual-browsing needs rather than raw option count
  - keep the editor-view mapping stable even when the shell/container is later retuned
- `/` filtering should work generically across currently open picker-style menus in the TUI, not only command palettes
  - treat this as a dependency for the search-editor rework if the current picker family does not already support it
  - clause-kind selection should rely on this generic filtering behavior rather than special-case bucket promotion
- picker-driven discovery should use the shared `Matching` / `Catalog` modes
  - default to `Matching` automatically when a picker or explorer opens
  - allow the user to change discovery mode through the `:` action menu while that picker or explorer is active
  - in `Matching`, show values and counts from the currently matching query context
  - in `Catalog`, show values and counts from the broader applicability slice only
  - apply the same mode concept to ontology/explorer-style browsing as well as compact pickers
  - search-side `Matching` consumers must be query-aware: the active canonical `SearchRequest` drives refresh scope and counts
  - search-side `Matching` refresh may run asynchronously and should support debounce/cancellation/stale-result protection so live editing does not stall the UI
  - ontology browse may use the same shared discovery boundary with async route/scope preparation, but should still stabilize into a concrete in-memory model for the active scope rather than behaving like a continuously polling live-query tree
- categorical pickers should support repeated or multi-select insertion into the currently selected group when the clause family naturally supports flat sibling creation
  - the implementor should treat this as a first-class interaction for picker families such as traits/families rather than an optional polish step
  - in the first pass, multi-select insertion should not ask the user to choose a boolean connective or grouping shape
  - selected values should inline as peer leaves in the currently selected group, inheriting that group’s existing boolean semantics
  - when repeated or multi-select insertion emits sibling leaves of the same family into a group, keep those siblings adjacent rather than interleaving them with unrelated leaf families in that same group
- numeric matcher dialogs should accept compact textual matcher input rather than forcing operator-first selection
  - examples: `1`, `>=5`, `<=10`, `1-5`
  - the dialog should parse recognized shorthand into canonical matcher types and report invalid input clearly
  - `scope` should default to subcategory mode `any` unless the user explicitly changes it
- make the editor mapping explicit by node family:
  - `pack` -> searchable picker showing pack labels in the UI and storing canonical pack names
    - rows should render labels only rather than exposing canonical pack names in the picker
    - picker filtering/search should be label-based
    - pack picker convenience multi-select should reuse the same shared multi-select picker abstraction used by other picker families rather than introducing a pack-specific variant
    - confirming multiple pack selections should emit multiple sibling `pack` leaves into the current group using the same flattening behavior as other multi-select picker insertions
  - `scope` -> one focused selector dialog with category-first flow and subcategory mode defaulting to `any`
    - user-facing subcategory-state labels should be:
      - `Any subcategory`
      - `Specific subcategory`
      - `No subcategory`
    - only show the subcategory picker when `Specific subcategory` is chosen
  - `level` / `price` / `actionCost` -> centered floating numeric matcher dialog with shorthand parsing
  - `rarity` / `traits` / `families` and similar categorical values -> shared picker family
  - `metadataPredicate` -> field-first editor that routes into the appropriate picker/input family based on field semantics
  - `metric` -> dedicated metric predicate dialog
  - `metricCompare` -> dedicated metric-compare dialog
  - `anyOf` / `allOf` / `not` -> created and reshaped through structure actions rather than value-entry dialogs
- standardize the editor architecture around a fixed reusable set of editor views irrespective of origin
  - searchable single-select picker
  - searchable multi-select picker
  - text input editor
  - numeric matcher editor with shorthand parsing
  - boolean picker
  - staged builder flow composed from those same primitives
  - node families should map onto these shared editor views rather than inventing bespoke per-kind shells unnecessarily
- keep the mapping from node family or field family to editor-view type explicit, centralized, and declarative
  - examples:
    - `traits` -> compact searchable picker
    - `derivedTags` -> taxonomy explorer
    - `rarity` -> compact single-select picker
    - `pack` -> compact searchable picker with the shared multi-select affordance
    - `level` -> numeric matcher editor
  - the mapping should live in one well-owned place rather than being inferred ad hoc in multiple screens
  - later retuning of which editor-view family a field uses should be a light-weight wiring/configuration change, not a canonical-model or workflow refactor
- retire the old metadata explorer as the conceptual center of query editing
  - preserve reusable discovery pieces such as field discovery, value discovery, and picker filtering where they still help
  - rebuild metadata editing as one editor family among several rather than the special advanced-filter hub
- use a hybrid metric-key discovery flow
  - open one searchable metric-key picker
  - group entries by user-facing families/namespaces where helpful
  - scope suggestions and available entries to the current search context where possible
  - rely on generic `/` filtering within that picker
  - once a metric is chosen, route into the standardized editor view that matches the metric value type
  - `metricCompare` should follow the same family, choosing compatible numeric metrics on both sides

Representative scope editor examples:

```text
Scope Filter

Category
> Spell
  Creature
  Equipment
  Rule

Subcategory Mode
> Any subcategory
  Specific subcategory
  No subcategory

Preview: scope: spell / any

Enter confirm
Esc cancel
```

```text
Scope Filter

Category
> Rule
  Spell
  Creature
  Equipment

Subcategory Mode
  Any subcategory
> Specific subcategory
  No subcategory

Subcategory
> Action
  Condition
  Trait

Preview: scope: rule / action

Enter confirm
Esc cancel
```

```text
Scope Filter

Category
> Rule
  Spell
  Creature
  Equipment

Subcategory Mode
  Any subcategory
  Specific subcategory
> No subcategory

Preview: scope: rule / null subcategory

Enter confirm
Esc cancel
```

Representative metric-key picker:

```text
Choose Metric Key

Armor
> AC Total
  AC Bonus

Saves
  Acrobatics

Weapons
  Accuracy Penalty

Total Armor Class.

Esc cancel
Enter choose
```

Representative metric-key picker while filtering:

```text
Choose Metric Key

Armor
> AC Total
  AC Bonus

Weapons
  Accuracy Penalty

Total Armor Class.

Search /ac
Enter choose
Backspace edit
Esc clear and cancel
```

Representative metric predicate editor after key selection:

```text
Metric Filter

Metric: AC Total
Value: >=20
Preview: metric AC Total >= 20

Enter confirm
Esc cancel
```

- introduce a structured presentation-alias layer for friendly tree rendering
  - canonical operators and node kinds remain stable in the data model
  - UI rendering should use a compact controlled vocabulary such as `rarity is unique` and `trait includes fire`
  - scope rendering should stay compact and human:
    - full-category scope with default subcategory selection should render as the bare category, for example `rule`
    - specific subcategory should render as `rule / action`
    - explicit null-subcategory selection should render as `rule / null subcategory`
  - do not scatter raw phrase formatting across per-screen rendering code
- explicitly defer from this first pass:
  - leader-style keybinding systems
  - shortcut/simplification workflows beyond the verbose core editor
  - dedicated null-selection UI affordances for `actionCost`
  - sibling multi-select/group-wrap shortcuts for tree restructuring
- do not allow arbitrary raw value entry for exact-valued picker-backed fields
  - picker filter text is for narrowing visible options only
  - committing a value still requires selecting a valid option from the picker or explorer
  - raw text input remains appropriate for top-level search and for genuinely text-search-style field editors only if such fields are intentionally exposed later
- treat runtime caches as runtime caches, not as user-state persistence
  - in-memory result windows/sessions, picker discovery caches, and ontology/discovery snapshots may persist during a live app session
  - they should not be treated as restart-restored user state
- preserve user-authored boolean structure unless an action explicitly changes it
  - do not auto-unwrap single-child groups just because they currently contain one child
  - do not auto-flatten nested `allOf` / `anyOf` groups
  - do not auto-reduce nested `not` structure except when the user explicitly invokes the unwrap behavior on a selected `not` node
- treat `allOf`, `anyOf`, and `not` as standard boolean concepts and do not add dedicated explanatory UI for each one in the first pass
  - general search-page help is still useful, but boolean group semantics do not need special-case educational chrome

Representative discovery-mode picker examples:

Matching mode:

```text
Traits
Mode: Matching
Filter: fir

Showing values from the current matching query context.

> fire          128
  firearm        24
  firesight       3

Actions
: switch to Catalog
Esc cancel
Enter add selected
```

Catalog mode:

```text
Traits
Mode: Catalog
Filter: fir

Showing valid values for the current applicability scope,
even if the current matching set is zero.

> fire          128
  firearm        24
  firesight       3
  first-world     0

Actions
: switch to Matching
Esc cancel
Enter add selected
```

Sub-slice ownership breakdown:

- `G1` owns:
  - canonical `SearchRequest` workspace state adoption
  - mode selection flow and workspace shell behavior
  - route/session handoff rules
  - root-group projection, tree projection, and summary/document projection
- `G2` owns:
  - clause-kind selection
  - focused editors and dialog-local builder state
  - picker-driven node creation, multi-select insertion, and metric-key discovery flow
  - reuse of the shared dialog and picker primitives from `Slice E` and `Slice F`
- `G3` owns:
  - node actions for wrapping, grouping, moving, unwrapping, and lifting
  - valid insertion-slot move semantics
  - structural-preservation rules for boolean groups
  - presentation-alias rendering for friendly tree output

Validation:

- `G1`
  - the TUI uses the same discriminated union directly
  - `SearchRequest` is the only durable semantic query state after migration
  - no parallel TUI semantic query model remains in `src/tui/search/**` or `src/tui/search-screen/**`
  - the TUI still exposes one explicit search-facing facade through `services.user.search`, even if the implementation behind that facade is split into narrower owners
  - surviving workflow responsibilities do not drift into `search-screen/**` feature code
  - TUI restart remains a fresh boot with no restoration of prior search routes, queries, drafts, or result sessions
  - initial search entry requires explicit mode choice through the floating picker
  - browse does not accept/store/render text search inputs
  - search requires a query
  - lookup requires a query
  - mode changes from inside the workspace reopen the picker and only reset state when the selected mode actually changes
  - route/session handoff carries only intentional canonical/runtime context and excludes editor-local transient state
  - the query summary is navigable and reflects the canonical tree using presentation-friendly rendering
  - the shared search page uses a flat root-level projection with compact nested-group placeholders rather than expanding nested subtrees inline
  - compact group placeholders include child counts, for example `anyOf(2 filters)`, rather than opaque `(...)` placeholders
  - the shared search page exposes a selectable `Filters >` entry path into the dedicated filter-builder even when nested structure is not yet present
  - creating or opening nested boolean structure transitions into a dedicated full-tree filter-builder view instead of an isolated drill-in subtree view
  - the TUI tree always presents a visible root group, defaulting to `allOf`, without requiring the shared canonical model to persist a synthetic root wrapper
  - tree nodes with ephemeral ids/insertion slots remain derived editor projections rather than becoming canonical query state
  - the search-screen summary/document layer remains a derived presentation owner rather than being replaced by raw canonical rendering or promoted into a second semantic model
  - equivalent TUI edits and MCP requests land on the same canonical search meaning
  - in-memory caches and sessions align with the new canonical/runtime shapes without being treated as durable restored state
- `G2`
  - canonical workspace state is never represented as `Partial<SearchRequest>` or `Partial<SearchFilterNode>`
  - focused editors use local builder-style draft state rather than a mirrored full draft AST
  - canonical workspace state never contains incomplete/invalid draft nodes
  - metadata predicates and metric predicates are offered as explicit top-level editing choices
  - metadata editing is no longer organized around the old metadata explorer as the primary workflow
  - metric-key selection uses the hybrid searchable picker approach rather than a separate namespace-only wizard or an unstructured raw-key entry flow
  - editor implementations map onto a fixed reusable set of editor views irrespective of node origin
  - repeated additions can land flat within the current group when that is the selected insertion context
  - first-pass multi-select insertion inlines selected values as peer leaves in the current group rather than prompting for a separate connective/grouping choice
  - repeated or multi-select insertion keeps same-family sibling leaves adjacent within the group instead of interleaving them arbitrarily with unrelated leaf families
  - numeric matcher editors accept and validate shorthand textual matcher input
  - picker/edit mapping by node family is implemented consistently rather than ad hoc per screen
  - picker-style menus support generic `/` filtering behavior across the application, not just in command palettes
  - picker and explorer surfaces default to `Matching` discovery mode and allow switching to `Catalog` through the `:` action menu
  - picker and explorer counts reflect the active discovery mode (`Matching` = current result set, `Catalog` = applicability slice)
  - search-side `Matching` mode is driven by the active canonical `SearchRequest`, not by a query-agnostic catalog model
  - search-side `Matching` refresh is async-safe, including debounce/cancellation or equivalent stale-result protection
  - ontology/explorer adoption continues to use the same discovery semantics, but route/scope loading resolves to a stable in-memory model rather than a permanently live async tree
  - picker-backed exact-valued fields do not allow arbitrary raw value commits
- `G3`
  - tree restructuring is available through keyboard/context actions on the selected node without requiring a separate structure-editing mode
  - move operations navigate valid insertion slots rather than arbitrary tree rows
  - the tree renderer and move mode preserve explicit visible insertion targets and stable cross-level movement semantics
  - first-pass group editing does not rely on sibling multi-select or sibling-wrap shortcuts
  - `not` can wrap any selected node, and selected `not` nodes support an explicit unwrap behavior instead of nesting another `not`
  - single-child groups and nested boolean groups persist unless the user explicitly restructures them
  - UI-friendly rendering comes from a structured presentation-alias layer rather than one-off string formatting

### Slice H: Shared Result-View Presentation Capability

Goals:

- introduce shared result-view capabilities for grouping and lightweight result metadata before lookup adopts them
- keep grouping axes, section headers, row metadata, and detail metadata out of lookup-specific list/detail code
- make the existing shared list/detail and result-formatting owners the durable home for these hooks

Implementation shape:

- extend the existing shared list/detail presentation and formatting seams rather than introducing a second result-view framework beside them
  - acceptable shape: grouping/rendering hooks added to current shared owners such as `src/tui/list-detail-presentation.ts`, `src/tui/list-detail-formatting.ts`, and the shared search-result presentation path
  - unacceptable shape: a parallel generic result-view abstraction family that duplicates screen-model, grouping, or row/detail rendering responsibilities already owned by the shared list/detail layer
- expose result grouping as a shared capability whose grouping axis can be configured by the active mode
- expose shared result-presentation extension points for list and detail rendering
  - grouped-section headings should come from a shared result-grouping hook
  - per-row badges/subtitles should come from shared row-presentation metadata
  - the detail pane should allow optional generic metadata lines
- keep this seam generic and mode-agnostic
  - lookup will later supply `matchType` as one consumer input
  - browse and ordinary search do not need a grouping axis in this pass
- keep grouped-result rendering as a consumer of the existing list/detail presentation path rather than a replacement for it

Likely file set:

- shared list/detail and result-formatting owners under `src/tui/**`
- related tests for grouped and ungrouped result presentation behavior

Validation:

- result grouping is implemented through shared list/detail/result-formatting hooks rather than a lookup-only list implementation
- grouped section headings, row badges/subtitles, and detail metadata lines are all driven through shared presentation/formatting hooks rather than lookup-specific rendering paths
- no parallel result-view framework is introduced beside the existing shared list/detail layer
- the existing shared list/detail/result-formatting owners, not lookup-specific code, are the durable owners of these capabilities

Representative generic grouped-result examples:

Grouped mode:

```text
Results
Group By: matchType

Group: Exact
- Fireball
- Fireball Wand

Group: Normalized Exact
- Fire Ball Variant

Group: Fuzzy
- Firebrand
- Firewall
```

Flat mode with row metadata:

```text
Results
Group By: none

- Fireball           [exact]
- Fire Ball Variant  [normalized]
- Firewall           [fuzzy]
```

### Slice I: Lookup Output And Mode-Specific Sort Semantics

Goals:

- make mode-specific sort validity explicit in the contract
- keep browse sorts on browse only
- keep search-mode ranking fixed rather than exposing user sort
- decide and encode lookup-specific sort behavior
- expose lookup match strength in output data so the TUI can render it intentionally
- make lookup feel distinct primarily through sort/group configuration and light row/detail treatment, not a separate result-navigation model

Implementation shape:

- default lookup sort policy to `tiered`
- when lookup sort policy is `tiered`, render results as grouped sections by `matchType`
  - use subtle section headers such as `Exact`, `Normalized Exact`, and `Fuzzy`
  - do not also add per-row match badges when grouped by tier
- when lookup sort policy is `global`, render one flat result list with subtle per-row `matchType` badges
  - do not also add section headers in this mode
- consume the shared result-view grouping and presentation hooks introduced in `Slice H`
  - lookup should plug into that generic grouping mechanism via `matchType`
- make ownership explicit:
  - the shared search result-view/presentation layer owns optional grouping axes, section headers, row-presentation metadata, and optional detail metadata lines
  - lookup is only a consumer that supplies `matchType` as mode-specific grouping/presentation input
  - do not let lookup screens or lookup-only controllers become the durable owner of grouped-result rendering behavior
- keep lookup sort selection in the normal sort-selection flow rather than inventing a lookup-only sort surface
  - the difference from browse is semantic and contractual, not a different interaction surface
- keep lookup result details implementation generic
  - selected-result details should reuse the shared result-detail pane/surface
  - lookup can add a light match-type line or subtitle in that generic detail area without introducing a lookup-specific detail architecture

Representative TUI examples:

Lookup results, default `tiered` presentation:

```text
Lookup
Query: fireball
Sort: alphabetical (tiered)

Exact
- Fireball

Normalized Exact
- Fire Ball Variant

Fuzzy
- Firebrand
- Firewall
```

Lookup results, `global` presentation:

```text
Lookup
Query: fireball
Sort: alphabetical (global)

- Fire Ball Variant   [normalized]
- Fireball            [exact]
- Firebrand           [fuzzy]
- Firewall            [fuzzy]
```

Representative output requirement:

- lookup result records should expose match type explicitly, such as:
  - `exact`
  - `normalized_exact`
  - `fuzzy`

Validation:

- lookup output exposes explicit match type
- browse/search/lookup sort validity is enforced by the request model or schema
- lookup defaults to `tiered` sort policy
- `tiered` lookup presentation uses section headers without per-row badges
- `global` lookup presentation uses per-row badges without section headers
- lookup detail treatment remains a light extension of the generic result-detail surface

### Slice J: Remove `isUnique` From Searchable Metadata Vocabulary

Goals:

- remove `isUnique` from metadata field ownership where it is exposed as searchable/filterable vocabulary
- remove `isUnique` from filter-value discovery surfaces
- update labels/examples/tests so `isUnique` is not presented as a filter concept

Non-goals:

- do not remove `record.isUnique` from normalized record/runtime shapes unless a separate audit proves it is dead
- do not change ranking or other non-filter logic that still legitimately uses record uniqueness

Likely file set:

- `src/domain/metadata-field-types.ts`
- `src/domain/search-types.ts`
- `src/domain/presentation-vocabulary.ts`
- `src/search/filters/registry.ts`
- `src/search/filters/semantics.ts`
- affected tests

Validation:

- `isUnique` is absent from search/filter vocabulary
- no search/filter-facing surface still advertises it

### Slice K: Public MCP Surface, Docs, And Hard Migration Follow-Through

Goals:

- update MCP tool schemas and descriptions to the new `mode` / `search` / `filter` contract
- update `docs/architecture/search.md`
- update `docs/architecture/tui.md`
- update `docs/architecture/boundaries.md`
- add or update ADRs for the durable search-contract and discovery-boundary decisions introduced by this plan
- update backlog wording if needed so it matches the actual intended end state
- migrate durable persisted search-adjacent artifacts only if any truly durable artifact is touched by this refactor
  - do not add compatibility readers
  - in-memory route payloads, result sessions, picker drafts, and other runtime-only state do not need cross-restart migration
  - if no durable persisted search-state artifact exists beyond the prepared index/runtime data, then there is no migration surface for this pass
- rebuild any affected derived artifacts only if implementation proves the new model requires it
  - no database/index schema migration is expected from the current planned contract and TUI work alone

Representative ontology handoff examples to preserve in workflow shape:

Ontology leaf/detail view:

```text
Ontology > Search Semantics > Traits > Fire

Fire
Live canonical records: 128
Scope: spell, creature, equipment

Actions
Enter open matching results
o open in search workspace
Esc back
```

Opened result reader:

```text
Results
Seeded From: Ontology / Traits / Fire
Query Context: browse
Filter: trait includes fire

- Fireball
- Flaming Star
- Efreet
```

Opened search workspace:

```text
Search
Seeded From: Ontology / Traits / Fire
Mode: Browse

Filters >
allOf
├─ trait includes fire
└─ [+ add filter]
```

Validation:

- MCP docs and schemas match the canonical contract
- architecture docs name the landed discovery boundary and approved owner path
- required ADR updates capture the durable decisions behind the new search contract, discovery boundary, and enforcement shape
- `docs/architecture/search.md`, `docs/architecture/tui.md`, and `docs/architecture/boundaries.md` are updated to reflect ADRs `0011` through `0014`, not just the implementation details of the landing branch
- no old serialized-shape readers remain
- any required rebuild steps are completed and documented

## 5. Validation Plan

Targeted validation during implementation:

- contract tests
  - `SearchRequest` supports the agreed discriminated union and shared-base split
  - `mode` branches enforce the expected presence/absence of `search`, `sort`, and `explain`
- schema tests
  - old flat root filter fields are removed
  - `mode` / `search` / `filter` are accepted and documented
- boolean/filter tests
  - `anyOf`, `allOf`, and `not` work across heterogeneous children
  - disjoint range unions are representable and compile correctly
  - negated links work through `not(linksTo(...))`
- metadata tests
  - atomic metadata predicates compile and evaluate correctly
  - `isUnique` no longer appears in searchable/filterable metadata vocabulary
- TUI tests
  - browse has no search input/state
  - search requires a query
  - lookup requires a query
  - TUI mode-specific UI and lowering behavior match the shared contract
- lookup tests
  - match type is exposed in output
  - lookup sort validity follows the final lookup sort contract
- migration/rebuild tests or checks
  - no backward-compatibility readers were added for runtime-only search state
  - any truly durable persisted artifact touched by the refactor was either rewritten directly or explicitly shown not to exist
  - any required rebuild step was run, but no index/schema rebuild should be assumed unless implementation proves it necessary

Refactor-completion checks:

- no remaining canonical shared/public field named `intent`
- no remaining canonical shared/public field named `parts`
- no remaining canonical shared/public field named `filters`
- no remaining canonical flat root filter fields such as `category`, `subcategory`, `levelMin`, `rarity`, `metadata`, `linksTo`, `excludeLinksTo`, `text`, `excludeQuery`, or `searchProfile`
- no alias fields, alias types, alias schemas, or legacy upgrade helpers preserving the old search model
- no `explain` outside the `search` branch
- no backward-compatibility readers for any durable persisted search-state artifact touched by this refactor
- no accidental persistence/migration logic added for runtime-only TUI/editor/search-session state

Final validation for implementation turn:

- `npm run build`
- `cd scripts && npm test`
- validate the finished code against this plan file before reporting completion

## 6. Landing Workflow

- create a dedicated git worktree under `/tmp`
- implement and validate in that worktree
- commit only once the full combined pass is coherent and validated
- inspect the shared `main` checkout before landing
- rebase the worktree branch onto current `main`
- rerun `npm run build` and `cd scripts && npm test`
- merge back only after the rebased worktree is green and `main` is clean
- remove the temporary worktree before reporting completion

## 7. Docs And Backlog Follow-Through

Required docs:

- `docs/architecture/search.md`
- `docs/architecture/tui.md`
- `docs/architecture/boundaries.md`

Architecture-doc follow-through should explicitly reflect ADRs `0011` through `0014` once implementation lands.

Required ADR follow-through:

- record the durable decisions behind:
  - the canonical `mode` / `search` / `filter` search contract
  - the shared app-facing discovery service boundary
  - the lint-enforced follow-through once that boundary becomes mandatory

Backlog follow-through:

- if both concerns fully land, move the completed backlog item files into history in the implementation task
- if the remaining scope changes shape, update the backlog wording so it describes the discriminated `mode` / `search` / `filter` destination rather than the older shorthand

## 8. Resolved Planning Decisions

- do replace the public MCP input shape for `pf2e_search` and `pf2e_list_records` in this pass
- the long-term public/shared search surface should use `mode` plus optional `search` plus a single root `filter` tree
- the TUI should adopt that same contract directly
- `intent` / `parts` should be removed outright rather than preserved as aliases
- the canonical filter model should use atomic leaves plus `anyOf` / `allOf` / `not`
- `isUnique` should not remain in searchable/filterable vocabulary
- serialized search-adjacent shapes touched by this refactor should be hard-migrated, with rebuilds if needed
- no database/index schema migration is expected unless implementation uncovers a concrete runtime need that the current prepared index cannot satisfy
