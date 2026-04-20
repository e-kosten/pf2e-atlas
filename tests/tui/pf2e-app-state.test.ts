import { describe, expect, it } from "vitest";

import {
  canPopPf2eAppRoute,
  createPf2eAppState,
  getCurrentPf2eAppRoute,
  pf2eAppReducer,
} from "../../src/tui/pf2e-app-state.js";

describe("pf2e app state", () => {
  it("pushes and pops nested routes with a stack", () => {
    const initial = createPf2eAppState();
    const pushedSearch = pf2eAppReducer(initial, {
      type: "push_route",
      route: { kind: "search" },
    });
    const pushedReview = pf2eAppReducer(pushedSearch, {
      type: "push_route",
      route: {
        kind: "review",
        session: {
          manifest: { id: "session-1" },
        } as never,
      },
    });

    expect(getCurrentPf2eAppRoute(pushedReview).kind).toBe("review");
    expect(canPopPf2eAppRoute(pushedReview)).toBe(true);

    const popped = pf2eAppReducer(pushedReview, { type: "pop_route" });
    expect(getCurrentPf2eAppRoute(popped)).toEqual({ kind: "search" });
  });

  it("keeps the root route when a pop is requested at depth one", () => {
    const initial = createPf2eAppState({ kind: "search" });
    const popped = pf2eAppReducer(initial, { type: "pop_route" });

    expect(getCurrentPf2eAppRoute(popped)).toEqual({ kind: "search" });
    expect(canPopPf2eAppRoute(popped)).toBe(false);
  });

  it("stores exact ontology return targets on search routes", () => {
    const ontologyRoute = {
      kind: "ontology" as const,
      model: {
        id: "searchSemantics",
        label: "Search Semantics",
        description: "Test ontology domain",
        rootNodes: [],
      },
      snapshot: {
        activePane: "detail" as const,
        browserState: {
          depth: 1,
          selectedNodeIds: ["root", "leaf"],
          filter: "rage",
          detailScroll: 3,
        },
        layoutMode: "split" as const,
        searchInput: "rage",
        searchMode: false,
      },
    };
    const state = createPf2eAppState(ontologyRoute);
    const next = pf2eAppReducer(state, {
      type: "push_route",
      route: {
        kind: "search",
        initialQuery: {
          kind: "listRecords",
          label: "Browse records with this value",
          filters: {
            category: "creature",
            limit: 20,
          },
        },
        origin: {
          kind: "ontology",
          route: ontologyRoute,
        },
      },
    });

    expect(getCurrentPf2eAppRoute(next)).toEqual({
      kind: "search",
      initialQuery: {
        kind: "listRecords",
        label: "Browse records with this value",
        filters: {
          category: "creature",
          limit: 20,
        },
      },
      origin: {
        kind: "ontology",
        route: ontologyRoute,
      },
    });
  });
});
