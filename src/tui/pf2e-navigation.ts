import React from "react";

import type { OntologyNodeQuery } from "../domain/ontology-types.js";
import {
  type DerivedTagWorkbenchMode,
  type DerivedTagReviewSession,
} from "../tags/editorial.js";
import { formatDerivedTagWorkbenchModeLabel } from "../tags/editorial-ui.js";
import type { Pf2eTerminalAppServices } from "./app-services.js";
import type { OntologyInspectExplorerSnapshot } from "./ontology-explorer/inspect-screen.js";
import {
  ROUTE_TRANSITION_STATUS_KIND,
  type RouteTransitionStatus,
} from "./route-transition-status.js";
import {
  PF2E_APP_AREA_ID,
  PF2E_APP_ROUTE_KIND,
  PF2E_SEARCH_ROUTE_ORIGIN_KIND,
  canPopPf2eAppRoute,
  createPf2eAppState,
  createPf2eOntologyRoute,
  createPf2eSearchEditorRoute,
  createPf2eSearchResultsRoute,
  getCurrentPf2eAppRoute,
  pf2eAppReducer,
  type Pf2eAppAreaId,
  type CreatePf2eDerivedTagSessionOptions,
  type Pf2eAppRoute,
  type Pf2eAppState,
  type Pf2eOntologyRoute,
} from "./pf2e-app-state.js";
import type { SearchTerminalPromptAdapters } from "./interaction-context-adapters.js";
import type { DerivedTagTerminalApp } from "./framework/types.js";

type Pf2eNavigationCommit =
  | { kind: "push"; route: Pf2eAppRoute }
  | { kind: "replace"; route: Pf2eAppRoute }
  | { kind: "pop" }
  | { kind: "exit" }
  | { kind: "sequence"; commits: readonly Pf2eNavigationCommit[] };

function applyNavigationCommit({
  commit,
  dispatch,
  onExit,
}: {
  commit: Pf2eNavigationCommit;
  dispatch: React.Dispatch<Parameters<typeof pf2eAppReducer>[1]>;
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
        applyNavigationCommit({ commit: nextCommit, dispatch, onExit });
      }
      return;
  }
}

function isPromiseLike<T>(value: T | Promise<T>): value is Promise<T> {
  return typeof value === "object" && value !== null && "then" in value;
}

export const PF2E_ONTOLOGY_SEARCH_INTENT_KIND = {
  EDITOR: "editor",
  RESULTS: "results",
} as const;

export const PF2E_NAVIGATION_MESSAGE = {
  OPENING_SEARCH_SEMANTICS: "Opening Search Semantics...",
  ONTOLOGY_OPEN_FAILED: "Could not open Search Semantics.",
  ONTOLOGY_QUERY_FAILED: "Query execution failed.",
  ONTOLOGY_RESULTS_FALLBACK: "Loading results for the selected ontology entry...",
} as const;

export type Pf2eOntologySearchNavigationIntent =
  | {
      kind: (typeof PF2E_ONTOLOGY_SEARCH_INTENT_KIND)["EDITOR"];
      query: OntologyNodeQuery;
      snapshot: OntologyInspectExplorerSnapshot;
    }
  | {
      kind: (typeof PF2E_ONTOLOGY_SEARCH_INTENT_KIND)["RESULTS"];
      query: OntologyNodeQuery;
      snapshot: OntologyInspectExplorerSnapshot;
    };

function buildOntologySearchCommit({
  ontologyRoute,
  intent,
  initialSession,
}: {
  ontologyRoute: Pf2eOntologyRoute;
  intent: Pf2eOntologySearchNavigationIntent;
  initialSession?: ReturnType<typeof createPf2eSearchResultsRoute>["initialSession"];
}): Pf2eNavigationCommit {
  const preparedOntologyRoute = createPf2eOntologyRoute({
    model: ontologyRoute.model,
    initialDiscoveryMode: ontologyRoute.initialDiscoveryMode,
    loadModelForDiscoveryMode: ontologyRoute.loadModelForDiscoveryMode,
    snapshot: intent.snapshot,
  });
  const origin = {
    kind: PF2E_SEARCH_ROUTE_ORIGIN_KIND.ONTOLOGY,
    route: preparedOntologyRoute,
  } as const;

  const searchRoute =
    intent.kind === PF2E_ONTOLOGY_SEARCH_INTENT_KIND.RESULTS
      ? createPf2eSearchResultsRoute({
          initialSession: initialSession!,
          origin,
        })
      : createPf2eSearchEditorRoute({
          initialQuery: intent.query,
          origin,
        });

  return {
    kind: "sequence",
    commits: [
      {
        kind: "replace",
        route: preparedOntologyRoute,
      },
      {
        kind: "push",
        route: searchRoute,
      },
    ],
  };
}

function buildOntologyBrowserCommit({
  model,
  initialDiscoveryMode,
  loadModelForDiscoveryMode,
  snapshot,
}: {
  model: Pf2eOntologyRoute["model"];
  initialDiscoveryMode?: Pf2eOntologyRoute["initialDiscoveryMode"];
  loadModelForDiscoveryMode?: Pf2eOntologyRoute["loadModelForDiscoveryMode"];
  snapshot?: OntologyInspectExplorerSnapshot;
}): Pf2eNavigationCommit {
  return {
    kind: "push",
    route: createPf2eOntologyRoute({
      model,
      initialDiscoveryMode,
      loadModelForDiscoveryMode,
      snapshot,
    }),
  };
}

function buildReviewRouteCommit(session: DerivedTagReviewSession): Pf2eNavigationCommit {
  return {
    kind: "push",
    route: {
      kind: PF2E_APP_ROUTE_KIND.REVIEW,
      session,
    },
  };
}

function getOntologyResultLoadingMessage(query: OntologyNodeQuery): string {
  const label = query.label?.trim();
  return label ? `Loading results for ${label}...` : PF2E_NAVIGATION_MESSAGE.ONTOLOGY_RESULTS_FALLBACK;
}

function waitForTransitionStatusPaint(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

export function usePf2eNavigation({
  initialRoute = { kind: PF2E_APP_ROUTE_KIND.AREAS },
  onExit,
  rootPath,
  services,
  terminal,
  workbenchSessionPrompts,
}: {
  initialRoute?: Pf2eAppRoute;
  onExit: () => void;
  rootPath: string;
  services: Pf2eTerminalAppServices;
  terminal: Pick<DerivedTagTerminalApp, "pauseForAnyKey">;
  workbenchSessionPrompts: Pick<
    SearchTerminalPromptAdapters,
    "promptOptionalSelectOption" | "promptSelectOption" | "promptTextInput"
  > & { pauseForAnyKey: DerivedTagTerminalApp["pauseForAnyKey"] };
}): {
  state: Pf2eAppState;
  route: Pf2eAppRoute;
  transitionPending: boolean;
  transitionStatus: RouteTransitionStatus | null;
  moveAreaSelection: (delta: number, itemCount: number) => void;
  moveTagRefinementSelection: (delta: number, itemCount: number) => void;
  setTagRefinementIndex: (index: number, itemCount: number) => void;
  backOrExit: () => void;
  exitApp: () => void;
  openArea: (areaId: Pf2eAppAreaId) => void;
  openOntologyBrowser: (snapshot?: OntologyInspectExplorerSnapshot) => void;
  openOntologySearch: (intent: Pf2eOntologySearchNavigationIntent) => void;
  openOntologySearchEditor: (query: OntologyNodeQuery, snapshot: OntologyInspectExplorerSnapshot) => void;
  openOntologySearchResults: (query: OntologyNodeQuery, snapshot: OntologyInspectExplorerSnapshot) => void;
  openReviewSession: (mode: DerivedTagWorkbenchMode, options: CreatePf2eDerivedTagSessionOptions) => void;
  promptForReviewSession: (mode: DerivedTagWorkbenchMode) => void;
  returnFromSearch: (searchRoute: Extract<Pf2eAppRoute, { kind: "search" }>) => void;
} {
  const [state, dispatch] = React.useReducer(pf2eAppReducer, initialRoute, createPf2eAppState);
  const route = getCurrentPf2eAppRoute(state);
  const [transitionPending, setTransitionPending] = React.useState(false);
  const [transitionMessage, setTransitionMessage] = React.useState<string | null>(null);
  const [transitionFrame, setTransitionFrame] = React.useState(0);
  const activeTransitionIdRef = React.useRef(0);
  const transitionPendingRef = React.useRef(false);
  const transitionStatus = React.useMemo<RouteTransitionStatus | null>(() => {
    if (!transitionMessage) {
      return null;
    }

    return {
      kind: ROUTE_TRANSITION_STATUS_KIND.PENDING,
      message: transitionMessage,
      frame: transitionFrame,
    };
  }, [transitionFrame, transitionMessage]);

  React.useEffect(() => {
    if (!transitionMessage) {
      setTransitionFrame(0);
      return;
    }

    const interval = setInterval(() => {
      setTransitionFrame((current) => current + 1);
    }, 90);

    return () => {
      clearInterval(interval);
    };
  }, [transitionMessage]);

  const runRouteTransition = React.useCallback(
    async ({
      message,
      prepare,
      onError,
    }: {
      message?: string;
      prepare: () => Pf2eNavigationCommit | null | Promise<Pf2eNavigationCommit | null>;
      onError?: (error: unknown) => Promise<void> | void;
    }): Promise<void> => {
      if (transitionPendingRef.current) {
        return;
      }

      const transitionId = activeTransitionIdRef.current + 1;
      activeTransitionIdRef.current = transitionId;
      transitionPendingRef.current = true;
      setTransitionPending(true);
      setTransitionFrame(0);
      setTransitionMessage(message ?? null);

      try {
        const preparedCommit = prepare();
        const commit = isPromiseLike(preparedCommit) ? await preparedCommit : preparedCommit;
        if (activeTransitionIdRef.current !== transitionId || !commit) {
          return;
        }

        applyNavigationCommit({
          commit,
          dispatch,
          onExit,
        });
      } catch (error) {
        if (activeTransitionIdRef.current !== transitionId) {
          return;
        }
        if (onError) {
          await onError(error);
          return;
        }
        throw error;
      } finally {
        if (activeTransitionIdRef.current === transitionId) {
          transitionPendingRef.current = false;
          setTransitionMessage(null);
          setTransitionFrame(0);
          setTransitionPending(false);
        }
      }
    },
    [dispatch, onExit],
  );

  const backOrExit = React.useCallback(() => {
    void runRouteTransition({
      prepare: () => (canPopPf2eAppRoute(state) ? { kind: "pop" } : { kind: "exit" }),
    });
  }, [runRouteTransition, state]);

  const exitApp = React.useCallback(() => {
    void runRouteTransition({
      prepare: () => ({ kind: "exit" }),
    });
  }, [runRouteTransition]);

  const moveAreaSelection = React.useCallback(
    (delta: number, itemCount: number) => {
      dispatch({ type: "move_area", delta, itemCount });
    },
    [dispatch],
  );

  const moveTagRefinementSelection = React.useCallback(
    (delta: number, itemCount: number) => {
      dispatch({ type: "move_tag_refinement", delta, itemCount });
    },
    [dispatch],
  );

  const setTagRefinementIndex = React.useCallback(
    (index: number, itemCount: number) => {
      dispatch({ type: "set_tag_refinement_index", index, itemCount });
    },
    [dispatch],
  );

  const openOntologyBrowser = React.useCallback(
    (snapshot?: OntologyInspectExplorerSnapshot) => {
      void runRouteTransition({
        message: PF2E_NAVIGATION_MESSAGE.OPENING_SEARCH_SEMANTICS,
        prepare: async () => {
          await waitForTransitionStatusPaint();
          const model = await services.user.ontology.loadSearchSemanticsDomain({
            discoveryMode: "matching",
          });
          return buildOntologyBrowserCommit({
            model,
            initialDiscoveryMode: "matching",
            loadModelForDiscoveryMode: (discoveryMode) =>
              services.user.ontology.loadSearchSemanticsDomain({ discoveryMode }),
            snapshot,
          });
        },
        onError: async (error) => {
          await terminal.pauseForAnyKey(`${PF2E_NAVIGATION_MESSAGE.ONTOLOGY_OPEN_FAILED}\n\n${(error as Error).message}`);
        },
      });
    },
    [runRouteTransition, services.user.ontology, terminal],
  );

  const openArea = React.useCallback(
    (areaId: Pf2eAppAreaId) => {
      if (areaId === PF2E_APP_AREA_ID.ONTOLOGY_SEARCH) {
        openOntologyBrowser();
        return;
      }

      void runRouteTransition({
        prepare: () => {
          if (areaId === PF2E_APP_AREA_ID.TAG_REFINEMENT) {
            return { kind: "push", route: { kind: PF2E_APP_ROUTE_KIND.TAG_REFINEMENT } };
          }
          return { kind: "push", route: createPf2eSearchEditorRoute() };
        },
      });
    },
    [openOntologyBrowser, runRouteTransition],
  );

  const openOntologySearch = React.useCallback(
    (intent: Pf2eOntologySearchNavigationIntent) => {
      const currentRoute = getCurrentPf2eAppRoute(state);
      if (currentRoute.kind !== PF2E_APP_ROUTE_KIND.ONTOLOGY) {
        return;
      }

      void runRouteTransition({
        message:
          intent.kind === PF2E_ONTOLOGY_SEARCH_INTENT_KIND.RESULTS
            ? getOntologyResultLoadingMessage(intent.query)
            : undefined,
        prepare: async () => {
          return buildOntologySearchCommit({
            ontologyRoute: currentRoute,
            intent,
            ...(intent.kind === PF2E_ONTOLOGY_SEARCH_INTENT_KIND.RESULTS
              ? {
                  initialSession: await services.user.search.executeQuery(
                    services.user.search.createQueryFromOntologyQuery(intent.query),
                  ),
                }
              : {}),
          });
        },
        onError: async (error) => {
          await terminal.pauseForAnyKey(`${PF2E_NAVIGATION_MESSAGE.ONTOLOGY_QUERY_FAILED}\n\n${(error as Error).message}`);
        },
      });
    },
    [runRouteTransition, services.user.search, state, terminal],
  );

  const openOntologySearchEditor = React.useCallback(
    (query: OntologyNodeQuery, snapshot: OntologyInspectExplorerSnapshot) => {
      openOntologySearch({
        kind: PF2E_ONTOLOGY_SEARCH_INTENT_KIND.EDITOR,
        query,
        snapshot,
      });
    },
    [openOntologySearch],
  );

  const openOntologySearchResults = React.useCallback(
    (query: OntologyNodeQuery, snapshot: OntologyInspectExplorerSnapshot) => {
      openOntologySearch({
        kind: PF2E_ONTOLOGY_SEARCH_INTENT_KIND.RESULTS,
        query,
        snapshot,
      });
    },
    [openOntologySearch],
  );

  const openReviewSession = React.useCallback(
    (mode: DerivedTagWorkbenchMode, options: CreatePf2eDerivedTagSessionOptions) => {
      void runRouteTransition({
        message: `Preparing ${formatDerivedTagWorkbenchModeLabel(mode)} session...`,
        prepare: async () => {
          const session = await services.dev.tagRefinement.createSession(rootPath, mode, options);
          return buildReviewRouteCommit(session);
        },
        onError: async (error) => {
          await terminal.pauseForAnyKey(
            `Could not create the ${formatDerivedTagWorkbenchModeLabel(mode)} session.\n\n${(error as Error).message}`,
          );
        },
      });
    },
    [rootPath, runRouteTransition, services.dev.tagRefinement, terminal],
  );

  const promptForReviewSession = React.useCallback(
    (mode: DerivedTagWorkbenchMode) => {
      void runRouteTransition({
        prepare: async () => {
          const session = await services.dev.tagRefinement.promptAndCreateSession(rootPath, mode, workbenchSessionPrompts);
          return session ? buildReviewRouteCommit(session) : null;
        },
        onError: async (error) => {
          await terminal.pauseForAnyKey(
            `Could not create the ${formatDerivedTagWorkbenchModeLabel(mode)} session.\n\n${(error as Error).message}`,
          );
        },
      });
    },
    [rootPath, runRouteTransition, services.dev.tagRefinement, terminal, workbenchSessionPrompts],
  );

  const returnFromSearch = React.useCallback(
    (searchRoute: Extract<Pf2eAppRoute, { kind: "search" }>) => {
      void runRouteTransition({
        prepare: () => {
          if (searchRoute.origin?.kind === PF2E_SEARCH_ROUTE_ORIGIN_KIND.ONTOLOGY) {
            const previousRoute = state.routeStack[state.routeStack.length - 2];
            if (previousRoute?.kind === PF2E_APP_ROUTE_KIND.ONTOLOGY) {
              return { kind: "pop" };
            }
            return { kind: "replace", route: searchRoute.origin.route };
          }
          return canPopPf2eAppRoute(state) ? { kind: "pop" } : { kind: "exit" };
        },
      });
    },
    [runRouteTransition, state],
  );

  return {
    state,
    route,
    transitionPending,
    transitionStatus,
    moveAreaSelection,
    moveTagRefinementSelection,
    setTagRefinementIndex,
    backOrExit,
    exitApp,
    openArea,
    openOntologyBrowser,
    openOntologySearch,
    openOntologySearchEditor,
    openOntologySearchResults,
    openReviewSession,
    promptForReviewSession,
    returnFromSearch,
  };
}
