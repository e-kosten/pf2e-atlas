import { orderFilterValues, type FilterValueOrdering } from "../../domain/filter-value-ordering.js";
import {
  SEARCH_REQUEST_VOCABULARY,
  type LookupSortSpec,
} from "../../domain/search-request-types.js";
import { formatOntologySearchVocabularyLabel } from "../../domain/presentation-vocabulary.js";
import { SEARCH_VOCABULARY } from "../../domain/search-types.js";
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

export const SEARCH_LOOKUP_SORT_SUFFIX = {
  TIERED: "Tiered" as const,
  GLOBAL: "Global" as const,
} as const;

export const SEARCH_LOOKUP_SORT_VALUES = {
  ALPHABETICAL_TIERED: `${SEARCH_REQUEST_VOCABULARY.SORT_KIND.ALPHABETICAL}${SEARCH_LOOKUP_SORT_SUFFIX.TIERED}` as const,
  ALPHABETICAL_GLOBAL: `${SEARCH_REQUEST_VOCABULARY.SORT_KIND.ALPHABETICAL}${SEARCH_LOOKUP_SORT_SUFFIX.GLOBAL}` as const,
  LEVEL_ASC_TIERED: `${SEARCH_REQUEST_VOCABULARY.SORT_KIND.LEVEL_ASC}${SEARCH_LOOKUP_SORT_SUFFIX.TIERED}` as const,
  LEVEL_ASC_GLOBAL: `${SEARCH_REQUEST_VOCABULARY.SORT_KIND.LEVEL_ASC}${SEARCH_LOOKUP_SORT_SUFFIX.GLOBAL}` as const,
  LEVEL_DESC_TIERED: `${SEARCH_REQUEST_VOCABULARY.SORT_KIND.LEVEL_DESC}${SEARCH_LOOKUP_SORT_SUFFIX.TIERED}` as const,
  LEVEL_DESC_GLOBAL: `${SEARCH_REQUEST_VOCABULARY.SORT_KIND.LEVEL_DESC}${SEARCH_LOOKUP_SORT_SUFFIX.GLOBAL}` as const,
} as const;

export const SEARCH_PROFILE_OPTIONS: Pf2eTerminalSearchProfileOption[] = [
  {
    value: SEARCH_VOCABULARY.PROFILE.BALANCED,
    label: "Balanced",
    description: "Default hybrid retrieval for concise themed searches.",
  },
  {
    value: SEARCH_VOCABULARY.PROFILE.LEXICAL,
    label: "Lexical",
    description: "Exact-wording heavy retrieval for names and precise PF2E terms.",
  },
  {
    value: SEARCH_VOCABULARY.PROFILE.CONCEPT,
    label: "Concept",
    description: "Semantic-forward retrieval for broader exploratory concept searches.",
  },
];

export const SEARCH_MODE_OPTIONS: Pf2eTerminalSearchModeOption[] = [
  {
    value: SEARCH_REQUEST_VOCABULARY.MODE.BROWSE,
    label: "Browse",
    description: "Deterministic listing over structured filters with no ranking required.",
  },
  {
    value: SEARCH_REQUEST_VOCABULARY.MODE.SEARCH,
    label: "Search",
    description: "Ranked lexical or semantic retrieval using the current search profile.",
  },
  {
    value: SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP,
    label: "Lookup",
    description: "Exact or near-exact name lookup within the current category boundaries.",
  },
];

export const SEARCH_SORT_OPTIONS: Record<Pf2eTerminalSearchMode, Pf2eTerminalSearchSortOption[]> = {
  [SEARCH_REQUEST_VOCABULARY.MODE.BROWSE]: [
    {
      value: SEARCH_REQUEST_VOCABULARY.SORT_KIND.ALPHABETICAL,
      label: "Alphabetical",
      description: "Read deterministic browse results in name order.",
    },
    {
      value: SEARCH_REQUEST_VOCABULARY.SORT_KIND.LEVEL_ASC,
      label: "Level Low-High",
      description: "Read results from lowest level to highest level.",
    },
    {
      value: SEARCH_REQUEST_VOCABULARY.SORT_KIND.LEVEL_DESC,
      label: "Level High-Low",
      description: "Read results from highest level to lowest level.",
    },
    {
      value: SEARCH_REQUEST_VOCABULARY.SORT_KIND.RANDOM,
      label: "Random",
      description: "Shuffle browse results into a stable random session order.",
    },
  ],
  [SEARCH_REQUEST_VOCABULARY.MODE.SEARCH]: [],
  [SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP]: [
    {
      value: SEARCH_LOOKUP_SORT_VALUES.ALPHABETICAL_TIERED,
      label: "Alphabetical (Tiered)",
      description: "Group exact, normalized, and fuzzy matches, then sort each tier alphabetically.",
    },
    {
      value: SEARCH_LOOKUP_SORT_VALUES.ALPHABETICAL_GLOBAL,
      label: "Alphabetical (Global)",
      description: "Keep one flat alphabetical list with per-row match badges.",
    },
    {
      value: SEARCH_LOOKUP_SORT_VALUES.LEVEL_ASC_TIERED,
      label: "Level Low-High (Tiered)",
      description: "Group lookup matches by strength, then read each tier from lowest level to highest level.",
    },
    {
      value: SEARCH_LOOKUP_SORT_VALUES.LEVEL_ASC_GLOBAL,
      label: "Level Low-High (Global)",
      description: "Keep one flat level-sorted list with per-row match badges.",
    },
    {
      value: SEARCH_LOOKUP_SORT_VALUES.LEVEL_DESC_TIERED,
      label: "Level High-Low (Tiered)",
      description: "Group lookup matches by strength, then read each tier from highest level to lowest level.",
    },
    {
      value: SEARCH_LOOKUP_SORT_VALUES.LEVEL_DESC_GLOBAL,
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
    case SEARCH_REQUEST_VOCABULARY.MODE.BROWSE:
      return SEARCH_REQUEST_VOCABULARY.SORT_KIND.ALPHABETICAL;
    case SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP:
      return SEARCH_LOOKUP_SORT_VALUES.ALPHABETICAL_TIERED;
    case SEARCH_REQUEST_VOCABULARY.MODE.SEARCH:
      return SEARCH_VOCABULARY.SORT_KIND.RANKED;
  }
}

export function createSortSeed(sort: Pf2eTerminalSearchSort): number | null {
  if (sort !== SEARCH_REQUEST_VOCABULARY.SORT_KIND.RANDOM) {
    return null;
  }

  return Math.trunc(Date.now() % 2147483647);
}

export function isLookupSort(sort: Pf2eTerminalSearchSort): sort is Pf2eTerminalLookupSort {
  return sort.endsWith(SEARCH_LOOKUP_SORT_SUFFIX.TIERED) || sort.endsWith(SEARCH_LOOKUP_SORT_SUFFIX.GLOBAL);
}

export function buildLookupSortSpec(sort: Pf2eTerminalLookupSort): LookupSortSpec {
  if (sort.startsWith(SEARCH_REQUEST_VOCABULARY.SORT_KIND.ALPHABETICAL)) {
    return {
      kind: SEARCH_REQUEST_VOCABULARY.SORT_KIND.ALPHABETICAL,
      policy: sort.endsWith(SEARCH_LOOKUP_SORT_SUFFIX.GLOBAL)
        ? SEARCH_REQUEST_VOCABULARY.LOOKUP_SORT_POLICY.GLOBAL
        : SEARCH_REQUEST_VOCABULARY.LOOKUP_SORT_POLICY.TIERED,
    };
  }
  if (sort.startsWith(SEARCH_REQUEST_VOCABULARY.SORT_KIND.LEVEL_ASC)) {
    return {
      kind: SEARCH_REQUEST_VOCABULARY.SORT_KIND.LEVEL_ASC,
      policy: sort.endsWith(SEARCH_LOOKUP_SORT_SUFFIX.GLOBAL)
        ? SEARCH_REQUEST_VOCABULARY.LOOKUP_SORT_POLICY.GLOBAL
        : SEARCH_REQUEST_VOCABULARY.LOOKUP_SORT_POLICY.TIERED,
    };
  }
  return {
    kind: SEARCH_REQUEST_VOCABULARY.SORT_KIND.LEVEL_DESC,
    policy: sort.endsWith(SEARCH_LOOKUP_SORT_SUFFIX.GLOBAL)
      ? SEARCH_REQUEST_VOCABULARY.LOOKUP_SORT_POLICY.GLOBAL
      : SEARCH_REQUEST_VOCABULARY.LOOKUP_SORT_POLICY.TIERED,
  };
}

export function getLookupSortPolicy(sort: Pf2eTerminalSearchSort): "tiered" | "global" | null {
  return isLookupSort(sort)
    ? (sort.endsWith(SEARCH_LOOKUP_SORT_SUFFIX.GLOBAL)
      ? SEARCH_REQUEST_VOCABULARY.LOOKUP_SORT_POLICY.GLOBAL
      : SEARCH_REQUEST_VOCABULARY.LOOKUP_SORT_POLICY.TIERED)
    : null;
}

export function formatSearchSortLabel(sort: Pf2eTerminalSearchSort): string {
  switch (sort) {
    case SEARCH_VOCABULARY.SORT_KIND.RANKED:
    case SEARCH_VOCABULARY.SORT_KIND.ALPHABETICAL:
    case SEARCH_VOCABULARY.SORT_KIND.RANDOM:
      return formatOntologySearchVocabularyLabel(sort);
    case SEARCH_VOCABULARY.SORT_KIND.LEVEL_ASC:
      return "Level Low-High";
    case SEARCH_VOCABULARY.SORT_KIND.LEVEL_DESC:
      return "Level High-Low";
    case SEARCH_LOOKUP_SORT_VALUES.ALPHABETICAL_TIERED:
      return "Alphabetical (tiered)";
    case SEARCH_LOOKUP_SORT_VALUES.ALPHABETICAL_GLOBAL:
      return "Alphabetical (global)";
    case SEARCH_LOOKUP_SORT_VALUES.LEVEL_ASC_TIERED:
      return "Level Low-High (tiered)";
    case SEARCH_LOOKUP_SORT_VALUES.LEVEL_ASC_GLOBAL:
      return "Level Low-High (global)";
    case SEARCH_LOOKUP_SORT_VALUES.LEVEL_DESC_TIERED:
      return "Level High-Low (tiered)";
    case SEARCH_LOOKUP_SORT_VALUES.LEVEL_DESC_GLOBAL:
      return "Level High-Low (global)";
  }
}
