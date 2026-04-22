import {
  metadataFilterNodeToSearchRequestParts,
  type SearchRequest,
  type SearchRequestIntent,
  type SearchRequestPart,
} from "./search-request-types.js";
import type { MetadataFilterNode } from "../search/filters/types.js";
import type { SearchCategoryInput, SearchProfile, SearchScope, SearchSort, SearchSubcategoryInput } from "./search-types.js";

export type LegacySearchFiltersCompat = {
  searchProfile?: SearchProfile;
  sort?: SearchSort;
  sortSeed?: number;
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
};

export type LegacyOntologyQueryCompat = {
  kind?: "listRecords" | "lookup" | "search";
  request?: SearchRequest;
  filters?: LegacySearchFiltersCompat;
};

function buildLegacyParts(input: LegacySearchFiltersCompat): SearchRequestPart[] {
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
            policy: { any: [input.rarity], all: [], exclude: [] },
          },
        ]
      : []),
    ...(input.actionCost !== undefined
      ? [
          {
            kind: "actionCostPolicy" as const,
            policy: { any: [input.actionCost], all: [], exclude: [] },
          },
        ]
      : []),
    ...metadataFilterNodeToSearchRequestParts(input.metadata ?? null),
  ];
}

export function coerceSearchRequest(
  input: SearchRequest | LegacySearchFiltersCompat,
  defaultIntent: SearchRequestIntent = "search",
): SearchRequest {
  if ("intent" in input) {
    return input;
  }

  return {
    intent: input.nameQuery ? "lookup" : defaultIntent,
    text: input.nameQuery ?? input.query,
    excludeQuery: input.excludeQuery,
    searchProfile: input.query ? input.searchProfile : undefined,
    sort: input.sort,
    sortSeed: input.sortSeed,
    explain: input.explain,
    pack: input.pack,
    linksTo: input.linksTo,
    linksToMode: input.linksToMode,
    excludeLinksTo: input.excludeLinksTo,
    category: input.category,
    scopes: input.scopes,
    parts: buildLegacyParts(input),
    priceMin: input.priceMin,
    priceMax: input.priceMax,
    offset: input.offset,
    limit: input.limit,
  };
}

export function resolveOntologyQueryRequest(query: LegacyOntologyQueryCompat): SearchRequest {
  if (query.request) {
    return query.request;
  }

  return coerceSearchRequest(
    query.filters ?? {},
    query.kind === "lookup" ? "lookup" : query.kind === "search" ? "search" : "browse",
  );
}
