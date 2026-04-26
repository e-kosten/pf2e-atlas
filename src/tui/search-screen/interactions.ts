import type { TerminalInteractionAction } from "../interaction-bindings.js";
import { buildTerminalInteractionHelpLines, formatTerminalInteractionFooter } from "../interaction-bindings.js";
import type { DerivedTagTerminalLine } from "../framework/types.js";
import type { TerminalListDetailNotificationTone } from "../list-detail-presentation.js";
import { applyTerminalListDetailRightBehavior } from "../list-detail-behavior.js";
import {
  useTerminalListDetailInteractionRouter,
} from "../list-detail-presentation.js";
import {
  buildDerivedTagTerminalActionTargetHelpLines,
  getDerivedTagTerminalActionTargetInteractionActions,
  type DerivedTagTerminalActionTargetOption,
  type DerivedTagTerminalActionTargetState,
} from "../action-target.js";
import type { SearchScreenState } from "./state.js";
import type { SearchScreenOrigin } from "./workflow-types.js";
import { formatResultPosition, formatSort } from "./state.js";
import { getSearchQueryCategory, getSearchQuerySubcategory } from "../search/query-state.js";
import type { SearchWorkspaceEntry } from "./workspace/workspace.js";
import {
  formatCountSummary,
  formatQueryStatus,
  formatMode,
  formatSearchScope,
} from "./workspace/workspace.js";

export type SearchInteractionContext = "editor" | "result-list" | "result-detail";

export type SearchScreenIntent =
  | { type: "show_help" }
  | { type: "quit" }
  | { type: "edit_query" }
  | { type: "execute" }
  | { type: "back_to_app" }
  | { type: "move_workspace_selection"; delta: number }
  | { type: "workspace_selection_boundary"; boundary: "start" | "end" }
  | { type: "edit_selected_workspace" }
  | { type: "toggle_pane" }
  | { type: "return_to_editor" }
  | { type: "move_result_selection"; delta: number }
  | { type: "result_selection_boundary"; boundary: "start" | "end" }
  | { type: "return_to_result_list" }
  | { type: "move_detail"; delta: number }
  | { type: "detail_boundary"; boundary: "start" | "end" };

export function getSearchInteractionContext(state: SearchScreenState): SearchInteractionContext {
  if (state.layout === "editor") {
    return "editor";
  }

  return state.activePane === "list" ? "result-list" : "result-detail";
}

export function getSearchInteractionActions(
  state: SearchScreenState,
  origin: SearchScreenOrigin = "app",
): TerminalInteractionAction[] {
  switch (getSearchInteractionContext(state)) {
    case "editor":
      return getSearchEditorInteractionActions(state, origin);
    case "result-list":
      return getSearchResultListInteractionActions(origin);
    case "result-detail":
      return getSearchResultDetailInteractionActions(origin);
  }
}

export function getSearchEditorInteractionActions(
  state: SearchScreenState,
  origin: SearchScreenOrigin = "app",
): TerminalInteractionAction[] {
  return [
    { id: "edit" },
    { id: "execute" },
    ...(state.query.mode === "browse" ? [] : [{ id: "search", label: "query" } as const]),
    { id: "actions" },
    { id: "help" },
    { id: "back", label: origin === "ontology" ? "return" : undefined },
    { id: "quit", label: origin === "ontology" ? "return" : "back" },
  ];
}

export function getSearchResultListInteractionActions(origin: SearchScreenOrigin = "app"): TerminalInteractionAction[] {
  return [
    { id: "back", label: origin === "ontology" ? "return" : "editor" },
    { id: "focus", label: "pane" },
    { id: "actions" },
    { id: "help" },
    { id: "quit", label: origin === "ontology" ? "return" : "back" },
  ];
}

export function getSearchResultDetailInteractionActions(
  origin: SearchScreenOrigin = "app",
): TerminalInteractionAction[] {
  return [
    { id: "back", label: "results" },
    { id: "focus", label: "pane" },
    { id: "actions" },
    { id: "help" },
    { id: "quit", label: origin === "ontology" ? "return" : "back" },
  ];
}

export function buildSearchFooterText(
  state: SearchScreenState,
  loadingMore: boolean,
  origin: SearchScreenOrigin = "app",
  options: {
    actionTargetState?: DerivedTagTerminalActionTargetState;
  } = {},
): string {
  const context = getSearchInteractionContext(state);

  if (options.actionTargetState?.activeTarget === "actions") {
    return formatTerminalInteractionFooter([
      ...getDerivedTagTerminalActionTargetInteractionActions(options.actionTargetState, "horizontal"),
      { id: "help" },
    ]);
  }

  if (context === "editor") {
    return formatTerminalInteractionFooter([
      { id: "move", label: "select" },
      { id: "jump" },
      { id: "page" },
      { id: "edge" },
      ...getSearchInteractionActions(state, origin),
    ]);
  }

  if (context === "result-list") {
    const footer = formatTerminalInteractionFooter([
      { id: "move", label: "select" },
      { id: "jump" },
      { id: "page" },
      { id: "edge" },
      ...getSearchInteractionActions(state, origin),
    ]);
    return loadingMore ? `${footer}  Loading more...` : footer;
  }

  return formatTerminalInteractionFooter([
    { id: "scroll" },
    { id: "jump" },
    { id: "page" },
    { id: "edge" },
    ...getSearchInteractionActions(state, origin),
  ]);
}

export function buildSearchHelpLines(
  state: SearchScreenState,
  workspaceEntries: SearchWorkspaceEntry[],
  origin: SearchScreenOrigin = "app",
  actionEntries: readonly DerivedTagTerminalActionTargetOption[] = [],
): DerivedTagTerminalLine[] {
  const context = getSearchInteractionContext(state);

  if (context === "editor") {
    const navigationActions: TerminalInteractionAction[] = [
      { id: "move", label: "select the editor row" },
      { id: "jump", helpText: "jump through the editor list" },
      { id: "page", helpText: "page through the editor list" },
      { id: "edge", helpText: "jump to the start or end of the editor list" },
    ];
    const actionActions: TerminalInteractionAction[] = [
      ...getSearchInteractionActions(state, origin).map<TerminalInteractionAction>((action) => ({
        ...action,
        helpText:
          action.id === "edit"
            ? "edit the focused query row or act on it"
            : action.id === "execute"
              ? "execute the current query and switch to results"
              : action.id === "search"
                ? "edit the current query text"
                : action.id === "actions"
                  ? "focus the query editor action rail"
                  : action.id === "help"
                    ? "show search editor help"
                    : origin === "ontology"
                      ? "return to the launching ontology view"
                      : "leave browse/search",
        label: action.id === "search" ? "edit query" : action.label,
      })),
    ];
    return [
      ...buildTerminalInteractionHelpLines([
        {
          title: "Navigation",
          actions: navigationActions,
        },
        {
          title: "Actions",
          actions: actionActions,
        },
      ]),
      ...(actionEntries.length > 0
        ? [
            { text: "" },
            ...buildDerivedTagTerminalActionTargetHelpLines({
              orientation: "horizontal",
              visibility: "onDemand",
              actions: actionEntries,
              contentHelpText: "Use the shared action rail here instead of a hidden command palette.",
            }),
          ]
        : []),
    ];
  }

  const navigationActions: TerminalInteractionAction[] = [
    {
      id: context === "result-list" ? "move" : "scroll",
      label: context === "result-list" ? "move through results" : "scroll the preview",
    },
    {
      id: "jump",
      helpText: context === "result-list" ? "jump through the active result pane" : "jump through the preview pane",
    },
    {
      id: "page",
      helpText: context === "result-list" ? "page through the active result pane" : "page through the preview pane",
    },
    { id: "edge", helpText: "jump to the start or end of the active pane" },
  ];
  const resultActions: TerminalInteractionAction[] = getSearchInteractionActions(state, origin).map((action) => ({
    ...action,
    helpText:
      action.id === "back" && context === "result-list" && origin === "ontology"
          ? "return to the exact ontology location that launched this result reader"
          : action.id === "back" && context === "result-list"
            ? "return to the query editor"
            : action.id === "back"
              ? "return to the result list"
                : action.id === "focus"
                  ? "switch focus between results and preview"
                  : action.id === "actions"
                    ? "focus the result action rail"
                    : action.id === "help"
                      ? "show search results help"
                      : origin === "ontology"
                      ? "return to the launching ontology view"
                      : "leave browse/search",
    label: action.id === "focus" ? "toggle pane" : action.label,
  }));

  return [
    ...buildTerminalInteractionHelpLines([
      {
        title: "Navigation",
        actions: navigationActions,
      },
      {
        title: "Actions",
        actions: resultActions,
      },
    ]),
    ...(actionEntries.length > 0
      ? [
          { text: "" },
          ...buildDerivedTagTerminalActionTargetHelpLines({
            orientation: "horizontal",
            visibility: "onDemand",
            actions: actionEntries,
            contentHelpText: "Use the shared result action rail instead of a hidden command palette.",
          }),
        ]
      : []),
  ];
}

export function buildSearchSubtitle(
  state: SearchScreenState,
  countState: import("./state.js").SearchCountState,
): string {
  const query = `${formatMode(state.query.mode)} | ${formatSearchScope(
    getSearchQueryCategory(state.query),
    getSearchQuerySubcategory(state.query),
  )}`;
  if (state.layout === "editor") {
    return `${query} | ${formatCountSummary(countState, state.query)} | ${formatQueryStatus(state)}`;
  }
  if (!state.session) {
    return `${query} | no applied session`;
  }
  return `${query} | ${formatSort(state.session.sort)} | ${formatResultPosition(state.resultSelectedIndex, state.session.total)} | ${formatQueryStatus(state)}`;
}

export function useSearchScreenInteractionRouter(options: {
  enabled: boolean;
  origin?: SearchScreenOrigin;
  state: SearchScreenState;
  workspaceEntryCount: number;
  resultCount: number;
  selectionJumpSize: number;
  pageSize: number;
  maxDetailScroll: number;
  hasSelectedResult: boolean;
  showNotification: (options: {
    message: string;
    tone?: TerminalListDetailNotificationTone;
  }) => void;
  onIntent: (intent: SearchScreenIntent) => void;
  actionTarget?: {
    state: DerivedTagTerminalActionTargetState;
    actionCount: number;
    onToggle: () => void;
    onLeave: () => void;
    onMove: (delta: number) => void;
    onApply: () => void;
  };
}): void {
  useTerminalListDetailInteractionRouter({
    enabled: options.enabled,
    list: {
      interactionActions:
        options.state.layout === "editor"
          ? getSearchEditorInteractionActions(options.state, options.origin)
          : getSearchResultListInteractionActions(options.origin),
      pageSize: options.pageSize,
      jumpSize: options.selectionJumpSize,
      includeConfirmKeys: true,
      includeHorizontalConfirmKeys: true,
    },
    detail: {
      interactionActions: getSearchResultDetailInteractionActions(options.origin),
      pageSize: options.pageSize,
      jumpSize: options.selectionJumpSize,
    },
    actionTarget: options.actionTarget
      ? {
          interactionActions: [
            ...getDerivedTagTerminalActionTargetInteractionActions(options.actionTarget.state, "horizontal"),
            { id: "help" },
          ],
          state: options.actionTarget.state,
          orientation: "horizontal",
        }
      : undefined,
    onRoute: ({ detail, list, actionTarget }) => {
      const activeRoute =
        options.state.layout === "editor" ? list : options.state.activePane === "list" ? list : detail;

      if (activeRoute.interactionAction?.id === "help") {
        options.onIntent({ type: "show_help" });
        return;
      }
      if (activeRoute.interactionAction?.id === "quit") {
        options.onIntent({ type: "quit" });
        return;
      }

      if (options.state.layout === "editor") {
        if (actionTarget?.actionTargetIntent?.kind === "toggle_target") {
          options.actionTarget?.onToggle();
          return;
        }
        if (actionTarget?.actionTargetIntent?.kind === "leave_actions") {
          options.actionTarget?.onLeave();
          return;
        }
        if (actionTarget?.actionTargetIntent?.kind === "move_action") {
          options.actionTarget?.onMove(actionTarget.actionTargetIntent.delta);
          return;
        }
        if (actionTarget?.actionTargetIntent?.kind === "apply_action") {
          options.actionTarget?.onApply();
          return;
        }
        if (options.actionTarget?.state.activeTarget === "actions") {
          if (actionTarget?.interactionAction?.id === "help") {
            options.onIntent({ type: "show_help" });
          }
          return;
        }
        if (activeRoute.interactionAction?.id === "search") {
          options.onIntent({ type: "edit_query" });
          return;
        }
        if (activeRoute.interactionAction?.id === "actions") {
          options.actionTarget?.onToggle();
          return;
        }
        if (activeRoute.interactionAction?.id === "execute") {
          options.onIntent({ type: "execute" });
          return;
        }
        if (activeRoute.interactionAction?.id === "back") {
          options.onIntent({ type: "back_to_app" });
          return;
        }
        if (list.navigationAction?.kind === "move") {
          options.onIntent({ type: "move_workspace_selection", delta: list.navigationAction.delta });
          return;
        }
        if (list.navigationAction?.kind === "boundary") {
          options.onIntent({ type: "workspace_selection_boundary", boundary: list.navigationAction.boundary });
          return;
        }
        if (activeRoute.interactionAction?.id === "edit") {
          options.onIntent({ type: "edit_selected_workspace" });
        }
        return;
      }

      if (actionTarget?.actionTargetIntent?.kind === "toggle_target") {
        options.actionTarget?.onToggle();
        return;
      }
      if (actionTarget?.actionTargetIntent?.kind === "leave_actions") {
        options.actionTarget?.onLeave();
        return;
      }
      if (actionTarget?.actionTargetIntent?.kind === "move_action") {
        options.actionTarget?.onMove(actionTarget.actionTargetIntent.delta);
        return;
      }
      if (actionTarget?.actionTargetIntent?.kind === "apply_action") {
        options.actionTarget?.onApply();
        return;
      }
      if (options.actionTarget?.state.activeTarget === "actions") {
        if (actionTarget?.interactionAction?.id === "help") {
          options.onIntent({ type: "show_help" });
        }
        return;
      }

      if (activeRoute.interactionAction?.id === "actions") {
        options.actionTarget?.onToggle();
        return;
      }

      if (activeRoute.interactionAction?.id === "focus") {
        options.onIntent({ type: "toggle_pane" });
        return;
      }

      if (options.state.activePane === "list") {
        if (list.interactionAction?.id === "back") {
          options.onIntent({ type: "return_to_editor" });
          return;
        }
        if (list.navigationAction?.kind === "confirm") {
          applyTerminalListDetailRightBehavior({
            contract: {
              rightIntent: "preview",
              destination: options.hasSelectedResult
                ? { availability: "already-satisfied" }
                : { availability: "unavailable" },
              deadEndPolicy: options.hasSelectedResult ? "notify" : "noop",
            },
            showNotification: options.showNotification,
          });
          return;
        }
        if (list.navigationAction?.kind === "move") {
          options.onIntent({ type: "move_result_selection", delta: list.navigationAction.delta });
          return;
        }
        if (list.navigationAction?.kind === "boundary") {
          options.onIntent({ type: "result_selection_boundary", boundary: list.navigationAction.boundary });
        }
        return;
      }

      if (detail.interactionAction?.id === "back") {
        options.onIntent({ type: "return_to_result_list" });
        return;
      }
      if (detail.navigationAction?.kind === "move") {
        options.onIntent({ type: "move_detail", delta: detail.navigationAction.delta });
        return;
      }
      if (detail.navigationAction?.kind === "boundary") {
        options.onIntent({ type: "detail_boundary", boundary: detail.navigationAction.boundary });
      }
    },
  });
}
