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

export const DEFAULT_SEARCH_STATE: SearchFormState = {
  query: "",
  mode: "browse",
  visibleFilterIds: [],
  hiddenFilterIds: [],
  filterClauses: [],
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
const COMPACT_INCLUDE_ANY_FIELDS = ["kind", "rarity", "pack"] as const;
const COMPACT_RANGE_FIELDS = ["level", "price"] as const;

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
  return encodeURIComponent(JSON.stringify(searchStateUrlPayload(state)));
}

export function encodeSearchExecutionState(state: SearchFormState): string {
  const {
    visibleFilterIds: _visibleFilterIds,
    hiddenFilterIds: _hiddenFilterIds,
    ...executionState
  } = state;
  return encodeURIComponent(JSON.stringify(executionState));
}

export function searchStateQueryString(state: SearchFormState): string {
  const compact = compactSearchStateParams(state);
  const params = compact ?? readableSearchStateParams(state);
  const query = params.toString();
  return query ? `?${query}` : "";
}

export function decodeSearchStateFromParams(params: URLSearchParams): SearchFormState {
  const legacyState = params.get("s");
  if (legacyState) {
    return decodeSearchState(legacyState);
  }

  const clauses = compactFilterClausesFromParams(params);
  return {
    ...DEFAULT_SEARCH_STATE,
    query: params.get("q") ?? DEFAULT_SEARCH_STATE.query,
    mode: compactModeValue(params),
    visibleFilterIds: dedupeStrings(params.getAll("show-filter")),
    hiddenFilterIds: dedupeStrings(params.getAll("hide-filter")),
    filterClauses: clauses,
    sort: oneOf(params.get("sort"), SORT_VALUES, DEFAULT_SEARCH_STATE.sort),
    pageSize: pageSizeParamValue(
      params.get("limit") ?? params.get("pageSize"),
      DEFAULT_SEARCH_STATE.pageSize,
    ),
    includeDiagnostics: booleanParamValue(
      params.get("diagnostics"),
      DEFAULT_SEARCH_STATE.includeDiagnostics,
    ),
  };
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

function searchStateUrlPayload(state: SearchFormState): Partial<SearchFormState> {
  const payload: Partial<SearchFormState> = {};
  if (state.query !== DEFAULT_SEARCH_STATE.query) {
    payload.query = state.query;
  }
  if (state.mode !== DEFAULT_SEARCH_STATE.mode) {
    payload.mode = state.mode;
  }
  if (
    !stringArraysEqual(state.visibleFilterIds, DEFAULT_SEARCH_STATE.visibleFilterIds)
  ) {
    payload.visibleFilterIds = state.visibleFilterIds;
  }
  if (!stringArraysEqual(state.hiddenFilterIds, DEFAULT_SEARCH_STATE.hiddenFilterIds)) {
    payload.hiddenFilterIds = state.hiddenFilterIds;
  }
  if (!filterClausesEqual(state.filterClauses, DEFAULT_SEARCH_STATE.filterClauses)) {
    payload.filterClauses = state.filterClauses;
  }
  if (state.sort !== DEFAULT_SEARCH_STATE.sort) {
    payload.sort = state.sort;
  }
  if (state.pageSize !== DEFAULT_SEARCH_STATE.pageSize) {
    payload.pageSize = state.pageSize;
  }
  if (state.includeDiagnostics !== DEFAULT_SEARCH_STATE.includeDiagnostics) {
    payload.includeDiagnostics = state.includeDiagnostics;
  }
  return payload;
}

function stringArraysEqual(left: string[], right: string[]): boolean {
  return (
    left.length === right.length && left.every((value, index) => value === right[index])
  );
}

function filterClausesEqual(left: FilterClause[], right: FilterClause[]): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

function compactSearchStateParams(state: SearchFormState): URLSearchParams | null {
  const params = baseSearchStateParams(state);
  if (
    !stringArraysEqual(state.visibleFilterIds, DEFAULT_SEARCH_STATE.visibleFilterIds) ||
    !stringArraysEqual(state.hiddenFilterIds, DEFAULT_SEARCH_STATE.hiddenFilterIds)
  ) {
    return null;
  }
  if (!appendCompactClauses(params, state.filterClauses)) {
    return null;
  }
  return params;
}

function readableSearchStateParams(state: SearchFormState): URLSearchParams {
  const params = baseSearchStateParams(state);
  for (const clause of state.filterClauses) {
    appendReadableClause(params, clause);
  }
  for (const fieldId of state.visibleFilterIds) {
    params.append("show-filter", fieldId);
  }
  for (const fieldId of state.hiddenFilterIds) {
    params.append("hide-filter", fieldId);
  }
  return params;
}

function baseSearchStateParams(state: SearchFormState): URLSearchParams {
  const params = new URLSearchParams();
  if (state.query !== DEFAULT_SEARCH_STATE.query) {
    params.set("q", state.query);
  }
  if (state.mode !== DEFAULT_SEARCH_STATE.mode) {
    params.set("mode", state.mode === "text_search" ? "text" : state.mode);
  }
  if (state.sort !== DEFAULT_SEARCH_STATE.sort) {
    params.set("sort", state.sort);
  }
  if (state.pageSize !== DEFAULT_SEARCH_STATE.pageSize) {
    params.set("limit", state.pageSize.toString());
  }
  if (state.includeDiagnostics !== DEFAULT_SEARCH_STATE.includeDiagnostics) {
    params.set("diagnostics", "true");
  }
  return params;
}

function appendCompactClauses(
  params: URLSearchParams,
  clauses: FilterClause[],
): boolean {
  for (const clause of clauses) {
    if (!appendCompactClause(params, clause)) {
      return false;
    }
  }
  return true;
}

function appendCompactClause(params: URLSearchParams, clause: FilterClause): boolean {
  if (clause.operator === "include_any") {
    return appendRepeatedValues(params, clause.field, clause.values ?? []);
  }
  if (clause.operator === "exclude_any") {
    return appendRepeatedValues(params, clause.field, clause.values ?? [], "exclude");
  }
  if (clause.operator === "include_all" && clause.field === "traits") {
    for (const value of clause.values ?? []) {
      params.append("trait", value);
    }
    return true;
  }
  if (clause.operator === "range") {
    const key = compactRangeParam(clause.field);
    if (!key || !clause.range) {
      return false;
    }
    params.set(key, rangeParamValue(clause.range));
    return true;
  }
  if (clause.operator === "metric_compare") {
    if (!clause.metric) {
      return false;
    }
    params.append("metric", metricParamValue(clause.metric));
    return true;
  }
  return false;
}

function appendReadableClause(params: URLSearchParams, clause: FilterClause) {
  if (appendCompactClause(params, clause)) {
    return;
  }
  if (clause.operator === "include_any" || clause.operator === "exclude_any") {
    const prefix = clause.operator === "include_any" ? "include" : "exclude";
    for (const value of clause.values ?? []) {
      params.append(`${prefix}-${clause.field}`, value);
    }
    return;
  }
  if (clause.operator === "include_all") {
    for (const value of clause.values ?? []) {
      params.append(`require-${clause.field}`, value);
    }
    return;
  }
  if (clause.operator === "range" && clause.range) {
    params.set(`range-${clause.field}`, rangeParamValue(clause.range));
    return;
  }
  if (clause.operator === "metric_compare" && clause.metric) {
    params.append("metric", metricParamValue(clause.metric));
  }
}

function appendRepeatedValues(
  params: URLSearchParams,
  field: string,
  values: string[],
  mode: "include" | "exclude" = "include",
): boolean {
  const key =
    mode === "exclude" ? compactExcludeAnyParam(field) : compactIncludeAnyParam(field);
  if (!key) {
    return false;
  }
  for (const value of values) {
    params.append(key, value);
  }
  return true;
}

function compactIncludeAnyParam(field: string): string | null {
  if (field === "traits") {
    return "any-trait";
  }
  return (COMPACT_INCLUDE_ANY_FIELDS as readonly string[]).includes(field)
    ? field
    : null;
}

function compactExcludeAnyParam(field: string): string | null {
  if (field === "traits") {
    return "exclude-trait";
  }
  return (COMPACT_INCLUDE_ANY_FIELDS as readonly string[]).includes(field)
    ? `exclude-${field}`
    : null;
}

function compactRangeParam(field: string): string | null {
  return (COMPACT_RANGE_FIELDS as readonly string[]).includes(field) ? field : null;
}

function rangeParamValue(range: { min?: number; max?: number }): string {
  if (range.min !== undefined && range.max !== undefined && range.min === range.max) {
    return range.min.toString();
  }
  if (range.min !== undefined || range.max !== undefined) {
    return `${range.min ?? ""}..${range.max ?? ""}`;
  }
  return "";
}

function metricParamValue(metric: MetricComparison): string {
  return `${metric.key}${metricOperatorToken(metric.op)}${metric.value}`;
}

function metricOperatorToken(op: MetricComparison["op"]): string {
  switch (op) {
    case "gt":
      return ">";
    case "gte":
      return ">=";
    case "lt":
      return "<";
    case "lte":
      return "<=";
    case "eq":
      return "=";
    default:
      return "=";
  }
}

function compactFilterClausesFromParams(params: URLSearchParams): FilterClause[] {
  return [
    repeatedValueClause("kind", "kind", params.getAll("kind"), "include_any"),
    repeatedValueClause("kind", "kind", params.getAll("exclude-kind"), "exclude_any"),
    repeatedValueClause("traits", "traits", params.getAll("trait"), "include_all"),
    repeatedValueClause("traits", "traits", params.getAll("any-trait"), "include_any"),
    repeatedValueClause(
      "traits",
      "traits",
      params.getAll("exclude-trait"),
      "exclude_any",
    ),
    repeatedValueClause("rarity", "rarity", params.getAll("rarity"), "include_any"),
    repeatedValueClause(
      "rarity",
      "rarity",
      params.getAll("exclude-rarity"),
      "exclude_any",
    ),
    repeatedValueClause("pack", "pack", params.getAll("pack"), "include_any"),
    repeatedValueClause("pack", "pack", params.getAll("exclude-pack"), "exclude_any"),
    rangeClause("level", params.get("level")),
    rangeClause("price", params.get("price")),
    ...params.getAll("metric").map(metricClause),
    ...genericFilterClausesFromParams(params),
  ].flatMap((clause) => (clause ? [clause] : []));
}

function genericFilterClausesFromParams(
  params: URLSearchParams,
): Array<FilterClause | null> {
  const clauses: Array<FilterClause | null> = [];
  const compactPrefixedKeys = new Set([
    "exclude-kind",
    "exclude-trait",
    "exclude-rarity",
    "exclude-pack",
  ]);
  for (const key of new Set(params.keys())) {
    if (compactPrefixedKeys.has(key)) {
      continue;
    }
    if (key.startsWith("include-")) {
      const field = key.slice("include-".length);
      clauses.push(
        repeatedValueClause(field, field, params.getAll(key), "include_any"),
      );
    } else if (key.startsWith("exclude-")) {
      const field = key.slice("exclude-".length);
      clauses.push(
        repeatedValueClause(field, field, params.getAll(key), "exclude_any"),
      );
    } else if (key.startsWith("require-")) {
      const field = key.slice("require-".length);
      clauses.push(
        repeatedValueClause(field, field, params.getAll(key), "include_all"),
      );
    } else if (key.startsWith("range-")) {
      const field = key.slice("range-".length);
      clauses.push(rangeClause(field, params.get(key)));
    }
  }
  return clauses;
}

function repeatedValueClause(
  idField: string,
  field: string,
  values: string[],
  operator: Extract<
    FilterClauseOperator,
    "include_any" | "include_all" | "exclude_any"
  >,
): FilterClause | null {
  const deduped = dedupeStrings(values.filter((value) => value.length > 0));
  if (deduped.length === 0) {
    return null;
  }
  return {
    id: `${idField}-${operator}`,
    field,
    operator,
    values: deduped,
  };
}

function rangeClause(field: string, value: string | null): FilterClause | null {
  if (!value) {
    return null;
  }
  const range = parseRangeParam(value);
  if (!range) {
    return null;
  }
  return {
    id: `${field}-range`,
    field,
    operator: "range",
    values: [],
    range,
  };
}

function parseRangeParam(value: string): { min?: number; max?: number } | null {
  if (value.includes("..")) {
    const [min, max] = value.split("..", 2);
    const range = {
      min: min ? finiteNumber(min) : undefined,
      max: max ? finiteNumber(max) : undefined,
    };
    return range.min === undefined && range.max === undefined ? null : range;
  }
  const exact = finiteNumber(value);
  return exact === undefined ? null : { min: exact, max: exact };
}

function metricClause(value: string): FilterClause | null {
  const metric = parseMetricParam(value);
  if (!metric) {
    return null;
  }
  return {
    id: `metric-${metric.op}-${metric.key}`,
    field: "metric",
    operator: "metric_compare",
    values: [],
    metric,
  };
}

function parseMetricParam(value: string): MetricComparison | null {
  const match = value.match(/^(.+?)(>=|<=|>|<|=|:)(.+)$/);
  if (!match) {
    return null;
  }
  const [, rawKey, token, rawValue] = match;
  const key = rawKey.trim();
  const metricValue = finiteNumber(rawValue.trim());
  if (!key || metricValue === undefined) {
    return null;
  }
  return {
    key,
    op: metricOperator(token),
    value: metricValue,
  };
}

function metricOperator(token: string): MetricComparison["op"] {
  switch (token) {
    case ">":
      return "gt";
    case ">=":
      return "gte";
    case "<":
      return "lt";
    case "<=":
      return "lte";
    case "=":
    case ":":
    default:
      return "eq";
  }
}

function finiteNumber(value: string): number | undefined {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : undefined;
}

function compactModeValue(params: URLSearchParams): SearchFormState["mode"] {
  const mode = params.get("mode");
  if (mode === "text" || mode === "text_search") {
    return "text_search";
  }
  if (mode === "browse") {
    return "browse";
  }
  return params.has("q") ? "text_search" : DEFAULT_SEARCH_STATE.mode;
}

function booleanParamValue(value: string | null, fallback: boolean): boolean {
  if (value === null) {
    return fallback;
  }
  return value === "true" || value === "1";
}

function pageSizeParamValue(value: string | null, fallback: number): number {
  if (value === null) {
    return fallback;
  }
  const parsed = Number(value);
  return pageSizeValue(parsed, fallback);
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
