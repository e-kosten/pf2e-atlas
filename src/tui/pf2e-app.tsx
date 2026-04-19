import React from "react";

import { DerivedTagMigrationReviewScreen } from "../tags/migration/review-ui.js";
import { formatDerivedTagMigrationModeLabel } from "../tags/migration/workbench-session-prompts.js";
import type { DerivedTagMigrationMode, DerivedTagMigrationSession } from "../tags/migration/types.js";
import { PF2E_APP_AREAS, PF2E_TERMINAL_TITLE } from "./app-areas.js";
import { Pf2eTerminalAppServicesProvider } from "./app-service-context.js";
import { loadPf2eTerminalAppServices, type Pf2eTerminalAppServices } from "./app-services.js";
import {
  canPopPf2eAppRoute,
  createPf2eAppState,
  getCurrentPf2eAppRoute,
  pf2eAppReducer,
  type CreatePf2eDerivedTagSessionOptions,
  type Pf2eAppRoute,
} from "./pf2e-app-state.js";
import { AreaMenuScreen } from "./area-menu-screen.js";
import { OntologyBrowserScreen } from "./ontology-explorer/screen.js";
import { OntologyDomainPickerScreen } from "./ontology-explorer/domain-picker-screen.js";
import { SearchScreen } from "./search-screen.js";
import { TerminalBusyScreen } from "./shared-screens.js";
import { formatTerminalInteractionFooter, resolveTerminalInteractionAction } from "./interaction-bindings.js";
import {
  TerminalTextScreen,
  runDerivedTagTerminalApp,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
} from "./terminal-ui.js";
import { TagRefinementMenuScreen, type TagRefinementMenuItem } from "./tag-refinement-menu-screen.js";

function StartupErrorScreen({ message, onExit }: { message: string; onExit: () => void }): React.JSX.Element {
  useDerivedTagTerminalInput((event) => {
    const interactionAction = resolveTerminalInteractionAction(event, [{ id: "back" }, { id: "quit" }]);
    if (interactionAction?.id === "back" || interactionAction?.id === "quit") {
      onExit();
    }
  });

  return (
    <TerminalTextScreen
      title={PF2E_TERMINAL_TITLE}
      body={[{ text: "Could not load the PF2E app services.", tone: "section" }, { text: "" }, { text: message }]}
      footer={[{ text: formatTerminalInteractionFooter([{ id: "back", label: "exit" }, { id: "quit", label: "exit" }]), tone: "dim" }]}
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
  const [state, dispatch] = React.useReducer(pf2eAppReducer, initialRoute, createPf2eAppState);
  const [busyMessage, setBusyMessage] = React.useState<string | null>(null);
  const route = getCurrentPf2eAppRoute(state);
  const queueItems = services.dev.tagRefinement.getQueueItems();
  const ontologyDomains = services.user.ontology.listDomains();

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
          const session = await services.dev.tagRefinement.promptAndCreateSession(rootPath, mode, terminal);
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
    [openReviewSession, rootPath, runWithBusyState, services.dev.tagRefinement, terminal],
  );

  const openOntologyDomain = React.useCallback(async () => {
    const selectedDomain = ontologyDomains[state.ontologyDomainSelectedIndex];
    if (!selectedDomain) {
      return;
    }
    await runWithBusyState(`Opening ${selectedDomain.label} ontology...`, async () => {
      try {
        const model = services.user.ontology.loadDomain(selectedDomain.id);
        dispatch({ type: "push_route", route: { kind: "ontology", model } });
      } catch (error) {
        await terminal.pauseForAnyKey(
          `Could not open the ${selectedDomain.label} ontology.\n\n${(error as Error).message}`,
        );
      }
    });
  }, [ontologyDomains, runWithBusyState, services.user.ontology, state.ontologyDomainSelectedIndex, terminal]);

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
      dispatch({ type: "push_route", route: { kind: "ontology_picker" } });
      return;
    }
    dispatch({ type: "push_route", route: { kind: "search" } });
  }, [state.selectedAreaIndex]);

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

  let screen: React.JSX.Element;
  if (busyMessage) {
    screen = <TerminalBusyScreen title={PF2E_TERMINAL_TITLE} message={busyMessage} />;
  } else if (route.kind === "ontology_picker") {
    screen = (
      <OntologyDomainPickerScreen
        domains={ontologyDomains}
        selectedIndex={state.ontologyDomainSelectedIndex}
        onBack={() => {
          if (canPopPf2eAppRoute(state)) {
            dispatch({ type: "pop_route" });
          } else {
            onExit();
          }
        }}
        onMove={(delta, itemCount) =>
          dispatch(
            delta === 0
              ? {
                  type: "set_ontology_domain_index",
                  index: Math.max(0, Math.min(state.ontologyDomainSelectedIndex, Math.max(0, itemCount - 1))),
                  itemCount,
                }
              : { type: "move_ontology_domain", delta, itemCount },
          )
        }
        onOpenSelected={() => {
          void openOntologyDomain();
        }}
      />
    );
  } else if (route.kind === "ontology") {
    screen = (
      <OntologyBrowserScreen
        model={route.model}
        onOpenQuery={(query) => {
          dispatch({ type: "push_route", route: { kind: "search", initialQuery: query } });
        }}
        onExit={() => {
          if (canPopPf2eAppRoute(state)) {
            dispatch({ type: "pop_route" });
          } else {
            onExit();
          }
        }}
      />
    );
  } else if (route.kind === "review") {
    screen = (
      <DerivedTagMigrationReviewScreen
        rootPath={rootPath}
        initialSession={route.session}
        onComplete={() => {
          if (canPopPf2eAppRoute(state)) {
            dispatch({ type: "pop_route" });
          } else {
            onExit();
          }
        }}
      />
    );
  } else if (route.kind === "search") {
    screen = (
      <SearchScreen
        initialQuery={route.initialQuery}
        onBack={() => {
          if (canPopPf2eAppRoute(state)) {
            dispatch({ type: "pop_route" });
          } else {
            onExit();
          }
        }}
      />
    );
  } else if (route.kind === "tag_refinement") {
    screen = (
      <TagRefinementMenuScreen
        selectedIndex={state.tagRefinementSelectedIndex}
        queueItems={queueItems}
        onBack={() => {
          if (canPopPf2eAppRoute(state)) {
            dispatch({ type: "pop_route" });
          } else {
            onExit();
          }
        }}
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
