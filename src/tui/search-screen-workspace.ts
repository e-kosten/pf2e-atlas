import type { MetadataFilterNode, MetadataPredicate, SearchCategory, SearchSubcategory } from "../types.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalFacetSelection,
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalSearchMode,
  Pf2eTerminalSearchQuery,
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
  | "addClause"
  | "clearClauses"
  | "reset"
  | "clearResults"
  | `queryNode:${string}`;

export type SearchWorkspaceEntry = {
  action: SearchWorkspaceAction;
  label: string;
  value: string;
  description: string;
  indent?: number;
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

function isMetadataPredicate(node: MetadataFilterNode): node is MetadataPredicate {
  return !("and" in node) && !("or" in node) && !("not" in node);
}

function getMetadataNodeChildren(node: MetadataFilterNode): MetadataFilterNode[] {
  if ("and" in node) {
    return node.and;
  }
  if ("or" in node) {
    return node.or;
  }
  if ("not" in node) {
    return [node.not];
  }
  return [];
}

function countMetadataPredicates(node: MetadataFilterNode): number {
  if (isMetadataPredicate(node)) {
    return 1;
  }
  return getMetadataNodeChildren(node).reduce((total, child) => total + countMetadataPredicates(child), 0);
}

function formatMetadataScalar(value: boolean | number | string): string {
  if (typeof value === "boolean") {
    return value ? "true" : "false";
  }
  return formatPolicyValue(value);
}

function formatMetadataPredicateValue(node: MetadataPredicate): string {
  if ("metric" in node) {
    return `${node.metric} ${node.op} ${formatMetadataScalar(node.value)}`;
  }
  if ("leftMetric" in node) {
    return `${node.leftMetric} ${node.op} ${node.rightMetric}`;
  }
  if ("values" in node) {
    const values = node.values.map((value) => formatMetadataScalar(value)).join(", ");
    switch (node.op) {
      case "includesAny":
        return `includes any ${values}`;
      case "includesAll":
        return `includes all ${values}`;
      case "excludesAny":
        return `excludes ${values}`;
      case "in":
        return `is one of ${values}`;
      case "notIn":
        return `is not ${values}`;
    }
  }
  if ("min" in node && "max" in node) {
    return `between ${node.min} and ${node.max}`;
  }
  if ("value" in node) {
    switch (node.op) {
      case "contains":
        return `contains ${formatMetadataScalar(node.value)}`;
      case "notContains":
        return `does not contain ${formatMetadataScalar(node.value)}`;
      case "eq":
        return `is ${formatMetadataScalar(node.value)}`;
      case "gte":
        return `>= ${node.value}`;
      case "lte":
        return `<= ${node.value}`;
    }
  }
  return JSON.stringify(node);
}

function describeMetadataNode(
  node: MetadataFilterNode,
  isRoot = false,
): {
  label: string;
  value: string;
  description: string;
} {
  if ("and" in node) {
    return {
      label: isRoot ? "Query Logic" : "AND Group",
      value: `${node.and.length} clause${node.and.length === 1 ? "" : "s"}`,
      description: "Every child clause in this group must match.",
    };
  }
  if ("or" in node) {
    return {
      label: isRoot ? "Query Logic" : "OR Group",
      value: `${node.or.length} clause${node.or.length === 1 ? "" : "s"}`,
      description: "Any child clause in this group may match.",
    };
  }
  if ("not" in node) {
    return {
      label: isRoot ? "Query Logic" : "NOT Group",
      value: "1 clause",
      description: "Negate the child clause in this group.",
    };
  }
  const label =
    node.field === "actorMetric"
      ? "Actor Metric"
      : node.field === "actorMetricCompare"
        ? "Actor Metric Compare"
        : node.field === "itemMetric"
          ? "Item Metric"
          : node.field === "itemMetricCompare"
            ? "Item Metric Compare"
            : humanizeIdentifier(node.field);
  return {
    label: isRoot ? "Query Clause" : label,
    value: formatMetadataPredicateValue(node),
    description: `Edit or remove this ${label.toLowerCase()} clause.`,
  };
}

function encodeQueryNodePath(path: number[]): string {
  return path.length > 0 ? path.join(".") : "root";
}

export function isQueryNodeAction(action: SearchWorkspaceAction): action is `queryNode:${string}` {
  return action.startsWith("queryNode:");
}

export function decodeQueryNodeActionPath(action: SearchWorkspaceAction): number[] | null {
  if (!isQueryNodeAction(action)) {
    return null;
  }
  const encodedPath = action.slice("queryNode:".length);
  if (encodedPath === "root") {
    return [];
  }
  const path = encodedPath
    .split(".")
    .map((segment) => Number.parseInt(segment, 10))
    .filter((segment) => Number.isInteger(segment));
  return path.length > 0 ? path : null;
}

function buildMetadataWorkspaceEntries(
  node: MetadataFilterNode,
  path: number[] = [],
  depth = 0,
): SearchWorkspaceEntry[] {
  const summary = describeMetadataNode(node, path.length === 0);
  const entries: SearchWorkspaceEntry[] = [
    {
      action: `queryNode:${encodeQueryNodePath(path)}`,
      label: summary.label,
      value: summary.value,
      description: summary.description,
      indent: depth,
    },
  ];

  const children = getMetadataNodeChildren(node);
  children.forEach((child, childIndex) => {
    entries.push(...buildMetadataWorkspaceEntries(child, [...path, childIndex], depth + 1));
  });

  return entries;
}

export function formatLevelRange(request: Pf2eTerminalSearchQuery): string {
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

function hasStructuredSignal(request: Pf2eTerminalSearchQuery): boolean {
  return Boolean(
    request.filters.category ||
    request.filters.subcategory ||
    request.filters.levelMin !== null ||
    request.filters.levelMax !== null ||
    hasFilterPolicy(request.filters.rarity) ||
    hasFilterPolicy(request.filters.actionCost) ||
    request.filters.facets.length > 0 ||
    request.filters.metadata,
  );
}

export function getExecuteAvailability(request: Pf2eTerminalSearchQuery): {
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

export function formatCountSummary(countState: SearchCountState, request: Pf2eTerminalSearchQuery): string {
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

export function formatQueryStatus(state: SearchScreenState): string {
  if (!state.session) {
    return "No applied query yet";
  }
  return JSON.stringify(state.query) === JSON.stringify(state.session.query)
    ? "Current editor matches applied query"
    : "Current editor has unapplied changes";
}

export function buildWorkspaceEntries(state: SearchScreenState, countState: SearchCountState): SearchWorkspaceEntry[] {
  const executeAvailability = getExecuteAvailability(state.query);
  const entries: SearchWorkspaceEntry[] = [
    {
      action: "execute",
      label: "Execute Query",
      value: formatCountSummary(countState, state.query),
      description: "Apply the current query editor state and switch into the results reader.",
      disabled: executeAvailability.disabled,
      disabledReason: executeAvailability.reason ?? undefined,
    },
    {
      action: "mode",
      label: "Mode",
      value: formatMode(state.query.mode),
      description:
        "Choose whether this query should browse deterministically, run ranked search, or perform exact lookup-style matching.",
    },
    {
      action: "query",
      label: "Query",
      value: state.query.queryText || "(none)",
      description:
        state.query.mode === "lookup"
          ? "Edit the lookup text used to find near-exact record names."
          : "Edit the free-text portion of the query. Browse mode can leave this empty.",
    },
    {
      action: "category",
      label: "Category",
      value: formatSearchCategory(state.query.filters.category),
      description: "Set the top-level category boundary for this query.",
    },
    {
      action: "subcategory",
      label: "Subcategory",
      value: formatSearchSubcategory(state.query.filters.subcategory),
      description: "Set the within-category boundary for this query.",
      disabled: !state.query.filters.category,
      disabledReason: state.query.filters.category
        ? undefined
        : "Choose a category first, then refine to a subcategory.",
    },
    {
      action: "levels",
      label: "Levels",
      value: formatLevelRange(state.query),
      description: "Constrain this query to a level band such as `3-8` or `<=5`.",
    },
    {
      action: "rarity",
      label: "Rarity",
      value: formatFilterPolicy(state.query.filters.rarity),
      description: "Cycle rarity values through include and exclude policies in one view.",
    },
    {
      action: "addFacet",
      label: "Edit Facet Filter",
      value: `${state.query.filters.facets.length + (hasFilterPolicy(state.query.filters.actionCost) ? 1 : 0)} active`,
      description: "Choose a discoverable metadata field and cycle each value through any, all, or exclude.",
      disabled: !state.query.filters.category,
      disabledReason: state.query.filters.category
        ? undefined
        : "Choose a category before editing discoverable facet filters.",
    },
    {
      action: "removeFacet",
      label: "Clear Facet Filter",
      value: `${state.query.filters.facets.length + (hasFilterPolicy(state.query.filters.actionCost) ? 1 : 0)} active`,
      description: "Remove an entire facet policy block from the current query.",
      disabled: state.query.filters.facets.length === 0 && !hasFilterPolicy(state.query.filters.actionCost),
      disabledReason:
        state.query.filters.facets.length > 0 || hasFilterPolicy(state.query.filters.actionCost)
          ? undefined
          : "No facet policies are currently applied.",
    },
    {
      action: "addClause",
      label: "Add Query Clause",
      value: state.query.filters.metadata
        ? `${countMetadataPredicates(state.query.filters.metadata)} active`
        : "None yet",
      description: "Add an explicit metadata clause or logic group to the canonical query model.",
      disabled: !state.query.filters.category,
      disabledReason: state.query.filters.category
        ? undefined
        : "Choose a category before adding scoped metadata clauses.",
    },
    {
      action: "reset",
      label: "Reset Query",
      value: "Restore defaults",
      description: "Discard the current query editor state and return to the default scope and filters.",
    },
    {
      action: "clearResults",
      label: "Discard Applied Results",
      value: state.session ? `${state.session.loadedCount}/${state.session.total} loaded` : "No results",
      description: "Clear the applied result reader while leaving the current query editor state untouched.",
      disabled: !state.session,
      disabledReason: state.session ? undefined : "There is no applied result reader to discard.",
    },
  ];

  if (state.query.mode === "search") {
    entries.splice(3, 0, {
      action: "profile",
      label: "Profile",
      value: state.query.searchProfile,
      description: "Choose the lexical, balanced, or concept retrieval profile used by ranked search.",
    });
  }

  if (state.query.filters.metadata) {
    entries.splice(
      entries.findIndex((entry) => entry.action === "reset"),
      0,
      ...buildMetadataWorkspaceEntries(state.query.filters.metadata),
    );
    entries.splice(
      entries.findIndex((entry) => entry.action === "reset"),
      0,
      {
        action: "clearClauses",
        label: "Clear Query Clauses",
        value: `${countMetadataPredicates(state.query.filters.metadata)} active`,
        description: "Remove every explicit metadata clause while keeping the rest of the query editor state intact.",
      },
    );
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
    text: `${"  ".repeat(entry.indent ?? 0)}${entry.label} | ${entry.value}${entry.disabled ? " | unavailable" : ""}`,
    tone: windowStart + offset === safeIndex ? "selected" : entry.disabled ? "dim" : "default",
    noWrap: true,
  }));
}

export function buildQuerySummaryLines(
  state: SearchScreenState,
  countState: SearchCountState,
): DerivedTagTerminalLine[] {
  const executeAvailability = getExecuteAvailability(state.query);
  const lines: DerivedTagTerminalLine[] = [
    { text: "Query Summary", tone: "section" },
    { text: `Mode: ${formatMode(state.query.mode)}` },
    { text: `Query: ${state.query.queryText || "(none)"}` },
    {
      text: `Scope: ${formatSearchCategory(state.query.filters.category)} / ${formatSearchSubcategory(state.query.filters.subcategory)}`,
    },
    { text: `Levels: ${formatLevelRange(state.query)}` },
    { text: `Rarity: ${formatFilterPolicy(state.query.filters.rarity)}` },
    {
      text: `Facet filters: ${state.query.filters.facets.length + (hasFilterPolicy(state.query.filters.actionCost) ? 1 : 0)}`,
    },
    {
      text: `Query clauses: ${state.query.filters.metadata ? countMetadataPredicates(state.query.filters.metadata) : 0}`,
    },
  ];

  if (state.query.mode === "search") {
    lines.splice(3, 0, { text: `Profile: ${state.query.searchProfile}` });
  }

  if (hasFilterPolicy(state.query.filters.actionCost)) {
    lines.push({ text: `Action Cost: ${formatFilterPolicy(state.query.filters.actionCost)}`, indent: 2 });
  }

  for (const facet of state.query.filters.facets) {
    lines.push({ text: formatFacetSelection(facet), indent: 2 });
  }

  if (state.query.filters.metadata) {
    for (const entry of buildMetadataWorkspaceEntries(state.query.filters.metadata)) {
      lines.push({
        text: `${entry.label}: ${entry.value}`,
        indent: 2 + (entry.indent ?? 0) * 2,
      });
    }
  }

  if (state.query.sourceLabel) {
    lines.push({ text: `Seeded from: ${state.query.sourceLabel}` });
  }

  lines.push({ text: "" });
  lines.push({ text: "Live Count", tone: "section" });
  if (executeAvailability.disabled) {
    lines.push({ text: executeAvailability.reason ?? "Unavailable for the current query.", tone: "warning" });
  } else if (countState.status === "loading") {
    lines.push({ text: "Counting lexical matches for the current query...", tone: "accent" });
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

  lines.push({ text: "Tab executes the current query and opens the results reader.", tone: "accent" });
  lines.push({ text: "" });
  lines.push({ text: "Applied Session", tone: "section" });
  lines.push({ text: formatQueryStatus(state) });
  if (!state.session) {
    lines.push({ text: "No applied query yet.", tone: "dim" });
  } else {
    lines.push({ text: `Sort: ${formatSort(state.session.sort)}` });
    lines.push({ text: `Position: ${formatResultPosition(state.resultSelectedIndex, state.session.total)}` });
    lines.push({ text: `Buffered: ${formatCount(state.session.loadedCount)}` });
    lines.push({ text: `Window: ${getSessionBufferRange(state.session)}` });
    lines.push({ text: `Applied mode: ${formatMode(state.session.query.mode)} | ${state.session.resultMode}` });
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
              text: "Press Enter, Right, Space, or Tab to execute the current query and switch to results.",
              tone: "accent" as const,
            },
          ]
        : [{ text: "Press Enter, Right, or Space to edit or act on this item.", tone: "accent" as const }]),
    { text: "" },
    ...buildQuerySummaryLines(state, countState),
  ];
}

export function buildFacetRemovalEntries(
  facets: Pf2eTerminalFacetSelection[],
  actionCost: Pf2eTerminalFilterValuePolicy<number>,
): Array<{ value: string; label: string; description: string }> {
  const entries = facets.map((facet) => ({
    value: facet.field,
    label: humanizeIdentifier(facet.field),
    description: `Clear ${formatFilterPolicy(facet.policy)} from the current query.`,
  }));

  if (hasFilterPolicy(actionCost)) {
    entries.unshift({
      value: "actionCost",
      label: "Action Cost",
      description: `Clear ${formatFilterPolicy(actionCost)} from the current query.`,
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
  request: Pf2eTerminalSearchQuery,
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
  request: Pf2eTerminalSearchQuery,
  selections: OntologyPickerSelectionMap,
  scopedFields: string[],
): Pf2eTerminalSearchQuery {
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

export function buildEditorCommandPaletteEntries(
  workspaceEntries: SearchWorkspaceEntry[],
): DerivedTagTerminalCommandOption<SearchWorkspaceAction>[] {
  return workspaceEntries
    .filter((entry) => !entry.disabled)
    .map((entry) => ({
      value: entry.action,
      label: entry.label,
      description: entry.description,
      keywords: [entry.value],
      disabled: entry.disabled,
      disabledReason: entry.disabledReason,
    }));
}
