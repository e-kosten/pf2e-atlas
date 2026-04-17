import React from "react";

import type { NormalizedRecord, SearchProfile } from "../types.js";
import { usePf2eTerminalAppServices } from "./app-service-context.js";
import {
  TerminalPaneScreen,
  TerminalTwoPaneScreen,
  getNormalizedKeyName,
  getRenderedTerminalLineCount,
  getTerminalPaneBodyHeight,
  getTerminalTwoPaneDetailWidth,
  moveSelection,
  normalizeTerminalTwoPaneLayoutMode,
  sliceRenderedTerminalLines,
  toggleTerminalTwoPaneFocus,
  toggleTerminalTwoPaneLayoutMode,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
  useDerivedTagTerminalSize,
  type DerivedTagTerminalLine,
  type DerivedTagTerminalTwoPaneFocus,
  type DerivedTagTerminalTwoPaneLayoutMode,
} from "./terminal-ui.js";
import {
  buildOntologyExplorerEntityDetailLines,
} from "./ontology-explorer/entity-page.js";
import {
  mapNormalizedRecordToOntologyExplorerEntityRecord,
} from "./ontology-explorer/entity-record.js";
import { clampWindowStart } from "./list-utils.js";

type SearchPromptMode = "lookup" | "search";

type SearchScreenState = {
  activePane: DerivedTagTerminalTwoPaneFocus;
  detailScroll: number;
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode;
  mode: SearchPromptMode;
  queryText: string;
  results: NormalizedRecord[];
  searchMode: "idle" | "lookup" | "search";
  searchProfile: SearchProfile | null;
  selectedIndex: number;
  total: number;
};

type SearchScreenAction =
  | { type: "toggle_focus" }
  | { type: "toggle_layout" }
  | { type: "leave_detail" }
  | { type: "move_selection"; delta: number }
  | { type: "selection_boundary"; boundary: "start" | "end" }
  | { type: "move_detail"; delta: number; maxDetailScroll: number }
  | { type: "detail_boundary"; boundary: "start" | "end"; maxDetailScroll: number }
  | { type: "set_results"; mode: SearchPromptMode; queryText: string; results: NormalizedRecord[]; total: number; searchProfile: SearchProfile | null }
  | { type: "clear_results" };

const SEARCH_LEFT_WIDTH = 48;

function createInitialSearchScreenState(): SearchScreenState {
  return {
    activePane: "list",
    detailScroll: 0,
    layoutMode: "split",
    mode: "search",
    queryText: "",
    results: [],
    searchMode: "idle",
    searchProfile: null,
    selectedIndex: 0,
    total: 0,
  };
}

function searchScreenReducer(state: SearchScreenState, action: SearchScreenAction): SearchScreenState {
  switch (action.type) {
    case "toggle_focus":
      return {
        ...state,
        activePane: toggleTerminalTwoPaneFocus(state.activePane),
      };
    case "toggle_layout":
      return {
        ...state,
        layoutMode: toggleTerminalTwoPaneLayoutMode(state.layoutMode, state.activePane),
      };
    case "leave_detail":
      return {
        ...state,
        activePane: "list",
        layoutMode: "split",
      };
    case "move_selection":
      return {
        ...state,
        detailScroll: 0,
        selectedIndex: moveSelection(state.selectedIndex, action.delta, state.results.length),
      };
    case "selection_boundary":
      return {
        ...state,
        detailScroll: 0,
        selectedIndex: state.results.length <= 0 ? 0 : action.boundary === "start" ? 0 : state.results.length - 1,
      };
    case "move_detail":
      return {
        ...state,
        detailScroll: Math.max(0, Math.min(action.maxDetailScroll, state.detailScroll + action.delta)),
      };
    case "detail_boundary":
      return {
        ...state,
        detailScroll: action.boundary === "start" ? 0 : action.maxDetailScroll,
      };
    case "set_results":
      return {
        ...state,
        activePane: "list",
        detailScroll: 0,
        layoutMode: "split",
        mode: action.mode,
        queryText: action.queryText,
        results: action.results,
        searchMode: action.mode,
        searchProfile: action.searchProfile,
        selectedIndex: 0,
        total: action.total,
      };
    case "clear_results":
      return createInitialSearchScreenState();
    default:
      return state;
  }
}

function buildSearchResultLabel(record: NormalizedRecord): string {
  const scope = record.subcategory ? `${record.category}/${record.subcategory}` : record.category;
  const level = record.level === null ? "-" : String(record.level);
  return `${record.name} | ${scope} | lvl ${level} | ${record.packLabel}`;
}

function buildSearchListLines(
  records: NormalizedRecord[],
  selectedIndex: number,
  bodyHeight: number,
): DerivedTagTerminalLine[] {
  if (records.length === 0) {
    return [
      { text: "No search results yet.", tone: "dim" },
      { text: "" },
      { text: "/ run ranked search  l exact lookup", tone: "accent" },
    ];
  }

  const visibleCount = Math.max(1, bodyHeight);
  const windowStart = clampWindowStart(selectedIndex, records.length, visibleCount);

  return records.slice(windowStart, windowStart + visibleCount).map((record, offset) => ({
    text: buildSearchResultLabel(record),
    tone: windowStart + offset === selectedIndex ? "selected" : "default",
    noWrap: true,
  }));
}

function buildSearchIntroLines(): DerivedTagTerminalLine[] {
  return [
    { text: "Search and Lookup", tone: "section" },
    { text: "This surface uses the same indexed data service as the MCP server." },
    { text: "" },
    { text: "Commands", tone: "section" },
    { text: "/ run ranked or semantic search", indent: 2 },
    { text: "l run exact name lookup", indent: 2 },
    { text: "c clear the current result set", indent: 2 },
  ];
}

function buildSearchDetailLines(record: NormalizedRecord | undefined, state: SearchScreenState): DerivedTagTerminalLine[] {
  if (!record) {
    return [
      ...buildSearchIntroLines(),
      { text: "" },
      { text: state.queryText
        ? `Last query: ${state.queryText}`
        : "Run a search or lookup to populate records.", tone: "dim" },
    ];
  }

  return [
    { text: state.mode === "lookup" ? "Exact Lookup Result" : "Search Result", tone: "section" },
    { text: `Query: ${state.queryText || "(none)"}` },
    { text: `Search profile: ${state.searchProfile ?? "lookup"}` },
    { text: "" },
    ...buildOntologyExplorerEntityDetailLines(mapNormalizedRecordToOntologyExplorerEntityRecord(record)),
  ];
}

function buildSearchSubtitle(state: SearchScreenState): string {
  if (state.searchMode === "idle") {
    return "Run exact lookup or ranked search against the indexed PF2E corpus";
  }

  const profile = state.mode === "lookup" ? "lookup" : (state.searchProfile ?? "structured");
  return `${state.mode} | ${state.total} result${state.total === 1 ? "" : "s"} | profile ${profile} | query "${state.queryText}"`;
}

export function SearchScreen({
  onBack,
}: {
  onBack: () => void;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const { dataService } = usePf2eTerminalAppServices();
  const size = useDerivedTagTerminalSize();
  const [busy, setBusy] = React.useState(false);
  const [state, dispatch] = React.useReducer(searchScreenReducer, undefined, createInitialSearchScreenState);

  const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(size.height, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
  const layoutMode = normalizeTerminalTwoPaneLayoutMode(state.layoutMode, state.activePane);
  const detailLines = buildSearchDetailLines(state.results[state.selectedIndex], state);
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
      defaultValue: mode === state.mode ? state.queryText : "",
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

    setBusy(true);
    try {
      if (mode === "lookup") {
        const lookup = dataService.lookup(trimmed);
        dispatch({
          type: "set_results",
          mode,
          queryText: trimmed,
          results: lookup.match ? [lookup.match, ...lookup.alternatives] : [],
          total: lookup.match ? 1 + lookup.alternatives.length : 0,
          searchProfile: null,
        });
        return;
      }

      const result = await dataService.search({
        query: trimmed,
        limit: 20,
        searchProfile: "balanced",
      });
      dispatch({
        type: "set_results",
        mode,
        queryText: trimmed,
        results: result.records,
        total: result.total,
        searchProfile: result.searchProfile,
      });
    } catch (error) {
      await terminal.pauseForAnyKey(`Search failed: ${(error as Error).message}`);
    } finally {
      setBusy(false);
    }
  }, [dataService, state.mode, state.queryText, terminal]);

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
      if ((normalized === "right" || normalized === "enter") && state.results.length > 0) {
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
            text: "z split-view  Tab/w list focus  Up/Down or j/k scroll  Ctrl+U/D jump  Space/b page  Home/End edge  Esc/backspace list  / search  l lookup  c clear  q back",
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
        lines: buildSearchListLines(state.results, state.selectedIndex, bodyHeight),
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
            ? "Tab/w focus  z detail-only  Up/Down or j/k move  Ctrl+U/D jump  Space/b page  Home/End edge  Enter/right detail  / search  l lookup  c clear  q back"
            : "Tab/w focus  z detail-only  Up/Down or j/k scroll  Ctrl+U/D jump  Space/b page  Home/End edge  Esc/backspace list  / search  l lookup  c clear  q back",
          tone: "dim",
        },
        {
          text: state.queryText
            ? `Query: ${state.queryText} | showing ${state.results.length}/${state.total}`
            : "No active query",
          tone: "accent",
        },
      ]}
      leftWidth={SEARCH_LEFT_WIDTH}
    />
  );
}
