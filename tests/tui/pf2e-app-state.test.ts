import { describe, expect, it } from "vitest";

import {
  PF2E_APP_ROUTE_KIND,
  PF2E_SEARCH_ROUTE_ENTRY_KIND,
  PF2E_SEARCH_ROUTE_ORIGIN_KIND,
  canPopPf2eAppRoute,
  createPf2eOntologyRoute,
  createPf2eSearchEditorRoute,
  createPf2eSearchResultsRoute,
  createPf2eAppState,
  getCurrentPf2eAppRoute,
  pf2eAppReducer,
} from "../../src/tui/pf2e-app-state.js";
import { browseQuery, scopeFilter } from "../helpers/search-request-fixture.js";

describe("pf2e app state", () => {
  it("pushes and pops nested routes with a stack", () => {
    const initial = createPf2eAppState();
    const pushedSearch = pf2eAppReducer(initial, {
      type: "push_route",
      route: createPf2eSearchEditorRoute(),
    });
    const pushedReview = pf2eAppReducer(pushedSearch, {
      type: "push_route",
      route: {
        kind: PF2E_APP_ROUTE_KIND.REVIEW,
        session: {
          manifest: { id: "session-1" },
        } as never,
      },
    });

    expect(getCurrentPf2eAppRoute(pushedReview).kind).toBe("review");
    expect(canPopPf2eAppRoute(pushedReview)).toBe(true);

    const popped = pf2eAppReducer(pushedReview, { type: "pop_route" });
    expect(getCurrentPf2eAppRoute(popped)).toEqual(createPf2eSearchEditorRoute());
  });

  it("keeps the root route when a pop is requested at depth one", () => {
    const initial = createPf2eAppState(createPf2eSearchEditorRoute());
    const popped = pf2eAppReducer(initial, { type: "pop_route" });

    expect(getCurrentPf2eAppRoute(popped)).toEqual(createPf2eSearchEditorRoute());
    expect(canPopPf2eAppRoute(popped)).toBe(false);
  });

  it("stores exact ontology return targets on search routes", () => {
    const ontologyRoute = createPf2eOntologyRoute({
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
    });
    const state = createPf2eAppState(ontologyRoute);
    const next = pf2eAppReducer(state, {
      type: "push_route",
      route: createPf2eSearchEditorRoute({
        initialQuery: browseQuery("Browse records with this value", {
          filter: scopeFilter("creature"),
          limit: 20,
        }),
        origin: {
          kind: PF2E_SEARCH_ROUTE_ORIGIN_KIND.ONTOLOGY,
          route: ontologyRoute,
        },
      }),
    });

    expect(getCurrentPf2eAppRoute(next)).toEqual({
      kind: PF2E_APP_ROUTE_KIND.SEARCH,
      entry: PF2E_SEARCH_ROUTE_ENTRY_KIND.EDITOR,
      initialQuery: browseQuery("Browse records with this value", {
        filter: scopeFilter("creature"),
        limit: 20,
      }),
      origin: {
        kind: PF2E_SEARCH_ROUTE_ORIGIN_KIND.ONTOLOGY,
        route: ontologyRoute,
      },
    });
  });

  it("keeps results routes explicit with a required prepared session", () => {
    const initialSession = {
      windowId: "window-1",
      query: {} as never,
      results: [],
      windowOffset: 0,
      resultMode: "structured",
      total: 0,
      loadedCount: 0,
      hasMore: false,
      nextOffset: null,
      searchProfile: null,
      sort: "alphabetical",
      sortSeed: null,
    } as const;

    const state = createPf2eAppState();
    const next = pf2eAppReducer(state, {
      type: "push_route",
      route: createPf2eSearchResultsRoute({
        initialSession,
      }),
    });

    expect(getCurrentPf2eAppRoute(next)).toEqual({
      kind: PF2E_APP_ROUTE_KIND.SEARCH,
      entry: PF2E_SEARCH_ROUTE_ENTRY_KIND.RESULTS,
      initialSession,
    });
  });
});
