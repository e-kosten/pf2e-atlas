import React from "react";

import type { SearchCategory, SearchProfile } from "../types.js";
import { usePf2eTerminalAppServices } from "./app-service-context.js";
import type {
  Pf2eTerminalSearchCategoryOption,
  Pf2eTerminalSearchProfileOption,
  Pf2eTerminalSearchRequest,
  Pf2eTerminalSearchSession,
} from "./search-service.js";
import {
  TerminalPaneScreen,
  TerminalTwoPaneScreen,
  getNormalizedKeyName,
  getRenderedTerminalLineCount,
  getTerminalPaneBodyHeight,
  getTerminalTwoPaneDetailWidth,
  moveSelection,
  sliceRenderedTerminalLines,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
  useDerivedTagTerminalSize,
  type DerivedTagTerminalLine,
} from "./terminal-ui.js";
import {
  buildOntologyExplorerEntityDetailLines,
} from "./ontology-explorer/entity-page.js";
import {
  mapNormalizedRecordToOntologyExplorerEntityRecord,
} from "./ontology-explorer/entity-record.js";
import {
  createDerivedTagTerminalTwoPaneState,
  getDerivedTagTerminalTwoPaneLayoutMode,
  reduceDerivedTagTerminalTwoPaneState,
  type DerivedTagTerminalTwoPaneAction,
  type DerivedTagTerminalTwoPaneState,
} from "./two-pane-state.js";
import { clampWindowStart } from "./list-utils.js";

type SearchPromptMode = "lookup" | "search";

type SearchScreenState = DerivedTagTerminalTwoPaneState & {
  queryDefaults: {
    category: SearchCategory | null;
    limit: number;
    mode: SearchPromptMode;
    queryText: string;
    searchProfile: SearchProfile;
  };
  selectedIndex: number;
  session: Pf2eTerminalSearchSession | null;
};

type SearchScreenAction =
  | DerivedTagTerminalTwoPaneAction
  | { type: "move_selection"; delta: number }
  | { type: "selection_boundary"; boundary: "start" | "end" }
  | { type: "set_category"; category: SearchCategory | null }
  | { type: "set_profile"; searchProfile: SearchProfile }
  | { type: "set_session"; session: Pf2eTerminalSearchSession }
  | { type: "clear_results" };

const SEARCH_LEFT_WIDTH = 48;

function createInitialSearchScreenState(): SearchScreenState {
  return {
    ...createDerivedTagTerminalTwoPaneState(),
    queryDefaults: {
      category: null,
      limit: 20,
      mode: "search",
      queryText: "",
      searchProfile: "balanced",
    },
    selectedIndex: 0,
    session: null,
  };
}

function searchScreenReducer(state: SearchScreenState, action: SearchScreenAction): SearchScreenState {
  switch (action.type) {
    case "toggle_focus":
    case "toggle_layout":
    case "leave_detail":
    case "move_detail":
    case "detail_boundary":
      return reduceDerivedTagTerminalTwoPaneState(state, action);
    case "move_selection":
      return {
        ...state,
        detailScroll: 0,
        selectedIndex: moveSelection(state.selectedIndex, action.delta, state.session?.results.length ?? 0),
      };
    case "selection_boundary":
      return {
        ...state,
        detailScroll: 0,
        selectedIndex: !state.session || state.session.results.length <= 0
          ? 0
          : action.boundary === "start"
            ? 0
            : state.session.results.length - 1,
      };
    case "set_category":
      return {
        ...state,
        queryDefaults: {
          ...state.queryDefaults,
          category: action.category,
        },
      };
    case "set_profile":
      return {
        ...state,
        queryDefaults: {
          ...state.queryDefaults,
          searchProfile: action.searchProfile,
        },
      };
    case "set_session":
      return {
        ...createDerivedTagTerminalTwoPaneState(),
        queryDefaults: {
          ...state.queryDefaults,
          category: action.session.request.category,
          mode: action.session.request.mode,
          queryText: action.session.request.queryText,
          searchProfile: action.session.request.searchProfile,
        },
        selectedIndex: 0,
        session: action.session,
      };
    case "clear_results":
      return {
        ...createDerivedTagTerminalTwoPaneState(),
        queryDefaults: state.queryDefaults,
        selectedIndex: 0,
        session: null,
      };
    default:
      return state;
  }
}

function buildSearchResultLabel(record: Pf2eTerminalSearchSession["results"][number]): string {
  const scope = record.subcategory ? `${record.category}/${record.subcategory}` : record.category;
  const level = record.level === null ? "-" : String(record.level);
  return `${record.name} | ${scope} | lvl ${level} | ${record.packLabel}`;
}

function formatSearchCategory(category: SearchCategory | null): string {
  if (!category) {
    return "any category";
  }
  return category === "characterCreation"
    ? "character creation"
    : category;
}

function buildSearchListLines(
  session: Pf2eTerminalSearchSession | null,
  selectedIndex: number,
  bodyHeight: number,
): DerivedTagTerminalLine[] {
  if (!session || session.results.length === 0) {
    return [
      { text: "No search results yet.", tone: "dim" },
      { text: "" },
      { text: "/ run search  l exact lookup  p profile  f category", tone: "accent" },
    ];
  }

  const visibleCount = Math.max(1, bodyHeight);
  const windowStart = clampWindowStart(selectedIndex, session.results.length, visibleCount);

  return session.results.slice(windowStart, windowStart + visibleCount).map((record, offset) => ({
    text: buildSearchResultLabel(record),
    tone: windowStart + offset === selectedIndex ? "selected" : "default",
    noWrap: true,
  }));
}

function buildSearchIntroLines(state: SearchScreenState): DerivedTagTerminalLine[] {
  return [
    { text: "Search and Lookup", tone: "section" },
    { text: "This surface uses the same indexed data service as the MCP server." },
    { text: "" },
    { text: "Current Search Defaults", tone: "section" },
    { text: `Search profile: ${state.queryDefaults.searchProfile}`, indent: 2 },
    { text: `Category filter: ${formatSearchCategory(state.queryDefaults.category)}`, indent: 2 },
    { text: `Page size: ${state.queryDefaults.limit}`, indent: 2 },
    { text: "" },
    { text: "Commands", tone: "section" },
    { text: "/ run ranked or semantic search", indent: 2 },
    { text: "l run exact name lookup", indent: 2 },
    { text: "p choose the default search profile", indent: 2 },
    { text: "f choose the category filter", indent: 2 },
    { text: "c clear the current result set", indent: 2 },
  ];
}

function buildSearchDetailLines(record: Pf2eTerminalSearchSession["results"][number] | undefined, state: SearchScreenState): DerivedTagTerminalLine[] {
  if (!record || !state.session) {
    return [
      ...buildSearchIntroLines(state),
      { text: "" },
      { text: state.queryDefaults.queryText
        ? `Last query: ${state.queryDefaults.queryText}`
        : "Run a search or lookup to populate records.", tone: "dim" },
    ];
  }

  const request = state.session.request;
  return [
    { text: request.mode === "lookup" ? "Exact Lookup Result" : "Search Result", tone: "section" },
    { text: `Query: ${request.queryText || "(none)"}` },
    { text: `Category: ${formatSearchCategory(request.category)}` },
    { text: `Search profile: ${state.session.searchProfile ?? "lookup"}` },
    { text: "" },
    ...buildOntologyExplorerEntityDetailLines(mapNormalizedRecordToOntologyExplorerEntityRecord(record)),
  ];
}

function buildSearchSubtitle(state: SearchScreenState): string {
  const searchDefaults = `profile ${state.queryDefaults.searchProfile} | ${formatSearchCategory(state.queryDefaults.category)}`;
  if (!state.session) {
    return `Run exact lookup or indexed search against the PF2E corpus | ${searchDefaults}`;
  }

  return `${state.session.request.mode} | ${state.session.total} result${state.session.total === 1 ? "" : "s"} | ${searchDefaults} | query "${state.session.request.queryText}"`;
}

function buildSearchFilterEntries(options: Pf2eTerminalSearchCategoryOption[]): Array<{
  value: string;
  label: string;
  description: string;
}> {
  return options.map((option) => ({
    value: option.value ?? "__all__",
    label: option.label,
    description: option.description,
  }));
}

function buildSearchProfileEntries(options: Pf2eTerminalSearchProfileOption[]): Array<{
  value: SearchProfile;
  label: string;
  description: string;
}> {
  return options.map((option) => ({
    value: option.value,
    label: option.label,
    description: option.description,
  }));
}

export function SearchScreen({
  onBack,
}: {
  onBack: () => void;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const { search } = usePf2eTerminalAppServices();
  const size = useDerivedTagTerminalSize();
  const [busy, setBusy] = React.useState(false);
  const [state, dispatch] = React.useReducer(searchScreenReducer, undefined, createInitialSearchScreenState);

  const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(size.height, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
  const layoutMode = getDerivedTagTerminalTwoPaneLayoutMode(state);
  const detailLines = buildSearchDetailLines(state.session?.results[state.selectedIndex], state);
  const selectionJumpSize = Math.max(1, Math.floor(bodyHeight / 2));
  const pageSize = Math.max(1, bodyHeight - 1);
  const renderedDetailLineCount = getRenderedTerminalLineCount(detailLines, getTerminalTwoPaneDetailWidth(size.width, layoutMode, SEARCH_LEFT_WIDTH));
  const maxDetailScroll = Math.max(0, renderedDetailLineCount - bodyHeight);
  const detailScroll = Math.min(state.detailScroll, maxDetailScroll);

  const runPrompt = React.useCallback(async (mode: SearchPromptMode) => {
    const query = await terminal.promptTextInput({
      title: mode === "lookup" ? "Lookup PF2E Record" : "Search PF2E Records",
      prompt: mode === "lookup"
        ? "Enter an exact or near-exact record name"
        : "Enter a short natural-language search phrase",
      defaultValue: mode === state.queryDefaults.mode ? state.queryDefaults.queryText : "",
      hint: mode === "lookup"
        ? "Example: Raise Shield"
        : "Example: ghost ship captain or battlefield control spell",
    });

    if (query === undefined) {
      return;
    }

    const trimmed = query.trim();
    if (!trimmed) {
      dispatch({ type: "clear_results" });
      return;
    }

    const request: Pf2eTerminalSearchRequest = {
      ...state.queryDefaults,
      mode,
      queryText: trimmed,
    };

    setBusy(true);
    try {
      const session = await search.runQuery(request);
      dispatch({ type: "set_session", session });
    } catch (error) {
      await terminal.pauseForAnyKey(`Search failed: ${(error as Error).message}`);
    } finally {
      setBusy(false);
    }
  }, [search, state.queryDefaults, terminal]);

  const chooseSearchProfile = React.useCallback(async () => {
    const selected = await terminal.promptSelectOption({
      title: "Search Profile",
      prompt: "Choose the default search profile",
      entries: buildSearchProfileEntries(search.getProfileOptions()),
      selectedValue: state.queryDefaults.searchProfile,
    });

    if (selected) {
      dispatch({ type: "set_profile", searchProfile: selected });
    }
  }, [search, state.queryDefaults.searchProfile, terminal]);

  const chooseCategoryFilter = React.useCallback(async () => {
    const selected = await terminal.promptSelectOption({
      title: "Category Filter",
      prompt: "Choose the default category scope",
      entries: buildSearchFilterEntries(search.getCategoryOptions()),
      selectedValue: state.queryDefaults.category ?? "__all__",
    });

    if (selected !== undefined) {
      dispatch({ type: "set_category", category: selected === "__all__" ? null : selected as SearchCategory });
    }
  }, [search, state.queryDefaults.category, terminal]);

  useDerivedTagTerminalInput((input, key) => {
    if (busy) {
      return;
    }

    const normalized = getNormalizedKeyName(input, key);

    if (normalized === "ctrl_c" || normalized === "q") {
      onBack();
      return;
    }
    if (state.activePane === "detail" && (normalized === "escape" || normalized === "backspace")) {
      dispatch({ type: "leave_detail" });
      return;
    }
    if (state.activePane === "list" && (normalized === "escape" || normalized === "backspace")) {
      onBack();
      return;
    }
    if (normalized === "tab" || normalized === "shift_tab" || normalized === "w") {
      dispatch({ type: "toggle_focus" });
      return;
    }
    if (normalized === "z") {
      dispatch({ type: "toggle_layout" });
      return;
    }
    if (normalized === "slash") {
      void runPrompt("search");
      return;
    }
    if (normalized === "l") {
      void runPrompt("lookup");
      return;
    }
    if (normalized === "p") {
      void chooseSearchProfile();
      return;
    }
    if (normalized === "f") {
      void chooseCategoryFilter();
      return;
    }
    if (normalized === "c") {
      dispatch({ type: "clear_results" });
      return;
    }

    if (state.activePane === "list") {
      if (normalized === "up" || normalized === "k") {
        dispatch({ type: "move_selection", delta: -1 });
        return;
      }
      if (normalized === "down" || normalized === "j") {
        dispatch({ type: "move_selection", delta: 1 });
        return;
      }
      if (normalized === "ctrl_u") {
        dispatch({ type: "move_selection", delta: -selectionJumpSize });
        return;
      }
      if (normalized === "ctrl_d") {
        dispatch({ type: "move_selection", delta: selectionJumpSize });
        return;
      }
      if (normalized === "page_up" || normalized === "b") {
        dispatch({ type: "move_selection", delta: -pageSize });
        return;
      }
      if (normalized === "page_down" || normalized === "space") {
        dispatch({ type: "move_selection", delta: pageSize });
        return;
      }
      if (normalized === "home") {
        dispatch({ type: "selection_boundary", boundary: "start" });
        return;
      }
      if (normalized === "end") {
        dispatch({ type: "selection_boundary", boundary: "end" });
        return;
      }
      if ((normalized === "right" || normalized === "enter") && (state.session?.results.length ?? 0) > 0) {
        dispatch({ type: "toggle_focus" });
      }
      return;
    }

    if (normalized === "up" || normalized === "k") {
      dispatch({ type: "move_detail", delta: -1, maxDetailScroll });
      return;
    }
    if (normalized === "down" || normalized === "j") {
      dispatch({ type: "move_detail", delta: 1, maxDetailScroll });
      return;
    }
    if (normalized === "ctrl_u") {
      dispatch({ type: "move_detail", delta: -selectionJumpSize, maxDetailScroll });
      return;
    }
    if (normalized === "ctrl_d") {
      dispatch({ type: "move_detail", delta: selectionJumpSize, maxDetailScroll });
      return;
    }
    if (normalized === "page_up" || normalized === "b") {
      dispatch({ type: "move_detail", delta: -pageSize, maxDetailScroll });
      return;
    }
    if (normalized === "page_down" || normalized === "space") {
      dispatch({ type: "move_detail", delta: pageSize, maxDetailScroll });
      return;
    }
    if (normalized === "home") {
      dispatch({ type: "detail_boundary", boundary: "start", maxDetailScroll });
      return;
    }
    if (normalized === "end") {
      dispatch({ type: "detail_boundary", boundary: "end", maxDetailScroll });
    }
  }, !busy);

  if (layoutMode === "detail-only") {
    return (
      <TerminalPaneScreen
        title="Search"
        subtitle={`${buildSearchSubtitle(state)} | focused detail`}
        pane={{
          title: "[FOCUSED DETAIL] Selected Record",
          lines: sliceRenderedTerminalLines(
            detailLines,
            size.width,
            detailScroll,
            bodyHeight,
          ),
          active: true,
        }}
        footer={[
          {
            text: "z split-view  Tab/w list focus  Up/Down or j/k scroll  Ctrl+U/D jump  Space/b page  Home/End edge  Esc/backspace list  / search  l lookup  p profile  f category  c clear  q back",
            tone: "dim",
          },
          {
            text: `${state.activePane} focus | detail-only layout | Detail scroll ${detailScroll}/${maxDetailScroll}`,
            tone: "accent",
          },
        ]}
      />
    );
  }

  return (
    <TerminalTwoPaneScreen
      title="Search"
      subtitle={buildSearchSubtitle(state)}
      left={{
        title: state.activePane === "list" ? "[RESULTS] Search Results" : "Search Results",
        lines: buildSearchListLines(state.session, state.selectedIndex, bodyHeight),
        active: state.activePane === "list",
      }}
      right={{
        title: state.activePane === "detail" ? "[DETAIL] Selected Record" : "Selected Record",
        lines: sliceRenderedTerminalLines(
          detailLines,
          getTerminalTwoPaneDetailWidth(size.width, layoutMode, SEARCH_LEFT_WIDTH),
          detailScroll,
          bodyHeight,
        ),
        active: state.activePane === "detail",
      }}
      footer={[
        {
          text: state.activePane === "list"
            ? "Tab/w focus  z detail-only  Up/Down or j/k move  Ctrl+U/D jump  Space/b page  Home/End edge  Enter/right detail  / search  l lookup  p profile  f category  c clear  q back"
            : "Tab/w focus  z detail-only  Up/Down or j/k scroll  Ctrl+U/D jump  Space/b page  Home/End edge  Esc/backspace list  / search  l lookup  p profile  f category  c clear  q back",
          tone: "dim",
        },
        {
          text: state.session
            ? `Query: ${state.session.request.queryText} | showing ${state.session.results.length}/${state.session.total} | ${formatSearchCategory(state.session.request.category)}`
            : `Defaults: ${state.queryDefaults.searchProfile} | ${formatSearchCategory(state.queryDefaults.category)}`,
          tone: "accent",
        },
      ]}
      leftWidth={SEARCH_LEFT_WIDTH}
    />
  );
}
