import * as http from "../generated/http";
import type {
  SavedListItemNoteView,
  UpdateSavedListItemNoteRequest,
  AddEncounterManualParticipantRequest,
  AddEncounterParticipantConditionRequest,
  AddEncounterRecordParticipantRequest,
  AddSavedListItemRequest,
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
import { clearCompatibilityFailure } from "./compatibility";
import { isRecord } from "./atlasTransport";
export type { AtlasRequestInit } from "./atlasTransport";
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

export async function verifyBackendCompatibility(): Promise<true> {
  await http.handshake({ cache: "no-store" });
  clearCompatibilityFailure();
  return true;
}

export async function getReadiness(): Promise<AppReadinessView> {
  return http.readiness();
}

export async function discoverFilterEditor(
  request: DiscoverFilterEditorRequest,
  signal?: AbortSignal,
): Promise<FilterEditorView> {
  const editor = await http.discoverFilterEditor(toWire(request), {
    intent: "read",
    signal,
  });
  return editor;
}

export async function discoverFilterValues(
  request: DiscoverFilterValuesRequest,
  signal?: AbortSignal,
): Promise<FilterValueListView> {
  const values = await http.discoverFilterValues(toWire(request), {
    intent: "read",
    signal,
  });
  return normalizeFilterValueList(values);
}

export async function searchPage(
  request: SearchResultsRequest,
  signal?: AbortSignal,
): Promise<SearchResultsPage> {
  const page = await http.searchPage(toWire(request), { signal, intent: "read" });
  return normalizeSearchResultsPage(page);
}

export async function getSimilarityParts(
  request: SimilarityPartsRequest,
  signal?: AbortSignal,
): Promise<SimilarityPartsView> {
  return http.similarityParts(toWire(request), { intent: "read", signal });
}

export async function getRelationshipOccurrences(
  request: RelationshipOccurrencesRequest,
  signal?: AbortSignal,
): Promise<RelationshipOccurrencesPage> {
  const value = await http.relationshipOccurrences(toWire(request), {
    intent: "read",
    signal,
  });
  if (!isRecord(value) || !isRecord(value.page))
    throw new AtlasApiError(200, "Invalid relationship links response");
  return {
    ...value,
    page: convertSearchPage(value.page),
  };
}

export async function getRecordDetail(
  recordKey: string,
  selection?: RecordDetailRequest,
  signal?: AbortSignal,
): Promise<RecordDetailView> {
  const detail = await (selection
    ? http.recordDetailAt(toWire(selection), { intent: "read", signal })
    : http.getRecordDetail(recordKey, { signal }));
  return convertRecordDetail(detail);
}
export async function getEmbeddedReading(
  request: EmbeddedReadingRequest,
  signal?: AbortSignal,
): Promise<EmbeddedReadingBatchView> {
  const result = await http.embeddedReading(toWire(request), {
    intent: "read",
    signal,
  });
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
  const detail = await http.encounterReference(
    encounterRef,
    participantKey,
    toWire(request),
    {
      signal,
      intent: "read",
    },
  );
  return convertRecordDetail(detail);
}

export async function getEncounters(): Promise<EncounterIndexView> {
  return normalizeEncounterResponse<EncounterIndexView>(http.encounters());
}

export async function getEncounterConditionDefinitions(
  encounterRef: string,
  participantKey: string,
): Promise<EncounterConditionCatalogView> {
  return normalizeEncounterResponse<EncounterConditionCatalogView>(
    http.participantEncounterConditionDefinitions(encounterRef, participantKey),
  );
}

export async function getEncounter(encounterRef: string): Promise<EncounterDetailView> {
  return convertEncounterDetail(await http.encounter(encounterRef));
}

export async function createEncounter(
  request: CreateEncounterRequest,
): Promise<EncounterCreateView> {
  return normalizeEncounterResponse<EncounterCreateView>(
    http.createEncounter(toWire(request), { intent: "action" }),
  );
}

export async function deleteEncounter(
  encounterRef: string,
): Promise<DeleteEncounterView> {
  return http.deleteEncounter(encounterRef, { intent: "action" });
}

export async function updateEncounter(
  request: UpdateEncounterRequest,
): Promise<EncounterMutationView> {
  return http.updateEncounter(request.encounter_key, toWire(request), {
    intent: "action",
  });
}

export async function addEncounterRecordParticipant(
  request: AddEncounterRecordParticipantRequest,
): Promise<EncounterMutationView> {
  return http.addEncounterRecordParticipant(request.encounter_ref, toWire(request), {
    intent: "action",
  });
}

export async function addEncounterManualParticipant(
  request: AddEncounterManualParticipantRequest,
): Promise<EncounterMutationView> {
  return http.addEncounterManualParticipant(request.encounter_ref, toWire(request), {
    intent: "action",
  });
}

export async function applyEncounterHpAction(
  encounterRef: string,
  request: import("../generated/atlas").ApplyEncounterHpActionRequest,
): Promise<import("../generated/atlas").EncounterMutationView> {
  return http.applyEncounterHpAction(
    encounterRef,
    request.participant_key,
    toWire(request),
    { intent: "action" },
  );
}

export async function applyEncounterSpellAction(
  encounterRef: string,
  request: import("../generated/atlas").ApplyEncounterSpellActionRequest,
): Promise<import("../generated/atlas").EncounterMutationView> {
  return http.applyEncounterSpellAction(
    encounterRef,
    request.participant_key,
    toWire(request) as http.ApplyEncounterSpellActionRequest,
    { intent: "action" },
  );
}

export async function updateEncounterParticipant(
  encounterRef: string,
  request: UpdateEncounterParticipantRequest,
): Promise<EncounterMutationView> {
  return http.updateEncounterParticipant(
    encounterRef,
    request.participant_key,
    toWire(request),
    { intent: "action" },
  );
}

export async function reorderEncounterParticipant(
  encounterRef: string,
  request: ReorderEncounterParticipantRequest,
): Promise<EncounterMutationView> {
  return http.reorderEncounterParticipant(encounterRef, toWire(request), {
    intent: "action",
  });
}

export async function removeEncounterParticipant(
  encounterRef: string,
  participantKey: string,
): Promise<EncounterMutationView> {
  return http.removeEncounterParticipant(encounterRef, participantKey, {
    intent: "action",
  });
}

export async function getEncounterTurnReview(
  encounterRef: string,
): Promise<import("../generated/atlas").EncounterTurnReviewView> {
  return normalizeEncounterResponse<
    import("../generated/atlas").EncounterTurnReviewView
  >(http.encounterTurnReview(encounterRef));
}

export async function advanceEncounterTurn(
  encounterRef: string,
  request: import("../generated/atlas").AdvanceEncounterTurnRequest,
): Promise<EncounterMutationView> {
  return http.advanceEncounterTurn(encounterRef, toWire(request), { intent: "action" });
}

export async function getInitiativeOptions(
  encounterRef: string,
): Promise<import("../generated/atlas").InitiativeOptionsView> {
  const result = await http.initiativeOptions(encounterRef);
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
  return http.rollInitiative(encounterRef, toWire(request), { intent: "action" });
}

export async function setEncounterTurn(
  request: SetEncounterTurnRequest,
): Promise<EncounterMutationView> {
  return http.setEncounterTurn(request.encounter_ref, toWire(request), {
    intent: "action",
  });
}

export async function addEncounterParticipantCondition(
  encounterRef: string,
  request: AddEncounterParticipantConditionRequest,
): Promise<EncounterMutationView> {
  return http.addEncounterParticipantCondition(
    encounterRef,
    request.participant_key,
    toWire(request),
    { intent: "action" },
  );
}

export async function updateEncounterParticipantCondition(
  encounterRef: string,
  participantKey: string,
  request: UpdateEncounterParticipantConditionRequest,
): Promise<EncounterMutationView> {
  return http.updateEncounterParticipantCondition(
    encounterRef,
    participantKey,
    request.condition_id.toString(),
    toWire(request),
    { intent: "action" },
  );
}

export async function previewEncounterCondition(
  encounterRef: string,
  participantKey: string,
  request:
    | AddEncounterParticipantConditionRequest
    | UpdateEncounterParticipantConditionRequest,
): Promise<import("../generated/atlas").ConditionEventPreviewView> {
  return normalizeEncounterResponse<
    import("../generated/atlas").ConditionEventPreviewView
  >(
    "condition_id" in request
      ? http.previewEncounterConditionUpdate(
          encounterRef,
          participantKey,
          request.condition_id.toString(),
          toWire(request),
          { intent: "read" },
        )
      : http.previewEncounterConditionAdd(
          encounterRef,
          participantKey,
          toWire(request),
          { intent: "read" },
        ),
  );
}

function actionCheck(
  value: http.EncounterActionCheckView,
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
  return http.encounterCastingReminders(encounterRef, toWire(request), {
    intent: "read",
  });
}

export async function previewEncounterDrainedConsequence(
  encounterRef: string,
  request: import("../generated/atlas").ResolveDrainedConsequenceRequest,
): Promise<import("../generated/atlas").ConditionHpChangeView> {
  const value = await http.previewEncounterDrainedConsequence(
    encounterRef,
    toWire(request),
    {
      intent: "read",
    },
  );
  return {
    current_after: value.current_after === null ? null : toBigInt(value.current_after),
    current_before:
      value.current_before === null ? null : toBigInt(value.current_before),
    maximum_after: value.maximum_after === null ? null : toBigInt(value.maximum_after),
    maximum_before:
      value.maximum_before === null ? null : toBigInt(value.maximum_before),
  };
}

export async function getEncounterConditionConsequences(
  encounterRef: string,
): Promise<import("../generated/atlas").PendingConditionConsequenceView[]> {
  const result = await http.pendingEncounterConditionConsequences(encounterRef);
  return result.map((value) => ({
    ...value,
    magnitude_increase: toBigInt(value.magnitude_increase),
  }));
}

export async function resolveEncounterDrainedConsequence(
  encounterRef: string,
  request: import("../generated/atlas").ResolveDrainedConsequenceRequest,
): Promise<EncounterMutationView> {
  return http.resolveEncounterDrainedConsequence(encounterRef, toWire(request), {
    intent: "action",
  });
}

function persistentProtection(
  value: http.PersistentDamageProtectionView,
): import("../generated/atlas").PersistentDamageProtectionView {
  return {
    ...value,
    weakness: toBigInt(value.weakness),
    resistance: toBigInt(value.resistance),
  };
}
function persistentDamageResult(
  value: http.PersistentDamageResultView,
): import("../generated/atlas").PersistentDamageResultView {
  return {
    ...value,
    total: toBigInt(value.total),
    hardness_absorbed: toBigInt(value.hardness_absorbed),
    groups: value.groups.map(persistentDamageOutcome),
  };
}
function persistentDamageOutcome(
  group: http.PersistentDamageOutcomeView,
): import("../generated/atlas").PersistentDamageOutcomeView {
  const { protection, ...rest } = group;
  return {
    ...rest,
    source_condition_id: toBigInt(group.source_condition_id),
    amount: toBigInt(group.amount),
    ...(protection ? { protection: persistentProtection(protection) } : {}),
  };
}
function persistentRecoveryResult(
  value: http.PersistentRecoveryResultView,
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
  const value = await http.persistentDamageReview(encounterRef, {
    participant_key: participantKey,
    boundary_key: boundaryKey || undefined,
  });
  return {
    ...value,
    round_number: toBigInt(value.round_number),
    encounter_revision: toBigInt(value.encounter_revision),
    damage_groups: value.damage_groups.map(convertPersistentGroup),
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
  };
}
export async function prepareEncounterPersistentDamage(
  encounterRef: string,
  request: import("../generated/atlas").PreparePersistentDamageRequest,
): Promise<import("../generated/atlas").PreparedPersistentDamageView> {
  const value = await http.preparePersistentDamage(encounterRef, toWire(request), {
    intent: "action",
  });
  const draft = value.recovery_draft;
  return {
    ...value,
    draft_context: {
      ...value.draft_context,
      round_number: toBigInt(value.draft_context.round_number),
    },
    damage: persistentDamageResult(value.damage),
    recovery: value.recovery ? persistentRecoveryResult(value.recovery) : null,
    recovery_draft: draft
      ? {
          ...draft,
          context: {
            ...draft.context,
            round_number: toBigInt(draft.context.round_number),
          },
          source_condition_id: toBigInt(draft.source_condition_id),
          dc: toBigInt(draft.dc),
          dc_override: draft.dc_override === null ? null : toBigInt(draft.dc_override),
        }
      : null,
  };
}

export async function applyEncounterPersistentDamage(
  encounterRef: string,
  request: import("../generated/atlas").ApplyPersistentDamageRequest,
): Promise<import("../generated/atlas").PersistentDamageResultView> {
  return persistentDamageResult(
    await http.applyPersistentDamage(encounterRef, toWire(request), {
      intent: "action",
    }),
  );
}

export async function recoverEncounterPersistentDamage(
  encounterRef: string,
  request: import("../generated/atlas").RecoverPersistentDamageRequest,
): Promise<import("../generated/atlas").PersistentRecoveryResultView> {
  const result = await http.recoverPersistentDamage(encounterRef, toWire(request), {
    intent: "action",
  });
  return persistentRecoveryResult(result);
}

function actionPreview(
  value: http.EncounterStrikeActionContextView,
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
  const value = await http.previewEncounterStrikeContext(
    encounterRef,
    toWire(request),
    {
      intent: "read",
    },
  );
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
  const value = await http.conditionLifecycleActions(encounterRef, {
    participant_key: participantKey,
    boundary_key: boundaryKey || undefined,
  });
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
  const value = await http.applyConditionLifecycle(encounterRef, toWire(request), {
    intent: "action",
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

function hpContext(
  value: http.HpEventContextView,
): import("../generated/atlas").HpEventContextView {
  return {
    ...value,
    base_death_threshold:
      value.base_death_threshold == null
        ? value.base_death_threshold
        : toBigInt(value.base_death_threshold),
  };
}

function hpLifecycle(
  value: http.PendingHpLifecycleView,
): import("../generated/atlas").PendingHpLifecycleView {
  return {
    ...value,
    before_hp: toBigInt(value.before_hp),
    after_hp: toBigInt(value.after_hp),
    context: hpContext(value.context),
    knockout_options: value.knockout_options.map((option) => ({
      ...option,
      context: hpContext(option.context),
    })),
  };
}
export async function getEncounterHpLifecycle(
  encounterRef: string,
): Promise<import("../generated/atlas").PendingHpLifecycleView[]> {
  const values = await http.pendingEncounterHpLifecycle(encounterRef);
  return values.map(hpLifecycle);
}
export async function previewEncounterHpLifecycle(
  encounterRef: string,
  request: import("../generated/atlas").ResolveEncounterHpLifecycleRequest,
): Promise<import("../generated/atlas").PendingHpLifecycleView> {
  return hpLifecycle(
    await http.previewEncounterHpLifecycle(encounterRef, toWire(request), {
      intent: "read",
    }),
  );
}
export async function resolveEncounterHpLifecycle(
  encounterRef: string,
  request: import("../generated/atlas").ResolveEncounterHpLifecycleRequest,
): Promise<EncounterMutationView> {
  return http.resolveEncounterHpLifecycle(encounterRef, toWire(request), {
    intent: "action",
  });
}

export async function setEncounterLifeSettings(
  encounterRef: string,
  participantKey: string,
  request: import("../generated/atlas").SetEncounterLifeSettingsRequest,
): Promise<EncounterMutationView> {
  return http.setEncounterLifeSettings(encounterRef, participantKey, toWire(request), {
    intent: "action",
  });
}

export async function removeEncounterParticipantCondition(
  encounterRef: string,
  participantKey: string,
  conditionId: bigint,
  request: import("../generated/atlas").RemoveEncounterParticipantConditionRequest,
): Promise<EncounterMutationView> {
  return http.removeEncounterParticipantCondition(
    encounterRef,
    participantKey,
    conditionId.toString(),
    toWire(request),
    { intent: "action" },
  );
}

export async function getSavedLists(): Promise<SavedListIndexView> {
  return http.savedLists();
}

export async function getSavedList(listRef: string): Promise<SavedListDetailView> {
  return checkedSavedListDetail(await http.savedList(listRef));
}

export async function filterSavedList(
  request: FilterSavedListRequest,
): Promise<SavedListDetailView> {
  return checkedSavedListDetail(
    await http.filterSavedList(request.list_ref, toWire(request), { intent: "read" }),
  );
}

export async function createSavedList(
  request: CreateSavedListRequest,
): Promise<SavedListCreateView> {
  return http.createSavedList(toWire(request), { intent: "action" });
}

export async function updateSavedList(
  request: UpdateSavedListRequest,
): Promise<SavedListUpdateView> {
  return http.updateSavedList(request.list_key, toWire(request), { intent: "action" });
}

export async function deleteSavedList(listRef: string): Promise<DeleteSavedListView> {
  return http.deleteSavedList(listRef, { intent: "action" });
}

export async function addSavedListItem(
  request: AddSavedListItemRequest,
): Promise<SavedListItemMutationView> {
  return http.addSavedListItem(request.list_ref, toWire(request), { intent: "action" });
}

export async function removeSavedListItem(
  request: RemoveSavedListItemRequest,
): Promise<SavedListItemMutationView> {
  return http.removeSavedListItem(request.list_ref, toWire(request), {
    intent: "action",
  });
}

export async function reorderSavedListItem(
  request: ReorderSavedListItemRequest,
): Promise<SavedListItemMutationView> {
  return http.reorderSavedListItem(request.list_ref, toWire(request), {
    intent: "action",
  });
}

async function normalizeEncounterResponse<T>(response: Promise<Wire<T>>): Promise<T> {
  const payload: unknown = await response;
  if (!isRecord(payload)) throw new AtlasApiError(200, "Invalid encounter response");
  const result: Record<string, unknown> = { ...payload };
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
    if (isRecord(value) && value.kind === "tracker_elapsed") return value;
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
  for (const field of ["outgoing", "incoming"] as const) {
    const turn = result[field];
    if (turn == null) continue;
    if (
      !isRecord(turn) ||
      !Array.isArray(turn.reminders) ||
      !Array.isArray(turn.persistent_damage_groups)
    )
      throw new AtlasApiError(200, "Invalid turn boundary");
    result[field] = {
      ...turn,
      source_revision: toBigInt(turn.source_revision),
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

function toWire<T>(value: T): Wire<T> {
  return normalizeRequestNumbers(value) as Wire<T>;
}

function normalizeRequestNumbers(value: unknown): unknown {
  if (
    typeof value === "number" &&
    (!Number.isFinite(value) ||
      (Number.isInteger(value) && !Number.isSafeInteger(value)))
  ) {
    throw new AtlasApiError(0, "Request numeric field exceeds JSON safe numeric range");
  }
  if (typeof value === "bigint") {
    if (
      value > BigInt(Number.MAX_SAFE_INTEGER) ||
      value < BigInt(Number.MIN_SAFE_INTEGER)
    ) {
      throw new AtlasApiError(
        0,
        "Request numeric field exceeds JSON safe integer range",
      );
    }
    return Number(value);
  }
  if (Array.isArray(value)) return value.map(normalizeRequestNumbers);
  if (isRecord(value)) {
    return Object.fromEntries(
      Object.entries(value).map(([key, nested]) => [
        key,
        normalizeRequestNumbers(nested),
      ]),
    );
  }
  return value;
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

function checkedSavedListDetail(value: http.SavedListDetailView): SavedListDetailView {
  if (!isRecord(value) || !Array.isArray(value.items))
    throw new AtlasApiError(200, "Invalid saved-list response");
  return value;
}

export async function getRelatedRecords(
  request: import("../generated/atlas").RelatedRecordsRequest,
  signal?: AbortSignal,
): Promise<import("../generated/atlas").RelatedRecordsView> {
  return http.relatedRecords(toWire(request), { intent: "read", signal });
}
export async function discoverFilterCounts(
  request: import("../generated/atlas").DiscoverFilterCountsRequest,
  signal?: AbortSignal,
): Promise<import("../generated/atlas").FilterCountsView> {
  const result = await http.discoverFilterCounts(toWire(request), {
    intent: "read",
    signal,
  });
  if (result.counts.status === "unavailable")
    return { ...result, counts: result.counts };
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
  const result = await http.rollTable(toWire(request), { intent: "action" });
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
  const result = await http.encounterStrikeOptions(encounter, toWire(request), {
    intent: "read",
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
  return http.rollEncounterStrike(encounter, toWire(request), { intent: "action" });
}

export async function rollFlatCheck(
  request: import("../generated/atlas").RollFlatCheckRequest,
): Promise<import("../generated/atlas").FlatCheckRollView> {
  const result = await http.rollFlatCheck(toWire(request), { intent: "action" });
  return { ...result, dc: toBigInt(result.dc) };
}
export async function getParties(): Promise<
  import("../generated/atlas").PartyIndexView
> {
  return http.parties();
}
export async function createParty(
  request: import("../generated/atlas").CreatePartyRequest,
): Promise<import("../generated/atlas").PartyView> {
  return http.createParty(toWire(request), { intent: "action" });
}
export async function updateParty(
  key: string,
  request: import("../generated/atlas").UpdatePartyRequest,
): Promise<import("../generated/atlas").PartyView> {
  return http.updateParty(key, toWire(request), { intent: "action" });
}
export async function deleteParty(
  key: string,
  expectedRevision: number,
): Promise<import("../generated/atlas").PartyMutationView> {
  return http.deleteParty(key, toWire({ expected_revision: expectedRevision }), {
    intent: "action",
  });
}
export async function addEncounterParty(
  encounterRef: string,
  request: import("../generated/atlas").AddEncounterPartyRequest,
): Promise<EncounterMutationView> {
  return http.addEncounterParty(encounterRef, toWire(request), { intent: "action" });
}

export async function getSavedListItemNote(
  listRef: string,
  recordKey: string,
): Promise<SavedListItemNoteView> {
  return http.savedListItemNote(listRef, recordKey);
}

export async function updateSavedListItemNote(
  request: UpdateSavedListItemNoteRequest,
): Promise<SavedListItemNoteView> {
  return http.updateSavedListItemNote(
    request.list_ref,
    request.record_key,
    toWire(request),
    { intent: "action" },
  );
}

export async function getDefinitionDetail(
  request: import("../generated/atlas").DefinitionDetailRequest,
  signal?: AbortSignal,
): Promise<import("../generated/atlas").DefinitionDetailView> {
  const result = await http.definitionDetail(toWire(request), {
    intent: "read",
    signal,
  });
  if (result.definition_key !== request.definition_key)
    throw new AtlasApiError(200, "Definition response identity mismatch");
  return {
    ...result,
    content:
      result.content == null
        ? result.content
        : {
            ...result.content,
            body: convertPreparedFieldBody(result.content.body),
          },
  };
}

export async function getTraitDetail(
  request: import("../generated/atlas").TraitDetailRequest,
  signal?: AbortSignal,
): Promise<import("../generated/atlas").TraitDetailView> {
  const result = await http.traitDefinition(toWire(request), {
    intent: "read",
    signal,
  });
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
  const result = await http.traitRecords(toWire(request), { intent: "read", signal });
  if (result.artifact_generation !== request.expected_artifact_generation)
    throw new AtlasApiError(409, "Reference collection changed. Reload trait.", {
      code: "artifact_generation_changed",
      message: "Reference collection changed. Reload trait.",
      recoverable_action: "reload_record",
    });
  if (result.trait_key !== request.trait_key || result.kind !== request.kind)
    throw new Error("The trait response does not match this request. Retry.");
  return { ...result, page: convertSearchPage(result.page) };
}

export async function updateEncounterManualTracker(
  encounterRef: string,
  request: import("../generated/atlas").UpdateEncounterManualTrackerRequest,
): Promise<import("../generated/atlas").EncounterMutationView> {
  return http.updateEncounterManualTracker(
    encounterRef,
    request.participant_key,
    toWire(request),
    { intent: "action" },
  );
}

export async function updateEncounterAbilityState(
  encounterRef: string,
  request: import("../generated/atlas").UpdateEncounterAbilityStateRequest,
): Promise<import("../generated/atlas").EncounterMutationView> {
  return http.updateEncounterAbilityState(
    encounterRef,
    request.participant_key,
    toWire(request),
    { intent: "action" },
  );
}

export async function updateTurnResources(
  encounterRef: string,
  request: import("../generated/atlas").UpdateTurnResourcesRequest,
): Promise<import("../generated/atlas").EncounterMutationView> {
  return http.updateTurnResources(
    encounterRef,
    request.participant_key,
    toWire(request),
    { intent: "action" },
  );
}

export async function setEncounterDifficulty(
  encounterRef: string,
  request: import("../generated/atlas").SetEncounterDifficultyRequest,
): Promise<EncounterMutationView> {
  return http.setEncounterDifficulty(encounterRef, toWire(request), {
    intent: "action",
  });
}
export async function previewEncounterDifficulty(
  encounterRef: string,
  request: import("../generated/atlas").PreviewEncounterDifficultyRequest,
): Promise<import("../generated/atlas").EncounterDifficultyView> {
  return http.previewEncounterDifficulty(encounterRef, toWire(request), {
    intent: "read",
  });
}

export function rollContent(
  request: import("../generated/atlas").RollContentRequest,
): Promise<import("../generated/atlas").ContentRollResultView> {
  return http.rollContent(toWire(request), { intent: "action" });
}

export async function previewEncounterRecovery(
  encounterRef: string,
  request: import("../generated/atlas").PreviewEncounterRecoveryRequest,
  signal?: AbortSignal,
): Promise<import("../generated/atlas").EncounterRecoveryPreviewView> {
  const result = await http.previewEncounterRecovery(encounterRef, toWire(request), {
    intent: "read",
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
  return http.applyEncounterRecovery(encounterRef, toWire(request), {
    intent: "action",
  });
}
