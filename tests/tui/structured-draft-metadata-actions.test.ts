import { describe, expect, it } from "vitest";

import { getSearchQueryCategory } from "../../src/tui/search/query-state.js";
import {
  applyGroupedFieldReplacementToQuery,
  buildGroupedFieldSeedState,
} from "../../src/tui/search-screen/structured-draft/structured-draft-metadata-actions.js";
import {
  allOfFilter,
  anyOfFilter,
  browseQuery,
  metadataPredicateFilter,
  notFilter,
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

    const { initialFieldState, seedGroupPath, seedQuery } = buildGroupedFieldSeedState(query, [1], {
      field: "traits",
      fieldMemberPaths: [[1, 0], [1, 1]],
    });

    expect(getSearchQueryCategory(seedQuery)).toBe("creature");
    expect(seedGroupPath).toEqual([1]);
    expect(seedQuery.filter).toEqual({
      kind: "allOf",
      children: [scopeFilter("creature")],
    });
    expect(initialFieldState.discreteSelections).toEqual({
      traits: {
        include: ["chaotic", "evil"],
        exclude: [],
      },
    });
    expect(initialFieldState.scalarClauses).toEqual({});
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

    const { nextFocusPath, nextQuery } = applyGroupedFieldReplacementToQuery(
      query,
      [1],
      "traits",
      [[1, 0], [1, 1]],
      [
        metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
      ],
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
    expect(nextFocusPath).toEqual([1, 0]);
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

    const { initialFieldState, seedQuery } = buildGroupedFieldSeedState(query, [1], {
      field: "traits",
      fieldMemberPaths: [[1, 0], [1, 2]],
    });

    expect(seedQuery.filter).toEqual(
      allOfFilter([
        scopeFilter("creature"),
        { kind: "pack", value: "monster-core" },
      ]),
    );
    expect(initialFieldState.discreteSelections).toEqual({
      traits: {
        include: ["chaotic"],
        exclude: ["unholy"],
      },
    });
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

    const { initialFieldState, preservedMetadata, seedQuery } = buildGroupedFieldSeedState(query, [1], {
      field: "traits",
      fieldMemberPaths: [[1, 0], [1, 0]],
    });

    expect(seedQuery.filter).toEqual(
      allOfFilter([
        scopeFilter("creature"),
        { kind: "pack", value: "monster-core" },
      ]),
    );
    expect(initialFieldState.discreteSelections).toEqual({
      traits: {
        include: ["chaotic", "evil"],
        exclude: [],
      },
    });
    expect(preservedMetadata).toBeNull();
  });

  it("replaces grouped field children directly instead of introducing nested all-of wrappers", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
        ]),
      ]),
      limit: 20,
    }).request;

    const { nextQuery } = applyGroupedFieldReplacementToQuery(
      query,
      [1],
      "traits",
      [[1, 0], [1, 1]],
      [
        metadataPredicateFilter({ field: "traits", op: "includes", value: "chaotic" }),
        notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" })),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
        notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "unholy" })),
      ],
    );

    expect(nextQuery.filter).toEqual(
      allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "includes", value: "chaotic" }),
          notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" })),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
          notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "unholy" })),
        ]),
      ]),
    );
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

    const { initialFieldState, preservedMetadata, seedGroupPath, seedQuery } = buildGroupedFieldSeedState(query, [1, 0], {
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
    expect(initialFieldState.discreteSelections).toEqual({
      traits: {
        include: ["chaotic", "evil"],
        exclude: [],
      },
    });
    expect(preservedMetadata).toEqual({
      field: "derivedTags",
      op: "includes",
      value: "coastal_setting",
    });
  });

  it("replaces direct action-cost shared-explorer leaf edits through grouped field mutation semantics", () => {
    const query = browseQuery("Browse actions", {
      filter: allOfFilter([scopeFilter("rule", "action"), { kind: "actionCost", match: { kind: "eq", value: 1 } }]),
      limit: 20,
    }).request;

    const { initialFieldState, seedQuery } = buildGroupedFieldSeedState(query, [], {
      field: "actionCost",
      fieldMemberPaths: [[1]],
    });
    expect(seedQuery.filter).toEqual(scopeFilter("rule", "action"));
    expect(initialFieldState.discreteSelections).toEqual({
      actionCost: {
        include: ["1"],
        exclude: [],
      },
    });

    const { nextQuery } = applyGroupedFieldReplacementToQuery(query, [], "actionCost", [[1]], [
      { kind: "actionCost", match: { kind: "eq", value: 2 } },
    ]);

    expect(nextQuery.filter).toEqual(
      allOfFilter([scopeFilter("rule", "action"), { kind: "actionCost", match: { kind: "eq", value: 2 } }]),
    );
  });
});
