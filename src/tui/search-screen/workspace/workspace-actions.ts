import React from "react";

import type { SearchScreenAction, SearchScreenState } from "../state.js";
import type { SearchStructuredEditorSession } from "../query-field-builder/query-field-builder-session.js";
import type { Pf2eTerminalSearchQuery } from "../../search/service.js";
import type { SearchScreenOrigin } from "../workflow-types.js";
import {
  buildEditorCommandPaletteEntries,
  buildResultCommandPaletteEntries,
  decodeQueryNodeActionPath,
  isQueryNodeAction,
  type SearchWorkspaceAction,
} from "../model.js";
import {
  getSearchQueryExcludeText,
  getSearchQuerySearchProfile,
  getSearchQueryText,
  setSearchQueryMetadataTree,
  setSearchQueryExcludeText,
  setSearchQuerySearchProfile,
  setSearchQueryText,
} from "../../search/query-state.js";
import type { SearchWorkspaceEntry } from "./workspace.js";
import { useSearchStructuredEditorActions } from "../structured-editor-actions.js";
import type {
  OpenSearchFilterExplorer,
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
  SearchWorkspaceUser,
} from "./workspace-action-types.js";

export function useSearchWorkspaceActions({
  applyQueryUpdate,
  dispatch,
  executeRequest,
  exitSearchScreen,
  jumpToResultPosition,
  maxDetailScroll,
  openFilterExplorer,
  origin,
  prompts,
  resultCount,
  selectedWorkspaceEntry,
  showSearchHelp,
  state,
  terminal,
  user,
  workspaceEntries,
  chooseResultSort,
}: {
  applyQueryUpdate: (update: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery) => void;
  dispatch: React.Dispatch<SearchScreenAction>;
  executeRequest: (query: Pf2eTerminalSearchQuery) => Promise<void>;
  exitSearchScreen: () => void;
  jumpToResultPosition: () => Promise<void>;
  maxDetailScroll: number;
  openFilterExplorer: OpenSearchFilterExplorer;
  origin: SearchScreenOrigin;
  prompts: SearchWorkspacePromptAdapters;
  resultCount: number;
  selectedWorkspaceEntry?: SearchWorkspaceEntry;
  showSearchHelp: () => void;
  state: SearchScreenState;
  terminal: SearchWorkspaceTerminal;
  user: SearchWorkspaceUser;
  workspaceEntries: SearchWorkspaceEntry[];
  chooseResultSort: () => Promise<void>;
}): {
  handleIntent: (intent: import("../model.js").SearchScreenIntent) => void;
  structuredEditorSession: SearchStructuredEditorSession | null;
} {
  const editQueryText = React.useCallback(async () => {
    if (state.query.mode === "browse") {
      return;
    }

    const queryText = await prompts.promptTextInput({
      title: state.query.mode === "lookup" ? "Lookup Text" : "Search Text",
      prompt:
        state.query.mode === "lookup"
          ? "Enter an exact or near-exact record name."
          : "Enter search text for the current query.",
      defaultValue: getSearchQueryText(state.query),
      hint: state.query.mode === "lookup" ? "Example: Raise Shield" : "Example: ghost ship captain",
      presentation: "centered",
    });

    if (queryText === undefined) {
      return;
    }

    applyQueryUpdate((request) => setSearchQueryText(request, queryText));
  }, [applyQueryUpdate, prompts, state.query]);

  const editExcludeText = React.useCallback(async () => {
    if (state.query.mode !== "search") {
      return;
    }

    const excludeText = await prompts.promptTextInput({
      title: "Exclude Text",
      prompt: "Enter text that ranked search should exclude from lexical matches",
      defaultValue: getSearchQueryExcludeText(state.query),
      hint: "Example: skeleton",
    });

    if (excludeText === undefined) {
      return;
    }

    applyQueryUpdate((request) => setSearchQueryExcludeText(request, excludeText));
  }, [applyQueryUpdate, prompts, state.query]);

  const chooseMode = React.useCallback(async () => {
    const result = await prompts.promptSelectOption({
      title: "Choose Search Mode",
      prompt: "",
      entries: user.search.getModeOptions().map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
        detailLines: [{ text: option.description }],
      })),
      presentation: "centered",
      choiceLayout: "horizontal",
      filtering: false,
      selectedValue: state.query.mode,
    });

    if (result.kind !== "selected" || result.value === state.query.mode) {
      return;
    }

    applyQueryUpdate((request) => ({
      ...user.search.createDefaultQuery(result.value),
      limit: request.limit,
    }));
  }, [applyQueryUpdate, prompts, state.query.mode, user.search]);

  const chooseSearchProfile = React.useCallback(async () => {
    const result = await prompts.promptSelectOption({
      title: "Search Profile",
      prompt: "Choose the current profile for ranked search mode",
      entries: user.search.getProfileOptions().map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: getSearchQuerySearchProfile(state.query) ?? "balanced",
    });

    if (result.kind === "selected") {
      applyQueryUpdate((request) => setSearchQuerySearchProfile(request, result.value));
    }
  }, [applyQueryUpdate, prompts, state.query, user.search]);
  const { openStructuredDraftSession, structuredEditorSession } = useSearchStructuredEditorActions({
    applyQueryUpdate,
    currentQuery: state.query,
    openFilterExplorer,
    prompts,
    terminal,
    user,
  });

  const resetQueryEditor = React.useCallback(() => {
    const defaultQuery = user.search.createDefaultQuery();
    dispatch({ type: "set_query", query: defaultQuery });
    dispatch({ type: "set_layout", layout: "editor", pane: "list" });
  }, [dispatch, user.search]);

  const runWorkspaceAction = React.useCallback(
    (action: SearchWorkspaceAction) => {
      const entry = workspaceEntries.find((candidate) => candidate.action === action);
      if (entry?.disabled) {
        return;
      }

      if (action === "execute") {
        void executeRequest(state.query);
        return;
      }
      if (action === "mode") {
        void chooseMode();
        return;
      }
      if (action === "query") {
        void editQueryText();
        return;
      }
      if (action === "exclude") {
        void editExcludeText();
        return;
      }
      if (action === "profile") {
        void chooseSearchProfile();
        return;
      }
      if (action === "queryTreeRoot") {
        openStructuredDraftSession({ kind: "queryTreeRoot" });
        return;
      }
      if (action === "addQueryPart") {
        openStructuredDraftSession({ kind: "addQueryPart" });
        return;
      }
      if (action === "clearClauses") {
        openStructuredDraftSession({ kind: "queryNode", path: [] }, setSearchQueryMetadataTree(state.query, null));
        return;
      }
      if (action === "reset") {
        resetQueryEditor();
        return;
      }
      if (action === "clearResults") {
        dispatch({ type: "clear_results" });
        return;
      }

      if (isQueryNodeAction(action)) {
        const path = decodeQueryNodeActionPath(action);
        if (!path) {
          return;
        }
        openStructuredDraftSession({ kind: "queryNode", path });
        return;
      }
    },
    [
      chooseMode,
      chooseSearchProfile,
      dispatch,
      editExcludeText,
      editQueryText,
      executeRequest,
      openStructuredDraftSession,
      resetQueryEditor,
      state.query,
      workspaceEntries,
    ],
  );

  const openSelectedWorkspaceEntry = React.useCallback(() => {
    if (!selectedWorkspaceEntry || selectedWorkspaceEntry.disabled) {
      return;
    }
    runWorkspaceAction(selectedWorkspaceEntry.action);
  }, [runWorkspaceAction, selectedWorkspaceEntry]);

  const openEditorCommandPalette = React.useCallback(async () => {
    const selected = await prompts.promptCommandPalette({
      title: "Query Editor Commands",
      prompt: "Filter query editor commands",
      entries: buildEditorCommandPaletteEntries(workspaceEntries),
    });
    if (!selected) {
      return;
    }
    if (workspaceEntries.find((entry) => entry.action === selected)?.disabled) {
      return;
    }
    runWorkspaceAction(selected);
  }, [prompts, runWorkspaceAction, workspaceEntries]);

  const openResultCommandPalette = React.useCallback(async () => {
    const selected = await prompts.promptCommandPalette({
      title: "Result Actions",
      prompt: "Filter result actions",
      entries: buildResultCommandPaletteEntries(state, origin),
    });
    if (selected === "jumpToResult") {
      void jumpToResultPosition();
      return;
    }
    if (selected === "sortResults") {
      void chooseResultSort();
      return;
    }
    if (selected === "openEditor") {
      dispatch({ type: "set_layout", layout: "editor", pane: "list" });
    }
  }, [chooseResultSort, dispatch, jumpToResultPosition, origin, prompts, state]);

  const handleIntent = React.useCallback(
    (intent: import("../model.js").SearchScreenIntent) => {
      switch (intent.type) {
        case "show_help":
          showSearchHelp();
          return;
        case "quit":
          exitSearchScreen();
          return;
        case "edit_query":
          void editQueryText();
          return;
        case "open_editor_commands":
          void openEditorCommandPalette();
          return;
        case "execute":
          void executeRequest(state.query);
          return;
        case "back_to_app":
          exitSearchScreen();
          return;
        case "move_workspace_selection":
          dispatch({
            type: "move_workspace_selection",
            delta: intent.delta,
            itemCount: workspaceEntries.length,
          });
          return;
        case "workspace_selection_boundary":
          dispatch({
            type: "workspace_selection_boundary",
            boundary: intent.boundary,
            itemCount: workspaceEntries.length,
          });
          return;
        case "edit_selected_workspace":
          openSelectedWorkspaceEntry();
          return;
        case "open_result_commands":
          void openResultCommandPalette();
          return;
        case "toggle_pane":
          dispatch({ type: "set_active_pane", pane: state.activePane === "list" ? "detail" : "list" });
          return;
        case "return_to_editor":
          if (origin === "ontology") {
            exitSearchScreen();
            return;
          }
          dispatch({ type: "set_layout", layout: "editor", pane: "list" });
          return;
        case "move_result_selection":
          dispatch({ type: "move_result_selection", delta: intent.delta, itemCount: resultCount });
          return;
        case "result_selection_boundary":
          dispatch({
            type: "result_selection_boundary",
            boundary: intent.boundary,
            itemCount: resultCount,
          });
          return;
        case "return_to_result_list":
          dispatch({ type: "set_active_pane", pane: "list" });
          return;
        case "move_detail":
          dispatch({ type: "move_detail", delta: intent.delta, maxDetailScroll });
          return;
        case "detail_boundary":
          dispatch({ type: "detail_boundary", boundary: intent.boundary, maxDetailScroll });
          return;
      }
    },
    [
      dispatch,
      editQueryText,
      executeRequest,
      exitSearchScreen,
      maxDetailScroll,
      openEditorCommandPalette,
      openResultCommandPalette,
      openSelectedWorkspaceEntry,
      origin,
      resultCount,
      showSearchHelp,
      state.activePane,
      state.query,
      workspaceEntries.length,
    ],
  );

  return {
    handleIntent,
    structuredEditorSession,
  };
}
