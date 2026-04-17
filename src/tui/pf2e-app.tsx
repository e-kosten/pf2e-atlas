import React from "react";

import type { SearchCategory, SearchSubcategory } from "../types.js";
import { DerivedTagMigrationReviewScreen } from "../tags/migration/review-ui.js";
import { formatDerivedTagMigrationModeLabel } from "../tags/migration/workbench-session-prompts.js";
import type {
  DerivedTagMigrationMode,
  DerivedTagMigrationReviewDecisionKind,
  DerivedTagMigrationSession,
} from "../tags/migration/types.js";
import { Pf2eTerminalAppServicesProvider } from "./app-service-context.js";
import {
  loadPf2eTerminalAppServices,
  type Pf2eTerminalAppServices,
} from "./app-services.js";
import { AreaMenuScreen, type Pf2eTopLevelArea } from "./area-menu-screen.js";
import { isBackOrExitKey } from "./keymap.js";
import { DerivedTagOntologyExplorerScreen } from "./ontology-explorer/screen.js";
import type { DerivedTagOntologyExplorerModel } from "./ontology-explorer/data.js";
import { SearchScreen } from "./search-screen.js";
import { TerminalBusyScreen } from "./shared-screens.js";
import {
  TerminalTextScreen,
  getNormalizedKeyName,
  moveSelectionWrapped,
  runDerivedTagTerminalApp,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
} from "./terminal-ui.js";
import { TagRefinementMenuScreen, type TagRefinementMenuItem } from "./tag-refinement-menu-screen.js";

type Pf2eAppRoute =
  | { kind: "areas" }
  | { kind: "tag_refinement" }
  | { kind: "search" }
  | { kind: "ontology"; model: DerivedTagOntologyExplorerModel }
  | { kind: "review"; session: DerivedTagMigrationSession };

export type Pf2eAppState = {
  route: Pf2eAppRoute;
  selectedAreaIndex: number;
  tagRefinementSelectedIndex: number;
};

export type Pf2eAppAction =
  | { type: "move_area"; delta: number }
  | { type: "move_tag_refinement"; delta: number; itemCount: number }
  | { type: "set_route"; route: Pf2eAppRoute }
  | { type: "set_tag_refinement_index"; index: number; itemCount: number };

const PF2E_TERMINAL_TITLE = "PF2E Terminal";
const PF2E_APP_AREAS: Pf2eTopLevelArea[] = [
  {
    id: "tag_refinement",
    label: "Tag Refinement",
    description: "Review authored queue items and create AI proposal, legacy-seed, legacy-rule, and exemplar-cleanup sessions.",
  },
  {
    id: "ontology_search",
    label: "Ontology Search",
    description: "Browse category -> family -> tag -> record and inspect how derived tags map onto live indexed records.",
  },
  {
    id: "search",
    label: "Search",
    description: "User-facing lookup and search over the same indexed PF2E data surfaced by the MCP server.",
  },
];

export function pf2eAppReducer(state: Pf2eAppState, action: Pf2eAppAction): Pf2eAppState {
  switch (action.type) {
    case "move_area":
      return {
        ...state,
        selectedAreaIndex: moveSelectionWrapped(state.selectedAreaIndex, action.delta, PF2E_APP_AREAS.length),
      };
    case "move_tag_refinement":
      return {
        ...state,
        tagRefinementSelectedIndex: action.itemCount <= 0
          ? 0
          : moveSelectionWrapped(state.tagRefinementSelectedIndex, action.delta, action.itemCount),
      };
    case "set_tag_refinement_index":
      return {
        ...state,
        tagRefinementSelectedIndex: action.itemCount <= 0 ? 0 : Math.max(0, Math.min(action.index, action.itemCount - 1)),
      };
    case "set_route":
      return {
        ...state,
        route: action.route,
      };
    default:
      return state;
  }
}

export function createPf2eAppState(initialRoute: Pf2eAppRoute = { kind: "areas" }): Pf2eAppState {
  return {
    route: initialRoute,
    selectedAreaIndex: 0,
    tagRefinementSelectedIndex: 0,
  };
}

function StartupErrorScreen({
  message,
  onExit,
}: {
  message: string;
  onExit: () => void;
}): React.JSX.Element {
  useDerivedTagTerminalInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);
    if (isBackOrExitKey(normalized)) {
      onExit();
    }
  });

  return (
    <TerminalTextScreen
      title={PF2E_TERMINAL_TITLE}
      body={[
        { text: "Could not load the PF2E app services.", tone: "section" },
        { text: "" },
        { text: message },
      ]}
      footer={[{ text: "q or Backspace exit", tone: "dim" }]}
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

  const queueItems = services.tagWorkbench.getQueueItems();

  const runWithBusyState = React.useCallback(async <T,>(message: string, task: () => Promise<T>): Promise<T> => {
    setBusyMessage(message);
    try {
      return await task();
    } finally {
      setBusyMessage(null);
    }
  }, []);

  const openReviewSession = React.useCallback((session: DerivedTagMigrationSession) => {
    dispatch({ type: "set_route", route: { kind: "review", session } });
  }, []);

  const createSessionAndOpenReview = React.useCallback(async (
    mode: DerivedTagMigrationMode,
    options: {
      category?: SearchCategory;
      subcategory?: SearchSubcategory;
      decisionKind?: DerivedTagMigrationReviewDecisionKind;
      family?: string;
      tag?: string;
      limit?: number;
      exemplarLimit?: number;
    },
  ) => {
    await runWithBusyState(`Preparing ${formatDerivedTagMigrationModeLabel(mode)} session...`, async () => {
      try {
        const session = await services.tagWorkbench.createSession(rootPath, mode, options);
        openReviewSession(session);
      } catch (error) {
        await terminal.pauseForAnyKey(`Could not create the ${formatDerivedTagMigrationModeLabel(mode)} session.\n\n${(error as Error).message}`);
      }
    });
  }, [openReviewSession, rootPath, runWithBusyState, services, terminal]);

  const startCustomSession = React.useCallback(async (mode: DerivedTagMigrationMode) => {
    await runWithBusyState(`Loading ${formatDerivedTagMigrationModeLabel(mode)} session options...`, async () => {
      try {
        const session = await services.tagWorkbench.promptAndCreateSession(rootPath, mode, terminal);
        if (session) {
          openReviewSession(session);
        }
      } catch (error) {
        await terminal.pauseForAnyKey(`Could not create the ${formatDerivedTagMigrationModeLabel(mode)} session.\n\n${(error as Error).message}`);
      }
    });
  }, [openReviewSession, rootPath, runWithBusyState, services, terminal]);

  const openOntology = React.useCallback(async () => {
    await runWithBusyState("Opening ontology explorer...", async () => {
      try {
        const model = services.tagWorkbench.getOntologyModel();
        dispatch({ type: "set_route", route: { kind: "ontology", model } });
      } catch (error) {
        await terminal.pauseForAnyKey(`Could not open ontology explorer.\n\n${(error as Error).message}`);
      }
    });
  }, [runWithBusyState, services, terminal]);

  const openSelectedArea = React.useCallback(() => {
    const selectedArea = PF2E_APP_AREAS[state.selectedAreaIndex];
    if (!selectedArea) {
      return;
    }

    if (selectedArea.id === "tag_refinement") {
      dispatch({ type: "set_route", route: { kind: "tag_refinement" } });
      return;
    }
    if (selectedArea.id === "ontology_search") {
      void openOntology();
      return;
    }
    dispatch({ type: "set_route", route: { kind: "search" } });
  }, [openOntology, state.selectedAreaIndex]);

  const openSelectedTagRefinementItem = React.useCallback((menuItems: TagRefinementMenuItem[]) => {
    const selectedItem = menuItems[state.tagRefinementSelectedIndex];
    if (!selectedItem) {
      return;
    }
    if (selectedItem.kind === "back") {
      dispatch({ type: "set_route", route: { kind: "areas" } });
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
    if (selectedItem.kind === "create_mode") {
      void startCustomSession(selectedItem.mode);
    }
  }, [createSessionAndOpenReview, startCustomSession, state.tagRefinementSelectedIndex]);

  const runQuickTagRefinementAction = React.useCallback((mode: "review_all" | DerivedTagMigrationMode) => {
    if (mode === "review_all") {
      void createSessionAndOpenReview("review_queue", {});
      return;
    }
    void startCustomSession(mode);
  }, [createSessionAndOpenReview, startCustomSession]);

  let screen: React.JSX.Element;
  if (busyMessage) {
    screen = <TerminalBusyScreen title={PF2E_TERMINAL_TITLE} message={busyMessage} />;
  } else if (state.route.kind === "ontology") {
    screen = (
      <DerivedTagOntologyExplorerScreen
        model={state.route.model}
        onExit={() => {
          dispatch({ type: "set_route", route: { kind: "areas" } });
        }}
      />
    );
  } else if (state.route.kind === "review") {
    screen = (
      <DerivedTagMigrationReviewScreen
        rootPath={rootPath}
        initialSession={state.route.session}
        onComplete={() => {
          dispatch({ type: "set_route", route: { kind: "tag_refinement" } });
        }}
      />
    );
  } else if (state.route.kind === "search") {
    screen = (
      <SearchScreen
        onBack={() => dispatch({ type: "set_route", route: { kind: "areas" } })}
      />
    );
  } else if (state.route.kind === "tag_refinement") {
    screen = (
      <TagRefinementMenuScreen
        selectedIndex={state.tagRefinementSelectedIndex}
        queueItems={queueItems}
        onBack={() => dispatch({ type: "set_route", route: { kind: "areas" } })}
        onMove={(delta, itemCount) => dispatch(delta === 0
          ? { type: "set_tag_refinement_index", index: Math.max(0, Math.min(state.tagRefinementSelectedIndex, Math.max(0, itemCount - 1))), itemCount }
          : { type: "move_tag_refinement", delta, itemCount })}
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
        onMove={(delta) => dispatch({ type: "move_area", delta })}
        onOpenSelectedArea={openSelectedArea}
        onQuit={onExit}
      />
    );
  }

  return (
    <Pf2eTerminalAppServicesProvider services={services}>
      {screen}
    </Pf2eTerminalAppServicesProvider>
  );
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
    | { kind: "loading" }
    | { kind: "ready"; services: Pf2eTerminalAppServices }
    | { kind: "error"; message: string }
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

  return (
    <Pf2eTerminalApp
      rootPath={rootPath}
      onExit={onExit}
      initialRoute={initialRoute}
      services={status.services}
    />
  );
}

function Pf2eTerminalAppRunner({
  rootPath,
  argv,
}: {
  rootPath: string;
  argv: string[];
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  return (
    <Pf2eTerminalBootstrap
      rootPath={rootPath}
      argv={argv}
      onExit={() => terminal.exitApp()}
    />
  );
}

export async function runPf2eTerminalApp(
  rootPath: string,
  argv: string[],
): Promise<void> {
  await runDerivedTagTerminalApp(
    <Pf2eTerminalAppRunner rootPath={rootPath} argv={argv} />,
  );
}
