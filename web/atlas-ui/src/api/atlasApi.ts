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
  EncounterMutationView,
  FilterEditorView,
  FilterSavedListRequest,
  FilterValueListView,
  SearchResultsRequest,
  RecordDetailView,
  RecordPresentationView,
  RecordDetailRequest,
  EmbeddedReadingRequest,
  EmbeddedReadingBatchView,
  RemoveSavedListItemRequest,
  ReorderEncounterParticipantRequest,
  SavedListCreateView,
  SearchResultsPage,
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
  signal?: AbortSignal,
): Promise<FilterEditorView> {
  const editor = await atlasFetch<unknown>("/api/filters/editor", {
    method: "POST",
    body: jsonBody(request),
    signal,
  });
  return editor as FilterEditorView;
}

export async function discoverFilterValues(
  request: DiscoverFilterValuesRequest,
  signal?: AbortSignal,
): Promise<FilterValueListView> {
  const values = await atlasFetch<unknown>("/api/filters/values", {
    method: "POST",
    body: jsonBody(request),
    signal,
  });
  return normalizeFilterValueList(values);
}

export async function searchPage(
  request: SearchResultsRequest,
  signal?: AbortSignal,
): Promise<SearchResultsPage> {
  const page = await atlasFetch<unknown>("/api/search/page", {
    signal,
    method: "POST",
    body: jsonBody(request),
  });
  return normalizeSearchResultsPage(page);
}

export async function getRecordDetail(
  recordKey: string,
  selection?: RecordDetailRequest,
  signal?: AbortSignal,
): Promise<RecordDetailView> {
  return selection
    ? atlasFetch("/api/records/detail", {
        method: "POST",
        body: jsonBody(selection),
        signal,
      })
    : atlasFetch(`/api/records/${encodeURIComponent(recordKey)}`, { signal });
}
export async function getEmbeddedReading(
  request: EmbeddedReadingRequest,
  signal?: AbortSignal,
): Promise<EmbeddedReadingBatchView> {
  return atlasFetch("/api/records/embedded-reading", {
    method: "POST",
    body: jsonBody(request),
    signal,
  });
}
export async function getEncounterReference(
  encounterRef: string,
  participantKey: string,
  request: import("../generated/atlas").EncounterReferenceRequest,
  signal?: AbortSignal,
): Promise<RecordDetailView> {
  const detail = await atlasFetch<RecordDetailView>(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(participantKey)}/reference`,
    {
      signal,
      method: "POST",
      body: jsonBody(request),
    },
  );
  return { ...detail, presentation: normalizeConditionNotes(detail.presentation) };
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
): Promise<EncounterMutationView> {
  return atlasFetch(`/api/encounters/${encodeURIComponent(request.encounter_key)}`, {
    method: "PATCH",
    body: jsonBody(request),
  });
}

export async function addEncounterRecordParticipant(
  request: AddEncounterRecordParticipantRequest,
): Promise<EncounterMutationView> {
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
): Promise<EncounterMutationView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(request.encounter_ref)}/participants/manual`,
    {
      method: "POST",
      body: jsonBody(request),
    },
  );
}

export async function applyEncounterHpAction(
  encounterRef: string,
  request: import("../generated/atlas").ApplyEncounterHpActionRequest,
): Promise<import("../generated/atlas").EncounterMutationView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(request.participant_key)}/hp`,
    { method: "POST", body: jsonBody(request) },
  );
}

export async function applyEncounterSpellAction(
  encounterRef: string,
  request: import("../generated/atlas").ApplyEncounterSpellActionRequest,
): Promise<import("../generated/atlas").EncounterMutationView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(request.participant_key)}/spells`,
    { method: "POST", body: jsonBody(request) },
  );
}

export async function updateEncounterParticipant(
  encounterRef: string,
  request: UpdateEncounterParticipantRequest,
): Promise<EncounterMutationView> {
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
): Promise<EncounterMutationView> {
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
): Promise<EncounterMutationView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(participantKey)}`,
    { method: "DELETE" },
  );
}

export async function getEncounterTurnReview(
  encounterRef: string,
): Promise<import("../generated/atlas").EncounterTurnReviewView> {
  return atlasFetchEncounter(
    `/api/encounters/${encodeURIComponent(encounterRef)}/turn-review`,
  );
}

export async function advanceEncounterTurn(
  encounterRef: string,
  request: import("../generated/atlas").AdvanceEncounterTurnRequest,
): Promise<EncounterMutationView> {
  return atlasFetch(`/api/encounters/${encodeURIComponent(encounterRef)}/advance`, {
    method: "POST",
    body: jsonBody(request),
  });
}

export async function getInitiativeOptions(
  encounterRef: string,
): Promise<import("../generated/atlas").InitiativeOptionsView> {
  const result = await atlasFetch<import("../generated/atlas").InitiativeOptionsView>(
    `/api/encounters/${encodeURIComponent(encounterRef)}/initiative`,
  );
  return {
    ...result,
    expected_revision: toBigInt(result.expected_revision),
    participants: result.participants.map((p) => ({
      ...p,
      expected_revision: toBigInt(p.expected_revision),
    })),
  };
}
export function rollInitiative(
  encounterRef: string,
  request: import("../generated/atlas").RollInitiativeRequest,
): Promise<import("../generated/atlas").InitiativeRollView> {
  return atlasFetch(`/api/encounters/${encodeURIComponent(encounterRef)}/initiative`, {
    method: "POST",
    body: jsonBody(request),
  });
}

export async function setEncounterTurn(
  request: SetEncounterTurnRequest,
): Promise<EncounterMutationView> {
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
): Promise<EncounterMutationView> {
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
): Promise<EncounterMutationView> {
  return atlasFetch(
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
): Promise<EncounterMutationView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(participantKey)}/conditions/${conditionId.toString()}`,
    { method: "DELETE" },
  );
}

export async function getSavedLists(): Promise<SavedListIndexView> {
  return atlasFetch("/api/lists");
}

export async function getSavedList(listRef: string): Promise<SavedListDetailView> {
  return checkedSavedListDetail(
    await atlasFetch(`/api/lists/${encodeURIComponent(listRef)}`),
  );
}

export async function filterSavedList(
  request: FilterSavedListRequest,
): Promise<SavedListDetailView> {
  return checkedSavedListDetail(
    await atlasFetch(`/api/lists/${encodeURIComponent(request.list_ref)}/filter`, {
      method: "POST",
      body: jsonBody(request),
    }),
  );
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

  validateResponseNumbers(payload);
  return payload as T;
}

function validateResponseNumbers(value: unknown): void {
  if (
    typeof value === "number" &&
    (!Number.isFinite(value) ||
      (Number.isInteger(value) && !Number.isSafeInteger(value)))
  )
    throw new AtlasApiError(
      200,
      "Invalid numeric field: response exceeds JSON safe numeric range",
    );
  if (Array.isArray(value)) value.forEach(validateResponseNumbers);
  else if (isRecord(value)) Object.values(value).forEach(validateResponseNumbers);
}

function normalizeConditionNotes(
  presentation: RecordPresentationView,
): RecordPresentationView {
  const body = presentation.body;
  const actor =
    body.kind === "creature"
      ? body.value
      : body.kind === "hazard"
        ? body.value.actor
        : null;
  if (!actor?.runtime) return presentation;
  const value = {
    ...actor,
    runtime: {
      ...actor.runtime,
      unapplied_effects: actor.runtime.unapplied_effects.map((note) =>
        note.condition_id === undefined
          ? note
          : { ...note, condition_id: toBigInt(note.condition_id) },
      ),
    },
  };
  return body.kind === "creature"
    ? { ...presentation, body: { ...body, value } }
    : body.kind === "hazard"
      ? { ...presentation, body: { ...body, value: { ...body.value, actor: value } } }
      : presentation;
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
  const duration = (value: unknown) => {
    if (!isRecord(value)) throw new AtlasApiError(200, "Invalid condition duration");
    return { ...value, amount: toBigInt(value.amount) };
  };
  const condition = (value: unknown) => {
    if (!isRecord(value)) throw new AtlasApiError(200, "Invalid encounter condition");
    return {
      ...value,
      condition_id: toBigInt(value.condition_id),
      ...optionalIntegers(value, ["value"]),
      ...(value.duration === undefined ? {} : { duration: duration(value.duration) }),
    };
  };
  const participant = (value: unknown) => {
    if (!isRecord(value) || !Array.isArray(value.conditions))
      throw new AtlasApiError(200, "Invalid encounter participant");
    if (
      typeof value.revision !== "number" ||
      !Number.isSafeInteger(value.revision) ||
      value.revision < 0
    )
      throw new AtlasApiError(200, "Invalid participant revision in API response");
    return {
      ...value,
      revision: BigInt(value.revision),
      position: toBigInt(value.position),
      initiative_order: toBigInt(value.initiative_order),
      temporary_hp: toBigInt(value.temporary_hp),
      ...optionalIntegers(value, ["initiative", "max_hp", "current_hp"]),
      conditions: value.conditions.map(condition),
      ...(value.presentation === undefined
        ? {}
        : {
            presentation: normalizeConditionNotes(
              value.presentation as RecordPresentationView,
            ),
          }),
    };
  };
  if (Array.isArray(result.encounters))
    result.encounters = result.encounters.map(summary);
  if (result.encounter !== undefined) result.encounter = summary(result.encounter);
  if (Array.isArray(result.participants)) {
    result.revision = toBigInt(result.revision);
    result.participants = result.participants.map(participant);
  }
  if (Array.isArray(result.reminders)) {
    result.expected_revision = toBigInt(result.expected_revision);
    result.reminders = result.reminders.map(condition);
  }
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

function normalizeSearchResultsPage(value: unknown): SearchResultsPage {
  if (!isRecord(value)) {
    throw new AtlasApiError(200, "Invalid search-page response");
  }
  const page = isRecord(value.page) ? value.page : {};
  return {
    ...value,
    page: {
      ...page,
      total: toBigInt(page.total),
    },
  } as SearchResultsPage;
}

function normalizeFilterValueList(value: unknown): FilterValueListView {
  if (!isRecord(value)) throw new AtlasApiError(200, "Invalid filter-value response");
  if (value.kind === "vocabulary" && isRecord(value.values)) {
    return {
      ...value,
      values: { ...value.values, total_values: toBigInt(value.values.total_values) },
    } as FilterValueListView;
  }
  if (value.kind === "contextual" && isRecord(value.outcome)) {
    if (value.outcome.status === "unavailable") return value as FilterValueListView;
    if (value.outcome.status === "available" && isRecord(value.outcome.value)) {
      const values = value.outcome.value;
      return {
        ...value,
        outcome: {
          ...value.outcome,
          value: {
            ...values,
            total_values: toBigInt(values.total_values),
            options: Array.isArray(values.options)
              ? values.options.map((option) =>
                  isRecord(option)
                    ? { ...option, distinct_roots: toBigInt(option.distinct_roots) }
                    : option,
                )
              : [],
          },
        },
      } as FilterValueListView;
    }
  }
  throw new AtlasApiError(200, "Invalid filter-value response");
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

function checkedSavedListDetail(value: unknown): SavedListDetailView {
  if (!isRecord(value) || !Array.isArray(value.items))
    throw new AtlasApiError(200, "Invalid saved-list response");
  return value as SavedListDetailView;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isAppError(value: unknown): value is AppError {
  return (
    typeof value === "object" && value !== null && "code" in value && "message" in value
  );
}

export async function getRelatedRecords(
  request: import("../generated/atlas").RelatedRecordsRequest,
  signal?: AbortSignal,
): Promise<import("../generated/atlas").RelatedRecordsView> {
  return atlasFetch("/api/related", {
    method: "POST",
    body: jsonBody(request),
    signal,
  });
}
export async function discoverFilterCounts(
  request: import("../generated/atlas").DiscoverFilterCountsRequest,
  signal?: AbortSignal,
): Promise<import("../generated/atlas").FilterCountsView> {
  const result = await atlasFetch<import("../generated/atlas").FilterCountsView>(
    "/api/filters/counts",
    { method: "POST", body: jsonBody(request), signal },
  );
  if (result.counts.status === "unavailable") return result;
  return {
    ...result,
    counts: {
      ...result.counts,
      value: {
        ...result.counts.value,
        total_values: toBigInt(result.counts.value.total_values),
        options: result.counts.value.options.map((option) => ({
          ...option,
          distinct_roots: toBigInt(option.distinct_roots),
        })),
      },
    },
  };
}

export async function rollTable(
  request: import("../generated/atlas").RollTableRequest,
): Promise<import("../generated/atlas").TableRollView> {
  return atlasFetch("/api/rolls/table", { method: "POST", body: jsonBody(request) });
}

export async function getEncounterStrikeOptions(
  encounter: string,
  request: import("../generated/atlas").EncounterStrikeOptionsRequest,
): Promise<import("../generated/atlas").EncounterStrikeOptionsView> {
  const result = await atlasFetch<
    import("../generated/atlas").EncounterStrikeOptionsView
  >(`/api/encounters/${encodeURIComponent(encounter)}/strike-options`, {
    method: "POST",
    body: jsonBody(request),
  });
  return {
    ...result,
    expected_revision: toBigInt(result.expected_revision),
    expected_participant_revision: toBigInt(result.expected_participant_revision),
  };
}
export async function rollEncounterStrike(
  encounter: string,
  request: import("../generated/atlas").RollEncounterStrikeRequest,
): Promise<import("../generated/atlas").EncounterStrikeRollView> {
  return atlasFetch(`/api/encounters/${encodeURIComponent(encounter)}/strike-rolls`, {
    method: "POST",
    body: jsonBody(request),
  });
}
