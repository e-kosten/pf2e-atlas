import React from "react";
import { DatabaseSync } from "node:sqlite";

import type { SearchCategory, SearchSubcategory } from "../types.js";
import { AreaMenuScreen, type Pf2eTopLevelArea } from "./area-menu-screen.js";
import { SearchScreen } from "./search-screen.js";
import { TagRefinementMenuScreen, type TagRefinementMenuItem } from "./tag-refinement-menu-screen.js";
import { TerminalBusyScreen } from "./shared-screens.js";
import { moveSelectionWrapped, runDerivedTagTerminalApp, useDerivedTagTerminalApp } from "./terminal-ui.js";
import { DerivedTagOntologyExplorerScreen } from "./ontology-explorer/screen.js";
import { DerivedTagMigrationReviewScreen } from "../tags/migration/review-ui.js";
import {
  DEFAULT_DERIVED_TAG_MIGRATION_WORKBENCH_SERVICES,
  createDerivedTagMigrationWorkbenchSession,
  getDerivedTagMigrationWorkbenchQueueItems,
  openDerivedTagMigrationWorkbenchOntology,
  promptAndCreateDerivedTagMigrationWorkbenchSession,
  type DerivedTagMigrationWorkbenchServices,
} from "../tags/migration/workbench-controller.js";
import { formatDerivedTagMigrationModeLabel } from "../tags/migration/workbench-session-prompts.js";
import type {
  DerivedTagMigrationMode,
  DerivedTagMigrationReviewDecisionKind,
  DerivedTagMigrationSession,
} from "../tags/migration/types.js";

type Pf2eAppRoute =
  | { kind: "areas" }
  | { kind: "tag_refinement" }
  | { kind: "search" }
  | { kind: "ontology"; db: DatabaseSync; cacheKey?: string }
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
    description: "Future first-class search surface for exact lookup, hard-filter browsing, and ranked/semantic retrieval.",
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

export function Pf2eTerminalApp({
  rootPath,
  argv,
  onExit,
  initialRoute = { kind: "areas" } satisfies Pf2eAppRoute,
  services = DEFAULT_DERIVED_TAG_MIGRATION_WORKBENCH_SERVICES,
}: {
  rootPath: string;
  argv: string[];
  onExit: () => void;
  initialRoute?: Pf2eAppRoute;
  services?: DerivedTagMigrationWorkbenchServices;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [state, dispatch] = React.useReducer(pf2eAppReducer, initialRoute, createPf2eAppState);
  const [busyMessage, setBusyMessage] = React.useState<string | null>(null);

  const queueItems = getDerivedTagMigrationWorkbenchQueueItems(services);

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
        const session = await createDerivedTagMigrationWorkbenchSession(rootPath, argv, mode, options, services);
        openReviewSession(session);
      } catch (error) {
        await terminal.pauseForAnyKey(`Could not create the ${formatDerivedTagMigrationModeLabel(mode)} session.\n\n${(error as Error).message}`);
      }
    });
  }, [argv, openReviewSession, rootPath, runWithBusyState, services, terminal]);

  const startCustomSession = React.useCallback(async (mode: DerivedTagMigrationMode) => {
    await runWithBusyState(`Loading ${formatDerivedTagMigrationModeLabel(mode)} session options...`, async () => {
      try {
        const session = await promptAndCreateDerivedTagMigrationWorkbenchSession(rootPath, argv, mode, terminal, services);
        if (session) {
          openReviewSession(session);
        }
      } catch (error) {
        await terminal.pauseForAnyKey(`Could not create the ${formatDerivedTagMigrationModeLabel(mode)} session.\n\n${(error as Error).message}`);
      }
    });
  }, [argv, openReviewSession, rootPath, runWithBusyState, services, terminal]);

  const openOntology = React.useCallback(async () => {
    await runWithBusyState("Opening ontology explorer...", async () => {
      const { db, cacheKey } = await openDerivedTagMigrationWorkbenchOntology(argv, services);
      dispatch({
        type: "set_route",
        route: {
          kind: "ontology",
          cacheKey,
          db,
        },
      });
    });
  }, [argv, runWithBusyState, services]);

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

  if (busyMessage) {
    return <TerminalBusyScreen title={PF2E_TERMINAL_TITLE} message={busyMessage} />;
  }

  if (state.route.kind === "ontology") {
    const route = state.route;
    return (
      <DerivedTagOntologyExplorerScreen
        db={route.db}
        options={{ cacheKey: route.cacheKey }}
        onExit={() => {
          route.db.close();
          dispatch({ type: "set_route", route: { kind: "areas" } });
        }}
      />
    );
  }

  if (state.route.kind === "review") {
    return (
      <DerivedTagMigrationReviewScreen
        rootPath={rootPath}
        initialSession={state.route.session}
        onComplete={() => {
          dispatch({ type: "set_route", route: { kind: "tag_refinement" } });
        }}
      />
    );
  }

  if (state.route.kind === "search") {
    return (
      <SearchScreen
        onBack={() => dispatch({ type: "set_route", route: { kind: "areas" } })}
      />
    );
  }

  if (state.route.kind === "tag_refinement") {
    return (
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
  }

  return (
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

function Pf2eTerminalAppRunner({
  rootPath,
  argv,
}: {
  rootPath: string;
  argv: string[];
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  return (
    <Pf2eTerminalApp
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
