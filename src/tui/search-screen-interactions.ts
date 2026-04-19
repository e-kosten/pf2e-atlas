import React from "react";

import type { TerminalInteractionAction, TerminalInteractionCommand } from "./interaction-bindings.js";
import {
  buildTerminalInteractionHelpLines,
  formatTerminalInteractionFooter,
  resolveTerminalInteractionAction,
} from "./interaction-bindings.js";
import {
  createDerivedTagTerminalListNavigationState,
  resolveDerivedTagTerminalListNavigationAction,
  useDerivedTagTerminalInput,
  type DerivedTagTerminalLine,
} from "./terminal-ui.js";
import type { SearchScreenState } from "./search-screen-state.js";
import { formatResultPosition, formatSort } from "./search-screen-state.js";
import type { SearchWorkspaceEntry } from "./search-screen-workspace.js";
import {
  buildDraftCommandPaletteEntries,
  formatCountSummary,
  formatDraftStatus,
  formatMode,
  formatSearchCategory,
  formatSearchSubcategory,
} from "./search-screen-workspace.js";
import { buildResultCommandPaletteEntries } from "./search-screen-results.js";

export type SearchInteractionContext = "draft" | "result-list" | "result-detail";

export type SearchScreenIntent =
  | { type: "show_help" }
  | { type: "quit" }
  | { type: "edit_query" }
  | { type: "open_setup_commands" }
  | { type: "execute" }
  | { type: "back_to_app" }
  | { type: "move_workspace_selection"; delta: number }
  | { type: "workspace_selection_boundary"; boundary: "start" | "end" }
  | { type: "edit_selected_workspace" }
  | { type: "open_result_commands" }
  | { type: "toggle_pane" }
  | { type: "return_to_setup" }
  | { type: "open_preview" }
  | { type: "move_result_selection"; delta: number }
  | { type: "result_selection_boundary"; boundary: "start" | "end" }
  | { type: "return_to_result_list" }
  | { type: "move_detail"; delta: number }
  | { type: "detail_boundary"; boundary: "start" | "end" };

export function getSearchInteractionContext(state: SearchScreenState): SearchInteractionContext {
  if (state.layout === "draft") {
    return "draft";
  }

  return state.activePane === "list" ? "result-list" : "result-detail";
}

export function getSearchInteractionActions(state: SearchScreenState): TerminalInteractionAction[] {
  switch (getSearchInteractionContext(state)) {
    case "draft":
      return getSearchDraftInteractionActions();
    case "result-list":
      return getSearchResultListInteractionActions();
    case "result-detail":
      return getSearchResultDetailInteractionActions();
  }
}

export function getSearchDraftInteractionActions(): TerminalInteractionAction[] {
  return [
    { id: "edit" },
    { id: "execute" },
    { id: "search", label: "query" },
    { id: "commands" },
    { id: "help" },
    { id: "back" },
    { id: "quit", label: "back" },
  ];
}

export function getSearchResultListInteractionActions(): TerminalInteractionAction[] {
  return [
    { id: "back", label: "setup" },
    { id: "preview" },
    { id: "focus", label: "pane" },
    { id: "commands" },
    { id: "help" },
    { id: "quit", label: "back" },
  ];
}

export function getSearchResultDetailInteractionActions(): TerminalInteractionAction[] {
  return [
    { id: "back", label: "results" },
    { id: "focus", label: "pane" },
    { id: "commands" },
    { id: "help" },
    { id: "quit", label: "back" },
  ];
}

export function buildSearchFooterText(state: SearchScreenState, loadingMore: boolean): string {
  const context = getSearchInteractionContext(state);

  if (context === "draft") {
    return formatTerminalInteractionFooter([
      { id: "move", label: "select" },
      { id: "jump" },
      { id: "page" },
      { id: "edge" },
      ...getSearchInteractionActions(state),
    ]);
  }

  if (context === "result-list") {
    const footer = formatTerminalInteractionFooter([
      { id: "move", label: "select" },
      { id: "jump" },
      { id: "page" },
      { id: "edge" },
      ...getSearchInteractionActions(state),
    ]);
    return loadingMore ? `${footer}  Loading more...` : footer;
  }

  return formatTerminalInteractionFooter([
    { id: "scroll" },
    { id: "jump" },
    { id: "page" },
    { id: "edge" },
    ...getSearchInteractionActions(state),
  ]);
}

export function buildSearchHelpLines(
  state: SearchScreenState,
  workspaceEntries: SearchWorkspaceEntry[],
): DerivedTagTerminalLine[] {
  const context = getSearchInteractionContext(state);

  if (context === "draft") {
    const navigationActions: TerminalInteractionAction[] = [
      { id: "move", label: "select the setup row" },
      { id: "jump", helpText: "jump through the setup list" },
      { id: "page", helpText: "page through the setup list" },
      { id: "edge", helpText: "jump to the start or end of the setup list" },
    ];
    const actionActions: TerminalInteractionAction[] = [
      ...getSearchInteractionActions(state).map<TerminalInteractionAction>((action) => ({
        ...action,
        helpText:
          action.id === "edit"
            ? "edit the focused setup row or act on it"
            : action.id === "execute"
              ? "execute the current setup and switch to results"
              : action.id === "search"
                ? "edit the current query text"
                : action.id === "commands"
                  ? "open the setup command palette"
                  : action.id === "help"
                    ? "show search setup help"
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
        title: "Setup Commands",
        commands: buildDraftCommandPaletteEntries(workspaceEntries).map<TerminalInteractionCommand>((entry) => ({
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
  const resultActions: TerminalInteractionAction[] = getSearchInteractionActions(state).map((action) => ({
    ...action,
    helpText:
      action.id === "preview"
        ? "open the focused result preview"
        : action.id === "back" && context === "result-list"
          ? "return to Scope & Filters"
          : action.id === "back"
            ? "return to the result list"
            : action.id === "focus"
              ? "switch focus between results and preview"
              : action.id === "commands"
                ? "open the results command palette"
                : action.id === "help"
                  ? "show search results help"
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
      commands: buildResultCommandPaletteEntries(state).map<TerminalInteractionCommand>((entry) => ({
        label: entry.label,
        description: entry.description ?? "No additional details.",
        aliases: entry.aliases,
      })),
    },
  ]);
}

export function buildSearchSubtitle(
  state: SearchScreenState,
  countState: import("./search-screen-state.js").SearchCountState,
): string {
  const draft = `${formatMode(state.draft.mode)} | ${formatSearchCategory(state.draft.filters.category)} / ${formatSearchSubcategory(state.draft.filters.subcategory)}`;
  if (state.layout === "draft") {
    return `${draft} | ${formatCountSummary(countState, state.draft)} | ${formatDraftStatus(state)}`;
  }
  if (!state.session) {
    return `${draft} | no applied session`;
  }
  return `${draft} | ${formatSort(state.session.sort)} | ${formatResultPosition(state.resultSelectedIndex, state.session.total)} | ${formatDraftStatus(state)}`;
}

export function useSearchScreenInteractionRouter(options: {
  enabled: boolean;
  state: SearchScreenState;
  workspaceEntryCount: number;
  resultCount: number;
  selectionJumpSize: number;
  pageSize: number;
  maxDetailScroll: number;
  hasSelectedResult: boolean;
  onIntent: (intent: SearchScreenIntent) => void;
}): void {
  const listNavigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());
  const detailNavigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());

  useDerivedTagTerminalInput(
    (event) => {
      const interactionAction = resolveTerminalInteractionAction(event, getSearchInteractionActions(options.state));

      if (interactionAction?.id === "help") {
        options.onIntent({ type: "show_help" });
        return;
      }
      if (interactionAction?.id === "quit") {
        options.onIntent({ type: "quit" });
        return;
      }

      const listNavigation = resolveDerivedTagTerminalListNavigationAction(
        event,
        {
          pageSize: options.pageSize,
          jumpSize: options.selectionJumpSize,
          includeConfirmKeys: true,
        },
        listNavigationStateRef.current,
      );
      listNavigationStateRef.current = listNavigation.state;
      const detailNavigation = resolveDerivedTagTerminalListNavigationAction(
        event,
        {
          pageSize: options.pageSize,
          jumpSize: options.selectionJumpSize,
        },
        detailNavigationStateRef.current,
      );
      detailNavigationStateRef.current = detailNavigation.state;

      if (options.state.layout === "draft") {
        if (interactionAction?.id === "search") {
          options.onIntent({ type: "edit_query" });
          return;
        }
        if (interactionAction?.id === "commands") {
          options.onIntent({ type: "open_setup_commands" });
          return;
        }
        if (interactionAction?.id === "execute") {
          options.onIntent({ type: "execute" });
          return;
        }
        if (interactionAction?.id === "back") {
          options.onIntent({ type: "back_to_app" });
          return;
        }
        if (listNavigation.action?.kind === "move") {
          options.onIntent({ type: "move_workspace_selection", delta: listNavigation.action.delta });
          return;
        }
        if (listNavigation.action?.kind === "boundary") {
          options.onIntent({ type: "workspace_selection_boundary", boundary: listNavigation.action.boundary });
          return;
        }
        if (interactionAction?.id === "edit") {
          options.onIntent({ type: "edit_selected_workspace" });
        }
        return;
      }

      if (interactionAction?.id === "commands") {
        options.onIntent({ type: "open_result_commands" });
        return;
      }

      if (interactionAction?.id === "focus") {
        options.onIntent({ type: "toggle_pane" });
        return;
      }

      if (options.state.activePane === "list") {
        if (interactionAction?.id === "back") {
          options.onIntent({ type: "return_to_setup" });
          return;
        }
        if (interactionAction?.id === "preview" && options.hasSelectedResult) {
          options.onIntent({ type: "open_preview" });
          return;
        }
        if (listNavigation.action?.kind === "move") {
          options.onIntent({ type: "move_result_selection", delta: listNavigation.action.delta });
          return;
        }
        if (listNavigation.action?.kind === "boundary") {
          options.onIntent({ type: "result_selection_boundary", boundary: listNavigation.action.boundary });
        }
        return;
      }

      if (interactionAction?.id === "back") {
        options.onIntent({ type: "return_to_result_list" });
        return;
      }
      if (detailNavigation.action?.kind === "move") {
        options.onIntent({ type: "move_detail", delta: detailNavigation.action.delta });
        return;
      }
      if (detailNavigation.action?.kind === "boundary") {
        options.onIntent({ type: "detail_boundary", boundary: detailNavigation.action.boundary });
      }
    },
    options.enabled,
  );
}
