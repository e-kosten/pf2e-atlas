import { useQuery } from "@tanstack/react-query";
import { Alert, Button } from "antd";
import { Edit2 } from "lucide-react";
import { useState } from "react";
import { getEncounter, getEncounterConditionDefinitions } from "../../api/atlasApi";
import type { EncounterSummaryView } from "../../generated/atlas";
import { useEncounterMutations } from "./useEncounterMutations";
import { EncounterInspectorPane } from "./EncounterInspectorPane";
import { EditEncounterModal } from "./EncounterModals";
import { EncounterRosterPane } from "./EncounterRosterPane";
import { navigateToAtlasRoute, type AtlasRoute } from "../../app/routes";
import { useConfirmDangerAction } from "../../shared/ui/actions/useConfirmDangerAction";
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
  return <EncounterDetail key={route.slug} route={route} />;
}
function EncounterDetail({ route }: EncounterDetailViewProps) {
  const confirmDangerAction = useConfirmDangerAction();
  const [selectedParticipantKey, setSelectedParticipantKey] = useState<string | null>(
    null,
  );
  const [editEncounterOpen, setEditEncounterOpen] = useState(false);
  const encounter = useQuery({
    queryKey: ["encounter", route.slug],
    queryFn: () => getEncounter(route.slug),
  });
  const conditionDefinitions = useQuery({
    queryKey: ["encounter-condition-definitions"],
    queryFn: getEncounterConditionDefinitions,
  });
  const mutations = useEncounterMutations(
    route.slug,
    encounter.data?.encounter.encounter_key,
  );
  const disabled =
    mutations.pending ||
    !!mutations.refreshError ||
    !!encounter.error ||
    !encounter.data;
  const selected =
    encounter.data?.participants.find(
      (participant) => participant.participant_key === selectedParticipantKey,
    ) ??
    encounter.data?.participants.find(
      (participant) =>
        participant.participant_key === encounter.data?.current_turn_participant_key,
    ) ??
    encounter.data?.participants[0];
  const recordPreview = useRecordPreview({
    encounterRef: encounter.data?.encounter.encounter_key ?? route.slug,
    identity: selected?.participant_key ?? "",
    generation: encounter.data?.artifact_generation ?? null,
    parentRecordKey: selected?.record_key,
    presentation: selected?.presentation,
  });
  const encounterSummary = encounter.data?.encounter ?? null;
  const updateEncounterStatus = async (status: EncounterSummaryView["status"]) => {
    if (!encounterSummary) {
      return;
    }
    await mutations.run({
      kind: "metadata",
      request: {
        encounter_key: encounterSummary.encounter_key,
        slug: encounterSummary.slug,
        name: encounterSummary.name,
        description: encounterSummary.description,
        note: encounter.data?.note,
        status,
      },
    });
    setEditEncounterOpen(false);
  };

  return (
    <div className="encounter-detail-view">
      <div>
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
            disabled={disabled}
            icon={<Edit2 size={14} />}
            onClick={() => setEditEncounterOpen(true)}
          >
            Edit
          </Button>
        </section>
        {encounter.error && !mutations.refreshError && (
          <Alert
            type="error"
            title="Encounter could not load"
            description={encounter.error.message}
            action={
              <Button
                loading={encounter.isFetching}
                onClick={() => void encounter.refetch()}
              >
                Retry encounter
              </Button>
            }
          />
        )}
        {mutations.refreshError && (
          <Alert
            type="error"
            title={mutations.refreshMessage}
            description={mutations.refreshError.message}
            action={
              <Button
                loading={mutations.pending}
                onClick={() => void mutations.retryRefresh()}
              >
                Retry encounter refresh
              </Button>
            }
          />
        )}
        {mutations.error && (
          <Alert
            type="error"
            title="Encounter action failed"
            description={mutations.errorMessage}
          />
        )}
      </div>
      <WorkspaceLayout
        filter={
          <EncounterRosterPane
            disabled={disabled}
            error={mutations.errorMessage}
            currentTurnParticipantKey={
              encounter.data?.current_turn_participant_key ?? null
            }
            loading={encounter.isLoading}
            onAddRecord={(request) => mutations.run({ kind: "add-record", request })}
            onAddManual={(request) => mutations.run({ kind: "add-manual", request })}
            onAdvanceTurn={() => void mutations.run({ kind: "turn" }).catch(() => {})}
            onRemove={(participant) =>
              confirmDangerAction({
                title: `Remove ${participant.display_name}?`,
                content: "This removes the participant from the encounter.",
                okText: "Remove",
                onConfirm: () =>
                  mutations.run({
                    kind: "remove",
                    participantKey: participant.participant_key,
                  }),
              })
            }
            onSelect={(participantKey) => {
              setSelectedParticipantKey(participantKey);
              recordPreview.close();
            }}
            onReorder={(participantKey, targetParticipantKey, placement) =>
              void mutations
                .run({
                  kind: "reorder",
                  request: {
                    participant_key: participantKey,
                    target_participant_key: targetParticipantKey,
                    placement,
                  },
                })
                .catch(() => {})
            }
            onSetTurn={(participant) =>
              void mutations
                .run({ kind: "turn", participantKey: participant.participant_key })
                .catch(() => {})
            }
            onUpdate={(request) => mutations.run({ kind: "participant", request })}
            participants={encounter.data?.participants ?? []}
            selectedParticipantKey={selected?.participant_key ?? null}
            slug={encounterSummary?.encounter_key ?? route.slug}
          />
        }
        results={
          <EncounterInspectorPane
            disabled={disabled}
            conditionsDisabled={
              conditionDefinitions.isPending || !!conditionDefinitions.error
            }
            conditionError={conditionDefinitions.error}
            onRetryConditions={() => void conditionDefinitions.refetch()}
            error={mutations.errorMessage}
            onAddCondition={(request) =>
              mutations.run({ kind: "add-condition", request })
            }
            onCloseRecordPreview={recordPreview.close}
            onOpenRecordFullPage={(recordKey) =>
              navigateToAtlasRoute({
                kind: "record",
                recordKey,
                selection: recordPreview.selection,
              })
            }
            onReference={recordPreview.open}
            onRemoveCondition={(participantKey, conditionId) =>
              mutations.run({ kind: "remove-condition", participantKey, conditionId })
            }
            onUpdate={(request) => mutations.run({ kind: "participant", request })}
            onUpdateCondition={(participantKey, condition) =>
              mutations.run({
                kind: "update-condition",
                participantKey,
                request: condition,
              })
            }
            participant={selected}
            participants={encounter.data?.participants ?? []}
            conditionDefinitions={conditionDefinitions.data?.conditions ?? []}
            previewAnchor={recordPreview.anchor}
            previewDetail={recordPreview.detail}
            previewSelection={recordPreview.selection}
            previewLoading={recordPreview.loading}
            previewError={recordPreview.error}
            onRetryPreview={recordPreview.retry}
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
          disabled={disabled}
          pending={mutations.pending}
          error={mutations.errorMessage}
          encounterNote={encounter.data?.note}
          open={editEncounterOpen}
          onArchive={() => updateEncounterStatus("archived")}
          onCancel={() => setEditEncounterOpen(false)}
          onComplete={() => updateEncounterStatus("complete")}
          onDelete={() =>
            confirmDangerAction({
              title: `Delete ${encounterSummary.name}?`,
              content: "This permanently deletes the encounter.",
              okText: "Delete",
              onConfirm: async () => {
                const result = await mutations.run({ kind: "delete" });
                if (result.active) navigateToAtlasRoute({ kind: "encounters" });
              },
            })
          }
          onSave={async (request) => {
            await mutations.run({ kind: "metadata", request });
            setEditEncounterOpen(false);
          }}
        />
      )}
    </div>
  );
}
