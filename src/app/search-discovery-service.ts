import type { Pf2eDataService } from "../data/service.js";
import { inferActorMetricValueType } from "../domain/actor-metrics.js";
import { normalizeSearchCategory, normalizeSearchSubcategory } from "../domain/categories.js";
import { orderFilterValues, type FilterValueOrdering } from "../domain/filter-value-ordering.js";
import { inferItemMetricValueType } from "../domain/item-metrics.js";
import type { MetadataFieldName, MetadataFieldType } from "../domain/metadata-field-types.js";
import {
  buildSearchFilterDiscoveryCatalogRequest,
  createSearchFilterDiscoveryContext,
  type SearchFilterDiscoveryMode,
  type SearchFilterDiscoveryApplicability,
  type SearchFilterDiscoveryContext,
  type SearchFilterDiscoveryOption,
  type SearchFilterDiscoveryRequest,
  type SearchFilterDiscoveryResult,
  type SearchPromotedFieldDomainKey,
} from "../domain/search-field-domains.js";
import type { SearchRequest, SearchRequestMode } from "../domain/search-request-types.js";
import type { FilterValueField, FilterValueQuery, SearchCategory, SearchSubcategory } from "../domain/search-types.js";
import { getMetadataFilterSemantics } from "../search/filters/semantics.js";

type SearchDiscoveryDataService = Pick<Pf2eDataService, "discoverFilterValues" | "getPack" | "listFilterValues">;

export type SearchDiscoveryScope = {
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
};

export type SearchDiscoveryField = {
  field: MetadataFieldName;
  fieldType: MetadataFieldType;
  discoverable: boolean;
  notes?: string;
  subcategories?: readonly SearchSubcategory[];
  valueOrdering?: FilterValueOrdering;
};

export type SearchDiscoveryMetricGroup = {
  metricField: "actorMetrics" | "itemMetrics";
  metadataField: "actorMetric" | "itemMetric";
  namespaces: ReadonlyArray<{ prefix: string; description: string }>;
};

export type SearchSemanticsDiscoveryReader = {
  scope: SearchDiscoveryScope | null;
  mode: SearchFilterDiscoveryMode;
  discoverFieldValues: (options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    field: string;
  }) => readonly SearchFilterDiscoveryOption[];
  discoverMetricKeys: (options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    metricField: "actorMetrics" | "itemMetrics";
    metricPrefix?: string;
  }) => readonly SearchFilterDiscoveryOption[];
  discoverMetricValues: (options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    metricField: "actorMetrics" | "itemMetrics";
    metricKey: string;
  }) => readonly SearchFilterDiscoveryOption[];
};

export type Pf2eApplicationSearchDiscoveryService = {
  discoverFilterValues: (request: SearchFilterDiscoveryRequest) => Promise<SearchFilterDiscoveryResult>;
  discoverCatalogFilterValues: (options: {
    applicability: SearchFilterDiscoveryApplicability;
    target: { field: string };
  }) => SearchFilterDiscoveryResult;
  discoverMetricKeys: (options: {
    applicability: SearchFilterDiscoveryApplicability;
    metricField: "actorMetrics" | "itemMetrics";
    metricPrefix?: string;
  }) => readonly SearchFilterDiscoveryOption[];
  discoverMetricValues: (options: {
    applicability: SearchFilterDiscoveryApplicability;
    metricField: "actorMetrics" | "itemMetrics";
    metricKey: string;
  }) => readonly SearchFilterDiscoveryOption[];
  getMetricDiscoveryGroups: (scope: SearchDiscoveryScope | null) => readonly SearchDiscoveryMetricGroup[];
  getScopedMetadataFields: (scope: SearchDiscoveryScope | null) => readonly SearchDiscoveryField[];
  isPromotedFieldAvailable: (
    field: SearchPromotedFieldDomainKey,
    applicability: SearchFilterDiscoveryApplicability,
  ) => boolean;
  createCatalogSearchSemanticsReader: () => SearchSemanticsDiscoveryReader;
  prepareSearchSemanticsReader: (
    request: Readonly<SearchRequest>,
    mode: SearchFilterDiscoveryMode,
  ) => Promise<SearchSemanticsDiscoveryReader>;
  resolvePackName: (packValue: string) => string | undefined;
};

export function createScopedSearchDiscoveryApplicability(
  mode: SearchRequestMode,
  category: SearchCategory | null,
  subcategory: SearchSubcategory | null,
  options: { pack?: string } = {},
): SearchFilterDiscoveryApplicability {
  return {
    mode,
    ...(options.pack ? { pack: options.pack } : {}),
    scopes: category ? [{ category, ...(subcategory ? { subcategory } : {}) }] : [],
  };
}

function buildDiscoveryCacheKey(parts: ReadonlyArray<string | null | undefined>): string {
  return parts.map((part) => part ?? "").join("|");
}

function buildDiscoveryContextCacheKey(context: SearchFilterDiscoveryContext): string {
  return buildDiscoveryCacheKey([
    context.applicability.mode,
    context.applicability.pack,
    ...context.applicability.scopes.flatMap((scope) => [scope.category, scope.subcategory ?? "all"]),
    JSON.stringify(context.request),
  ]);
}

function resolveContextSearchRequest(
  mode: SearchFilterDiscoveryMode,
  context: SearchFilterDiscoveryContext,
): Readonly<SearchRequest> {
  if (mode === "catalog") {
    return buildSearchFilterDiscoveryCatalogRequest(context.applicability);
  }

  const request = context.request;
  if (
    (request.mode === "search" || request.mode === "lookup") &&
    !request.filter &&
    !request.search.query.trim()
  ) {
    return buildSearchFilterDiscoveryCatalogRequest(context.applicability);
  }

  return request;
}

function buildFilterValueQuery(
  field: FilterValueField,
  applicability: SearchFilterDiscoveryApplicability,
  extras: { metric?: string; metricPrefix?: string } = {},
): FilterValueQuery {
  const query: FilterValueQuery = {
    field,
    ...(extras.metric ? { metric: extras.metric } : {}),
    ...(extras.metricPrefix ? { metricPrefix: extras.metricPrefix } : {}),
  };
  const [firstScope] = applicability.scopes;

  if (applicability.scopes.length === 1 && firstScope) {
    const category = normalizeSearchCategory(firstScope.category);
    if (category) {
      query.category = category;
    }
    if (firstScope.subcategory) {
      const subcategory = normalizeSearchSubcategory(firstScope.subcategory);
      if (subcategory) {
        query.subcategory = subcategory;
      }
    }
    return query;
  }

  if (applicability.scopes.length > 1) {
    query.scopes = applicability.scopes.flatMap((scope) => {
      const category = normalizeSearchCategory(scope.category);
      if (!category) {
        return [];
      }
      const subcategory = scope.subcategory ? normalizeSearchSubcategory(scope.subcategory) : null;
      return [
        {
          category,
          ...(subcategory ? { subcategory } : {}),
        },
      ];
    });
  }

  return query;
}

export function createPf2eApplicationSearchDiscoveryService(
  dataService: SearchDiscoveryDataService,
): Pf2eApplicationSearchDiscoveryService {
  const semantics = getMetadataFilterSemantics();
  const metadataFieldsByName = new Map<MetadataFieldName, SearchDiscoveryField>(
    semantics.metadataFields.map((field) => [
      field.field,
      {
        field: field.field,
        fieldType: field.fieldType,
        discoverable: field.discoverable,
        notes: field.notes,
        subcategories: field.subcategories,
        valueOrdering: field.valueOrdering,
      },
    ]),
  );
  const scopedFieldCache = new Map<string, readonly SearchDiscoveryField[]>();
  const metricGroupCache = new Map<string, readonly SearchDiscoveryMetricGroup[]>();
  const catalogFilterValueCache = new Map<string, SearchFilterDiscoveryResult>();
  const asyncFilterValueCache = new Map<string, Promise<SearchFilterDiscoveryResult>>();
  const asyncDiscoveryOptionCache = new Map<string, Promise<readonly SearchFilterDiscoveryOption[]>>();
  const metricKeyCache = new Map<string, readonly SearchFilterDiscoveryOption[]>();
  const metricValueCache = new Map<string, readonly SearchFilterDiscoveryOption[]>();

  function getFieldValueOrdering(field: string): FilterValueOrdering | undefined {
    return metadataFieldsByName.get(field as MetadataFieldName)?.valueOrdering;
  }

  function getScopedMetadataFields(scope: SearchDiscoveryScope | null): readonly SearchDiscoveryField[] {
    if (!scope) {
      return [];
    }

    const cacheKey = buildDiscoveryCacheKey([scope.category, scope.subcategory ?? "all"]);
    const cached = scopedFieldCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const fields = semantics.metadataFieldsByCategory[scope.category]
      .map((field) => metadataFieldsByName.get(field))
      .filter((field): field is SearchDiscoveryField => Boolean(field))
      .filter(
        (field) => !scope.subcategory || !field.subcategories || field.subcategories.includes(scope.subcategory),
      );
    scopedFieldCache.set(cacheKey, fields);
    return fields;
  }

  function getMetricDiscoveryGroups(scope: SearchDiscoveryScope | null): readonly SearchDiscoveryMetricGroup[] {
    if (!scope) {
      return [];
    }

    const cacheKey = buildDiscoveryCacheKey([scope.category, scope.subcategory ?? "all"]);
    const cached = metricGroupCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const groups: SearchDiscoveryMetricGroup[] = [];
    if (semantics.actorMetricDiscovery?.categories.includes(scope.category)) {
      groups.push({
        metricField: "actorMetrics",
        metadataField: "actorMetric",
        namespaces: semantics.actorMetricDiscovery.namespaces,
      });
    }
    if (semantics.itemMetricDiscovery?.categories.includes(scope.category)) {
      groups.push({
        metricField: "itemMetrics",
        metadataField: "itemMetric",
        namespaces: semantics.itemMetricDiscovery.namespaces,
      });
    }

    metricGroupCache.set(cacheKey, groups);
    return groups;
  }

  function discoverCatalogFilterValues(options: {
    applicability: SearchFilterDiscoveryApplicability;
    target: { field: string };
  }): SearchFilterDiscoveryResult {
    const cacheKey = buildDiscoveryCacheKey([
      options.applicability.mode,
      options.applicability.pack,
      ...options.applicability.scopes.flatMap((scope) => [scope.category, scope.subcategory ?? "all"]),
      options.target.field,
    ]);
    const cached = catalogFilterValueCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const query = buildFilterValueQuery(options.target.field as FilterValueField, options.applicability);
    const values = orderFilterValues(
      dataService.listFilterValues(query).values,
      getFieldValueOrdering(options.target.field),
    );
    const result: SearchFilterDiscoveryResult = {
      mode: "catalog",
      target: options.target,
      options: values.map((entry) => ({
        id: entry.value,
        value: entry.value,
        count: entry.count,
      })),
    };
    catalogFilterValueCache.set(cacheKey, result);
    return result;
  }

  async function discoverFieldOptions(options: {
    mode: SearchFilterDiscoveryMode;
    context: SearchFilterDiscoveryContext;
    field: FilterValueField;
    metric?: string;
    metricPrefix?: string;
  }): Promise<readonly SearchFilterDiscoveryOption[]> {
    const cacheKey = buildDiscoveryCacheKey([
      options.mode,
      buildDiscoveryContextCacheKey(options.context),
      options.field,
      options.metric ?? "",
      options.metricPrefix ?? "",
    ]);
    const cached = asyncDiscoveryOptionCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const promise = dataService
      .discoverFilterValues(
        buildFilterValueQuery(options.field, options.context.applicability, {
          metric: options.metric,
          metricPrefix: options.metricPrefix,
        }),
        resolveContextSearchRequest(options.mode, options.context),
      )
      .then((result) =>
        orderFilterValues(result.values, getFieldValueOrdering(options.field)).map((entry) => ({
          id: entry.value,
          value: entry.value,
          count: entry.count,
        })),
      );
    asyncDiscoveryOptionCache.set(cacheKey, promise);
    void promise.catch(() => {
      if (asyncDiscoveryOptionCache.get(cacheKey) === promise) {
        asyncDiscoveryOptionCache.delete(cacheKey);
      }
    });
    return promise;
  }

  async function discoverFilterValues(request: SearchFilterDiscoveryRequest): Promise<SearchFilterDiscoveryResult> {
    const cacheKey = buildDiscoveryCacheKey([
      request.mode,
      buildDiscoveryContextCacheKey(request.context),
      request.target.field,
    ]);
    const cached = asyncFilterValueCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const promise = discoverFieldOptions({
      mode: request.mode,
      context: request.context,
      field: request.target.field as FilterValueField,
    }).then(
      (options): SearchFilterDiscoveryResult => ({
        mode: request.mode,
        target: request.target,
        options: [...options],
      }),
    );
    asyncFilterValueCache.set(cacheKey, promise);
    void promise.catch(() => {
      if (asyncFilterValueCache.get(cacheKey) === promise) {
        asyncFilterValueCache.delete(cacheKey);
      }
    });
    return promise;
  }

  function discoverMetricKeys(options: {
    applicability: SearchFilterDiscoveryApplicability;
    metricField: "actorMetrics" | "itemMetrics";
    metricPrefix?: string;
  }): readonly SearchFilterDiscoveryOption[] {
    const cacheKey = buildDiscoveryCacheKey([
      options.applicability.mode,
      options.applicability.pack,
      ...options.applicability.scopes.flatMap((scope) => [scope.category, scope.subcategory ?? "all"]),
      options.metricField,
      options.metricPrefix ?? "",
    ]);
    const cached = metricKeyCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const query = buildFilterValueQuery(options.metricField, options.applicability, {
      metricPrefix: options.metricPrefix,
    });
    const result = dataService.listFilterValues(query).values.map((entry) => ({
      id: entry.value,
      value: entry.value,
      count: entry.count,
    }));
    metricKeyCache.set(cacheKey, result);
    return result;
  }

  function discoverMetricValues(options: {
    applicability: SearchFilterDiscoveryApplicability;
    metricField: "actorMetrics" | "itemMetrics";
    metricKey: string;
  }): readonly SearchFilterDiscoveryOption[] {
    const cacheKey = buildDiscoveryCacheKey([
      options.applicability.mode,
      options.applicability.pack,
      ...options.applicability.scopes.flatMap((scope) => [scope.category, scope.subcategory ?? "all"]),
      options.metricField,
      options.metricKey,
    ]);
    const cached = metricValueCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const query = buildFilterValueQuery(options.metricField, options.applicability, {
      metric: options.metricKey,
    });
    const result = dataService.listFilterValues(query).values.map((entry) => ({
      id: entry.value,
      value: entry.value,
      count: entry.count,
    }));
    metricValueCache.set(cacheKey, result);
    return result;
  }

  function createCatalogSearchSemanticsReader(): SearchSemanticsDiscoveryReader {
    return {
      scope: null,
      mode: "catalog",
      discoverFieldValues: ({ category, subcategory, field }) =>
        discoverCatalogFilterValues({
          applicability: createScopedSearchDiscoveryApplicability("browse", category, subcategory),
          target: { field },
        }).options,
      discoverMetricKeys: ({ category, subcategory, metricField, metricPrefix }) =>
        discoverMetricKeys({
          applicability: createScopedSearchDiscoveryApplicability("browse", category, subcategory),
          metricField,
          metricPrefix,
        }),
      discoverMetricValues: ({ category, subcategory, metricField, metricKey }) =>
        discoverMetricValues({
          applicability: createScopedSearchDiscoveryApplicability("browse", category, subcategory),
          metricField,
          metricKey,
        }),
    };
  }

  async function prepareSearchSemanticsReader(
    request: Readonly<SearchRequest>,
    mode: SearchFilterDiscoveryMode,
  ): Promise<SearchSemanticsDiscoveryReader> {
    const context = createSearchFilterDiscoveryContext(request);
    const [scopeEntry] = context.applicability.scopes;
    const category = scopeEntry ? normalizeSearchCategory(scopeEntry.category) : null;
    const subcategory =
      scopeEntry?.subcategory === undefined || scopeEntry.subcategory === null
        ? null
        : normalizeSearchSubcategory(scopeEntry.subcategory);
    const scope =
      scopeEntry && category && (scopeEntry.subcategory === undefined || scopeEntry.subcategory === null || subcategory)
        ? ({ category, subcategory } satisfies SearchDiscoveryScope)
        : null;
    if (!scope) {
      return {
        scope: null,
        mode,
        discoverFieldValues: () => [],
        discoverMetricKeys: () => [],
        discoverMetricValues: () => [],
      };
    }
    const preparedScope = scope;

    const fieldValuesByField = new Map<string, readonly SearchFilterDiscoveryOption[]>();
    const metricKeysByNamespace = new Map<string, readonly SearchFilterDiscoveryOption[]>();
    const metricValuesByKey = new Map<string, readonly SearchFilterDiscoveryOption[]>();
    const scopedFields = getScopedMetadataFields(scope).filter((field) => field.discoverable);
    const metricGroups = getMetricDiscoveryGroups(scope);

    const fieldEntries = await Promise.all(
      scopedFields.map(async (field) => [
        field.field,
        await discoverFieldOptions({
          mode,
          context,
          field: field.field as FilterValueField,
        }),
      ] as const),
    );
    fieldEntries.forEach(([field, options]) => {
      fieldValuesByField.set(field, options);
    });

    const metricKeyEntries = await Promise.all(
      metricGroups.flatMap((group) =>
        group.namespaces.map(async (namespace) => [
          `${group.metricField}|${namespace.prefix}`,
          await discoverFieldOptions({
            mode,
            context,
            field: group.metricField,
            metricPrefix: namespace.prefix,
          }),
        ] as const),
      ),
    );
    metricKeyEntries.forEach(([cacheKey, options]) => {
      metricKeysByNamespace.set(cacheKey, options);
    });

    const metricValueEntries = await Promise.all(
      metricKeyEntries.flatMap(([namespaceKey, options]) => {
        const [metricField] = namespaceKey.split("|") as ["actorMetrics" | "itemMetrics", string];
        return options
          .filter((option) => {
            const metricKey = String(option.value);
            const valueType =
              metricField === "actorMetrics"
                ? inferActorMetricValueType(metricKey)
                : inferItemMetricValueType(metricKey);
            return valueType === "text" || valueType === "boolean";
          })
          .map(async (option) => {
            const metricKey = String(option.value);
            return [
              `${metricField}|${metricKey}`,
              await discoverFieldOptions({
                mode,
                context,
                field: metricField,
                metric: metricKey,
              }),
            ] as const;
          });
      }),
    );
    metricValueEntries.forEach(([cacheKey, options]) => {
      metricValuesByKey.set(cacheKey, options);
    });

    function matchesScope(category: SearchCategory, subcategory: SearchSubcategory | null): boolean {
      return category === preparedScope.category && (subcategory ?? null) === preparedScope.subcategory;
    }

    return {
      scope: preparedScope,
      mode,
      discoverFieldValues: ({ category, subcategory, field }) =>
        matchesScope(category, subcategory) ? (fieldValuesByField.get(field) ?? []) : [],
      discoverMetricKeys: ({ category, subcategory, metricField, metricPrefix }) =>
        matchesScope(category, subcategory) ? (metricKeysByNamespace.get(`${metricField}|${metricPrefix ?? ""}`) ?? []) : [],
      discoverMetricValues: ({ category, subcategory, metricField, metricKey }) =>
        matchesScope(category, subcategory) ? (metricValuesByKey.get(`${metricField}|${metricKey}`) ?? []) : [],
    };
  }

  return {
    discoverFilterValues,
    discoverCatalogFilterValues,
    discoverMetricKeys,
    discoverMetricValues,
    getMetricDiscoveryGroups,
    getScopedMetadataFields,
    isPromotedFieldAvailable: (field, applicability) =>
      discoverCatalogFilterValues({
        applicability,
        target: { field },
      }).options.length > 0,
    createCatalogSearchSemanticsReader,
    prepareSearchSemanticsReader,
    resolvePackName: (packValue) => dataService.getPack(packValue)?.name,
  };
}
