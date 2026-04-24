import {
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../../domain/categories.js";
import { createScopedSearchDiscoveryApplicability } from "../../app/search-discovery-service.js";
import {
  buildAllOfFilter,
  buildAnyOfFilter,
  findSearchScopeFilter,
  type SearchFilterNode,
  type SearchRequest,
} from "../../domain/search-request-types.js";
import type { MetadataFieldSemantics } from "../../search/filters/semantics.js";
import type {
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataFilterNode,
  MetadataSetField,
} from "./metadata-filter-draft.js";
import type {
  SearchCategory,
  SearchProfile,
  SearchSubcategory,
} from "../../domain/search-types.js";
import {
  canonicalFilterToMetadataNode,
  metadataFilterNodeToCanonicalFilter,
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
  SearchServiceDependencies,
} from "./service-types.js";

const DEFAULT_QUERY_LIMIT = 50;
const DEFAULT_SEARCH_PROFILE: SearchProfile = "balanced";

function trimOptionalText(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

type QueryRootOperator = "allOf" | "anyOf";

function hasPolicyValues<T extends number | string>(policy: Pf2eTerminalFilterValuePolicy<T>): boolean {
  return policy.any.length > 0 || policy.all.length > 0 || policy.exclude.length > 0;
}

function isCanonicalMetadataFilterCandidate(filter: SearchFilterNode): boolean {
  return (
    filter.kind === "metadataPredicate" ||
    filter.kind === "metric" ||
    filter.kind === "metricCompare" ||
    filter.kind === "anyOf" ||
    filter.kind === "allOf" ||
    filter.kind === "not"
  );
}

function buildPolicyLeaf(
  kind: "rarity" | "actionCost",
  value: string | number,
): SearchFilterNode {
  return kind === "rarity"
    ? { kind, match: { kind: "eq", value: value as string } }
    : { kind, match: { kind: "eq", value: value as number } };
}

function buildPolicyFilter(
  kind: "rarity" | "actionCost",
  policy: Pf2eTerminalFilterValuePolicy<string> | Pf2eTerminalFilterValuePolicy<number>,
): SearchFilterNode | null {
  const anyChildren = policy.any.map((value) => buildPolicyLeaf(kind, value));
  const allChildren = policy.all.map((value) => buildPolicyLeaf(kind, value));
  const excludeChildren = policy.exclude.map((value) => buildPolicyLeaf(kind, value));
  const children: SearchFilterNode[] = [];

  if (anyChildren.length === 1) {
    children.push(anyChildren[0]!);
  } else if (anyChildren.length > 1) {
    children.push({ kind: "anyOf", children: anyChildren });
  }

  if (allChildren.length === 1) {
    children.push(allChildren[0]!);
  } else if (allChildren.length > 1) {
    children.push({ kind: "allOf", children: allChildren });
  }

  if (excludeChildren.length === 1) {
    children.push({ kind: "not", child: excludeChildren[0]! });
  } else if (excludeChildren.length > 1) {
    children.push({ kind: "not", child: { kind: "anyOf", children: excludeChildren } });
  }

  if (children.length === 0) {
    return null;
  }

  return children.length === 1 ? children[0]! : { kind: "allOf", children };
}

export function buildSearchFilterValuePolicyNode(
  kind: "rarity" | "actionCost",
  policy: Pf2eTerminalFilterValuePolicy<string> | Pf2eTerminalFilterValuePolicy<number>,
): SearchFilterNode | null {
  return buildPolicyFilter(kind, policy);
}

function collectPolicyEqValues<K extends "rarity" | "actionCost">(
  filter: SearchFilterNode,
  kind: K,
): Array<K extends "rarity" ? string : number> | null {
  if (filter.kind === kind && filter.match.kind === "eq") {
    return [filter.match.value] as Array<K extends "rarity" ? string : number>;
  }

  if (filter.kind === "anyOf" || filter.kind === "allOf") {
    const values: Array<K extends "rarity" ? string : number> = [];
    for (const child of filter.children) {
      const childValues = collectPolicyEqValues(child, kind);
      if (!childValues) {
        return null;
      }
      values.push(...childValues);
    }
    return values;
  }

  return null;
}

function extractPolicyFilter<K extends "rarity" | "actionCost">(
  filter: SearchFilterNode,
  kind: K,
): Pf2eTerminalFilterValuePolicy<K extends "rarity" ? string : number> | null {
  if (filter.kind === kind && filter.match.kind === "eq") {
    return {
      any: [filter.match.value] as Array<K extends "rarity" ? string : number>,
      all: [],
      exclude: [],
    };
  }

  if (filter.kind === "anyOf") {
    const values = collectPolicyEqValues(filter, kind);
    return values
      ? {
          any: values,
          all: [],
          exclude: [],
        }
      : null;
  }

  if (filter.kind === "allOf") {
    const policy: Pf2eTerminalFilterValuePolicy<K extends "rarity" ? string : number> = {
      any: [],
      all: [],
      exclude: [],
    };

    for (const child of filter.children) {
      if (child.kind === "not") {
        const excludedValues = collectPolicyEqValues(child.child, kind);
        if (!excludedValues) {
          return null;
        }
        policy.exclude.push(...excludedValues);
        continue;
      }

      const values = collectPolicyEqValues(child, kind);
      if (!values) {
        return null;
      }

      if (child.kind === "allOf") {
        policy.all.push(...values);
      } else {
        policy.any.push(...values);
      }
    }

    return policy;
  }

  if (filter.kind === "not") {
    const excludedValues = collectPolicyEqValues(filter.child, kind);
    return excludedValues
      ? {
          any: [],
          all: [],
          exclude: excludedValues,
        }
      : null;
  }

  return null;
}

export function tryExtractSearchFilterValuePolicy(
  filter: SearchFilterNode,
):
  | { kind: "rarity"; policy: Pf2eTerminalFilterValuePolicy<string> }
  | { kind: "actionCost"; policy: Pf2eTerminalFilterValuePolicy<number> }
  | null {
  const rarityPolicy = extractPolicyFilter(filter, "rarity");
  if (rarityPolicy && hasPolicyValues(rarityPolicy)) {
    return {
      kind: "rarity",
      policy: cloneStringPolicy(rarityPolicy as Pf2eTerminalFilterValuePolicy<string>),
    };
  }

  const actionCostPolicy = extractPolicyFilter(filter, "actionCost");
  if (actionCostPolicy && hasPolicyValues(actionCostPolicy)) {
    return {
      kind: "actionCost",
      policy: cloneNumberPolicy(actionCostPolicy as Pf2eTerminalFilterValuePolicy<number>),
    };
  }

  return null;
}

function getQueryRootOperator(filter: SearchFilterNode | undefined): QueryRootOperator {
  return filter?.kind === "anyOf" ? "anyOf" : "allOf";
}

function getTopLevelQueryChildren(filter: SearchFilterNode | undefined): SearchFilterNode[] {
  if (!filter) {
    return [];
  }

  if (filter.kind === "allOf" || filter.kind === "anyOf") {
    return [...filter.children];
  }

  return [filter];
}

function buildQueryFromTopLevelChildren(
  query: Pf2eTerminalSearchQuery,
  rootOperator: QueryRootOperator,
  children: SearchFilterNode[],
): Pf2eTerminalSearchQuery {
  const filter = rootOperator === "anyOf" ? buildAnyOfFilter(children) : buildAllOfFilter(children);
  return {
    ...query,
    ...(filter ? { filter } : { filter: undefined }),
  };
}

function removeFirstMatchingTopLevelChild(
  children: SearchFilterNode[],
  predicate: (child: SearchFilterNode) => boolean,
): SearchFilterNode[] {
  let removed = false;
  return children.filter((child) => {
    if (!removed && predicate(child)) {
      removed = true;
      return false;
    }
    return true;
  });
}

function removeAllMatchingTopLevelChildren(
  children: SearchFilterNode[],
  predicate: (child: SearchFilterNode) => boolean,
): SearchFilterNode[] {
  return children.filter((child) => !predicate(child));
}

function insertAfterCanonicalPrefix(
  children: SearchFilterNode[],
  node: SearchFilterNode,
  predicate: (child: SearchFilterNode) => boolean,
): SearchFilterNode[] {
  let index = 0;
  while (index < children.length && predicate(children[index]!)) {
    index += 1;
  }
  return [...children.slice(0, index), node, ...children.slice(index)];
}

function isTopLevelPolicyChild(
  child: SearchFilterNode,
  kind: "rarity" | "actionCost",
): boolean {
  const policy = extractPolicyFilter(child, kind);
  return Boolean(policy && hasPolicyValues(policy));
}

function isTopLevelMetadataChild(child: SearchFilterNode): boolean {
  return isCanonicalMetadataFilterCandidate(child) && Boolean(canonicalFilterToMetadataNode(child));
}

function buildLevelFilter(levelRange: {
  levelMin: number | null;
  levelMax: number | null;
}): Extract<SearchFilterNode, { kind: "level" }> | null {
  const { levelMin, levelMax } = levelRange;
  if (levelMin === null && levelMax === null) {
    return null;
  }

  if (levelMin !== null && levelMax !== null) {
    return levelMin === levelMax
      ? { kind: "level", match: { kind: "eq", value: levelMin } }
      : {
          kind: "level",
          match: {
            kind: "between",
            min: Math.min(levelMin, levelMax),
            max: Math.max(levelMin, levelMax),
          },
        };
  }

  return levelMin !== null
    ? { kind: "level", match: { kind: "gte", value: levelMin } }
    : { kind: "level", match: { kind: "lte", value: levelMax! } };
}

function findTopLevelLevelFilter(
  filter: SearchFilterNode | undefined,
): Extract<SearchFilterNode, { kind: "level" }> | null {
  for (const child of getTopLevelQueryChildren(filter)) {
    if (child.kind === "level") {
      return child;
    }
  }

  return null;
}

function findTopLevelPolicyFilter<K extends "rarity" | "actionCost">(
  filter: SearchFilterNode | undefined,
  kind: K,
): Pf2eTerminalFilterValuePolicy<K extends "rarity" ? string : number> | null {
  for (const child of getTopLevelQueryChildren(filter)) {
    const policy = extractPolicyFilter(child, kind);
    if (policy && hasPolicyValues(policy)) {
      return policy;
    }
  }

  return null;
}

function extractQueryMetadataTree(
  filter: SearchFilterNode | undefined,
): MetadataFilterNode | null {
  const rootOperator = getQueryRootOperator(filter);
  const metadataChildren = getTopLevelQueryChildren(filter).filter((child) => isTopLevelMetadataChild(child));

  if (metadataChildren.length === 0) {
    return null;
  }

  return canonicalFilterToMetadataNode(
    metadataChildren.length === 1
      ? metadataChildren[0]!
      : { kind: rootOperator, children: metadataChildren },
  );
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
  const part = findTopLevelLevelFilter(query.filter);
  return {
    levelMin:
      part?.match.kind === "eq"
        ? part.match.value
        : part?.match.kind === "gte"
          ? part.match.value
          : part?.match.kind === "between"
            ? part.match.min
            : null,
    levelMax:
      part?.match.kind === "eq"
        ? part.match.value
        : part?.match.kind === "lte"
          ? part.match.value
          : part?.match.kind === "between"
            ? part.match.max
            : null,
  };
}

export function getSearchQueryRarityPolicy(query: Pf2eTerminalSearchQuery): Pf2eTerminalFilterValuePolicy<string> {
  return cloneStringPolicy(findTopLevelPolicyFilter(query.filter, "rarity") ?? createEmptyStringPolicy());
}

export function getSearchQueryActionCostPolicy(query: Pf2eTerminalSearchQuery): Pf2eTerminalFilterValuePolicy<number> {
  return cloneNumberPolicy(findTopLevelPolicyFilter(query.filter, "actionCost") ?? createEmptyNumberPolicy());
}

export function getSearchQueryMetadataTree(query: Pf2eTerminalSearchQuery): MetadataFilterNode | null {
  return extractQueryMetadataTree(query.filter);
}

export function getSearchQueryRootOperator(query: Pf2eTerminalSearchQuery): QueryRootOperator {
  return getQueryRootOperator(query.filter);
}

export function setSearchQueryCategory(
  query: Pf2eTerminalSearchQuery,
  category: SearchCategory | null,
): Pf2eTerminalSearchQuery {
  const rootOperator = getQueryRootOperator(query.filter);
  let children = removeFirstMatchingTopLevelChild(getTopLevelQueryChildren(query.filter), (child) => child.kind === "scope");
  if (category) {
    children = [
      {
        kind: "scope",
        category,
        subcategory: { kind: "any" },
      },
      ...children,
    ];
  }
  return buildQueryFromTopLevelChildren(query, rootOperator, children);
}

export function setSearchQuerySubcategory(
  query: Pf2eTerminalSearchQuery,
  subcategory: SearchSubcategory | null,
): Pf2eTerminalSearchQuery {
  const currentScope = findSearchScopeFilter(query.filter);
  const category = normalizeSearchCategory(currentScope?.category ?? null) ?? null;
  if (!category) {
    return query;
  }

  const rootOperator = getQueryRootOperator(query.filter);
  const children = removeFirstMatchingTopLevelChild(getTopLevelQueryChildren(query.filter), (child) => child.kind === "scope");
  return buildQueryFromTopLevelChildren(query, rootOperator, [
    {
      kind: "scope",
      category,
      subcategory: subcategory ? { kind: "eq", value: subcategory } : { kind: "any" },
    },
    ...children,
  ]);
}

export function setSearchQueryLevelRange(
  query: Pf2eTerminalSearchQuery,
  levelRange: {
    levelMin: number | null;
    levelMax: number | null;
  },
): Pf2eTerminalSearchQuery {
  const rootOperator = getQueryRootOperator(query.filter);
  let children = removeFirstMatchingTopLevelChild(getTopLevelQueryChildren(query.filter), (child) => child.kind === "level");
  const levelFilter = buildLevelFilter(levelRange);
  if (levelFilter) {
    children = insertAfterCanonicalPrefix(children, levelFilter, (child) => child.kind === "scope");
  }
  return buildQueryFromTopLevelChildren(query, rootOperator, children);
}

export function setSearchQueryRarityPolicy(
  query: Pf2eTerminalSearchQuery,
  policy: Pf2eTerminalFilterValuePolicy<string>,
): Pf2eTerminalSearchQuery {
  const rootOperator = getQueryRootOperator(query.filter);
  let children = removeFirstMatchingTopLevelChild(
    getTopLevelQueryChildren(query.filter),
    (child) => isTopLevelPolicyChild(child, "rarity"),
  );
  const rarityFilter = buildPolicyFilter("rarity", cloneStringPolicy(policy));
  if (rarityFilter) {
    children = insertAfterCanonicalPrefix(children, rarityFilter, (child) => child.kind === "scope" || child.kind === "level");
  }
  return buildQueryFromTopLevelChildren(query, rootOperator, children);
}

export function setSearchQueryActionCostPolicy(
  query: Pf2eTerminalSearchQuery,
  policy: Pf2eTerminalFilterValuePolicy<number>,
): Pf2eTerminalSearchQuery {
  const rootOperator = getQueryRootOperator(query.filter);
  let children = removeFirstMatchingTopLevelChild(
    getTopLevelQueryChildren(query.filter),
    (child) => isTopLevelPolicyChild(child, "actionCost"),
  );
  const actionCostFilter = buildPolicyFilter("actionCost", cloneNumberPolicy(policy));
  if (actionCostFilter) {
    children = insertAfterCanonicalPrefix(
      children,
      actionCostFilter,
      (child) => child.kind === "scope" || child.kind === "level" || isTopLevelPolicyChild(child, "rarity"),
    );
  }
  return buildQueryFromTopLevelChildren(query, rootOperator, children);
}

export function setSearchQueryMetadataTree(
  query: Pf2eTerminalSearchQuery,
  node: MetadataFilterNode | null,
): Pf2eTerminalSearchQuery {
  const rootOperator = getQueryRootOperator(query.filter);
  let children = removeAllMatchingTopLevelChildren(getTopLevelQueryChildren(query.filter), isTopLevelMetadataChild);
  const metadataFilter = metadataFilterNodeToCanonicalFilter(node);
  if (metadataFilter) {
    children = insertAfterCanonicalPrefix(
      children,
      metadataFilter,
      (child) =>
        child.kind === "scope" ||
        child.kind === "level" ||
        isTopLevelPolicyChild(child, "rarity") ||
        isTopLevelPolicyChild(child, "actionCost"),
    );
  }
  return buildQueryFromTopLevelChildren(query, rootOperator, children);
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
