import type {
  BasicSearchFilter,
  FilterClause,
  FilterClauseOperator,
  FilterDiscoveryContext,
  MetricComparison,
  OpenResultWindowRequest,
  RecordListSortView,
} from "../generated/atlas";

export type NumericRangeState = {
  min: number | null;
  max: number | null;
};

export type MetricComparisonState = {
  key: string | null;
  op: MetricComparison["op"];
  value: number | null;
};

export type SearchFormState = {
  query: string;
  mode: "browse" | "text_search";
  visibleFilterIds: string[];
  hiddenFilterIds: string[];
  filterClauses: FilterClause[];
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

export const DEFAULT_FILTER_CLAUSES: FilterClause[] = [
  {
    id: "kind-include_any",
    field: "kind",
    operator: "include_any",
    values: ["spell", "feat", "equipment"],
  },
];

export const DEFAULT_SEARCH_STATE: SearchFormState = {
  query: "",
  mode: "browse",
  visibleFilterIds: [],
  hiddenFilterIds: [],
  filterClauses: DEFAULT_FILTER_CLAUSES,
  sort: "record_key",
  pageSize: 25,
  includeDiagnostics: false,
};

export const SORT_OPTIONS: Array<{ value: SortKey; label: string }> = [
  { value: "record_key", label: "Record Key" },
  { value: "alphabetical", label: "Alphabetical" },
  { value: "level_asc", label: "Level Up" },
  { value: "level_desc", label: "Level Down" },
  { value: "price_asc", label: "Price Up" },
  { value: "price_desc", label: "Price Down" },
];

const MODE_VALUES = ["browse", "text_search"] as const;
const SORT_VALUES = SORT_OPTIONS.map(({ value }) => value);
const FILTER_OPERATORS: readonly FilterClauseOperator[] = [
  "include_any",
  "include_all",
  "exclude_any",
  "range",
  "metric_compare",
];
const METRIC_OPERATORS = ["gt", "gte", "lt", "lte", "eq"] as const;
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

export function buildFilterDiscoveryContext(
  state: SearchFormState,
): FilterDiscoveryContext {
  return {
    kind: "filtered",
    filter: buildBasicFilter(state),
  };
}

export function encodeSearchState(state: SearchFormState): string {
  return encodeURIComponent(JSON.stringify(state));
}

export function encodeSearchExecutionState(state: SearchFormState): string {
  const {
    visibleFilterIds: _visibleFilterIds,
    hiddenFilterIds: _hiddenFilterIds,
    ...executionState
  } = state;
  return encodeURIComponent(JSON.stringify(executionState));
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
      visibleFilterIds: dedupeStrings(
        stringArrayValue(
          decoded.visibleFilterIds,
          DEFAULT_SEARCH_STATE.visibleFilterIds,
        ),
      ),
      hiddenFilterIds: dedupeStrings(
        stringArrayValue(decoded.hiddenFilterIds, DEFAULT_SEARCH_STATE.hiddenFilterIds),
      ),
      filterClauses:
        filterClausesValue(decoded.filterClauses) ?? DEFAULT_SEARCH_STATE.filterClauses,
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
  return { clauses: filterClausesValue(state.filterClauses) ?? [] };
}

function filterClausesValue(value: unknown): FilterClause[] | null {
  if (!Array.isArray(value)) {
    return null;
  }
  return value.flatMap((item) => {
    if (!isRecord(item)) {
      return [];
    }
    const id = stringValue(item.id, "");
    const field = stringValue(item.field, "");
    const operator = oneOf<FilterClauseOperator>(
      item.operator,
      FILTER_OPERATORS,
      "include_any",
    );
    if (!id || !field) {
      return [];
    }
    const clause: FilterClause = {
      id,
      field,
      operator,
      values: stringArrayValue(item.values, []),
    };
    const values = clause.values ?? [];
    if (operator === "range") {
      const range = rangeValue(item.range);
      if (!range) {
        return [];
      }
      clause.range = {
        min: range.min ?? undefined,
        max: range.max ?? undefined,
      };
      clause.values = [];
      return [clause];
    }
    if (operator === "metric_compare") {
      const metric = metricValue(item.metric);
      if (!metric) {
        return [];
      }
      clause.metric = metric;
      clause.values = [];
      return [clause];
    }
    if (values.length === 0) {
      return [];
    }
    return [clause];
  });
}

function dedupeStrings(values: string[]): string[] {
  return Array.from(new Set(values));
}

function rangeValue(value: unknown): NumericRangeState | null {
  if (!isRecord(value)) {
    return null;
  }
  const range = {
    min: nullableFiniteNumber(value.min, null),
    max: nullableFiniteNumber(value.max, null),
  };
  return range.min === null && range.max === null ? null : range;
}

function metricValue(value: unknown): MetricComparison | null {
  if (!isRecord(value)) {
    return null;
  }
  const key = stringValue(value.key, "").trim();
  const op = oneOf(value.op, METRIC_OPERATORS, "gte");
  const metricValue =
    typeof value.value === "number" && Number.isFinite(value.value)
      ? value.value
      : null;
  if (!key || metricValue === null) {
    return null;
  }
  return { key, op, value: metricValue };
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

function nullableFiniteNumber(value: unknown, fallback: number | null): number | null {
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
