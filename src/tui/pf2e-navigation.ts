import React from "react";

import type { OntologyNodeQuery } from "../domain/ontology-types.js";
import {
  formatDerivedTagMigrationModeLabel,
  type DerivedTagMigrationMode,
  type DerivedTagMigrationSession,
} from "../tags/index.js";
import type { Pf2eTerminalAppServices } from "./app-services.js";
import type { OntologyInspectExplorerSnapshot } from "./ontology-explorer/inspect-screen.js";
import {
  ROUTE_TRANSITION_STATUS_KIND,
  type RouteTransitionStatus,
} from "./route-transition-status.js";
import {
  canPopPf2eAppRoute,
  getCurrentPf2eAppRoute,
  type CreatePf2eDerivedTagSessionOptions,
  type Pf2eAppAction,
  type Pf2eAppRoute,
  type Pf2eAppState,
  type Pf2eOntologyRoute,
  type Pf2eSearchRoute,
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
        applyNavigationCommit({ commit: nextCommit, dispatch, onExit });
      }
      return;
  }
}

function isPromiseLike<T>(value: T | Promise<T>): value is Promise<T> {
  return typeof value === "object" && value !== null && "then" in value;
}

function buildOntologySearchCommit({
  query,
  snapshot,
  initialSession,
}: {
  query: OntologyNodeQuery;
  snapshot: OntologyInspectExplorerSnapshot;
  initialSession?: Pf2eSearchRoute["initialSession"];
}): Pf2eNavigationCommit {
  const ontologyRoute: Pf2eOntologyRoute = {
    kind: "ontology",
    snapshot,
  };

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
        },
      },
    ],
  };
}

function buildReviewRouteCommit(session: DerivedTagMigrationSession): Pf2eNavigationCommit {
  return {
    kind: "push",
    route: {
      kind: "review",
      session,
    },
  };
}

function getOntologyResultLoadingMessage(query: OntologyNodeQuery): string {
  const label = query.label?.trim();
  return label ? `Loading results for ${label}...` : "Loading results for the selected ontology entry...";
}

export function usePf2eNavigation({
  state,
  dispatch,
  onExit,
  rootPath,
  services,
  terminal,
  workbenchSessionPrompts,
}: {
  state: Pf2eAppState;
  dispatch: React.Dispatch<Pf2eAppAction>;
  onExit: () => void;
  rootPath: string;
  services: Pf2eTerminalAppServices;
  terminal: Pick<DerivedTagTerminalApp, "pauseForAnyKey">;
  workbenchSessionPrompts: Pick<
    SearchTerminalPromptAdapters,
    "promptOptionalSelectOption" | "promptSelectOption" | "promptTextInput"
  > & { pauseForAnyKey: DerivedTagTerminalApp["pauseForAnyKey"] };
}): {
  route: Pf2eAppRoute;
  transitionPending: boolean;
  transitionStatus: RouteTransitionStatus | null;
  backOrExit: () => void;
  exitApp: () => void;
  openArea: (areaId: "tag_refinement" | "ontology_search" | "search") => void;
  openOntologyQuery: (query: OntologyNodeQuery, snapshot: OntologyInspectExplorerSnapshot) => void;
  openReviewSession: (mode: DerivedTagMigrationMode, options: CreatePf2eDerivedTagSessionOptions) => void;
  promptForReviewSession: (mode: DerivedTagMigrationMode) => void;
  returnFromSearch: (searchRoute: Extract<Pf2eAppRoute, { kind: "search" }>) => void;
} {
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

  const openArea = React.useCallback(
    (areaId: "tag_refinement" | "ontology_search" | "search") => {
      void runRouteTransition({
        prepare: () => {
          if (areaId === "tag_refinement") {
            return { kind: "push", route: { kind: "tag_refinement" } };
          }
          if (areaId === "ontology_search") {
            return { kind: "push", route: { kind: "ontology" } };
          }
          return { kind: "push", route: { kind: "search" } };
        },
      });
    },
    [runRouteTransition],
  );

  const openOntologyQuery = React.useCallback(
    (query: OntologyNodeQuery, snapshot: OntologyInspectExplorerSnapshot) => {
      const openInResults = Boolean((query as OntologyNodeQuery & { openInResults?: boolean }).openInResults);
      void runRouteTransition({
        message: openInResults ? getOntologyResultLoadingMessage(query) : undefined,
        prepare: async () => {
          if (!openInResults) {
            return buildOntologySearchCommit({ query, snapshot });
          }

          const initialSession = await services.user.search.executeQuery(
            services.user.search.createQueryFromOntologyQuery(query),
          );
          return buildOntologySearchCommit({
            query,
            snapshot,
            initialSession,
          });
        },
        onError: async (error) => {
          await terminal.pauseForAnyKey(`Query execution failed.\n\n${(error as Error).message}`);
        },
      });
    },
    [runRouteTransition, services.user.search, terminal],
  );

  const openReviewSession = React.useCallback(
    (mode: DerivedTagMigrationMode, options: CreatePf2eDerivedTagSessionOptions) => {
      void runRouteTransition({
        message: `Preparing ${formatDerivedTagMigrationModeLabel(mode)} session...`,
        prepare: async () => {
          const session = await services.dev.tagRefinement.createSession(rootPath, mode, options);
          return buildReviewRouteCommit(session);
        },
        onError: async (error) => {
          await terminal.pauseForAnyKey(
            `Could not create the ${formatDerivedTagMigrationModeLabel(mode)} session.\n\n${(error as Error).message}`,
          );
        },
      });
    },
    [rootPath, runRouteTransition, services.dev.tagRefinement, terminal],
  );

  const promptForReviewSession = React.useCallback(
    (mode: DerivedTagMigrationMode) => {
      void runRouteTransition({
        prepare: async () => {
          const session = await services.dev.tagRefinement.promptAndCreateSession(rootPath, mode, workbenchSessionPrompts);
          return session ? buildReviewRouteCommit(session) : null;
        },
        onError: async (error) => {
          await terminal.pauseForAnyKey(
            `Could not create the ${formatDerivedTagMigrationModeLabel(mode)} session.\n\n${(error as Error).message}`,
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
          if (searchRoute.origin?.kind === "ontology") {
            const previousRoute = state.routeStack[state.routeStack.length - 2];
            if (previousRoute?.kind === "ontology") {
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
    route,
    transitionPending,
    transitionStatus,
    backOrExit,
    exitApp,
    openArea,
    openOntologyQuery,
    openReviewSession,
    promptForReviewSession,
    returnFromSearch,
  };
}
