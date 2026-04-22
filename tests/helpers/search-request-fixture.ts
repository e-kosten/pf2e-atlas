import type { MetadataFilterNode } from "../../src/domain/metadata-filter-types.js";
import {
  metadataFilterNodeToSearchRequestParts,
  type SearchRequest,
  type SearchRequestPart,
} from "../../src/domain/search-request-types.js";
import type { SearchCategoryInput, SearchScope, SearchSubcategoryInput } from "../../src/domain/search-types.js";

type SearchRequestFixtureInput = {
  searchProfile?: SearchRequest["searchProfile"];
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
  sort?: SearchRequest["sort"];
  sortSeed?: number;
};

function buildParts(input: SearchRequestFixtureInput): SearchRequestPart[] {
  return [
    ...(input.subcategory
      ? [
          {
            kind: "subcategory" as const,
            subcategory: input.subcategory,
          },
        ]
      : []),
    ...(input.levelMin !== undefined || input.levelMax !== undefined
      ? [
          {
            kind: "levelRange" as const,
            levelMin: input.levelMin ?? null,
            levelMax: input.levelMax ?? null,
          },
        ]
      : []),
    ...(input.rarity
      ? [
          {
            kind: "rarityPolicy" as const,
            policy: {
              any: [input.rarity],
              all: [],
              exclude: [],
            },
          },
        ]
      : []),
    ...(input.actionCost !== undefined
      ? [
          {
            kind: "actionCostPolicy" as const,
            policy: {
              any: [input.actionCost],
              all: [],
              exclude: [],
            },
          },
        ]
      : []),
    ...metadataFilterNodeToSearchRequestParts(input.metadata ?? null),
  ];
}

function buildBaseRequest(input: SearchRequestFixtureInput): Omit<SearchRequest, "intent"> {
  return {
    text: input.nameQuery ?? input.query,
    excludeQuery: input.excludeQuery,
    searchProfile: input.searchProfile,
    sort: input.sort,
    sortSeed: input.sortSeed,
    explain: input.explain,
    pack: input.pack,
    linksTo: input.linksTo,
    linksToMode: input.linksToMode,
    excludeLinksTo: input.excludeLinksTo,
    category: input.category,
    scopes: input.scopes,
    parts: buildParts(input),
    priceMin: input.priceMin,
    priceMax: input.priceMax,
    offset: input.offset,
    limit: input.limit,
  };
}

export function browseRequest(input: SearchRequestFixtureInput = {}): SearchRequest {
  return {
    intent: "browse",
    ...buildBaseRequest(input),
  };
}

export function searchRequest(input: SearchRequestFixtureInput = {}): SearchRequest {
  if (input.nameQuery && !input.query) {
    return {
      intent: "lookup",
      ...buildBaseRequest(input),
      text: input.nameQuery,
    };
  }

  return {
    intent: "search",
    ...buildBaseRequest(input),
    text: input.query,
  };
}

export function lookupRequest(input: SearchRequestFixtureInput = {}): SearchRequest {
  return {
    intent: "lookup",
    ...buildBaseRequest(input),
    text: input.nameQuery,
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

      const value = Reflect.get(target, property, receiver);
      return typeof value === "function" ? value.bind(target) : value;
    },
  });
}
