import { EncounterModalProvider } from "./EncounterModal";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Alert, Button, Grid } from "antd";
import { Edit2 } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";
import { getEncounter, getEncounterConditionDefinitions } from "../../api/atlasApi";
import { useEncounterStrike } from "./useEncounterStrike";
import { useEncounterMutations } from "./useEncounterMutations";
import { EncounterInspectorPane } from "./EncounterInspectorPane";
import { EditEncounterModal } from "./EncounterModals";
import { EncounterTurnReview } from "./EncounterTurnReview";
import { useEncounterTurnReview } from "./useEncounterTurnReview";
import { EncounterRosterPane } from "./EncounterRosterPane";
import { InitiativeRollModal } from "./InitiativeRollModal";
import { participantUpdate } from "./participantEdits";
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
  const client = useQueryClient();
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
  const [initiativeOpen, setInitiativeOpen] = useState<{ participant?: string } | null>(
    null,
  );
  const encounter = useQuery({
    queryKey: ["encounter", route.slug],
    queryFn: () => getEncounter(route.slug),
  });
  usePageTitle(encounter.data?.encounter.name ?? "Encounter");
  const mutations = useEncounterMutations(
    route.slug,
    encounter.data?.encounter.encounter_key,
  );
  const strikes = useEncounterStrike({
    encounterRef: encounter.data?.encounter.encounter_key ?? route.slug,
    revision: encounter.data?.revision ?? 0n,
    generation: encounter.data?.artifact_generation ?? null,
    participants: encounter.data?.participants ?? [],
    mutations,
  });
  const mutationDisabled =
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
  const conditionDefinitions = useQuery({
    queryKey: [
      "encounter-condition-definitions",
      route.slug,
      selected?.participant_key,
      encounter.data?.artifact_generation,
      encounter.data?.revision.toString(),
    ],
    queryFn: () =>
      getEncounterConditionDefinitions(route.slug, selected!.participant_key),
    enabled: !!selected,
    placeholderData: (previous, previousQuery) =>
      previousQuery?.queryKey[1] === route.slug &&
      previousQuery.queryKey[2] === selected?.participant_key &&
      previousQuery.queryKey[3] === encounter.data?.artifact_generation
        ? previous
        : undefined,
  });
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
  const advanceTurn = async (expectedRevision: bigint) => {
    // Pin the outgoing inspector before the canonical current turn changes.
    if (encounter.data?.current_turn_participant_key)
      setSelectedParticipantKey(encounter.data.current_turn_participant_key);
    followAfterRefresh.current = true;
    try {
      const result = await mutations.run({ kind: "advance", expectedRevision });
      if (result.active && result.refreshed) {
        followAfterRefresh.current = false;
        return true;
      }
    } catch {
      followAfterRefresh.current = false;
      await turnReview.refresh();
    }
    return false;
  };
  const turnReview = useEncounterTurnReview({
    encounterRef: encounter.data?.encounter.encounter_key ?? route.slug,
    mutations,
    onAdvance: advanceTurn,
    onFinish: () =>
      followTurn(client.getQueryData<typeof encounter.data>(["encounter", route.slug])),
    expectedRevision: encounter.data?.revision ?? 0n,
  });
  const disabled = mutationDisabled || turnReview.preparing;
  const encounterSummary = encounter.data?.encounter ?? null;

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
              else if (turnReview.open) void turnReview.refresh();
            })
          }
        >
          Retry encounter refresh
        </Button>
      }
    />
  ) : null;

  return (
    <EncounterModalProvider area={host}>
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
              aria-label="Settings"
              disabled={disabled}
              icon={<Edit2 size={14} />}
              onClick={() => setEditEncounterOpen(true)}
            >
              Settings
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
          {turnReview.error && !turnReview.open && (
            <Alert
              type="error"
              title="Turn review could not load"
              description={turnReview.error.message}
              action={
                <Button
                  loading={turnReview.loading}
                  onClick={() => void turnReview.refresh()}
                >
                  Retry turn review
                </Button>
              }
            />
          )}
          {mutations.error && mutations.errorKind !== "spell" && (
            <Alert
              type="error"
              title="Encounter action failed"
              description={mutations.errorMessage}
            />
          )}
          {turnReview.data?.next_turn_unavailable_reason ===
            "no_initiative_participants" && (
            <Alert
              type="info"
              title="Assign initiative to start turn progression."
              action={
                <Button onClick={() => setInitiativeOpen({})}>Assign initiative</Button>
              }
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
            <Button
              disabled={disabled || !!turnReview.data?.next_turn_unavailable_reason}
              loading={turnReview.preparing ? { delay: 200 } : false}
              onClick={() => void turnReview.begin()}
            >
              Next turn
            </Button>
          </div>
        )}
        <WorkspaceLayout
          filter={
            <EncounterRosterPane
              onRoll={async (request) => {
                const result = await mutations.run({ kind: "initiative", request });
                if (!("rolls" in result.receipt))
                  throw new Error("Missing roll receipt");
                return result.receipt;
              }}
              onRollInitiative={(participant) => setInitiativeOpen({ participant })}
              recovery={recovery}
              disabled={disabled}
              error={
                mutations.errorKind === "spell" ? undefined : mutations.errorMessage
              }
              currentTurnParticipantKey={
                encounter.data?.current_turn_participant_key ?? null
              }
              loading={encounter.isLoading}
              onAddRecord={async (request) => {
                const result = await mutations.run({ kind: "add-record", request });
                return result.active && result.refreshed;
              }}
              onAddParty={async (request) => {
                await mutations.run({ kind: "add-party", request });
              }}
              onAddManual={async (request) => {
                const result = await mutations.run({ kind: "add-manual", request });
                return result.active && result.refreshed;
              }}
              advanceDisabled={!!turnReview.data?.next_turn_unavailable_reason}
              advanceLoading={turnReview.preparing}
              onAdvanceTurn={() => void turnReview.begin()}
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
              onReorder={async (participantKey, targetParticipantKey, placement) => {
                const result = await mutations.run({
                  kind: "reorder",
                  request: {
                    participant_key: participantKey,
                    target_participant_key: targetParticipantKey,
                    placement,
                  },
                });
                return result.active && result.refreshed;
              }}
              onSetTurn={(participant) => {
                followAfterRefresh.current = true;
                void mutations
                  .run({ kind: "turn", participantKey: participant.participant_key })
                  .then((result) => {
                    if (result.active && result.refreshed) followTurn(result.view);
                  })
                  .catch(() => {
                    followAfterRefresh.current = false;
                  });
              }}
              onUpdate={(request) => mutations.run({ kind: "participant", request })}
              participants={encounter.data?.participants ?? []}
              selectedParticipantKey={selected?.participant_key ?? null}
              slug={encounterSummary?.encounter_key ?? route.slug}
            />
          }
          results={
            <EncounterInspectorPane
              currentTurnKey={encounter.data?.current_turn_participant_key}
              onBeginStrike={strikes.launch}
              renderStrikeContext={strikes.renderContext}
              onConditionRecovery={async (request) => {
                const result = await mutations.run({
                  kind: "condition-lifecycle",
                  request,
                });
                if (!("check" in result.receipt))
                  throw new Error("Recovery result unavailable.");
                return result.receipt;
              }}
              onResolveHpLifecycle={async (request) => {
                const result = await mutations.run({
                  kind: "resolve-hp-lifecycle",
                  request,
                });
                return result.active && result.refreshed;
              }}
              encounterKey={encounter.data?.encounter.encounter_key ?? route.slug}
              artifactGeneration={encounter.data?.artifact_generation ?? null}
              encounterRevision={encounter.data?.revision ?? 0n}
              pending={mutations.pending}
              recovery={recovery}
              spellError={
                mutations.errorKind === "spell" ? mutations.errorMessage : undefined
              }
              onSpellAction={async (participant, action, expectedRevision) => {
                try {
                  const result = await mutations.run({
                    kind: "spell",
                    request: {
                      participant_key: participant.participant_key,
                      expected_revision: expectedRevision ?? participant.revision,
                      action,
                    },
                  });
                  return result.active && result.refreshed;
                } catch {
                  return false;
                }
              }}
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
                conditionDefinitions.isPending ||
                conditionDefinitions.isPlaceholderData ||
                !!conditionDefinitions.error
              }
              conditionError={conditionDefinitions.error}
              onRetryConditions={() => void conditionDefinitions.refetch()}
              error={
                mutations.errorKind === "spell" ? undefined : mutations.errorMessage
              }
              onAddCondition={async (request) => {
                const result = await mutations.run({ kind: "add-condition", request });
                return result.active && result.refreshed;
              }}
              onCloseRecordPreview={recordPreview.close}
              onOpenRecordFullPage={(recordKey) =>
                navigateToAtlasRoute({
                  kind: "record",
                  recordKey,
                  selection: recordPreview.selection,
                })
              }
              onReference={recordPreview.open}
              onPreviewRankChange={recordPreview.changeRank}
              onRemoveCondition={async (
                participantKey,
                conditionId,
                expectedRevision,
              ) => {
                const result = await mutations.run({
                  kind: "remove-condition",
                  participantKey,
                  conditionId,
                  expectedRevision,
                });
                return result.active && result.refreshed;
              }}
              onPreviewCondition={mutations.previewCondition}
              onResolveDrained={async (request) => {
                const result = await mutations.run({
                  kind: "resolve-drained",
                  request,
                });
                return result.active && result.refreshed;
              }}
              onLifeSettings={async (participantKey, settings) => {
                const result = await mutations.run({
                  kind: "life-settings",
                  participantKey,
                  request: {
                    expected_encounter_revision: encounter.data!.revision,
                    ...settings,
                  },
                });
                return result.active && result.refreshed;
              }}
              onRecoverPersistent={async (request) => {
                const result = await mutations.run({
                  kind: "persistent-recovery",
                  request,
                });
                if (!("success" in result.receipt))
                  throw new Error("Recovery result unavailable.");
                return result.receipt;
              }}
              onUpdate={async (request) => {
                const result = await mutations.run({ kind: "participant", request });
                return result.active && result.refreshed;
              }}
              onUpdateCondition={async (participantKey, condition) => {
                const result = await mutations.run({
                  kind: "update-condition",
                  participantKey,
                  request: condition,
                });
                return result.active && result.refreshed;
              }}
              participant={selected}
              participants={encounter.data?.participants ?? []}
              conditionDefinitions={conditionDefinitions.data?.conditions ?? []}
              damageTypes={conditionDefinitions.data?.damage_types ?? []}
              standardRecoveryDc={
                conditionDefinitions.data?.standard_persistent_recovery_dc
              }
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
          collapsiblePanes={["filter"]}
          labels={{ filter: "Roster", results: "Participant" }}
          selectedRecordKey={selected?.record_key ?? selected?.participant_key ?? null}
          widthSpecs={ENCOUNTER_WIDTH_SPECS}
        />
        <EncounterTurnReview
          key={turnReview.contextKey}
          open={turnReview.open}
          boundaryKey={turnReview.boundaryKey}
          data={turnReview.data}
          participants={encounter.data?.participants ?? []}
          pending={mutations.pending}
          disabled={disabled}
          loading={turnReview.loading}
          error={turnReview.error}
          actionError={mutations.errorMessage}
          recovery={recovery}
          receipt={turnReview.receipt}
          outcomes={turnReview.outcomes}
          onRetry={() => void turnReview.refresh()}
          onCancel={turnReview.close}
          encounterRef={encounter.data?.encounter.encounter_key ?? route.slug}
          generation={encounter.data?.artifact_generation ?? null}
          damageTypes={conditionDefinitions.data?.damage_types ?? []}
          onLifecycle={async (request) => {
            const receipt = await turnReview.execute({
              kind: "condition-lifecycle",
              request,
            });
            if (!receipt || !("check" in receipt))
              throw new Error("Condition result unavailable.");
            return receipt;
          }}
          onApplyPersistent={async (request) => {
            const receipt = await turnReview.execute({
              kind: "persistent-damage",
              request,
            });
            if (!receipt || !("groups" in receipt))
              throw new Error("Damage result unavailable.");
            return receipt;
          }}
          onRecoverPersistent={async (request) => {
            const receipt = await turnReview.execute({
              kind: "persistent-recovery",
              request,
            });
            if (!receipt || !("success" in receipt))
              throw new Error("Recovery result unavailable.");
            return receipt;
          }}
          onRemove={(reminder) =>
            turnReview.execute({
              kind: "remove-condition",
              participantKey: reminder.participant_key,
              conditionId: reminder.condition_id,
              expectedRevision: turnReview.data!.expected_revision,
            })
          }
        />
        {encounterSummary && (
          <EditEncounterModal
            encounter={encounterSummary}
            disabled={disabled}
            pending={mutations.pending}
            error={mutations.errorKind === "spell" ? undefined : mutations.errorMessage}
            encounterNote={encounter.data?.note}
            open={editEncounterOpen}
            onCancel={() => setEditEncounterOpen(false)}
            onDelete={async () => {
              const result = await mutations.run({ kind: "delete" });
              if (result.active) navigateToAtlasRoute({ kind: "encounters" });
            }}
            onSave={async (request) => {
              await mutations.run({ kind: "metadata", request });
              setEditEncounterOpen(false);
            }}
          />
        )}
        {initiativeOpen && encounter.data && (
          <InitiativeRollModal
            onSetTotal={async (key, value) => {
              const participant = encounter.data?.participants.find(
                (p) => p.participant_key === key,
              );
              if (!participant) throw new Error("Participant unavailable");
              await mutations.run({
                kind: "participant",
                request: participantUpdate(participant, {
                  initiative: value === undefined ? undefined : BigInt(value),
                }),
              });
            }}
            encounterKey={encounter.data.encounter.encounter_key}
            participants={encounter.data.participants}
            initialParticipant={initiativeOpen.participant}
            disabled={disabled}
            onClose={() => {
              if (!mutations.pending) setInitiativeOpen(null);
            }}
            onRoll={async (request) => {
              const result = await mutations.run({ kind: "initiative", request });
              if (!("rolls" in result.receipt)) throw new Error("Missing roll receipt");
              return result.receipt;
            }}
          />
        )}
      </div>
    </EncounterModalProvider>
  );
}
