import type { TerminalInteractionAction, TerminalInteractionCommand } from "./interaction-bindings.js";
import { buildTerminalInteractionHelpLines, formatTerminalInteractionFooter } from "./interaction-bindings.js";
import type { DerivedTagTerminalLine } from "./terminal-ui.js";
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
  if (state.layout === "draft") {
    return formatTerminalInteractionFooter([
      { id: "move", label: "select" },
      { id: "jump" },
      { id: "page" },
      { id: "edge" },
      ...getSearchDraftInteractionActions(),
    ]);
  }

  if (state.activePane === "list") {
    const footer = formatTerminalInteractionFooter([
      { id: "move", label: "select" },
      { id: "jump" },
      { id: "page" },
      { id: "edge" },
      ...getSearchResultListInteractionActions(),
    ]);
    return loadingMore ? `${footer}  Loading more...` : footer;
  }

  return formatTerminalInteractionFooter([
    { id: "scroll" },
    { id: "jump" },
    { id: "page" },
    { id: "edge" },
    ...getSearchResultDetailInteractionActions(),
  ]);
}

export function buildSearchHelpLines(
  state: SearchScreenState,
  workspaceEntries: SearchWorkspaceEntry[],
): DerivedTagTerminalLine[] {
  if (state.layout === "draft") {
    const navigationActions: TerminalInteractionAction[] = [
      { id: "move", label: "select the setup row" },
      { id: "jump", helpText: "jump through the setup list" },
      { id: "page", helpText: "page through the setup list" },
      { id: "edge", helpText: "jump to the start or end of the setup list" },
    ];
    const actionActions: TerminalInteractionAction[] = [
      ...getSearchDraftInteractionActions().map<TerminalInteractionAction>((action) => ({
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
      id: state.activePane === "list" ? "move" : "scroll",
      label: state.activePane === "list" ? "move through results" : "scroll the preview",
    },
    {
      id: "jump",
      helpText: state.activePane === "list" ? "jump through the active result pane" : "jump through the preview pane",
    },
    {
      id: "page",
      helpText: state.activePane === "list" ? "page through the active result pane" : "page through the preview pane",
    },
    { id: "edge", helpText: "jump to the start or end of the active pane" },
  ];
  const resultActions: TerminalInteractionAction[] = (
    state.activePane === "list" ? getSearchResultListInteractionActions() : getSearchResultDetailInteractionActions()
  ).map((action) => ({
    ...action,
    helpText:
      action.id === "preview"
        ? "open the focused result preview"
        : action.id === "back" && state.activePane === "list"
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
