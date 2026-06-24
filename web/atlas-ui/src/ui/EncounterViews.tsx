import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button, Checkbox, Table } from "antd";
import type { ColumnsType } from "antd/es/table";
import { Edit2, Plus, Trash2 } from "lucide-react";
import { useState } from "react";
import {
  addEncounterParticipantCondition,
  deleteEncounter,
  getEncounter,
  getEncounters,
  getRecordDetail,
  removeEncounterParticipant,
  removeEncounterParticipantCondition,
  reorderEncounterParticipant,
  setEncounterTurn,
  updateEncounter,
  updateEncounterParticipant,
  updateEncounterParticipantCondition,
} from "../api/atlasApi";
import type {
  AddEncounterParticipantConditionRequest,
  EncounterSummaryView,
  UpdateEncounterParticipantConditionRequest,
  UpdateEncounterParticipantRequest,
} from "../generated/atlas";
import { EncounterInspectorPane } from "./encounter/EncounterInspectorPane";
import {
  CreateEncounterModal,
  EditEncounterForm,
  EditEncounterModal,
} from "./encounter/EncounterModals";
import { EncounterRosterPane } from "./encounter/EncounterRosterPane";
import { EncounterTurnPane } from "./encounter/EncounterTurnPane";
import { encounterPath, navigateToAtlasRoute, type AtlasRoute } from "./routes";
import { WorkspaceLayout } from "./WorkspaceLayout";

type EncounterIndexViewProps = {
  route: Extract<AtlasRoute, { kind: "encounters" }>;
};

type EncounterDetailViewProps = {
  route: Extract<AtlasRoute, { kind: "encounter" }>;
};

type EncounterEditViewProps = {
  route: Extract<AtlasRoute, { kind: "encounterEdit" }>;
};

const ENCOUNTER_WIDTH_SPECS = {
  filter: { defaultWidth: 360, minWidth: 300 },
  results: { defaultWidth: 620, minWidth: 420 },
  detail: { defaultWidth: 360, minWidth: 320 },
};

export function EncounterIndexView(_props: EncounterIndexViewProps) {
  const queryClient = useQueryClient();
  const [createOpen, setCreateOpen] = useState(false);
  const [showArchived, setShowArchived] = useState(false);
  const encounters = useQuery({ queryKey: ["encounters"], queryFn: getEncounters });
  const visible = (encounters.data?.encounters ?? []).filter(
    (encounter) => showArchived || encounter.status !== "archived",
  );
  const deleteMutation = useMutation({
    mutationFn: deleteEncounter,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["encounters"] }),
  });

  return (
    <main className="encounter-index-view">
      <section className="list-index-view__toolbar">
        <div>
          <h2>Encounters</h2>
          <p>{visible.length} active encounters</p>
        </div>
        <div className="encounter-actions">
          <Checkbox
            checked={showArchived}
            onChange={(event) => setShowArchived(event.target.checked)}
          >
            Archived
          </Checkbox>
          <Button
            icon={<Plus size={16} />}
            onClick={() => setCreateOpen(true)}
            type="primary"
          >
            New Encounter
          </Button>
        </div>
      </section>
      <section className="list-index-view__table">
        <Table
          columns={encounterColumns(
            (encounter) =>
              navigateToAtlasRoute({ kind: "encounterEdit", slug: encounter.slug }),
            (encounter) => {
              if (confirm(`Delete ${encounter.name}?`)) {
                deleteMutation.mutate(encounter.slug);
              }
            },
          )}
          dataSource={visible}
          loading={encounters.isLoading || encounters.isFetching}
          locale={{ emptyText: "No encounters" }}
          pagination={false}
          onRow={(encounter) => ({
            className: "encounter-index-row",
            tabIndex: 0,
            onClick: () =>
              navigateToAtlasRoute({ kind: "encounter", slug: encounter.slug }),
            onKeyDown: (event) => {
              if (event.key === "Enter") {
                navigateToAtlasRoute({ kind: "encounter", slug: encounter.slug });
              }
            },
          })}
          rowKey={(encounter) => encounter.encounter_key}
          size="middle"
        />
      </section>
      <CreateEncounterModal
        open={createOpen}
        onCancel={() => setCreateOpen(false)}
        onCreated={(slug) => {
          setCreateOpen(false);
          navigateToAtlasRoute({ kind: "encounter", slug });
        }}
      />
    </main>
  );
}

export function EncounterDetailView({ route }: EncounterDetailViewProps) {
  const queryClient = useQueryClient();
  const [selectedParticipantKey, setSelectedParticipantKey] = useState<string | null>(
    null,
  );
  const [editEncounterOpen, setEditEncounterOpen] = useState(false);
  const [previewRecordKey, setPreviewRecordKey] = useState<string | null>(null);
  const encounter = useQuery({
    queryKey: ["encounter", route.slug],
    queryFn: () => getEncounter(route.slug),
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
  const detail = useQuery({
    queryKey: ["encounter-record-detail", selected?.record_key],
    enabled: Boolean(selected?.record_key && selected.status === "active"),
    queryFn: () => getRecordDetail(selected?.record_key ?? ""),
  });
  const referencePreview = useQuery({
    queryKey: ["encounter-reference-preview", previewRecordKey],
    enabled: previewRecordKey !== null,
    queryFn: () => getRecordDetail(previewRecordKey!),
  });
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
    mutationFn: (request: { participantKey: string; conditionId: bigint }) =>
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
            onRemove={(participant) => {
              if (confirm(`Remove ${participant.display_name}?`)) {
                removeParticipant.mutate(participant.participant_key);
              }
            }}
            onSelect={(participantKey) => {
              setSelectedParticipantKey(participantKey);
              setPreviewRecordKey(null);
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
            detailLoading={detail.isLoading || detail.isFetching}
            onCloseReferencePreview={() => setPreviewRecordKey(null)}
            onOpenReferenceFullPage={(recordKey) =>
              navigateToAtlasRoute({ kind: "record", recordKey })
            }
            onReference={setPreviewRecordKey}
            onUpdate={(participant) => updateParticipant.mutate(participant)}
            participant={selected}
            previewDetail={referencePreview.data}
            previewLoading={referencePreview.isLoading || referencePreview.isFetching}
            previewRecordKey={previewRecordKey}
            recordDetail={detail.data}
          />
        }
        detail={
          <EncounterTurnPane
            current={selected ?? null}
            participants={encounter.data?.participants ?? []}
            onAddCondition={(request) => addCondition.mutate(request)}
            onRemoveCondition={(participantKey, conditionId) =>
              removeCondition.mutate({ participantKey, conditionId })
            }
            onUpdateCondition={(participantKey, condition) =>
              updateCondition.mutate({ participantKey, condition })
            }
            onUpdate={(participant) => updateParticipant.mutate(participant)}
          />
        }
        labels={{ filter: "Roster", results: "Record", detail: "Selected" }}
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
          onDelete={() => {
            if (confirm(`Delete ${encounterSummary.name}?`)) {
              deleteEncounter(route.slug).then(() =>
                navigateToAtlasRoute({ kind: "encounters" }),
              );
            }
          }}
          onSave={(request) => {
            updateEncounterMutation.mutate(request);
            setEditEncounterOpen(false);
          }}
        />
      )}
    </>
  );
}

export function EncounterEditView({ route }: EncounterEditViewProps) {
  const queryClient = useQueryClient();
  const encounter = useQuery({
    queryKey: ["encounter", route.slug],
    queryFn: () => getEncounter(route.slug),
  });
  const updateEncounterMutation = useMutation({
    mutationFn: updateEncounter,
    onSuccess: async (view) => {
      await queryClient.invalidateQueries({ queryKey: ["encounter", route.slug] });
      await queryClient.invalidateQueries({ queryKey: ["encounters"] });
      navigateToAtlasRoute({ kind: "encounter", slug: view.encounter.slug });
    },
  });
  const deleteMutation = useMutation({
    mutationFn: deleteEncounter,
    onSuccess: () => navigateToAtlasRoute({ kind: "encounters" }),
  });

  return (
    <main className="encounter-edit-view">
      <section className="encounter-edit-view__header">
        <div>
          <h2>Edit Encounter</h2>
          <p>{encounter.data?.encounter.name ?? route.slug}</p>
        </div>
        <Button
          onClick={() => navigateToAtlasRoute({ kind: "encounter", slug: route.slug })}
        >
          Done
        </Button>
      </section>
      {encounter.isLoading || !encounter.data ? (
        <div className="detail-empty">Loading encounter...</div>
      ) : (
        <section className="encounter-edit-view__content">
          <EditEncounterForm
            encounter={encounter.data.encounter}
            encounterNote={encounter.data.note}
            onSave={(request) => updateEncounterMutation.mutate(request)}
          />
          <div className="encounter-edit-view__metadata">
            <span>Created {encounter.data.encounter.created_at}</span>
            <span>Updated {encounter.data.encounter.updated_at}</span>
          </div>
          <Button
            danger
            onClick={() => {
              if (confirm(`Delete ${encounter.data.encounter.name}?`)) {
                deleteMutation.mutate(route.slug);
              }
            }}
          >
            Delete encounter
          </Button>
        </section>
      )}
    </main>
  );
}

function encounterColumns(
  onEdit: (encounter: EncounterSummaryView) => void,
  onDelete: (encounter: EncounterSummaryView) => void,
): ColumnsType<EncounterSummaryView> {
  return [
    {
      title: "Name",
      dataIndex: "name",
      render: (_value, encounter) => (
        <a href={encounterPath(encounter.slug)}>{encounter.name}</a>
      ),
    },
    { title: "Status", dataIndex: "status", width: 120 },
    { title: "Round", dataIndex: "round_number", width: 90 },
    { title: "Participants", dataIndex: "participant_count", width: 120 },
    { title: "Updated", dataIndex: "updated_at", width: 220 },
    {
      title: "",
      width: 112,
      render: (_value, encounter) => (
        <span className="encounter-row-actions">
          <Button
            aria-label={`Edit ${encounter.name}`}
            icon={<Edit2 size={14} />}
            size="small"
            onClick={(event) => {
              event.stopPropagation();
              onEdit(encounter);
            }}
          />
          <Button
            danger
            aria-label={`Delete ${encounter.name}`}
            icon={<Trash2 size={14} />}
            size="small"
            onClick={(event) => {
              event.stopPropagation();
              onDelete(encounter);
            }}
          />
        </span>
      ),
    },
  ];
}
