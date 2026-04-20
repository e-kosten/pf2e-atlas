import React from "react";

import {
  DerivedTagMigrationReviewScreen,
  formatDerivedTagMigrationModeLabel,
  type DerivedTagMigrationMode,
  type DerivedTagMigrationSession,
} from "../tags/index.js";
import { PF2E_APP_AREAS, PF2E_TERMINAL_TITLE } from "./app-areas.js";
import { Pf2eTerminalAppServicesProvider } from "./app-service-context.js";
import { loadPf2eTerminalAppServices, type Pf2eTerminalAppServices } from "./app-services.js";
import {
  canPopPf2eAppRoute,
  createPf2eAppState,
  getCurrentPf2eAppRoute,
  pf2eAppReducer,
  type CreatePf2eDerivedTagSessionOptions,
  type Pf2eOntologyRoute,
  type Pf2eAppRoute,
} from "./pf2e-app-state.js";
import { AreaMenuScreen } from "./area-menu-screen.js";
import { OntologyInspectScreen } from "./ontology-explorer/inspect-screen.js";
import { SearchScreen } from "./search-screen/screen.js";
import { TerminalBusyScreen, TerminalMessageScreen } from "./shared-screens.js";
import { createTerminalInteractionContextAdapters } from "./interaction-context-adapters.js";
import { useDerivedTagTerminalApp } from "./framework/context.js";
import { runDerivedTagTerminalApp } from "./framework/provider.js";
import { TagRefinementMenuScreen, type TagRefinementMenuItem } from "./tag-refinement-menu-screen.js";

type OntologyQueryOpenHandler = NonNullable<React.ComponentProps<typeof OntologyInspectScreen>["onOpenQuery"]>;

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
  const [busyMessage, setBusyMessage] = React.useState<string | null>(null);
  const route = getCurrentPf2eAppRoute(state);
  const queueItems = services.dev.tagRefinement.getQueueItems();
  const ontologyExplorerModel = React.useMemo(
    () => (route.kind === "ontology" ? services.user.ontology.loadSearchSemanticsDomain() : null),
    [route.kind, services.user.ontology],
  );

  const runWithBusyState = React.useCallback(async <T,>(message: string, task: () => Promise<T>): Promise<T> => {
    setBusyMessage(message);
    try {
      return await task();
    } finally {
      setBusyMessage(null);
    }
  }, []);

  const openReviewSession = React.useCallback((session: DerivedTagMigrationSession) => {
    dispatch({ type: "push_route", route: { kind: "review", session } });
  }, []);

  const createSessionAndOpenReview = React.useCallback(
    async (mode: DerivedTagMigrationMode, options: CreatePf2eDerivedTagSessionOptions) => {
      await runWithBusyState(`Preparing ${formatDerivedTagMigrationModeLabel(mode)} session...`, async () => {
        try {
          const session = await services.dev.tagRefinement.createSession(rootPath, mode, options);
          openReviewSession(session);
        } catch (error) {
          await terminal.pauseForAnyKey(
            `Could not create the ${formatDerivedTagMigrationModeLabel(mode)} session.\n\n${(error as Error).message}`,
          );
        }
      });
    },
    [openReviewSession, rootPath, runWithBusyState, services.dev.tagRefinement, terminal],
  );

  const startCustomSession = React.useCallback(
    async (mode: DerivedTagMigrationMode) => {
      await runWithBusyState(`Loading ${formatDerivedTagMigrationModeLabel(mode)} session options...`, async () => {
        try {
          const session = await services.dev.tagRefinement.promptAndCreateSession(rootPath, mode, workbenchSessionPrompts);
          if (session) {
            openReviewSession(session);
          }
        } catch (error) {
          await terminal.pauseForAnyKey(
            `Could not create the ${formatDerivedTagMigrationModeLabel(mode)} session.\n\n${(error as Error).message}`,
          );
        }
      });
    },
    [openReviewSession, rootPath, runWithBusyState, services.dev.tagRefinement, terminal, workbenchSessionPrompts],
  );

  const openSelectedArea = React.useCallback(() => {
    const selectedArea = PF2E_APP_AREAS[state.selectedAreaIndex];
    if (!selectedArea) {
      return;
    }

    if (selectedArea.id === "tag_refinement") {
      dispatch({ type: "push_route", route: { kind: "tag_refinement" } });
      return;
    }
    if (selectedArea.id === "ontology_search") {
      dispatch({ type: "push_route", route: { kind: "ontology" } });
      return;
    }
    dispatch({ type: "push_route", route: { kind: "search" } });
  }, [state.selectedAreaIndex]);

  const returnToPreviousRouteOrExit = React.useCallback(() => {
    if (canPopPf2eAppRoute(state)) {
      dispatch({ type: "pop_route" });
    } else {
      onExit();
    }
  }, [onExit, state]);

  const handleSearchBack = React.useCallback(
    (searchRoute: Extract<Pf2eAppRoute, { kind: "search" }>) => {
      if (searchRoute.origin?.kind === "ontology") {
        const previousRoute = state.routeStack[state.routeStack.length - 2];
        if (previousRoute?.kind === "ontology") {
          dispatch({ type: "pop_route" });
          return;
        }
        dispatch({ type: "replace_route", route: searchRoute.origin.route });
        return;
      }
      returnToPreviousRouteOrExit();
    },
    [returnToPreviousRouteOrExit, state],
  );

  const openSelectedTagRefinementItem = React.useCallback(
    (menuItems: TagRefinementMenuItem[]) => {
      const selectedItem = menuItems[state.tagRefinementSelectedIndex];
      if (!selectedItem) {
        return;
      }
      if (selectedItem.kind === "back") {
        dispatch({ type: "pop_route" });
        return;
      }
      if (selectedItem.kind === "review_all") {
        void createSessionAndOpenReview("review_queue", {});
        return;
      }
      if (selectedItem.kind === "review_queue_item") {
        void createSessionAndOpenReview("review_queue", {
          decisionKind: selectedItem.queueItem.kind,
          category: selectedItem.queueItem.category,
          family: selectedItem.queueItem.family,
          tag: selectedItem.queueItem.tag,
        });
        return;
      }
      void startCustomSession(selectedItem.mode);
    },
    [createSessionAndOpenReview, startCustomSession, state.tagRefinementSelectedIndex],
  );

  const runQuickTagRefinementAction = React.useCallback(
    (mode: "review_all" | DerivedTagMigrationMode) => {
      if (mode === "review_all") {
        void createSessionAndOpenReview("review_queue", {});
        return;
      }
      void startCustomSession(mode);
    },
    [createSessionAndOpenReview, startCustomSession],
  );

  const openOntologyQuery = React.useCallback(
    (query: Parameters<OntologyQueryOpenHandler>[0], snapshot: Parameters<OntologyQueryOpenHandler>[1]) => {
      const ontologyRoute: Pf2eOntologyRoute = {
        kind: "ontology",
        snapshot,
      };
      dispatch({ type: "replace_route", route: ontologyRoute });
      dispatch({
        type: "push_route",
        route: {
          kind: "search",
          initialQuery: query,
          origin: {
            kind: "ontology",
            route: ontologyRoute,
          },
        },
      });
    },
    [],
  );

  let screen: React.JSX.Element;
  if (busyMessage) {
    screen = <TerminalBusyScreen title={PF2E_TERMINAL_TITLE} message={busyMessage} />;
  } else if (route.kind === "ontology") {
    screen = (
      <OntologyInspectScreen
        initialSnapshot={route.snapshot}
        model={ontologyExplorerModel!}
        onOpenQuery={openOntologyQuery}
        onExit={returnToPreviousRouteOrExit}
      />
    );
  } else if (route.kind === "review") {
    screen = (
      <DerivedTagMigrationReviewScreen
        rootPath={rootPath}
        initialSession={route.session}
        onComplete={returnToPreviousRouteOrExit}
      />
    );
  } else if (route.kind === "search") {
    screen = (
      <SearchScreen
        initialQuery={route.initialQuery}
        origin={route.origin?.kind === "ontology" ? "ontology" : "app"}
        onBack={() => handleSearchBack(route)}
      />
    );
  } else if (route.kind === "tag_refinement") {
    screen = (
      <TagRefinementMenuScreen
        selectedIndex={state.tagRefinementSelectedIndex}
        queueItems={queueItems}
        onBack={returnToPreviousRouteOrExit}
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
        onQuit={onExit}
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
