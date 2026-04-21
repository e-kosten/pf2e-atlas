import {
  CATEGORY_SUBCATEGORY_MAP,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../../domain/categories.js";
import { getMetadataFilterSemantics, type MetadataFieldSemantics } from "../../domain/metadata-semantics.js";
import type { MetadataFieldName } from "../../domain/metadata-field-registry.js";
import type {
  SearchCategory,
  SearchSubcategory,
} from "../../domain/search-types.js";
import {
  applyDiscoverableQueryFieldSelections,
  buildDiscoverableQueryFieldSelections,
  getQueryFieldOptions,
  getScopedMetadataFields,
} from "./discoverable-fields.js";
import {
  applyFilterExplorerDraft,
  buildFilterExplorerMetadataNode,
  cloneFilterExplorerDraft,
  createFilterExplorerDraftFromMetadataNode,
  createFilterExplorerDraftFromQuery,
  withFilterExplorerComposeDraft,
} from "./filter-explorer.js";
import { buildSearchFilters } from "./filter-building.js";
import { createSearchQueryFromOntologyQuery } from "./ontology-query.js";
import {
  createDefaultQuery,
  isActionCostAvailableInScope,
  normalizeSearchQuery,
} from "./query-state.js";
import {
  appendSearchSessionWindowPage,
  buildSearchWindowFilters,
  createSearchSessionFromWindow,
  replaceSearchSessionWindowPage,
} from "./search-window-session.js";
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
} from "./service-options.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalSearchQuery,
  Pf2eTerminalSearchService,
  Pf2eTerminalSearchSession,
  Pf2eTerminalSearchSort,
  SearchServiceDependencies,
} from "./service-types.js";

export type {
  LegacyFacetSelection,
  Pf2eTerminalFacetField,
  Pf2eTerminalFacetFieldOption,
  Pf2eTerminalFacetValueOption,
  Pf2eTerminalFilterExplorerDraft,
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
} from "./service-types.js";
export {
  applyFilterExplorerDraft,
  buildFilterExplorerMetadataNode,
  buildSearchFilterExplorerModel,
  buildSearchFilterExplorerTargetResolver,
  cloneFilterExplorerDraft,
  createFilterExplorerDraftFromMetadataNode,
  createFilterExplorerDraftFromQuery,
  withFilterExplorerComposeDraft,
} from "./filter-explorer.js";
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
} from "./query-state.js";
export type {
  Pf2eTerminalMetadataQueryPart,
  Pf2eTerminalQueryPart,
  Pf2eTerminalQueryPartKind,
  Pf2eTerminalQueryPartPolicy,
} from "./query-parts.js";

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

  return {
    createDefaultQuery: () => createDefaultQuery(),
    createQueryFromOntologyQuery: (query) =>
      createSearchQueryFromOntologyQuery(query, dependencies, fieldSemanticsByName),
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
    createFilterExplorerDraft: (query, scopedFields) =>
      createFilterExplorerDraftFromQuery(query, scopedFields, fieldSemanticsByName),
    createFilterExplorerDraftFromMetadataNode: (node, scopedFields) =>
      createFilterExplorerDraftFromMetadataNode(node, scopedFields, fieldSemanticsByName),
    buildFilterExplorerMetadataNode: (draft) => buildFilterExplorerMetadataNode(draft, fieldSemanticsByName),
    applyFilterExplorerDraft: (query, draft) => applyFilterExplorerDraft(query, draft, fieldSemanticsByName),
    buildDiscoverableQueryFieldSelections: (query, scopedFields) =>
      buildDiscoverableQueryFieldSelections(query, scopedFields, fieldSemanticsByName),
    applyDiscoverableQueryFieldSelections: (query, selections, scopedFields) =>
      applyDiscoverableQueryFieldSelections(query, selections, scopedFields, fieldSemanticsByName),
    getCategoryOptions: () => {
      const categorySummary = dependencies.getSearchCategorySummary?.() ?? {
        categories: dependencies.getSearchVocabulary().categories,
      };
      return [
        {
          value: null,
          label: "Any Category",
          description: "Search or browse across the full indexed PF2E corpus.",
        },
        ...categorySummary.categories.map((category) => ({
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
        buildSearchWindowFilters(normalizedQuery, {
          sort,
          sortSeed,
          limit,
        }),
        { mode: normalizedQuery.mode },
      );
      return createSearchSessionFromWindow(normalizedQuery, result);
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

        nextSession = appendSearchSessionWindowPage(nextSession, result);
      }

      return Promise.resolve(nextSession);
    },
    readResultWindow: (session, options) => {
      const limit = Math.max(1, options.limit);
      const clampedOffset = Math.max(0, Math.min(options.offset, Math.max(0, session.total - limit)));
      const result = dependencies.readSearchWindowPage(session.windowId, clampedOffset, limit);

      return Promise.resolve(replaceSearchSessionWindowPage(session, result));
    },
    changeSort: async (session, sort) => {
      dependencies.closeSearchWindow(session.windowId);
      const sortSeed = sort === "random" ? createSortSeed(sort) : null;
      const result = await dependencies.openSearchWindow(
        buildSearchWindowFilters(session.query, {
          sort,
          sortSeed,
          limit: Math.max(session.query.limit, session.loadedCount),
        }),
        { mode: session.query.mode },
      );
      return createSearchSessionFromWindow(session.query, result);
    },
  };
}
