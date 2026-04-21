import React from "react";
import { act, create, type ReactTestRenderer } from "react-test-renderer";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { OntologyDomainModel, OntologyNodeQuery } from "../../src/domain/ontology-types.js";
import type { Pf2eTerminalAppServices } from "../../src/tui/app-services.js";
import {
  PF2E_APP_AREA_ID,
  PF2E_APP_ROUTE_KIND,
  PF2E_SEARCH_ROUTE_ENTRY_KIND,
  PF2E_SEARCH_ROUTE_ORIGIN_KIND,
  createPf2eAppState,
  createPf2eOntologyRoute,
  pf2eAppReducer,
  type Pf2eAppRoute,
  type Pf2eAppState,
} from "../../src/tui/pf2e-app-state.js";
import {
  PF2E_ONTOLOGY_SEARCH_INTENT_KIND,
  PF2E_NAVIGATION_MESSAGE,
  usePf2eNavigation,
} from "../../src/tui/pf2e-navigation.js";
import { ROUTE_TRANSITION_STATUS_KIND } from "../../src/tui/route-transition-status.js";

function createDeferred<T>(): {
  promise: Promise<T>;
  resolve: (value: T) => void;
  reject: (reason?: unknown) => void;
} {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((promiseResolve, promiseReject) => {
    resolve = promiseResolve;
    reject = promiseReject;
  });
  return { promise, resolve, reject };
}

async function flushReact(): Promise<void> {
  await new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
  await Promise.resolve();
  await Promise.resolve();
}

function createOntologyModel(): OntologyDomainModel {
  return {
    id: "searchSemantics",
    label: "Search Semantics",
    description: "Search semantics ontology",
    rootNodes: [],
  };
}

function createOntologyQuery(overrides: Partial<OntologyNodeQuery> = {}): OntologyNodeQuery {
  return {
    kind: "listRecords",
    label: "Browse records with this value",
    filters: {
      category: "creature",
      limit: 20,
    },
    ...overrides,
  };
}

function createSnapshot() {
  return {
    activePane: "detail" as const,
    browserState: {
      depth: 1,
      selectedNodeIds: ["root", "leaf"],
      filter: "rage",
      detailScroll: 2,
    },
    layoutMode: "split" as const,
    searchInput: "rage",
    searchMode: false,
  };
}

function createSearchSession() {
  return {
    windowId: "window-1",
    query: {} as never,
    results: [],
    windowOffset: 0,
    resultMode: "structured" as const,
    total: 1,
    loadedCount: 1,
    hasMore: false,
    nextOffset: null,
    searchProfile: null,
    sort: "alphabetical" as const,
    sortSeed: null,
  };
}

type NavigationHarnessSnapshot = {
  state: Pf2eAppState;
  navigation: ReturnType<typeof usePf2eNavigation>;
};

function NavigationHarness({
  capture,
  initialRoute = { kind: PF2E_APP_ROUTE_KIND.AREAS } satisfies Pf2eAppRoute,
  services,
  terminal,
}: {
  capture: { current: NavigationHarnessSnapshot | null };
  initialRoute?: Pf2eAppRoute;
  services: Pf2eTerminalAppServices;
  terminal: { pauseForAnyKey: ReturnType<typeof vi.fn> };
}): React.JSX.Element {
  const [state, dispatch] = React.useReducer(pf2eAppReducer, initialRoute, createPf2eAppState);
  const navigation = usePf2eNavigation({
    state,
    dispatch,
    onExit: vi.fn(),
    rootPath: process.cwd(),
    services,
    terminal,
    workbenchSessionPrompts: {
      promptOptionalSelectOption: vi.fn(),
      promptSelectOption: vi.fn(),
      promptTextInput: vi.fn(),
      pauseForAnyKey: terminal.pauseForAnyKey,
    },
  });

  capture.current = { state, navigation };
  return <></>;
}

function createNavigationTestServices({
  model = createOntologyModel(),
  executeQuery = vi.fn(async () => createSearchSession()),
  createQueryFromOntologyQuery = vi.fn((query: OntologyNodeQuery) => ({ query } as never)),
} = {}) {
  const loadSearchSemanticsDomain = vi.fn(() => model);
  const services = {
    user: {
      ontology: {
        loadSearchSemanticsDomain,
      },
      search: {
        createQueryFromOntologyQuery,
        executeQuery,
      },
    },
    dev: {
      tagRefinement: {
        createSession: vi.fn(),
        getQueueItems: vi.fn(() => []),
        promptAndCreateSession: vi.fn(),
      },
    },
  } as unknown as Pf2eTerminalAppServices;

  return {
    services,
    loadSearchSemanticsDomain,
    createQueryFromOntologyQuery,
    executeQuery,
  };
}

async function renderNavigationHarness({
  initialRoute,
  services,
  terminal,
}: {
  initialRoute?: Pf2eAppRoute;
  services: Pf2eTerminalAppServices;
  terminal: { pauseForAnyKey: ReturnType<typeof vi.fn> };
}): Promise<{
  capture: { current: NavigationHarnessSnapshot | null };
  renderer: ReactTestRenderer;
}> {
  const capture: { current: NavigationHarnessSnapshot | null } = { current: null };
  let renderer!: ReactTestRenderer;

  await act(async () => {
    renderer = create(
      <NavigationHarness capture={capture} initialRoute={initialRoute} services={services} terminal={terminal} />,
    );
    await flushReact();
  });

  return { capture, renderer };
}

describe("pf2e navigation", () => {
  const renderers: ReactTestRenderer[] = [];

  afterEach(async () => {
    while (renderers.length > 0) {
      const renderer = renderers.pop();
      if (!renderer) {
        continue;
      }
      await act(async () => {
        renderer.unmount();
        await flushReact();
      });
    }
  });

  it("opens the ontology area with a prepared ontology route", async () => {
    const terminal = {
      pauseForAnyKey: vi.fn(),
    };
    const model = createOntologyModel();
    const { services, loadSearchSemanticsDomain } = createNavigationTestServices({ model });
    const { capture, renderer } = await renderNavigationHarness({ services, terminal });
    renderers.push(renderer);

    await act(async () => {
      capture.current!.navigation.openArea(PF2E_APP_AREA_ID.ONTOLOGY_SEARCH);
      await flushReact();
    });

    expect(loadSearchSemanticsDomain).toHaveBeenCalledTimes(1);
    expect(capture.current!.state.routeStack).toEqual([
      { kind: PF2E_APP_ROUTE_KIND.AREAS },
      {
        kind: PF2E_APP_ROUTE_KIND.ONTOLOGY,
        model,
      },
    ]);
    expect(capture.current!.navigation.transitionStatus).toBeNull();
  });

  it("keeps the current route mounted while ontology browser preparation is pending", async () => {
    const terminal = {
      pauseForAnyKey: vi.fn(),
    };
    const model = createOntologyModel();
    const deferredModel = createDeferred<OntologyDomainModel>();
    const { services } = createNavigationTestServices({
      model: deferredModel.promise as unknown as OntologyDomainModel,
    });
    const { capture, renderer } = await renderNavigationHarness({ services, terminal });
    renderers.push(renderer);

    await act(async () => {
      capture.current!.navigation.openArea(PF2E_APP_AREA_ID.ONTOLOGY_SEARCH);
      await flushReact();
    });

    expect(capture.current!.navigation.transitionPending).toBe(true);
    expect(capture.current!.navigation.transitionStatus).toEqual({
      kind: ROUTE_TRANSITION_STATUS_KIND.PENDING,
      message: PF2E_NAVIGATION_MESSAGE.OPENING_SEARCH_SEMANTICS,
      frame: 0,
    });
    expect(capture.current!.state.routeStack).toEqual([{ kind: PF2E_APP_ROUTE_KIND.AREAS }]);

    await act(async () => {
      deferredModel.resolve(model);
      await flushReact();
    });

    expect(capture.current!.state.routeStack).toEqual([
      { kind: PF2E_APP_ROUTE_KIND.AREAS },
      {
        kind: PF2E_APP_ROUTE_KIND.ONTOLOGY,
        model,
      },
    ]);
    expect(capture.current!.navigation.transitionStatus).toBeNull();
  });

  it("opens ontology results only after the session is prepared", async () => {
    const terminal = {
      pauseForAnyKey: vi.fn(),
    };
    const model = createOntologyModel();
    const query = createOntologyQuery({ label: "Rage" });
    const snapshot = createSnapshot();
    const session = createSearchSession();
    const deferredSession = createDeferred<typeof session>();
    const executeQuery = vi.fn(async () => deferredSession.promise);
    const createQueryFromOntologyQuery = vi.fn((currentQuery: OntologyNodeQuery) => ({ source: currentQuery } as never));
    const { services } = createNavigationTestServices({
      model,
      executeQuery,
      createQueryFromOntologyQuery,
    });
    const initialRoute = createPf2eOntologyRoute({ model });
    const { capture, renderer } = await renderNavigationHarness({ initialRoute, services, terminal });
    renderers.push(renderer);

    await act(async () => {
      capture.current!.navigation.openOntologySearch({
        kind: PF2E_ONTOLOGY_SEARCH_INTENT_KIND.RESULTS,
        query,
        snapshot,
      });
      await flushReact();
    });

    expect(capture.current!.navigation.transitionPending).toBe(true);
    expect(capture.current!.navigation.transitionStatus).toEqual({
      kind: ROUTE_TRANSITION_STATUS_KIND.PENDING,
      message: "Loading results for Rage...",
      frame: 0,
    });
    expect(capture.current!.navigation.route).toEqual(initialRoute);

    await act(async () => {
      deferredSession.resolve(session);
      await flushReact();
    });

    const preparedOntologyRoute = createPf2eOntologyRoute({
      model,
      snapshot,
    });
    expect(createQueryFromOntologyQuery).toHaveBeenCalledWith(query);
    expect(executeQuery).toHaveBeenCalledTimes(1);
    expect(capture.current!.state.routeStack).toEqual([
      preparedOntologyRoute,
      {
        kind: PF2E_APP_ROUTE_KIND.SEARCH,
        entry: PF2E_SEARCH_ROUTE_ENTRY_KIND.RESULTS,
        initialSession: session,
        origin: {
          kind: PF2E_SEARCH_ROUTE_ORIGIN_KIND.ONTOLOGY,
          route: preparedOntologyRoute,
        },
      },
    ]);
    expect(capture.current!.navigation.transitionStatus).toBeNull();
  });

  it("opens ontology search editor intent without executing a query", async () => {
    const terminal = {
      pauseForAnyKey: vi.fn(),
    };
    const model = createOntologyModel();
    const query = createOntologyQuery();
    const snapshot = createSnapshot();
    const executeQuery = vi.fn(async () => createSearchSession());
    const { services } = createNavigationTestServices({ model, executeQuery });
    const { capture, renderer } = await renderNavigationHarness({
      initialRoute: createPf2eOntologyRoute({ model }),
      services,
      terminal,
    });
    renderers.push(renderer);

    await act(async () => {
      capture.current!.navigation.openOntologySearchEditor(query, snapshot);
      await flushReact();
    });

    const preparedOntologyRoute = createPf2eOntologyRoute({
      model,
      snapshot,
    });
    expect(executeQuery).not.toHaveBeenCalled();
    expect(capture.current!.navigation.route).toEqual({
      kind: PF2E_APP_ROUTE_KIND.SEARCH,
      entry: PF2E_SEARCH_ROUTE_ENTRY_KIND.EDITOR,
      initialQuery: query,
      origin: {
        kind: PF2E_SEARCH_ROUTE_ORIGIN_KIND.ONTOLOGY,
        route: preparedOntologyRoute,
      },
    });
  });

  it("opens ontology results through the explicit results helper", async () => {
    const terminal = {
      pauseForAnyKey: vi.fn(),
    };
    const model = createOntologyModel();
    const query = createOntologyQuery({ label: "Helper Results" });
    const snapshot = createSnapshot();
    const session = createSearchSession();
    const executeQuery = vi.fn(async () => session);
    const createQueryFromOntologyQuery = vi.fn((currentQuery: OntologyNodeQuery) => ({ source: currentQuery } as never));
    const { services } = createNavigationTestServices({ model, executeQuery, createQueryFromOntologyQuery });
    const { capture, renderer } = await renderNavigationHarness({
      initialRoute: createPf2eOntologyRoute({ model }),
      services,
      terminal,
    });
    renderers.push(renderer);

    await act(async () => {
      capture.current!.navigation.openOntologySearchResults(query, snapshot);
      await flushReact();
    });

    expect(createQueryFromOntologyQuery).toHaveBeenCalledWith(query);
    expect(executeQuery).toHaveBeenCalledTimes(1);
    expect(capture.current!.navigation.route).toEqual({
      kind: PF2E_APP_ROUTE_KIND.SEARCH,
      entry: PF2E_SEARCH_ROUTE_ENTRY_KIND.RESULTS,
      initialSession: session,
      origin: {
        kind: PF2E_SEARCH_ROUTE_ORIGIN_KIND.ONTOLOGY,
        route: createPf2eOntologyRoute({
          model,
          snapshot,
        }),
      },
    });
  });
});
