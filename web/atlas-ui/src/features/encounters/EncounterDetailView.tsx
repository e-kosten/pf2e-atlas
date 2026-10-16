import { useQuery } from "@tanstack/react-query";
import { Alert, Button, Grid } from "antd";
import { Edit2 } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";
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
  const screens = Grid.useBreakpoint();
  const narrow = screens.md === false;
  const [participantOpen, setParticipantOpen] = useState(false);
  const host = useRef<HTMLDivElement>(null);
  const rosterPosition = useRef({ page: 0, pane: 0 });
  const focusDestination = useRef<"participant" | "roster" | null>(null);
  const followAfterRefresh = useRef(false);
  useLayoutEffect(() => {
    const destination = focusDestination.current;
    if (!destination) return;
    focusDestination.current = null;
    if (destination === "participant") {
      host.current?.querySelector<HTMLElement>(".encounter-record-pane")?.focus();
    } else {
      const pane = host.current?.querySelector<HTMLElement>(".encounter-roster");
      if (pane) pane.scrollTop = rosterPosition.current.pane;
      window.scrollTo({ top: rosterPosition.current.page });
      host.current
        ?.querySelector<HTMLElement>("[data-participant-key][aria-pressed=true]")
        ?.focus({ preventScroll: true });
    }
  }, [participantOpen, narrow]);
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
    reloadEncounter: () => encounter.refetch({ throwOnError: true }),
  });
  const revealParticipant = (key: string) => {
    setSelectedParticipantKey(key);
    recordPreview.close();
    setParticipantOpen(true);
    if (narrow) {
      if (!participantOpen) {
        rosterPosition.current = {
          page: window.scrollY,
          pane: host.current?.querySelector(".encounter-roster")?.scrollTop ?? 0,
        };
      }
      focusDestination.current = "participant";
      if (participantOpen)
        host.current?.querySelector<HTMLElement>(".encounter-record-pane")?.focus();
    }
  };
  const followTurn = (view: typeof encounter.data) => {
    if (view?.current_turn_participant_key) {
      revealParticipant(view.current_turn_participant_key);
      const reader = host.current?.querySelector<HTMLElement>(".encounter-record-pane");
      if (reader) reader.scrollTop = 0;
      followAfterRefresh.current = false;
    }
  };
  const advanceTurn = async () => {
    followAfterRefresh.current = true;
    try {
      const result = await mutations.run({ kind: "turn" });
      if (result.active && result.refreshed) followTurn(result.view);
    } catch {
      followAfterRefresh.current = false;
    }
  };
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

  const recovery = mutations.refreshError ? (
    <Alert
      type="error"
      title={mutations.refreshMessage}
      description={mutations.refreshError.message}
      action={
        <Button
          loading={mutations.pending}
          disabled={mutations.pending}
          onClick={() =>
            void mutations.retryRefresh().then((view) => {
              if (followAfterRefresh.current) followTurn(view);
            })
          }
        >
          Retry encounter refresh
        </Button>
      }
    />
  ) : null;

  return (
    <div
      ref={host}
      className={`encounter-detail-view${participantOpen ? " encounter-detail-view--participant" : ""}`}
    >
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
            aria-label="Encounter settings"
            disabled={disabled}
            icon={<Edit2 size={14} />}
            onClick={() => setEditEncounterOpen(true)}
          >
            Encounter settings
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
        {recovery}
        {mutations.error && (
          <Alert
            type="error"
            title="Encounter action failed"
            description={mutations.errorMessage}
          />
        )}
      </div>
      {narrow && participantOpen && (
        <div className="encounter-mobile-navigation">
          <Button
            onClick={() => {
              focusDestination.current = "roster";
              setParticipantOpen(false);
              recordPreview.close();
            }}
          >
            Back to roster
          </Button>
          <Button disabled={disabled} onClick={() => void advanceTurn()}>
            Next turn
          </Button>
        </div>
      )}
      <WorkspaceLayout
        filter={
          <EncounterRosterPane
            recovery={recovery}
            disabled={disabled}
            error={mutations.errorMessage}
            currentTurnParticipantKey={
              encounter.data?.current_turn_participant_key ?? null
            }
            loading={encounter.isLoading}
            onAddRecord={async (request) => {
              const result = await mutations.run({ kind: "add-record", request });
              return result.active && result.refreshed;
            }}
            onAddManual={async (request) => {
              const result = await mutations.run({ kind: "add-manual", request });
              return result.active && result.refreshed;
            }}
            onAdvanceTurn={() => void advanceTurn()}
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
            onSelect={revealParticipant}
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
            onHpAction={async (participant, action) => {
              const result = await mutations.run({
                kind: "hp",
                request: {
                  participant_key: participant.participant_key,
                  expected_revision: participant.revision,
                  action,
                },
              });
              return result.active && result.refreshed;
            }}
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
            previewReadingHistory={recordPreview.readingHistory}
            previewLanding={recordPreview.landing}
            previewDetail={recordPreview.detail}
            previewSelection={recordPreview.selection}
            previewLoading={recordPreview.loading}
            previewError={recordPreview.error}
            onRetryPreview={recordPreview.retry}
            onReloadPreview={recordPreview.reload}
            reloadPreviewLabel={recordPreview.reloadLabel}
            retryPreviewLabel={recordPreview.retryLabel}
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
