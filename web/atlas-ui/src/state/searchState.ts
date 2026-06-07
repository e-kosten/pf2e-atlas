import type {
  BasicSearchFilter,
  FilterClause,
  FilterClauseOperator,
  OpenResultWindowRequest,
  RecordListSortView,
} from "../generated/atlas";

export type LibraryPrototype = "ant" | "mantine";
export type TraitOperator = "include_all" | "include_any";

export type SearchFormState = {
  query: string;
  mode: "browse" | "text_search";
  kinds: string[];
  rarity: string[];
  traits: string[];
  traitOperator: TraitOperator;
  excludedTraits: string[];
  levelMin: number | null;
  levelMax: number | null;
  sort: SortKey;
  pageSize: number;
  includeDiagnostics: boolean;
};

export type SortKey =
  | "record_key"
  | "alphabetical"
  | "level_asc"
  | "level_desc"
  | "price_asc"
  | "price_desc";

export const DEFAULT_SEARCH_STATE: SearchFormState = {
  query: "",
  mode: "browse",
  kinds: ["spell", "feat", "equipment"],
  rarity: [],
  traits: [],
  traitOperator: "include_all",
  excludedTraits: [],
  levelMin: null,
  levelMax: null,
  sort: "record_key",
  pageSize: 25,
  includeDiagnostics: false,
};

export const KIND_OPTIONS = [
  { value: "creature", label: "Creature" },
  { value: "character", label: "Character" },
  { value: "companion", label: "Companion" },
  { value: "army", label: "Army" },
  { value: "hazard", label: "Hazard" },
  { value: "vehicle", label: "Vehicle" },
  { value: "equipment", label: "Equipment" },
  { value: "feat", label: "Feat" },
  { value: "spell", label: "Spell" },
  { value: "affliction", label: "Affliction" },
  { value: "rule", label: "Rule" },
  { value: "character_option", label: "Character Option" },
  { value: "lore", label: "Lore" },
  { value: "tooling", label: "Tooling" },
  { value: "campaign_feature", label: "Campaign Feature" },
];

export const RARITY_OPTIONS = [
  { value: "common", label: "Common" },
  { value: "uncommon", label: "Uncommon" },
  { value: "rare", label: "Rare" },
  { value: "unique", label: "Unique" },
];

export const TRAIT_OPTIONS = [
  { value: "attack", label: "Attack" },
  { value: "auditory", label: "Auditory" },
  { value: "cantrip", label: "Cantrip" },
  { value: "concentrate", label: "Concentrate" },
  { value: "divine", label: "Divine" },
  { value: "emotion", label: "Emotion" },
  { value: "fire", label: "Fire" },
  { value: "flourish", label: "Flourish" },
  { value: "incapacitation", label: "Incapacitation" },
  { value: "manipulate", label: "Manipulate" },
  { value: "mental", label: "Mental" },
  { value: "move", label: "Move" },
  { value: "occult", label: "Occult" },
  { value: "primal", label: "Primal" },
  { value: "visual", label: "Visual" },
];

export const SORT_OPTIONS: Array<{ value: SortKey; label: string }> = [
  { value: "record_key", label: "Record Key" },
  { value: "alphabetical", label: "Alphabetical" },
  { value: "level_asc", label: "Level Up" },
  { value: "level_desc", label: "Level Down" },
  { value: "price_asc", label: "Price Up" },
  { value: "price_desc", label: "Price Down" },
];

const MODE_VALUES = ["browse", "text_search"] as const;
const TRAIT_OPERATOR_VALUES = ["include_all", "include_any"] as const;
const SORT_VALUES = SORT_OPTIONS.map(({ value }) => value);
const MIN_PAGE_SIZE = 1;
const MAX_PAGE_SIZE = 100;

export function buildOpenRequest(
  state: SearchFormState,
  pageNumber: number,
): OpenResultWindowRequest {
  const filter = buildBasicFilter(state);
  const page = { number: pageNumber, size: state.pageSize };
  const sort = sortView(state.sort);
  const query = state.query.trim();

  if (state.mode === "text_search" && query.length > 0) {
    return {
      mode: {
        kind: "text_search",
        query,
        filter,
      },
      page,
      include_diagnostics: state.includeDiagnostics,
    };
  }

  return {
    mode: {
      kind: "list_records",
      filter,
      sort,
    },
    page,
    include_diagnostics: state.includeDiagnostics,
  };
}

export function encodeSearchState(state: SearchFormState): string {
  return encodeURIComponent(JSON.stringify(state));
}

export function decodeSearchState(value: string | null): SearchFormState {
  if (!value) {
    return DEFAULT_SEARCH_STATE;
  }
  try {
    const decoded = JSON.parse(decodeURIComponent(value));
    if (!isRecord(decoded)) {
      return DEFAULT_SEARCH_STATE;
    }
    return {
      ...DEFAULT_SEARCH_STATE,
      query: stringValue(decoded.query, DEFAULT_SEARCH_STATE.query),
      mode: oneOf(decoded.mode, MODE_VALUES, DEFAULT_SEARCH_STATE.mode),
      kinds: stringArrayValue(decoded.kinds, DEFAULT_SEARCH_STATE.kinds),
      rarity: stringArrayValue(decoded.rarity, DEFAULT_SEARCH_STATE.rarity),
      traits: stringArrayValue(decoded.traits, DEFAULT_SEARCH_STATE.traits),
      traitOperator: oneOf(
        decoded.traitOperator,
        TRAIT_OPERATOR_VALUES,
        DEFAULT_SEARCH_STATE.traitOperator,
      ),
      excludedTraits: stringArrayValue(
        decoded.excludedTraits,
        DEFAULT_SEARCH_STATE.excludedTraits,
      ),
      levelMin: nullableFiniteNumber(decoded.levelMin, DEFAULT_SEARCH_STATE.levelMin),
      levelMax: nullableFiniteNumber(decoded.levelMax, DEFAULT_SEARCH_STATE.levelMax),
      sort: oneOf(decoded.sort, SORT_VALUES, DEFAULT_SEARCH_STATE.sort),
      pageSize: pageSizeValue(decoded.pageSize, DEFAULT_SEARCH_STATE.pageSize),
      includeDiagnostics: booleanValue(
        decoded.includeDiagnostics,
        DEFAULT_SEARCH_STATE.includeDiagnostics,
      ),
    };
  } catch {
    return DEFAULT_SEARCH_STATE;
  }
}

function buildBasicFilter(state: SearchFormState): BasicSearchFilter {
  const clauses: FilterClause[] = [];
  pushValues(clauses, "kind", "include_any", state.kinds);
  pushValues(clauses, "rarity", "include_any", state.rarity);
  pushValues(clauses, "traits", state.traitOperator, state.traits);
  pushValues(clauses, "traits", "exclude_any", state.excludedTraits);

  if (state.levelMin !== null || state.levelMax !== null) {
    clauses.push({
      id: "level-range",
      field: "level",
      operator: "range",
      values: [],
      range: {
        min: state.levelMin ?? undefined,
        max: state.levelMax ?? undefined,
      },
    });
  }

  return { clauses };
}

function pushValues(
  clauses: FilterClause[],
  field: string,
  operator: FilterClauseOperator,
  values: string[],
) {
  if (values.length === 0) {
    return;
  }
  clauses.push({
    id: `${field}-${operator}`,
    field,
    operator,
    values,
  });
}

function sortView(sort: SortKey): RecordListSortView {
  return { kind: sort };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function stringValue(value: unknown, fallback: string): string {
  return typeof value === "string" ? value : fallback;
}

function stringArrayValue(value: unknown, fallback: string[]): string[] {
  if (!Array.isArray(value) || !value.every((item) => typeof item === "string")) {
    return fallback;
  }
  return value;
}

function oneOf<T extends string>(
  value: unknown,
  allowed: readonly T[],
  fallback: T,
): T {
  return typeof value === "string" && allowed.includes(value as T)
    ? (value as T)
    : fallback;
}

function nullableFiniteNumber(
  value: unknown,
  fallback: number | null,
): number | null {
  if (value === null || value === undefined) {
    return value === null ? null : fallback;
  }
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function pageSizeValue(value: unknown, fallback: number): number {
  if (typeof value !== "number" || !Number.isInteger(value)) {
    return fallback;
  }
  return Math.min(MAX_PAGE_SIZE, Math.max(MIN_PAGE_SIZE, value));
}

function booleanValue(value: unknown, fallback: boolean): boolean {
  return typeof value === "boolean" ? value : fallback;
}
