import React from "react";

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
import { usePf2eNavigation } from "./pf2e-navigation.js";

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
  const ontologyExplorerModel = React.useMemo(
    () => (route.kind === "ontology" ? services.user.ontology.loadSearchSemanticsDomain() : null),
    [route.kind, services.user.ontology],
  );

  const openSelectedArea = React.useCallback(() => {
    const selectedArea = PF2E_APP_AREAS[state.selectedAreaIndex];
    if (!selectedArea) {
      return;
    }
    navigation.openArea(selectedArea.id);
  }, [navigation, state.selectedAreaIndex]);

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

  let screen: React.JSX.Element;
  if (route.kind === "ontology") {
    screen = (
      <OntologyInspectScreen
        initialSnapshot={route.snapshot}
        model={ontologyExplorerModel!}
        onOpenQuery={navigation.openOntologyQuery}
        onExit={navigation.backOrExit}
        transitionStatus={navigation.transitionStatus}
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
        transitionStatus={navigation.transitionStatus}
        origin={route.origin?.kind === "ontology" ? "ontology" : "app"}
        onBack={() => navigation.returnFromSearch(route)}
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
        transitionStatus={navigation.transitionStatus}
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
        transitionStatus={navigation.transitionStatus}
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
