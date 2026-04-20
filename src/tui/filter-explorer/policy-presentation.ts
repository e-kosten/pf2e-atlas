import type { DerivedTagTerminalSegment, DerivedTagTerminalTone } from "../framework/types.js";
import type { FilterExplorerPolicyState } from "./types.js";

export type FilterExplorerPolicyTextFallback = "symbol" | "text";

export type FilterExplorerPolicyPresentation = {
  state?: FilterExplorerPolicyState;
  symbol: string;
  text: string;
  label: string;
  tone: DerivedTagTerminalTone;
};

const FILTER_EXPLORER_POLICY_PRESENTATION: Record<
  FilterExplorerPolicyState | "off",
  Omit<FilterExplorerPolicyPresentation, "state">
> = {
  any: {
    symbol: "∪",
    text: "ANY",
    label: "include any",
    tone: "success",
  },
  all: {
    symbol: "∩",
    text: "ALL",
    label: "require all",
    tone: "success",
  },
  exclude: {
    symbol: "¬",
    text: "NOT",
    label: "exclude",
    tone: "danger",
  },
  off: {
    symbol: "·",
    text: "OFF",
    label: "off",
    tone: "dim",
  },
};

export function getFilterExplorerPolicyPresentation(
  state: FilterExplorerPolicyState | undefined,
): FilterExplorerPolicyPresentation {
  const presentation = FILTER_EXPLORER_POLICY_PRESENTATION[state ?? "off"];
  return {
    state,
    ...presentation,
  };
}

export function formatFilterExplorerPolicyToken(
  state: FilterExplorerPolicyState | undefined,
  fallback: FilterExplorerPolicyTextFallback = "symbol",
): string {
  const presentation = getFilterExplorerPolicyPresentation(state);
  return fallback === "text" ? presentation.text : presentation.symbol;
}

export function buildFilterExplorerPolicyBadgeSegments(
  state: FilterExplorerPolicyState | undefined,
  options: {
    fallback?: FilterExplorerPolicyTextFallback;
    includeBrackets?: boolean;
  } = {},
): DerivedTagTerminalSegment[] {
  const presentation = getFilterExplorerPolicyPresentation(state);
  const token = options.fallback === "text" ? presentation.text : presentation.symbol;
  return [
    ...(options.includeBrackets === false ? [] : [{ text: "[", tone: "dim" as const }]),
    { text: token, tone: presentation.tone },
    ...(options.includeBrackets === false ? [] : [{ text: "]", tone: "dim" as const }]),
  ];
}

export function buildFilterExplorerPolicyLabelSegments(
  state: FilterExplorerPolicyState | undefined,
  options: {
    fallback?: FilterExplorerPolicyTextFallback;
    includeBrackets?: boolean;
  } = {},
): DerivedTagTerminalSegment[] {
  const presentation = getFilterExplorerPolicyPresentation(state);
  return [
    ...buildFilterExplorerPolicyBadgeSegments(state, options),
    { text: ` ${presentation.label}`, tone: presentation.tone },
  ];
}

export function buildFilterExplorerPolicySequenceSegments(
  states: ReadonlyArray<FilterExplorerPolicyState | undefined>,
  options: {
    fallback?: FilterExplorerPolicyTextFallback;
    includeBrackets?: boolean;
    includeLabels?: boolean;
    separator?: string;
  } = {},
): DerivedTagTerminalSegment[] {
  const separator = options.separator ?? " -> ";
  return states.flatMap((state, index) => [
    ...(index > 0 ? [{ text: separator, tone: "dim" as const }] : []),
    ...(options.includeLabels === false
      ? buildFilterExplorerPolicyBadgeSegments(state, options)
      : buildFilterExplorerPolicyLabelSegments(state, options)),
  ]);
}

type FilterExplorerPolicySelectionLike<T> = {
  any: readonly T[];
  all: readonly T[];
  exclude: readonly T[];
};

export function formatFilterExplorerPolicySummary<T>(
  selection: FilterExplorerPolicySelectionLike<T>,
  options: {
    emptyLabel?: string;
    fallback?: FilterExplorerPolicyTextFallback;
    valueFormatter?: (value: T) => string;
  } = {},
): string {
  const fallback = options.fallback ?? "symbol";
  const valueFormatter = options.valueFormatter ?? ((value: T) => String(value));
  const parts: string[] = [];

  for (const state of ["any", "all", "exclude"] as const) {
    if (selection[state].length === 0) {
      continue;
    }

    parts.push(
      `${formatFilterExplorerPolicyToken(state, fallback)} ${selection[state].map((value) => valueFormatter(value)).join(", ")}`,
    );
  }

  return parts.length > 0 ? parts.join(" | ") : (options.emptyLabel ?? "(any)");
}

export function formatFilterExplorerPolicyCycleCopy(
  states: readonly FilterExplorerPolicyState[],
  options: {
    fallback?: FilterExplorerPolicyTextFallback;
  } = {},
): string {
  const fallback = options.fallback ?? "symbol";
  const parts = states.map((state) => {
    const presentation = getFilterExplorerPolicyPresentation(state);
    const token = fallback === "text" ? presentation.text : presentation.symbol;
    return `${token} ${presentation.label}`;
  });

  if (parts.length <= 1) {
    return parts[0] ?? "";
  }
  if (parts.length === 2) {
    return `${parts[0]} or ${parts[1]}`;
  }

  return `${parts.slice(0, -1).join(", ")}, or ${parts.at(-1)}`;
}
