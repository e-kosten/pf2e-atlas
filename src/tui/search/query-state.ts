import {
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../../domain/categories.js";
import { createScopedSearchDiscoveryApplicability } from "../../app/search-discovery-service.js";
import { findSearchScopeFilter, type SearchRequest } from "../../domain/search-request-types.js";
import type { MetadataFieldSemantics } from "../../search/filters/semantics.js";
import type {
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataFilterNode,
  MetadataSetField,
} from "../../domain/metadata-filter-types.js";
import type {
  SearchCategory,
  SearchProfile,
  SearchSubcategory,
} from "../../domain/search-types.js";
import {
  extractLegacyQueryPartsFromCanonicalFilter,
  isMetadataQueryPart,
  legacyQueryPartsToCanonicalFilter,
  metadataFilterNodeToRootQueryParts,
  rootMetadataQueryPartsToFilterNode,
  type Pf2eTerminalMetadataQueryPart,
  type Pf2eTerminalQueryPart,
} from "./query-parts.js";
import {
  cloneNumberPolicy,
  cloneStringPolicy,
  createEmptyNumberPolicy,
  createEmptyStringPolicy,
  normalizeQueryFieldPolicy,
} from "./policies.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalSearchMode,
  Pf2eTerminalSearchQuery,
  Pf2eTerminalSearchStructuredPart,
  SearchServiceDependencies,
} from "./service-types.js";

const DEFAULT_QUERY_LIMIT = 50;
const DEFAULT_SEARCH_PROFILE: SearchProfile = "balanced";

function trimOptionalText(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

function getLegacyQueryState(query: Pf2eTerminalSearchQuery): {
  category: SearchCategory | null;
  parts: Pf2eTerminalQueryPart[];
} {
  return extractLegacyQueryPartsFromCanonicalFilter(query.filter);
}

function rebuildQueryFilter(
  query: Pf2eTerminalSearchQuery,
  category: SearchCategory | null,
  parts: readonly Pf2eTerminalQueryPart[],
): Pf2eTerminalSearchQuery {
  const filter = legacyQueryPartsToCanonicalFilter(category, parts);
  return {
    ...query,
    ...(filter ? { filter } : {}),
    ...(!filter ? { filter: undefined } : {}),
  };
}

export function createDefaultQuery(mode: Pf2eTerminalSearchMode = "browse"): Pf2eTerminalSearchQuery {
  switch (mode) {
    case "browse":
      return {
        mode: "browse",
        limit: DEFAULT_QUERY_LIMIT,
      };
    case "lookup":
      return {
        mode: "lookup",
        limit: DEFAULT_QUERY_LIMIT,
        search: {
          query: "",
        },
      };
    case "search":
      return {
        mode: "search",
        limit: DEFAULT_QUERY_LIMIT,
        search: {
          query: "",
          profile: DEFAULT_SEARCH_PROFILE,
        },
      };
  }
}

export function splitMetadataTreeIntoParts(node: MetadataFilterNode | null): Pf2eTerminalSearchStructuredPart[] {
  return metadataFilterNodeToRootQueryParts(node);
}

export function getSearchQueryText(query: Pf2eTerminalSearchQuery): string {
  if (query.mode === "browse") {
    return "";
  }

  return query.search?.query ?? "";
}

export function getSearchQueryExcludeText(query: Pf2eTerminalSearchQuery): string {
  return query.mode === "search" ? query.search?.exclude ?? "" : "";
}

export function getSearchQuerySearchProfile(query: Pf2eTerminalSearchQuery): SearchProfile | null {
  return query.mode === "search" ? (query.search?.profile ?? DEFAULT_SEARCH_PROFILE) : null;
}

export function setSearchQueryText(
  query: Pf2eTerminalSearchQuery,
  text: string,
): Pf2eTerminalSearchQuery {
  if (query.mode === "browse") {
    return query;
  }

  const nextQuery = text;
  return {
    ...query,
    search: {
      ...query.search,
      query: nextQuery,
    },
  };
}

export function setSearchQueryExcludeText(
  query: Pf2eTerminalSearchQuery,
  exclude: string,
): Pf2eTerminalSearchQuery {
  if (query.mode !== "search") {
    return query;
  }

  const nextExclude = trimOptionalText(exclude);
  return {
    ...query,
    search: {
      ...query.search,
      ...(nextExclude ? { exclude: nextExclude } : {}),
      ...(!nextExclude ? { exclude: undefined } : {}),
    },
  };
}

export function setSearchQuerySearchProfile(
  query: Pf2eTerminalSearchQuery,
  profile: SearchProfile,
): Pf2eTerminalSearchQuery {
  if (query.mode !== "search") {
    return query;
  }

  return {
    ...query,
    search: {
      ...query.search,
      profile,
    },
  };
}

export function getSearchQueryCategory(query: Pf2eTerminalSearchQuery): SearchCategory | null {
  return normalizeSearchCategory(findSearchScopeFilter(query.filter)?.category ?? null) ?? null;
}

export function getSearchQueryPart<TKind extends Pf2eTerminalSearchStructuredPart["kind"]>(
  query: Pf2eTerminalSearchQuery,
  kind: TKind,
): Extract<Pf2eTerminalSearchStructuredPart, { kind: TKind }> | null {
  return (
    getLegacyQueryState(query).parts.find(
      (part): part is Extract<Pf2eTerminalSearchStructuredPart, { kind: TKind }> => part.kind === kind,
    ) ?? null
  );
}

export function getSearchQuerySubcategory(query: Pf2eTerminalSearchQuery): SearchSubcategory | null {
  const scope = findSearchScopeFilter(query.filter);
  if (!scope || scope.subcategory.kind !== "eq") {
    return null;
  }

  return normalizeSearchSubcategory(scope.subcategory.value) ?? null;
}

export function getSearchQueryLevelRange(query: Pf2eTerminalSearchQuery): {
  levelMin: number | null;
  levelMax: number | null;
} {
  const part = getSearchQueryPart(query, "levelRange");
  return {
    levelMin: part?.levelMin ?? null,
    levelMax: part?.levelMax ?? null,
  };
}

export function getSearchQueryRarityPolicy(query: Pf2eTerminalSearchQuery): Pf2eTerminalFilterValuePolicy<string> {
  return cloneStringPolicy(getSearchQueryPart(query, "rarityPolicy")?.policy ?? createEmptyStringPolicy());
}

export function getSearchQueryActionCostPolicy(query: Pf2eTerminalSearchQuery): Pf2eTerminalFilterValuePolicy<number> {
  return cloneNumberPolicy(getSearchQueryPart(query, "actionCostPolicy")?.policy ?? createEmptyNumberPolicy());
}

export function getSearchQueryMetadataTree(query: Pf2eTerminalSearchQuery): MetadataFilterNode | null {
  return rootMetadataQueryPartsToFilterNode(getLegacyQueryState(query).parts) ?? null;
}

export function setSearchQueryCategory(
  query: Pf2eTerminalSearchQuery,
  category: SearchCategory | null,
): Pf2eTerminalSearchQuery {
  return rebuildQueryFilter(query, category, []);
}

export function setSearchQueryPart(
  query: Pf2eTerminalSearchQuery,
  part: Exclude<Pf2eTerminalSearchStructuredPart, Pf2eTerminalMetadataQueryPart>,
): Pf2eTerminalSearchQuery {
  const legacyState = getLegacyQueryState(query);
  const nextParts = legacyState.parts.filter(
    (candidate) => candidate.kind !== part.kind && !isMetadataQueryPart(candidate),
  );
  nextParts.push(part);
  nextParts.push(...legacyState.parts.filter(isMetadataQueryPart));
  return rebuildQueryFilter(query, legacyState.category, nextParts);
}

export function removeSearchQueryPart(
  query: Pf2eTerminalSearchQuery,
  kind: Exclude<Pf2eTerminalSearchStructuredPart["kind"], Pf2eTerminalMetadataQueryPart["kind"]>,
): Pf2eTerminalSearchQuery {
  const legacyState = getLegacyQueryState(query);
  return rebuildQueryFilter(
    query,
    legacyState.category,
    legacyState.parts.filter((part) => part.kind !== kind),
  );
}

export function setSearchQueryMetadataTree(
  query: Pf2eTerminalSearchQuery,
  node: MetadataFilterNode | null,
): Pf2eTerminalSearchQuery {
  const legacyState = getLegacyQueryState(query);
  return rebuildQueryFilter(query, legacyState.category, [
    ...legacyState.parts.filter((part) => !isMetadataQueryPart(part)),
    ...splitMetadataTreeIntoParts(node),
  ]);
}

export function isActionCostAvailableInScope(
  dependencies: SearchServiceDependencies,
  category: SearchCategory | null,
  subcategory: SearchSubcategory | null,
): boolean {
  if (!category) {
    return false;
  }

  return dependencies.discovery.isPromotedFieldAvailable(
    "actionCost",
    createScopedSearchDiscoveryApplicability("browse", category, subcategory),
  );
}

export function buildMetadataNodeForQueryFieldSelection(
  field: Pf2eTerminalFacetField,
  policy: Pf2eTerminalFilterValuePolicy<string>,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): MetadataFilterNode | null {
  const normalizedPolicy = normalizeQueryFieldPolicy(field, policy, fieldSemanticsByName);
  if (!normalizedPolicy) {
    return null;
  }

  const fieldSemantics = fieldSemanticsByName.get(field);
  if (!fieldSemantics) {
    return null;
  }

  if (fieldSemantics.fieldType === "set") {
    const clauses: MetadataFilterNode[] = [];
    if (normalizedPolicy.any.length > 0) {
      clauses.push({
        field: field as MetadataSetField,
        op: "includesAny",
        values: normalizedPolicy.any,
      });
    }
    if (normalizedPolicy.all.length > 0) {
      clauses.push({
        field: field as MetadataSetField,
        op: "includesAll",
        values: normalizedPolicy.all,
      });
    }
    if (normalizedPolicy.exclude.length > 0) {
      clauses.push({
        field: field as MetadataSetField,
        op: "excludesAny",
        values: normalizedPolicy.exclude,
      });
    }
    return clauses.length === 0 ? null : clauses.length === 1 ? clauses[0]! : { and: clauses };
  }

  if (fieldSemantics.fieldType === "enumString") {
    const clauses: MetadataFilterNode[] = [];
    if (normalizedPolicy.any.length === 1) {
      clauses.push({
        field: field as MetadataEnumStringField,
        op: "eq",
        value: normalizedPolicy.any[0]!,
      });
    } else if (normalizedPolicy.any.length > 1) {
      clauses.push({
        field: field as MetadataEnumStringField,
        op: "in",
        values: normalizedPolicy.any,
      });
    }
    if (normalizedPolicy.exclude.length > 0) {
      clauses.push({
        field: field as MetadataEnumStringField,
        op: "notIn",
        values: normalizedPolicy.exclude,
      });
    }
    return clauses.length === 0 ? null : clauses.length === 1 ? clauses[0]! : { and: clauses };
  }

  if (fieldSemantics.fieldType === "boolean") {
    const clauses: MetadataFilterNode[] = [];
    for (const value of normalizedPolicy.any) {
      clauses.push({
        field: field as MetadataBooleanField,
        op: "eq",
        value: value === "true",
      });
    }
    for (const value of normalizedPolicy.exclude) {
      clauses.push({
        not: {
          field: field as MetadataBooleanField,
          op: "eq",
          value: value === "true",
        },
      });
    }
    return clauses.length === 0 ? null : clauses.length === 1 ? clauses[0]! : { and: clauses };
  }

  return null;
}

export function normalizeSearchQuery(query: Pf2eTerminalSearchQuery): Pf2eTerminalSearchQuery {
  const limit = query.limit ?? DEFAULT_QUERY_LIMIT;

  switch (query.mode) {
    case "browse":
      return {
        mode: "browse",
        limit,
        offset: query.offset,
        filter: query.filter,
        sort: query.sort,
      };
    case "lookup":
      return {
        mode: "lookup",
        limit,
        offset: query.offset,
        filter: query.filter,
        sort: query.sort,
        search: {
          query: query.search?.query?.trim() ?? "",
        },
      };
    case "search": {
      const exclude = trimOptionalText(query.search?.exclude);
      return {
        mode: "search",
        limit,
        offset: query.offset,
        filter: query.filter,
        explain: query.explain,
        search: {
          query: query.search?.query?.trim() ?? "",
          profile: query.search?.profile ?? DEFAULT_SEARCH_PROFILE,
          ...(exclude ? { exclude } : {}),
        },
      };
    }
  }
}
