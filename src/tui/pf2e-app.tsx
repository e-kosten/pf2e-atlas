import React from "react";

import {
  type DerivedTagWorkbenchMode,
} from "../tags/editorial.js";
import { DerivedTagReviewScreen } from "../tags/editorial-ui.js";
import { PF2E_APP_AREAS, PF2E_TERMINAL_TITLE } from "./app-areas.js";
import { Pf2eTerminalAppServicesProvider } from "./app-service-context.js";
import { loadPf2eTerminalAppServices, type Pf2eTerminalAppServices } from "./app-services.js";
import {
  PF2E_APP_ROUTE_KIND,
  PF2E_SEARCH_ROUTE_ENTRY_KIND,
  PF2E_SEARCH_ROUTE_ORIGIN_KIND,
  type Pf2eAppRoute,
} from "./pf2e-app-state.js";
import { AreaMenuScreen } from "./area-menu-screen.js";
import {
  OntologyInspectScreen,
} from "./ontology-explorer/inspect-screen.js";
import { FILTER_EXPLORER_LAUNCH_INTENT, type FilterExplorerSelectTargetOutcome } from "./filter-explorer/index.js";
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
  initialRoute = { kind: PF2E_APP_ROUTE_KIND.AREAS } satisfies Pf2eAppRoute,
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
  const navigation = usePf2eNavigation({
    initialRoute,
    onExit,
    rootPath,
    services,
    terminal,
    workbenchSessionPrompts,
  });
  const queueItems = services.dev.tagRefinement.getQueueItems();
  const state = navigation.state;
  const route = navigation.route;
  const transitionStatus = navigation.transitionStatus;

  const openSelectedArea = React.useCallback(() => {
    const selectedArea = PF2E_APP_AREAS[state.selectedAreaIndex];
    if (!selectedArea) {
      return;
    }
    navigation.openArea(selectedArea.id);
  }, [navigation, state.selectedAreaIndex]);

  const openOntologySearch = React.useCallback(
    (
      outcome: FilterExplorerSelectTargetOutcome,
      snapshot: Parameters<typeof navigation.openOntologySearchEditor>[1],
    ) => {
      const intent = outcome.queryIntent;
      if (intent.launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS) {
        navigation.openOntologySearchResults(intent.query, snapshot);
        return;
      }

      navigation.openOntologySearchEditor(intent.query, snapshot);
    },
    [navigation],
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
    (mode: "review_all" | DerivedTagWorkbenchMode) => {
      if (mode === "review_all") {
        navigation.openReviewSession("review_queue", {});
        return;
      }
      navigation.promptForReviewSession(mode);
    },
    [navigation],
  );

  let screen: React.JSX.Element;
  if (route.kind === PF2E_APP_ROUTE_KIND.ONTOLOGY) {
    screen = (
      <OntologyInspectScreen
        routeData={{
          model: route.model,
          initialDiscoveryMode: route.initialDiscoveryMode,
          loadModelForDiscoveryMode: route.loadModelForDiscoveryMode,
          snapshot: route.snapshot,
        }}
        onSelectTarget={(outcome, snapshot) => openOntologySearch(outcome, snapshot)}
        onExit={navigation.backOrExit}
        transitionStatus={transitionStatus}
      />
    );
  } else if (route.kind === PF2E_APP_ROUTE_KIND.REVIEW) {
    screen = (
      <DerivedTagReviewScreen
        rootPath={rootPath}
        initialSession={route.session}
        onComplete={navigation.backOrExit}
      />
    );
  } else if (route.kind === PF2E_APP_ROUTE_KIND.SEARCH) {
    const origin = route.origin?.kind === PF2E_SEARCH_ROUTE_ORIGIN_KIND.ONTOLOGY ? "ontology" : "app";
    screen =
      route.entry === PF2E_SEARCH_ROUTE_ENTRY_KIND.RESULTS ? (
        <SearchScreen
          entry="results"
          initialSession={route.initialSession}
          transitionStatus={transitionStatus}
          origin={origin}
          onBack={() => navigation.returnFromSearch(route)}
        />
      ) : (
        <SearchScreen
          entry="editor"
          initialRequest={route.initialQuery?.request}
          promptForInitialMode={!route.initialQuery && !route.origin}
          transitionStatus={transitionStatus}
          origin={origin}
          onBack={() => navigation.returnFromSearch(route)}
        />
      );
  } else if (route.kind === PF2E_APP_ROUTE_KIND.TAG_REFINEMENT) {
    screen = (
      <TagRefinementMenuScreen
        selectedIndex={state.tagRefinementSelectedIndex}
        queueItems={queueItems}
        onBack={navigation.backOrExit}
        onMove={(delta, itemCount) =>
          delta === 0
            ? navigation.setTagRefinementIndex(
                Math.max(0, Math.min(state.tagRefinementSelectedIndex, Math.max(0, itemCount - 1))),
                itemCount,
              )
            : navigation.moveTagRefinementSelection(delta, itemCount)
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
        onMove={(delta) => navigation.moveAreaSelection(delta, PF2E_APP_AREAS.length)}
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
  initialRoute = { kind: PF2E_APP_ROUTE_KIND.AREAS } satisfies Pf2eAppRoute,
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
