import { describe, expect, it } from "vitest";

import {
  buildLookupRequest,
  buildSearchRequestFromTransportInput,
} from "../../src/server/search-request-adapter.js";

describe("search request adapter", () => {
  it("passes through canonical browse requests", () => {
    const request = buildSearchRequestFromTransportInput({
      mode: "browse",
      filter: {
        kind: "allOf",
        children: [
          {
            kind: "scope",
            category: "creature",
            subcategory: { kind: "any" },
          },
          {
            kind: "linkedFrom",
            source: "actions:action-refocus-1",
          },
        ],
      },
      sort: { kind: "alphabetical" },
      limit: 20,
      offset: 5,
    });

    expect(request).toEqual({
      mode: "browse",
      filter: {
        kind: "allOf",
        children: [
          {
            kind: "scope",
            category: "creature",
            subcategory: { kind: "any" },
          },
          {
            kind: "linkedFrom",
            source: "actions:action-refocus-1",
          },
        ],
      },
      sort: { kind: "alphabetical" },
      limit: 20,
      offset: 5,
    });
    expect("search" in request).toBe(false);
    expect("explain" in request).toBe(false);
  });

  it("passes through canonical search requests", () => {
    const request = buildSearchRequestFromTransportInput({
      mode: "search",
      search: {
        query: "ghost sailor ship",
        exclude: "harbor",
        profile: "concept",
      },
      filter: {
        kind: "linkedFrom",
        source: "actionspf2e:action-track-1",
      },
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
      filter: {
        kind: "linkedFrom",
        source: "actionspf2e:action-track-1",
      },
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
