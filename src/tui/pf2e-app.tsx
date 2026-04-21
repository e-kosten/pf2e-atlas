import React from "react";

import type { OntologyDomainModel, OntologyNodeQuery } from "../domain/ontology-types.js";
import {
  DerivedTagMigrationReviewScreen,
  type DerivedTagMigrationMode,
} from "../tags/index.js";
import { PF2E_APP_AREAS, PF2E_TERMINAL_TITLE } from "./app-areas.js";
import { Pf2eTerminalAppServicesProvider } from "./app-service-context.js";
import { loadPf2eTerminalAppServices, type Pf2eTerminalAppServices } from "./app-services.js";
import {
  createPf2eAppState,
  pf2eAppReducer,
  type Pf2eAppAction,
  type Pf2eAppRoute,
  type Pf2eSearchRoute,
} from "./pf2e-app-state.js";
import { AreaMenuScreen } from "./area-menu-screen.js";
import {
  OntologyInspectScreen,
  type OntologyInspectExplorerSnapshot,
  type OntologyInspectRouteData,
} from "./ontology-explorer/inspect-screen.js";
import { FILTER_EXPLORER_LAUNCH_INTENT } from "./filter-explorer/index.js";
import { SearchScreen } from "./search-screen/screen.js";
import { TerminalBusyScreen, TerminalMessageScreen } from "./shared-screens.js";
import { createTerminalInteractionContextAdapters } from "./interaction-context-adapters.js";
import { useDerivedTagTerminalApp } from "./framework/context.js";
import { runDerivedTagTerminalApp } from "./framework/provider.js";
import { TagRefinementMenuScreen, type TagRefinementMenuItem } from "./tag-refinement-menu-screen.js";
import { usePf2eNavigation } from "./pf2e-navigation.js";
import {
  ROUTE_TRANSITION_STATUS_KIND,
  type RouteTransitionStatus,
} from "./route-transition-status.js";

const PF2E_AREA_ID = {
  ONTOLOGY_SEARCH: "ontology_search",
} as const;

const PF2E_TRANSITION_COPY = {
  OPENING_SEARCH_SEMANTICS: "Opening Search Semantics...",
  OPENING_ONTOLOGY_RESULTS_FALLBACK: "Loading results for the selected ontology entry...",
  ONTOLOGY_OPEN_FAILED: "Could not open Search Semantics.",
  ONTOLOGY_QUERY_FAILED: "Query execution failed.",
} as const;

type Pf2ePreparedOntologyRoute = Extract<Pf2eAppRoute, { kind: "ontology" }> & OntologyInspectRouteData;
type Pf2eLocalNavigationCommit =
  | { kind: "push"; route: Pf2eAppRoute }
  | { kind: "replace"; route: Pf2eAppRoute }
  | { kind: "pop" }
  | { kind: "exit" }
  | { kind: "sequence"; commits: readonly Pf2eLocalNavigationCommit[] };

function isPromiseLike<T>(value: T | Promise<T>): value is Promise<T> {
  return typeof value === "object" && value !== null && "then" in value;
}

function isPreparedOntologyRoute(route: Pf2eAppRoute): route is Pf2ePreparedOntologyRoute {
  return route.kind === "ontology" && "model" in route;
}

function createPreparedOntologyRoute(
  model: OntologyDomainModel,
  snapshot?: OntologyInspectExplorerSnapshot,
): Pf2ePreparedOntologyRoute {
  return { kind: "ontology", model, ...(snapshot ? { snapshot } : {}) };
}

function buildPreparedOntologySearchCommit({
  model,
  query,
  snapshot,
  initialSession,
}: {
  model: OntologyDomainModel;
  query: OntologyNodeQuery;
  snapshot: OntologyInspectExplorerSnapshot;
  initialSession?: Pf2eSearchRoute["initialSession"];
}): Pf2eLocalNavigationCommit {
  const ontologyRoute = createPreparedOntologyRoute(model, snapshot);

  return {
    kind: "sequence",
    commits: [
      {
        kind: "replace",
        route: ontologyRoute,
      },
      {
        kind: "push",
        route: {
          kind: "search",
          initialQuery: query,
          initialSession,
          origin: {
            kind: "ontology",
            route: ontologyRoute,
          },
        } as Pf2eAppRoute,
      },
    ],
  };
}

function applyLocalNavigationCommit({
  commit,
  dispatch,
  onExit,
}: {
  commit: Pf2eLocalNavigationCommit;
  dispatch: React.Dispatch<Pf2eAppAction>;
  onExit: () => void;
}): void {
  switch (commit.kind) {
    case "push":
      dispatch({ type: "push_route", route: commit.route });
      return;
    case "replace":
      dispatch({ type: "replace_route", route: commit.route });
      return;
    case "pop":
      dispatch({ type: "pop_route" });
      return;
    case "exit":
      onExit();
      return;
    case "sequence":
      for (const nextCommit of commit.commits) {
        applyLocalNavigationCommit({ commit: nextCommit, dispatch, onExit });
      }
      return;
  }
}

function getOntologyResultLoadingMessage(query: OntologyNodeQuery): string {
  const label = query.label?.trim();
  return label
    ? `Loading results for ${label}...`
    : PF2E_TRANSITION_COPY.OPENING_ONTOLOGY_RESULTS_FALLBACK;
}

function StartupErrorScreen({ message, onExit }: { message: string; onExit: () => void }): React.JSX.Element {
  return (
    <TerminalMessageScreen
      title={PF2E_TERMINAL_TITLE}
      interactionActions={[{ id: "quit", label: "exit" }]}
      body={[{ text: "Could not load the PF2E app services.", tone: "section" }, { text: "" }, { text: message }]}
      onBack={onExit}
    />
  );
}

export function Pf2eTerminalApp({
  rootPath,
  onExit,
  initialRoute = { kind: "areas" } satisfies Pf2eAppRoute,
  services,
}: {
  rootPath: string;
  onExit: () => void;
  initialRoute?: Pf2eAppRoute;
  services: Pf2eTerminalAppServices;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const promptAdapters = React.useMemo(() => createTerminalInteractionContextAdapters(terminal), [terminal]);
  const workbenchSessionPrompts = React.useMemo(
    () => ({
      ...promptAdapters,
      pauseForAnyKey: terminal.pauseForAnyKey,
    }),
    [promptAdapters, terminal],
  );
  const [state, dispatch] = React.useReducer(pf2eAppReducer, initialRoute, createPf2eAppState);
  const queueItems = services.dev.tagRefinement.getQueueItems();
  const navigation = usePf2eNavigation({
    state,
    dispatch,
    onExit,
    rootPath,
    services,
    terminal,
    workbenchSessionPrompts,
  });
  const route = navigation.route;
  const [localTransitionMessage, setLocalTransitionMessage] = React.useState<string | null>(null);
  const [localTransitionFrame, setLocalTransitionFrame] = React.useState(0);
  const activeLocalTransitionIdRef = React.useRef(0);
  const localTransitionPendingRef = React.useRef(false);
  const localTransitionStatus = React.useMemo<RouteTransitionStatus | null>(() => {
    if (!localTransitionMessage) {
      return null;
    }

    return {
      kind: ROUTE_TRANSITION_STATUS_KIND.PENDING,
      message: localTransitionMessage,
      frame: localTransitionFrame,
    };
  }, [localTransitionFrame, localTransitionMessage]);
  const transitionStatus = localTransitionStatus ?? navigation.transitionStatus;
  const preparedOntologyRoute = isPreparedOntologyRoute(route) ? route : null;

  React.useEffect(() => {
    if (!localTransitionMessage) {
      setLocalTransitionFrame(0);
      return;
    }

    const interval = setInterval(() => {
      setLocalTransitionFrame((current) => current + 1);
    }, 90);

    return () => {
      clearInterval(interval);
    };
  }, [localTransitionMessage]);

  const runLocalTransition = React.useCallback(
    async ({
      message,
      prepare,
      onError,
    }: {
      message?: string;
      prepare: () => Pf2eLocalNavigationCommit | null | Promise<Pf2eLocalNavigationCommit | null>;
      onError?: (error: unknown) => Promise<void> | void;
    }): Promise<void> => {
      if (localTransitionPendingRef.current || navigation.transitionPending) {
        return;
      }

      const transitionId = activeLocalTransitionIdRef.current + 1;
      activeLocalTransitionIdRef.current = transitionId;
      localTransitionPendingRef.current = true;
      setLocalTransitionFrame(0);
      setLocalTransitionMessage(message ?? null);

      try {
        const preparedCommit = prepare();
        const commit = isPromiseLike(preparedCommit) ? await preparedCommit : preparedCommit;
        if (activeLocalTransitionIdRef.current !== transitionId || !commit) {
          return;
        }

        applyLocalNavigationCommit({
          commit,
          dispatch,
          onExit,
        });
      } catch (error) {
        if (activeLocalTransitionIdRef.current !== transitionId) {
          return;
        }
        if (onError) {
          await onError(error);
          return;
        }
        throw error;
      } finally {
        if (activeLocalTransitionIdRef.current === transitionId) {
          localTransitionPendingRef.current = false;
          setLocalTransitionMessage(null);
          setLocalTransitionFrame(0);
        }
      }
    },
    [dispatch, navigation.transitionPending, onExit],
  );

  const openSelectedArea = React.useCallback(() => {
    const selectedArea = PF2E_APP_AREAS[state.selectedAreaIndex];
    if (!selectedArea) {
      return;
    }
    if (selectedArea.id === PF2E_AREA_ID.ONTOLOGY_SEARCH) {
      void runLocalTransition({
        message: PF2E_TRANSITION_COPY.OPENING_SEARCH_SEMANTICS,
        prepare: async () => {
          await new Promise<void>((resolve) => {
            setTimeout(resolve, 0);
          });
          const loadedModel = services.user.ontology.loadSearchSemanticsDomain() as
            | OntologyDomainModel
            | Promise<OntologyDomainModel>;
          const model = isPromiseLike(loadedModel) ? await loadedModel : loadedModel;
          return { kind: "push", route: createPreparedOntologyRoute(model) };
        },
        onError: async (error) => {
          await terminal.pauseForAnyKey(
            `${PF2E_TRANSITION_COPY.ONTOLOGY_OPEN_FAILED}\n\n${(error as Error).message}`,
          );
        },
      });
      return;
    }
    navigation.openArea(selectedArea.id);
  }, [navigation, runLocalTransition, services.user.ontology, state.selectedAreaIndex, terminal]);

  const openOntologyQuery = React.useCallback(
    (
      query: OntologyNodeQuery,
      snapshot: OntologyInspectExplorerSnapshot,
      launchIntent: (typeof FILTER_EXPLORER_LAUNCH_INTENT)[keyof typeof FILTER_EXPLORER_LAUNCH_INTENT],
    ) => {
      if (!preparedOntologyRoute) {
        return;
      }

      const openInResults = launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS;
      void runLocalTransition({
        message: openInResults ? getOntologyResultLoadingMessage(query) : undefined,
        prepare: async () => {
          if (!openInResults) {
            return buildPreparedOntologySearchCommit({
              model: preparedOntologyRoute.model,
              query,
              snapshot,
            });
          }

          const initialSession = await services.user.search.executeQuery(
            services.user.search.createQueryFromOntologyQuery(query),
          );
          return buildPreparedOntologySearchCommit({
            model: preparedOntologyRoute.model,
            query,
            snapshot,
            initialSession,
          });
        },
        onError: async (error) => {
          await terminal.pauseForAnyKey(
            `${PF2E_TRANSITION_COPY.ONTOLOGY_QUERY_FAILED}\n\n${(error as Error).message}`,
          );
        },
      });
    },
    [preparedOntologyRoute, runLocalTransition, services.user.search, terminal],
  );

  const openSelectedTagRefinementItem = React.useCallback(
    (menuItems: TagRefinementMenuItem[]) => {
      const selectedItem = menuItems[state.tagRefinementSelectedIndex];
      if (!selectedItem) {
        return;
      }
      if (selectedItem.kind === "back") {
        navigation.backOrExit();
        return;
      }
      if (selectedItem.kind === "review_all") {
        navigation.openReviewSession("review_queue", {});
        return;
      }
      if (selectedItem.kind === "review_queue_item") {
        navigation.openReviewSession("review_queue", {
          decisionKind: selectedItem.queueItem.kind,
          category: selectedItem.queueItem.category,
          family: selectedItem.queueItem.family,
          tag: selectedItem.queueItem.tag,
        });
        return;
      }
      navigation.promptForReviewSession(selectedItem.mode);
    },
    [navigation, state.tagRefinementSelectedIndex],
  );

  const runQuickTagRefinementAction = React.useCallback(
    (mode: "review_all" | DerivedTagMigrationMode) => {
      if (mode === "review_all") {
        navigation.openReviewSession("review_queue", {});
        return;
      }
      navigation.promptForReviewSession(mode);
    },
    [navigation],
  );

  const returnFromSearch = React.useCallback(
    (searchRoute: Extract<Pf2eAppRoute, { kind: "search" }>) => {
      if (searchRoute.origin?.kind !== "ontology") {
        navigation.returnFromSearch(searchRoute);
        return;
      }

      void runLocalTransition({
        prepare: () => {
          const previousRoute = state.routeStack[state.routeStack.length - 2];
          if (previousRoute && isPreparedOntologyRoute(previousRoute)) {
            return { kind: "pop" };
          }

          const originRoute = searchRoute.origin?.route;
          if (!originRoute) {
            return { kind: "pop" };
          }
          if (isPreparedOntologyRoute(originRoute as Pf2eAppRoute)) {
            return { kind: "replace", route: originRoute as Pf2eAppRoute };
          }

          return { kind: "pop" };
        },
      });
    },
    [navigation, runLocalTransition, state.routeStack],
  );

  let screen: React.JSX.Element;
  if (preparedOntologyRoute) {
    screen = (
      <OntologyInspectScreen
        routeData={preparedOntologyRoute}
        onOpenQuery={openOntologyQuery}
        onExit={navigation.backOrExit}
        transitionStatus={transitionStatus}
      />
    );
  } else if (route.kind === "ontology") {
    screen = (
      <TerminalMessageScreen
        title={PF2E_TERMINAL_TITLE}
        interactionActions={[{ id: "back", label: "back" }]}
        body={[
          { text: "Search Semantics was not prepared before navigation.", tone: "section" },
          { text: "" },
          { text: "Return and reopen the area after the prepared transition wiring lands." },
        ]}
        onBack={navigation.backOrExit}
      />
    );
  } else if (route.kind === "review") {
    screen = (
      <DerivedTagMigrationReviewScreen
        rootPath={rootPath}
        initialSession={route.session}
        onComplete={navigation.backOrExit}
      />
    );
  } else if (route.kind === "search") {
    screen = (
      <SearchScreen
        initialQuery={route.initialQuery}
        initialSession={route.initialSession}
        transitionStatus={transitionStatus}
        origin={route.origin?.kind === "ontology" ? "ontology" : "app"}
        onBack={() => returnFromSearch(route)}
      />
    );
  } else if (route.kind === "tag_refinement") {
    screen = (
      <TagRefinementMenuScreen
        selectedIndex={state.tagRefinementSelectedIndex}
        queueItems={queueItems}
        onBack={navigation.backOrExit}
        onMove={(delta, itemCount) =>
          dispatch(
            delta === 0
              ? {
                  type: "set_tag_refinement_index",
                  index: Math.max(0, Math.min(state.tagRefinementSelectedIndex, Math.max(0, itemCount - 1))),
                  itemCount,
                }
              : { type: "move_tag_refinement", delta, itemCount },
          )
        }
        onOpenSelected={openSelectedTagRefinementItem}
        onQuickAction={runQuickTagRefinementAction}
        transitionStatus={transitionStatus}
      />
    );
  } else {
    screen = (
      <AreaMenuScreen
        title={PF2E_TERMINAL_TITLE}
        selectedAreaIndex={state.selectedAreaIndex}
        areas={PF2E_APP_AREAS}
        pendingReviewCount={queueItems.length}
        onMove={(delta) => dispatch({ type: "move_area", delta, itemCount: PF2E_APP_AREAS.length })}
        onOpenSelectedArea={openSelectedArea}
        onQuit={navigation.exitApp}
        transitionStatus={transitionStatus}
      />
    );
  }

  return <Pf2eTerminalAppServicesProvider services={services}>{screen}</Pf2eTerminalAppServicesProvider>;
}

export function Pf2eTerminalBootstrap({
  rootPath,
  argv,
  onExit,
  initialRoute = { kind: "areas" } satisfies Pf2eAppRoute,
  loadServices = loadPf2eTerminalAppServices,
}: {
  rootPath: string;
  argv: string[];
  onExit: () => void;
  initialRoute?: Pf2eAppRoute;
  loadServices?: (argv: string[]) => Promise<Pf2eTerminalAppServices>;
}): React.JSX.Element {
  const [status, setStatus] = React.useState<
    { kind: "loading" } | { kind: "ready"; services: Pf2eTerminalAppServices } | { kind: "error"; message: string }
  >({ kind: "loading" });

  React.useEffect(() => {
    let active = true;
    let loadedServices: Pf2eTerminalAppServices | null = null;

    void loadServices(argv)
      .then((services) => {
        if (!active) {
          services.close();
          return;
        }
        loadedServices = services;
        setStatus({ kind: "ready", services });
      })
      .catch((error) => {
        if (active) {
          setStatus({ kind: "error", message: (error as Error).message });
        }
      });

    return () => {
      active = false;
      loadedServices?.close();
    };
  }, [argv, loadServices]);

  if (status.kind === "loading") {
    return <TerminalBusyScreen title={PF2E_TERMINAL_TITLE} message="Loading PF2E app services..." />;
  }

  if (status.kind === "error") {
    return <StartupErrorScreen message={status.message} onExit={onExit} />;
  }

  return <Pf2eTerminalApp rootPath={rootPath} onExit={onExit} initialRoute={initialRoute} services={status.services} />;
}

function Pf2eTerminalAppRunner({ rootPath, argv }: { rootPath: string; argv: string[] }): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  return <Pf2eTerminalBootstrap rootPath={rootPath} argv={argv} onExit={() => terminal.exitApp()} />;
}

export async function runPf2eTerminalApp(rootPath: string, argv: string[]): Promise<void> {
  await runDerivedTagTerminalApp(<Pf2eTerminalAppRunner rootPath={rootPath} argv={argv} />);
}
