import { describe, expect, it } from "vitest";

import { getSearchQueryCategory } from "../../src/tui/search/query-state.js";
import {
  applyGroupedFieldSeedQueryToQuery,
  buildGroupedFieldSeedState,
} from "../../src/tui/search-screen/structured-draft/structured-draft-metadata-actions.js";
import {
  allOfFilter,
  browseQuery,
  metadataPredicateFilter,
  scopeFilter,
} from "../helpers/search-request-fixture.js";

describe("structured draft grouped explorer helpers", () => {
  it("preserves the query scope when seeding grouped field explorer edits", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "contains", value: "chaotic" }),
          metadataPredicateFilter({ field: "traits", op: "contains", value: "evil" }),
        ]),
      ]),
      limit: 20,
    }).request;

    const { seedGroupPath, seedQuery } = buildGroupedFieldSeedState(query, [1]);

    expect(getSearchQueryCategory(seedQuery)).toBe("creature");
    expect(seedGroupPath).toEqual([1]);
    expect(seedQuery.filter).toEqual(
      allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "contains", value: "chaotic" }),
          metadataPredicateFilter({ field: "traits", op: "contains", value: "evil" }),
        ]),
      ]),
    );
  });

  it("replaces the active group in place when applying grouped explorer edits", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "contains", value: "chaotic" }),
          metadataPredicateFilter({ field: "traits", op: "contains", value: "evil" }),
        ]),
      ]),
      limit: 20,
    }).request;

    const { seedGroupPath } = buildGroupedFieldSeedState(query, [1]);
    const nextSeedQuery = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "contains", value: "evil" }),
          metadataPredicateFilter({ field: "traits", op: "contains", value: "humanoid" }),
        ]),
      ]),
      limit: 20,
    }).request;

    const { nextFocusPath, nextQuery } = applyGroupedFieldSeedQueryToQuery(
      query,
      [1],
      seedGroupPath,
      "traits",
      nextSeedQuery,
    );

    expect(nextQuery.filter).toEqual(
      allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "contains", value: "evil" }),
          metadataPredicateFilter({ field: "traits", op: "contains", value: "humanoid" }),
        ]),
      ]),
    );
    expect(nextFocusPath).toEqual([1]);
  });
});
