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
  type SearchNumericMatch,
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
  cloneNumberSelection,
  cloneStringSelection,
  createEmptyNumberSelection,
  createEmptyStringSelection,
  normalizeQueryFieldSelection,
} from "./selections.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalSearchMode,
  Pf2eTerminalSearchQuery,
  Pf2eTerminalValueSelection,
  SearchServiceDependencies,
} from "./service-types.js";

const DEFAULT_QUERY_LIMIT = 50;
const DEFAULT_SEARCH_PROFILE: SearchProfile = "balanced";

function trimOptionalText(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

type QueryRootOperator = "allOf" | "anyOf";

function hasSelectionValues<T extends number | string>(selection: Pf2eTerminalValueSelection<T>): boolean {
  return selection.include.length > 0 || selection.exclude.length > 0;
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

function buildSelectionLeaf(
  kind: "rarity" | "actionCost",
  value: string | number,
): SearchFilterNode {
  return kind === "rarity"
    ? { kind, match: { kind: "eq", value: value as string } }
    : { kind, match: { kind: "eq", value: value as number } };
}

function buildPackLeaf(value: string): SearchFilterNode {
  return {
    kind: "pack",
    value,
  };
}

function buildSelectionFilter(
  kind: "rarity" | "actionCost",
  selection: Pf2eTerminalValueSelection<string> | Pf2eTerminalValueSelection<number>,
): SearchFilterNode | null {
  if (kind === "rarity") {
    const includeValues = (selection.include as string[]).filter((value) => value.length > 0);
    const excludeValues = (selection.exclude as string[]).filter((value) => value.length > 0);
    const children: SearchFilterNode[] = [];
    if (includeValues.length === 1) {
      children.push({ kind: "rarity", match: { kind: "eq", value: includeValues[0]! } });
    } else if (includeValues.length > 1) {
      children.push({ kind: "rarity", match: { kind: "in", values: includeValues } });
    }
    if (excludeValues.length === 1) {
      children.push({ kind: "rarity", match: { kind: "notIn", values: [excludeValues[0]!] } });
    } else if (excludeValues.length > 1) {
      children.push({ kind: "rarity", match: { kind: "notIn", values: excludeValues } });
    }
    if (children.length === 0) {
      return null;
    }
    return children.length === 1 ? children[0]! : { kind: "allOf", children };
  }

  const includeChildren = selection.include.map((value) => buildSelectionLeaf(kind, value));
  const excludeChildren = selection.exclude.map((value) => buildSelectionLeaf(kind, value));
  const children: SearchFilterNode[] = [];

  if (includeChildren.length === 1) {
    children.push(includeChildren[0]!);
  } else if (includeChildren.length > 1) {
    children.push({ kind: "anyOf", children: includeChildren });
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

export function buildSearchFilterValueSelectionNode(
  kind: "rarity" | "actionCost",
  selection: Pf2eTerminalValueSelection<string> | Pf2eTerminalValueSelection<number>,
): SearchFilterNode | null {
  return buildSelectionFilter(kind, selection);
}

export function buildSearchFilterPackSelectionNode(
  selection: Pf2eTerminalValueSelection<string>,
): SearchFilterNode | null {
  const normalizedSelection = cloneStringSelection(selection);
  const includeChildren = normalizedSelection.include.map((value) => buildPackLeaf(value));
  const excludeChildren = normalizedSelection.exclude.map((value) => buildPackLeaf(value));
  const children: SearchFilterNode[] = [];

  if (includeChildren.length === 1) {
    children.push(includeChildren[0]!);
  } else if (includeChildren.length > 1) {
    children.push({ kind: "anyOf", children: includeChildren });
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

function collectSelectionEqValues<K extends "rarity" | "actionCost">(
  filter: SearchFilterNode,
  kind: K,
): Array<K extends "rarity" ? string : number> | null {
  if (filter.kind === kind && filter.match.kind === "eq") {
    return [filter.match.value] as Array<K extends "rarity" ? string : number>;
  }
  if (kind === "rarity" && filter.kind === "rarity" && filter.match.kind === "in") {
    return [...filter.match.values] as Array<K extends "rarity" ? string : number>;
  }

  if (filter.kind === "anyOf" || filter.kind === "allOf") {
    const values: Array<K extends "rarity" ? string : number> = [];
    for (const child of filter.children) {
      const childValues = collectSelectionEqValues(child, kind);
      if (!childValues) {
        return null;
      }
      values.push(...childValues);
    }
    return values;
  }

  return null;
}

function extractSelectionFilter<K extends "rarity" | "actionCost">(
  filter: SearchFilterNode,
  kind: K,
): Pf2eTerminalValueSelection<K extends "rarity" ? string : number> | null {
  if (filter.kind === kind && filter.match.kind === "eq") {
    return {
      include: [filter.match.value] as Array<K extends "rarity" ? string : number>,
      exclude: [],
    };
  }
  if (kind === "rarity" && filter.kind === "rarity" && filter.match.kind === "in") {
    return {
      include: [...filter.match.values] as Array<K extends "rarity" ? string : number>,
      exclude: [],
    };
  }
  if (kind === "rarity" && filter.kind === "rarity" && filter.match.kind === "notIn") {
    return {
      include: [],
      exclude: [...filter.match.values] as Array<K extends "rarity" ? string : number>,
    };
  }

  if (filter.kind === "anyOf") {
    const values = collectSelectionEqValues(filter, kind);
    return values
      ? {
          include: values,
          exclude: [],
        }
      : null;
  }

  if (filter.kind === "allOf") {
    const selection: Pf2eTerminalValueSelection<K extends "rarity" ? string : number> = {
      include: [],
      exclude: [],
    };

    for (const child of filter.children) {
      if (kind === "rarity" && child.kind === "rarity" && child.match.kind === "notIn") {
        selection.exclude.push(...child.match.values as Array<K extends "rarity" ? string : number>);
        continue;
      }
      if (child.kind === "not") {
        const excludedValues = collectSelectionEqValues(child.child, kind);
        if (!excludedValues) {
          return null;
        }
        selection.exclude.push(...excludedValues);
        continue;
      }

      const values = collectSelectionEqValues(child, kind);
      if (!values) {
        return null;
      }

      selection.include.push(...values);
    }

    return selection;
  }

  if (filter.kind === "not") {
    const excludedValues = collectSelectionEqValues(filter.child, kind);
    return excludedValues
      ? {
          include: [],
          exclude: excludedValues,
        }
      : null;
  }

  return null;
}

function collectPackValues(filter: SearchFilterNode): string[] | null {
  if (filter.kind === "pack") {
    return [filter.value];
  }

  if (filter.kind === "anyOf" || filter.kind === "allOf") {
    const values: string[] = [];
    for (const child of filter.children) {
      const childValues = collectPackValues(child);
      if (!childValues) {
        return null;
      }
      values.push(...childValues);
    }
    return values;
  }

  return null;
}

function extractPackSelection(filter: SearchFilterNode): Pf2eTerminalValueSelection<string> | null {
  if (filter.kind === "pack") {
    return {
      include: [filter.value],
      exclude: [],
    };
  }

  if (filter.kind === "anyOf") {
    const values = collectPackValues(filter);
    return values
      ? {
          include: values,
          exclude: [],
        }
      : null;
  }

  if (filter.kind === "allOf") {
    const selection: Pf2eTerminalValueSelection<string> = {
      include: [],
      exclude: [],
    };

    for (const child of filter.children) {
      if (child.kind === "not") {
        const excludedValues = collectPackValues(child.child);
        if (!excludedValues) {
          return null;
        }
        selection.exclude.push(...excludedValues);
        continue;
      }

      const values = collectPackValues(child);
      if (!values) {
        return null;
      }
      selection.include.push(...values);
    }

    return selection;
  }

  if (filter.kind === "not") {
    const excludedValues = collectPackValues(filter.child);
    return excludedValues
      ? {
          include: [],
          exclude: excludedValues,
        }
      : null;
  }

  return null;
}

export function tryExtractSearchFilterValueSelection(
  filter: SearchFilterNode,
):
  | { kind: "rarity"; selection: Pf2eTerminalValueSelection<string> }
  | { kind: "actionCost"; selection: Pf2eTerminalValueSelection<number> }
  | null {
  const raritySelection = extractSelectionFilter(filter, "rarity");
  if (raritySelection && hasSelectionValues(raritySelection)) {
    return {
      kind: "rarity",
      selection: cloneStringSelection(raritySelection),
    };
  }

  const actionCostSelection = extractSelectionFilter(filter, "actionCost");
  if (actionCostSelection && hasSelectionValues(actionCostSelection)) {
    return {
      kind: "actionCost",
      selection: cloneNumberSelection(actionCostSelection),
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

function isTopLevelSelectionChild(
  child: SearchFilterNode,
  kind: "rarity" | "actionCost",
): boolean {
  const selection = extractSelectionFilter(child, kind);
  return Boolean(selection && hasSelectionValues(selection));
}

function isTopLevelPackSelectionChild(child: SearchFilterNode): boolean {
  const selection = extractPackSelection(child);
  return Boolean(selection && hasSelectionValues(selection));
}

function isTopLevelMetadataChild(child: SearchFilterNode): boolean {
  return isCanonicalMetadataFilterCandidate(child) && Boolean(canonicalFilterToMetadataNode(child));
}

function isSearchNumericMatch(
  value: {
    levelMin: number | null;
    levelMax: number | null;
  } | SearchNumericMatch | null,
): value is SearchNumericMatch {
  return Boolean(value && typeof value === "object" && "kind" in value);
}

function normalizeSearchNumericMatch(match: SearchNumericMatch): SearchNumericMatch {
  if (match.kind !== "between") {
    return match;
  }

  return {
    kind: "between",
    min: Math.min(match.min, match.max),
    max: Math.max(match.min, match.max),
  };
}

function buildLevelFilter(levelRange: {
  levelMin: number | null;
  levelMax: number | null;
} | SearchNumericMatch | null): Extract<SearchFilterNode, { kind: "level" }> | null {
  if (levelRange === null) {
    return null;
  }

  if (isSearchNumericMatch(levelRange)) {
    return {
      kind: "level",
      match: normalizeSearchNumericMatch(levelRange),
    };
  }

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

function findTopLevelSelectionFilter<K extends "rarity" | "actionCost">(
  filter: SearchFilterNode | undefined,
  kind: K,
): Pf2eTerminalValueSelection<K extends "rarity" ? string : number> | null {
  for (const child of getTopLevelQueryChildren(filter)) {
    const selection = extractSelectionFilter(child, kind);
    if (selection && hasSelectionValues(selection)) {
      return selection;
    }
  }

  return null;
}

function findTopLevelPackSelectionFilter(
  filter: SearchFilterNode | undefined,
): Pf2eTerminalValueSelection<string> | null {
  for (const child of getTopLevelQueryChildren(filter)) {
    const selection = extractPackSelection(child);
    if (selection && hasSelectionValues(selection)) {
      return selection;
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

  return query.search.query;
}

export function getSearchQueryExcludeText(query: Pf2eTerminalSearchQuery): string {
  return query.mode === "search" ? query.search.exclude ?? "" : "";
}

export function getSearchQuerySearchProfile(query: Pf2eTerminalSearchQuery): SearchProfile | null {
  return query.mode === "search" ? (query.search.profile ?? DEFAULT_SEARCH_PROFILE) : null;
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
  const match = getSearchQueryLevelMatch(query);
  return {
    levelMin:
      match?.kind === "eq"
        ? match.value
        : match?.kind === "gt" || match?.kind === "gte"
          ? match.value
          : match?.kind === "between"
            ? match.min
            : null,
    levelMax:
      match?.kind === "eq"
        ? match.value
        : match?.kind === "lt" || match?.kind === "lte"
          ? match.value
          : match?.kind === "between"
            ? match.max
            : null,
  };
}

export function getSearchQueryLevelMatch(query: Pf2eTerminalSearchQuery): SearchNumericMatch | null {
  return findTopLevelLevelFilter(query.filter)?.match ?? null;
}

export function getSearchQueryRaritySelection(query: Pf2eTerminalSearchQuery): Pf2eTerminalValueSelection<string> {
  return cloneStringSelection(findTopLevelSelectionFilter(query.filter, "rarity") ?? createEmptyStringSelection());
}

export function getSearchQueryActionCostSelection(query: Pf2eTerminalSearchQuery): Pf2eTerminalValueSelection<number> {
  return cloneNumberSelection(
    findTopLevelSelectionFilter(query.filter, "actionCost") ?? createEmptyNumberSelection(),
  );
}

export function getSearchQueryPackSelection(query: Pf2eTerminalSearchQuery): Pf2eTerminalValueSelection<string> {
  return cloneStringSelection(findTopLevelPackSelectionFilter(query.filter) ?? createEmptyStringSelection());
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
  } | SearchNumericMatch | null,
): Pf2eTerminalSearchQuery {
  const rootOperator = getQueryRootOperator(query.filter);
  let children = removeFirstMatchingTopLevelChild(getTopLevelQueryChildren(query.filter), (child) => child.kind === "level");
  const levelFilter = buildLevelFilter(levelRange);
  if (levelFilter) {
    children = insertAfterCanonicalPrefix(children, levelFilter, (child) => child.kind === "scope");
  }
  return buildQueryFromTopLevelChildren(query, rootOperator, children);
}

export function setSearchQueryRaritySelection(
  query: Pf2eTerminalSearchQuery,
  selection: Pf2eTerminalValueSelection<string>,
): Pf2eTerminalSearchQuery {
  const rootOperator = getQueryRootOperator(query.filter);
  let children = removeFirstMatchingTopLevelChild(
    getTopLevelQueryChildren(query.filter),
    (child) => isTopLevelSelectionChild(child, "rarity"),
  );
  const rarityFilter = buildSelectionFilter("rarity", cloneStringSelection(selection));
  if (rarityFilter) {
    children = insertAfterCanonicalPrefix(children, rarityFilter, (child) => child.kind === "scope" || child.kind === "level");
  }
  return buildQueryFromTopLevelChildren(query, rootOperator, children);
}

export function setSearchQueryActionCostSelection(
  query: Pf2eTerminalSearchQuery,
  selection: Pf2eTerminalValueSelection<number>,
): Pf2eTerminalSearchQuery {
  const rootOperator = getQueryRootOperator(query.filter);
  let children = removeFirstMatchingTopLevelChild(
    getTopLevelQueryChildren(query.filter),
    (child) => isTopLevelSelectionChild(child, "actionCost"),
  );
  const actionCostFilter = buildSelectionFilter("actionCost", cloneNumberSelection(selection));
  if (actionCostFilter) {
    children = insertAfterCanonicalPrefix(
      children,
      actionCostFilter,
      (child) =>
        child.kind === "scope" ||
        child.kind === "level" ||
        isTopLevelPackSelectionChild(child) ||
        isTopLevelSelectionChild(child, "rarity"),
    );
  }
  return buildQueryFromTopLevelChildren(query, rootOperator, children);
}

export function setSearchQueryPackSelection(
  query: Pf2eTerminalSearchQuery,
  selection: Pf2eTerminalValueSelection<string>,
): Pf2eTerminalSearchQuery {
  const rootOperator = getQueryRootOperator(query.filter);
  let children = removeFirstMatchingTopLevelChild(
    getTopLevelQueryChildren(query.filter),
    isTopLevelPackSelectionChild,
  );
  const packFilter = buildSearchFilterPackSelectionNode(selection);
  if (packFilter) {
    children = insertAfterCanonicalPrefix(
      children,
      packFilter,
      (child) =>
        child.kind === "scope" || child.kind === "level" || isTopLevelSelectionChild(child, "rarity"),
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
        isTopLevelPackSelectionChild(child) ||
        isTopLevelSelectionChild(child, "rarity") ||
        isTopLevelSelectionChild(child, "actionCost"),
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
  selection: Pf2eTerminalValueSelection<string>,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): MetadataFilterNode | null {
  const normalizedSelection = normalizeQueryFieldSelection(field, selection, fieldSemanticsByName);
  if (!normalizedSelection) {
    return null;
  }

  const fieldSemantics = fieldSemanticsByName.get(field);
  if (!fieldSemantics) {
    return null;
  }

  if (fieldSemantics.fieldType === "set") {
    const includeClauses = normalizedSelection.include.map(
      (value): MetadataFilterNode => ({
        field: field as MetadataSetField,
        op: "includes",
        value,
      }),
    );
    const excludeClauses = normalizedSelection.exclude.map(
      (value): MetadataFilterNode => ({
        not: {
          field: field as MetadataSetField,
          op: "includes",
          value,
        },
      }),
    );
    const clauses: MetadataFilterNode[] = [];
    if (includeClauses.length === 1) {
      clauses.push(includeClauses[0]!);
    } else if (includeClauses.length > 1) {
      clauses.push({ or: includeClauses });
    }
    clauses.push(...excludeClauses);
    return clauses.length === 0 ? null : clauses.length === 1 ? clauses[0]! : { and: clauses };
  }

  if (fieldSemantics.fieldType === "enumString") {
    const clauses: MetadataFilterNode[] = [];
    if (normalizedSelection.include.length === 1) {
      clauses.push({
        field: field as MetadataEnumStringField,
        op: "eq",
        value: normalizedSelection.include[0]!,
      });
    } else if (normalizedSelection.include.length > 1) {
      clauses.push({
        field: field as MetadataEnumStringField,
        op: "in",
        values: normalizedSelection.include,
      });
    }
    if (normalizedSelection.exclude.length > 0) {
      clauses.push({
        field: field as MetadataEnumStringField,
        op: "notIn",
        values: normalizedSelection.exclude,
      });
    }
    return clauses.length === 0 ? null : clauses.length === 1 ? clauses[0]! : { and: clauses };
  }

  if (fieldSemantics.fieldType === "boolean") {
    const clauses: MetadataFilterNode[] = [];
    for (const value of normalizedSelection.include) {
      clauses.push({
        field: field as MetadataBooleanField,
        op: "eq",
        value: value === "true",
      });
    }
    for (const value of normalizedSelection.exclude) {
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
          query: query.search.query.trim(),
        },
      };
    case "search": {
      const exclude = trimOptionalText(query.search.exclude);
      return {
        mode: "search",
        limit,
        offset: query.offset,
        filter: query.filter,
        explain: query.explain,
        search: {
          query: query.search.query.trim(),
          profile: query.search.profile ?? DEFAULT_SEARCH_PROFILE,
          ...(exclude ? { exclude } : {}),
        },
      };
    }
  }
}
