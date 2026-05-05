import { describe, expect, it } from "vitest";

import type { MetadataFilterNode } from "../../src/tui/search/metadata-filter-draft.js";
import { getSearchQueryCategory, setSearchQueryMetadataTree } from "../../src/tui/search/query-state.js";
import { buildSearchFilterExplorerFieldState } from "../../src/tui/search-screen/filter-explorer-field-state.js";
import {
  buildGroupedFieldReplacementNodes,
  buildGroupedFieldSeedState,
  type StructuredDraftGroupedFieldSearchAdapter,
} from "../../src/tui/search-screen/structured-draft/structured-draft-grouped-field.js";
import {
  allOfFilter,
  anyOfFilter,
  browseQuery,
  metadataPredicateFilter,
  notFilter,
  scopeFilter,
} from "../helpers/search-request-fixture.js";

describe("structured draft grouped-field helpers", () => {
  const groupedFieldSearchAdapter: StructuredDraftGroupedFieldSearchAdapter = {
    applyDiscoverableQueryFieldSelections: (query, selections) => {
      const selection = selections.traits ?? { include: [], exclude: [] };
      const clauses: MetadataFilterNode[] = [
        ...selection.include.map(
          (value): MetadataFilterNode => ({ field: "traits", op: "includes", value }),
        ),
        ...selection.exclude.map(
          (value): MetadataFilterNode => ({
            not: { field: "traits", op: "includes", value },
          }),
        ),
      ];
      return setSearchQueryMetadataTree(query, clauses.length === 1 ? clauses[0]! : { and: clauses });
    },
  };

  it("emits flat set-field replacement clauses when adding a new grouped field", () => {
    const query = browseQuery("Browse creatures", {
      filter: scopeFilter("creature"),
      limit: 20,
    }).request;
    const fieldState = buildSearchFilterExplorerFieldState({
      discreteClauses: [
        { field: "traits", value: "evil", operator: "include" },
        { field: "traits", value: "humanoid", operator: "include" },
        { field: "traits", value: "unholy", operator: "exclude" },
      ],
      scalarClauses: {},
    });

    expect(
      buildGroupedFieldReplacementNodes(groupedFieldSearchAdapter, query, fieldState, {
        value: "traits",
        label: "Traits",
        description: "Trait query field for the current browse scope.",
        fieldType: "set",
        editor: "sharedExplorer",
      }),
    ).toEqual([
      metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
      metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
      notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "unholy" })),
    ]);
  });

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

  it("seeds direct action-cost shared-explorer leaf edits from canonical action-cost clauses", () => {
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

  });
});
