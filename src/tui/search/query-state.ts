import {
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../../domain/categories.js";
import { createScopedSearchDiscoveryApplicability } from "../../app/search-discovery-service.js";
import {
  buildAllOfFilter,
  buildAnyOfFilter,
  findSearchScopeFilter,
  SEARCH_REQUEST_VOCABULARY,
  type SearchFilterNode,
  type SearchNumericMatch,
} from "../../domain/search-request-types.js";
import type { MetadataFieldSemantics } from "../../domain/metadata-field-catalog.js";
import type {
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataSetField,
} from "../../domain/metadata-field-types.js";
import type {
  SearchCategory,
  SearchProfile,
  SearchSubcategory,
} from "../../domain/search-types.js";
import { SEARCH_VOCABULARY } from "../../domain/search-types.js";
import {
  cloneNumberSelection,
  cloneStringSelection,
  createEmptyNumberSelection,
  createEmptyStringSelection,
  normalizeNumberSelection,
  normalizeQueryFieldSelection,
  normalizeStringSelection,
} from "./selections.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalSearchMode,
  Pf2eTerminalSearchQuery,
  Pf2eTerminalValueSelection,
  SearchServiceDependencies,
} from "./service-types.js";

const DEFAULT_QUERY_LIMIT = 50;
const DEFAULT_SEARCH_PROFILE: SearchProfile = SEARCH_VOCABULARY.PROFILE.BALANCED;
const SEARCH_FILTER_NODE_KIND = SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND;
type SearchFilterNodeKind = typeof SEARCH_FILTER_NODE_KIND;

function trimOptionalText(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

type QueryRootOperator = SearchFilterNodeKind["ALL_OF"] | SearchFilterNodeKind["ANY_OF"];

function hasSelectionValues<T extends number | string>(selection: Pf2eTerminalValueSelection<T>): boolean {
  return selection.include.length > 0 || selection.exclude.length > 0;
}

function extractCanonicalPredicateProjection(filter: SearchFilterNode): SearchFilterNode | null {
  if (filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE || filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METRIC || filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METRIC_COMPARE) {
    return filter;
  }
  if (filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT) {
    const child = extractCanonicalPredicateProjection(filter.child);
    return child ? { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT, child } : null;
  }
  if (filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF || filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF) {
    const children = filter.children
      .map((child) => extractCanonicalPredicateProjection(child))
      .filter((child): child is SearchFilterNode => Boolean(child));
    const node = filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF ? buildAllOfFilter(children) : buildAnyOfFilter(children);
    return node ?? null;
  }
  return null;
}

function pruneCanonicalPredicateProjection(filter: SearchFilterNode): SearchFilterNode | null {
  if (filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE || filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METRIC || filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METRIC_COMPARE) {
    return null;
  }
  if (filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT) {
    const child = pruneCanonicalPredicateProjection(filter.child);
    return child ? { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT, child } : null;
  }
  if (filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF || filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF) {
    const children = filter.children
      .map((child) => pruneCanonicalPredicateProjection(child))
      .filter((child): child is SearchFilterNode => Boolean(child));
    const node = filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF ? buildAllOfFilter(children) : buildAnyOfFilter(children);
    return node ?? null;
  }
  return filter;
}

function isScopeDependentFilterNode(filter: SearchFilterNode): boolean {
  return (
    filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE ||
    filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METRIC ||
    filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METRIC_COMPARE ||
    filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ACTION_COST
  );
}

function pruneScopeDependentFilterNode(filter: SearchFilterNode): SearchFilterNode | null {
  if (isScopeDependentFilterNode(filter)) {
    return null;
  }
  if (filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT) {
    const child = pruneScopeDependentFilterNode(filter.child);
    return child ? { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT, child } : null;
  }
  if (filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF || filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF) {
    const children = filter.children
      .map((child) => pruneScopeDependentFilterNode(child))
      .filter((child): child is SearchFilterNode => Boolean(child));
    if (children.length === 0) {
      return null;
    }
    return children.length === 1 ? children[0]! : { kind: filter.kind, children };
  }
  return filter;
}

function buildSelectionLeaf(
  kind: SearchFilterNodeKind["RARITY"] | SearchFilterNodeKind["ACTION_COST"],
  value: string | number,
): SearchFilterNode {
  return kind === SEARCH_FILTER_NODE_KIND["RARITY"]
    ? { kind, match: { kind: "eq", value: value as string } }
    : { kind, match: { kind: "eq", value: value as number } };
}

function buildPackLeaf(value: string): SearchFilterNode {
  return {
    kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.PACK,
    value,
  };
}

function buildSelectionFilter(
  kind: SearchFilterNodeKind["RARITY"] | SearchFilterNodeKind["ACTION_COST"],
  selection: Pf2eTerminalValueSelection<string> | Pf2eTerminalValueSelection<number>,
): SearchFilterNode | null {
  if (kind === SEARCH_FILTER_NODE_KIND["RARITY"]) {
    const normalizedSelection = normalizeStringSelection(selection as Pf2eTerminalValueSelection<string>);
    const includeValues = normalizedSelection.include;
    const excludeValues = normalizedSelection.exclude;
    const children: SearchFilterNode[] = [];
    if (includeValues.length === 1) {
      children.push({ kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY, match: { kind: "eq", value: includeValues[0]! } });
    } else if (includeValues.length > 1) {
      children.push({ kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY, match: { kind: "in", values: includeValues } });
    }
    if (excludeValues.length === 1) {
      children.push({
        kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY,
        match: {
          kind: SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.NOT_IN,
          values: [excludeValues[0]!],
        },
      });
    } else if (excludeValues.length > 1) {
      children.push({
        kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY,
        match: { kind: SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.NOT_IN, values: excludeValues },
      });
    }
    if (children.length === 0) {
      return null;
    }
    return children.length === 1 ? children[0]! : { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF, children };
  }

  const normalizedSelection = normalizeNumberSelection(selection as Pf2eTerminalValueSelection<number>);
  const includeChildren = normalizedSelection.include.map((value) => buildSelectionLeaf(kind, value));
  const excludeChildren = normalizedSelection.exclude.map((value) => buildSelectionLeaf(kind, value));
  const children: SearchFilterNode[] = [];

  if (includeChildren.length === 1) {
    children.push(includeChildren[0]!);
  } else if (includeChildren.length > 1) {
    children.push({ kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF, children: includeChildren });
  }

  children.push(...excludeChildren.map((child) => ({ kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT, child }) satisfies SearchFilterNode));

  if (children.length === 0) {
    return null;
  }

  return children.length === 1 ? children[0]! : { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF, children };
}

export function buildSearchFilterValueSelectionNode(
  kind: SearchFilterNodeKind["RARITY"] | SearchFilterNodeKind["ACTION_COST"],
  selection: Pf2eTerminalValueSelection<string> | Pf2eTerminalValueSelection<number>,
): SearchFilterNode | null {
  return buildSelectionFilter(kind, selection);
}

export function buildSearchFilterPackSelectionNode(
  selection: Pf2eTerminalValueSelection<string>,
): SearchFilterNode | null {
  const normalizedSelection = normalizeStringSelection(selection);
  const includeChildren = normalizedSelection.include.map((value) => buildPackLeaf(value));
  const excludeChildren = normalizedSelection.exclude.map((value) => buildPackLeaf(value));
  const children: SearchFilterNode[] = [];

  if (includeChildren.length === 1) {
    children.push(includeChildren[0]!);
  } else if (includeChildren.length > 1) {
    children.push({ kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF, children: includeChildren });
  }

  children.push(...excludeChildren.map((child) => ({ kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT, child }) satisfies SearchFilterNode));

  if (children.length === 0) {
    return null;
  }

  return children.length === 1 ? children[0]! : { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF, children };
}

function collectSelectionEqValues<K extends SearchFilterNodeKind["RARITY"] | SearchFilterNodeKind["ACTION_COST"]>(
  filter: SearchFilterNode,
  kind: K,
): Array<K extends SearchFilterNodeKind["RARITY"] ? string : number> | null {
  if (filter.kind === kind && filter.match.kind === "eq") {
    return [filter.match.value] as Array<K extends SearchFilterNodeKind["RARITY"] ? string : number>;
  }
  if (kind === SEARCH_FILTER_NODE_KIND["RARITY"] && filter.kind === SEARCH_FILTER_NODE_KIND["RARITY"] && filter.match.kind === "in") {
    return [...filter.match.values] as Array<K extends SearchFilterNodeKind["RARITY"] ? string : number>;
  }

  if (filter.kind === SEARCH_FILTER_NODE_KIND["ANY_OF"] || filter.kind === SEARCH_FILTER_NODE_KIND["ALL_OF"]) {
    const values: Array<K extends SearchFilterNodeKind["RARITY"] ? string : number> = [];
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

function extractSelectionFilter<K extends SearchFilterNodeKind["RARITY"] | SearchFilterNodeKind["ACTION_COST"]>(
  filter: SearchFilterNode,
  kind: K,
): Pf2eTerminalValueSelection<K extends SearchFilterNodeKind["RARITY"] ? string : number> | null {
  if (filter.kind === kind && filter.match.kind === "eq") {
    return {
      include: [filter.match.value] as Array<K extends SearchFilterNodeKind["RARITY"] ? string : number>,
      exclude: [],
    };
  }
  if (kind === SEARCH_FILTER_NODE_KIND["RARITY"] && filter.kind === SEARCH_FILTER_NODE_KIND["RARITY"] && filter.match.kind === "in") {
    return {
      include: [...filter.match.values] as Array<K extends SearchFilterNodeKind["RARITY"] ? string : number>,
      exclude: [],
    };
  }
  if (
    kind === SEARCH_FILTER_NODE_KIND["RARITY"] &&
    filter.kind === SEARCH_FILTER_NODE_KIND["RARITY"] &&
    filter.match.kind === SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.NOT_IN
  ) {
    return {
      include: [],
      exclude: [...filter.match.values] as Array<K extends SearchFilterNodeKind["RARITY"] ? string : number>,
    };
  }

  if (filter.kind === SEARCH_FILTER_NODE_KIND["ANY_OF"]) {
    const values = collectSelectionEqValues(filter, kind);
    return values
      ? {
          include: values,
          exclude: [],
        }
      : null;
  }

  if (filter.kind === SEARCH_FILTER_NODE_KIND["ALL_OF"]) {
    const selection: Pf2eTerminalValueSelection<K extends SearchFilterNodeKind["RARITY"] ? string : number> = {
      include: [],
      exclude: [],
    };

    for (const child of filter.children) {
      if (
        kind === SEARCH_FILTER_NODE_KIND["RARITY"] &&
        child.kind === SEARCH_FILTER_NODE_KIND["RARITY"] &&
        child.match.kind === SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.NOT_IN
      ) {
        selection.exclude.push(...(child.match.values as Array<K extends SearchFilterNodeKind["RARITY"] ? string : number>));
        continue;
      }
      if (child.kind === SEARCH_FILTER_NODE_KIND["NOT"]) {
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

  if (filter.kind === SEARCH_FILTER_NODE_KIND["NOT"]) {
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
  if (filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.PACK) {
    return [filter.value];
  }

  if (filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF || filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF) {
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
  if (filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.PACK) {
    return {
      include: [filter.value],
      exclude: [],
    };
  }

  if (filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF) {
    const values = collectPackValues(filter);
    return values
      ? {
          include: values,
          exclude: [],
        }
      : null;
  }

  if (filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF) {
    const selection: Pf2eTerminalValueSelection<string> = {
      include: [],
      exclude: [],
    };

    for (const child of filter.children) {
      if (child.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT) {
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

  if (filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT) {
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
  | { kind: SearchFilterNodeKind["RARITY"]; selection: Pf2eTerminalValueSelection<string> }
  | { kind: SearchFilterNodeKind["ACTION_COST"]; selection: Pf2eTerminalValueSelection<number> }
  | null {
  const raritySelection = extractSelectionFilter(filter, SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY);
  if (raritySelection && hasSelectionValues(raritySelection)) {
    return {
      kind: SEARCH_FILTER_NODE_KIND["RARITY"],
      selection: cloneStringSelection(raritySelection),
    };
  }

  const actionCostSelection = extractSelectionFilter(filter, SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ACTION_COST);
  if (actionCostSelection && hasSelectionValues(actionCostSelection)) {
    return {
      kind: SEARCH_FILTER_NODE_KIND["ACTION_COST"],
      selection: cloneNumberSelection(actionCostSelection),
    };
  }

  return null;
}

function getQueryRootOperator(filter: SearchFilterNode | undefined): QueryRootOperator {
  return filter?.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF ? SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF : SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF;
}

function getTopLevelQueryChildren(filter: SearchFilterNode | undefined): SearchFilterNode[] {
  if (!filter) {
    return [];
  }

  if (filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF || filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF) {
    return [...filter.children];
  }

  return [filter];
}

function buildQueryFromTopLevelChildren(
  query: Pf2eTerminalSearchQuery,
  rootOperator: QueryRootOperator,
  children: SearchFilterNode[],
): Pf2eTerminalSearchQuery {
  const filter = rootOperator === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF ? buildAnyOfFilter(children) : buildAllOfFilter(children);
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
  return insertNodesAfterCanonicalPrefix(children, [node], predicate);
}

function insertNodesAfterCanonicalPrefix(
  children: SearchFilterNode[],
  nodes: SearchFilterNode[],
  predicate: (child: SearchFilterNode) => boolean,
): SearchFilterNode[] {
  let index = 0;
  while (index < children.length && predicate(children[index]!)) {
    index += 1;
  }
  return [...children.slice(0, index), ...nodes, ...children.slice(index)];
}

function flattenSelectionFilterForRoot(
  filter: SearchFilterNode,
  rootOperator: QueryRootOperator,
): SearchFilterNode[] {
  return filter.kind === rootOperator ? [...filter.children] : [filter];
}

function isTopLevelSelectionChild(
  child: SearchFilterNode,
  kind: SearchFilterNodeKind["RARITY"] | SearchFilterNodeKind["ACTION_COST"],
): boolean {
  const selection = extractSelectionFilter(child, kind);
  return Boolean(selection && hasSelectionValues(selection));
}

function isTopLevelPackSelectionChild(child: SearchFilterNode): boolean {
  const selection = extractPackSelection(child);
  return Boolean(selection && hasSelectionValues(selection));
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
} | SearchNumericMatch | null): Extract<SearchFilterNode, { kind: SearchFilterNodeKind["LEVEL"] }> | null {
  if (levelRange === null) {
    return null;
  }

  if (isSearchNumericMatch(levelRange)) {
    return {
      kind: SEARCH_FILTER_NODE_KIND["LEVEL"],
      match: normalizeSearchNumericMatch(levelRange),
    };
  }

  const { levelMin, levelMax } = levelRange;
  if (levelMin === null && levelMax === null) {
    return null;
  }

  if (levelMin !== null && levelMax !== null) {
    return levelMin === levelMax
      ? { kind: SEARCH_FILTER_NODE_KIND["LEVEL"], match: { kind: "eq", value: levelMin } }
      : {
          kind: SEARCH_FILTER_NODE_KIND["LEVEL"],
          match: {
            kind: "between",
            min: Math.min(levelMin, levelMax),
            max: Math.max(levelMin, levelMax),
          },
        };
  }

  return levelMin !== null
    ? { kind: SEARCH_FILTER_NODE_KIND["LEVEL"], match: { kind: "gte", value: levelMin } }
    : { kind: SEARCH_FILTER_NODE_KIND["LEVEL"], match: { kind: "lte", value: levelMax! } };
}

function findTopLevelLevelFilter(
  filter: SearchFilterNode | undefined,
): Extract<SearchFilterNode, { kind: SearchFilterNodeKind["LEVEL"] }> | null {
  for (const child of getTopLevelQueryChildren(filter)) {
    if (child.kind === SEARCH_FILTER_NODE_KIND["LEVEL"]) {
      return child;
    }
  }

  return null;
}

function findTopLevelSelectionFilter<K extends SearchFilterNodeKind["RARITY"] | SearchFilterNodeKind["ACTION_COST"]>(
  filter: SearchFilterNode | undefined,
  kind: K,
): Pf2eTerminalValueSelection<K extends SearchFilterNodeKind["RARITY"] ? string : number> | null {
  const mergedSelection: Pf2eTerminalValueSelection<K extends SearchFilterNodeKind["RARITY"] ? string : number> = {
    include: [],
    exclude: [],
  };
  for (const child of getTopLevelQueryChildren(filter)) {
    const selection = extractSelectionFilter(child, kind);
    if (selection && hasSelectionValues(selection)) {
      mergedSelection.include.push(...selection.include);
      mergedSelection.exclude.push(...selection.exclude);
    }
  }

  return hasSelectionValues(mergedSelection) ? mergedSelection : null;
}

function findTopLevelPackSelectionFilter(
  filter: SearchFilterNode | undefined,
): Pf2eTerminalValueSelection<string> | null {
  const mergedSelection: Pf2eTerminalValueSelection<string> = {
    include: [],
    exclude: [],
  };
  for (const child of getTopLevelQueryChildren(filter)) {
    const selection = extractPackSelection(child);
    if (selection && hasSelectionValues(selection)) {
      mergedSelection.include.push(...selection.include);
      mergedSelection.exclude.push(...selection.exclude);
    }
  }

  return hasSelectionValues(mergedSelection) ? mergedSelection : null;
}

function extractQueryPredicateFilter(
  filter: SearchFilterNode | undefined,
): SearchFilterNode | null {
  const rootOperator = getQueryRootOperator(filter);
  const metadataChildren = getTopLevelQueryChildren(filter)
    .map((child) => extractCanonicalPredicateProjection(child))
    .filter((child): child is SearchFilterNode => Boolean(child));

  if (metadataChildren.length === 0) {
    return null;
  }

  return metadataChildren.length === 1
    ? metadataChildren[0]!
    : { kind: rootOperator, children: metadataChildren };
}

export function createDefaultQuery(mode: Pf2eTerminalSearchMode = SEARCH_REQUEST_VOCABULARY.MODE.BROWSE): Pf2eTerminalSearchQuery {
  switch (mode) {
    case SEARCH_REQUEST_VOCABULARY.MODE.BROWSE:
      return {
        mode: SEARCH_REQUEST_VOCABULARY.MODE.BROWSE,
        limit: DEFAULT_QUERY_LIMIT,
      };
    case SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP:
      return {
        mode: SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP,
        limit: DEFAULT_QUERY_LIMIT,
        search: {
          query: "",
        },
      };
    case SEARCH_REQUEST_VOCABULARY.MODE.SEARCH:
      return {
        mode: SEARCH_REQUEST_VOCABULARY.MODE.SEARCH,
        limit: DEFAULT_QUERY_LIMIT,
        search: {
          query: "",
          profile: DEFAULT_SEARCH_PROFILE,
        },
      };
  }
}

export function getSearchQueryText(query: Pf2eTerminalSearchQuery): string {
  if (query.mode === SEARCH_REQUEST_VOCABULARY.MODE.BROWSE) {
    return "";
  }

  return query.search.query;
}

export function getSearchQueryExcludeText(query: Pf2eTerminalSearchQuery): string {
  return query.mode === SEARCH_REQUEST_VOCABULARY.MODE.SEARCH ? query.search.exclude ?? "" : "";
}

export function getSearchQuerySearchProfile(query: Pf2eTerminalSearchQuery): SearchProfile | null {
  return query.mode === SEARCH_REQUEST_VOCABULARY.MODE.SEARCH ? (query.search.profile ?? DEFAULT_SEARCH_PROFILE) : null;
}

export function setSearchQueryText(
  query: Pf2eTerminalSearchQuery,
  text: string,
): Pf2eTerminalSearchQuery {
  if (query.mode === SEARCH_REQUEST_VOCABULARY.MODE.BROWSE) {
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
  if (query.mode !== SEARCH_REQUEST_VOCABULARY.MODE.SEARCH) {
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
  if (query.mode !== SEARCH_REQUEST_VOCABULARY.MODE.SEARCH) {
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
  if (!scope || scope.subcategory.kind !== SEARCH_REQUEST_VOCABULARY.SCOPE_SUBCATEGORY_MATCH_KIND.EQ) {
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
  return normalizeStringSelection(findTopLevelSelectionFilter(query.filter, SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY) ?? createEmptyStringSelection());
}

export function getSearchQueryActionCostSelection(query: Pf2eTerminalSearchQuery): Pf2eTerminalValueSelection<number> {
  return normalizeNumberSelection(findTopLevelSelectionFilter(query.filter, SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ACTION_COST) ?? createEmptyNumberSelection());
}

export function getSearchQueryPackSelection(query: Pf2eTerminalSearchQuery): Pf2eTerminalValueSelection<string> {
  return normalizeStringSelection(findTopLevelPackSelectionFilter(query.filter) ?? createEmptyStringSelection());
}

export function getSearchQueryPredicateFilter(query: Pf2eTerminalSearchQuery): SearchFilterNode | null {
  return extractQueryPredicateFilter(query.filter);
}

export function getSearchQueryRootOperator(query: Pf2eTerminalSearchQuery): QueryRootOperator {
  return getQueryRootOperator(query.filter);
}

export function setSearchQueryCategory(
  query: Pf2eTerminalSearchQuery,
  category: SearchCategory | null,
): Pf2eTerminalSearchQuery {
  const rootOperator = getQueryRootOperator(query.filter);
  let children = removeFirstMatchingTopLevelChild(getTopLevelQueryChildren(query.filter), (child) => child.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.SCOPE);
  if (category) {
    children = [
      {
        kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.SCOPE,
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
  const children = removeFirstMatchingTopLevelChild(getTopLevelQueryChildren(query.filter), (child) => child.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.SCOPE);
  return buildQueryFromTopLevelChildren(query, rootOperator, [
    {
      kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.SCOPE,
      category,
      subcategory: subcategory ? { kind: "eq", value: subcategory } : { kind: "any" },
    },
    ...children,
  ]);
}

export function replaceSearchQueryRootScope(
  query: Pf2eTerminalSearchQuery,
  scope: Extract<SearchFilterNode, { kind: SearchFilterNodeKind["SCOPE"] }> | null,
): Pf2eTerminalSearchQuery {
  const rootOperator = getQueryRootOperator(query.filter);
  const currentCategory = getSearchQueryCategory(query);
  const nextCategory = normalizeSearchCategory(scope?.category ?? null) ?? null;
  const shouldPruneScopeDependentClauses = currentCategory !== nextCategory;
  const children = removeAllMatchingTopLevelChildren(
    getTopLevelQueryChildren(query.filter),
    (child) => child.kind === SEARCH_FILTER_NODE_KIND["SCOPE"],
  )
    .map((child) => (shouldPruneScopeDependentClauses ? pruneScopeDependentFilterNode(child) : child))
    .filter((child): child is SearchFilterNode => Boolean(child));
  return buildQueryFromTopLevelChildren(query, rootOperator, scope ? [scope, ...children] : children);
}

export function setSearchQueryLevelRange(
  query: Pf2eTerminalSearchQuery,
  levelRange: {
    levelMin: number | null;
    levelMax: number | null;
  } | SearchNumericMatch | null,
): Pf2eTerminalSearchQuery {
  const rootOperator = getQueryRootOperator(query.filter);
  let children = removeFirstMatchingTopLevelChild(getTopLevelQueryChildren(query.filter), (child) => child.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.LEVEL);
  const levelFilter = buildLevelFilter(levelRange);
  if (levelFilter) {
    children = insertAfterCanonicalPrefix(children, levelFilter, (child) => child.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.SCOPE);
  }
  return buildQueryFromTopLevelChildren(query, rootOperator, children);
}

export function setSearchQueryRaritySelection(
  query: Pf2eTerminalSearchQuery,
  selection: Pf2eTerminalValueSelection<string>,
): Pf2eTerminalSearchQuery {
  const rootOperator = getQueryRootOperator(query.filter);
  let children = removeAllMatchingTopLevelChildren(
    getTopLevelQueryChildren(query.filter),
    (child) => isTopLevelSelectionChild(child, SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY),
  );
  const rarityFilter = buildSelectionFilter(SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY, cloneStringSelection(selection));
  if (rarityFilter) {
    children = insertNodesAfterCanonicalPrefix(
      children,
      flattenSelectionFilterForRoot(rarityFilter, rootOperator),
      (child) => child.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.SCOPE || child.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.LEVEL,
    );
  }
  return buildQueryFromTopLevelChildren(query, rootOperator, children);
}

export function setSearchQueryActionCostSelection(
  query: Pf2eTerminalSearchQuery,
  selection: Pf2eTerminalValueSelection<number>,
): Pf2eTerminalSearchQuery {
  const rootOperator = getQueryRootOperator(query.filter);
  let children = removeAllMatchingTopLevelChildren(
    getTopLevelQueryChildren(query.filter),
    (child) => isTopLevelSelectionChild(child, SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ACTION_COST),
  );
  const actionCostFilter = buildSelectionFilter(SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ACTION_COST, cloneNumberSelection(selection));
  if (actionCostFilter) {
    children = insertNodesAfterCanonicalPrefix(
      children,
      flattenSelectionFilterForRoot(actionCostFilter, rootOperator),
      (child) =>
        child.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.SCOPE ||
        child.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.LEVEL ||
        isTopLevelPackSelectionChild(child) ||
        isTopLevelSelectionChild(child, SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY),
    );
  }
  return buildQueryFromTopLevelChildren(query, rootOperator, children);
}

export function setSearchQueryPackSelection(
  query: Pf2eTerminalSearchQuery,
  selection: Pf2eTerminalValueSelection<string>,
): Pf2eTerminalSearchQuery {
  const rootOperator = getQueryRootOperator(query.filter);
  let children = removeAllMatchingTopLevelChildren(
    getTopLevelQueryChildren(query.filter),
    isTopLevelPackSelectionChild,
  );
  const packFilter = buildSearchFilterPackSelectionNode(selection);
  if (packFilter) {
    children = insertNodesAfterCanonicalPrefix(
      children,
      flattenSelectionFilterForRoot(packFilter, rootOperator),
      (child) =>
        child.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.SCOPE || child.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.LEVEL || isTopLevelSelectionChild(child, SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY),
    );
  }
  return buildQueryFromTopLevelChildren(query, rootOperator, children);
}

export function setSearchQueryPredicateFilter(
  query: Pf2eTerminalSearchQuery,
  node: SearchFilterNode | null,
): Pf2eTerminalSearchQuery {
  const rootOperator = getQueryRootOperator(query.filter);
  let children = getTopLevelQueryChildren(query.filter)
    .map((child) => pruneCanonicalPredicateProjection(child))
    .filter((child): child is SearchFilterNode => Boolean(child));
  if (node) {
    children = insertAfterCanonicalPrefix(
      children,
      node,
      (child) =>
        child.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.SCOPE ||
        child.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.LEVEL ||
        isTopLevelPackSelectionChild(child) ||
        isTopLevelSelectionChild(child, SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY) ||
        isTopLevelSelectionChild(child, SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ACTION_COST),
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
    SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ACTION_COST,
    createScopedSearchDiscoveryApplicability(SEARCH_REQUEST_VOCABULARY.MODE.BROWSE, category, subcategory),
  );
}

export function buildSearchFilterNodeForQueryFieldSelection(
  field: Pf2eTerminalFacetField,
  selection: Pf2eTerminalValueSelection<string>,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): SearchFilterNode | null {
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
      (value): SearchFilterNode => ({
        kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE,
        predicate: {
          field: field as MetadataSetField,
          op: "includes",
          value,
        },
      }),
    );
    const excludeClauses = normalizedSelection.exclude.map(
      (value): SearchFilterNode => ({
        kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT,
        child: {
          kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE,
          predicate: {
            field: field as MetadataSetField,
            op: "includes",
            value,
          },
        },
      }),
    );
    const clauses: SearchFilterNode[] = [];
    if (includeClauses.length === 1) {
      clauses.push(includeClauses[0]!);
    } else if (includeClauses.length > 1) {
      clauses.push({ kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF, children: includeClauses });
    }
    clauses.push(...excludeClauses);
    return clauses.length === 0 ? null : clauses.length === 1 ? clauses[0]! : { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF, children: clauses };
  }

  if (fieldSemantics.fieldType === "enumString") {
    const clauses: SearchFilterNode[] = [];
    if (normalizedSelection.include.length === 1) {
      clauses.push({
        kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE,
        predicate: {
          field: field as MetadataEnumStringField,
          op: "eq",
          value: normalizedSelection.include[0]!,
        },
      });
    } else if (normalizedSelection.include.length > 1) {
      clauses.push({
        kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF,
        children: normalizedSelection.include.map((value) => ({
          kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE,
          predicate: { field: field as MetadataEnumStringField, op: "eq", value },
        })),
      });
    }
    if (normalizedSelection.exclude.length > 0) {
      clauses.push({
        kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT,
        child:
          normalizedSelection.exclude.length === 1
            ? {
                kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE,
                predicate: {
                  field: field as MetadataEnumStringField,
                  op: "eq",
                  value: normalizedSelection.exclude[0]!,
                },
              }
            : {
                kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF,
                children: normalizedSelection.exclude.map((value) => ({
                  kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE,
                  predicate: { field: field as MetadataEnumStringField, op: "eq", value },
                })),
              },
      });
    }
    return clauses.length === 0 ? null : clauses.length === 1 ? clauses[0]! : { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF, children: clauses };
  }

  if (fieldSemantics.fieldType === "boolean") {
    const clauses: SearchFilterNode[] = [];
    for (const value of normalizedSelection.include) {
      clauses.push({
        kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE,
        predicate: {
          field: field as MetadataBooleanField,
          op: "eq",
          value: value === "true",
        },
      });
    }
    for (const value of normalizedSelection.exclude) {
      clauses.push({
        kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT,
        child: {
          kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE,
          predicate: {
            field: field as MetadataBooleanField,
            op: "eq",
            value: value === "true",
          },
        },
      });
    }
    return clauses.length === 0 ? null : clauses.length === 1 ? clauses[0]! : { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF, children: clauses };
  }

  return null;
}

export function normalizeSearchQuery(query: Pf2eTerminalSearchQuery): Pf2eTerminalSearchQuery {
  const limit = query.limit ?? DEFAULT_QUERY_LIMIT;

  switch (query.mode) {
    case SEARCH_REQUEST_VOCABULARY.MODE.BROWSE:
      return {
        mode: SEARCH_REQUEST_VOCABULARY.MODE.BROWSE,
        limit,
        offset: query.offset,
        filter: query.filter,
        sort: query.sort,
      };
    case SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP:
      return {
        mode: SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP,
        limit,
        offset: query.offset,
        filter: query.filter,
        sort: query.sort,
        search: {
          query: query.search.query.trim(),
        },
      };
    case SEARCH_REQUEST_VOCABULARY.MODE.SEARCH: {
      const exclude = trimOptionalText(query.search.exclude);
      return {
        mode: SEARCH_REQUEST_VOCABULARY.MODE.SEARCH,
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
