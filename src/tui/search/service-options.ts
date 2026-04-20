import { orderFilterValues, type FilterValueOrdering } from "../../domain/filter-value-ordering.js";
import type { SearchCategory, SearchSubcategory } from "../../domain/search-types.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalFacetValueOption,
  Pf2eTerminalSearchMode,
  Pf2eTerminalSearchModeOption,
  Pf2eTerminalSearchProfileOption,
  Pf2eTerminalSearchSort,
  Pf2eTerminalSearchSortOption,
} from "./service-types.js";

export const SEARCH_PROFILE_OPTIONS: Pf2eTerminalSearchProfileOption[] = [
  {
    value: "balanced",
    label: "Balanced",
    description: "Default hybrid retrieval for concise themed searches.",
  },
  {
    value: "lexical",
    label: "Lexical",
    description: "Exact-wording heavy retrieval for names and precise PF2E terms.",
  },
  {
    value: "concept",
    label: "Concept",
    description: "Semantic-forward retrieval for broader exploratory concept searches.",
  },
];

export const SEARCH_MODE_OPTIONS: Pf2eTerminalSearchModeOption[] = [
  {
    value: "browse",
    label: "Browse",
    description: "Deterministic listing over structured filters with no ranking required.",
  },
  {
    value: "search",
    label: "Search",
    description: "Ranked lexical or semantic retrieval using the current search profile.",
  },
  {
    value: "lookup",
    label: "Lookup",
    description: "Exact or near-exact name lookup within the current category boundaries.",
  },
];

export const SEARCH_SORT_OPTIONS: Record<Pf2eTerminalSearchMode, Pf2eTerminalSearchSortOption[]> = {
  browse: [
    {
      value: "alphabetical",
      label: "Alphabetical",
      description: "Read deterministic browse results in name order.",
    },
    {
      value: "levelAsc",
      label: "Level Low-High",
      description: "Read results from lowest level to highest level.",
    },
    {
      value: "levelDesc",
      label: "Level High-Low",
      description: "Read results from highest level to lowest level.",
    },
    {
      value: "random",
      label: "Random",
      description: "Shuffle browse results into a stable random session order.",
    },
  ],
  search: [
    {
      value: "ranked",
      label: "Ranked",
      description: "Keep the current search profile's relevance order.",
    },
    {
      value: "alphabetical",
      label: "Alphabetical",
      description: "Read matched results in name order.",
    },
    {
      value: "levelAsc",
      label: "Level Low-High",
      description: "Read matched results from lowest level to highest level.",
    },
    {
      value: "levelDesc",
      label: "Level High-Low",
      description: "Read matched results from highest level to lowest level.",
    },
    {
      value: "random",
      label: "Random",
      description: "Shuffle matched results into a stable random session order.",
    },
  ],
  lookup: [
    {
      value: "ranked",
      label: "Closest Match",
      description: "Keep the best name-match ordering for lookup results.",
    },
    {
      value: "alphabetical",
      label: "Alphabetical",
      description: "Read lookup matches in name order.",
    },
    {
      value: "levelAsc",
      label: "Level Low-High",
      description: "Read lookup matches from lowest level to highest level.",
    },
    {
      value: "levelDesc",
      label: "Level High-Low",
      description: "Read lookup matches from highest level to lowest level.",
    },
    {
      value: "random",
      label: "Random",
      description: "Shuffle lookup matches into a stable random session order.",
    },
  ],
};

export const FACET_FIELD_EXCLUSIONS = new Set<Pf2eTerminalFacetField>(["rarity"]);

export function humanizeIdentifier(value: string): string {
  return value
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .split(/[_\s-]+/)
    .filter((segment) => segment.length > 0)
    .map((segment) => `${segment[0]!.toUpperCase()}${segment.slice(1)}`)
    .join(" ");
}

export function formatCategoryLabel(category: SearchCategory): string {
  return humanizeIdentifier(category);
}

export function formatSubcategoryLabel(subcategory: SearchSubcategory): string {
  return humanizeIdentifier(subcategory);
}

export function formatFilterValueLabel(value: string): string {
  if (value === "true") {
    return "True";
  }
  if (value === "false") {
    return "False";
  }
  return humanizeIdentifier(value);
}

export function orderStringValues(values: readonly string[], ordering?: FilterValueOrdering): string[] {
  return orderFilterValues(
    values.map((value) => ({ value, count: 0 })),
    ordering,
  ).map((entry) => entry.value);
}

export function createFacetValueOptions(
  values: ReadonlyArray<{ value: string; count: number }>,
  options: {
    ordering?: FilterValueOrdering;
    labelFormatter?: (value: string) => string;
  } = {},
): Pf2eTerminalFacetValueOption[] {
  const labelFormatter = options.labelFormatter ?? formatFilterValueLabel;
  return orderFilterValues(values, options.ordering).map((entry) => ({
    value: entry.value,
    label: labelFormatter(entry.value),
    description: `${entry.count} live canonical record${entry.count === 1 ? "" : "s"}.`,
    count: entry.count,
  }));
}

export function getDefaultSort(mode: Pf2eTerminalSearchMode): Pf2eTerminalSearchSort {
  return mode === "browse" ? "alphabetical" : "ranked";
}

export function createSortSeed(sort: Pf2eTerminalSearchSort): number | null {
  if (sort !== "random") {
    return null;
  }

  return Math.trunc(Date.now() % 2147483647);
}
