import {
  buildAllOfFilter,
  buildScopeFilter,
  type SearchFilterNode,
  type SearchRequest,
  type SearchRequestMode,
} from "../domain/search-request-types.js";
import type { LookupOptions } from "../domain/search-types.js";

type SearchRequestTransportInput = {
  searchProfile?: "lexical" | "balanced" | "concept";
  explain?: boolean;
  nameQuery?: string;
  query?: string;
  excludeQuery?: string;
  filter?: SearchFilterNode;
  offset?: number;
  limit?: number;
};

export function buildSearchRequestFromTransportInput(
  mode: SearchRequestMode,
  input: SearchRequestTransportInput,
): SearchRequest {
  if (mode === "browse") {
    return {
      mode,
      filter: input.filter,
      offset: input.offset,
      limit: input.limit,
    };
  }

  if (mode === "lookup") {
    return {
      mode,
      search: {
        query: input.nameQuery ?? "",
      },
      filter: input.filter,
      offset: input.offset,
      limit: input.limit,
    };
  }

  return {
    mode,
    search: {
      query: input.query ?? "",
      exclude: input.excludeQuery,
      profile: input.searchProfile,
    },
    explain: input.explain,
    filter: input.filter,
    offset: input.offset,
    limit: input.limit,
  };
}

export function buildLookupRequest(name: string, options: LookupOptions = {}): SearchRequest {
  return {
    mode: "lookup",
    search: {
      query: name,
    },
    filter: buildAllOfFilter([
      options.pack
        ? {
            kind: "pack",
            value: options.pack,
          }
        : undefined,
      options.category ? buildScopeFilter(options.category, options.subcategory ?? null) : undefined,
    ]),
    limit: 5,
  };
}
