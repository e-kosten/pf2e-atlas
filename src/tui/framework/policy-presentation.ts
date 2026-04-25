import type {
  DerivedTagTerminalPolicyState,
  DerivedTagTerminalSegment,
  DerivedTagTerminalTone,
} from "./types.js";

export type FilterExplorerPolicyTextFallback = "symbol" | "discoverable";

export type FilterExplorerPolicyPresentation = {
  state?: DerivedTagTerminalPolicyState;
  symbol: string;
  discoverable: string;
  label: string;
  tone: DerivedTagTerminalTone;
};

const FILTER_EXPLORER_POLICY_PRESENTATION: Record<
  DerivedTagTerminalPolicyState | "off",
  Omit<FilterExplorerPolicyPresentation, "state">
> = {
  any: {
    symbol: "✓",
    discoverable: "includeAny",
    label: "include any",
    tone: "success",
  },
  all: {
    symbol: "✓",
    discoverable: "includeAll",
    label: "require all",
    tone: "success",
  },
  exclude: {
    symbol: "x",
    discoverable: "exclude",
    label: "exclude",
    tone: "danger",
  },
  off: {
    symbol: "·",
    discoverable: "off",
    label: "off",
    tone: "dim",
  },
};

export function getFilterExplorerPolicyPresentation(
  state: DerivedTagTerminalPolicyState | undefined,
): FilterExplorerPolicyPresentation {
  const presentation = FILTER_EXPLORER_POLICY_PRESENTATION[state ?? "off"];
  return {
    state,
    ...presentation,
  };
}

export function formatFilterExplorerPolicyToken(
  state: DerivedTagTerminalPolicyState | undefined,
  fallback: FilterExplorerPolicyTextFallback = "symbol",
): string {
  const presentation = getFilterExplorerPolicyPresentation(state);
  if (fallback === "discoverable") {
    return presentation.discoverable;
  }
  return presentation.symbol;
}

export function buildFilterExplorerPolicyBadgeSegments(
  state: DerivedTagTerminalPolicyState | undefined,
  options: {
    fallback?: FilterExplorerPolicyTextFallback;
    includeBrackets?: boolean;
  } = {},
): DerivedTagTerminalSegment[] {
  const presentation = getFilterExplorerPolicyPresentation(state);
  const token = formatFilterExplorerPolicyToken(state, options.fallback);
  return [
    ...(options.includeBrackets === false ? [] : [{ text: "[", tone: "dim" as const }]),
    { text: token, tone: presentation.tone },
    ...(options.includeBrackets === false ? [] : [{ text: "]", tone: "dim" as const }]),
  ];
}

export function buildFilterExplorerPolicyLabelSegments(
  state: DerivedTagTerminalPolicyState | undefined,
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
  states: ReadonlyArray<DerivedTagTerminalPolicyState | undefined>,
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
  states: readonly DerivedTagTerminalPolicyState[],
  options: {
    fallback?: FilterExplorerPolicyTextFallback;
  } = {},
): string {
  const fallback = options.fallback ?? "symbol";
  const parts = states.map((state) => {
    const presentation = getFilterExplorerPolicyPresentation(state);
    const token = formatFilterExplorerPolicyToken(state, fallback);
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
