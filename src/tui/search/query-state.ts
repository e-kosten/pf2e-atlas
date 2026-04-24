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
} from "../../domain/metadata-filter-types.js";
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

type QueryFilterState = {
  rootOperator: QueryRootOperator;
  scope: Extract<SearchFilterNode, { kind: "scope" }> | null;
  level: Extract<SearchFilterNode, { kind: "level" }> | null;
  rarityPolicy: Pf2eTerminalFilterValuePolicy<string>;
  actionCostPolicy: Pf2eTerminalFilterValuePolicy<number>;
  otherChildren: SearchFilterNode[];
};

function createEmptyFilterState(rootOperator: QueryRootOperator = "allOf"): QueryFilterState {
  return {
    rootOperator,
    scope: null,
    level: null,
    rarityPolicy: createEmptyStringPolicy(),
    actionCostPolicy: createEmptyNumberPolicy(),
    otherChildren: [],
  };
}

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

function decomposeQueryFilter(filter: SearchFilterNode | undefined): QueryFilterState {
  const rootOperator: QueryRootOperator = filter?.kind === "anyOf" ? "anyOf" : "allOf";
  const topLevelChildren =
    filter?.kind === "allOf" || filter?.kind === "anyOf" ? filter.children : filter ? [filter] : [];
  const state = createEmptyFilterState(rootOperator);

  for (const child of topLevelChildren) {
    if (child.kind === "scope" && !state.scope) {
      state.scope = child;
      continue;
    }

    if (child.kind === "level" && !state.level) {
      state.level = child;
      continue;
    }

    if (!hasPolicyValues(state.rarityPolicy)) {
      const rarityPolicy = extractPolicyFilter(child, "rarity");
      if (rarityPolicy && hasPolicyValues(rarityPolicy)) {
        state.rarityPolicy = cloneStringPolicy(rarityPolicy as Pf2eTerminalFilterValuePolicy<string>);
        continue;
      }
    }

    if (!hasPolicyValues(state.actionCostPolicy)) {
      const actionCostPolicy = extractPolicyFilter(child, "actionCost");
      if (actionCostPolicy && hasPolicyValues(actionCostPolicy)) {
        state.actionCostPolicy = cloneNumberPolicy(actionCostPolicy as Pf2eTerminalFilterValuePolicy<number>);
        continue;
      }
    }

    if (isCanonicalMetadataFilterCandidate(child) && canonicalFilterToMetadataNode(child)) {
      continue;
    }

    state.otherChildren.push(child);
  }

  return state;
}

function rebuildQueryWithFilterState(
  query: Pf2eTerminalSearchQuery,
  state: QueryFilterState,
  metadataTree: MetadataFilterNode | null,
): Pf2eTerminalSearchQuery {
  const children: SearchFilterNode[] = [];

  if (state.scope) {
    children.push(state.scope);
  }
  if (state.level) {
    children.push(state.level);
  }

  const rarityFilter = buildPolicyFilter("rarity", state.rarityPolicy);
  if (rarityFilter) {
    children.push(rarityFilter);
  }

  const actionCostFilter = buildPolicyFilter("actionCost", state.actionCostPolicy);
  if (actionCostFilter) {
    children.push(actionCostFilter);
  }

  const metadataFilter = metadataFilterNodeToCanonicalFilter(metadataTree);
  if (metadataFilter) {
    children.push(metadataFilter);
  }

  children.push(...state.otherChildren);

  const filter =
    state.rootOperator === "anyOf"
      ? buildAnyOfFilter(children)
      : buildAllOfFilter(children);
  return {
    ...query,
    ...(filter ? { filter } : { filter: undefined }),
  };
}

function extractQueryMetadataTree(
  filter: SearchFilterNode | undefined,
): MetadataFilterNode | null {
  const rootOperator: QueryRootOperator = filter?.kind === "anyOf" ? "anyOf" : "allOf";
  const topLevelChildren =
    filter?.kind === "allOf" || filter?.kind === "anyOf" ? filter.children : filter ? [filter] : [];
  const metadataChildren = topLevelChildren.filter(
    (child) => isCanonicalMetadataFilterCandidate(child) && canonicalFilterToMetadataNode(child),
  );

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
  const part = decomposeQueryFilter(query.filter).level;
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
  return cloneStringPolicy(decomposeQueryFilter(query.filter).rarityPolicy);
}

export function getSearchQueryActionCostPolicy(query: Pf2eTerminalSearchQuery): Pf2eTerminalFilterValuePolicy<number> {
  return cloneNumberPolicy(decomposeQueryFilter(query.filter).actionCostPolicy);
}

export function getSearchQueryMetadataTree(query: Pf2eTerminalSearchQuery): MetadataFilterNode | null {
  return extractQueryMetadataTree(query.filter);
}

export function getSearchQueryRootOperator(query: Pf2eTerminalSearchQuery): QueryRootOperator {
  return decomposeQueryFilter(query.filter).rootOperator;
}

export function setSearchQueryCategory(
  query: Pf2eTerminalSearchQuery,
  category: SearchCategory | null,
): Pf2eTerminalSearchQuery {
  const state = decomposeQueryFilter(query.filter);
  state.scope = category
    ? {
        kind: "scope",
        category,
        subcategory: { kind: "any" },
      }
    : null;
  return rebuildQueryWithFilterState(query, state, extractQueryMetadataTree(query.filter));
}

export function setSearchQuerySubcategory(
  query: Pf2eTerminalSearchQuery,
  subcategory: SearchSubcategory | null,
): Pf2eTerminalSearchQuery {
  const state = decomposeQueryFilter(query.filter);
  const category = normalizeSearchCategory(state.scope?.category ?? null) ?? null;
  if (!category) {
    return query;
  }

  state.scope = {
    kind: "scope",
    category,
    subcategory: subcategory ? { kind: "eq", value: subcategory } : { kind: "any" },
  };
  return rebuildQueryWithFilterState(query, state, extractQueryMetadataTree(query.filter));
}

export function setSearchQueryLevelRange(
  query: Pf2eTerminalSearchQuery,
  levelRange: {
    levelMin: number | null;
    levelMax: number | null;
  },
): Pf2eTerminalSearchQuery {
  const state = decomposeQueryFilter(query.filter);
  const { levelMin, levelMax } = levelRange;
  state.level =
    levelMin === null && levelMax === null
      ? null
      : levelMin !== null && levelMax !== null
        ? levelMin === levelMax
          ? { kind: "level", match: { kind: "eq", value: levelMin } }
          : {
              kind: "level",
              match: {
                kind: "between",
                min: Math.min(levelMin, levelMax),
                max: Math.max(levelMin, levelMax),
              },
            }
        : levelMin !== null
          ? { kind: "level", match: { kind: "gte", value: levelMin } }
          : { kind: "level", match: { kind: "lte", value: levelMax! } };
  return rebuildQueryWithFilterState(query, state, extractQueryMetadataTree(query.filter));
}

export function setSearchQueryRarityPolicy(
  query: Pf2eTerminalSearchQuery,
  policy: Pf2eTerminalFilterValuePolicy<string>,
): Pf2eTerminalSearchQuery {
  const state = decomposeQueryFilter(query.filter);
  state.rarityPolicy = cloneStringPolicy(policy);
  return rebuildQueryWithFilterState(query, state, extractQueryMetadataTree(query.filter));
}

export function setSearchQueryActionCostPolicy(
  query: Pf2eTerminalSearchQuery,
  policy: Pf2eTerminalFilterValuePolicy<number>,
): Pf2eTerminalSearchQuery {
  const state = decomposeQueryFilter(query.filter);
  state.actionCostPolicy = cloneNumberPolicy(policy);
  return rebuildQueryWithFilterState(query, state, extractQueryMetadataTree(query.filter));
}

export function setSearchQueryMetadataTree(
  query: Pf2eTerminalSearchQuery,
  node: MetadataFilterNode | null,
): Pf2eTerminalSearchQuery {
  const state = decomposeQueryFilter(query.filter);
  return rebuildQueryWithFilterState(query, state, node);
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
