import { describe, expect, it } from "vitest";

import { getSearchQueryCategory } from "../../src/tui/search/query-state.js";
import {
  applyGroupedFieldSeedQueryToQuery,
  buildGroupedFieldSeedState,
} from "../../src/tui/search-screen/structured-draft/structured-draft-metadata-actions.js";
import {
  allOfFilter,
  anyOfFilter,
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
          metadataPredicateFilter({ field: "traits", op: "includes", value: "chaotic" }),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
        ]),
      ]),
      limit: 20,
    }).request;

    const { initialDraft, seedGroupPath, seedQuery } = buildGroupedFieldSeedState(query, [1], {
      field: "traits",
      fieldMemberPaths: [[1, 0], [1, 1]],
    });

    expect(getSearchQueryCategory(seedQuery)).toBe("creature");
    expect(seedGroupPath).toEqual([1]);
    expect(seedQuery.filter).toEqual({
      kind: "allOf",
      children: [scopeFilter("creature")],
    });
    expect(initialDraft.discreteClauses).toEqual([
      { field: "traits", value: "chaotic", operator: "include" },
      { field: "traits", value: "evil", operator: "include" },
    ]);
  });

  it("replaces the active group in place when applying grouped explorer edits", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "includes", value: "chaotic" }),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
        ]),
      ]),
      limit: 20,
    }).request;

    const { seedGroupPath } = buildGroupedFieldSeedState(query, [1], {
      field: "traits",
      fieldMemberPaths: [[1, 0], [1, 1]],
    });
    const nextSeedQuery = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
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
          metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
        ]),
      ]),
    );
    expect(nextFocusPath).toEqual([1]);
  });

  it("keeps non-field siblings in the grouped seed while lifting the target field into the explorer draft", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "includes", value: "chaotic" }),
          { kind: "pack", value: "monster-core" },
          {
            kind: "not",
            child: metadataPredicateFilter({ field: "traits", op: "includes", value: "unholy" }),
          },
        ]),
      ]),
      limit: 20,
    }).request;

    const { initialDraft, seedQuery } = buildGroupedFieldSeedState(query, [1], {
      field: "traits",
      fieldMemberPaths: [[1, 0], [1, 2]],
    });

    expect(seedQuery.filter).toEqual(
      allOfFilter([
        scopeFilter("creature"),
        { kind: "pack", value: "monster-core" },
      ]),
    );
    expect(initialDraft.discreteClauses).toEqual([
      { field: "traits", value: "chaotic", operator: "include" },
      { field: "traits", value: "unholy", operator: "exclude" },
    ]);
  });

  it("lifts grouped metadata any-of members into the explorer draft instead of dropping them", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          anyOfFilter([
            metadataPredicateFilter({ field: "traits", op: "includes", value: "chaotic" }),
            metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
          ]),
          { kind: "pack", value: "monster-core" },
        ]),
      ]),
      limit: 20,
    }).request;

    const { initialDraft, preservedMetadata, seedQuery } = buildGroupedFieldSeedState(query, [1], {
      field: "traits",
      fieldMemberPaths: [[1, 0], [1, 0]],
    });

    expect(seedQuery.filter).toEqual(
      allOfFilter([
        scopeFilter("creature"),
        { kind: "pack", value: "monster-core" },
      ]),
    );
    expect(initialDraft.discreteClauses).toEqual([
      { field: "traits", value: "chaotic", operator: "include" },
      { field: "traits", value: "evil", operator: "include" },
    ]);
    expect(preservedMetadata).toBeNull();
  });

  it("preserves outer group context when seeding a nested grouped field editor", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          anyOfFilter([
            metadataPredicateFilter({ field: "traits", op: "includes", value: "chaotic" }),
            metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
          ]),
          metadataPredicateFilter({ field: "derivedTags", op: "includes", value: "coastal_setting" }),
          { kind: "pack", value: "monster-core" },
        ]),
      ]),
      limit: 20,
    }).request;

    const { initialDraft, preservedMetadata, seedGroupPath, seedQuery } = buildGroupedFieldSeedState(query, [1, 0], {
      field: "traits",
      fieldMemberPaths: [[1, 0, 0], [1, 0, 1]],
    });

    expect(seedGroupPath).toEqual([1, 0]);
    expect(seedQuery.filter).toEqual(
      allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          metadataPredicateFilter({ field: "derivedTags", op: "includes", value: "coastal_setting" }),
          { kind: "pack", value: "monster-core" },
        ]),
      ]),
    );
    expect(initialDraft.discreteClauses).toEqual([
      { field: "traits", value: "chaotic", operator: "include" },
      { field: "traits", value: "evil", operator: "include" },
    ]);
    expect(preservedMetadata).toEqual({
      field: "derivedTags",
      op: "includes",
      value: "coastal_setting",
    });
  });
});
