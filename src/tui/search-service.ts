import { CATEGORY_SUBCATEGORY_MAP, normalizeSearchCategory, normalizeSearchSubcategory } from "../domain/categories.js";
import { getMetadataFilterSemantics, type MetadataFieldSemantics } from "../domain/metadata-semantics.js";
import type { MetadataFieldName } from "../domain/metadata-field-registry.js";
import type { SearchVocabularyResult } from "../data/vocabulary.js";
import type {
  OntologyNodeQuery,
  SearchCategory,
  SearchFilters,
  SearchSubcategory,
  SearchWindowPage,
} from "../domain/index.js";
import {
  applyDiscoverableQueryFieldSelections,
  buildDiscoverableQueryFieldSelections,
  getQueryFieldOptions,
  getScopedMetadataFields,
} from "./search/discoverable-fields.js";
import { buildSearchFilters } from "./search/filter-building.js";
import {
  createDefaultQuery,
  isActionCostAvailableInScope,
  normalizeSearchQuery,
  splitMetadataTreeIntoParts,
} from "./search/query-state.js";
import {
  FACET_FIELD_EXCLUSIONS,
  SEARCH_MODE_OPTIONS,
  SEARCH_PROFILE_OPTIONS,
  SEARCH_SORT_OPTIONS,
  createFacetValueOptions,
  createSortSeed,
  formatCategoryLabel,
  formatSubcategoryLabel,
  getDefaultSort,
  humanizeIdentifier,
} from "./search/service-options.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalSearchQuery,
  Pf2eTerminalSearchService,
  Pf2eTerminalSearchSession,
  Pf2eTerminalSearchSort,
  SearchServiceDependencies,
} from "./search/service-types.js";

export type {
  LegacyFacetSelection,
  Pf2eTerminalFacetField,
  Pf2eTerminalFacetFieldOption,
  Pf2eTerminalFacetValueOption,
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalQueryField,
  Pf2eTerminalQueryFieldEditor,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalQueryFieldSelectionMap,
  Pf2eTerminalSearchCategoryOption,
  Pf2eTerminalSearchFilters,
  Pf2eTerminalSearchMode,
  Pf2eTerminalSearchModeOption,
  Pf2eTerminalSearchProfileOption,
  Pf2eTerminalSearchQuery,
  Pf2eTerminalSearchService,
  Pf2eTerminalSearchSession,
  Pf2eTerminalSearchSort,
  Pf2eTerminalSearchSortOption,
  Pf2eTerminalSearchStructuredPart,
  Pf2eTerminalSearchSubcategoryOption,
  SearchServiceDependencies,
} from "./search/service-types.js";
export {
  getSearchQueryActionCostPolicy,
  getSearchQueryCategory,
  getSearchQueryLevelRange,
  getSearchQueryMetadataTree,
  getSearchQueryPart,
  getSearchQueryRarityPolicy,
  getSearchQuerySubcategory,
  removeSearchQueryPart,
  setSearchQueryCategory,
  setSearchQueryMetadataTree,
  setSearchQueryPart,
} from "./search/query-state.js";
export type {
  Pf2eTerminalMetadataQueryPart,
  Pf2eTerminalQueryPart,
  Pf2eTerminalQueryPartKind,
  Pf2eTerminalQueryPartPolicy,
} from "./search-query-parts.js";

function createSessionFromResult(query: Pf2eTerminalSearchQuery, result: SearchWindowPage): Pf2eTerminalSearchSession {
  const sessionQuery =
    result.limit === query.limit
      ? query
      : {
          ...query,
          limit: result.limit,
        };

  return {
    windowId: result.id,
    query: sessionQuery,
    results: result.records,
    windowOffset: result.offset,
    resultMode: result.mode,
    total: result.total,
    loadedCount: result.records.length,
    hasMore: result.hasMore,
    nextOffset: result.nextOffset,
    searchProfile: result.searchProfile,
    sort: result.sort,
    sortSeed: result.sortSeed,
  };
}

export function createPf2eTerminalSearchService(dependencies: SearchServiceDependencies): Pf2eTerminalSearchService {
  const filterSemantics = getMetadataFilterSemantics();
  const fieldSemanticsByName = new Map<Pf2eTerminalFacetField, MetadataFieldSemantics>(
    filterSemantics.metadataFields.map((entry) => [entry.field, entry]),
  );

  function getFieldValueOrdering(field: MetadataFieldName) {
    return fieldSemanticsByName.get(field)?.valueOrdering;
  }

  function isActionCostRelevant(category: SearchCategory | null, subcategory: SearchSubcategory | null): boolean {
    return isActionCostAvailableInScope(dependencies, category, subcategory);
  }

  function buildWindowFilters(
    query: Pf2eTerminalSearchQuery,
    options: {
      sort: Pf2eTerminalSearchSort;
      sortSeed: number | null;
      limit: number;
      offset?: number;
    },
  ): SearchFilters {
    const offset = options.offset ?? 0;
    if (query.mode === "lookup") {
      return buildSearchFilters(query, {
        limit: options.limit,
        offset,
        nameQuery: query.queryText,
        sort: options.sort,
        sortSeed: options.sortSeed,
      });
    }

    return buildSearchFilters(query, {
      limit: options.limit,
      offset,
      query: query.mode === "search" ? query.queryText : undefined,
      searchProfile: query.mode === "search" ? query.searchProfile : undefined,
      sort: options.sort,
      sortSeed: options.sortSeed,
    });
  }

  function createQueryFromOntologyQuery(query: OntologyNodeQuery): Pf2eTerminalSearchQuery {
    const defaultQuery = createDefaultQuery();
    const category = normalizeSearchCategory(query.filters.category) ?? null;
    const normalizedSubcategory = normalizeSearchSubcategory(query.filters.subcategory) ?? null;
    const subcategory =
      category && normalizedSubcategory && CATEGORY_SUBCATEGORY_MAP[category].includes(normalizedSubcategory)
        ? normalizedSubcategory
        : null;
    const parts: Pf2eTerminalSearchQuery["filters"]["parts"] = [];
    if (subcategory) {
      parts.push({ kind: "subcategory", subcategory });
    }
    if (query.filters.levelMin !== undefined || query.filters.levelMax !== undefined) {
      parts.push({
        kind: "levelRange",
        levelMin: query.filters.levelMin ?? null,
        levelMax: query.filters.levelMax ?? null,
      });
    }
    if (query.filters.rarity) {
      parts.push({
        kind: "rarityPolicy",
        policy: {
          any: [query.filters.rarity],
          all: [],
          exclude: [],
        },
      });
    }
    if (query.filters.actionCost !== undefined) {
      parts.push({
        kind: "actionCostPolicy",
        policy: {
          any: [query.filters.actionCost],
          all: [],
          exclude: [],
        },
      });
    }
    parts.push(...splitMetadataTreeIntoParts(query.filters.metadata ?? null));

    return normalizeSearchQuery(
      {
        ...defaultQuery,
        mode: query.kind === "lookup" ? "lookup" : query.kind === "search" ? "search" : "browse",
        limit: query.filters.limit ?? defaultQuery.limit,
        queryText: query.filters.query ?? query.filters.nameQuery ?? "",
        searchProfile: query.filters.searchProfile ?? defaultQuery.searchProfile,
        sourceLabel: query.label ?? null,
        filters: {
          ...defaultQuery.filters,
          category,
          parts,
        },
      },
      dependencies,
      fieldSemanticsByName,
    );
  }

  return {
    createDefaultQuery: () => createDefaultQuery(),
    createQueryFromOntologyQuery,
    getAvailableRootQueryPartKinds: (category, subcategory) => [
      ...(category && CATEGORY_SUBCATEGORY_MAP[category].length > 0 ? (["subcategory"] as const) : []),
      "levelRange",
      "rarityPolicy",
      ...(isActionCostRelevant(category, subcategory) ? (["actionCostPolicy"] as const) : []),
      ...(category ? (["metadataPredicate", "metadataGroup", "metadataNot"] as const) : []),
    ],
    getRootQueryParts: (query) => normalizeSearchQuery(query, dependencies, fieldSemanticsByName).filters.parts,
    applyRootQueryParts: (query, parts) =>
      normalizeSearchQuery(
        {
          ...query,
          filters: {
            ...query.filters,
            parts,
          },
        },
        dependencies,
        fieldSemanticsByName,
      ),
    buildDiscoverableQueryFieldSelections: (query, scopedFields) =>
      buildDiscoverableQueryFieldSelections(query, scopedFields, fieldSemanticsByName),
    applyDiscoverableQueryFieldSelections: (query, selections, scopedFields) =>
      applyDiscoverableQueryFieldSelections(query, selections, scopedFields, fieldSemanticsByName),
    getCategoryOptions: () => {
      const vocabulary = dependencies.getSearchVocabulary();
      return [
        {
          value: null,
          label: "Any Category",
          description: "Search or browse across the full indexed PF2E corpus.",
        },
        ...vocabulary.categories.map((category: SearchVocabularyResult["categories"][number]) => ({
          value: category.value,
          label: formatCategoryLabel(category.value),
          description: `${category.count} indexed canonical record${category.count === 1 ? "" : "s"}.`,
        })),
      ];
    },
    getFacetFieldOptions: (category, subcategory) => {
      const candidateFields = getScopedMetadataFields(filterSemantics, category, subcategory)
        .map((field) => fieldSemanticsByName.get(field))
        .filter((field): field is MetadataFieldSemantics => Boolean(field))
        .filter(
          (field) =>
            field.discoverable &&
            !FACET_FIELD_EXCLUSIONS.has(field.field) &&
            (["set", "enumString", "boolean"].includes(field.fieldType) ||
              (field.field === "actionCost" &&
                dependencies.listFilterValues({
                  field: "actionCost",
                  ...(category ? { category } : {}),
                  ...(subcategory ? { subcategory } : {}),
                }).values.length > 0)),
        );

      return candidateFields.map((field) => ({
        value: field.field,
        label: humanizeIdentifier(field.field),
        description: field.notes ?? `${field.fieldType} facet for the current browse scope.`,
        fieldType: field.fieldType,
      }));
    },
    getQueryFieldOptions: (category, subcategory) =>
      getQueryFieldOptions(filterSemantics, fieldSemanticsByName, category, subcategory),
    getFacetValueOptions: (field, category, subcategory) =>
      createFacetValueOptions(
        dependencies.listFilterValues({
          field,
          ...(category ? { category } : {}),
          ...(subcategory ? { subcategory } : {}),
        }).values,
        {
          ordering: getFieldValueOrdering(field),
        },
      ),
    getProfileOptions: () => SEARCH_PROFILE_OPTIONS,
    getResultSortOptions: (mode) => SEARCH_SORT_OPTIONS[mode],
    getRarityOptions: (category, subcategory) =>
      createFacetValueOptions(
        dependencies.listFilterValues({
          field: "rarity",
          ...(category ? { category } : {}),
          ...(subcategory ? { subcategory } : {}),
        }).values,
        {
          ordering: getFieldValueOrdering("rarity"),
        },
      ),
    getActionCostOptions: (category, subcategory) =>
      isActionCostRelevant(category, subcategory)
        ? createFacetValueOptions(
            dependencies.listFilterValues({
              field: "actionCost",
              ...(category ? { category } : {}),
              ...(subcategory ? { subcategory } : {}),
            }).values,
            {
              ordering: getFieldValueOrdering("actionCost"),
              labelFormatter: (value) => `${value} action${value === "1" ? "" : "s"}`,
            },
          )
        : [],
    getSubcategoryOptions: (category) => {
      if (!category) {
        return [
          {
            value: null,
            label: "Any Subcategory",
            description: "Choose a category first to narrow the browse scope further.",
          },
        ];
      }

      return [
        {
          value: null,
          label: "Any Subcategory",
          description: `Browse every ${formatCategoryLabel(category)} record in the current category.`,
        },
        ...CATEGORY_SUBCATEGORY_MAP[category].map((subcategory) => ({
          value: subcategory,
          label: formatSubcategoryLabel(subcategory),
          description: `Restrict the workspace to ${formatSubcategoryLabel(subcategory)} records.`,
        })),
      ];
    },
    getModeOptions: () => SEARCH_MODE_OPTIONS,
    getDefaultSort: (mode) => getDefaultSort(mode),
    normalizeQuery: (query) => normalizeSearchQuery(query, dependencies, fieldSemanticsByName),
    countQuery: (query) => {
      const normalizedQuery = normalizeSearchQuery(query, dependencies, fieldSemanticsByName);
      if (normalizedQuery.mode === "lookup") {
        if (!normalizedQuery.queryText) {
          return Promise.resolve({
            searchProfile: null,
            mode: "structured",
            total: 0,
          });
        }
        return dependencies.countRecords(
          buildSearchFilters(normalizedQuery, {
            limit: 1,
            nameQuery: normalizedQuery.queryText,
          }),
          { mode: "lookup" },
        );
      }

      if (normalizedQuery.mode === "browse" || !normalizedQuery.queryText) {
        return dependencies.countRecords(buildSearchFilters(normalizedQuery, { limit: 1 }), {
          mode: "browse",
        });
      }

      return dependencies.countRecords(
        buildSearchFilters(normalizedQuery, {
          limit: 1,
          query: normalizedQuery.queryText,
          searchProfile: normalizedQuery.searchProfile,
        }),
        { mode: "search", lexicalOnly: true },
      );
    },
    disposeSession: (session) => {
      dependencies.closeSearchWindow(session.windowId);
    },
    executeQuery: async (query, options = {}) => {
      const normalizedQuery = normalizeSearchQuery(query, dependencies, fieldSemanticsByName);
      const sort = options.sort ?? getDefaultSort(normalizedQuery.mode);
      const sortSeed = sort === "random" ? createSortSeed(sort) : null;
      const limit = options.limit ?? normalizedQuery.limit;
      const result = await dependencies.openSearchWindow(
        buildWindowFilters(normalizedQuery, {
          sort,
          sortSeed,
          limit,
        }),
        { mode: normalizedQuery.mode },
      );
      return createSessionFromResult(normalizedQuery, result);
    },
    loadMore: (session, options = {}) => {
      if (!session.hasMore || session.nextOffset === null) {
        return Promise.resolve(session);
      }

      const minimumLoadedCount = Math.max(
        session.loadedCount + 1,
        options.minimumLoadedCount ?? session.loadedCount + session.query.limit,
      );
      let nextSession = session;

      while (nextSession.hasMore && nextSession.nextOffset !== null && nextSession.loadedCount < minimumLoadedCount) {
        const result = dependencies.readSearchWindowPage(
          nextSession.windowId,
          nextSession.nextOffset,
          nextSession.query.limit,
        );

        nextSession = {
          ...nextSession,
          query:
            result.limit === nextSession.query.limit
              ? nextSession.query
              : {
                  ...nextSession.query,
                  limit: result.limit,
                },
          results: [...nextSession.results, ...result.records],
          total: result.total,
          loadedCount: nextSession.results.length + result.records.length,
          hasMore: result.hasMore,
          nextOffset: result.nextOffset,
          resultMode: result.mode,
          searchProfile: result.searchProfile,
        };
      }

      return Promise.resolve(nextSession);
    },
    readResultWindow: (session, options) => {
      const limit = Math.max(1, options.limit);
      const clampedOffset = Math.max(0, Math.min(options.offset, Math.max(0, session.total - limit)));
      const result = dependencies.readSearchWindowPage(session.windowId, clampedOffset, limit);

      return Promise.resolve({
        ...session,
        query:
          result.limit === session.query.limit
            ? session.query
            : {
                ...session.query,
                limit: result.limit,
              },
        results: result.records,
        windowOffset: result.offset,
        total: result.total,
        loadedCount: result.records.length,
        hasMore: result.hasMore,
        nextOffset: result.nextOffset,
        resultMode: result.mode,
        searchProfile: result.searchProfile,
      });
    },
    changeSort: async (session, sort) => {
      dependencies.closeSearchWindow(session.windowId);
      const sortSeed = sort === "random" ? createSortSeed(sort) : null;
      const result = await dependencies.openSearchWindow(
        buildWindowFilters(session.query, {
          sort,
          sortSeed,
          limit: Math.max(session.query.limit, session.loadedCount),
        }),
        { mode: session.query.mode },
      );
      return createSessionFromResult(session.query, result);
    },
  };
}
