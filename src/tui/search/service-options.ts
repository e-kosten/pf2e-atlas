import { orderFilterValues, type FilterValueOrdering } from "../../domain/filter-value-ordering.js";
import type { LookupSortSpec } from "../../domain/search-request-types.js";
import { formatOntologySearchVocabularyLabel } from "../../domain/presentation-vocabulary.js";
import type { SearchCategory, SearchSubcategory } from "../../domain/search-types.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalFacetValueOption,
  Pf2eTerminalLookupSort,
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
  search: [],
  lookup: [
    {
      value: "alphabeticalTiered",
      label: "Alphabetical (Tiered)",
      description: "Group exact, normalized, and fuzzy matches, then sort each tier alphabetically.",
    },
    {
      value: "alphabeticalGlobal",
      label: "Alphabetical (Global)",
      description: "Keep one flat alphabetical list with per-row match badges.",
    },
    {
      value: "levelAscTiered",
      label: "Level Low-High (Tiered)",
      description: "Group lookup matches by strength, then read each tier from lowest level to highest level.",
    },
    {
      value: "levelAscGlobal",
      label: "Level Low-High (Global)",
      description: "Keep one flat level-sorted list with per-row match badges.",
    },
    {
      value: "levelDescTiered",
      label: "Level High-Low (Tiered)",
      description: "Group lookup matches by strength, then read each tier from highest level to lowest level.",
    },
    {
      value: "levelDescGlobal",
      label: "Level High-Low (Global)",
      description: "Keep one flat reverse-level list with per-row match badges.",
    },
  ],
};

export const FACET_FIELD_EXCLUSIONS = new Set<Pf2eTerminalFacetField>(["rarity"]);

export function formatCategoryLabel(category: SearchCategory): string {
  return formatOntologySearchVocabularyLabel(category);
}

export function formatSubcategoryLabel(subcategory: SearchSubcategory): string {
  return formatOntologySearchVocabularyLabel(subcategory);
}

export function formatFilterValueLabel(value: string): string {
  return formatOntologySearchVocabularyLabel(value);
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
  switch (mode) {
    case "browse":
      return "alphabetical";
    case "lookup":
      return "alphabeticalTiered";
    case "search":
      return "ranked";
  }
}

export function createSortSeed(sort: Pf2eTerminalSearchSort): number | null {
  if (sort !== "random") {
    return null;
  }

  return Math.trunc(Date.now() % 2147483647);
}

export function isLookupSort(sort: Pf2eTerminalSearchSort): sort is Pf2eTerminalLookupSort {
  return sort.endsWith("Tiered") || sort.endsWith("Global");
}

export function buildLookupSortSpec(sort: Pf2eTerminalLookupSort): LookupSortSpec {
  if (sort.startsWith("alphabetical")) {
    return { kind: "alphabetical", policy: sort.endsWith("Global") ? "global" : "tiered" };
  }
  if (sort.startsWith("levelAsc")) {
    return { kind: "levelAsc", policy: sort.endsWith("Global") ? "global" : "tiered" };
  }
  return { kind: "levelDesc", policy: sort.endsWith("Global") ? "global" : "tiered" };
}

export function getLookupSortPolicy(sort: Pf2eTerminalSearchSort): "tiered" | "global" | null {
  return isLookupSort(sort) ? (sort.endsWith("Global") ? "global" : "tiered") : null;
}

export function formatSearchSortLabel(sort: Pf2eTerminalSearchSort): string {
  switch (sort) {
    case "ranked":
    case "alphabetical":
    case "random":
      return formatOntologySearchVocabularyLabel(sort);
    case "levelAsc":
      return "Level Low-High";
    case "levelDesc":
      return "Level High-Low";
    case "alphabeticalTiered":
      return "Alphabetical (tiered)";
    case "alphabeticalGlobal":
      return "Alphabetical (global)";
    case "levelAscTiered":
      return "Level Low-High (tiered)";
    case "levelAscGlobal":
      return "Level Low-High (global)";
    case "levelDescTiered":
      return "Level High-Low (tiered)";
    case "levelDescGlobal":
      return "Level High-Low (global)";
  }
}
