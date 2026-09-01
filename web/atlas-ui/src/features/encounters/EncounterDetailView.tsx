import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button } from "antd";
import { Edit2 } from "lucide-react";
import { useState } from "react";
import {
  addEncounterParticipantCondition,
  deleteEncounter,
  getEncounter,
  getEncounterConditionDefinitions,
  removeEncounterParticipant,
  removeEncounterParticipantCondition,
  reorderEncounterParticipant,
  setEncounterTurn,
  updateEncounter,
  updateEncounterParticipant,
  updateEncounterParticipantCondition,
} from "../../api/atlasApi";
import type {
  AddEncounterParticipantConditionRequest,
  EncounterSummaryView,
  UpdateEncounterParticipantConditionRequest,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import { EncounterInspectorPane } from "./EncounterInspectorPane";
import { EditEncounterModal } from "./EncounterModals";
import { EncounterRosterPane } from "./EncounterRosterPane";
import { navigateToAtlasRoute, type AtlasRoute } from "../../app/routes";
import { confirmDangerAction } from "../../shared/ui/actions/confirmDangerAction";
import { WorkspaceLayout } from "../../shared/layout/WorkspaceLayout";
import { useRecordPreview } from "../../shared/records/useRecordPreview";

type EncounterDetailViewProps = {
  route: Extract<AtlasRoute, { kind: "encounter" }>;
};

const ENCOUNTER_WIDTH_SPECS = {
  filter: { defaultWidth: 360, minWidth: 300 },
  results: { defaultWidth: 620, minWidth: 420 },
  detail: { defaultWidth: 360, minWidth: 320 },
};

export function EncounterDetailView({ route }: EncounterDetailViewProps) {
  const queryClient = useQueryClient();
  const [selectedParticipantKey, setSelectedParticipantKey] = useState<string | null>(
    null,
  );
  const [editEncounterOpen, setEditEncounterOpen] = useState(false);
  const recordPreview = useRecordPreview();
  const encounter = useQuery({
    queryKey: ["encounter", route.slug],
    queryFn: () => getEncounter(route.slug),
  });
  const conditionDefinitions = useQuery({
    queryKey: ["encounter-condition-definitions"],
    queryFn: getEncounterConditionDefinitions,
  });
  const selected =
    encounter.data?.participants.find(
      (participant) => participant.participant_key === selectedParticipantKey,
    ) ??
    encounter.data?.participants.find(
      (participant) =>
        participant.participant_key === encounter.data?.current_turn_participant_key,
    ) ??
    encounter.data?.participants[0];
  const invalidateEncounter = async () => {
    await queryClient.invalidateQueries({ queryKey: ["encounter", route.slug] });
    await queryClient.invalidateQueries({ queryKey: ["encounters"] });
  };
  const updateParticipant = useMutation({
    mutationFn: (request: UpdateEncounterParticipantRequest) =>
      updateEncounterParticipant(route.slug, request),
    onSuccess: invalidateEncounter,
  });
  const updateEncounterMutation = useMutation({
    mutationFn: updateEncounter,
    onSuccess: invalidateEncounter,
  });
  const reorderParticipant = useMutation({
    mutationFn: (request: {
      participantKey: string;
      targetParticipantKey: string;
      placement: "before" | "after";
    }) =>
      reorderEncounterParticipant(route.slug, {
        participant_key: request.participantKey,
        target_participant_key: request.targetParticipantKey,
        placement: request.placement,
      }),
    onSuccess: invalidateEncounter,
  });
  const removeParticipant = useMutation({
    mutationFn: (participantKey: string) =>
      removeEncounterParticipant(route.slug, participantKey),
    onSuccess: invalidateEncounter,
  });
  const startTurn = useMutation({
    mutationFn: (participantKey: string | null) =>
      setEncounterTurn({
        encounter_ref: route.slug,
        ...(participantKey ? { participant_key: participantKey } : {}),
      }),
    onSuccess: invalidateEncounter,
  });
  const addCondition = useMutation({
    mutationFn: (request: AddEncounterParticipantConditionRequest) =>
      addEncounterParticipantCondition(route.slug, request),
    onSuccess: invalidateEncounter,
  });
  const updateCondition = useMutation({
    mutationFn: (request: {
      participantKey: string;
      condition: UpdateEncounterParticipantConditionRequest;
    }) =>
      updateEncounterParticipantCondition(
        route.slug,
        request.participantKey,
        request.condition,
      ),
    onSuccess: invalidateEncounter,
  });
  const removeCondition = useMutation({
    mutationFn: (request: { participantKey: string; conditionId: number }) =>
      removeEncounterParticipantCondition(
        route.slug,
        request.participantKey,
        request.conditionId,
      ),
    onSuccess: invalidateEncounter,
  });
  const encounterSummary = encounter.data?.encounter ?? null;
  const updateEncounterStatus = (status: EncounterSummaryView["status"]) => {
    if (!encounterSummary) {
      return;
    }
    updateEncounterMutation.mutate({
      encounter_key: encounterSummary.encounter_key,
      slug: encounterSummary.slug,
      name: encounterSummary.name,
      description: encounterSummary.description,
      note: encounter.data?.note,
      status,
    });
  };

  return (
    <>
      <section className="encounter-detail-header">
        <div>
          <h2>{encounterSummary?.name ?? route.slug}</h2>
          <p>
            {encounterSummary
              ? `${encounterSummary.status} - round ${
                  encounterSummary.round_number ?? 1
                } - ${encounterSummary.participant_count} participants`
              : "Loading encounter"}
          </p>
        </div>
        <Button
          aria-label="Edit encounter"
          icon={<Edit2 size={14} />}
          onClick={() => setEditEncounterOpen(true)}
        >
          Edit
        </Button>
      </section>
      <WorkspaceLayout
        filter={
          <EncounterRosterPane
            currentTurnParticipantKey={
              encounter.data?.current_turn_participant_key ?? null
            }
            loading={encounter.isLoading}
            onAddComplete={invalidateEncounter}
            onAdvanceTurn={() => startTurn.mutate(null)}
            onRemove={(participant) =>
              confirmDangerAction({
                title: `Remove ${participant.display_name}?`,
                content: "This removes the participant from the encounter.",
                okText: "Remove",
                onConfirm: () => {
                  removeParticipant.mutate(participant.participant_key);
                },
              })
            }
            onSelect={(participantKey) => {
              setSelectedParticipantKey(participantKey);
              recordPreview.close();
            }}
            onReorder={(participantKey, targetParticipantKey, placement) =>
              reorderParticipant.mutate({
                participantKey,
                targetParticipantKey,
                placement,
              })
            }
            onSetTurn={(participant) => startTurn.mutate(participant.participant_key)}
            onUpdate={(participant) => updateParticipant.mutate(participant)}
            participants={encounter.data?.participants ?? []}
            selectedParticipantKey={selected?.participant_key ?? null}
            slug={route.slug}
          />
        }
        results={
          <EncounterInspectorPane
            onAddCondition={(request) => addCondition.mutate(request)}
            onCloseRecordPreview={recordPreview.close}
            onOpenRecordFullPage={(recordKey) =>
              navigateToAtlasRoute({ kind: "record", recordKey })
            }
            onReference={recordPreview.open}
            onRemoveCondition={(participantKey, conditionId) =>
              removeCondition.mutate({ participantKey, conditionId })
            }
            onUpdate={(participant) => updateParticipant.mutate(participant)}
            onUpdateCondition={(participantKey, condition) =>
              updateCondition.mutate({ participantKey, condition })
            }
            participant={selected}
            participants={encounter.data?.participants ?? []}
            conditionDefinitions={conditionDefinitions.data?.conditions ?? []}
            previewAnchor={recordPreview.anchor}
            previewDetail={recordPreview.detail}
            previewLoading={recordPreview.loading}
            previewRecordKey={recordPreview.recordKey}
          />
        }
        labels={{ filter: "Roster", results: "Participant" }}
        selectedRecordKey={selected?.record_key ?? selected?.participant_key ?? null}
        widthSpecs={ENCOUNTER_WIDTH_SPECS}
      />
      {encounterSummary && (
        <EditEncounterModal
          encounter={encounterSummary}
          encounterNote={encounter.data?.note}
          open={editEncounterOpen}
          onArchive={() => {
            updateEncounterStatus("archived");
            setEditEncounterOpen(false);
          }}
          onCancel={() => setEditEncounterOpen(false)}
          onComplete={() => {
            updateEncounterStatus("complete");
            setEditEncounterOpen(false);
          }}
          onDelete={() =>
            confirmDangerAction({
              title: `Delete ${encounterSummary.name}?`,
              content: "This permanently deletes the encounter.",
              okText: "Delete",
              onConfirm: () => {
                deleteEncounter(route.slug).then(() =>
                  navigateToAtlasRoute({ kind: "encounters" }),
                );
              },
            })
          }
          onSave={(request) => {
            updateEncounterMutation.mutate(request);
            setEditEncounterOpen(false);
          }}
        />
      )}
    </>
  );
}
