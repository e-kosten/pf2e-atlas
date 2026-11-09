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
  UpdateEncounterParticipantEdit,
  UpdateEncounterRequest,
  UpdateSavedListRequest,
} from "../generated/atlas";
import { clearCompatibilityFailure } from "./compatibility";
import { isRecord } from "./atlasTransport";
export type { AtlasRequestInit } from "./atlasTransport";
import { AtlasApiError } from "./apiError";
import {
  checkRequest,
  checkSearchPage,
  checkSearchResults,
  checkFilterCounts,
  checkFilterValues,
  checkEncounterDetail,
  checkTurnReview,
  checkEncounterResponse,
  checkPersistentGroup,
} from "./apiChecks";
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
  return http.discoverFilterEditor(checkRequest(request), {
    intent: "read",
    signal,
  });
}

export async function discoverFilterValues(
  request: DiscoverFilterValuesRequest,
  signal?: AbortSignal,
): Promise<FilterValueListView> {
  return checkFilterValues(
    await http.discoverFilterValues(checkRequest(request), {
      intent: "read",
      signal,
    }),
  );
}

export async function searchPage(
  request: SearchResultsRequest,
  signal?: AbortSignal,
): Promise<SearchResultsPage> {
  return checkSearchResults(
    await http.searchPage(checkRequest(request), { signal, intent: "read" }),
  );
}

export async function getSimilarityParts(
  request: SimilarityPartsRequest,
  signal?: AbortSignal,
): Promise<SimilarityPartsView> {
  return http.similarityParts(checkRequest(request), { intent: "read", signal });
}

export async function getRelationshipOccurrences(
  request: RelationshipOccurrencesRequest,
  signal?: AbortSignal,
): Promise<RelationshipOccurrencesPage> {
  const value = await http.relationshipOccurrences(checkRequest(request), {
    intent: "read",
    signal,
  });
  if (!isRecord(value) || !isRecord(value.page))
    throw new AtlasApiError(200, "Invalid relationship links response");
  checkSearchPage(value);
  return value;
}

export async function getRecordDetail(
  recordKey: string,
  selection?: RecordDetailRequest,
  signal?: AbortSignal,
): Promise<RecordDetailView> {
  const detail = await (selection
    ? http.recordDetailAt(checkRequest(selection), { intent: "read", signal })
    : http.getRecordDetail(recordKey, { signal }));
  return detail;
}
export async function getEmbeddedReading(
  request: EmbeddedReadingRequest,
  signal?: AbortSignal,
): Promise<EmbeddedReadingBatchView> {
  return http.embeddedReading(checkRequest(request), {
    intent: "read",
    signal,
  });
}
export async function getEncounterReference(
  encounterRef: string,
  participantKey: string,
  request: import("../generated/atlas").EncounterReferenceRequest,
  signal?: AbortSignal,
): Promise<RecordDetailView> {
  return http.encounterReference(encounterRef, participantKey, checkRequest(request), {
    signal,
    intent: "read",
  });
}

export async function getEncounters(): Promise<EncounterIndexView> {
  return checkEncounterResponse(await http.encounters());
}

export async function getEncounterConditionDefinitions(
  encounterRef: string,
  participantKey: string,
): Promise<EncounterConditionCatalogView> {
  return checkEncounterResponse(
    await http.participantEncounterConditionDefinitions(encounterRef, participantKey),
  );
}

export async function getEncounter(encounterRef: string): Promise<EncounterDetailView> {
  return checkEncounterDetail(await http.encounter(encounterRef));
}

export async function createEncounter(
  request: CreateEncounterRequest,
): Promise<EncounterCreateView> {
  return checkEncounterResponse(
    await http.createEncounter(checkRequest(request), { intent: "action" }),
  );
}

export async function deleteEncounter(
  encounterRef: string,
): Promise<DeleteEncounterView> {
  return http.deleteEncounter(encounterRef, { intent: "action" });
}

export async function updateEncounter(
  encounterRef: string,
  request: UpdateEncounterRequest,
): Promise<EncounterMutationView> {
  return http.updateEncounter(encounterRef, checkRequest(request), {
    intent: "action",
  });
}

export async function addEncounterRecordParticipant(
  encounterRef: string,
  request: AddEncounterRecordParticipantRequest,
): Promise<EncounterMutationView> {
  return http.addEncounterRecordParticipant(encounterRef, checkRequest(request), {
    intent: "action",
  });
}

export async function addEncounterManualParticipant(
  encounterRef: string,
  request: AddEncounterManualParticipantRequest,
): Promise<EncounterMutationView> {
  return http.addEncounterManualParticipant(encounterRef, checkRequest(request), {
    intent: "action",
  });
}

export async function applyEncounterHpAction(
  encounterRef: string,
  participantKey: string,
  request: import("../generated/atlas").ApplyEncounterHpActionRequest,
): Promise<import("../generated/atlas").EncounterMutationView> {
  return http.applyEncounterHpAction(
    encounterRef,
    participantKey,
    checkRequest(request),
    { intent: "action" },
  );
}

export async function applyEncounterSpellAction(
  encounterRef: string,
  participantKey: string,
  request: import("../generated/atlas").ApplyEncounterSpellActionRequest,
): Promise<import("../generated/atlas").EncounterMutationView> {
  return http.applyEncounterSpellAction(
    encounterRef,
    participantKey,
    checkRequest(request),
    { intent: "action" },
  );
}

export async function updateEncounterParticipant(
  encounterRef: string,
  participantKey: string,
  request: UpdateEncounterParticipantEdit,
): Promise<EncounterMutationView> {
  return http.updateEncounterParticipant(
    encounterRef,
    participantKey,
    checkRequest(request),
    { intent: "action" },
  );
}

export async function reorderEncounterParticipant(
  encounterRef: string,
  request: ReorderEncounterParticipantRequest,
): Promise<EncounterMutationView> {
  return http.reorderEncounterParticipant(encounterRef, checkRequest(request), {
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
  return checkTurnReview(await http.encounterTurnReview(encounterRef));
}

export async function advanceEncounterTurn(
  encounterRef: string,
  request: import("../generated/atlas").AdvanceEncounterTurnRequest,
): Promise<EncounterMutationView> {
  return http.advanceEncounterTurn(encounterRef, checkRequest(request), {
    intent: "action",
  });
}

export async function getInitiativeOptions(
  encounterRef: string,
): Promise<import("../generated/atlas").InitiativeOptionsView> {
  return http.initiativeOptions(encounterRef);
}
export function rollInitiative(
  encounterRef: string,
  request: import("../generated/atlas").RollInitiativeRequest,
): Promise<import("../generated/atlas").InitiativeRollView> {
  return http.rollInitiative(encounterRef, checkRequest(request), { intent: "action" });
}

export async function setEncounterTurn(
  encounterRef: string,
  request: SetEncounterTurnRequest,
): Promise<EncounterMutationView> {
  return http.setEncounterTurn(encounterRef, checkRequest(request), {
    intent: "action",
  });
}

export async function addEncounterParticipantCondition(
  encounterRef: string,
  participantKey: string,
  request: AddEncounterParticipantConditionRequest,
): Promise<EncounterMutationView> {
  return http.addEncounterParticipantCondition(
    encounterRef,
    participantKey,
    checkRequest(request),
    { intent: "action" },
  );
}

function conditionPath(conditionId: number): string {
  if (!Number.isSafeInteger(conditionId))
    throw new AtlasApiError(0, "Condition ID exceeds JSON safe integer range");
  return conditionId.toString();
}

export async function updateEncounterParticipantCondition(
  encounterRef: string,
  participantKey: string,
  conditionId: number,
  request: UpdateEncounterParticipantConditionRequest,
): Promise<EncounterMutationView> {
  return http.updateEncounterParticipantCondition(
    encounterRef,
    participantKey,
    conditionPath(conditionId),
    checkRequest(request),
    { intent: "action" },
  );
}

export async function previewEncounterConditionAdd(
  encounterRef: string,
  participantKey: string,
  request: AddEncounterParticipantConditionRequest,
): Promise<import("../generated/atlas").ConditionEventPreviewView> {
  return checkEncounterResponse(
    await http.previewEncounterConditionAdd(
      encounterRef,
      participantKey,
      checkRequest(request),
      {
        intent: "read",
      },
    ),
  );
}

export async function previewEncounterConditionUpdate(
  encounterRef: string,
  participantKey: string,
  conditionId: number,
  request: UpdateEncounterParticipantConditionRequest,
): Promise<import("../generated/atlas").ConditionEventPreviewView> {
  return checkEncounterResponse(
    await http.previewEncounterConditionUpdate(
      encounterRef,
      participantKey,
      conditionPath(conditionId),
      checkRequest(request),
      { intent: "read" },
    ),
  );
}

export async function encounterCastingReminders(
  encounterRef: string,
  request: import("../generated/atlas").EncounterCastingRemindersRequest,
): Promise<import("../generated/atlas").EncounterCastingRemindersView> {
  return http.encounterCastingReminders(encounterRef, checkRequest(request), {
    intent: "read",
  });
}

export async function previewEncounterDrainedConsequence(
  encounterRef: string,
  request: import("../generated/atlas").ResolveDrainedConsequenceRequest,
): Promise<import("../generated/atlas").ConditionHpChangeView> {
  return http.previewEncounterDrainedConsequence(encounterRef, checkRequest(request), {
    intent: "read",
  });
}

export async function getEncounterConditionConsequences(
  encounterRef: string,
): Promise<import("../generated/atlas").PendingConditionConsequenceView[]> {
  return http.pendingEncounterConditionConsequences(encounterRef);
}

export async function resolveEncounterDrainedConsequence(
  encounterRef: string,
  request: import("../generated/atlas").ResolveDrainedConsequenceRequest,
): Promise<EncounterMutationView> {
  return http.resolveEncounterDrainedConsequence(encounterRef, checkRequest(request), {
    intent: "action",
  });
}

export async function getEncounterPersistentReview(
  encounterRef: string,
  participantKey: string,
  boundaryKey: string | null = null,
): Promise<import("../generated/atlas").PersistentReviewView> {
  const result = await http.persistentDamageReview(encounterRef, {
    participant_key: participantKey,
    boundary_key: boundaryKey || undefined,
  });
  result.damage_groups.forEach(checkPersistentGroup);
  return result;
}
export async function prepareEncounterPersistentDamage(
  encounterRef: string,
  request: import("../generated/atlas").PreparePersistentDamageRequest,
): Promise<import("../generated/atlas").PreparedPersistentDamageView> {
  return http.preparePersistentDamage(encounterRef, checkRequest(request), {
    intent: "action",
  });
}

export async function applyEncounterPersistentDamage(
  encounterRef: string,
  request: import("../generated/atlas").ApplyPersistentDamageRequest,
): Promise<import("../generated/atlas").PersistentDamageResultView> {
  return http.applyPersistentDamage(encounterRef, checkRequest(request), {
    intent: "action",
  });
}

export async function recoverEncounterPersistentDamage(
  encounterRef: string,
  request: import("../generated/atlas").RecoverPersistentDamageRequest,
): Promise<import("../generated/atlas").PersistentRecoveryResultView> {
  return http.recoverPersistentDamage(encounterRef, checkRequest(request), {
    intent: "action",
  });
}

export async function previewEncounterStrikeContext(
  encounterRef: string,
  request: import("../generated/atlas").EncounterStrikeContextRequest,
): Promise<import("../generated/atlas").EncounterStrikeContextView> {
  return http.previewEncounterStrikeContext(encounterRef, checkRequest(request), {
    intent: "read",
  });
}

export async function getEncounterConditionLifecycleActions(
  encounterRef: string,
  participantKey: string,
  boundaryKey: string | null = null,
): Promise<import("../generated/atlas").ConditionLifecycleActionsView> {
  return http.conditionLifecycleActions(encounterRef, {
    participant_key: participantKey,
    boundary_key: boundaryKey || undefined,
  });
}
export async function applyEncounterConditionLifecycle(
  encounterRef: string,
  request: import("../generated/atlas").ApplyConditionLifecycleRequest,
): Promise<import("../generated/atlas").ConditionLifecycleResultView> {
  return http.applyConditionLifecycle(encounterRef, checkRequest(request), {
    intent: "action",
  });
}

export async function getEncounterHpLifecycle(
  encounterRef: string,
): Promise<import("../generated/atlas").PendingHpLifecycleView[]> {
  return http.pendingEncounterHpLifecycle(encounterRef);
}
export async function previewEncounterHpLifecycle(
  encounterRef: string,
  request: import("../generated/atlas").ResolveEncounterHpLifecycleRequest,
): Promise<import("../generated/atlas").PendingHpLifecycleView> {
  return http.previewEncounterHpLifecycle(encounterRef, checkRequest(request), {
    intent: "read",
  });
}
export async function resolveEncounterHpLifecycle(
  encounterRef: string,
  request: import("../generated/atlas").ResolveEncounterHpLifecycleRequest,
): Promise<EncounterMutationView> {
  return http.resolveEncounterHpLifecycle(encounterRef, checkRequest(request), {
    intent: "action",
  });
}

export async function setEncounterLifeSettings(
  encounterRef: string,
  participantKey: string,
  request: import("../generated/atlas").SetEncounterLifeSettingsRequest,
): Promise<EncounterMutationView> {
  return http.setEncounterLifeSettings(
    encounterRef,
    participantKey,
    checkRequest(request),
    {
      intent: "action",
    },
  );
}

export async function removeEncounterParticipantCondition(
  encounterRef: string,
  participantKey: string,
  conditionId: number,
  request: import("../generated/atlas").RemoveEncounterParticipantConditionRequest,
): Promise<EncounterMutationView> {
  return http.removeEncounterParticipantCondition(
    encounterRef,
    participantKey,
    conditionPath(conditionId),
    checkRequest(request),
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
  listRef: string,
  request: FilterSavedListRequest,
  signal?: AbortSignal,
): Promise<SavedListDetailView> {
  return checkedSavedListDetail(
    await http.filterSavedList(listRef, checkRequest(request), {
      signal,
      intent: "read",
    }),
  );
}

export async function createSavedList(
  request: CreateSavedListRequest,
): Promise<SavedListCreateView> {
  return http.createSavedList(checkRequest(request), { intent: "action" });
}

export async function updateSavedList(
  listRef: string,
  request: UpdateSavedListRequest,
): Promise<SavedListUpdateView> {
  return http.updateSavedList(listRef, checkRequest(request), {
    intent: "action",
  });
}

export async function deleteSavedList(listRef: string): Promise<DeleteSavedListView> {
  return http.deleteSavedList(listRef, { intent: "action" });
}

export async function addSavedListItem(
  listRef: string,
  request: AddSavedListItemRequest,
): Promise<SavedListItemMutationView> {
  return http.addSavedListItem(listRef, checkRequest(request), {
    intent: "action",
  });
}

export async function removeSavedListItem(
  listRef: string,
  request: RemoveSavedListItemRequest,
): Promise<SavedListItemMutationView> {
  return http.removeSavedListItem(listRef, checkRequest(request), {
    intent: "action",
  });
}

export async function reorderSavedListItem(
  listRef: string,
  request: ReorderSavedListItemRequest,
): Promise<SavedListItemMutationView> {
  return http.reorderSavedListItem(listRef, checkRequest(request), {
    intent: "action",
  });
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
  return http.relatedRecords(checkRequest(request), { intent: "read", signal });
}
export async function discoverFilterCounts(
  request: import("../generated/atlas").DiscoverFilterCountsRequest,
  signal?: AbortSignal,
): Promise<import("../generated/atlas").FilterCountsView> {
  return checkFilterCounts(
    await http.discoverFilterCounts(checkRequest(request), {
      intent: "read",
      signal,
    }),
  );
}

export async function rollTable(
  request: import("../generated/atlas").RollTableRequest,
): Promise<import("../generated/atlas").TableRollView> {
  return http.rollTable(checkRequest(request), { intent: "action" });
}

export async function getEncounterStrikeOptions(
  encounter: string,
  request: import("../generated/atlas").EncounterStrikeOptionsRequest,
): Promise<import("../generated/atlas").EncounterStrikeOptionsView> {
  return http.encounterStrikeOptions(encounter, checkRequest(request), {
    intent: "read",
  });
}
export async function rollEncounterStrike(
  encounter: string,
  request: import("../generated/atlas").RollEncounterStrikeRequest,
): Promise<import("../generated/atlas").EncounterStrikeRollView> {
  return http.rollEncounterStrike(encounter, checkRequest(request), {
    intent: "action",
  });
}

export async function rollFlatCheck(
  request: import("../generated/atlas").RollFlatCheckRequest,
): Promise<import("../generated/atlas").FlatCheckRollView> {
  return http.rollFlatCheck(checkRequest(request), { intent: "action" });
}
export async function getParties(): Promise<
  import("../generated/atlas").PartyIndexView
> {
  return http.parties();
}
export async function createParty(
  request: import("../generated/atlas").CreatePartyRequest,
): Promise<import("../generated/atlas").PartyView> {
  return http.createParty(checkRequest(request), { intent: "action" });
}
export async function updateParty(
  key: string,
  request: import("../generated/atlas").UpdatePartyRequest,
): Promise<import("../generated/atlas").PartyView> {
  return http.updateParty(key, checkRequest(request), { intent: "action" });
}
export async function deleteParty(
  key: string,
  expectedRevision: number,
): Promise<import("../generated/atlas").PartyMutationView> {
  return http.deleteParty(key, checkRequest({ expected_revision: expectedRevision }), {
    intent: "action",
  });
}
export async function addEncounterParty(
  encounterRef: string,
  request: import("../generated/atlas").AddEncounterPartyRequest,
): Promise<EncounterMutationView> {
  return http.addEncounterParty(encounterRef, checkRequest(request), {
    intent: "action",
  });
}

export async function getSavedListItemNote(
  listRef: string,
  recordKey: string,
): Promise<SavedListItemNoteView> {
  return http.savedListItemNote(listRef, recordKey);
}

export async function updateSavedListItemNote(
  listRef: string,
  recordKey: string,
  request: UpdateSavedListItemNoteRequest,
): Promise<SavedListItemNoteView> {
  return http.updateSavedListItemNote(listRef, recordKey, checkRequest(request), {
    intent: "action",
  });
}

export async function getDefinitionDetail(
  request: import("../generated/atlas").DefinitionDetailRequest,
  signal?: AbortSignal,
): Promise<import("../generated/atlas").DefinitionDetailView> {
  const result = await http.definitionDetail(checkRequest(request), {
    intent: "read",
    signal,
  });
  if (result.definition_key !== request.definition_key)
    throw new AtlasApiError(200, "Definition response identity mismatch");
  return result;
}

export async function getTraitDetail(
  request: import("../generated/atlas").TraitDetailRequest,
  signal?: AbortSignal,
): Promise<import("../generated/atlas").TraitDetailView> {
  const result = await http.traitDefinition(checkRequest(request), {
    intent: "read",
    signal,
  });
  if (result.trait_key !== request.trait_key)
    throw new Error("The trait response does not match this request. Retry.");
  return result;
}
export async function getTraitRecords(
  request: import("../generated/atlas").TraitRecordsRequest,
  signal?: AbortSignal,
): Promise<import("../generated/atlas").TraitRecordsPage> {
  const result = await http.traitRecords(checkRequest(request), {
    intent: "read",
    signal,
  });
  if (result.artifact_generation !== request.expected_artifact_generation)
    throw new AtlasApiError(409, "Reference collection changed. Reload trait.", {
      code: "artifact_generation_changed",
      message: "Reference collection changed. Reload trait.",
      recoverable_action: "reload_record",
    });
  if (result.trait_key !== request.trait_key || result.kind !== request.kind)
    throw new Error("The trait response does not match this request. Retry.");
  checkSearchPage(result);
  return result;
}

export async function updateEncounterManualTracker(
  encounterRef: string,
  participantKey: string,
  request: import("../generated/atlas").UpdateEncounterManualTrackerRequest,
): Promise<import("../generated/atlas").EncounterMutationView> {
  return http.updateEncounterManualTracker(
    encounterRef,
    participantKey,
    checkRequest(request),
    { intent: "action" },
  );
}

export async function updateEncounterAbilityState(
  encounterRef: string,
  participantKey: string,
  request: import("../generated/atlas").UpdateEncounterAbilityStateRequest,
): Promise<import("../generated/atlas").EncounterMutationView> {
  return http.updateEncounterAbilityState(
    encounterRef,
    participantKey,
    checkRequest(request),
    { intent: "action" },
  );
}

export async function updateTurnResources(
  encounterRef: string,
  participantKey: string,
  request: import("../generated/atlas").UpdateTurnResourcesRequest,
): Promise<import("../generated/atlas").EncounterMutationView> {
  return http.updateTurnResources(encounterRef, participantKey, checkRequest(request), {
    intent: "action",
  });
}

export async function setEncounterDifficulty(
  encounterRef: string,
  request: import("../generated/atlas").SetEncounterDifficultyRequest,
): Promise<EncounterMutationView> {
  return http.setEncounterDifficulty(encounterRef, checkRequest(request), {
    intent: "action",
  });
}
export async function previewEncounterDifficulty(
  encounterRef: string,
  request: import("../generated/atlas").PreviewEncounterDifficultyRequest,
): Promise<import("../generated/atlas").EncounterDifficultyView> {
  return http.previewEncounterDifficulty(encounterRef, checkRequest(request), {
    intent: "read",
  });
}

export function rollContent(
  request: import("../generated/atlas").RollContentRequest,
): Promise<import("../generated/atlas").ContentRollResultView> {
  return http.rollContent(checkRequest(request), { intent: "action" });
}

export async function previewEncounterRecovery(
  encounterRef: string,
  request: import("../generated/atlas").PreviewEncounterRecoveryRequest,
  signal?: AbortSignal,
): Promise<import("../generated/atlas").EncounterRecoveryPreviewView> {
  return http.previewEncounterRecovery(encounterRef, checkRequest(request), {
    intent: "read",
    signal,
  });
}
export function applyEncounterRecovery(
  encounterRef: string,
  request: import("../generated/atlas").ApplyEncounterRecoveryRequest,
): Promise<EncounterMutationView> {
  return http.applyEncounterRecovery(encounterRef, checkRequest(request), {
    intent: "action",
  });
}
