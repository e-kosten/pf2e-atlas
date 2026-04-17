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
  Pf2eTerminalSearchMode,
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
import { buildOntologyExplorerEntityDetailLines } from "./ontology-explorer/entity-page.js";
import { mapNormalizedRecordToOntologyExplorerEntityRecord } from "./ontology-explorer/entity-record.js";
import {
  createDerivedTagTerminalTwoPaneState,
  getDerivedTagTerminalTwoPaneLayoutMode,
  reduceDerivedTagTerminalTwoPaneState,
  type DerivedTagTerminalTwoPaneAction,
  type DerivedTagTerminalTwoPaneState,
} from "./two-pane-state.js";
import { clampWindowStart } from "./list-utils.js";

type SearchScreenState = DerivedTagTerminalTwoPaneState & {
  draft: Pf2eTerminalSearchRequest;
  selectedIndex: number;
  session: Pf2eTerminalSearchSession | null;
};

type SearchScreenAction =
  | DerivedTagTerminalTwoPaneAction
  | { type: "move_selection"; delta: number }
  | { type: "selection_boundary"; boundary: "start" | "end" }
  | { type: "set_draft"; request: Pf2eTerminalSearchRequest }
  | { type: "set_session"; session: Pf2eTerminalSearchSession }
  | { type: "clear_results" };

const SEARCH_LEFT_WIDTH = 52;

function createInitialSearchScreenState(initialRequest: Pf2eTerminalSearchRequest): SearchScreenState {
  return {
    ...createDerivedTagTerminalTwoPaneState(),
    draft: initialRequest,
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
    case "set_draft":
      return {
        ...state,
        draft: action.request,
      };
    case "set_session":
      return {
        ...createDerivedTagTerminalTwoPaneState(),
        draft: action.session.request,
        selectedIndex: 0,
        session: action.session,
      };
    case "clear_results":
      return {
        ...createDerivedTagTerminalTwoPaneState(),
        draft: state.draft,
        selectedIndex: 0,
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

function formatFacetSelection(facet: Pf2eTerminalFacetSelection): string {
  return `${humanizeIdentifier(facet.field)}: ${facet.values.map((value) => humanizeIdentifier(value)).join(", ")}`;
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
    const levelMin = Number.parseInt(betweenMatch[1]!, 10);
    const levelMax = Number.parseInt(betweenMatch[2]!, 10);
    return { levelMin, levelMax };
  }

  const singleMatch = trimmed.match(/^\d+$/);
  if (singleMatch) {
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

function buildSearchListLines(
  session: Pf2eTerminalSearchSession | null,
  selectedIndex: number,
  bodyHeight: number,
): DerivedTagTerminalLine[] {
  if (!session || session.results.length === 0) {
    return [
      { text: "No results in the current workspace session.", tone: "dim" },
      { text: "" },
      { text: "e execute draft  / edit query  m mode  c category  s subcategory", tone: "accent" },
      { text: "p profile  v levels  r rarity  a action cost  f add facet  Shift+C clear results", tone: "accent" },
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

function buildWorkspaceSummaryLines(state: SearchScreenState): DerivedTagTerminalLine[] {
  const lines: DerivedTagTerminalLine[] = [
    { text: "Workspace", tone: "section" },
    { text: `Draft mode: ${formatMode(state.draft.mode)}` },
    { text: `Draft query: ${state.draft.queryText || "(none)"}` },
    { text: `Draft profile: ${state.draft.searchProfile}` },
    { text: `Scope: ${formatSearchCategory(state.draft.filters.category)} / ${formatSearchSubcategory(state.draft.filters.subcategory)}` },
    { text: `Level range: ${formatLevelRange(state.draft)}` },
    { text: `Rarity: ${state.draft.filters.rarity ?? "(any)"}` },
    { text: `Action cost: ${state.draft.filters.actionCost ?? "(any)"}` },
    { text: `Facet filters: ${state.draft.filters.facets.length}` },
  ];

  if (state.draft.filters.facets.length > 0) {
    for (const facet of state.draft.filters.facets) {
      lines.push({ text: formatFacetSelection(facet), indent: 2 });
    }
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

function buildSearchIntroLines(state: SearchScreenState): DerivedTagTerminalLine[] {
  return [
    ...buildWorkspaceSummaryLines(state),
    { text: "" },
    { text: "Commands", tone: "section" },
    { text: "e execute the current draft", indent: 2 },
    { text: "/ edit query text", indent: 2 },
    { text: "m choose browse/search/lookup mode", indent: 2 },
    { text: "p choose the ranked-search profile", indent: 2 },
    { text: "c choose the category scope", indent: 2 },
    { text: "s choose the subcategory scope", indent: 2 },
    { text: "v edit the level range", indent: 2 },
    { text: "r choose rarity", indent: 2 },
    { text: "a choose action cost", indent: 2 },
    { text: "f add a discoverable facet filter", indent: 2 },
    { text: "d remove one applied facet value", indent: 2 },
    { text: "x reset the draft workspace", indent: 2 },
    { text: "Shift+C clear the current results only", indent: 2 },
  ];
}

function buildSearchDetailLines(
  record: Pf2eTerminalSearchSession["results"][number] | undefined,
  state: SearchScreenState,
): DerivedTagTerminalLine[] {
  if (!record || !state.session) {
    return buildSearchIntroLines(state);
  }

  return [
    ...buildWorkspaceSummaryLines(state),
    { text: "" },
    { text: state.session.request.mode === "lookup" ? "Selected Lookup Result" : "Selected Record", tone: "section" },
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

function buildFacetRemovalEntries(facets: Pf2eTerminalFacetSelection[]): Array<{
  value: string;
  label: string;
  description: string;
}> {
  return facets.flatMap((facet) => facet.values.map((value) => ({
    value: `${facet.field}:${value}`,
    label: `${humanizeIdentifier(facet.field)} = ${humanizeIdentifier(value)}`,
    description: "Remove this value from the current draft filter stack.",
  })));
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
    const selected = await terminal.promptSelectOption({
      title: "Rarity Filter",
      prompt: "Choose the draft rarity boundary",
      entries: [
        {
          value: "__all__",
          label: "Any Rarity",
          description: "Do not restrict by rarity.",
        },
        ...options.map((option) => ({
          value: option.value,
          label: option.label,
          description: option.description,
        })),
      ],
      selectedValue: state.draft.filters.rarity ?? "__all__",
    });

    if (selected !== undefined) {
      applyDraftUpdate((request) => ({
        ...request,
        filters: {
          ...request.filters,
          rarity: selected === "__all__" ? null : selected,
        },
      }));
    }
  }, [applyDraftUpdate, state.draft.filters.category, state.draft.filters.rarity, state.draft.filters.subcategory, terminal, user.search]);

  const chooseActionCostFilter = React.useCallback(async () => {
    const options = user.search.getActionCostOptions(state.draft.filters.category, state.draft.filters.subcategory);
    const selected = await terminal.promptSelectOption({
      title: "Action Cost Filter",
      prompt: "Choose the draft action-cost boundary",
      entries: [
        {
          value: "__all__",
          label: "Any Action Cost",
          description: "Do not restrict by action cost.",
        },
        ...options.map((option) => ({
          value: option.value,
          label: option.label,
          description: option.description,
        })),
      ],
      selectedValue: state.draft.filters.actionCost === null ? "__all__" : String(state.draft.filters.actionCost),
    });

    if (selected !== undefined) {
      applyDraftUpdate((request) => ({
        ...request,
        filters: {
          ...request.filters,
          actionCost: selected === "__all__" ? null : Number.parseInt(selected, 10),
        },
      }));
    }
  }, [applyDraftUpdate, state.draft.filters.actionCost, state.draft.filters.category, state.draft.filters.subcategory, terminal, user.search]);

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
      await terminal.pauseForAnyKey("Choose a category before adding a discoverable facet filter.");
      return;
    }

    const fieldOptions = user.search.getFacetFieldOptions(state.draft.filters.category, state.draft.filters.subcategory);
    if (fieldOptions.length === 0) {
      await terminal.pauseForAnyKey("No discoverable facet fields are available for the current browse scope.");
      return;
    }

    const selectedField = await terminal.promptSelectOption({
      title: "Facet Field",
      prompt: "Choose a discoverable field to add to the draft",
      entries: fieldOptions.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
    });

    if (!selectedField) {
      return;
    }

    const valueOptions = user.search.getFacetValueOptions(
      selectedField as Pf2eTerminalFacetField,
      state.draft.filters.category,
      state.draft.filters.subcategory,
    );
    if (valueOptions.length === 0) {
      await terminal.pauseForAnyKey("No live values are available for that field in the current scope.");
      return;
    }

    const selectedValue = await terminal.promptSelectOption({
      title: "Facet Value",
      prompt: `Choose a value for ${humanizeIdentifier(selectedField)}`,
      entries: valueOptions.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
    });

    if (!selectedValue) {
      return;
    }

    applyDraftUpdate((request) => {
      const facets = [...request.filters.facets];
      const currentIndex = facets.findIndex((facet) => facet.field === selectedField);
      if (currentIndex >= 0) {
        const current = facets[currentIndex]!;
        facets[currentIndex] = {
          field: current.field,
          values: [...new Set([...current.values, selectedValue])],
        };
      } else {
        facets.push({
          field: selectedField as Pf2eTerminalFacetField,
          values: [selectedValue],
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
    if (state.draft.filters.facets.length === 0) {
      await terminal.pauseForAnyKey("There are no facet values to remove from the draft.");
      return;
    }

    const selected = await terminal.promptSelectOption({
      title: "Remove Facet Value",
      prompt: "Choose a facet value to remove from the draft",
      entries: buildFacetRemovalEntries(state.draft.filters.facets),
    });

    if (!selected) {
      return;
    }

    const separatorIndex = selected.indexOf(":");
    const field = selected.slice(0, separatorIndex) as Pf2eTerminalFacetField;
    const value = selected.slice(separatorIndex + 1);
    applyDraftUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        facets: request.filters.facets
          .map((facet) => facet.field !== field
            ? facet
            : {
              field: facet.field,
              values: facet.values.filter((candidate) => candidate !== value),
            })
          .filter((facet) => facet.values.length > 0),
      },
    }));
  }, [applyDraftUpdate, state.draft.filters.facets, terminal]);

  const resetDraftWorkspace = React.useCallback(() => {
    dispatch({ type: "set_draft", request: user.search.createDefaultRequest() });
  }, [user.search]);

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
    if (normalized === "e") {
      void executeRequest(state.draft);
      return;
    }
    if (normalized === "slash") {
      void editQueryText();
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
      if (key.shift) {
        dispatch({ type: "clear_results" });
      } else {
        void chooseCategoryFilter();
      }
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
    if (normalized === "a") {
      void chooseActionCostFilter();
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
        title="Browse/Search"
        subtitle={`${buildSearchSubtitle(state)} | focused detail`}
        pane={{
          title: "[FOCUSED DETAIL] Workspace",
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
            text: "z split-view  Tab/w list focus  Up/Down or j/k scroll  Ctrl+U/D jump  Space/b page  Home/End edge  Esc/backspace list  e execute  / query  m mode  c category  s subcategory  v levels  r rarity  a action  f add facet  d drop facet  x reset  Shift+C clear  q back",
            tone: "dim",
          },
          {
            text: `${formatDraftStatus(state)} | detail scroll ${detailScroll}/${maxDetailScroll}`,
            tone: "accent",
          },
        ]}
      />
    );
  }

  return (
    <TerminalTwoPaneScreen
      title="Browse/Search"
      subtitle={buildSearchSubtitle(state)}
      left={{
        title: state.activePane === "list" ? "[RESULTS] Workspace Results" : "Workspace Results",
        lines: buildSearchListLines(state.session, state.selectedIndex, bodyHeight),
        active: state.activePane === "list",
      }}
      right={{
        title: state.activePane === "detail" ? "[DETAIL] Workspace & Record" : "Workspace & Record",
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
            ? "Tab/w focus  z detail-only  Up/Down or j/k move  Ctrl+U/D jump  Space/b page  Home/End edge  Enter/right detail  e execute  / query  m mode  c category  s subcategory  v levels  r rarity  a action  f add facet  d drop facet  x reset  Shift+C clear  q back"
            : "Tab/w focus  z detail-only  Up/Down or j/k scroll  Ctrl+U/D jump  Space/b page  Home/End edge  Esc/backspace list  e execute  / query  m mode  c category  s subcategory  v levels  r rarity  a action  f add facet  d drop facet  x reset  Shift+C clear  q back",
          tone: "dim",
        },
        {
          text: state.session
            ? `${formatDraftStatus(state)} | showing ${state.session.results.length}/${state.session.total} | ${formatMode(state.draft.mode)}`
            : `${formatDraftStatus(state)} | ${formatMode(state.draft.mode)} | ${formatSearchCategory(state.draft.filters.category)}`,
          tone: "accent",
        },
      ]}
      leftWidth={SEARCH_LEFT_WIDTH}
    />
  );
}
