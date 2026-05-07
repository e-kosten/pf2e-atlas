import { describe, expect, it } from "vitest";

import type { SearchFilterNode } from "../../src/domain/search-request-types.js";
import {
  createDefaultQuery,
  setSearchQueryActionCostSelection,
  setSearchQueryExcludeText,
  setSearchQueryLevelRange,
  setSearchQuerySearchProfile,
  setSearchQueryCategory,
  setSearchQueryPredicateFilter,
  setSearchQueryRaritySelection,
  setSearchQueryText,
} from "../../src/tui/search/query-state.js";
import { createInitialSearchScreenState, SEARCH_COUNT_STATUS, type SearchCountState } from "../../src/tui/search-screen/state.js";
import {
  buildWorkspaceEntries,
} from "../../src/tui/search-screen/workspace/workspace.js";
import {
  buildSearchQuerySummary,
  formatLevelRange,
  getVisibleSearchQuerySummaryEntries,
} from "../../src/tui/search-screen/workspace/query-summary.js";

function createMetadataTree(): SearchFilterNode {
  return {
    kind: "anyOf",
    children: [
      {
        kind: "metadataPredicate",
        predicate: {
          field: "traits",
          op: "includes",
          value: "ghost",
        },
      },
      {
        kind: "not",
        child: {
          kind: "metadataPredicate",
          predicate: {
            field: "publicationRemaster",
            op: "eq",
            value: true,
          },
        },
      },
    ],
  };
}

function createStructuredQuery(options: { includeRarity?: boolean } = {}) {
  let query = createDefaultQuery("search");
  query = setSearchQueryText(query, "ghost captain");
  query = setSearchQueryExcludeText(query, "skeleton");
  query = setSearchQuerySearchProfile(query, "concept");
  query = setSearchQueryCategory(query, "creature");
  query = setSearchQueryLevelRange(query, {
    levelMin: 5,
    levelMax: 7,
  });
  if (options.includeRarity ?? true) {
    query = setSearchQueryRaritySelection(query, {
      include: ["rare"],
      exclude: [],
    });
  }
  query = setSearchQueryActionCostSelection(query, {
    include: [2],
    exclude: [],
  });
  return setSearchQueryPredicateFilter(query, createMetadataTree());
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

    expect(summary.activeStructuredPartCount).toBe(5);
    expect(summary.metadataPredicateCount).toBe(2);
    expect(visibleEntries.map((entry) => entry.key)).toEqual([
      "mode",
      "query",
      "exclude",
      "profile",
      "queryTree:root",
      "queryNode:0",
      "queryNode:1",
      "queryNode:2",
      "queryNode:3",
      "queryNode:4",
    ]);
    expect(visibleEntries.find((entry) => entry.key === "queryTree:root")).toMatchObject({
      anchor: { kind: "queryTreeRoot" },
      label: "Query Logic",
      value: "All of (5 filters)",
    });
    expect(visibleEntries.find((entry) => entry.key === "exclude")).toMatchObject({
      anchor: { kind: "exclude" },
      label: "Exclude",
      value: "skeleton",
    });
    expect(visibleEntries.find((entry) => entry.key === "queryNode:0")).toMatchObject({
      anchor: { kind: "queryNode", path: [0] },
      label: "Filter",
      value: "Scope: Creature",
    });
    expect(visibleEntries.find((entry) => entry.key === "queryNode:4")).toMatchObject({
      anchor: { kind: "queryNode", path: [4] },
      label: "Filter",
      value: "Any of (2 filters)",
    });
  });

  it("updates the root-tree projection when canonical children change", () => {
    const withoutRarity = buildSearchQuerySummary(createStructuredQuery({ includeRarity: false }));
    const withRarity = buildSearchQuerySummary(createStructuredQuery({ includeRarity: true }));

    expect(withoutRarity.activeStructuredPartCount).toBe(4);
    expect(withRarity.activeStructuredPartCount).toBe(5);
    expect(getVisibleSearchQuerySummaryEntries(withoutRarity).find((entry) => entry.key === "queryTree:root")?.value).toBe(
      "All of (4 filters)",
    );
    expect(getVisibleSearchQuerySummaryEntries(withRarity).find((entry) => entry.key === "queryTree:root")?.value).toBe(
      "All of (5 filters)",
    );
  });

  it("preserves non-scope filters when clearing category back to any category", () => {
    const clearedCategory = setSearchQueryCategory(createStructuredQuery(), null);
    const summary = buildSearchQuerySummary(clearedCategory);
    const visibleEntries = getVisibleSearchQuerySummaryEntries(summary);

    expect(summary.activeStructuredPartCount).toBe(4);
    expect(visibleEntries.find((entry) => entry.key === "queryTree:root")?.value).toBe("All of (4 filters)");
    expect(visibleEntries.find((entry) => entry.key === "queryNode:0")?.value).toBe("Level: 5-7");
    expect(visibleEntries.find((entry) => entry.key === "queryNode:3")?.value).toBe("Any of (2 filters)");
    expect(visibleEntries.some((entry) => entry.value === "Scope: Creature")).toBe(false);
  });

  it("preserves strict level matchers in canonical summaries", () => {
    let query = createDefaultQuery("search");
    query = setSearchQueryCategory(query, "creature");
    query = setSearchQueryLevelRange(query, { kind: "gt", value: 5 });

    const summary = buildSearchQuerySummary(query);
    const visibleEntries = getVisibleSearchQuerySummaryEntries(summary);

    expect(visibleEntries.find((entry) => entry.key === "queryNode:1")?.value).toBe("Level: > 5");
    expect(formatLevelRange(query)).toBe("> L5");
  });

  it("drives workspace structured rows from the same summary model", () => {
    const query = createStructuredQuery();
    const state = createInitialSearchScreenState(query);
    const workspaceEntries = buildWorkspaceEntries(state, createIdleCountState());

    expect(
      workspaceEntries.map((entry) => entry.label),
    ).toEqual([
      "Mode",
      "Query",
      "Exclude",
      "Profile",
      "Filters >",
      "Filter",
      "Filter",
      "Filter",
      "Filter",
      "Filter",
      "Clear Query Clauses",
      "Reset Query",
      "Discard Applied Results",
      "Execute Query",
    ]);
    expect(workspaceEntries.findIndex((entry) => entry.action === "profile")).toBeLessThan(
      workspaceEntries.findIndex((entry) => entry.action === "addQueryPart"),
    );
    expect(workspaceEntries.find((entry) => entry.action === "addQueryPart")?.value).toBe("5 active");
  });
});
