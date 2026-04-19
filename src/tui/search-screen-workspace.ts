import type { SearchCategory, SearchSubcategory } from "../types.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalFacetSelection,
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalSearchMode,
  Pf2eTerminalSearchRequest,
} from "./search-service.js";
import type { DerivedTagTerminalCommandOption, DerivedTagTerminalLine } from "./terminal-ui.js";
import { clampWindowStart } from "./list-utils.js";
import type { OntologyPickerSelectionMap } from "./ontology-explorer/picker-screen.js";
import type { SearchCountState, SearchScreenState } from "./search-screen-state.js";
import { formatCount, formatResultPosition, formatSort, getSessionBufferRange } from "./search-screen-state.js";

export type SearchWorkspaceAction =
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

export type SearchWorkspaceEntry = {
  action: SearchWorkspaceAction;
  label: string;
  value: string;
  description: string;
  disabled?: boolean;
  disabledReason?: string;
};

function humanizeIdentifier(value: string): string {
  return value
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .split(/[_\s-]+/)
    .filter((segment) => segment.length > 0)
    .map((segment) => `${segment[0]!.toUpperCase()}${segment.slice(1)}`)
    .join(" ");
}

export function formatSearchCategory(category: SearchCategory | null): string {
  return category ? humanizeIdentifier(category) : "Any Category";
}

export function formatSearchSubcategory(subcategory: SearchSubcategory | null): string {
  return subcategory ? humanizeIdentifier(subcategory) : "Any Subcategory";
}

export function formatMode(mode: Pf2eTerminalSearchMode): string {
  return humanizeIdentifier(mode);
}

function formatPolicyValue(value: number | string): string {
  return typeof value === "number" ? String(value) : humanizeIdentifier(value);
}

export function formatFilterPolicy<T extends number | string>(policy: Pf2eTerminalFilterValuePolicy<T>): string {
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

export function hasFilterPolicy<T extends number | string>(policy: Pf2eTerminalFilterValuePolicy<T>): boolean {
  return policy.any.length > 0 || policy.all.length > 0 || policy.exclude.length > 0;
}

function formatFacetSelection(facet: Pf2eTerminalFacetSelection): string {
  return `${humanizeIdentifier(facet.field)}: ${formatFilterPolicy(facet.policy)}`;
}

export function formatLevelRange(request: Pf2eTerminalSearchRequest): string {
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

function hasStructuredSignal(request: Pf2eTerminalSearchRequest): boolean {
  return Boolean(
    request.filters.category ||
    request.filters.subcategory ||
    request.filters.levelMin !== null ||
    request.filters.levelMax !== null ||
    hasFilterPolicy(request.filters.rarity) ||
    hasFilterPolicy(request.filters.actionCost) ||
    request.filters.facets.length > 0,
  );
}

export function getExecuteAvailability(request: Pf2eTerminalSearchRequest): {
  disabled: boolean;
  reason: string | null;
} {
  if (request.mode === "lookup" && !request.queryText.trim()) {
    return {
      disabled: true,
      reason: "Unavailable until you enter a lookup name.",
    };
  }

  if (request.mode === "search" && !request.queryText.trim() && !hasStructuredSignal(request)) {
    return {
      disabled: true,
      reason: "Unavailable until search mode has text or at least one structured filter.",
    };
  }

  return {
    disabled: false,
    reason: null,
  };
}

export function formatCountSummary(countState: SearchCountState, request: Pf2eTerminalSearchRequest): string {
  const availability = getExecuteAvailability(request);
  if (availability.disabled) {
    return availability.reason ?? "Unavailable";
  }
  if (countState.status === "loading") {
    return "Counting matches...";
  }
  if (countState.status === "error") {
    return countState.message ?? "Live count unavailable.";
  }
  if (countState.status === "ready" && countState.result) {
    const noun = request.mode === "lookup" ? "candidate" : "match";
    return `${countState.result.total} ${noun}${countState.result.total === 1 ? "" : "es"}`;
  }
  return "Count pending";
}

export function parseLevelRangeInput(value: string): { levelMin: number | null; levelMax: number | null } | string {
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

export function formatDraftStatus(state: SearchScreenState): string {
  if (!state.session) {
    return "No applied query yet";
  }
  return JSON.stringify(state.draft) === JSON.stringify(state.session.request)
    ? "Current setup matches applied query"
    : "Current setup has unapplied changes";
}

export function buildWorkspaceEntries(state: SearchScreenState, countState: SearchCountState): SearchWorkspaceEntry[] {
  const executeAvailability = getExecuteAvailability(state.draft);
  const entries: SearchWorkspaceEntry[] = [
    {
      action: "execute",
      label: "Execute Query",
      value: formatCountSummary(countState, state.draft),
      description: "Apply the current query setup and switch into the results reader.",
      disabled: executeAvailability.disabled,
      disabledReason: executeAvailability.reason ?? undefined,
    },
    {
      action: "mode",
      label: "Mode",
      value: formatMode(state.draft.mode),
      description:
        "Choose whether this query setup should browse deterministically, run ranked search, or perform exact lookup-style matching.",
    },
    {
      action: "query",
      label: "Query",
      value: state.draft.queryText || "(none)",
      description:
        state.draft.mode === "lookup"
          ? "Edit the lookup text used to find near-exact record names."
          : "Edit the free-text portion of the query setup. Browse mode can leave this empty.",
    },
    {
      action: "category",
      label: "Category",
      value: formatSearchCategory(state.draft.filters.category),
      description: "Set the top-level category boundary for this query setup.",
    },
    {
      action: "subcategory",
      label: "Subcategory",
      value: formatSearchSubcategory(state.draft.filters.subcategory),
      description: "Set the within-category boundary for this query setup.",
      disabled: !state.draft.filters.category,
      disabledReason: state.draft.filters.category
        ? undefined
        : "Choose a category first, then refine to a subcategory.",
    },
    {
      action: "levels",
      label: "Levels",
      value: formatLevelRange(state.draft),
      description: "Constrain this query setup to a level band such as `3-8` or `<=5`.",
    },
    {
      action: "rarity",
      label: "Rarity",
      value: formatFilterPolicy(state.draft.filters.rarity),
      description: "Cycle rarity values through include and exclude policies in one view.",
    },
    {
      action: "addFacet",
      label: "Edit Facet Filter",
      value: `${state.draft.filters.facets.length + (hasFilterPolicy(state.draft.filters.actionCost) ? 1 : 0)} active`,
      description: "Choose a discoverable metadata field and cycle each value through any, all, or exclude.",
      disabled: !state.draft.filters.category,
      disabledReason: state.draft.filters.category
        ? undefined
        : "Choose a category before editing discoverable facet filters.",
    },
    {
      action: "removeFacet",
      label: "Clear Facet Filter",
      value: `${state.draft.filters.facets.length + (hasFilterPolicy(state.draft.filters.actionCost) ? 1 : 0)} active`,
      description: "Remove an entire facet policy block from the current query setup.",
      disabled: state.draft.filters.facets.length === 0 && !hasFilterPolicy(state.draft.filters.actionCost),
      disabledReason:
        state.draft.filters.facets.length > 0 || hasFilterPolicy(state.draft.filters.actionCost)
          ? undefined
          : "No facet policies are currently applied.",
    },
    {
      action: "reset",
      label: "Reset Filters",
      value: "Restore defaults",
      description: "Discard the current query setup and return to the default scope and filters.",
    },
    {
      action: "clearResults",
      label: "Discard Applied Results",
      value: state.session ? `${state.session.loadedCount}/${state.session.total} loaded` : "No results",
      description: "Clear the applied result reader while leaving the current query setup untouched.",
      disabled: !state.session,
      disabledReason: state.session ? undefined : "There is no applied result reader to discard.",
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

export function buildWorkspaceLines(
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

export function buildDraftSummaryLines(
  state: SearchScreenState,
  countState: SearchCountState,
): DerivedTagTerminalLine[] {
  const executeAvailability = getExecuteAvailability(state.draft);
  const lines: DerivedTagTerminalLine[] = [
    { text: "Query Summary", tone: "section" },
    { text: `Mode: ${formatMode(state.draft.mode)}` },
    { text: `Query: ${state.draft.queryText || "(none)"}` },
    {
      text: `Scope: ${formatSearchCategory(state.draft.filters.category)} / ${formatSearchSubcategory(state.draft.filters.subcategory)}`,
    },
    { text: `Levels: ${formatLevelRange(state.draft)}` },
    { text: `Rarity: ${formatFilterPolicy(state.draft.filters.rarity)}` },
    {
      text: `Facet filters: ${state.draft.filters.facets.length + (hasFilterPolicy(state.draft.filters.actionCost) ? 1 : 0)}`,
    },
  ];

  if (state.draft.mode === "search") {
    lines.splice(3, 0, { text: `Profile: ${state.draft.searchProfile}` });
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
  lines.push({ text: "Live Count", tone: "section" });
  if (executeAvailability.disabled) {
    lines.push({ text: executeAvailability.reason ?? "Unavailable for the current query setup.", tone: "warning" });
  } else if (countState.status === "loading") {
    lines.push({ text: "Counting lexical matches for the current query setup...", tone: "accent" });
  } else if (countState.status === "error") {
    lines.push({ text: countState.message ?? "Live count unavailable.", tone: "warning" });
  } else if (countState.status === "ready" && countState.result) {
    lines.push({
      text: `${countState.result.total} matching record${countState.result.total === 1 ? "" : "s"} before result ordering.`,
      tone: countState.result.total === 0 ? "warning" : "default",
    });
  } else {
    lines.push({ text: "Count pending.", tone: "dim" });
  }

  lines.push({ text: "Tab executes the current query setup and opens the results reader.", tone: "accent" });
  lines.push({ text: "" });
  lines.push({ text: "Applied Session", tone: "section" });
  lines.push({ text: formatDraftStatus(state) });
  if (!state.session) {
    lines.push({ text: "No applied query yet.", tone: "dim" });
  } else {
    lines.push({ text: `Sort: ${formatSort(state.session.sort)}` });
    lines.push({ text: `Position: ${formatResultPosition(state.resultSelectedIndex, state.session.total)}` });
    lines.push({ text: `Buffered: ${formatCount(state.session.loadedCount)}` });
    lines.push({ text: `Window: ${getSessionBufferRange(state.session)}` });
    lines.push({ text: `Applied mode: ${formatMode(state.session.request.mode)} | ${state.session.resultMode}` });
  }

  return lines;
}

export function buildWorkspaceEntryDetailLines(
  entry: SearchWorkspaceEntry,
  state: SearchScreenState,
  countState: SearchCountState,
): DerivedTagTerminalLine[] {
  const descriptionTone = entry.disabled ? "warning" : "default";
  return [
    { text: entry.label, tone: "section" },
    { text: `Current value: ${entry.value}` },
    {
      text: entry.disabled ? `Unavailable: ${entry.disabledReason ?? entry.description}` : entry.description,
      tone: descriptionTone,
    },
    ...(entry.disabled
      ? []
      : entry.action === "execute"
        ? [
            {
              text: "Press Enter, Right, Space, or Tab to execute the current query setup and switch to results.",
              tone: "accent" as const,
            },
          ]
        : [{ text: "Press Enter, Right, or Space to edit or act on this item.", tone: "accent" as const }]),
    { text: "" },
    ...buildDraftSummaryLines(state, countState),
  ];
}

export function buildFacetRemovalEntries(
  facets: Pf2eTerminalFacetSelection[],
  actionCost: Pf2eTerminalFilterValuePolicy<number>,
): Array<{ value: string; label: string; description: string }> {
  const entries = facets.map((facet) => ({
    value: facet.field,
    label: humanizeIdentifier(facet.field),
    description: `Clear ${formatFilterPolicy(facet.policy)} from the current query setup.`,
  }));

  if (hasFilterPolicy(actionCost)) {
    entries.unshift({
      value: "actionCost",
      label: "Action Cost",
      description: `Clear ${formatFilterPolicy(actionCost)} from the current query setup.`,
    });
  }

  return entries;
}

function createEmptyStringPolicy(): OntologyPickerSelectionMap[string] {
  return {
    any: [],
    all: [],
    exclude: [],
  };
}

export function buildFacetPickerInitialSelections(
  request: Pf2eTerminalSearchRequest,
  scopedFields: string[],
): OntologyPickerSelectionMap {
  const initialSelections = Object.fromEntries(
    scopedFields.map((field) => [field, createEmptyStringPolicy()]),
  ) as OntologyPickerSelectionMap;

  for (const facet of request.filters.facets) {
    if (!scopedFields.includes(facet.field)) {
      continue;
    }
    initialSelections[facet.field] = {
      any: [...facet.policy.any],
      all: [...facet.policy.all],
      exclude: [...facet.policy.exclude],
    };
  }

  if (scopedFields.includes("actionCost")) {
    initialSelections.actionCost = {
      any: request.filters.actionCost.any.map((value) => String(value)),
      all: [],
      exclude: request.filters.actionCost.exclude.map((value) => String(value)),
    };
  }

  return initialSelections;
}

export function applyFacetPickerSelectionsToRequest(
  request: Pf2eTerminalSearchRequest,
  selections: OntologyPickerSelectionMap,
  scopedFields: string[],
): Pf2eTerminalSearchRequest {
  const retainedFacets = request.filters.facets.filter((facet) => !scopedFields.includes(facet.field));
  const nextFacets = [...retainedFacets];

  for (const field of scopedFields) {
    if (field === "actionCost") {
      continue;
    }
    const selection = selections[field] ?? createEmptyStringPolicy();
    if (selection.any.length === 0 && selection.all.length === 0 && selection.exclude.length === 0) {
      continue;
    }
    nextFacets.push({
      field: field as Pf2eTerminalFacetField,
      policy: {
        any: [...selection.any],
        all: [...selection.all],
        exclude: [...selection.exclude],
      },
    });
  }

  const actionCostSelection = selections.actionCost ?? createEmptyStringPolicy();
  return {
    ...request,
    filters: {
      ...request.filters,
      actionCost: {
        any: actionCostSelection.any
          .map((value) => Number.parseInt(value, 10))
          .filter((value) => Number.isFinite(value)),
        all: [],
        exclude: actionCostSelection.exclude
          .map((value) => Number.parseInt(value, 10))
          .filter((value) => Number.isFinite(value)),
      },
      facets: nextFacets,
    },
  };
}

export function buildDraftCommandPaletteEntries(
  workspaceEntries: SearchWorkspaceEntry[],
): DerivedTagTerminalCommandOption<SearchWorkspaceAction>[] {
  return workspaceEntries.map((entry) => ({
    value: entry.action,
    label: entry.label,
    description: entry.description,
    keywords: [entry.value],
    disabled: entry.disabled,
    disabledReason: entry.disabledReason,
  }));
}
