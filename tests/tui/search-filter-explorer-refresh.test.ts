import { describe, expect, it } from "vitest";

import type { OntologyDomainModel } from "../../src/domain/ontology-types.js";
import {
  createSearchFilterExplorerLoadingModel,
  isSearchFilterExplorerLoadingModel,
} from "../../src/tui/search-screen/filter-explorer-loading-model.js";
import {
  planSearchFilterExplorerRefresh,
  shouldApplySearchFilterExplorerRefresh,
} from "../../src/tui/search-screen/filter-explorer-refresh.js";

function createExplorerDomain(label: string): OntologyDomainModel {
  return {
    id: "searchSemantics",
    label,
    description: `${label} explorer`,
    rootNodes: [
      {
        id: `${label.toLowerCase()}:node`,
        kind: "group",
        label,
        filterText: label.toLowerCase(),
        detailTitle: label,
        detailLines: [{ text: label }],
      },
    ],
  };
}

describe("search filter explorer refresh planning", () => {
  it("keeps the loading placeholder local to the screen owner", () => {
    const loadingModel = createSearchFilterExplorerLoadingModel("Derived Tags Explorer");
    const loadedModel = createExplorerDomain("Derived Tags");

    expect(loadingModel.id).toBe("searchSemantics");
    expect(isSearchFilterExplorerLoadingModel(loadingModel)).toBe(true);
    expect(isSearchFilterExplorerLoadingModel(loadedModel)).toBe(false);
  });

  it("invalidates queued refreshes when switching back to the currently displayed mode", () => {
    const cache = new Map([
      ["matching", createExplorerDomain("Matching")] as const,
    ]);
    const queuedCatalogPlan = planSearchFilterExplorerRefresh({
      nextMode: "catalog",
      displayedMode: "matching",
      cache,
      currentRequestId: 0,
    });
    expect(queuedCatalogPlan).toEqual({
      kind: "load",
      pendingMode: "catalog",
      requestId: 1,
    });

    const revertPlan = planSearchFilterExplorerRefresh({
      nextMode: "matching",
      displayedMode: "matching",
      cache,
      currentRequestId: queuedCatalogPlan.requestId,
    });
    expect(revertPlan).toEqual({
      kind: "retainCurrent",
      mode: "matching",
      requestId: 2,
    });
    expect(
      shouldApplySearchFilterExplorerRefresh({
        currentRequestId: revertPlan.requestId,
        completedRequestId: queuedCatalogPlan.requestId,
      }),
    ).toBe(false);
  });

  it("invalidates in-flight refreshes when switching to a cached mode", () => {
    const cache = new Map([
      ["matching", createExplorerDomain("Matching")] as const,
      ["catalog", createExplorerDomain("Catalog")] as const,
    ]);
    const inFlightCatalogRefresh = planSearchFilterExplorerRefresh({
      nextMode: "catalog",
      displayedMode: "matching",
      cache: new Map([["matching", cache.get("matching")!]]),
      currentRequestId: 0,
    });
    expect(inFlightCatalogRefresh).toEqual({
      kind: "load",
      pendingMode: "catalog",
      requestId: 1,
    });

    const cachedMatchingPlan = planSearchFilterExplorerRefresh({
      nextMode: "matching",
      displayedMode: "catalog",
      cache,
      currentRequestId: inFlightCatalogRefresh.requestId,
    });
    expect(cachedMatchingPlan).toEqual({
      kind: "useCached",
      mode: "matching",
      model: cache.get("matching"),
      requestId: 2,
    });
    expect(
      shouldApplySearchFilterExplorerRefresh({
        currentRequestId: cachedMatchingPlan.requestId,
        completedRequestId: inFlightCatalogRefresh.requestId,
      }),
    ).toBe(false);
  });
});
