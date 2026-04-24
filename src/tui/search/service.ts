import {
  CATEGORY_SUBCATEGORY_MAP,
} from "../../domain/categories.js";
import { createScopedSearchDiscoveryApplicability } from "../../app/search-discovery-service.js";
import { getMetadataFilterSemantics, type MetadataFieldSemantics } from "../../search/filters/semantics.js";
import type { MetadataFieldName } from "../../domain/metadata-field-types.js";
import {
  describeMetadataFieldType,
  formatMetadataFieldLabel,
} from "../../domain/presentation-vocabulary.js";
import type {
  SearchCategory,
  SearchSubcategory,
} from "../../domain/search-types.js";
import {
  applyDiscoverableQueryFieldSelections,
  buildDiscoverableQueryFieldSelections,
  getQueryFieldOptions,
} from "./discoverable-fields.js";
import {
  applyFilterExplorerDraft,
  buildFilterExplorerMetadataNode,
  prepareFilterExplorerDraftFromMetadataNode,
  prepareFilterExplorerDraftFromQuery,
} from "../filter-explorer/search-draft-query.js";
import { buildSearchRequest } from "./filter-building.js";
import { createSearchQueryFromOntologyQuery } from "./ontology-query.js";
import {
  createDefaultQuery,
  getSearchQueryActionCostPolicy,
  getSearchQueryCategory,
  getSearchQueryRarityPolicy,
  getSearchQuerySubcategory,
  isActionCostAvailableInScope,
  normalizeSearchQuery,
  setSearchQueryActionCostPolicy,
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
} from "./service-options.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalSearchService,
  SearchServiceDependencies,
} from "./service-types.js";

export type {
  Pf2eTerminalFacetField,
  Pf2eTerminalFacetFieldOption,
  Pf2eTerminalPreparedFilterExplorerContext,
  Pf2eTerminalPreparedFilterExplorerDraft,
  Pf2eTerminalFacetValueOption,
  Pf2eTerminalFilterExplorerDraft,
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalQueryField,
  Pf2eTerminalQueryFieldEditor,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalQueryFieldSelectionMap,
  Pf2eTerminalSearchCategoryOption,
  Pf2eTerminalSearchMode,
  Pf2eTerminalSearchModeOption,
  Pf2eTerminalSearchProfileOption,
  Pf2eTerminalSearchQuery,
  Pf2eTerminalSearchService,
  Pf2eTerminalSearchSession,
  Pf2eTerminalSearchSort,
  Pf2eTerminalSearchSortOption,
  Pf2eTerminalSearchSubcategoryOption,
  SearchServiceDependencies,
} from "./service-types.js";

const DEFAULT_QUERY_LIMIT = 50;

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

  function normalizeServiceQuery(query: import("./service-types.js").Pf2eTerminalSearchQuery) {
    const normalizedQuery = normalizeSearchQuery(query);
    const category = getSearchQueryCategory(normalizedQuery);
    const subcategory = getSearchQuerySubcategory(normalizedQuery);
    if (isActionCostRelevant(category, subcategory)) {
      return normalizedQuery;
    }

    const actionCostPolicy = getSearchQueryActionCostPolicy(normalizedQuery);
    return actionCostPolicy.any.length > 0 || actionCostPolicy.all.length > 0 || actionCostPolicy.exclude.length > 0
      ? setSearchQueryActionCostPolicy(normalizedQuery, { any: [], all: [], exclude: [] })
      : normalizedQuery;
  }

  return {
    createDefaultQuery: (mode) => createDefaultQuery(mode),
    createQueryFromOntologyQuery: (query) =>
      createSearchQueryFromOntologyQuery(query, dependencies, fieldSemanticsByName),
    prepareFilterExplorerDraft: (query, scopedFields) =>
      prepareFilterExplorerDraftFromQuery(query, scopedFields, fieldSemanticsByName),
    prepareFilterExplorerDraftFromMetadataNode: (node, scopedFields) =>
      prepareFilterExplorerDraftFromMetadataNode(node, scopedFields, fieldSemanticsByName),
    buildFilterExplorerMetadataNode: (draft, options) =>
      buildFilterExplorerMetadataNode(draft, fieldSemanticsByName, options),
    applyFilterExplorerDraft: (query, draft, options) =>
      applyFilterExplorerDraft(query, draft, fieldSemanticsByName, options),
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
      const candidateFields = dependencies.discovery
        .getScopedMetadataFields(category ? { category, subcategory } : null)
        .filter(
          (field) =>
            field.discoverable &&
            !FACET_FIELD_EXCLUSIONS.has(field.field) &&
            (["set", "enumString", "boolean"].includes(field.fieldType) ||
              (field.field === "actionCost" && isActionCostRelevant(category, subcategory))),
        );

      return candidateFields.map((field) => ({
        value: field.field,
        label: formatMetadataFieldLabel(field.field),
        description: field.notes ?? `${describeMetadataFieldType(field.fieldType)} facet for the current browse scope.`,
        fieldType: field.fieldType,
      }));
    },
    getQueryFieldOptions: (category, subcategory) =>
      getQueryFieldOptions(
        dependencies.discovery.getScopedMetadataFields(category ? { category, subcategory } : null),
        dependencies.discovery.getMetricDiscoveryGroups(category ? { category, subcategory } : null),
        category,
      ),
    getFacetValueOptions: (field, category, subcategory) =>
      createFacetValueOptions(
        dependencies.discovery
          .discoverCatalogFilterValues({
            applicability: createScopedSearchDiscoveryApplicability("browse", category, subcategory),
            target: { field },
          })
          .options.map((entry) => ({
            value: String(entry.value),
            count: entry.count,
          })),
        {
          ordering: getFieldValueOrdering(field),
        },
      ),
    getProfileOptions: () => SEARCH_PROFILE_OPTIONS,
    getResultSortOptions: (mode) => SEARCH_SORT_OPTIONS[mode],
    getRarityOptions: (category, subcategory) =>
      createFacetValueOptions(
        dependencies.discovery
          .discoverCatalogFilterValues({
            applicability: createScopedSearchDiscoveryApplicability("browse", category, subcategory),
            target: { field: "rarity" },
          })
          .options.map((entry) => ({
            value: String(entry.value),
            count: entry.count,
          })),
        {
          ordering: getFieldValueOrdering("rarity"),
        },
      ),
    getActionCostOptions: (category, subcategory) =>
      isActionCostRelevant(category, subcategory)
        ? createFacetValueOptions(
            dependencies.discovery
              .discoverCatalogFilterValues({
                applicability: createScopedSearchDiscoveryApplicability("browse", category, subcategory),
                target: { field: "actionCost" },
              })
              .options.map((entry) => ({
                value: String(entry.value),
                count: entry.count,
              })),
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
    normalizeQuery: (query) => normalizeServiceQuery(query),
    countQuery: (query) => {
      const normalizedQuery = normalizeServiceQuery(query);
      if (normalizedQuery.mode === "lookup") {
        if (!normalizedQuery.search.query) {
          return Promise.resolve({
            searchProfile: null,
            mode: "structured",
            total: 0,
          });
        }
        return dependencies.countRecords(
          buildSearchRequest(normalizedQuery, {
            limit: 1,
            text: normalizedQuery.search.query,
          }),
          {},
        );
      }

      if (normalizedQuery.mode === "browse" || !normalizedQuery.search.query) {
        return dependencies.countRecords(buildSearchRequest(normalizedQuery, { limit: 1 }));
      }

      return dependencies.countRecords(
        buildSearchRequest(normalizedQuery, {
          limit: 1,
          text: normalizedQuery.search.query,
          exclude: normalizedQuery.search.exclude,
          searchProfile: normalizedQuery.search.profile,
        }),
        { lexicalOnly: true },
      );
    },
    disposeSession: (session) => {
      dependencies.closeSearchWindow(session.windowId);
    },
    executeQuery: async (query, options = {}) => {
      const normalizedQuery = normalizeServiceQuery(query);
      const sort = options.sort ?? getDefaultSort(normalizedQuery.mode);
      const sortSeed = sort === "random" ? createSortSeed(sort) : null;
      const limit = options.limit ?? normalizedQuery.limit ?? DEFAULT_QUERY_LIMIT;
      const result = await dependencies.openSearchWindow(
        buildSearchWindowFilters(normalizedQuery, {
          sort,
          sortSeed,
          limit,
        }),
      );
      return createSearchSessionFromWindow(normalizedQuery, result);
    },
    loadMore: (session, options = {}) => {
      if (!session.hasMore || session.nextOffset === null) {
        return Promise.resolve(session);
      }

      const minimumLoadedCount = Math.max(
        session.loadedCount + 1,
        options.minimumLoadedCount ?? session.loadedCount + (session.query.limit ?? DEFAULT_QUERY_LIMIT),
      );
      let nextSession = session;

      while (nextSession.hasMore && nextSession.nextOffset !== null && nextSession.loadedCount < minimumLoadedCount) {
        const result = dependencies.readSearchWindowPage(
          nextSession.windowId,
          nextSession.nextOffset,
          nextSession.query.limit ?? DEFAULT_QUERY_LIMIT,
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
          limit: Math.max(session.query.limit ?? DEFAULT_QUERY_LIMIT, session.loadedCount),
        }),
      );
      return createSearchSessionFromWindow(session.query, result);
    },
  };
}
