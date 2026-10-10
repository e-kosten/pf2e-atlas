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
  EncounterUpdateView,
  FilterEditorView,
  FilterSavedListRequest,
  FilterValueListView,
  OpenResultWindowRequest,
  ReadResultWindowPageRequest,
  RecordDetailView,
  RecordDetailRequest,
  RemoveSavedListItemRequest,
  ReorderEncounterParticipantRequest,
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
  return editor as FilterEditorView;
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

export async function getRecordDetail(
  recordKey: string,
  selection?: RecordDetailRequest,
): Promise<RecordDetailView> {
  return selection
    ? atlasFetch("/api/records/detail", { method: "POST", body: jsonBody(selection) })
    : atlasFetch(`/api/records/${encodeURIComponent(recordKey)}`);
}

export async function getEncounters(): Promise<EncounterIndexView> {
  return atlasFetchEncounter("/api/encounters");
}

export async function getEncounterConditionDefinitions(): Promise<EncounterConditionCatalogView> {
  return atlasFetchEncounter("/api/encounters/condition-definitions");
}

export async function getEncounter(encounterRef: string): Promise<EncounterDetailView> {
  return atlasFetchEncounter(`/api/encounters/${encodeURIComponent(encounterRef)}`);
}

export async function createEncounter(
  request: CreateEncounterRequest,
): Promise<EncounterCreateView> {
  return atlasFetchEncounter("/api/encounters", {
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
  return atlasFetchEncounter(
    `/api/encounters/${encodeURIComponent(request.encounter_key)}`,
    {
      method: "PATCH",
      body: jsonBody(request),
    },
  );
}

export async function addEncounterRecordParticipant(
  request: AddEncounterRecordParticipantRequest,
): Promise<EncounterDetailView> {
  return atlasFetchEncounter(
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
  return atlasFetchEncounter(
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
  return atlasFetchEncounter(
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
  return atlasFetchEncounter(
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
  return atlasFetchEncounter(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(participantKey)}`,
    { method: "DELETE" },
  );
}

export async function setEncounterTurn(
  request: SetEncounterTurnRequest,
): Promise<EncounterDetailView> {
  return atlasFetchEncounter(
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
  return atlasFetchEncounter(
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
  return atlasFetchEncounter(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(participantKey)}/conditions/${request.condition_id.toString()}`,
    {
      method: "PATCH",
      body: jsonBody(request),
    },
  );
}

export async function removeEncounterParticipantCondition(
  encounterRef: string,
  participantKey: string,
  conditionId: bigint,
): Promise<EncounterDetailView> {
  return atlasFetchEncounter(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(participantKey)}/conditions/${conditionId.toString()}`,
    { method: "DELETE" },
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

async function atlasFetchEncounter<T>(
  path: string,
  init: RequestInit = {},
): Promise<T> {
  const payload = await atlasFetch<unknown>(path, init);
  if (!isRecord(payload)) throw new AtlasApiError(200, "Invalid encounter response");
  const result = { ...payload };
  const summary = (value: unknown) => {
    if (!isRecord(value)) throw new AtlasApiError(200, "Invalid encounter summary");
    return { ...value, round_number: toBigInt(value.round_number) };
  };
  const optionalIntegers = (value: Record<string, unknown>, fields: string[]) =>
    Object.fromEntries(
      fields
        .filter((field) => value[field] !== undefined)
        .map((field) => [field, toBigInt(value[field])]),
    );
  const condition = (value: unknown) => {
    if (!isRecord(value)) throw new AtlasApiError(200, "Invalid encounter condition");
    return {
      ...value,
      condition_id: toBigInt(value.condition_id),
      ...optionalIntegers(value, ["value", "duration_rounds"]),
    };
  };
  const participant = (value: unknown) => {
    if (!isRecord(value) || !Array.isArray(value.conditions))
      throw new AtlasApiError(200, "Invalid encounter participant");
    return {
      ...value,
      position: toBigInt(value.position),
      initiative_order: toBigInt(value.initiative_order),
      temporary_hp: toBigInt(value.temporary_hp),
      ...optionalIntegers(value, ["initiative", "max_hp", "current_hp"]),
      conditions: value.conditions.map(condition),
    };
  };
  if (Array.isArray(result.encounters))
    result.encounters = result.encounters.map(summary);
  if (result.encounter !== undefined) result.encounter = summary(result.encounter);
  if (Array.isArray(result.participants))
    result.participants = result.participants.map(participant);
  if (result.participant_key !== undefined) return participant(result) as T;
  if (Array.isArray(result.conditions))
    result.conditions = result.conditions.map((value) => {
      if (!isRecord(value))
        throw new AtlasApiError(200, "Invalid condition definition");
      return { ...value, ...optionalIntegers(value, ["default_value"]) };
    });
  return result as T;
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
      (!Number.isFinite(nestedValue) ||
        (Number.isInteger(nestedValue) && !Number.isSafeInteger(nestedValue)))
    ) {
      throw new AtlasApiError(
        0,
        "Request numeric field exceeds JSON safe numeric range",
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

function normalizeFilterValueList(value: unknown): FilterValueListView {
  if (!isRecord(value) || !isRecord(value.values))
    throw new AtlasApiError(200, "Invalid filter-value response");
  const values = value.values;
  return {
    values: {
      ...values,
      total_values: toBigInt(values.total_values),
      options: Array.isArray(values.options)
        ? values.options.map((o) =>
            isRecord(o) ? { ...o, distinct_roots: toBigInt(o.distinct_roots) } : o,
          )
        : [],
    },
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
  if (typeof value === "number" && Number.isSafeInteger(value)) {
    return BigInt(value);
  }
  if (typeof value === "string" && /^-?\d+$/.test(value)) {
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

export async function getRecordGraph(
  record_key: string,
  owners: import("../generated/atlas").OwnedContentLocator[] = [],
  source_fingerprint: string | null = null,
): Promise<import("../generated/atlas").GraphContextView | null> {
  return atlasFetch("/api/graph", {
    method: "POST",
    body: jsonBody({
      record_key,
      owners: owners.length ? owners : null,
      source_fingerprint,
      outgoing_limit: 100,
      backlink_limit: 100,
    }),
  });
}
export async function getRecordRemaster(
  key: string,
): Promise<import("../generated/atlas").RemasterLinksView | null> {
  return atlasFetch("/api/records/" + encodeURIComponent(key) + "/remaster");
}
export async function getRecordVariants(
  key: string,
): Promise<import("../generated/atlas").VariantGroupView | null> {
  return atlasFetch("/api/records/" + encodeURIComponent(key) + "/variants");
}
export async function getSimilarRecords(
  key: string,
): Promise<import("../generated/atlas").SimilarRecordsView | null> {
  return atlasFetch("/api/records/" + encodeURIComponent(key) + "/similar");
}
export async function discoverFilterCounts(
  request: import("../generated/atlas").DiscoverFilterCountsRequest,
): Promise<import("../generated/atlas").FilterCountsView> {
  const result = await atlasFetch<import("../generated/atlas").FilterCountsView>(
    "/api/filters/counts",
    { method: "POST", body: jsonBody(request) },
  );
  return {
    ...result,
    counts: {
      ...result.counts,
      states: result.counts.states.map((s) => ({
        ...s,
        occurrences: toBigInt(s.occurrences),
        distinct_roots: toBigInt(s.distinct_roots),
      })),
    },
  };
}
