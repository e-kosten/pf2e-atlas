import { describe, expect, it } from "vitest";

import {
  buildLookupRequest,
  buildSearchRequestFromTransportInput,
} from "../../src/server/search-request-adapter.js";

describe("search request adapter", () => {
  it("builds browse requests without a search branch", () => {
    const request = buildSearchRequestFromTransportInput("browse", {
      query: "ghost sailor ship",
      searchProfile: "lexical",
      explain: true,
      filter: {
        kind: "scope",
        category: "creature",
        subcategory: { kind: "any" },
      },
      limit: 20,
      offset: 5,
    });

    expect(request).toEqual({
      mode: "browse",
      filter: {
        kind: "scope",
        category: "creature",
        subcategory: { kind: "any" },
      },
      limit: 20,
      offset: 5,
    });
    expect("search" in request).toBe(false);
    expect("explain" in request).toBe(false);
  });

  it("builds lookup requests from nameQuery only", () => {
    const request = buildSearchRequestFromTransportInput("lookup", {
      nameQuery: "Last Sentinel",
      query: "ignored broad query",
      filter: {
        kind: "scope",
        category: "creature",
        subcategory: { kind: "any" },
      },
      limit: 5,
    });

    expect(request).toEqual({
      mode: "lookup",
      search: {
        query: "Last Sentinel",
      },
      filter: {
        kind: "scope",
        category: "creature",
        subcategory: { kind: "any" },
      },
      limit: 5,
      offset: undefined,
    });
  });

  it("builds search requests with the search branch fields only", () => {
    const request = buildSearchRequestFromTransportInput("search", {
      query: "ghost sailor ship",
      excludeQuery: "harbor",
      searchProfile: "concept",
      explain: true,
      limit: 10,
      offset: 2,
    });

    expect(request).toEqual({
      mode: "search",
      search: {
        query: "ghost sailor ship",
        exclude: "harbor",
        profile: "concept",
      },
      explain: true,
      filter: undefined,
      limit: 10,
      offset: 2,
    });
  });

  it("builds canonical lookup requests with pack and scope filters", () => {
    expect(
      buildLookupRequest("Raise Shield", {
        pack: "actions",
        category: "rule",
        subcategory: "action",
      }),
    ).toEqual({
      mode: "lookup",
      search: {
        query: "Raise Shield",
      },
      filter: {
        kind: "allOf",
        children: [
          {
            kind: "pack",
            value: "actions",
          },
          {
            kind: "scope",
            category: "rule",
            subcategory: { kind: "eq", value: "action" },
          },
        ],
      },
      limit: 5,
    });
  });
});
