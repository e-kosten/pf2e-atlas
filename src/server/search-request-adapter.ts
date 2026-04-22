import {
  metadataFilterNodeToSearchRequestParts,
  type SearchRequest,
  type SearchRequestIntent,
  type SearchRequestPart,
} from "../domain/search-request-types.js";
import type { MetadataFilterNode } from "../domain/metadata-filter-types.js";
import type { LookupOptions, SearchCategoryInput, SearchSubcategoryInput } from "../domain/search-types.js";

type SearchRequestTransportInput = {
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
  scopes?: SearchRequest["scopes"];
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

function buildTransportParts(input: {
  subcategory?: SearchSubcategoryInput;
  levelMin?: number;
  levelMax?: number;
  rarity?: string;
  actionCost?: number;
  metadata?: MetadataFilterNode;
}): SearchRequestPart[] {
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

export function buildSearchRequestFromTransportInput(
  intent: SearchRequestIntent,
  input: SearchRequestTransportInput,
): SearchRequest {
  return {
    intent,
    text: intent === "lookup" ? input.nameQuery : intent === "search" ? input.query : undefined,
    excludeQuery: intent === "search" ? input.excludeQuery : undefined,
    searchProfile: intent === "search" ? input.searchProfile : undefined,
    explain: input.explain,
    pack: input.pack,
    linksTo: input.linksTo,
    linksToMode: input.linksToMode,
    excludeLinksTo: input.excludeLinksTo,
    category: input.category,
    scopes: input.scopes,
    parts: buildTransportParts(input),
    priceMin: input.priceMin,
    priceMax: input.priceMax,
    offset: input.offset,
    limit: input.limit,
  };
}

export function buildLookupRequest(name: string, options: LookupOptions = {}): SearchRequest {
  return {
    intent: "lookup",
    text: name,
    pack: options.pack,
    category: options.category,
    parts: buildTransportParts({
      subcategory: options.subcategory,
    }),
    limit: 5,
  };
}
