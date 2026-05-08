import {
  buildAllOfFilter,
  buildScopeFilter,
  type SearchRequest,
  SEARCH_REQUEST_VOCABULARY,
} from "../domain/search-request-types.js";
import type { LookupOptions } from "../domain/search-types.js";

export type SearchRequestTransportInput = SearchRequest;

export function buildSearchRequestFromTransportInput(input: SearchRequestTransportInput): SearchRequest {
  return input;
}

export function buildLookupRequest(name: string, options: LookupOptions = {}): SearchRequest {
  return {
    mode: SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP,
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
