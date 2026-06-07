import type {
  AppError,
  AppReadinessView,
  DiscoverFilterFieldsRequest,
  DiscoverFilterValuesRequest,
  FilterFieldListView,
  FilterValueListView,
  OpenResultWindowRequest,
  ReadResultWindowPageRequest,
  RecordDetailView,
  ResultWindowPage,
} from "../generated/atlas";

const API_BASE = import.meta.env.VITE_ATLAS_API_BASE ?? "";

export class AtlasApiError extends Error {
  readonly appError?: AppError;
  readonly status: number;

  constructor(status: number, message: string, appError?: AppError) {
    super(message);
    this.name = "AtlasApiError";
    this.status = status;
    this.appError = appError;
  }
}

export async function getReadiness(): Promise<AppReadinessView> {
  return atlasFetch("/api/readiness");
}

export async function discoverFilterFields(
  request: DiscoverFilterFieldsRequest,
): Promise<FilterFieldListView> {
  const fields = await atlasFetch<unknown>("/api/filters/fields", {
    method: "POST",
    body: JSON.stringify(request),
  });
  return normalizeFilterFieldList(fields);
}

export async function discoverFilterValues(
  request: DiscoverFilterValuesRequest,
): Promise<FilterValueListView> {
  const values = await atlasFetch<unknown>("/api/filters/values", {
    method: "POST",
    body: JSON.stringify(request),
  });
  return normalizeFilterValueList(values);
}

export async function openResultWindow(
  request: OpenResultWindowRequest,
): Promise<ResultWindowPage> {
  const page = await atlasFetch<unknown>("/api/result-windows", {
    method: "POST",
    body: JSON.stringify(request),
  });
  return normalizeResultWindowPage(page);
}

export async function readResultWindowPage(
  windowId: bigint,
  request: ReadResultWindowPageRequest,
): Promise<ResultWindowPage> {
  const page = await atlasFetch<unknown>(
    `/api/result-windows/${windowId.toString()}/page`,
    {
      method: "POST",
      body: JSON.stringify(request),
    },
  );
  return normalizeResultWindowPage(page);
}

export async function getRecordDetail(recordKey: string): Promise<RecordDetailView> {
  return atlasFetch(`/api/records/${encodeURIComponent(recordKey)}`);
}

async function atlasFetch<T>(path: string, init: RequestInit = {}): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...init.headers,
    },
  });
  const text = await response.text();
  const parsed = parseJsonResponse(text);
  const payload = parsed.ok ? parsed.value : undefined;

  if (!response.ok) {
    const appError = isAppError(payload) ? payload : undefined;
    throw new AtlasApiError(
      response.status,
      appError?.message ?? fallbackErrorMessage(response, text, parsed),
      appError,
    );
  }

  if (!parsed.ok) {
    throw new AtlasApiError(
      response.status,
      fallbackErrorMessage(response, text, parsed),
    );
  }

  return payload as T;
}

function parseJsonResponse(
  text: string,
): { ok: true; value: unknown } | { ok: false; error?: unknown } {
  if (text.length === 0) {
    return { ok: true, value: undefined };
  }
  try {
    return { ok: true, value: JSON.parse(text) };
  } catch (error) {
    return { ok: false, error };
  }
}

function fallbackErrorMessage(
  response: Response,
  text: string,
  parsed: { ok: true; value: unknown } | { ok: false; error?: unknown },
): string {
  if (!parsed.ok && text.trim().length > 0) {
    return text;
  }
  if (!parsed.ok) {
    return "Invalid JSON response";
  }
  return response.statusText || `HTTP ${response.status}`;
}

function normalizeResultWindowPage(value: unknown): ResultWindowPage {
  if (!isRecord(value)) {
    throw new AtlasApiError(200, "Invalid result-window response");
  }
  const page = isRecord(value.page) ? value.page : {};
  return {
    ...value,
    window_id: toBigInt(value.window_id),
    page: {
      ...page,
      total: toBigInt(page.total),
    },
  } as ResultWindowPage;
}

function normalizeFilterFieldList(value: unknown): FilterFieldListView {
  if (!isRecord(value)) {
    throw new AtlasApiError(200, "Invalid filter-field response");
  }
  return {
    ...value,
    matching_record_count: toBigInt(value.matching_record_count),
  } as FilterFieldListView;
}

function normalizeFilterValueList(value: unknown): FilterValueListView {
  if (!isRecord(value)) {
    throw new AtlasApiError(200, "Invalid filter-value response");
  }
  return {
    ...value,
    matching_record_count: toBigInt(value.matching_record_count),
    options: Array.isArray(value.options)
      ? value.options.map((option) =>
          isRecord(option) && "count" in option
            ? { ...option, count: toBigInt(option.count) }
            : option,
        )
      : [],
  } as FilterValueListView;
}

function toBigInt(value: unknown): bigint {
  if (typeof value === "bigint") {
    return value;
  }
  if (typeof value === "number" || typeof value === "string") {
    return BigInt(value);
  }
  throw new AtlasApiError(200, "Invalid numeric field in API response");
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isAppError(value: unknown): value is AppError {
  return (
    typeof value === "object" && value !== null && "code" in value && "message" in value
  );
}
