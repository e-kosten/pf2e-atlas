import type {
  SavedListItemNoteView,
  UpdateSavedListItemNoteRequest,
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
  RelationshipOccurrencesRequest,
  RelationshipOccurrencesPage,
  RecordDetailView,
  RecordDetailRequest,
  EmbeddedReadingRequest,
  EmbeddedReadingBatchView,
  RemoveSavedListItemRequest,
  ReorderEncounterParticipantRequest,
  ReorderSavedListItemRequest,
  SavedListCreateView,
  SearchResultsPage,
  SimilarityPartsRequest,
  SimilarityPartsView,
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
import {
  API_CONTRACT,
  API_CONTRACT_HEADER,
  checkResponseCompatibility,
  clearCompatibilityFailure,
  getCompatibilityFailure,
} from "./compatibility";
import { AtlasConnectionError } from "./availability";
import { AtlasApiError } from "./apiError";
import { toBigInt, type Wire } from "./wire";
import { convertSearchPage } from "./pageConversion";
import {
  convertEncounterDetail,
  convertConditionParameters,
  convertPersistentGroup,
} from "./encounterConversion";
import {
  convertRecordDetail,
  convertPreparedContentField,
  convertPreparedFieldBody,
} from "./presentationConversion";
export { AtlasApiError } from "./apiError";

const API_BASE = import.meta.env.VITE_ATLAS_API_BASE ?? "";

export async function verifyBackendCompatibility(): Promise<true> {
  await atlasFetch("/api/compatibility", { cache: "no-store" });
  clearCompatibilityFailure();
  return true;
}

export async function getReadiness(): Promise<AppReadinessView> {
  return atlasFetch("/api/readiness");
}

export async function discoverFilterEditor(
  request: DiscoverFilterEditorRequest,
  signal?: AbortSignal,
): Promise<FilterEditorView> {
  const editor = await atlasFetch<unknown>("/api/filters/editor", {
    intent: "read",
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
    intent: "read",
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
    intent: "read",
    method: "POST",
    body: jsonBody(request),
  });
  return normalizeSearchResultsPage(page);
}

export async function getSimilarityParts(
  request: SimilarityPartsRequest,
  signal?: AbortSignal,
): Promise<SimilarityPartsView> {
  return atlasFetch("/api/search/similarity/parts", {
    intent: "read",
    method: "POST",
    body: jsonBody(request),
    signal,
  });
}

export async function getRelationshipOccurrences(
  request: RelationshipOccurrencesRequest,
  signal?: AbortSignal,
): Promise<RelationshipOccurrencesPage> {
  const value = await atlasFetch<unknown>("/api/search/relationship-occurrences", {
    intent: "read",
    method: "POST",
    body: jsonBody(request),
    signal,
  });
  if (!isRecord(value) || !isRecord(value.page))
    throw new AtlasApiError(200, "Invalid relationship links response");
  return {
    ...value,
    page: convertSearchPage(
      value.page as Wire<import("../generated/atlas").SearchPageView>,
    ),
  } as RelationshipOccurrencesPage;
}

export async function getRecordDetail(
  recordKey: string,
  selection?: RecordDetailRequest,
  signal?: AbortSignal,
): Promise<RecordDetailView> {
  const detail = await (selection
    ? atlasFetch<Wire<RecordDetailView>>("/api/records/detail", {
        intent: "read",
        method: "POST",
        body: jsonBody(selection),
        signal,
      })
    : atlasFetch<Wire<RecordDetailView>>(
        `/api/records/${encodeURIComponent(recordKey)}`,
        { signal },
      ));
  return convertRecordDetail(detail);
}
export async function getEmbeddedReading(
  request: EmbeddedReadingRequest,
  signal?: AbortSignal,
): Promise<EmbeddedReadingBatchView> {
  const result = await atlasFetch<Wire<EmbeddedReadingBatchView>>(
    "/api/records/embedded-reading",
    {
      intent: "read",
      method: "POST",
      body: jsonBody(request),
      signal,
    },
  );
  return {
    ...result,
    entries: Array.isArray(result.entries)
      ? result.entries.map((entry) => ({
          ...entry,
          content: Array.isArray(entry.content)
            ? entry.content.map(convertPreparedContentField)
            : entry.content,
        }))
      : result.entries,
  };
}
export async function getEncounterReference(
  encounterRef: string,
  participantKey: string,
  request: import("../generated/atlas").EncounterReferenceRequest,
  signal?: AbortSignal,
): Promise<RecordDetailView> {
  const detail = await atlasFetch<Wire<RecordDetailView>>(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(participantKey)}/reference`,
    {
      signal,
      intent: "read",
      method: "POST",
      body: jsonBody(request),
    },
  );
  return convertRecordDetail(detail);
}

export async function getEncounters(): Promise<EncounterIndexView> {
  return atlasFetchEncounter("/api/encounters");
}

export async function getEncounterConditionDefinitions(
  encounterRef: string,
  participantKey: string,
): Promise<EncounterConditionCatalogView> {
  return atlasFetchEncounter(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(participantKey)}/condition-definitions`,
  );
}

export async function getEncounter(encounterRef: string): Promise<EncounterDetailView> {
  return convertEncounterDetail(
    await atlasFetch<Wire<EncounterDetailView>>(
      `/api/encounters/${encodeURIComponent(encounterRef)}`,
    ),
  );
}

export async function createEncounter(
  request: CreateEncounterRequest,
): Promise<EncounterCreateView> {
  return atlasFetchEncounter("/api/encounters", {
    intent: "action",
    method: "POST",
    body: jsonBody(request),
  });
}

export async function deleteEncounter(
  encounterRef: string,
): Promise<DeleteEncounterView> {
  return atlasFetch(`/api/encounters/${encodeURIComponent(encounterRef)}`, {
    intent: "action",
    method: "DELETE",
  });
}

export async function updateEncounter(
  request: UpdateEncounterRequest,
): Promise<EncounterMutationView> {
  return atlasFetch(`/api/encounters/${encodeURIComponent(request.encounter_key)}`, {
    intent: "action",
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
      intent: "action",
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
      intent: "action",
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
    { intent: "action", method: "POST", body: jsonBody(request) },
  );
}

export async function applyEncounterSpellAction(
  encounterRef: string,
  request: import("../generated/atlas").ApplyEncounterSpellActionRequest,
): Promise<import("../generated/atlas").EncounterMutationView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(request.participant_key)}/spells`,
    { intent: "action", method: "POST", body: jsonBody(request) },
  );
}

export async function updateEncounterParticipant(
  encounterRef: string,
  request: UpdateEncounterParticipantRequest,
): Promise<EncounterMutationView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(request.participant_key)}`,
    {
      intent: "action",
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
      intent: "action",
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
    { intent: "action", method: "DELETE" },
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
    intent: "action",
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
    intent: "action",
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
      intent: "action",
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
      intent: "action",
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
      intent: "action",
      method: "PATCH",
      body: jsonBody(request),
    },
  );
}

export async function previewEncounterCondition(
  encounterRef: string,
  participantKey: string,
  request:
    | AddEncounterParticipantConditionRequest
    | UpdateEncounterParticipantConditionRequest,
): Promise<import("../generated/atlas").ConditionEventPreviewView> {
  const suffix = "condition_id" in request ? `/${request.condition_id.toString()}` : "";
  return atlasFetchEncounter(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(participantKey)}/conditions${suffix}/preview`,
    { intent: "read", method: "POST", body: jsonBody(request) },
  );
}

function actionCheck(
  value: import("../generated/atlas").EncounterActionCheckView,
): import("../generated/atlas").EncounterActionCheckView {
  return {
    ...value,
    dc: toBigInt(value.dc),
    sources: value.sources.map((source) => ({
      ...source,
      condition_id: source.condition_id === null ? null : toBigInt(source.condition_id),
    })),
  };
}

export async function encounterCastingReminders(
  encounterRef: string,
  request: import("../generated/atlas").EncounterCastingRemindersRequest,
): Promise<import("../generated/atlas").EncounterCastingRemindersView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/casting-reminders`,
    {
      intent: "read",
      method: "POST",
      body: jsonBody(request),
    },
  );
}

export async function previewEncounterDrainedConsequence(
  encounterRef: string,
  request: import("../generated/atlas").ResolveDrainedConsequenceRequest,
): Promise<import("../generated/atlas").ConditionHpChangeView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/condition-consequences/preview`,
    { intent: "read", method: "POST", body: jsonBody(request) },
  );
}

export async function getEncounterConditionConsequences(
  encounterRef: string,
): Promise<import("../generated/atlas").PendingConditionConsequenceView[]> {
  const result = await atlasFetch<
    import("../generated/atlas").PendingConditionConsequenceView[]
  >(`/api/encounters/${encodeURIComponent(encounterRef)}/condition-consequences`);
  return result.map((value) => ({
    ...value,
    magnitude_increase: toBigInt(value.magnitude_increase),
  }));
}

export async function resolveEncounterDrainedConsequence(
  encounterRef: string,
  request: import("../generated/atlas").ResolveDrainedConsequenceRequest,
): Promise<EncounterMutationView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/condition-consequences/drained`,
    { intent: "action", method: "POST", body: jsonBody(request) },
  );
}

function persistentProtection(
  value: import("../generated/atlas").PersistentDamageProtectionView,
): import("../generated/atlas").PersistentDamageProtectionView {
  return {
    ...value,
    weakness: toBigInt(value.weakness),
    resistance: toBigInt(value.resistance),
  };
}
function persistentDamageResult(
  value: import("../generated/atlas").PersistentDamageResultView,
): import("../generated/atlas").PersistentDamageResultView {
  return {
    ...value,
    total: toBigInt(value.total),
    hardness_absorbed: toBigInt(value.hardness_absorbed),
    groups: value.groups.map(persistentDamageOutcome),
  };
}
function persistentDamageOutcome(
  group: import("../generated/atlas").PersistentDamageOutcomeView,
): import("../generated/atlas").PersistentDamageOutcomeView {
  return {
    ...group,
    source_condition_id: toBigInt(group.source_condition_id),
    amount: toBigInt(group.amount),
    ...(group.protection ? { protection: persistentProtection(group.protection) } : {}),
  };
}
function persistentRecoveryResult(
  value: import("../generated/atlas").PersistentRecoveryResultView,
): import("../generated/atlas").PersistentRecoveryResultView {
  return {
    ...value,
    dc: value.dc === null ? null : toBigInt(value.dc),
  };
}
export async function getEncounterPersistentReview(
  encounterRef: string,
  participantKey: string,
  boundaryKey: string | null = null,
): Promise<import("../generated/atlas").PersistentReviewView> {
  const value = await atlasFetchEncounter<
    import("../generated/atlas").PersistentReviewView
  >(
    `/api/encounters/${encodeURIComponent(encounterRef)}/persistent-review?participant_key=${encodeURIComponent(participantKey)}${boundaryKey ? `&boundary_key=${encodeURIComponent(boundaryKey)}` : ""}`,
  );
  return {
    ...value,
    round_number: toBigInt(value.round_number),
    episodes: value.episodes.map((episode) => ({
      ...episode,
      damage_resolution:
        episode.damage_resolution?.kind === "known"
          ? {
              ...episode.damage_resolution,
              protection: persistentProtection(episode.damage_resolution.protection),
            }
          : episode.damage_resolution,
    })),
    damage_result:
      value.damage_result === null ? null : persistentDamageResult(value.damage_result),
    recovery_results: value.recovery_results.map(persistentRecoveryResult),
    prepared_damage: value.prepared_damage.map((prepared) => ({
      ...prepared,
      group: persistentDamageOutcome(prepared.group),
    })),
    prepared_recovery: value.prepared_recovery.map(persistentRecoveryResult),
  };
}
export async function prepareEncounterPersistentDamage(
  encounterRef: string,
  request: import("../generated/atlas").PreparePersistentDamageRequest,
): Promise<import("../generated/atlas").PreparedPersistentDamageView> {
  const value = await atlasFetch<
    import("../generated/atlas").PreparedPersistentDamageView
  >(`/api/encounters/${encodeURIComponent(encounterRef)}/persistent-preparation`, {
    intent: "action",
    method: "POST",
    body: jsonBody(request),
  });
  return {
    ...value,
    damage: persistentDamageResult(value.damage),
    recovery: value.recovery ? persistentRecoveryResult(value.recovery) : null,
  };
}

export async function applyEncounterPersistentDamage(
  encounterRef: string,
  request: import("../generated/atlas").ApplyPersistentDamageRequest,
): Promise<import("../generated/atlas").PersistentDamageResultView> {
  return persistentDamageResult(
    await atlasFetch(
      `/api/encounters/${encodeURIComponent(encounterRef)}/persistent-damage`,
      { intent: "action", method: "POST", body: jsonBody(request) },
    ),
  );
}

export async function recoverEncounterPersistentDamage(
  encounterRef: string,
  request: import("../generated/atlas").RecoverPersistentDamageRequest,
): Promise<import("../generated/atlas").PersistentRecoveryResultView> {
  const result = await atlasFetch<
    import("../generated/atlas").PersistentRecoveryResultView
  >(`/api/encounters/${encodeURIComponent(encounterRef)}/persistent-recovery`, {
    intent: "action",
    method: "POST",
    body: jsonBody(request),
  });
  return persistentRecoveryResult(result);
}

function actionPreview(
  value: import("../generated/atlas").EncounterStrikeActionContextView,
): import("../generated/atlas").EncounterStrikeActionContextView {
  return {
    ...value,
    issues: value.issues.map((issue) => ({
      ...issue,
      source_condition_id:
        issue.source_condition_id === null ? null : toBigInt(issue.source_condition_id),
    })),
    disruption: value.disruption === null ? null : actionCheck(value.disruption),
  };
}

export async function previewEncounterStrikeContext(
  encounterRef: string,
  request: import("../generated/atlas").EncounterStrikeContextRequest,
): Promise<import("../generated/atlas").EncounterStrikeContextView> {
  const value = await atlasFetch<
    import("../generated/atlas").EncounterStrikeContextView
  >(`/api/encounters/${encodeURIComponent(encounterRef)}/strike-context`, {
    intent: "read",
    method: "POST",
    body: jsonBody(request),
  });
  return {
    ...value,
    options: {
      ...value.options,
      expected_revision: toBigInt(value.options.expected_revision),
      expected_participant_revision: toBigInt(
        value.options.expected_participant_revision,
      ),
    },
    action: actionPreview(value.action),
    targeting_check:
      value.targeting_check === null ? null : actionCheck(value.targeting_check),
  };
}

export async function getEncounterConditionLifecycleActions(
  encounterRef: string,
  participantKey: string,
  boundaryKey: string | null = null,
): Promise<import("../generated/atlas").ConditionLifecycleActionsView> {
  const value = await atlasFetch<
    import("../generated/atlas").ConditionLifecycleActionsView
  >(
    `/api/encounters/${encodeURIComponent(encounterRef)}/condition-lifecycle-actions?participant_key=${encodeURIComponent(participantKey)}${boundaryKey ? `&boundary_key=${encodeURIComponent(boundaryKey)}` : ""}`,
  );
  return {
    ...value,
    encounter_revision: toBigInt(value.encounter_revision),
    actions: value.actions.map((action) => {
      const condition_ids = action.condition_ids.map(toBigInt);
      const check =
        "check" in action
          ? {
              dc: action.check.dc === null ? null : toBigInt(action.check.dc),
              modifier:
                action.check.modifier === null ? null : toBigInt(action.check.modifier),
            }
          : undefined;
      switch (action.kind) {
        case "dying_recovery":
          return { ...action, condition_ids, check: check! };
        case "retch":
          return {
            ...action,
            condition_ids,
            check: check!,
            condition_id: toBigInt(action.condition_id),
            issues: action.issues.map((issue) => ({
              ...issue,
              source_condition_id:
                issue.source_condition_id === null
                  ? null
                  : toBigInt(issue.source_condition_id),
            })),
          };
        case "reminder":
          return {
            ...action,
            condition_ids,
            condition_id: toBigInt(action.condition_id),
          };
        case "confused_damage_recovery":
          return {
            ...action,
            condition_ids,
            check: check!,
            source_condition_ids: action.source_condition_ids.map(toBigInt),
          };
        case "stunned_actions_lost":
          return {
            ...action,
            condition_ids,
            condition_id: toBigInt(action.condition_id),
            actions_lost:
              action.actions_lost === null ? null : toBigInt(action.actions_lost),
          };
        default:
          return { ...action, condition_ids };
      }
    }),
  };
}
export async function applyEncounterConditionLifecycle(
  encounterRef: string,
  request: import("../generated/atlas").ApplyConditionLifecycleRequest,
): Promise<import("../generated/atlas").ConditionLifecycleResultView> {
  const value = await atlasFetch<
    import("../generated/atlas").ConditionLifecycleResultView
  >(`/api/encounters/${encodeURIComponent(encounterRef)}/condition-lifecycle`, {
    intent: "action",
    method: "POST",
    body: jsonBody(request),
  });
  return {
    ...value,
    check:
      value.check === null
        ? null
        : {
            ...value.check,
            dc: value.check.dc === null ? null : toBigInt(value.check.dc),
            modifier:
              value.check.modifier === null ? null : toBigInt(value.check.modifier),
            table_result:
              value.check.table_result === null
                ? null
                : toBigInt(value.check.table_result),
          },
  };
}

function hpLifecycle(
  value: import("../generated/atlas").PendingHpLifecycleView,
): import("../generated/atlas").PendingHpLifecycleView {
  return {
    ...value,
    before_hp: toBigInt(value.before_hp),
    after_hp: toBigInt(value.after_hp),
    context: {
      ...value.context,
      base_death_threshold:
        value.context.base_death_threshold === null
          ? null
          : toBigInt(value.context.base_death_threshold),
    },
  };
}
export async function getEncounterHpLifecycle(
  encounterRef: string,
): Promise<import("../generated/atlas").PendingHpLifecycleView[]> {
  const values = await atlasFetch<
    import("../generated/atlas").PendingHpLifecycleView[]
  >(`/api/encounters/${encodeURIComponent(encounterRef)}/hp-lifecycle`);
  return values.map(hpLifecycle);
}
export async function previewEncounterHpLifecycle(
  encounterRef: string,
  request: import("../generated/atlas").ResolveEncounterHpLifecycleRequest,
): Promise<import("../generated/atlas").PendingHpLifecycleView> {
  return hpLifecycle(
    await atlasFetch(
      `/api/encounters/${encodeURIComponent(encounterRef)}/hp-lifecycle/preview`,
      { intent: "read", method: "POST", body: jsonBody(request) },
    ),
  );
}
export async function resolveEncounterHpLifecycle(
  encounterRef: string,
  request: import("../generated/atlas").ResolveEncounterHpLifecycleRequest,
): Promise<EncounterMutationView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/hp-lifecycle/resolve`,
    { intent: "action", method: "POST", body: jsonBody(request) },
  );
}

export async function setEncounterLifeSettings(
  encounterRef: string,
  participantKey: string,
  request: import("../generated/atlas").SetEncounterLifeSettingsRequest,
): Promise<EncounterMutationView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(participantKey)}/life-settings`,
    { intent: "action", method: "PATCH", body: jsonBody(request) },
  );
}

export async function removeEncounterParticipantCondition(
  encounterRef: string,
  participantKey: string,
  conditionId: bigint,
  request: import("../generated/atlas").RemoveEncounterParticipantConditionRequest,
): Promise<EncounterMutationView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(participantKey)}/conditions/${conditionId.toString()}`,
    { intent: "action", method: "DELETE", body: jsonBody(request) },
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
      intent: "read",
      method: "POST",
      body: jsonBody(request),
    }),
  );
}

export async function createSavedList(
  request: CreateSavedListRequest,
): Promise<SavedListCreateView> {
  return atlasFetch("/api/lists", {
    intent: "action",
    method: "POST",
    body: jsonBody(request),
  });
}

export async function updateSavedList(
  request: UpdateSavedListRequest,
): Promise<SavedListUpdateView> {
  return atlasFetch(`/api/lists/${encodeURIComponent(request.list_key)}`, {
    intent: "action",
    method: "PATCH",
    body: jsonBody(request),
  });
}

export async function deleteSavedList(listRef: string): Promise<DeleteSavedListView> {
  return atlasFetch(`/api/lists/${encodeURIComponent(listRef)}`, {
    intent: "action",
    method: "DELETE",
  });
}

export async function addSavedListItem(
  request: AddSavedListItemRequest,
): Promise<SavedListItemMutationView> {
  return atlasFetch(`/api/lists/${encodeURIComponent(request.list_ref)}/items`, {
    intent: "action",
    method: "POST",
    body: jsonBody(request),
  });
}

export async function removeSavedListItem(
  request: RemoveSavedListItemRequest,
): Promise<SavedListItemMutationView> {
  return atlasFetch(`/api/lists/${encodeURIComponent(request.list_ref)}/items`, {
    intent: "action",
    method: "DELETE",
    body: jsonBody(request),
  });
}

export async function reorderSavedListItem(
  request: ReorderSavedListItemRequest,
): Promise<SavedListItemMutationView> {
  return atlasFetch(
    `/api/lists/${encodeURIComponent(request.list_ref)}/items/reorder`,
    { intent: "action", method: "POST", body: jsonBody(request) },
  );
}

// Non-GET callers must choose failure semantics. Actions include transient dice
// commands as well as saved mutations; intent never grants automatic retry.
export type AtlasRequestInit = Omit<RequestInit, "method"> &
  (
    | { method?: "GET"; intent?: "read" }
    | { method: "POST" | "PATCH" | "DELETE"; intent: "read" | "action" }
  );

async function atlasFetch<T>(path: string, init: AtlasRequestInit = {}): Promise<T> {
  const failure = getCompatibilityFailure();
  if (failure && path !== "/api/compatibility") throw failure;
  const headers = new Headers(init.headers);
  headers.set("Content-Type", "application/json");
  headers.set(API_CONTRACT_HEADER, API_CONTRACT);
  const { intent, ...request } = init;
  const write = intent === "action";
  let response: Response;
  try {
    response = await fetch(`${API_BASE}${path}`, { ...request, headers });
  } catch (cause) {
    if (
      init.signal?.aborted ||
      (cause instanceof DOMException && cause.name === "AbortError")
    )
      throw cause;
    throw new AtlasConnectionError(write, cause);
  }
  // Preserve genuine proxy/service failures while rejecting old successful APIs
  // before attempting to interpret their DTOs. Old servers return 404 here.
  if (
    response.headers.has(API_CONTRACT_HEADER) ||
    response.ok ||
    response.status === 409 ||
    (path === "/api/compatibility" && response.status === 404)
  ) {
    checkResponseCompatibility(response);
  }
  let text: string;
  try {
    text = await response.text();
  } catch (cause) {
    if (init.signal?.aborted) throw cause;
    throw new AtlasConnectionError(write, cause);
  }
  const parsed = parseJsonResponse(text);
  const payload = parsed.ok ? parsed.value : undefined;

  if (!response.ok) {
    const appError = isAppError(payload) ? payload : undefined;
    if (!appError && [502, 503, 504].includes(response.status))
      throw new AtlasConnectionError(write, new Error(`HTTP ${response.status}`));
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

async function atlasFetchEncounter<T>(
  path: string,
  init: AtlasRequestInit = {},
): Promise<T> {
  const payload = await atlasFetch<unknown>(path, init);
  if (!isRecord(payload)) throw new AtlasApiError(200, "Invalid encounter response");
  const result = { ...payload };
  const summary = (value: unknown) => {
    if (!isRecord(value)) throw new AtlasApiError(200, "Invalid encounter summary");
    return {
      ...value,
      round_number: toBigInt(value.round_number),
      round_duration_seconds: toBigInt(value.round_duration_seconds),
    };
  };
  const optionalIntegers = (value: Record<string, unknown>, fields: string[]) =>
    Object.fromEntries(
      fields
        .filter((field) => value[field] !== undefined)
        .map((field) => [field, toBigInt(value[field])]),
    );
  const duration = (value: unknown) => {
    if (!isRecord(value)) throw new AtlasApiError(200, "Invalid condition duration");
    return {
      ...value,
      amount: toBigInt(value.amount),
      ...optionalIntegers(value, ["remaining_rounds"]),
    };
  };
  const conditionParameters = (value: unknown) => {
    if (!isRecord(value)) throw new AtlasApiError(200, "Invalid condition parameters");
    return convertConditionParameters(
      value as Wire<
        import("../generated/atlas").EncounterParticipantConditionView["parameters"]
      >,
    );
  };
  const persistentGroup = (value: unknown) =>
    convertPersistentGroup(
      value as Wire<import("../generated/atlas").PersistentDamageGroupView>,
    );
  const condition = (value: unknown) => {
    if (!isRecord(value)) throw new AtlasApiError(200, "Invalid encounter condition");
    return {
      ...value,
      condition_id: toBigInt(value.condition_id),
      ...(value.parameters === undefined
        ? {}
        : { parameters: conditionParameters(value.parameters) }),
      ...optionalIntegers(value, ["value"]),
      ...(value.duration === undefined ? {} : { duration: duration(value.duration) }),
    };
  };
  const reminder = (value: unknown) => {
    if (isRecord(value) && value.kind === "condition_recovery") {
      if (!Array.isArray(value.condition_ids))
        throw new AtlasApiError(200, "Invalid condition recovery reminder");
      return { ...value, condition_ids: value.condition_ids.map(toBigInt) };
    }
    return condition(value);
  };
  if (isRecord(result.persistent_precedence)) {
    const source = (value: unknown) => {
      if (!isRecord(value))
        throw new AtlasApiError(200, "Invalid persistent source choice");
      return value.kind === "existing"
        ? { ...value, condition_id: toBigInt(value.condition_id) }
        : value;
    };
    const choice = result.persistent_precedence;
    result.persistent_precedence = {
      ...choice,
      sources: Array.isArray(choice.sources)
        ? choice.sources.map((option) => {
            if (!isRecord(option))
              throw new AtlasApiError(200, "Invalid persistent source option");
            return { ...option, source: source(option.source) };
          })
        : [],
    };
  }
  if (Array.isArray(result.damage_groups))
    result.damage_groups = result.damage_groups.map(persistentGroup);
  if (result.standard_persistent_recovery_dc !== undefined)
    result.standard_persistent_recovery_dc = toBigInt(
      result.standard_persistent_recovery_dc,
    );
  if (result.encounter_revision !== undefined)
    result.encounter_revision = toBigInt(result.encounter_revision);
  if (Array.isArray(result.consequences))
    result.consequences = result.consequences.map((value) => {
      if (!isRecord(value))
        throw new AtlasApiError(200, "Invalid condition consequence");
      return {
        ...value,
        ...optionalIntegers(value, ["amount", "magnitude_increase", "before", "after"]),
        ...(value.kind === "stunned_actions_lost"
          ? {
              stunned_after:
                value.stunned_after === null ? null : toBigInt(value.stunned_after),
            }
          : {}),
      };
    });
  if (Array.isArray(result.encounters))
    result.encounters = result.encounters.map(summary);
  if (result.encounter !== undefined) result.encounter = summary(result.encounter);
  if (Array.isArray(result.reminders)) {
    result.expected_revision = toBigInt(result.expected_revision);
    result.reminders = result.reminders.map(reminder);
  }
  if (result.expected_revision !== undefined)
    result.expected_revision = toBigInt(result.expected_revision);
  if (result.outgoing != null) {
    const turn = result.outgoing;
    if (
      !isRecord(turn) ||
      !Array.isArray(turn.reminders) ||
      !Array.isArray(turn.persistent_damage_groups)
    )
      throw new AtlasApiError(200, "Invalid outgoing turn");
    result.outgoing = {
      ...turn,
      round_number: toBigInt(turn.round_number),
      reminders: turn.reminders.map(reminder),
      persistent_damage_groups: turn.persistent_damage_groups.map(persistentGroup),
    };
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
    page: convertSearchPage(page as Wire<import("../generated/atlas").SearchPageView>),
    rows: Array.isArray(value.rows)
      ? value.rows.map((row) =>
          isRecord(row)
            ? {
                ...row,
                relationship_evidence: Array.isArray(row.relationship_evidence)
                  ? row.relationship_evidence.map((evidence) =>
                      isRecord(evidence) && isRecord(evidence.page)
                        ? {
                            ...evidence,
                            page: convertSearchPage(
                              evidence.page as Wire<
                                import("../generated/atlas").SearchPageView
                              >,
                            ),
                          }
                        : evidence,
                    )
                  : [],
              }
            : row,
        )
      : [],
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
    intent: "read",
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
    { intent: "read", method: "POST", body: jsonBody(request), signal },
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
  const result = await atlasFetch<Wire<import("../generated/atlas").TableRollView>>(
    "/api/rolls/table",
    {
      intent: "action",
      method: "POST",
      body: jsonBody(request),
    },
  );
  return {
    ...result,
    matches: Array.isArray(result.matches)
      ? result.matches.map(convertRecordDetail)
      : result.matches,
  };
}

export async function getEncounterStrikeOptions(
  encounter: string,
  request: import("../generated/atlas").EncounterStrikeOptionsRequest,
): Promise<import("../generated/atlas").EncounterStrikeOptionsView> {
  const result = await atlasFetch<
    import("../generated/atlas").EncounterStrikeOptionsView
  >(`/api/encounters/${encodeURIComponent(encounter)}/strike-options`, {
    intent: "read",
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
    intent: "action",
    method: "POST",
    body: jsonBody(request),
  });
}

export async function rollFlatCheck(
  request: import("../generated/atlas").RollFlatCheckRequest,
): Promise<import("../generated/atlas").FlatCheckRollView> {
  const result = await atlasFetch<Wire<import("../generated/atlas").FlatCheckRollView>>(
    "/api/rolls/flat-check",
    {
      intent: "action",
      method: "POST",
      body: jsonBody(request),
    },
  );
  return { ...result, dc: toBigInt(result.dc) };
}
export async function getParties(): Promise<
  import("../generated/atlas").PartyIndexView
> {
  return atlasFetch("/api/parties");
}
export async function createParty(
  request: import("../generated/atlas").CreatePartyRequest,
): Promise<import("../generated/atlas").PartyView> {
  return atlasFetch("/api/parties", {
    intent: "action",
    method: "POST",
    body: jsonBody(request),
  });
}
export async function updateParty(
  key: string,
  request: import("../generated/atlas").UpdatePartyRequest,
): Promise<import("../generated/atlas").PartyView> {
  return atlasFetch(`/api/parties/${encodeURIComponent(key)}`, {
    intent: "action",
    method: "PATCH",
    body: jsonBody(request),
  });
}
export async function deleteParty(
  key: string,
  expectedRevision: number,
): Promise<import("../generated/atlas").PartyMutationView> {
  return atlasFetch(`/api/parties/${encodeURIComponent(key)}`, {
    intent: "action",
    method: "DELETE",
    body: jsonBody({ expected_revision: expectedRevision }),
  });
}
export async function addEncounterParty(
  encounterRef: string,
  request: import("../generated/atlas").AddEncounterPartyRequest,
): Promise<EncounterMutationView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/party`,
    { intent: "action", method: "POST", body: jsonBody(request) },
  );
}

export async function getSavedListItemNote(
  listRef: string,
  recordKey: string,
): Promise<SavedListItemNoteView> {
  return atlasFetch(
    `/api/lists/${encodeURIComponent(listRef)}/items/${encodeURIComponent(recordKey)}/note`,
  );
}

export async function updateSavedListItemNote(
  request: UpdateSavedListItemNoteRequest,
): Promise<SavedListItemNoteView> {
  return atlasFetch(
    `/api/lists/${encodeURIComponent(request.list_ref)}/items/${encodeURIComponent(request.record_key)}/note`,
    {
      intent: "action",
      method: "PATCH",
      body: jsonBody(request),
    },
  );
}

export async function getTraitDetail(
  request: import("../generated/atlas").TraitDetailRequest,
  signal?: AbortSignal,
): Promise<import("../generated/atlas").TraitDetailView> {
  const result = await atlasFetch<Wire<import("../generated/atlas").TraitDetailView>>(
    "/api/traits/detail",
    {
      intent: "read",
      method: "POST",
      body: jsonBody(request),
      signal,
    },
  );
  if (result.trait_key !== request.trait_key)
    throw new Error("The trait response does not match this request. Retry.");
  return {
    ...result,
    content:
      result.content == null
        ? result.content
        : { ...result.content, body: convertPreparedFieldBody(result.content.body) },
  };
}
export async function getTraitRecords(
  request: import("../generated/atlas").TraitRecordsRequest,
  signal?: AbortSignal,
): Promise<import("../generated/atlas").TraitRecordsPage> {
  const result = await atlasFetch<Wire<import("../generated/atlas").TraitRecordsPage>>(
    "/api/traits/records",
    {
      intent: "read",
      method: "POST",
      body: jsonBody(request),
      signal,
    },
  );
  if (result.artifact_generation !== request.expected_artifact_generation)
    throw new AtlasApiError(409, "Reference collection changed. Reload trait.", {
      code: "artifact_generation_changed",
      message: "Reference collection changed. Reload trait.",
      recoverable_action: "reload_record",
    });
  if (result.trait_key !== request.trait_key || result.kind !== request.kind)
    throw new Error("The trait response does not match this request. Retry.");
  if (
    request.kind === "similar" &&
    request.search_profile != null &&
    result.search_profile !== request.search_profile
  )
    throw new Error(
      "The Similar response does not match the selected search model. Retry.",
    );
  return { ...result, page: convertSearchPage(result.page) };
}

export async function updateEncounterManualTracker(
  encounterRef: string,
  request: import("../generated/atlas").UpdateEncounterManualTrackerRequest,
): Promise<import("../generated/atlas").EncounterMutationView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(request.participant_key)}/manual-trackers`,
    { intent: "action", method: "POST", body: jsonBody(request) },
  );
}

export async function updateEncounterAbilityState(
  encounterRef: string,
  request: import("../generated/atlas").UpdateEncounterAbilityStateRequest,
): Promise<import("../generated/atlas").EncounterMutationView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(request.participant_key)}/ability-state`,
    { intent: "action", method: "POST", body: jsonBody(request) },
  );
}

export async function updateTurnResources(
  encounterRef: string,
  request: import("../generated/atlas").UpdateTurnResourcesRequest,
): Promise<import("../generated/atlas").EncounterMutationView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/participants/${encodeURIComponent(request.participant_key)}/turn-resources`,
    { intent: "action", method: "POST", body: jsonBody(request) },
  );
}

export async function setEncounterDifficulty(
  encounterRef: string,
  request: import("../generated/atlas").SetEncounterDifficultyRequest,
): Promise<EncounterMutationView> {
  return atlasFetch(`/api/encounters/${encodeURIComponent(encounterRef)}/difficulty`, {
    intent: "action",
    method: "PATCH",
    body: jsonBody(request),
  });
}
export async function previewEncounterDifficulty(
  encounterRef: string,
  request: import("../generated/atlas").PreviewEncounterDifficultyRequest,
): Promise<import("../generated/atlas").EncounterDifficultyView> {
  return atlasFetch(
    `/api/encounters/${encodeURIComponent(encounterRef)}/difficulty/preview`,
    { intent: "read", method: "POST", body: jsonBody(request) },
  );
}

export function rollContent(
  request: import("../generated/atlas").RollContentRequest,
): Promise<import("../generated/atlas").ContentRollResultView> {
  return atlasFetch("/api/rolls/content", {
    intent: "action",
    method: "POST",
    body: jsonBody(request),
  });
}

export async function previewEncounterRecovery(
  encounterRef: string,
  request: import("../generated/atlas").PreviewEncounterRecoveryRequest,
  signal?: AbortSignal,
): Promise<import("../generated/atlas").EncounterRecoveryPreviewView> {
  const result = await atlasFetch<
    Wire<import("../generated/atlas").EncounterRecoveryPreviewView>
  >(`/api/encounters/${encodeURIComponent(encounterRef)}/recovery/preview`, {
    intent: "read",
    method: "POST",
    body: jsonBody(request),
    signal,
  });
  return {
    ...result,
    encounter_revision: toBigInt(result.encounter_revision),
    participants: result.participants.map((p) => ({
      ...p,
      current_hp: p.current_hp === null ? null : toBigInt(p.current_hp),
      resulting_hp: p.resulting_hp === null ? null : toBigInt(p.resulting_hp),
      resulting_max_hp:
        p.resulting_max_hp === null ? null : toBigInt(p.resulting_max_hp),
      healing: p.healing === null ? null : toBigInt(p.healing),
    })),
  };
}
export function applyEncounterRecovery(
  encounterRef: string,
  request: import("../generated/atlas").ApplyEncounterRecoveryRequest,
): Promise<EncounterMutationView> {
  return atlasFetch(`/api/encounters/${encodeURIComponent(encounterRef)}/recovery`, {
    intent: "action",
    method: "POST",
    body: jsonBody(request),
  });
}
