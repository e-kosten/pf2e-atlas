import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import {
  applyEncounterHpAction,
  addEncounterManualParticipant,
  addEncounterParticipantCondition,
  addEncounterRecordParticipant,
  AtlasApiError,
  deleteEncounter,
  getEncounter,
  removeEncounterParticipant,
  removeEncounterParticipantCondition,
  reorderEncounterParticipant,
  setEncounterTurn,
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
  | { kind: "hp"; request: ApplyEncounterHpActionRequest }
  | { kind: "metadata"; request: UpdateEncounterRequest }
  | { kind: "participant"; request: UpdateEncounterParticipantRequest }
  | { kind: "add-record"; request: AddEncounterRecordParticipantRequest }
  | { kind: "add-manual"; request: AddEncounterManualParticipantRequest }
  | { kind: "reorder"; request: ReorderEncounterParticipantRequest }
  | { kind: "remove"; participantKey: string }
  | { kind: "turn"; participantKey?: string }
  | { kind: "add-condition"; request: AddEncounterParticipantConditionRequest }
  | {
      kind: "update-condition";
      participantKey: string;
      request: UpdateEncounterParticipantConditionRequest;
    }
  | { kind: "remove-condition"; participantKey: string; conditionId: bigint }
  | { kind: "delete" };

export function useEncounterMutations(encounterRef: string, encounterKey?: string) {
  const queryClient = useQueryClient();
  const inFlight = useRef(false);
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
    mutationFn: (command: EncounterMutationCommand) => {
      switch (command.kind) {
        case "hp":
          return applyEncounterHpAction(currentRef, command.request);
        case "metadata":
          return updateEncounter(command.request);
        case "participant":
          return updateEncounterParticipant(currentRef, command.request);
        case "add-record":
          return addEncounterRecordParticipant({
            ...command.request,
            encounter_ref: currentRef,
          });
        case "add-manual":
          return addEncounterManualParticipant({
            ...command.request,
            encounter_ref: currentRef,
          });
        case "reorder":
          return reorderEncounterParticipant(currentRef, command.request);
        case "remove":
          return removeEncounterParticipant(currentRef, command.participantKey);
        case "turn":
          return setEncounterTurn({
            encounter_ref: currentRef,
            participant_key: command.participantKey,
          });
        case "add-condition":
          return addEncounterParticipantCondition(currentRef, command.request);
        case "update-condition":
          return updateEncounterParticipantCondition(
            currentRef,
            command.participantKey,
            command.request,
          );
        case "remove-condition":
          return removeEncounterParticipantCondition(
            currentRef,
            command.participantKey,
            command.conditionId,
          );
        case "delete":
          return deleteEncounter(currentRef);
      }
    },
    onSuccess: async (receipt, command) => {
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
    onError: async (error) => {
      if (
        error instanceof AtlasApiError &&
        error.appError?.code === "encounter_participant_conflict"
      ) {
        setRefreshOrigin("conflict");
        await attemptRefresh();
      }
    },
  });
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
  const conflict =
    mutation.error instanceof AtlasApiError &&
    mutation.error.appError?.code === "encounter_participant_conflict";
  return {
    run,
    pending: mutation.isPending || refreshing,
    error: mutation.error,
    errorMessage: conflict
      ? "Participant changed. Review the refreshed values before retrying your edit."
      : mutation.error?.message,
    refreshError,
    refreshMessage:
      refreshOrigin === "saved"
        ? "Saved, but the encounter could not refresh"
        : "Participant changed; current encounter could not load",
    retryRefresh,
  };
}
