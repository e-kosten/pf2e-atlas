import type { MetadataFilterNode, MetadataPredicate } from "../../src/domain/metadata-filter-types.js";
import {
  buildAllOfFilter,
  buildAnyOfFilter,
  type BrowseSortSpec,
  type LookupSortSpec,
  type SearchFilterNode,
  type SearchRequest,
} from "../../src/domain/search-request-types.js";
import type { SearchCategoryInput, SearchProfile, SearchScope, SearchSort, SearchSubcategoryInput } from "../../src/domain/search-types.js";

type SearchRequestFixtureInput = {
  searchProfile?: SearchProfile;
  explain?: boolean;
  nameQuery?: string;
  query?: string;
  excludeQuery?: string;
  linksTo?: string[];
  linksToMode?: "any" | "all";
  excludeLinksTo?: string[];
  pack?: string;
  category?: SearchCategoryInput;
  subcategory?: SearchSubcategoryInput;
  scopes?: SearchScope[];
  levelMin?: number;
  levelMax?: number;
  rarity?: string;
  metadata?: MetadataFilterNode;
  priceMin?: number;
  priceMax?: number;
  actionCost?: number;
  offset?: number;
  limit?: number;
  sort?: SearchSort;
  sortSeed?: number;
};

function metadataPredicateToFilter(predicate: MetadataPredicate): SearchFilterNode {
  if (predicate.field === "actorMetric" || predicate.field === "itemMetric") {
    return {
      kind: "metric",
      metric: predicate.metric,
      op:
        predicate.op === "=="
          ? "eq"
          : predicate.op === "!="
            ? "notEq"
            : predicate.op === ">"
              ? "gt"
              : predicate.op === ">="
                ? "gte"
                : predicate.op === "<"
                  ? "lt"
                  : "lte",
      value: predicate.value,
    };
  }

  if (predicate.field === "actorMetricCompare" || predicate.field === "itemMetricCompare") {
    return {
      kind: "metricCompare",
      leftMetric: predicate.leftMetric,
      op:
        predicate.op === "=="
          ? "eq"
          : predicate.op === "!="
            ? "notEq"
            : predicate.op === ">"
              ? "gt"
              : predicate.op === ">="
                ? "gte"
                : predicate.op === "<"
                  ? "lt"
                  : "lte",
      rightMetric: predicate.rightMetric,
    };
  }

  if ("values" in predicate) {
    const op = predicate.op === "in" || predicate.op === "notIn" ? "eq" : "includes";
    const nodes = predicate.values.map(
      (value) =>
        ({
          kind: "metadataPredicate",
          predicate: { field: predicate.field, op, value },
        }) as SearchFilterNode,
    );

    if (predicate.op === "includesAll") {
      return nodes.length === 1 ? nodes[0]! : { kind: "allOf", children: nodes };
    }

    if (predicate.op === "includesAny" || predicate.op === "in") {
      return nodes.length === 1 ? nodes[0]! : { kind: "anyOf", children: nodes };
    }

    return {
      kind: "not",
      child: nodes.length === 1 ? nodes[0]! : { kind: "anyOf", children: nodes },
    };
  }

  if ("min" in predicate && "max" in predicate) {
    return {
      kind: "metadataPredicate",
      predicate: { field: predicate.field, op: "between", min: predicate.min, max: predicate.max } as never,
    };
  }

  return {
    kind: "metadataPredicate",
    predicate: ("value" in predicate
      ? { field: predicate.field, op: predicate.op, value: predicate.value }
      : predicate) as never,
  };
}

function metadataFilterNodeToFilter(node: MetadataFilterNode | null): SearchFilterNode | undefined {
  if (!node) {
    return undefined;
  }

  if ("and" in node) {
    return buildAllOfFilter(node.and.map((child) => metadataFilterNodeToFilter(child)));
  }

  if ("or" in node) {
    return buildAnyOfFilter(node.or.map((child) => metadataFilterNodeToFilter(child)));
  }

  if ("not" in node) {
    const child = metadataFilterNodeToFilter(node.not);
    return child ? { kind: "not", child } : undefined;
  }

  return metadataPredicateToFilter(node);
}

function buildScopeFilter(input: SearchRequestFixtureInput): SearchFilterNode | undefined {
  if (input.scopes && input.scopes.length > 0) {
    return buildAnyOfFilter(
      input.scopes.map((scope) => ({
        kind: "scope",
        category: scope.category,
        subcategory: scope.subcategories?.[0]
          ? { kind: "eq", value: scope.subcategories[0] }
          : { kind: "any" },
      })),
    );
  }

  if (!input.category) {
    return undefined;
  }

  return {
    kind: "scope",
    category: input.category,
    subcategory: input.subcategory ? { kind: "eq", value: input.subcategory } : { kind: "any" },
  };
}

function buildLinksFilter(input: SearchRequestFixtureInput): SearchFilterNode | undefined {
  const links = input.linksTo?.map((target) => ({ kind: "linksTo", target }) as SearchFilterNode) ?? [];
  const linksFilter =
    links.length === 0
      ? undefined
      : input.linksToMode === "all"
        ? buildAllOfFilter(links)
        : buildAnyOfFilter(links);

  const excludedLinks = input.excludeLinksTo?.map((target) => ({ kind: "linksTo", target }) as SearchFilterNode) ?? [];
  const excludedFilter =
    excludedLinks.length === 0
      ? undefined
      : {
          kind: "not",
          child: excludedLinks.length === 1 ? excludedLinks[0]! : ({ kind: "anyOf", children: excludedLinks } as SearchFilterNode),
        };

  return buildAllOfFilter([linksFilter, excludedFilter]);
}

function buildRangeFilter(
  kind: "level" | "price",
  min: number | undefined,
  max: number | undefined,
): SearchFilterNode | undefined {
  if (min === undefined && max === undefined) {
    return undefined;
  }

  if (min !== undefined && max !== undefined) {
    return min === max
      ? { kind, match: { kind: "eq", value: min } }
      : { kind, match: { kind: "between", min, max } };
  }

  return min !== undefined ? { kind, match: { kind: "gte", value: min } } : { kind, match: { kind: "lte", value: max! } };
}

function buildFilter(input: SearchRequestFixtureInput): SearchFilterNode | undefined {
  return buildAllOfFilter([
    input.pack ? ({ kind: "pack", value: input.pack } as SearchFilterNode) : undefined,
    buildScopeFilter(input),
    buildLinksFilter(input),
    buildRangeFilter("level", input.levelMin, input.levelMax),
    buildRangeFilter("price", input.priceMin, input.priceMax),
    input.rarity ? ({ kind: "rarity", match: { kind: "eq", value: input.rarity } } as SearchFilterNode) : undefined,
    input.actionCost !== undefined
      ? ({ kind: "actionCost", match: { kind: "eq", value: input.actionCost } } as SearchFilterNode)
      : undefined,
    metadataFilterNodeToFilter(input.metadata ?? null),
  ]);
}

function buildBrowseSort(sort: SearchSort | undefined, sortSeed: number | undefined): BrowseSortSpec | undefined {
  if (!sort || sort === "ranked") {
    return undefined;
  }
  return sort === "random" ? { kind: "random", seed: sortSeed } : { kind: sort };
}

function buildLookupSort(sort: SearchSort | undefined): LookupSortSpec | undefined {
  if (!sort || sort === "ranked" || sort === "random") {
    return undefined;
  }
  return { kind: sort, policy: "tiered" };
}

export function browseRequest(input: SearchRequestFixtureInput = {}): SearchRequest {
  return {
    mode: "browse",
    filter: buildFilter(input),
    sort: buildBrowseSort(input.sort, input.sortSeed),
    offset: input.offset,
    limit: input.limit,
  };
}

export function searchRequest(input: SearchRequestFixtureInput = {}): SearchRequest {
  if (input.nameQuery && !input.query) {
    return {
      mode: "lookup",
      search: { query: input.nameQuery },
      filter: buildFilter(input),
      sort: buildLookupSort(input.sort),
      offset: input.offset,
      limit: input.limit,
    };
  }

  return {
    mode: "search",
    search: {
      query: input.query ?? "",
      exclude: input.excludeQuery,
      profile: input.searchProfile,
    },
    explain: input.explain,
    filter: buildFilter(input),
    offset: input.offset,
    limit: input.limit,
  };
}

export function lookupRequest(input: SearchRequestFixtureInput = {}): SearchRequest {
  return {
    mode: "lookup",
    search: { query: input.nameQuery ?? "" },
    filter: buildFilter(input),
    sort: buildLookupSort(input.sort),
    offset: input.offset,
    limit: input.limit,
  };
}

export function browseQuery(label: string, input: SearchRequestFixtureInput = {}) {
  return {
    label,
    request: browseRequest(input),
  };
}

export function adaptLegacySearchCalls<
  Service extends {
    search: (request: SearchRequest) => Promise<unknown>;
    listRecords: (request: SearchRequest) => unknown;
    countRecords: (request: SearchRequest, options?: { lexicalOnly?: boolean }) => Promise<unknown>;
    openSearchWindow: (request: SearchRequest) => Promise<unknown>;
  },
>(service: Service) {
  return new Proxy(service, {
    get(target, property, receiver) {
      if (property === "search") {
        return (input: SearchRequestFixtureInput = {}) => target.search(searchRequest(input));
      }

      if (property === "listRecords") {
        return (input: SearchRequestFixtureInput = {}) => target.listRecords(browseRequest(input));
      }

      if (property === "countRecords") {
        return (
          input: SearchRequestFixtureInput = {},
          options: { mode?: "browse" | "search"; lexicalOnly?: boolean } = {},
        ) =>
          target.countRecords(options.mode === "browse" ? browseRequest(input) : searchRequest(input), {
            lexicalOnly: options.lexicalOnly,
          });
      }

      if (property === "openSearchWindow") {
        return (
          input: SearchRequestFixtureInput = {},
          options: { mode?: "browse" | "search" } = {},
        ) => target.openSearchWindow(options.mode === "browse" ? browseRequest(input) : searchRequest(input));
      }

      const value: unknown = Reflect.get(target, property, receiver);
      if (typeof value === "function") {
        return (...args: unknown[]): unknown => Reflect.apply(value, target, args);
      }
      return value;
    },
  });
}
