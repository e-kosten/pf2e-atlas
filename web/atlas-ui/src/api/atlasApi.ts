import type {
  AddEncounterManualParticipantRequest,
  AddEncounterParticipantConditionRequest,
  AddEncounterRecordParticipantRequest,
  AddSavedListItemRequest,
  AppError,
  AppReadinessView,
  CreateEncounterRequest,
  CreateSavedListRequest,
  DeleteEncounterView,
  DeleteSavedListView,
  DiscoverFilterEditorRequest,
  DiscoverFilterValuesRequest,
  EncounterCreateView,
  EncounterConditionCatalogView,
  EncounterDetailView,
  EncounterIndexView,
  EncounterParticipantView,
  EncounterParticipantResetResultView,
  EncounterSpellCastRequest,
  EncounterSpellCastResultView,
  EncounterUpdateView,
  FilterEditorView,
  FilterSavedListRequest,
  FilterValueListView,
  OpenResultWindowRequest,
  ReadResultWindowPageRequest,
  RecordDetailView,
  RemoveSavedListItemRequest,
  ReorderEncounterParticipantRequest,
  ResetEncounterParticipantRequest,
  SavedListCreateView,
  ResultWindowPage,
  SavedListDetailView,
  SavedListIndexView,
  SavedListItemMutationView,
  SavedListUpdateView,
  SetEncounterTurnRequest,
  UpdateEncounterParticipantConditionRequest,
  UpdateEncounterParticipantRequest,
  UpdateEncounterRequest,
  UpdateSavedListRequest,
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

export async function discoverFilterEditor(
  request: DiscoverFilterEditorRequest,
): Promise<FilterEditorView> {
  const editor = await atlasFetch<unknown>("/api/filters/editor", {
    method: "POST",
    body: jsonBody(request),
  });
  return normalizeFilterEditor(editor);
}

export async function discoverFilterValues(
  request: DiscoverFilterValuesRequest,
): Promise<FilterValueListView> {
  const values = await atlasFetch<unknown>("/api/filters/values", {
    method: "POST",
    body: jsonBody(request),
  });
  return normalizeFilterValueList(values);
}

export async function openResultWindow(
  request: OpenResultWindowRequest,
): Promise<ResultWindowPage> {
  const page = await atlasFetch<unknown>("/api/result-windows", {
    method: "POST",
    body: jsonBody(request),
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
      body: jsonBody(request),
    },
  );
  return normalizeResultWindowPage(page);
}

export async function getRecordDetail(recordKey: string): Promise<RecordDetailView> {
  return atlasFetch(`/api/records/${encodeURIComponent(recordKey)}`);
}

export async function getEncounters(): Promise<EncounterIndexView> {
  return atlasFetch("/api/encounters");
}

export async function getEncounterConditionDefinitions(): Promise<EncounterConditionCatalogView> {
  return atlasFetch("/api/encounters/condition-definitions");
}

export async function getEncounter(encounterRef: string): Promise<EncounterDetailView> {
  return atlasFetch(`/api/encounters/${encodeURIComponent(encounterRef)}`);
}

export async function createEncounter(
  request: CreateEncounterRequest,
): Promise<EncounterCreateView> {
  return atlasFetch("/api/encounters", {
    method: "POST",
    body: jsonBody(request),
  });
}

export async function deleteEncounter(
  encounterRef: string,
): Promise<DeleteEncounterView> {
  return atlasFetch(`/api/encounters/${encodeURIComponent(encounterRef)}`, {
    method: "DELETE",
  });
}

export async function updateEncounter(
  request: UpdateEncounterRequest,
): Promise<EncounterUpdateView> {
  return atlasFetch(`/api/encounters/${encodeURIComponent(request.encounter_key)}`, {
    method: "PATCH",
    body: jsonBody(request),
  });
}

export async function addEncounterRecordParticipant(
  request: AddEncounterRecordParticipantRequest,
): Promise<EncounterDetailView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(request.encounter_ref)}/participants/record`,
    {
      method: "POST",
      body: jsonBody(request),
    },
  );
}

export async function addEncounterManualParticipant(
  request: AddEncounterManualParticipantRequest,
): Promise<EncounterDetailView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(request.encounter_ref)}/participants/manual`,
    {
      method: "POST",
      body: jsonBody(request),
    },
  );
}

export async function updateEncounterParticipant(
  encounterRef: string,
  request: UpdateEncounterParticipantRequest,
): Promise<EncounterParticipantView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(request.participant_key)}`,
    {
      method: "PATCH",
      body: jsonBody(request),
    },
  );
}

export async function reorderEncounterParticipant(
  encounterRef: string,
  request: ReorderEncounterParticipantRequest,
): Promise<EncounterDetailView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/reorder`,
    {
      method: "POST",
      body: jsonBody(request),
    },
  );
}

export async function removeEncounterParticipant(
  encounterRef: string,
  participantKey: string,
): Promise<EncounterDetailView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(participantKey)}`,
    { method: "DELETE" },
  );
}

export async function setEncounterTurn(
  request: SetEncounterTurnRequest,
): Promise<EncounterDetailView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(request.encounter_ref)}/turn`,
    {
      method: "POST",
      body: jsonBody(request),
    },
  );
}

export async function addEncounterParticipantCondition(
  encounterRef: string,
  request: AddEncounterParticipantConditionRequest,
): Promise<EncounterDetailView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(request.participant_key)}/conditions`,
    {
      method: "POST",
      body: jsonBody(request),
    },
  );
}

export async function updateEncounterParticipantCondition(
  encounterRef: string,
  participantKey: string,
  request: UpdateEncounterParticipantConditionRequest,
): Promise<EncounterDetailView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(participantKey)}/conditions/${safeIntegerPathSegment(request.condition_id)}`,
    {
      method: "PATCH",
      body: jsonBody(request),
    },
  );
}

export async function removeEncounterParticipantCondition(
  encounterRef: string,
  participantKey: string,
  conditionId: number,
): Promise<EncounterDetailView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(participantKey)}/conditions/${safeIntegerPathSegment(conditionId)}`,
    { method: "DELETE" },
  );
}

export async function mutateEncounterSpellCast(
  encounterRef: string,
  participantKey: string,
  request: EncounterSpellCastRequest,
): Promise<EncounterSpellCastResultView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(participantKey)}/spell-casts`,
    {
      method: "POST",
      body: jsonBody(request),
    },
  );
}

export async function resetEncounterParticipant(
  encounterRef: string,
  participantKey: string,
  request: ResetEncounterParticipantRequest,
): Promise<EncounterParticipantResetResultView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(participantKey)}/reset`,
    {
      method: "POST",
      body: jsonBody(request),
    },
  );
}

export async function getSavedLists(): Promise<SavedListIndexView> {
  return atlasFetch("/api/lists");
}

export async function getSavedList(listRef: string): Promise<SavedListDetailView> {
  const list = await atlasFetch<unknown>(`/api/lists/${encodeURIComponent(listRef)}`);
  return normalizeSavedListDetail(list);
}

export async function filterSavedList(
  request: FilterSavedListRequest,
): Promise<SavedListDetailView> {
  const list = await atlasFetch<unknown>(
    `/api/lists/${encodeURIComponent(request.list_ref)}/filter`,
    {
      method: "POST",
      body: jsonBody(request),
    },
  );
  return normalizeSavedListDetail(list);
}

export async function createSavedList(
  request: CreateSavedListRequest,
): Promise<SavedListCreateView> {
  return atlasFetch("/api/lists", {
    method: "POST",
    body: jsonBody(request),
  });
}

export async function updateSavedList(
  request: UpdateSavedListRequest,
): Promise<SavedListUpdateView> {
  return atlasFetch(`/api/lists/${encodeURIComponent(request.list_key)}`, {
    method: "PATCH",
    body: jsonBody(request),
  });
}

export async function deleteSavedList(listRef: string): Promise<DeleteSavedListView> {
  return atlasFetch(`/api/lists/${encodeURIComponent(listRef)}`, {
    method: "DELETE",
  });
}

export async function addSavedListItem(
  request: AddSavedListItemRequest,
): Promise<SavedListItemMutationView> {
  return atlasFetch(`/api/lists/${encodeURIComponent(request.list_ref)}/items`, {
    method: "POST",
    body: jsonBody(request),
  });
}

export async function removeSavedListItem(
  request: RemoveSavedListItemRequest,
): Promise<SavedListItemMutationView> {
  return atlasFetch(`/api/lists/${encodeURIComponent(request.list_ref)}/items`, {
    method: "DELETE",
    body: jsonBody(request),
  });
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

function jsonBody(value: unknown): string {
  return JSON.stringify(value, (_key, nestedValue) => {
    if (
      typeof nestedValue === "number" &&
      Number.isInteger(nestedValue) &&
      !Number.isSafeInteger(nestedValue)
    ) {
      throw new AtlasApiError(
        0,
        "Request numeric field exceeds JSON safe integer range",
      );
    }
    if (typeof nestedValue !== "bigint") {
      return nestedValue;
    }
    if (
      nestedValue > BigInt(Number.MAX_SAFE_INTEGER) ||
      nestedValue < BigInt(Number.MIN_SAFE_INTEGER)
    ) {
      throw new AtlasApiError(
        0,
        "Request numeric field exceeds JSON safe integer range",
      );
    }
    return Number(nestedValue);
  });
}

function safeIntegerPathSegment(value: number): string {
  if (!Number.isSafeInteger(value)) {
    throw new AtlasApiError(0, "Request numeric field exceeds JSON safe integer range");
  }
  return value.toString();
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

function normalizeFilterEditor(value: unknown): FilterEditorView {
  if (!isRecord(value)) {
    throw new AtlasApiError(200, "Invalid filter-editor response");
  }
  return {
    ...value,
    matching_record_count: toBigInt(value.matching_record_count),
  } as FilterEditorView;
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

function normalizeSavedListDetail(value: unknown): SavedListDetailView {
  if (!isRecord(value)) {
    throw new AtlasApiError(200, "Invalid saved-list response");
  }
  return {
    ...value,
    items: Array.isArray(value.items)
      ? value.items.map((item) =>
          isRecord(item) ? { ...item, position: toBigInt(item.position) } : item,
        )
      : [],
  } as SavedListDetailView;
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
