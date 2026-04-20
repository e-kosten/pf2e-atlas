import type { TerminalInteractionAction, TerminalInteractionCommand } from "../interaction-bindings.js";
import { buildTerminalInteractionHelpLines, formatTerminalInteractionFooter } from "../interaction-bindings.js";
import type { DerivedTagTerminalLine } from "../framework/types.js";
import {
  createTerminalDetailInteractionContext,
  createTerminalListInteractionContext,
  useTerminalInteractionContextRouter,
} from "../interaction-context-router.js";
import type { SearchScreenState } from "./state.js";
import type { SearchScreenOrigin } from "./workflow-types.js";
import { formatResultPosition, formatSort } from "./state.js";
import { getSearchQuerySubcategory } from "../search/service.js";
import type { SearchWorkspaceEntry } from "./workspace.js";
import {
  buildEditorCommandPaletteEntries,
  formatCountSummary,
  formatQueryStatus,
  formatMode,
  formatSearchScope,
} from "./workspace.js";
import { buildResultCommandPaletteEntries } from "./results.js";

export type SearchInteractionContext = "editor" | "result-list" | "result-detail";

export type SearchScreenIntent =
  | { type: "show_help" }
  | { type: "quit" }
  | { type: "edit_query" }
  | { type: "open_editor_commands" }
  | { type: "execute" }
  | { type: "back_to_app" }
  | { type: "move_workspace_selection"; delta: number }
  | { type: "workspace_selection_boundary"; boundary: "start" | "end" }
  | { type: "edit_selected_workspace" }
  | { type: "open_result_commands" }
  | { type: "toggle_pane" }
  | { type: "return_to_editor" }
  | { type: "open_preview" }
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
      return getSearchEditorInteractionActions(origin);
    case "result-list":
      return getSearchResultListInteractionActions(origin);
    case "result-detail":
      return getSearchResultDetailInteractionActions(origin);
  }
}

export function getSearchEditorInteractionActions(origin: SearchScreenOrigin = "app"): TerminalInteractionAction[] {
  return [
    { id: "edit" },
    { id: "execute" },
    { id: "search", label: "query" },
    { id: "commands" },
    { id: "help" },
    { id: "back", label: origin === "ontology" ? "return" : undefined },
    { id: "quit", label: origin === "ontology" ? "return" : "back" },
  ];
}

export function getSearchResultListInteractionActions(origin: SearchScreenOrigin = "app"): TerminalInteractionAction[] {
  return [
    { id: "back", label: origin === "ontology" ? "return" : "editor" },
    { id: "preview" },
    { id: "focus", label: "pane" },
    { id: "commands" },
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
    { id: "commands" },
    { id: "help" },
    { id: "quit", label: origin === "ontology" ? "return" : "back" },
  ];
}

export function buildSearchFooterText(
  state: SearchScreenState,
  loadingMore: boolean,
  origin: SearchScreenOrigin = "app",
): string {
  const context = getSearchInteractionContext(state);

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
                : action.id === "commands"
                  ? "open the query editor command palette"
                  : action.id === "help"
                    ? "show search editor help"
                    : origin === "ontology"
                      ? "return to the launching ontology view"
                      : "leave browse/search",
        label: action.id === "search" ? "edit query" : action.label,
      })),
    ];
    return buildTerminalInteractionHelpLines([
      {
        title: "Navigation",
        actions: navigationActions,
      },
      {
        title: "Actions",
        actions: actionActions,
      },
      {
        title: "Editor Commands",
        commands: buildEditorCommandPaletteEntries(workspaceEntries).map<TerminalInteractionCommand>((entry) => ({
          label: entry.label,
          description: entry.description ?? "No additional details.",
          aliases: entry.aliases,
        })),
      },
    ]);
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
      action.id === "preview"
        ? "open the focused result preview"
        : action.id === "back" && context === "result-list" && origin === "ontology"
          ? "return to the exact ontology location that launched this result reader"
          : action.id === "back" && context === "result-list"
            ? "return to the query editor"
            : action.id === "back"
              ? "return to the result list"
              : action.id === "focus"
                ? "switch focus between results and preview"
                : action.id === "commands"
                  ? "open the results command palette"
                  : action.id === "help"
                    ? "show search results help"
                    : origin === "ontology"
                      ? "return to the launching ontology view"
                      : "leave browse/search",
    label: action.id === "focus" ? "toggle pane" : action.label,
  }));

  return buildTerminalInteractionHelpLines([
    {
      title: "Navigation",
      actions: navigationActions,
    },
    {
      title: "Actions",
      actions: resultActions,
    },
    {
      title: "Results Commands",
      commands: buildResultCommandPaletteEntries(state, origin).map<TerminalInteractionCommand>((entry) => ({
        label: entry.label,
        description: entry.description ?? "No additional details.",
        aliases: entry.aliases,
      })),
    },
  ]);
}

export function buildSearchSubtitle(
  state: SearchScreenState,
  countState: import("./state.js").SearchCountState,
): string {
  const query = `${formatMode(state.query.mode)} | ${formatSearchScope(
    state.query.filters.category,
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
  onIntent: (intent: SearchScreenIntent) => void;
}): void {
  const editorContext = createTerminalListInteractionContext("editor", {
    interactionActions: getSearchEditorInteractionActions(options.origin),
    pageSize: options.pageSize,
    jumpSize: options.selectionJumpSize,
    includeConfirmKeys: true,
  });
  const resultListContext = createTerminalListInteractionContext("resultList", {
    interactionActions: getSearchResultListInteractionActions(options.origin),
    pageSize: options.pageSize,
    jumpSize: options.selectionJumpSize,
    includeConfirmKeys: true,
  });
  const resultDetailContext = createTerminalDetailInteractionContext("resultDetail", {
    interactionActions: getSearchResultDetailInteractionActions(options.origin),
    pageSize: options.pageSize,
    jumpSize: options.selectionJumpSize,
  });

  useTerminalInteractionContextRouter({
    enabled: options.enabled,
    contexts: [editorContext, resultListContext, resultDetailContext],
    onRoute: ({ editor, resultDetail, resultList }) => {
      const activeRoute =
        options.state.layout === "editor" ? editor : options.state.activePane === "list" ? resultList : resultDetail;

      if (activeRoute.interactionAction?.id === "help") {
        options.onIntent({ type: "show_help" });
        return;
      }
      if (activeRoute.interactionAction?.id === "quit") {
        options.onIntent({ type: "quit" });
        return;
      }

      if (options.state.layout === "editor") {
        if (activeRoute.interactionAction?.id === "search") {
          options.onIntent({ type: "edit_query" });
          return;
        }
        if (activeRoute.interactionAction?.id === "commands") {
          options.onIntent({ type: "open_editor_commands" });
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
        if (editor.navigationAction?.kind === "move") {
          options.onIntent({ type: "move_workspace_selection", delta: editor.navigationAction.delta });
          return;
        }
        if (editor.navigationAction?.kind === "boundary") {
          options.onIntent({ type: "workspace_selection_boundary", boundary: editor.navigationAction.boundary });
          return;
        }
        if (activeRoute.interactionAction?.id === "edit") {
          options.onIntent({ type: "edit_selected_workspace" });
        }
        return;
      }

      if (activeRoute.interactionAction?.id === "commands") {
        options.onIntent({ type: "open_result_commands" });
        return;
      }

      if (activeRoute.interactionAction?.id === "focus") {
        options.onIntent({ type: "toggle_pane" });
        return;
      }

      if (options.state.activePane === "list") {
        if (resultList.interactionAction?.id === "back") {
          options.onIntent({ type: "return_to_editor" });
          return;
        }
        if (resultList.interactionAction?.id === "preview" && options.hasSelectedResult) {
          options.onIntent({ type: "open_preview" });
          return;
        }
        if (resultList.navigationAction?.kind === "move") {
          options.onIntent({ type: "move_result_selection", delta: resultList.navigationAction.delta });
          return;
        }
        if (resultList.navigationAction?.kind === "boundary") {
          options.onIntent({ type: "result_selection_boundary", boundary: resultList.navigationAction.boundary });
        }
        return;
      }

      if (resultDetail.interactionAction?.id === "back") {
        options.onIntent({ type: "return_to_result_list" });
        return;
      }
      if (resultDetail.navigationAction?.kind === "move") {
        options.onIntent({ type: "move_detail", delta: resultDetail.navigationAction.delta });
        return;
      }
      if (resultDetail.navigationAction?.kind === "boundary") {
        options.onIntent({ type: "detail_boundary", boundary: resultDetail.navigationAction.boundary });
      }
    },
  });
}
