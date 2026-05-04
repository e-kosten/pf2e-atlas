import { describe, expect, it } from "vitest";

import { compileSearchRequest } from "../../src/search/request-compilation.js";
import { browseRequest, linkedFromFilter } from "../helpers/search-request-fixture.js";

describe("search request compilation", () => {
  it("passes linkedFrom through to the execution filter contract", () => {
    const compiled = compileSearchRequest(
      browseRequest({
        filter: linkedFromFilter("actions:action-refocus-1"),
        sort: { kind: "alphabetical" },
        limit: 20,
        offset: 3,
      }),
    );

    expect(compiled).toEqual({
      sort: "alphabetical",
      explain: undefined,
      nameQuery: undefined,
      query: undefined,
      excludeQuery: undefined,
      searchProfile: undefined,
      filter: {
        kind: "linkedFrom",
        source: "actions:action-refocus-1",
      },
      offset: 3,
      limit: 20,
    });
  });
});
