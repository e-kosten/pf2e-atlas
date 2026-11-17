import { AtlasDrawer } from "../../shared/ui/overlays/AtlasDrawer";
import { useRef, useState } from "react";
import { InitiativeEntryEditor } from "./InitiativeEntryEditor";
import { MarkdownDocument } from "../../shared/documents/MarkdownDocument";
import type {
  AddConditionDraft,
  UpdateConditionDraft,
} from "./useConditionEventPreview";
import { ReferenceHelpProvider } from "../../shared/records/ReferenceHelp";
import type { EncounterMutationCommand } from "./useEncounterMutations";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import {
  EncounterAbilityControls,
  type AbilityStateHandler,
} from "./EncounterAbilityControls";
import {
  EncounterManualTrackers,
  type ManualTrackerHandler,
} from "./EncounterManualTrackers";
import { EncounterAdjustments, type AdjustmentHandler } from "./EncounterAdjustments";
import { EncounterAbilityStates } from "./EncounterAbilityStates";
import {
  EncounterTurnResources,
  type TurnResourceHandler,
} from "./EncounterTurnResources";
import { EncounterOriginalRecord } from "./EncounterOriginalRecord";
import { EncounterLifeStatus } from "./EncounterLifeStatus";
import { EncounterConditionRecovery } from "./EncounterConditionLifecycle";
import { FactRow, factText } from "../../shared/records/presentationFacts";
import { EncounterStrikeControl } from "./EncounterStrikeControl";
import { Button, Space } from "antd";
import { ActorEncounterLayout } from "../../shared/records/ActorEncounterLayout";
import { HazardDefenses, hasHazardDefenses } from "../../shared/records/HazardDefenses";
import { ActorDefenses } from "../../shared/records/ActorReference";
import { RecordIdentityBadges } from "../../shared/records/RecordIdentityBadges";
import type {
  EncounterHpAction,
  EncounterParticipantView,
  EncounterConditionDefinitionView,
  UpdateEncounterParticipantEdit,
} from "../../generated/atlas";
import {
  EncounterSpellControls,
  encounterCastingAnnotations,
} from "./EncounterSpellControls";
import { RecordPreviewDrawer } from "../../shared/records/RecordPreviewDrawer";
import { recordClassification } from "../../shared/records/RecordPresentation";
import { EncounterActorReading } from "./EncounterActorReading";
import type { useRecordPreview } from "../../shared/records/useRecordPreview";
import { EncounterParticipantName } from "./EncounterParticipantName";
import { EncounterParticipantSettings } from "./EncounterParticipantSettings";
import { EncounterConditionControls } from "./EncounterConditionControls";
import { EncounterHpControls } from "./EncounterHpControls";
export function EncounterInspectorPane({
  disabled,
  encounterRevision,
  currentTurnKey,
  encounterKey,
  artifactGeneration,
  pending,
  recovery,
  conditionsDisabled,
  conditionError,
  onRetryConditions,
  error,
  errorKind,
  spellError,
  onAddCondition,
  onRemoveCondition,
  onUpdateCondition,
  onUpdate,
  onReset,
  onLifeSettings,
  onRecoverPersistent,
  onResolveDrained,
  onPreviewCondition,
  onTurnResources,
  onAbilityState,
  onManualTracker,
  onAdjustment,
  onHpAction,
  onSpellAction,
  onConditionRecovery,
  onResolveHpLifecycle,
  onBeginStrike,
  onPrepareStrike,
  renderStrikeContext,
  requestedConditionEdit,
  participant,
  participants,
  conditionDefinitions,
  damageTypes,
  standardRecoveryDc,
  preview,
}: {
  onReset?: () => void;
  onAbilityState?: AbilityStateHandler;
  onManualTracker?: ManualTrackerHandler;
  onAdjustment?: AdjustmentHandler;
  disabled: boolean;
  encounterRevision: number;
  currentTurnKey?: string | null;
  encounterKey: string;
  artifactGeneration: string | null;
  pending: boolean;
  recovery?: React.ReactNode;
  conditionsDisabled: boolean;
  conditionError: Error | null;
  onRetryConditions: () => void;
  error?: string;
  errorKind?: EncounterMutationCommand["kind"];
  spellError?: string;
  onAddCondition: (condition: AddConditionDraft) => Promise<boolean>;
  onRemoveCondition: (
    participantKey: string,
    conditionId: number,
    expectedRevision: number,
  ) => Promise<boolean>;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateConditionDraft,
  ) => Promise<boolean>;
  onUpdate: (
    participantKey: string,
    request: UpdateEncounterParticipantEdit,
  ) => Promise<boolean>;
  onLifeSettings: (
    participantKey: string,
    settings: Omit<
      import("../../generated/atlas").SetEncounterLifeSettingsRequest,
      "request_id" | "expected_encounter_revision"
    >,
  ) => Promise<boolean>;
  onPreviewCondition: (
    participantKey: string,
    request: import("./useConditionEventPreview").ConditionEventInput,
  ) => Promise<import("../../generated/atlas").ConditionEventPreviewView>;
  onResolveDrained: (
    request: Omit<
      import("../../generated/atlas").ResolveDrainedConsequenceRequest,
      "request_id"
    >,
  ) => Promise<boolean>;
  onRecoverPersistent: (
    request: Omit<
      import("../../generated/atlas").RecoverPersistentDamageRequest,
      "request_id"
    >,
  ) => Promise<import("../../generated/atlas").PersistentRecoveryResultView>;
  onConditionRecovery: React.ComponentProps<
    typeof EncounterConditionRecovery
  >["onApply"];
  onResolveHpLifecycle: React.ComponentProps<typeof EncounterLifeStatus>["onResolve"];
  onBeginStrike: (request: import("./useEncounterStrike").StrikeLaunch) => void;
  onPrepareStrike?: (request: import("./useEncounterStrike").StrikeLaunch) => void;
  renderStrikeContext?: (requestId: string) => React.ReactNode;
  requestedConditionEdit?: import("./conditionEditRequest").ConditionEditRequest;
  onSpellAction: (
    participant: EncounterParticipantView,
    action: import("../../generated/atlas").EncounterSpellAction,
    expectedRevision?: number,
  ) => Promise<boolean>;
  onTurnResources?: (
    participant: EncounterParticipantView,
    action: Parameters<TurnResourceHandler>[0],
  ) => Promise<void>;
  onHpAction: (
    participant: EncounterParticipantView,
    action: EncounterHpAction,
  ) => Promise<boolean>;
  participant: EncounterParticipantView | undefined;
  participants: EncounterParticipantView[];
  conditionDefinitions: EncounterConditionDefinitionView[];
  damageTypes?: Array<{
    value: string;
    label: string;
  }>;
  standardRecoveryDc?: number;
  preview: ReturnType<typeof useRecordPreview>;
}) {
  if (!participant) {
    return (
      <section className="encounter-pane detail-empty">Select a participant.</section>
    );
  }
  if (participant.participant_kind === "initiative_entry") {
    return (
      <InitiativeEntryInspector
        key={participant.participant_key}
        participant={participant}
        disabled={disabled}
        pending={pending}
        error={errorKind === "participant" ? error : undefined}
        onUpdate={onUpdate}
      />
    );
  }
  return (
    <ReferenceHelpProvider
      onHelp={preview.openHelp}
      onReference={preview.open}
      generation={artifactGeneration ?? undefined}
    >
      <section
        className="encounter-pane encounter-record-pane"
        tabIndex={-1}
        aria-label="Participant"
      >
        <EncounterConditionRecovery
          key={participant.participant_key}
          encounterRef={encounterKey}
          revision={encounterRevision}
          participant={participant}
          disabled={disabled}
          onApply={onConditionRecovery}
        >
          <EncounterParticipantSurface
            key={`participant-${participant.participant_key}`}
            disabled={disabled}
            encounterRevision={encounterRevision}
            currentTurnKey={currentTurnKey}
            encounterKey={encounterKey}
            artifactGeneration={artifactGeneration}
            pending={pending}
            recovery={recovery}
            conditionsDisabled={conditionsDisabled}
            conditionError={conditionError}
            onRetryConditions={onRetryConditions}
            error={error}
            errorKind={errorKind}
            spellError={spellError}
            conditionDefinitions={conditionDefinitions}
            damageTypes={damageTypes}
            standardRecoveryDc={standardRecoveryDc}
            onAddCondition={onAddCondition}
            onReference={preview.open}
            onRemoveCondition={onRemoveCondition}
            onUpdate={onUpdate}
            onReset={onReset}
            onLifeSettings={onLifeSettings}
            onRecoverPersistent={onRecoverPersistent}
            onResolveDrained={onResolveDrained}
            onPreviewCondition={onPreviewCondition}
            onTurnResources={onTurnResources}
            onAbilityState={onAbilityState}
            onManualTracker={onManualTracker}
            onAdjustment={onAdjustment}
            onHpAction={onHpAction}
            onSpellAction={onSpellAction}
            onUpdateCondition={onUpdateCondition}
            participant={participant}
            onResolveHpLifecycle={onResolveHpLifecycle}
            onBeginStrike={onBeginStrike}
            onPrepareStrike={onPrepareStrike}
            renderStrikeContext={renderStrikeContext}
            requestedConditionEdit={requestedConditionEdit}
            participants={participants}
          />
        </EncounterConditionRecovery>
        <RecordPreviewDrawer
          preview={preview}
          context="encounter"
          visibleOwnerRecordKey={participant.record_key}
          actions={
            preview.detail?.presentation.body.kind === "spell" && (
              <EncounterSpellControls
                encounterRevision={encounterRevision}
                participant={participant}
                detail={preview.detail}
                disabled={disabled || preview.loading || !!preview.error}
                pending={pending}
                error={spellError}
                recovery={recovery}
                encounterRef={encounterKey}
                onAction={(action, expectedRevision) =>
                  onSpellAction(participant, action, expectedRevision)
                }
              />
            )
          }
        />
      </section>
    </ReferenceHelpProvider>
  );
}
function ParticipantNotes({ source }: { source: string }) {
  const [open, setOpen] = useState(false);
  const long = source.length > 650 || source.split("\n").length > 8;
  return (
    <section className="encounter-reference__notes" aria-label="GM notes">
      <h2>GM notes</h2>
      {long ? (
        <>
          <Button type="text" onClick={() => setOpen(true)}>
            Read GM notes
          </Button>
          <AtlasDrawer
            open={open}
            title="GM notes"
            size="min(760px, 100vw)"
            onClose={() => setOpen(false)}
          >
            <MarkdownDocument source={source} />
          </AtlasDrawer>
        </>
      ) : (
        <MarkdownDocument source={source} />
      )}
    </section>
  );
}

function InitiativeEntryInspector({
  participant,
  disabled,
  pending,
  error,
  onUpdate,
}: {
  participant: EncounterParticipantView;
  disabled: boolean;
  pending: boolean;
  error?: string;
  onUpdate: (key: string, edit: UpdateEncounterParticipantEdit) => Promise<boolean>;
}) {
  const [editing, setEditing] = useState(false);
  const [revision, setRevision] = useState(participant.revision);
  const pane = useRef<HTMLElement>(null);
  const editButton = useRef<HTMLButtonElement>(null);
  const initiative = participant.initiative;
  return (
    <section
      ref={pane}
      className="encounter-pane encounter-record-pane"
      aria-label="Custom turn"
      tabIndex={-1}
    >
      <section className="encounter-reference">
        <div className="encounter-reference__notes">
          <div className="encounter-reference__entry-heading">
            <div>
              <h2>{participant.display_name}</h2>
              <p className="muted">Initiative {initiative}</p>
            </div>
            <Button
              ref={editButton}
              disabled={disabled || initiative == null}
              onClick={() => {
                setRevision(participant.revision);
                setEditing(true);
              }}
            >
              Edit custom turn
            </Button>
          </div>
          {participant.note && <MarkdownDocument source={participant.note} />}
        </div>
        {editing && initiative != null && (
          <InitiativeEntryEditor
            initial={{
              display_name: participant.display_name,
              initiative,
              note: participant.note ?? "",
            }}
            conflict={
              revision !== participant.revision
                ? {
                    latest: {
                      display_name: participant.display_name,
                      initiative,
                      note: participant.note ?? "",
                    },
                    keepDraft: () => setRevision(participant.revision),
                  }
                : undefined
            }
            disabled={disabled}
            pending={pending}
            error={error}
            onSave={(values) =>
              onUpdate(participant.participant_key, {
                ...values,
                expected_revision: revision,
                note: values.note?.trim() ? values.note : null,
              })
            }
            onClose={() => setEditing(false)}
            getDismissFocus={() => ({ opener: editButton.current, pane: pane.current })}
          />
        )}
      </section>
    </section>
  );
}

function EncounterParticipantSurface({
  disabled,
  encounterRevision,
  currentTurnKey,
  encounterKey,
  artifactGeneration,
  pending,
  recovery,
  conditionsDisabled,
  conditionError,
  onRetryConditions,
  error,
  errorKind,
  spellError,
  conditionDefinitions,
  damageTypes,
  standardRecoveryDc,
  onAddCondition,
  onReference,
  onRemoveCondition,
  onUpdate,
  onReset,
  onLifeSettings,
  onRecoverPersistent,
  onResolveDrained,
  onPreviewCondition,
  onTurnResources,
  onAbilityState,
  onManualTracker,
  onAdjustment,
  onHpAction,
  onSpellAction,
  onUpdateCondition,
  participant,
  onResolveHpLifecycle,
  onBeginStrike,
  onPrepareStrike,
  renderStrikeContext,
  requestedConditionEdit,
  participants,
}: {
  onAbilityState?: AbilityStateHandler;
  onManualTracker?: ManualTrackerHandler;
  onAdjustment?: AdjustmentHandler;
  disabled: boolean;
  encounterRevision: number;
  currentTurnKey?: string | null;
  encounterKey: string;
  artifactGeneration: string | null;
  pending: boolean;
  recovery?: React.ReactNode;
  onReset?: () => void;
  conditionsDisabled: boolean;
  conditionError: Error | null;
  onRetryConditions: () => void;
  error?: string;
  errorKind?: EncounterMutationCommand["kind"];
  spellError?: string;
  conditionDefinitions: EncounterConditionDefinitionView[];
  damageTypes?: Array<{
    value: string;
    label: string;
  }>;
  standardRecoveryDc?: number;
  onAddCondition: (condition: AddConditionDraft) => Promise<boolean>;
  onReference: import("../../shared/records/PreparedContent").RecordReferenceHandler;
  onRemoveCondition: (
    participantKey: string,
    conditionId: number,
    expectedRevision: number,
  ) => Promise<boolean>;
  onUpdate: (
    participantKey: string,
    request: UpdateEncounterParticipantEdit,
  ) => Promise<boolean>;
  onLifeSettings: (
    participantKey: string,
    settings: Omit<
      import("../../generated/atlas").SetEncounterLifeSettingsRequest,
      "request_id" | "expected_encounter_revision"
    >,
  ) => Promise<boolean>;
  onPreviewCondition: (
    participantKey: string,
    request: import("./useConditionEventPreview").ConditionEventInput,
  ) => Promise<import("../../generated/atlas").ConditionEventPreviewView>;
  onResolveDrained: (
    request: Omit<
      import("../../generated/atlas").ResolveDrainedConsequenceRequest,
      "request_id"
    >,
  ) => Promise<boolean>;
  onRecoverPersistent: (
    request: Omit<
      import("../../generated/atlas").RecoverPersistentDamageRequest,
      "request_id"
    >,
  ) => Promise<import("../../generated/atlas").PersistentRecoveryResultView>;
  onSpellAction: (
    participant: EncounterParticipantView,
    action: import("../../generated/atlas").EncounterSpellAction,
    expectedRevision?: number,
  ) => Promise<boolean>;
  onTurnResources?: (
    participant: EncounterParticipantView,
    action: Parameters<TurnResourceHandler>[0],
  ) => Promise<void>;
  onHpAction: (
    participant: EncounterParticipantView,
    action: EncounterHpAction,
  ) => Promise<boolean>;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateConditionDraft,
  ) => Promise<boolean>;
  participant: EncounterParticipantView;
  onResolveHpLifecycle: React.ComponentProps<typeof EncounterLifeStatus>["onResolve"];
  onBeginStrike: (request: import("./useEncounterStrike").StrikeLaunch) => void;
  onPrepareStrike?: (request: import("./useEncounterStrike").StrikeLaunch) => void;
  renderStrikeContext?: (requestId: string) => React.ReactNode;
  requestedConditionEdit?: import("./conditionEditRequest").ConditionEditRequest;
  participants: EncounterParticipantView[];
}) {
  const errorFor = (...kinds: EncounterMutationCommand["kind"][]) =>
    errorKind && kinds.includes(errorKind) ? error : undefined;
  const updateParticipant = (
    changes: Omit<UpdateEncounterParticipantEdit, "expected_revision">,
  ) =>
    onUpdate(participant.participant_key, {
      expected_revision: participant.revision,
      ...changes,
    });
  const presentation = participant.presentation;
  const body = presentation?.body;
  const actor =
    body?.kind === "creature"
      ? body.value
      : body?.kind === "hazard"
        ? body.value.actor
        : undefined;
  const hasHp = !(
    body?.kind === "hazard" &&
    body.value.defense_block.state === "value" &&
    body.value.defense_block.value === false
  );
  const statusControl =
    participant.participant_kind === "hazard" ? (
      <Button
        size="small"
        disabled={disabled}
        onClick={() =>
          void updateParticipant({
            hazard_disabled: !participant.hazard_disabled,
          }).catch(() => {})
        }
      >
        {participant.hazard_disabled ? "Activate" : "Disable"}
      </Button>
    ) : (
      <EncounterLifeStatus
        encounterRef={encounterKey}
        revision={encounterRevision}
        participant={participant}
        participants={participants}
        currentTurnKey={currentTurnKey}
        disabled={disabled}
        onSave={(settings) => onLifeSettings(participant.participant_key, settings)}
        onDefeated={() => updateParticipant({ defeated: !participant.defeated })}
        onResolve={onResolveHpLifecycle}
      />
    );
  const hpControls = (
    <>
      {hasHp && (
        <EncounterHpControls
          pending={pending}
          recovery={recovery}
          error={errorFor("hp")}
          disabled={disabled}
          current={participant}
          onAction={(action) => onHpAction(participant, action)}
          statusControl={statusControl}
        />
      )}
    </>
  );
  const turnControls = (
    <EncounterTurnResources
      value={participant.turn_resources}
      onReference={onReference}
      disabled={disabled}
      isCurrent={currentTurnKey === participant.participant_key}
      onChange={
        onTurnResources ? (action) => onTurnResources(participant, action) : undefined
      }
    />
  );
  const conditionControls = (
    <div className="encounter-runtime-stack">
      {turnControls}
      {conditionError && (
        <ErrorNotice
          error={conditionError}
          compact
          type="warning"
          title="Condition choices unavailable"
          action={<Button onClick={onRetryConditions}>Retry</Button>}
        />
      )}
      <EncounterConditionControls
        requestedEdit={requestedConditionEdit}
        encounterRef={encounterKey}
        encounterRevision={encounterRevision}
        pending={pending}
        recovery={recovery}
        disabled={disabled}
        addDisabled={conditionsDisabled}
        error={errorFor(
          "add-condition",
          "update-condition",
          "remove-condition",
          "ability-state",
        )}
        onAbilityState={onAbilityState}
        conditionDefinitions={conditionDefinitions}
        damageTypes={damageTypes}
        standardRecoveryDc={standardRecoveryDc}
        current={participant}
        onAddCondition={onAddCondition}
        onReference={onReference}
        onRemoveCondition={onRemoveCondition}
        onUpdateCondition={onUpdateCondition}
        onPreviewCondition={onPreviewCondition}
        onRecoverPersistent={onRecoverPersistent}
        onResolveDrained={onResolveDrained}
        participants={participants}
      />
      <EncounterAdjustments
        key={participant.participant_key}
        participant={participant}
        participants={participants}
        encounterRef={encounterKey}
        encounterRevision={encounterRevision}
        referenceGeneration={artifactGeneration}
        disabled={disabled}
        pending={pending}
        recovery={recovery}
        error={errorFor("gm-adjustment")}
        onChange={onAdjustment}
      />
      <EncounterManualTrackers
        participant={participant}
        participants={participants}
        encounterRevision={encounterRevision}
        disabled={disabled}
        pending={pending}
        recovery={recovery}
        error={errorFor("manual-tracker")}
        onChange={onManualTracker}
      />
      <EncounterAbilityStates
        participantKey={participant.participant_key}
        abilities={participant.ability_states}
        disabled={disabled}
        onChange={onAbilityState}
        onReference={onReference}
      />
    </div>
  );
  const contextualArmorClass = actor?.runtime?.contextual_armor_class ?? [];
  const contextualDefenses = contextualArmorClass.length > 0 && (
    <div className="record-fact-list">
      {contextualArmorClass.map((defense) => (
        <FactRow
          key={defense.opponent_key}
          label={`AC against ${participants.find((p) => p.participant_key === defense.opponent_key)?.display_name ?? defense.opponent_key}`}
        >
          {factText(defense.value)}
        </FactRow>
      ))}
    </div>
  );
  const showDurability =
    hasHp ||
    contextualArmorClass.length > 0 ||
    (body?.kind === "hazard" && hasHazardDefenses(body.value));
  return (
    <section className="encounter-reference">
      <div className="encounter-reference__header">
        <div className="encounter-reference__identity">
          <div className="encounter-reference__title">
            <div className="encounter-reference__name">
              <EncounterParticipantName
                key={participant.participant_key}
                participant={participant}
                disabled={disabled}
                onUpdate={onUpdate}
              />
              <EncounterParticipantSettings
                onReset={onReset}
                participant={participant}
                disabled={disabled}
                pending={pending}
                error={errorFor("participant", "life-settings")}
                recovery={recovery}
                onUpdate={onUpdate}
                onLifeSettings={(settings) =>
                  onLifeSettings(participant.participant_key, settings)
                }
              />
            </div>
            <p className="record-reference__kind">
              {presentation
                ? recordClassification(presentation)
                : participant.participant_kind}
            </p>
            <div className="encounter-reference__actions">
              {participant.record_key && (
                <EncounterOriginalRecord
                  recordKey={participant.record_key}
                  label={
                    participant.participant_kind === "hazard"
                      ? "View hazard reference"
                      : "View creature reference"
                  }
                />
              )}
              {!hasHp && statusControl}
            </div>
          </div>
          <RecordIdentityBadges identity={presentation?.identity} />
          {presentation?.identity.publication?.trim() && (
            <p className="muted record-reference__source">
              Source: <span>{presentation.identity.publication}</span>
            </p>
          )}
        </div>
      </div>
      {participant.note && <ParticipantNotes source={participant.note} />}
      {participant.status === "unresolved" && (
        <p>Reference unavailable. Saved participant settings remain editable.</p>
      )}
      {participant.status === "unchecked" && (
        <p>
          Reference details could not be checked. Saved participant settings remain
          editable.
        </p>
      )}
      {!presentation && (
        <ActorEncounterLayout durability={hpControls} runtime={conditionControls} />
      )}

      {presentation && (
        <EncounterActorReading
          actorEncounter={
            body?.kind === "creature" || body?.kind === "hazard"
              ? {
                  participantKey: participant.participant_key,
                  durability: showDurability ? (
                    <section
                      className="encounter-reference__durability"
                      aria-label="HP and defenses"
                    >
                      {hpControls}
                      {body?.kind === "hazard" ? (
                        <HazardDefenses hazard={body.value} hideHitPoints />
                      ) : (
                        actor && <ActorDefenses actor={actor} hideHitPoints />
                      )}
                      {contextualDefenses}
                    </section>
                  ) : undefined,
                  conditions: conditionControls,
                }
              : undefined
          }
          reading={
            artifactGeneration && participant.record_key
              ? {
                  generation: artifactGeneration,
                  renderAbilityControls: (navigation) => {
                    const abilities =
                      participant.ability_states?.filter(
                        (ability) =>
                          JSON.stringify(ability.navigation.embedded_path) ===
                          JSON.stringify(navigation.embedded_path),
                      ) ?? [];
                    const render = (kind: "option" | "uses") => {
                      const matching = abilities.filter(
                        (ability) => ability.state.kind === kind,
                      );
                      const visibleOptions = matching.filter(
                        ({ state }) =>
                          state.kind === "option" &&
                          (!state.always_active || state.choices.length > 0),
                      );
                      return matching.length ? (
                        <Space wrap>
                          {matching.map((ability) => (
                            <EncounterAbilityControls
                              key={`${participant.participant_key}:${String(ability.rule_ordinal)}`}
                              ability={ability}
                              showLabel={
                                visibleOptions.length > 1 &&
                                visibleOptions.includes(ability)
                              }
                              disabled={disabled}
                              onChange={onAbilityState}
                            />
                          ))}
                        </Space>
                      ) : null;
                    };
                    return { activation: render("option"), usage: render("uses") };
                  },
                  parent: {
                    record_key: participant.record_key,
                    embedded_path: [],
                    field: null,
                    passage: null,
                    source_fingerprint: null,
                  },
                }
              : undefined
          }
          activityRolls={(activity) =>
            activity.kind === "strike" && artifactGeneration ? (
              <EncounterStrikeControl
                key={JSON.stringify([
                  activity.navigation,
                  participant.participant_key,
                  artifactGeneration,
                ])}
                encounterKey={encounterKey}
                generation={artifactGeneration}
                participant={participant}
                activity={activity}
                onBeginStrike={onBeginStrike}
                onPrepareStrike={onPrepareStrike}
                renderStrikeContext={renderStrikeContext}
                disabled={disabled}
              />
            ) : null
          }
          castingAnnotations={encounterCastingAnnotations({
            participant,
            disabled,
            pending,
            error: spellError,
            recovery,
            onAction: (action, expectedRevision) =>
              onSpellAction(participant, action, expectedRevision),
          })}
          presentation={presentation}
          onReference={onReference}
          encounter
          hideIdentity
        />
      )}
    </section>
  );
}
