import React from "react";

import type { Pf2eTerminalAppServices } from "./app-services.js";
import type { SearchScreenAction, SearchScreenState } from "./search-screen-state.js";
import type { Pf2eTerminalSearchRequest } from "./search-service.js";
import type { SearchScreenOrigin } from "./search-workflow-types.js";
import type { DerivedTagTerminalApp } from "./terminal-ui.js";
import {
  buildDraftCommandPaletteEntries,
  buildFacetRemovalEntries,
  buildResultCommandPaletteEntries,
  formatLevelRange,
  parseLevelRangeInput,
  type SearchWorkspaceAction,
} from "./search-screen-model.js";
import type { SearchWorkspaceEntry } from "./search-screen-workspace.js";

export function useSearchWorkspaceActions({
  applyDraftUpdate,
  dispatch,
  executeRequest,
  exitSearchScreen,
  jumpToResultPosition,
  maxDetailScroll,
  onOpenFacetPicker,
  origin,
  resultCount,
  selectedWorkspaceEntry,
  showSearchHelp,
  state,
  terminal,
  user,
  workspaceEntries,
  chooseResultSort,
}: {
  applyDraftUpdate: (update: (request: Pf2eTerminalSearchRequest) => Pf2eTerminalSearchRequest) => void;
  dispatch: React.Dispatch<SearchScreenAction>;
  executeRequest: (request: Pf2eTerminalSearchRequest) => Promise<void>;
  exitSearchScreen: () => void;
  jumpToResultPosition: () => Promise<void>;
  maxDetailScroll: number;
  onOpenFacetPicker: () => Promise<void>;
  origin: SearchScreenOrigin;
  resultCount: number;
  selectedWorkspaceEntry?: SearchWorkspaceEntry;
  showSearchHelp: () => void;
  state: SearchScreenState;
  terminal: Pick<
    DerivedTagTerminalApp,
    | "pauseForAnyKey"
    | "promptCommandPalette"
    | "promptMultiSelectOption"
    | "promptOptionalSelectOption"
    | "promptPolicySelectOption"
    | "promptSelectOption"
    | "promptTextInput"
    | "showDialog"
  >;
  user: Pick<Pf2eTerminalAppServices["user"], "search">;
  workspaceEntries: SearchWorkspaceEntry[];
  chooseResultSort: () => Promise<void>;
}): {
  handleIntent: (intent: import("./search-screen-model.js").SearchScreenIntent) => void;
} {
  const editQueryText = React.useCallback(async () => {
    const queryText = await terminal.promptTextInput({
      title: "Query Text",
      prompt:
        state.draft.mode === "lookup"
          ? "Enter an exact or near-exact record name"
          : "Enter search text for the current query setup",
      defaultValue: state.draft.queryText,
      hint: state.draft.mode === "lookup" ? "Example: Raise Shield" : "Example: ghost ship captain",
    });

    if (queryText === undefined) {
      return;
    }

    applyDraftUpdate((request) => ({
      ...request,
      queryText,
    }));
  }, [applyDraftUpdate, state.draft.mode, state.draft.queryText, terminal]);

  const chooseMode = React.useCallback(async () => {
    const result = await terminal.promptSelectOption({
      title: "Workspace Mode",
      prompt: "Choose how the current query setup should execute",
      entries: user.search.getModeOptions().map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.draft.mode,
    });

    if (result.kind !== "selected") {
      return;
    }

    applyDraftUpdate((request) => ({
      ...request,
      mode: result.value,
    }));
  }, [applyDraftUpdate, state.draft.mode, terminal, user.search]);

  const chooseSearchProfile = React.useCallback(async () => {
    const result = await terminal.promptSelectOption({
      title: "Search Profile",
      prompt: "Choose the current profile for ranked search mode",
      entries: user.search.getProfileOptions().map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.draft.searchProfile,
    });

    if (result.kind === "selected") {
      applyDraftUpdate((request) => ({
        ...request,
        searchProfile: result.value,
      }));
    }
  }, [applyDraftUpdate, state.draft.searchProfile, terminal, user.search]);

  const chooseCategoryFilter = React.useCallback(async () => {
    const [allCategoryOption, ...categoryEntries] = user.search.getCategoryOptions();
    const result = await terminal.promptOptionalSelectOption({
      title: "Category Scope",
      prompt: "Choose the current category boundary",
      allOption: {
        label: allCategoryOption?.label ?? "Any Category",
        description: allCategoryOption?.description,
      },
      entries: categoryEntries.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.draft.filters.category ?? null,
    });

    if (result.kind === "cancelled") {
      return;
    }

    applyDraftUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        category: result.kind === "all" ? null : result.value,
        subcategory: null,
      },
    }));
  }, [applyDraftUpdate, state.draft.filters.category, terminal, user.search]);

  const chooseSubcategoryFilter = React.useCallback(async () => {
    if (!state.draft.filters.category) {
      await terminal.pauseForAnyKey("Choose a category before selecting a subcategory.");
      return;
    }

    const [allSubcategoryOption, ...subcategoryEntries] = user.search.getSubcategoryOptions(
      state.draft.filters.category,
    );
    const result = await terminal.promptOptionalSelectOption({
      title: "Subcategory Scope",
      prompt: "Choose the current subcategory boundary",
      allOption: {
        label: allSubcategoryOption?.label ?? "Any Subcategory",
        description: allSubcategoryOption?.description,
      },
      entries: subcategoryEntries.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.draft.filters.subcategory ?? null,
    });

    if (result.kind === "cancelled") {
      return;
    }

    applyDraftUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        subcategory: result.kind === "all" ? null : result.value,
      },
    }));
  }, [applyDraftUpdate, state.draft.filters.category, state.draft.filters.subcategory, terminal, user.search]);

  const chooseRarityFilter = React.useCallback(async () => {
    const options = user.search.getRarityOptions(state.draft.filters.category, state.draft.filters.subcategory);
    const selected = await terminal.promptPolicySelectOption({
      title: "Rarity Filter",
      prompt: "Cycle rarities through include and exclude. Press Esc or Left when finished.",
      allowedStates: ["any", "exclude"],
      entries: options.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValues: state.draft.filters.rarity,
    });

    applyDraftUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        rarity: {
          any: selected.any,
          all: [],
          exclude: selected.exclude,
        },
      },
    }));
  }, [
    applyDraftUpdate,
    state.draft.filters.category,
    state.draft.filters.rarity,
    state.draft.filters.subcategory,
    terminal,
    user.search,
  ]);

  const editLevelRange = React.useCallback(async () => {
    const input = await terminal.promptTextInput({
      title: "Level Range",
      prompt: "Enter `3-8`, `5`, `5+`, or `<=10`. Leave blank to clear.",
      defaultValue:
        state.draft.filters.levelMin === null && state.draft.filters.levelMax === null
          ? ""
          : formatLevelRange(state.draft).replaceAll("L", "").replace("<= ", "<="),
      hint: "Examples: 3-8 or <=5",
    });

    if (input === undefined) {
      return;
    }

    const parsed = parseLevelRangeInput(input);
    if (typeof parsed === "string") {
      await terminal.pauseForAnyKey(parsed);
      return;
    }

    applyDraftUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        levelMin: parsed.levelMin,
        levelMax: parsed.levelMax,
      },
    }));
  }, [applyDraftUpdate, state.draft, terminal]);

  const removeFacetFilter = React.useCallback(async () => {
    if (
      state.draft.filters.facets.length === 0 &&
      state.draft.filters.actionCost.any.length === 0 &&
      state.draft.filters.actionCost.all.length === 0 &&
      state.draft.filters.actionCost.exclude.length === 0
    ) {
      await terminal.pauseForAnyKey("There are no facet policies to clear from the current query setup.");
      return;
    }

    const selected = await terminal.promptMultiSelectOption({
      title: "Clear Facet Filter",
      prompt: "Toggle facet fields to clear. Press Esc or Left when finished.",
      entries: buildFacetRemovalEntries(state.draft.filters.facets, state.draft.filters.actionCost),
      selectedValues: [],
    });
    applyDraftUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        actionCost: selected.includes("actionCost") ? { any: [], all: [], exclude: [] } : request.filters.actionCost,
        facets: request.filters.facets.filter((facet) => !selected.includes(facet.field)),
      },
    }));
  }, [applyDraftUpdate, state.draft.filters.actionCost, state.draft.filters.facets, terminal]);

  const resetDraftWorkspace = React.useCallback(() => {
    const defaultRequest = user.search.createDefaultRequest();
    dispatch({ type: "set_draft", request: defaultRequest });
    dispatch({ type: "set_layout", layout: "draft", pane: "list" });
  }, [dispatch, user.search]);

  const runWorkspaceAction = React.useCallback(
    (action: SearchWorkspaceAction) => {
      const entry = workspaceEntries.find((candidate) => candidate.action === action);
      if (entry?.disabled) {
        return;
      }

      switch (action) {
        case "execute":
          void executeRequest(state.draft);
          return;
        case "mode":
          void chooseMode();
          return;
        case "query":
          void editQueryText();
          return;
        case "profile":
          void chooseSearchProfile();
          return;
        case "category":
          void chooseCategoryFilter();
          return;
        case "subcategory":
          void chooseSubcategoryFilter();
          return;
        case "levels":
          void editLevelRange();
          return;
        case "rarity":
          void chooseRarityFilter();
          return;
        case "addFacet":
          void onOpenFacetPicker();
          return;
        case "removeFacet":
          void removeFacetFilter();
          return;
        case "reset":
          resetDraftWorkspace();
          return;
        case "clearResults":
          dispatch({ type: "clear_results" });
          return;
      }
    },
    [
      chooseCategoryFilter,
      chooseMode,
      chooseRarityFilter,
      chooseSearchProfile,
      chooseSubcategoryFilter,
      dispatch,
      editLevelRange,
      editQueryText,
      executeRequest,
      onOpenFacetPicker,
      removeFacetFilter,
      resetDraftWorkspace,
      state.draft,
      workspaceEntries,
    ],
  );

  const openSelectedWorkspaceEntry = React.useCallback(() => {
    if (!selectedWorkspaceEntry || selectedWorkspaceEntry.disabled) {
      return;
    }
    runWorkspaceAction(selectedWorkspaceEntry.action);
  }, [runWorkspaceAction, selectedWorkspaceEntry]);

  const openDraftCommandPalette = React.useCallback(async () => {
    const selected = await terminal.promptCommandPalette({
      title: "Search Setup Commands",
      prompt: "Filter setup commands",
      entries: buildDraftCommandPaletteEntries(workspaceEntries),
    });
    if (!selected) {
      return;
    }
    if (workspaceEntries.find((entry) => entry.action === selected)?.disabled) {
      return;
    }
    runWorkspaceAction(selected);
  }, [runWorkspaceAction, terminal, workspaceEntries]);

  const openResultCommandPalette = React.useCallback(async () => {
    const selected = await terminal.promptCommandPalette({
      title: "Result Commands",
      prompt: "Filter result commands",
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
    if (selected === "openSetup") {
      dispatch({ type: "set_layout", layout: "draft", pane: "list" });
    }
  }, [chooseResultSort, dispatch, jumpToResultPosition, origin, state, terminal]);

  const handleIntent = React.useCallback(
    (intent: import("./search-screen-model.js").SearchScreenIntent) => {
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
        case "open_setup_commands":
          void openDraftCommandPalette();
          return;
        case "execute":
          void executeRequest(state.draft);
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
        case "return_to_setup":
          if (origin === "ontology") {
            exitSearchScreen();
            return;
          }
          dispatch({ type: "set_layout", layout: "draft", pane: "list" });
          return;
        case "open_preview":
          dispatch({ type: "set_active_pane", pane: "detail" });
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
      openDraftCommandPalette,
      openResultCommandPalette,
      openSelectedWorkspaceEntry,
      origin,
      resultCount,
      showSearchHelp,
      state.activePane,
      state.draft,
      workspaceEntries.length,
    ],
  );

  return {
    handleIntent,
  };
}
