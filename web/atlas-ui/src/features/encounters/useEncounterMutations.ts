import { useMutation, useQueryClient } from "@tanstack/react-query";
import { App } from "antd";
import { useEffect, useRef, useState } from "react";
import {
  setEncounterDifficulty,
  previewEncounterDifficulty,
  updateTurnResources,
  applyEncounterHpAction,
  applyEncounterConditionLifecycle,
  rollInitiative,
  applyEncounterSpellAction,
  advanceEncounterTurn,
  addEncounterManualParticipant,
  addEncounterParty,
  addEncounterParticipantCondition,
  addEncounterRecordParticipant,
  AtlasApiError,
  deleteEncounter,
  getEncounter,
  removeEncounterParticipant,
  removeEncounterParticipantCondition,
  reorderEncounterParticipant,
  setEncounterTurn,
  setEncounterLifeSettings,
  recoverEncounterPersistentDamage,
  applyEncounterPersistentDamage,
  resolveEncounterDrainedConsequence,
  resolveEncounterHpLifecycle,
  previewEncounterCondition,
  updateEncounter,
  updateEncounterParticipant,
  updateEncounterParticipantCondition,
} from "../../api/atlasApi";
import type {
  ApplyEncounterHpActionRequest,
  EncounterDetailView,
  AddEncounterManualParticipantRequest,
  AddEncounterParticipantConditionRequest,
  AddEncounterRecordParticipantRequest,
  ReorderEncounterParticipantRequest,
  UpdateEncounterParticipantConditionRequest,
  UpdateEncounterParticipantRequest,
  UpdateEncounterRequest,
} from "../../generated/atlas";

export type EncounterMutationCommand =
  | {
      kind: "turn-resources";
      request: Omit<
        import("../../generated/atlas").UpdateTurnResourcesRequest,
        "request_id"
      >;
    }
  | {
      kind: "condition-lifecycle";
      request: Omit<
        import("../../generated/atlas").ApplyConditionLifecycleRequest,
        "request_id"
      >;
    }
  | {
      kind: "resolve-hp-lifecycle";
      request: Omit<
        import("../../generated/atlas").ResolveEncounterHpLifecycleRequest,
        "request_id"
      >;
    }
  | {
      kind: "initiative";
      request: import("../../generated/atlas").RollInitiativeRequest;
    }
  | {
      kind: "spell";
      request: import("../../generated/atlas").ApplyEncounterSpellActionRequest;
    }
  | { kind: "hp"; request: Omit<ApplyEncounterHpActionRequest, "request_id"> }
  | {
      kind: "difficulty";
      request: import("../../generated/atlas").SetEncounterDifficultyRequest;
    }
  | { kind: "metadata"; request: UpdateEncounterRequest }
  | { kind: "participant"; request: UpdateEncounterParticipantRequest }
  | { kind: "add-record"; request: AddEncounterRecordParticipantRequest }
  | {
      kind: "add-party";
      request: import("../../generated/atlas").AddEncounterPartyRequest;
    }
  | { kind: "add-manual"; request: AddEncounterManualParticipantRequest }
  | { kind: "reorder"; request: ReorderEncounterParticipantRequest }
  | { kind: "remove"; participantKey: string }
  | { kind: "turn"; participantKey: string }
  | { kind: "advance"; expectedRevision: bigint }
  | {
      kind: "add-condition";
      request: Omit<AddEncounterParticipantConditionRequest, "request_id">;
    }
  | {
      kind: "update-condition";
      participantKey: string;
      request: Omit<UpdateEncounterParticipantConditionRequest, "request_id">;
    }
  | {
      kind: "remove-condition";
      participantKey: string;
      conditionId: bigint;
      expectedRevision: bigint;
    }
  | {
      kind: "life-settings";
      participantKey: string;
      request: Omit<
        import("../../generated/atlas").SetEncounterLifeSettingsRequest,
        "request_id"
      >;
    }
  | {
      kind: "persistent-recovery";
      request: Omit<
        import("../../generated/atlas").RecoverPersistentDamageRequest,
        "request_id"
      >;
    }
  | {
      kind: "resolve-drained";
      request: Omit<
        import("../../generated/atlas").ResolveDrainedConsequenceRequest,
        "request_id"
      >;
    }
  | {
      kind: "persistent-damage";
      request: Omit<
        import("../../generated/atlas").ApplyPersistentDamageRequest,
        "request_id"
      >;
    }
  | { kind: "delete" };

function isEncounterConflict(error: unknown): boolean {
  return (
    error instanceof AtlasApiError &&
    (error.appError?.code === "encounter_participant_conflict" ||
      error.appError?.code === "encounter_turn_conflict")
  );
}

export function useEncounterMutations(encounterRef: string, encounterKey?: string) {
  const { message } = App.useApp();
  const queryClient = useQueryClient();
  const inFlight = useRef(false);
  const eventRequests = useRef(new Map<string, { identity: string; id: string }>());
  const commandIdentity = (command: EncounterMutationCommand) =>
    JSON.stringify(command, (_key, value) =>
      typeof value === "bigint" ? value.toString() : value,
    );
  const requestSlot = (command: EncounterMutationCommand) => {
    switch (command.kind) {
      case "add-condition":
        return `${command.kind}:${command.request.participant_key}`;
      case "update-condition":
        return `${command.kind}:${command.participantKey}:${command.request.condition_id}`;
      case "remove-condition":
        return `${command.kind}:${command.participantKey}:${command.conditionId}`;
      case "turn-resources":
        return `${command.kind}:${command.request.participant_key}`;
      case "hp":
        return `${command.kind}:${command.request.participant_key}`;
      case "life-settings":
        return `${command.kind}:${command.participantKey}`;
      case "condition-lifecycle":
        return `${command.kind}:${command.request.participant_key}:${command.request.action.kind}`;
      case "persistent-damage":
        return `${command.kind}:${command.request.participant_key}`;
      case "persistent-recovery":
        return `${command.kind}:${command.request.participant_key}:${command.request.episode_id}`;
      case "resolve-hp-lifecycle":
      case "resolve-drained":
        return `${command.kind}:${command.request.consequence_key}`;
      default:
        return command.kind;
    }
  };
  const eventRequestId = (command: EncounterMutationCommand) => {
    const identity = commandIdentity(command);
    const slot = requestSlot(command);
    let retained = eventRequests.current.get(slot);
    if (retained?.identity !== identity) {
      retained = { identity, id: crypto.randomUUID() };
      eventRequests.current.set(slot, retained);
    }
    return retained.id;
  };
  const active = useRef(true);
  useEffect(() => {
    active.current = true;
    return () => {
      active.current = false;
    };
  }, []);
  const [refreshError, setRefreshError] = useState<Error | null>(null);
  const refreshSucceeded = useRef(true);
  const [refreshOrigin, setRefreshOrigin] = useState<"saved" | "conflict">("saved");
  const [refreshing, setRefreshing] = useState(false);
  const refreshKey = useRef<string | undefined>(undefined);
  const refreshedView = useRef<EncounterDetailView | undefined>(undefined);
  const currentRef = encounterKey ?? encounterRef;
  const refresh = async (key = refreshKey.current ?? currentRef) => {
    await queryClient.invalidateQueries({ queryKey: ["encounters"] });
    await queryClient.cancelQueries({
      queryKey: ["encounter", encounterRef],
      exact: true,
    });
    if (key !== encounterRef) {
      await queryClient.cancelQueries({ queryKey: ["encounter", key], exact: true });
    }
    const view = await queryClient.fetchQuery({
      queryKey: ["encounter", key],
      queryFn: () => getEncounter(key),
      staleTime: 0,
      retry: false,
    });
    queryClient.setQueryData(["encounter", encounterRef], view);
    setRefreshError(null);
    return view;
  };
  const attemptRefresh = async (encounterKey?: string) => {
    try {
      refreshedView.current = await refresh(encounterKey);
      refreshSucceeded.current = true;
      return refreshedView.current;
    } catch (error) {
      refreshSucceeded.current = false;
      refreshedView.current = undefined;
      setRefreshError(
        error instanceof Error ? error : new Error("Encounter refresh failed"),
      );
    }
  };
  const mutation = useMutation({
    retry: false,
    mutationFn: (
      command: EncounterMutationCommand,
    ): Promise<
      | import("../../generated/atlas").EncounterMutationView
      | import("../../generated/atlas").InitiativeRollView
      | import("../../generated/atlas").PersistentRecoveryResultView
      | import("../../generated/atlas").PersistentDamageResultView
      | import("../../generated/atlas").ConditionLifecycleResultView
    > => {
      switch (command.kind) {
        case "turn-resources":
          return updateTurnResources(currentRef, {
            ...command.request,
            request_id: eventRequestId(command),
          });
        case "initiative":
          return rollInitiative(currentRef, command.request);
        case "spell":
          return applyEncounterSpellAction(currentRef, command.request);
        case "hp":
          return applyEncounterHpAction(currentRef, {
            ...command.request,
            request_id: eventRequestId(command),
          });
        case "difficulty":
          return setEncounterDifficulty(currentRef, command.request);
        case "metadata":
          return updateEncounter(command.request);
        case "participant":
          return updateEncounterParticipant(currentRef, command.request);
        case "add-record":
          return addEncounterRecordParticipant({
            ...command.request,
            encounter_ref: currentRef,
          });
        case "add-party":
          return addEncounterParty(currentRef, command.request);
        case "add-manual":
          return addEncounterManualParticipant({
            ...command.request,
            encounter_ref: currentRef,
          });
        case "reorder":
          return reorderEncounterParticipant(currentRef, command.request);
        case "remove":
          return removeEncounterParticipant(currentRef, command.participantKey);
        case "advance":
          return advanceEncounterTurn(currentRef, {
            request_id: eventRequestId(command),
            expected_revision: command.expectedRevision,
          });
        case "turn":
          return setEncounterTurn({
            encounter_ref: currentRef,
            participant_key: command.participantKey,
          });
        case "add-condition":
          return addEncounterParticipantCondition(currentRef, {
            ...command.request,
            request_id: eventRequestId(command),
          });
        case "update-condition":
          return updateEncounterParticipantCondition(
            currentRef,
            command.participantKey,
            { ...command.request, request_id: eventRequestId(command) },
          );
        case "remove-condition":
          return removeEncounterParticipantCondition(
            currentRef,
            command.participantKey,
            command.conditionId,
            {
              request_id: eventRequestId(command),
              expected_encounter_revision: command.expectedRevision,
              intent: "remove_effect",
            },
          );
        case "condition-lifecycle":
          return applyEncounterConditionLifecycle(currentRef, {
            ...command.request,
            request_id: eventRequestId(command),
          });
        case "life-settings":
          return setEncounterLifeSettings(currentRef, command.participantKey, {
            ...command.request,
            request_id: eventRequestId(command),
          });
        case "persistent-damage":
          return applyEncounterPersistentDamage(currentRef, {
            ...command.request,
            request_id: eventRequestId(command),
          });
        case "persistent-recovery":
          return recoverEncounterPersistentDamage(currentRef, {
            ...command.request,
            request_id: eventRequestId(command),
          });
        case "resolve-hp-lifecycle":
          return resolveEncounterHpLifecycle(currentRef, {
            ...command.request,
            request_id: eventRequestId(command),
          });
        case "resolve-drained":
          return resolveEncounterDrainedConsequence(currentRef, {
            ...command.request,
            request_id: eventRequestId(command),
          });
        case "delete":
          return deleteEncounter(currentRef);
      }
    },
    onSuccess: async (receipt, command) => {
      if (
        eventRequests.current.get(requestSlot(command))?.identity ===
        commandIdentity(command)
      )
        eventRequests.current.delete(requestSlot(command));
      if (["persistent-damage", "persistent-recovery"].includes(command.kind))
        await queryClient.invalidateQueries({
          queryKey: ["persistent-review", currentRef],
        });
      await queryClient.invalidateQueries({ queryKey: ["hp-lifecycle", currentRef] });
      refreshKey.current = receipt.encounter_key;
      setRefreshOrigin("saved");
      refreshSucceeded.current = true;
      if (command.kind === "delete") {
        queryClient.removeQueries({
          queryKey: ["encounter", encounterRef],
          exact: true,
        });
        queryClient.removeQueries({
          queryKey: ["encounter", receipt.encounter_key],
          exact: true,
        });
        await queryClient.invalidateQueries({ queryKey: ["encounters"] });
      } else {
        await attemptRefresh(receipt.encounter_key);
      }
    },
    onError: async (error, command) => {
      const conflict = isEncounterConflict(error);
      if (conflict) {
        setRefreshOrigin("conflict");
        await attemptRefresh();
      }
      if (command.kind !== "spell" || !active.current) return;
      const action = command.request.action;
      // The editor owns its draft and displays conflicts inside its modal.
      if (action.kind === "set_entry_resources") return;
      if (conflict && refreshSucceeded.current) {
        const participant = refreshedView.current?.participants.find(
          (p) => p.participant_key === command.request.participant_key,
        );
        if (
          action.kind === "set_expended" &&
          participant?.spellcasting?.spells.some(
            (spell) =>
              spell.navigation.record_key === action.selection.record_key &&
              JSON.stringify(spell.navigation.embedded_path) ===
                JSON.stringify(action.selection.embedded_path) &&
              spell.expended === action.expended,
          )
        )
          return;
        void message.info("Encounter updated. Try again.", 4);
      } else {
        const verb =
          action.kind === "cast"
            ? "record spell use"
            : action.kind === "undo"
              ? "undo"
              : action.expended
                ? "expend"
                : "restore";
        void message.error(
          conflict
            ? "Couldn’t refresh the encounter. Try reloading."
            : `Couldn’t ${verb}. Try again.`,
          4,
        );
      }
    },
  });
  const previewDifficulty = async (
    request: import("../../generated/atlas").PreviewEncounterDifficultyRequest,
  ) => {
    try {
      return await previewEncounterDifficulty(currentRef, request);
    } catch (error) {
      if (isEncounterConflict(error)) {
        setRefreshOrigin("conflict");
        await attemptRefresh();
      }
      throw error;
    }
  };
  const previewCondition = async (
    participantKey: string,
    request: import("./useConditionEventPreview").ConditionEventInput,
  ) => {
    const command: EncounterMutationCommand =
      "condition_id" in request
        ? { kind: "update-condition", participantKey, request }
        : { kind: "add-condition", request };
    try {
      return await previewEncounterCondition(currentRef, participantKey, {
        ...request,
        request_id: eventRequestId(command),
      });
    } catch (error) {
      if (isEncounterConflict(error)) {
        setRefreshOrigin("conflict");
        await attemptRefresh();
      }
      throw error;
    }
  };
  const run = async (command: EncounterMutationCommand) => {
    if (inFlight.current || !refreshSucceeded.current) {
      throw new Error("Wait for the encounter to refresh before editing again.");
    }
    inFlight.current = true;
    try {
      const receipt = await mutation.mutateAsync(command);
      return {
        receipt,
        refreshed: refreshSucceeded.current,
        active: active.current,
        view: refreshedView.current,
      };
    } finally {
      inFlight.current = false;
    }
  };
  const retryRefresh = async () => {
    if (inFlight.current) return;
    inFlight.current = true;
    setRefreshing(true);
    try {
      return await attemptRefresh();
    } finally {
      inFlight.current = false;
      setRefreshing(false);
    }
  };
  const conflict = isEncounterConflict(mutation.error);
  return {
    run,
    previewCondition,
    previewDifficulty,
    pending: mutation.isPending || refreshing,
    error: mutation.error,
    errorKind: mutation.variables?.kind,
    errorMessage: conflict
      ? "Encounter changed. Review the refreshed values before retrying your edit."
      : mutation.error?.message,
    refreshError,
    refreshMessage:
      refreshOrigin === "saved"
        ? "Saved, but the encounter could not refresh"
        : "Participant changed; current encounter could not load",
    retryRefresh,
  };
}
