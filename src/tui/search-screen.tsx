import React from "react";

import type {
  OntologyNodeQuery,
  SearchCategory,
  SearchProfile,
  SearchSubcategory,
} from "../types.js";
import { usePf2eTerminalAppServices } from "./app-service-context.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalFacetSelection,
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalSearchMode,
  Pf2eTerminalSearchRequest,
  Pf2eTerminalSearchSession,
} from "./search-service.js";
import {
  TerminalThreePaneScreen,
  getDerivedTagTerminalListNavigationAction,
  getNormalizedKeyName,
  getRenderedTerminalLineCount,
  getTerminalPaneBodyHeight,
  getTerminalThreePaneDimensions,
  moveSelection,
  sliceRenderedTerminalLines,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
  useDerivedTagTerminalSize,
  type DerivedTagTerminalLine,
} from "./terminal-ui.js";
import { buildOntologyExplorerEntityDetailLines } from "./ontology-explorer/entity-page.js";
import { mapNormalizedRecordToOntologyExplorerEntityRecord } from "./ontology-explorer/entity-record.js";
import { clampWindowStart } from "./list-utils.js";

type SearchWorkspaceAction =
  | "execute"
  | "mode"
  | "query"
  | "profile"
  | "category"
  | "subcategory"
  | "levels"
  | "rarity"
  | "addFacet"
  | "removeFacet"
  | "reset"
  | "clearResults";

type SearchWorkspaceEntry = {
  action: SearchWorkspaceAction;
  label: string;
  value: string;
  description: string;
  disabled?: boolean;
};

type SearchScreenPane = "workspace" | "results" | "detail";

type SearchScreenState = {
  activePane: SearchScreenPane;
  detailScroll: number;
  draft: Pf2eTerminalSearchRequest;
  workspaceSelectedIndex: number;
  resultSelectedIndex: number;
  session: Pf2eTerminalSearchSession | null;
};

type SearchScreenAction =
  | { type: "set_active_pane"; pane: SearchScreenPane }
  | { type: "move_workspace_selection"; delta: number; itemCount: number }
  | { type: "workspace_selection_boundary"; boundary: "start" | "end"; itemCount: number }
  | { type: "move_result_selection"; delta: number; itemCount: number }
  | { type: "result_selection_boundary"; boundary: "start" | "end"; itemCount: number }
  | { type: "move_detail"; delta: number; maxDetailScroll: number }
  | { type: "detail_boundary"; boundary: "start" | "end"; maxDetailScroll: number }
  | { type: "set_draft"; request: Pf2eTerminalSearchRequest }
  | { type: "set_session"; session: Pf2eTerminalSearchSession }
  | { type: "clear_results" };

const SEARCH_LEFT_WIDTH = 36;
const SEARCH_CENTER_WIDTH = 42;

function createInitialSearchScreenState(initialRequest: Pf2eTerminalSearchRequest): SearchScreenState {
  return {
    activePane: "workspace",
    detailScroll: 0,
    draft: initialRequest,
    workspaceSelectedIndex: 0,
    resultSelectedIndex: 0,
    session: null,
  };
}

function searchScreenReducer(state: SearchScreenState, action: SearchScreenAction): SearchScreenState {
  switch (action.type) {
    case "set_active_pane":
      return {
        ...state,
        activePane: action.pane,
      };
    case "move_workspace_selection":
      return {
        ...state,
        detailScroll: 0,
        workspaceSelectedIndex: action.itemCount <= 0
          ? 0
          : moveSelection(state.workspaceSelectedIndex, action.delta, action.itemCount),
      };
    case "workspace_selection_boundary":
      return {
        ...state,
        detailScroll: 0,
        workspaceSelectedIndex: action.itemCount <= 0
          ? 0
          : action.boundary === "start"
            ? 0
            : action.itemCount - 1,
      };
    case "move_result_selection":
      return {
        ...state,
        detailScroll: 0,
        resultSelectedIndex: action.itemCount <= 0
          ? 0
          : moveSelection(state.resultSelectedIndex, action.delta, action.itemCount),
      };
    case "result_selection_boundary":
      return {
        ...state,
        detailScroll: 0,
        resultSelectedIndex: action.itemCount <= 0
          ? 0
          : action.boundary === "start"
            ? 0
            : action.itemCount - 1,
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
    case "set_draft":
      return {
        ...state,
        draft: action.request,
      };
    case "set_session":
      return {
        ...state,
        activePane: action.session.results.length > 0 ? "results" : "workspace",
        detailScroll: 0,
        draft: action.session.request,
        resultSelectedIndex: 0,
        session: action.session,
      };
    case "clear_results":
      return {
        ...state,
        activePane: "workspace",
        detailScroll: 0,
        resultSelectedIndex: 0,
        session: null,
      };
    default:
      return state;
  }
}

function humanizeIdentifier(value: string): string {
  return value
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .split(/[_\s-]+/)
    .filter((segment) => segment.length > 0)
    .map((segment) => `${segment[0]!.toUpperCase()}${segment.slice(1)}`)
    .join(" ");
}

function formatSearchCategory(category: SearchCategory | null): string {
  return category ? humanizeIdentifier(category) : "Any Category";
}

function formatSearchSubcategory(subcategory: SearchSubcategory | null): string {
  return subcategory ? humanizeIdentifier(subcategory) : "Any Subcategory";
}

function formatMode(mode: Pf2eTerminalSearchMode): string {
  return humanizeIdentifier(mode);
}

function formatPolicyValue(value: number | string): string {
  return typeof value === "number" ? String(value) : humanizeIdentifier(value);
}

function formatFilterPolicy<T extends number | string>(
  policy: Pf2eTerminalFilterValuePolicy<T>,
): string {
  const parts: string[] = [];
  if (policy.any.length > 0) {
    parts.push(`any: ${policy.any.map((value) => formatPolicyValue(value)).join(", ")}`);
  }
  if (policy.all.length > 0) {
    parts.push(`all: ${policy.all.map((value) => formatPolicyValue(value)).join(", ")}`);
  }
  if (policy.exclude.length > 0) {
    parts.push(`exclude: ${policy.exclude.map((value) => formatPolicyValue(value)).join(", ")}`);
  }
  return parts.length > 0 ? parts.join(" | ") : "(any)";
}

function hasFilterPolicy<T extends number | string>(policy: Pf2eTerminalFilterValuePolicy<T>): boolean {
  return policy.any.length > 0 || policy.all.length > 0 || policy.exclude.length > 0;
}

function formatFacetSelection(facet: Pf2eTerminalFacetSelection): string {
  return `${humanizeIdentifier(facet.field)}: ${formatFilterPolicy(facet.policy)}`;
}

function formatLevelRange(request: Pf2eTerminalSearchRequest): string {
  const { levelMin, levelMax } = request.filters;
  if (levelMin === null && levelMax === null) {
    return "(any)";
  }
  if (levelMin !== null && levelMax !== null) {
    return levelMin === levelMax ? `L${levelMin}` : `L${levelMin}-L${levelMax}`;
  }
  if (levelMin !== null) {
    return `L${levelMin}+`;
  }
  return `<= L${levelMax}`;
}

function formatLevelRangeInputValue(request: Pf2eTerminalSearchRequest): string {
  const { levelMin, levelMax } = request.filters;
  if (levelMin === null && levelMax === null) {
    return "";
  }
  if (levelMin !== null && levelMax !== null) {
    return levelMin === levelMax ? String(levelMin) : `${levelMin}-${levelMax}`;
  }
  if (levelMin !== null) {
    return `${levelMin}+`;
  }
  return `<=${levelMax}`;
}

function formatDraftStatus(state: SearchScreenState): string {
  if (!state.session) {
    return "Draft not executed yet";
  }
  return JSON.stringify(state.draft) === JSON.stringify(state.session.request)
    ? "Draft matches applied query"
    : "Draft has unapplied changes";
}

function parseLevelRangeInput(value: string): { levelMin: number | null; levelMax: number | null } | string {
  const trimmed = value.trim();
  if (!trimmed) {
    return { levelMin: null, levelMax: null };
  }

  const betweenMatch = trimmed.match(/^(\d+)\s*-\s*(\d+)$/);
  if (betweenMatch) {
    return {
      levelMin: Number.parseInt(betweenMatch[1]!, 10),
      levelMax: Number.parseInt(betweenMatch[2]!, 10),
    };
  }

  if (/^\d+$/.test(trimmed)) {
    const level = Number.parseInt(trimmed, 10);
    return { levelMin: level, levelMax: level };
  }

  const minMatch = trimmed.match(/^(\d+)\+$/);
  if (minMatch) {
    return { levelMin: Number.parseInt(minMatch[1]!, 10), levelMax: null };
  }

  const maxMatch = trimmed.match(/^<=?\s*(\d+)$/);
  if (maxMatch) {
    return { levelMin: null, levelMax: Number.parseInt(maxMatch[1]!, 10) };
  }

  return "Use `3-8`, `5`, `5+`, or `<=10`.";
}

function buildSearchResultLabel(record: Pf2eTerminalSearchSession["results"][number]): string {
  const scope = record.subcategory ? `${record.category}/${record.subcategory}` : record.category;
  const level = record.level === null ? "-" : String(record.level);
  return `${record.name} | ${scope} | lvl ${level} | ${record.packLabel}`;
}

function buildWorkspaceEntries(state: SearchScreenState): SearchWorkspaceEntry[] {
  const entries: SearchWorkspaceEntry[] = [
    {
      action: "execute",
      label: "Run Draft Query",
      value: state.session
        ? `${state.session.results.length}/${state.session.total} shown`
        : "No applied session",
      description: "Run the current draft using the selected browse, search, or lookup mode.",
    },
    {
      action: "mode",
      label: "Mode",
      value: formatMode(state.draft.mode),
      description: "Choose whether this draft should browse deterministically, run ranked search, or perform exact lookup.",
    },
    {
      action: "query",
      label: "Query",
      value: state.draft.queryText || "(none)",
      description: "Edit the free-text portion of the draft. Browse mode can leave this empty.",
    },
    {
      action: "category",
      label: "Category",
      value: formatSearchCategory(state.draft.filters.category),
      description: "Set the top-level category boundary for the draft.",
    },
    {
      action: "subcategory",
      label: "Subcategory",
      value: formatSearchSubcategory(state.draft.filters.subcategory),
      description: state.draft.filters.category
        ? "Set the within-category boundary for the draft."
        : "Choose a category first, then refine to a subcategory.",
      disabled: !state.draft.filters.category,
    },
    {
      action: "levels",
      label: "Levels",
      value: formatLevelRange(state.draft),
      description: "Constrain the draft to a level band such as `3-8` or `<=5`.",
    },
    {
      action: "rarity",
      label: "Rarity",
      value: formatFilterPolicy(state.draft.filters.rarity),
      description: "Cycle rarity values through any and exclude policies in one view.",
    },
    {
      action: "addFacet",
      label: "Edit Facet Filter",
      value: `${
        state.draft.filters.facets.length + (hasFilterPolicy(state.draft.filters.actionCost) ? 1 : 0)
      } active`,
      description: state.draft.filters.category
        ? "Choose a discoverable metadata field and cycle each value through any, all, or exclude."
        : "Choose a category before editing discoverable facet filters.",
      disabled: !state.draft.filters.category,
    },
    {
      action: "removeFacet",
      label: "Clear Facet Filter",
      value: `${
        state.draft.filters.facets.length + (hasFilterPolicy(state.draft.filters.actionCost) ? 1 : 0)
      } active`,
      description: state.draft.filters.facets.length > 0 || hasFilterPolicy(state.draft.filters.actionCost)
        ? "Remove an entire facet policy block from the current draft."
        : "No facet policies are currently applied.",
      disabled: state.draft.filters.facets.length === 0 && !hasFilterPolicy(state.draft.filters.actionCost),
    },
    {
      action: "reset",
      label: "Reset Draft",
      value: "Restore defaults",
      description: "Discard the current draft filters and return to the default workspace state.",
    },
    {
      action: "clearResults",
      label: "Clear Results",
      value: state.session ? `${state.session.results.length} shown` : "No results",
      description: state.session
        ? "Clear the applied result set while leaving the draft untouched."
        : "There is no applied result set to clear.",
      disabled: !state.session,
    },
  ];

  if (state.draft.mode === "search") {
    entries.splice(3, 0, {
      action: "profile",
      label: "Profile",
      value: state.draft.searchProfile,
      description: "Choose the lexical, balanced, or concept retrieval profile used by ranked search.",
    });
  }

  return entries;
}

function buildWorkspaceLines(
  entries: SearchWorkspaceEntry[],
  selectedIndex: number,
  bodyHeight: number,
): DerivedTagTerminalLine[] {
  const visibleCount = Math.max(1, bodyHeight);
  const safeIndex = Math.max(0, Math.min(selectedIndex, Math.max(0, entries.length - 1)));
  const windowStart = clampWindowStart(safeIndex, entries.length, visibleCount);

  return entries.slice(windowStart, windowStart + visibleCount).map((entry, offset) => ({
    text: `${entry.label} | ${entry.value}${entry.disabled ? " | unavailable" : ""}`,
    tone: windowStart + offset === safeIndex ? "selected" : entry.disabled ? "dim" : "default",
    noWrap: true,
  }));
}

function buildResultLines(
  session: Pf2eTerminalSearchSession | null,
  selectedIndex: number,
  bodyHeight: number,
): DerivedTagTerminalLine[] {
  if (!session) {
    return [
      { text: "No applied results yet.", tone: "section" },
      { text: "Use the left pane to set scope and filters, then run the draft query.", tone: "dim" },
      { text: "The result list stays here once a session is applied.", tone: "dim" },
    ];
  }

  if (session.results.length === 0) {
    return [
      { text: "No results in the applied session.", tone: "section" },
      { text: `Applied mode: ${formatMode(session.request.mode)} | ${session.resultMode}`, tone: "dim" },
      { text: "Change the draft scope or query, then run again.", tone: "dim" },
    ];
  }

  const visibleCount = Math.max(1, bodyHeight);
  const safeIndex = Math.max(0, Math.min(selectedIndex, session.results.length - 1));
  const windowStart = clampWindowStart(safeIndex, session.results.length, visibleCount);

  return session.results.slice(windowStart, windowStart + visibleCount).map((record, offset) => ({
    text: buildSearchResultLabel(record),
    tone: windowStart + offset === safeIndex ? "selected" : "default",
    noWrap: true,
  }));
}

function buildWorkspaceSummaryLines(state: SearchScreenState): DerivedTagTerminalLine[] {
  const lines: DerivedTagTerminalLine[] = [
    { text: "Workspace", tone: "section" },
    { text: `Draft mode: ${formatMode(state.draft.mode)}` },
    { text: `Draft query: ${state.draft.queryText || "(none)"}` },
    { text: `Scope: ${formatSearchCategory(state.draft.filters.category)} / ${formatSearchSubcategory(state.draft.filters.subcategory)}` },
    { text: `Level range: ${formatLevelRange(state.draft)}` },
    { text: `Rarity: ${formatFilterPolicy(state.draft.filters.rarity)}` },
    {
      text: `Facet filters: ${
        state.draft.filters.facets.length + (hasFilterPolicy(state.draft.filters.actionCost) ? 1 : 0)
      }`,
    },
  ];

  if (state.draft.mode === "search") {
    lines.splice(3, 0, { text: `Draft profile: ${state.draft.searchProfile}` });
  }

  if (hasFilterPolicy(state.draft.filters.actionCost)) {
    lines.push({ text: `Action Cost: ${formatFilterPolicy(state.draft.filters.actionCost)}`, indent: 2 });
  }

  for (const facet of state.draft.filters.facets) {
    lines.push({ text: formatFacetSelection(facet), indent: 2 });
  }

  if (state.draft.sourceLabel) {
    lines.push({ text: `Seeded from: ${state.draft.sourceLabel}` });
  }

  lines.push({ text: "" });
  lines.push({ text: "Applied Session", tone: "section" });
  lines.push({ text: formatDraftStatus(state) });
  if (!state.session) {
    lines.push({ text: "No applied query yet.", tone: "dim" });
  } else {
    lines.push({ text: `Applied mode: ${formatMode(state.session.request.mode)} | ${state.session.resultMode}` });
    lines.push({ text: `Applied query: ${state.session.request.queryText || "(none)"}` });
    lines.push({ text: `Results: ${state.session.results.length}/${state.session.total}` });
  }

  return lines;
}

function buildWorkspaceEntryDetailLines(
  entry: SearchWorkspaceEntry,
  state: SearchScreenState,
): DerivedTagTerminalLine[] {
  return [
    { text: entry.label, tone: "section" },
    { text: `Current value: ${entry.value}` },
    {
      text: entry.disabled
        ? `Unavailable: ${entry.description}`
        : entry.description,
      tone: entry.disabled ? "warning" : "default",
    },
    ...(entry.disabled
      ? []
      : [{ text: "Press Enter to edit or act on this item.", tone: "accent" as const }]),
    { text: "" },
    ...buildWorkspaceSummaryLines(state),
  ];
}

function buildResultDetailLines(
  record: Pf2eTerminalSearchSession["results"][number],
  resultIndex: number,
  resultCount: number,
): DerivedTagTerminalLine[] {
  return [
    { text: "Result Preview", tone: "section" },
    { text: `Showing result ${resultIndex + 1} of ${resultCount}` },
    { text: "" },
    ...buildOntologyExplorerEntityDetailLines(mapNormalizedRecordToOntologyExplorerEntityRecord(record)),
  ];
}

function buildSearchSubtitle(state: SearchScreenState): string {
  const draft = `${formatMode(state.draft.mode)} | ${formatSearchCategory(state.draft.filters.category)} / ${formatSearchSubcategory(state.draft.filters.subcategory)}`;
  if (!state.session) {
    return `${draft} | no applied session`;
  }

  const dirtyLabel = JSON.stringify(state.draft) === JSON.stringify(state.session.request)
    ? "applied"
    : "draft pending";
  return `${draft} | ${state.session.resultMode} | ${state.session.total} result${state.session.total === 1 ? "" : "s"} | ${dirtyLabel}`;
}

function buildFacetRemovalEntries(
  facets: Pf2eTerminalFacetSelection[],
  actionCost: Pf2eTerminalFilterValuePolicy<number>,
): Array<{
  value: string;
  label: string;
  description: string;
}> {
  const entries = facets.map((facet) => ({
    value: facet.field,
    label: humanizeIdentifier(facet.field),
    description: `Clear ${formatFilterPolicy(facet.policy)} from the current draft filter stack.`,
  }));

  if (hasFilterPolicy(actionCost)) {
    entries.unshift({
      value: "actionCost",
      label: "Action Cost",
      description: `Clear ${formatFilterPolicy(actionCost)} from the current draft filter stack.`,
    });
  }

  return entries;
}

function getNextPane(activePane: SearchScreenPane): SearchScreenPane {
  switch (activePane) {
    case "workspace":
      return "results";
    case "results":
      return "detail";
    case "detail":
    default:
      return "workspace";
  }
}

function getPreviousPane(activePane: SearchScreenPane): SearchScreenPane {
  switch (activePane) {
    case "workspace":
      return "detail";
    case "results":
      return "workspace";
    case "detail":
    default:
      return "results";
  }
}

function buildFooterText(activePane: SearchScreenPane): string {
  switch (activePane) {
    case "workspace":
      return "Up/Down select  Ctrl-U/D jump  PgUp/PgDn page  Home/End edge  Enter edit/run  Right results  / query  Tab cycle  Esc/backspace back  q back";
    case "results":
      return "Up/Down select  Ctrl-U/D jump  PgUp/PgDn page  Home/End edge  Left filters  Right detail  Enter detail  / query  Tab cycle  Esc filters  q back";
    case "detail":
    default:
      return "Up/Down scroll  Ctrl-U/D jump  PgUp/PgDn page  Home/End edge  Left results  / query  Tab cycle  Esc results  q back";
  }
}

export function SearchScreen({
  initialQuery,
  onBack,
}: {
  initialQuery?: OntologyNodeQuery;
  onBack: () => void;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const { user } = usePf2eTerminalAppServices();
  const size = useDerivedTagTerminalSize();
  const [busy, setBusy] = React.useState(false);
  const initialRequest = React.useMemo(
    () => initialQuery ? user.search.createRequestFromOntologyQuery(initialQuery) : user.search.createDefaultRequest(),
    [initialQuery, user.search],
  );
  const [state, dispatch] = React.useReducer(searchScreenReducer, initialRequest, createInitialSearchScreenState);
  const autoRanInitialQuery = React.useRef(false);

  const workspaceEntries = buildWorkspaceEntries(state);
  const workspaceSelectedIndex = Math.max(0, Math.min(state.workspaceSelectedIndex, Math.max(0, workspaceEntries.length - 1)));
  const selectedWorkspaceEntry = workspaceEntries[workspaceSelectedIndex] ?? workspaceEntries[0];

  const resultCount = state.session?.results.length ?? 0;
  const resultSelectedIndex = Math.max(0, Math.min(state.resultSelectedIndex, Math.max(0, resultCount - 1)));
  const selectedResult = resultCount > 0 ? state.session?.results[resultSelectedIndex] ?? null : null;

  const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(size.height, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
  const dimensions = getTerminalThreePaneDimensions(size.width, SEARCH_LEFT_WIDTH, SEARCH_CENTER_WIDTH);
  const selectionJumpSize = Math.max(1, Math.floor(bodyHeight / 2));
  const pageSize = Math.max(1, bodyHeight - 1);
  const detailLines = selectedResult
    ? buildResultDetailLines(selectedResult, resultSelectedIndex, resultCount)
    : selectedWorkspaceEntry
      ? buildWorkspaceEntryDetailLines(selectedWorkspaceEntry, state)
      : buildWorkspaceSummaryLines(state);
  const renderedDetailLineCount = getRenderedTerminalLineCount(detailLines, dimensions.rightWidth);
  const maxDetailScroll = Math.max(0, renderedDetailLineCount - bodyHeight);
  const detailScroll = Math.min(state.detailScroll, maxDetailScroll);

  const applyDraftUpdate = React.useCallback((update: (request: Pf2eTerminalSearchRequest) => Pf2eTerminalSearchRequest) => {
    dispatch({ type: "set_draft", request: user.search.normalizeRequest(update(state.draft)) });
  }, [state.draft, user.search]);

  const executeRequest = React.useCallback(async (request: Pf2eTerminalSearchRequest) => {
    setBusy(true);
    try {
      const session = await user.search.runQuery(request);
      dispatch({ type: "set_session", session });
    } catch (error) {
      await terminal.pauseForAnyKey(`Workspace query failed.\n\n${(error as Error).message}`);
    } finally {
      setBusy(false);
    }
  }, [terminal, user.search]);

  React.useEffect(() => {
    if (!initialQuery || autoRanInitialQuery.current) {
      return;
    }
    autoRanInitialQuery.current = true;
    void executeRequest(initialRequest);
  }, [executeRequest, initialQuery, initialRequest]);

  const editQueryText = React.useCallback(async () => {
    const queryText = await terminal.promptTextInput({
      title: "Draft Query",
      prompt: state.draft.mode === "lookup"
        ? "Enter an exact or near-exact record name"
        : "Enter search text for the current draft",
      defaultValue: state.draft.queryText,
      hint: state.draft.mode === "lookup"
        ? "Example: Raise Shield"
        : "Example: ghost ship captain",
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
    const selected = await terminal.promptSelectOption({
      title: "Workspace Mode",
      prompt: "Choose how the current draft should execute",
      entries: user.search.getModeOptions().map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.draft.mode,
    });

    if (!selected) {
      return;
    }

    applyDraftUpdate((request) => ({
      ...request,
      mode: selected,
    }));
  }, [applyDraftUpdate, state.draft.mode, terminal, user.search]);

  const chooseSearchProfile = React.useCallback(async () => {
    const selected = await terminal.promptSelectOption({
      title: "Search Profile",
      prompt: "Choose the draft profile for ranked search mode",
      entries: user.search.getProfileOptions().map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.draft.searchProfile,
    });

    if (selected) {
      applyDraftUpdate((request) => ({
        ...request,
        searchProfile: selected as SearchProfile,
      }));
    }
  }, [applyDraftUpdate, state.draft.searchProfile, terminal, user.search]);

  const chooseCategoryFilter = React.useCallback(async () => {
    const selected = await terminal.promptSelectOption({
      title: "Category Scope",
      prompt: "Choose the draft category boundary",
      entries: user.search.getCategoryOptions().map((option) => ({
        value: option.value ?? "__all__",
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.draft.filters.category ?? "__all__",
    });

    if (selected !== undefined) {
      applyDraftUpdate((request) => ({
        ...request,
        filters: {
          ...request.filters,
          category: selected === "__all__" ? null : selected as SearchCategory,
          subcategory: null,
        },
      }));
    }
  }, [applyDraftUpdate, state.draft.filters.category, terminal, user.search]);

  const chooseSubcategoryFilter = React.useCallback(async () => {
    if (!state.draft.filters.category) {
      await terminal.pauseForAnyKey("Choose a category before selecting a subcategory.");
      return;
    }

    const selected = await terminal.promptSelectOption({
      title: "Subcategory Scope",
      prompt: "Choose the draft subcategory boundary",
      entries: user.search.getSubcategoryOptions(state.draft.filters.category).map((option) => ({
        value: option.value ?? "__all__",
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.draft.filters.subcategory ?? "__all__",
    });

    if (selected !== undefined) {
      applyDraftUpdate((request) => ({
        ...request,
        filters: {
          ...request.filters,
          subcategory: selected === "__all__" ? null : selected as SearchSubcategory,
        },
      }));
    }
  }, [applyDraftUpdate, state.draft.filters.category, state.draft.filters.subcategory, terminal, user.search]);

  const chooseRarityFilter = React.useCallback(async () => {
    const options = user.search.getRarityOptions(state.draft.filters.category, state.draft.filters.subcategory);
    const selected = await terminal.promptPolicySelectOption({
      title: "Rarity Filter",
      prompt: "Cycle draft rarities through any and exclude. Press Esc or Left when finished.",
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
  }, [applyDraftUpdate, state.draft.filters.category, state.draft.filters.rarity, state.draft.filters.subcategory, terminal, user.search]);

  const editLevelRange = React.useCallback(async () => {
    const input = await terminal.promptTextInput({
      title: "Level Range",
      prompt: "Enter `3-8`, `5`, `5+`, or `<=10`. Leave blank to clear.",
      defaultValue: formatLevelRangeInputValue(state.draft),
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

  const addFacetFilter = React.useCallback(async () => {
    if (!state.draft.filters.category) {
      await terminal.pauseForAnyKey("Choose a category before editing a discoverable facet filter.");
      return;
    }

    const fieldOptions = user.search.getFacetFieldOptions(state.draft.filters.category, state.draft.filters.subcategory);
    if (fieldOptions.length === 0) {
      await terminal.pauseForAnyKey("No discoverable facet fields are available for the current browse scope.");
      return;
    }

    const selectedField = await terminal.promptSelectOption({
      title: "Facet Field",
      prompt: "Choose a discoverable field to edit in the draft",
      entries: fieldOptions.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
    });

    if (!selectedField) {
      return;
    }

    const selectedFieldOption = fieldOptions.find((option) => option.value === selectedField);
    if (!selectedFieldOption) {
      return;
    }

    const valueOptions = selectedField === "actionCost"
      ? user.search.getActionCostOptions(state.draft.filters.category, state.draft.filters.subcategory)
      : user.search.getFacetValueOptions(
        selectedField as Pf2eTerminalFacetField,
        state.draft.filters.category,
        state.draft.filters.subcategory,
      );
    if (valueOptions.length === 0) {
      await terminal.pauseForAnyKey("No live values are available for that field in the current scope.");
      return;
    }

    const selectedPolicy = await terminal.promptPolicySelectOption({
      title: "Facet Policy",
      prompt: `Cycle values for ${humanizeIdentifier(selectedField)}. Press Esc or Left when finished.`,
      allowedStates: selectedFieldOption.fieldType === "set"
        ? ["any", "all", "exclude"]
        : ["any", "exclude"],
      entries: valueOptions.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValues: selectedField === "actionCost"
        ? {
          any: state.draft.filters.actionCost.any.map((value) => String(value)),
          exclude: state.draft.filters.actionCost.exclude.map((value) => String(value)),
        }
        : state.draft.filters.facets.find((facet) => facet.field === selectedField)?.policy,
    });

    applyDraftUpdate((request) => {
      if (selectedField === "actionCost") {
        return {
          ...request,
          filters: {
            ...request.filters,
            actionCost: {
              any: selectedPolicy.any.map((value) => Number.parseInt(value, 10)).filter((value) => Number.isFinite(value)),
              all: [],
              exclude: selectedPolicy.exclude.map((value) => Number.parseInt(value, 10)).filter((value) => Number.isFinite(value)),
            },
          },
        };
      }

      const facets = [...request.filters.facets];
      const currentIndex = facets.findIndex((facet) => facet.field === selectedField);
      if (selectedPolicy.any.length === 0 && selectedPolicy.all.length === 0 && selectedPolicy.exclude.length === 0) {
        return {
          ...request,
          filters: {
            ...request.filters,
            facets: facets.filter((facet) => facet.field !== selectedField),
          },
        };
      }

      if (currentIndex >= 0) {
        facets[currentIndex] = {
          field: facets[currentIndex]!.field,
          policy: selectedPolicy,
        };
      } else {
        facets.push({
          field: selectedField as Pf2eTerminalFacetField,
          policy: selectedPolicy,
        });
      }

      return {
        ...request,
        filters: {
          ...request.filters,
          facets,
        },
      };
    });
  }, [applyDraftUpdate, state.draft.filters.category, state.draft.filters.subcategory, terminal, user.search]);

  const removeFacetFilter = React.useCallback(async () => {
    if (state.draft.filters.facets.length === 0 && !hasFilterPolicy(state.draft.filters.actionCost)) {
      await terminal.pauseForAnyKey("There are no facet policies to clear from the draft.");
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
        actionCost: selected.includes("actionCost")
          ? { any: [], all: [], exclude: [] }
          : request.filters.actionCost,
        facets: request.filters.facets.filter((facet) => !selected.includes(facet.field)),
      },
    }));
  }, [applyDraftUpdate, state.draft.filters.actionCost, state.draft.filters.facets, terminal]);

  const resetDraftWorkspace = React.useCallback(() => {
    dispatch({ type: "set_draft", request: user.search.createDefaultRequest() });
    dispatch({ type: "set_active_pane", pane: "workspace" });
  }, [user.search]);

  const openSelectedWorkspaceEntry = React.useCallback(() => {
    if (!selectedWorkspaceEntry || selectedWorkspaceEntry.disabled) {
      return;
    }

    switch (selectedWorkspaceEntry.action) {
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
        void addFacetFilter();
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
      default:
        return;
    }
  }, [
    addFacetFilter,
    chooseCategoryFilter,
    chooseMode,
    chooseRarityFilter,
    chooseSearchProfile,
    chooseSubcategoryFilter,
    editLevelRange,
    editQueryText,
    executeRequest,
    removeFacetFilter,
    resetDraftWorkspace,
    selectedWorkspaceEntry,
    state.draft,
  ]);

  useDerivedTagTerminalInput((input, key) => {
    if (busy) {
      return;
    }

    const normalized = getNormalizedKeyName(input, key);
    const workspaceNavigation = getDerivedTagTerminalListNavigationAction(normalized, {
      pageSize,
      jumpSize: selectionJumpSize,
      includeConfirmKeys: true,
    });
    const resultsNavigation = getDerivedTagTerminalListNavigationAction(normalized, {
      pageSize,
      jumpSize: selectionJumpSize,
      includeConfirmKeys: true,
    });
    const detailNavigation = getDerivedTagTerminalListNavigationAction(normalized, {
      pageSize,
      jumpSize: selectionJumpSize,
    });

    if (normalized === "ctrl_c" || normalized === "q") {
      onBack();
      return;
    }
    if (normalized === "slash") {
      void editQueryText();
      return;
    }
    if (normalized === "tab" || normalized === "w") {
      dispatch({ type: "set_active_pane", pane: getNextPane(state.activePane) });
      return;
    }
    if (normalized === "shift_tab") {
      dispatch({ type: "set_active_pane", pane: getPreviousPane(state.activePane) });
      return;
    }

    if (state.activePane === "workspace") {
      if (normalized === "escape" || normalized === "backspace") {
        onBack();
        return;
      }
      if (normalized === "right") {
        dispatch({ type: "set_active_pane", pane: "results" });
        return;
      }
      if (workspaceNavigation?.kind === "move") {
        dispatch({ type: "move_workspace_selection", delta: workspaceNavigation.delta, itemCount: workspaceEntries.length });
        return;
      }
      if (workspaceNavigation?.kind === "boundary") {
        dispatch({ type: "workspace_selection_boundary", boundary: workspaceNavigation.boundary, itemCount: workspaceEntries.length });
        return;
      }
      if (workspaceNavigation?.kind === "confirm") {
        openSelectedWorkspaceEntry();
        return;
      }
      if (normalized === "m") {
        void chooseMode();
        return;
      }
      if (normalized === "p") {
        void chooseSearchProfile();
        return;
      }
      if (normalized === "c") {
        void chooseCategoryFilter();
        return;
      }
      if (normalized === "s") {
        void chooseSubcategoryFilter();
        return;
      }
      if (normalized === "v") {
        void editLevelRange();
        return;
      }
      if (normalized === "r") {
        void chooseRarityFilter();
        return;
      }
      if (normalized === "f") {
        void addFacetFilter();
        return;
      }
      if (normalized === "d") {
        void removeFacetFilter();
        return;
      }
      if (normalized === "x") {
        resetDraftWorkspace();
        return;
      }
      if (normalized === "e") {
        void executeRequest(state.draft);
      }
      return;
    }

    if (state.activePane === "results") {
      if (normalized === "escape" || normalized === "backspace" || normalized === "left") {
        dispatch({ type: "set_active_pane", pane: "workspace" });
        return;
      }
      if ((normalized === "right" || resultsNavigation?.kind === "confirm") && selectedResult) {
        dispatch({ type: "set_active_pane", pane: "detail" });
        return;
      }
      if (resultsNavigation?.kind === "move") {
        dispatch({ type: "move_result_selection", delta: resultsNavigation.delta, itemCount: resultCount });
        return;
      }
      if (resultsNavigation?.kind === "boundary") {
        dispatch({ type: "result_selection_boundary", boundary: resultsNavigation.boundary, itemCount: resultCount });
      }
      return;
    }

    if (normalized === "escape" || normalized === "backspace" || normalized === "left") {
      dispatch({ type: "set_active_pane", pane: selectedResult ? "results" : "workspace" });
      return;
    }
    if (detailNavigation?.kind === "move") {
      dispatch({ type: "move_detail", delta: detailNavigation.delta, maxDetailScroll });
      return;
    }
    if (detailNavigation?.kind === "boundary") {
      dispatch({ type: "detail_boundary", boundary: detailNavigation.boundary, maxDetailScroll });
    }
  }, !busy);

  return (
    <TerminalThreePaneScreen
      title="Browse/Search"
      subtitle={buildSearchSubtitle(state)}
      left={{
        title: state.activePane === "workspace" ? "[WORKSPACE] Scope & Filters" : "Scope & Filters",
        lines: buildWorkspaceLines(workspaceEntries, workspaceSelectedIndex, bodyHeight),
        active: state.activePane === "workspace",
      }}
      center={{
        title: state.activePane === "results"
          ? `[RESULTS] ${state.session ? `${state.session.results.length}/${state.session.total} shown` : "No applied session"}`
          : `Results${state.session ? ` | ${state.session.results.length}/${state.session.total} shown` : " | No applied session"}`,
        lines: buildResultLines(state.session, resultSelectedIndex, bodyHeight),
        active: state.activePane === "results",
      }}
      right={{
        title: state.activePane === "detail"
          ? `[PREVIEW] ${selectedResult ? selectedResult.name : selectedWorkspaceEntry?.label ?? "Workspace"}`
          : `Preview${selectedResult ? ` | ${selectedResult.name}` : selectedWorkspaceEntry ? ` | ${selectedWorkspaceEntry.label}` : ""}`,
        lines: sliceRenderedTerminalLines(
          detailLines,
          dimensions.rightWidth,
          detailScroll,
          bodyHeight,
        ),
        active: state.activePane === "detail",
      }}
      footer={[
        {
          text: buildFooterText(state.activePane),
          tone: "dim",
        },
        {
          text: state.session
            ? `${formatDraftStatus(state)} | ${state.session.results.length}/${state.session.total} shown | focus ${humanizeIdentifier(state.activePane)}`
            : `${formatDraftStatus(state)} | ${formatMode(state.draft.mode)} | focus ${humanizeIdentifier(state.activePane)}`,
          tone: "accent",
        },
      ]}
      leftWidth={SEARCH_LEFT_WIDTH}
      centerWidth={SEARCH_CENTER_WIDTH}
    />
  );
}
