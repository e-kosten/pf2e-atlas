import { describe, expect, it } from "vitest";

import type { MetadataFilterNode } from "../../src/search/filters/types.js";
import {
  createDefaultQuery,
  setSearchQueryCategory,
  setSearchQueryMetadataTree,
  setSearchQueryPart,
} from "../../src/tui/search/query-state.js";
import { createInitialSearchScreenState, SEARCH_COUNT_STATUS, type SearchCountState } from "../../src/tui/search-screen/state.js";
import {
  buildWorkspaceEntries,
} from "../../src/tui/search-screen/workspace/workspace.js";
import {
  buildSearchQuerySummary,
  getVisibleSearchQuerySummaryEntries,
} from "../../src/tui/search-screen/workspace/query-summary.js";

function createMetadataTree(): MetadataFilterNode {
  return {
    or: [
      {
        field: "traits",
        op: "includesAny",
        values: ["ghost"],
      },
      {
        not: {
          field: "publicationRemaster",
          op: "eq",
          value: true,
        },
      },
    ],
  };
}

function createStructuredQuery(options: { includeRarity?: boolean } = {}) {
  let query = {
    ...createDefaultQuery(),
    mode: "search" as const,
    queryText: "ghost captain",
    searchProfile: "concept" as const,
    sourceLabel: "Ontology > Creature > Ghost",
  };
  query = setSearchQueryCategory(query, "creature");
  query = setSearchQueryPart(query, {
    kind: "levelRange",
    levelMin: 5,
    levelMax: 7,
  });
  if (options.includeRarity ?? true) {
    query = setSearchQueryPart(query, {
      kind: "rarityPolicy",
      policy: {
        any: ["rare"],
        all: [],
        exclude: [],
      },
    });
  }
  query = setSearchQueryPart(query, {
    kind: "actionCostPolicy",
    policy: {
      any: [2],
      all: [],
      exclude: [],
    },
  });
  return setSearchQueryMetadataTree(query, createMetadataTree());
}

function createIdleCountState(): SearchCountState {
  return {
    status: SEARCH_COUNT_STATUS.IDLE,
    result: null,
    message: null,
  };
}

describe("search query summary", () => {
  it("builds stable structured summary entries with visible row semantics", () => {
    const summary = buildSearchQuerySummary(createStructuredQuery());
    const visibleEntries = getVisibleSearchQuerySummaryEntries(summary);

    expect(summary.sourceLabel).toBe("Ontology > Creature > Ghost");
    expect(summary.activeStructuredPartCount).toBe(5);
    expect(summary.metadataPredicateCount).toBe(2);
    expect(visibleEntries.map((entry) => entry.key)).toEqual([
      "mode",
      "query",
      "profile",
      "queryPart:category",
      "queryPart:levelRange",
      "queryPart:rarity",
      "queryPart:actionCost",
      "queryNode:root",
      "queryNode:0",
      "queryNode:1",
      "queryNode:1.0",
    ]);
    expect(visibleEntries.find((entry) => entry.key === "queryPart:category")).toMatchObject({
      anchor: { kind: "queryPart", part: "category" },
      value: "Creature",
    });
    expect(visibleEntries.find((entry) => entry.key === "queryNode:1.0")).toMatchObject({
      anchor: { kind: "queryNode", path: [1, 0] },
      label: "Publication Remaster",
    });
  });

  it("keeps metadata anchors stable when unrelated visible sections appear", () => {
    const withoutRarity = buildSearchQuerySummary(createStructuredQuery({ includeRarity: false }));
    const withRarity = buildSearchQuerySummary(createStructuredQuery({ includeRarity: true }));

    expect(
      getVisibleSearchQuerySummaryEntries(withoutRarity)
        .filter((entry) => entry.kind === "metadata")
        .map((entry) => ({ key: entry.key, anchor: entry.anchor })),
    ).toEqual(
      getVisibleSearchQuerySummaryEntries(withRarity)
        .filter((entry) => entry.kind === "metadata")
        .map((entry) => ({ key: entry.key, anchor: entry.anchor })),
    );
  });

  it("drives workspace structured rows from the same summary model", () => {
    const query = createStructuredQuery();
    const summary = buildSearchQuerySummary(query);
    const state = createInitialSearchScreenState(query);
    const workspaceEntries = buildWorkspaceEntries(state, createIdleCountState());

    expect(
      workspaceEntries
        .filter(
          (entry) =>
            entry.action === "profile" ||
            entry.action === "clearClauses" ||
            entry.action.startsWith("queryPart:") ||
            entry.action.startsWith("queryNode:"),
        )
        .map((entry) => entry.label),
    ).toEqual([
      ...getVisibleSearchQuerySummaryEntries(summary)
        .filter((entry) => entry.kind !== "mode" && entry.kind !== "query")
        .map((entry) => entry.label),
      "Clear Query Clauses",
    ]);
    expect(workspaceEntries.find((entry) => entry.action === "addQueryPart")?.value).toBe("5 active");
  });
});
