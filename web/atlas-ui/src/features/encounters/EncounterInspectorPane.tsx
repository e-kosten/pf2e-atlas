import {
  EncounterTurnResources,
  type TurnResourceHandler,
} from "./EncounterTurnResources";
import { EncounterOriginalRecord } from "./EncounterOriginalRecord";
import { EncounterLifeStatus } from "./EncounterLifeStatus";
import { EncounterConditionRecovery } from "./EncounterConditionLifecycle";
import { FactRow, factText } from "../../shared/records/presentationFacts";
import { EncounterStrikeControl } from "./EncounterStrikeControl";
import { Alert, Button } from "antd";
import { ActorEncounterLayout } from "../../shared/records/ActorEncounterLayout";
import { HazardDefenses } from "../../shared/records/HazardDefenses";
import { ActorDefenses } from "../../shared/records/ActorReference";
import { RecordRarity } from "../../shared/records/RecordRarity";
import { TraitBadge } from "../../shared/records/TraitBadge";
import type { getRecordDetail } from "../../api/atlasApi";
import type {
  AddEncounterParticipantConditionRequest,
  EncounterHpAction,
  EncounterParticipantView,
  EncounterConditionDefinitionView,
  UpdateEncounterParticipantConditionRequest,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import {
  EncounterSpellControls,
  encounterCastingAnnotations,
} from "./EncounterSpellControls";
import { participantUpdate } from "./participantEdits";
import { RecordReferenceDrawer } from "../../shared/records/RecordReferenceDrawer";
import { recordClassification } from "../../shared/records/RecordPresentation";
import { EncounterActorReading } from "./EncounterActorReading";
import type { RecordPreviewAnchor } from "../../shared/records/recordPreviewTypes";
import { EncounterParticipantSettings } from "./EncounterParticipantSettings";
import { EncounterConditionControls } from "./EncounterConditionControls";
import { EncounterHpControls } from "./EncounterHpControls";
import { atlasRoutePath } from "../../app/routes";

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
  spellError,
  onCloseRecordPreview,
  onOpenRecordFullPage,
  onReference,
  onAddCondition,
  onRemoveCondition,
  onUpdateCondition,
  onUpdate,
  onLifeSettings,
  onRecoverPersistent,
  onResolveDrained,
  onPreviewCondition,
  onTurnResources,
  onHpAction,
  onSpellAction,
  onConditionRecovery,
  onResolveHpLifecycle,
  onBeginStrike,
  renderStrikeContext,
  requestedConditionEdit,
  participant,
  participants,
  conditionDefinitions,
  damageTypes,
  standardRecoveryDc,
  previewDetail,
  previewLoading,
  previewError,
  onRetryPreview,
  onReloadPreview,
  reloadPreviewLabel,
  retryPreviewLabel,
  previewRecordKey,
  previewAnchor,
  previewReadingHistory,
  previewLanding,
  previewSelection,
  onPreviewRankChange,
}: {
  disabled: boolean;
  encounterRevision: bigint;
  currentTurnKey?: string | null;
  encounterKey: string;
  artifactGeneration: string | null;
  pending: boolean;
  recovery?: React.ReactNode;
  conditionsDisabled: boolean;
  conditionError: Error | null;
  onRetryConditions: () => void;
  error?: string;
  spellError?: string;
  onCloseRecordPreview: () => void;
  onOpenRecordFullPage: (recordKey: string) => void;
  onReference: import("../../shared/records/PreparedContent").RecordReferenceHandler;
  onAddCondition: (
    condition: Omit<AddEncounterParticipantConditionRequest, "request_id">,
  ) => Promise<boolean>;
  onRemoveCondition: (
    participantKey: string,
    conditionId: bigint,
    expectedRevision: bigint,
  ) => Promise<boolean>;
  onUpdateCondition: (
    participantKey: string,
    condition: Omit<UpdateEncounterParticipantConditionRequest, "request_id">,
  ) => Promise<boolean>;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => Promise<boolean>;
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
  renderStrikeContext?: (requestId: string) => React.ReactNode;
  requestedConditionEdit?: import("./conditionEditRequest").ConditionEditRequest;
  onSpellAction: (
    participant: EncounterParticipantView,
    action: import("../../generated/atlas").EncounterSpellAction,
    expectedRevision?: bigint,
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
  damageTypes?: Array<{ value: string; label: string }>;
  standardRecoveryDc?: bigint;
  onPreviewRankChange?: (rank: number | undefined) => void;
  previewSelection?: import("../../generated/atlas").RecordNavigationView;
  previewDetail: Awaited<ReturnType<typeof getRecordDetail>> | undefined;
  previewLoading: boolean;
  previewError?: Error | null;
  onRetryPreview?: () => void;
  onReloadPreview?: () => void;
  reloadPreviewLabel?: string;
  retryPreviewLabel?: string;
  previewRecordKey: string | null;
  previewAnchor: RecordPreviewAnchor | null;
  previewReadingHistory?: import("../../shared/records/recordPreviewTypes").RecordPreviewContentProps["readingHistory"];
  previewLanding?: import("../../generated/atlas").RecordNavigationView;
}) {
  if (!participant) {
    return (
      <section className="encounter-pane detail-empty">Select a participant.</section>
    );
  }
  return (
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
          spellError={spellError}
          conditionDefinitions={conditionDefinitions}
          damageTypes={damageTypes}
          standardRecoveryDc={standardRecoveryDc}
          onAddCondition={onAddCondition}
          onReference={onReference}
          onRemoveCondition={onRemoveCondition}
          onUpdate={onUpdate}
          onLifeSettings={onLifeSettings}
          onRecoverPersistent={onRecoverPersistent}
          onResolveDrained={onResolveDrained}
          onPreviewCondition={onPreviewCondition}
          onTurnResources={onTurnResources}
          onHpAction={onHpAction}
          onSpellAction={onSpellAction}
          onUpdateCondition={onUpdateCondition}
          participant={participant}
          onResolveHpLifecycle={onResolveHpLifecycle}
          onBeginStrike={onBeginStrike}
          renderStrikeContext={renderStrikeContext}
          requestedConditionEdit={requestedConditionEdit}
          participants={participants}
        />
      </EncounterConditionRecovery>
      {previewRecordKey && (
        <RecordReferenceDrawer
          actions={
            previewDetail?.presentation.body.kind === "spell" && (
              <EncounterSpellControls
                encounterRevision={encounterRevision}
                participant={participant}
                detail={previewDetail}
                disabled={disabled || previewLoading || !!previewError}
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
          context="encounter"
          onRankChange={onPreviewRankChange}
          visibleOwnerRecordKey={participant.record_key}
          anchor={previewAnchor}
          readingHistory={previewReadingHistory}
          landing={previewLanding}
          detail={previewDetail}
          fullPageHref={atlasRoutePath({
            kind: "record",
            recordKey: previewRecordKey,
            selection: previewSelection,
            landing: previewLanding,
          })}
          loading={previewLoading}
          error={previewError}
          onRetry={onRetryPreview}
          onReload={onReloadPreview}
          reloadLabel={reloadPreviewLabel}
          retryLabel={retryPreviewLabel}
          onClose={onCloseRecordPreview}
          onOpenFullPage={() => onOpenRecordFullPage(previewRecordKey)}
          onReference={onReference}
        />
      )}
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
  spellError,
  conditionDefinitions,
  damageTypes,
  standardRecoveryDc,
  onAddCondition,
  onReference,
  onRemoveCondition,
  onUpdate,
  onLifeSettings,
  onRecoverPersistent,
  onResolveDrained,
  onPreviewCondition,
  onTurnResources,
  onHpAction,
  onSpellAction,
  onUpdateCondition,
  participant,
  onResolveHpLifecycle,
  onBeginStrike,
  renderStrikeContext,
  requestedConditionEdit,
  participants,
}: {
  disabled: boolean;
  encounterRevision: bigint;
  currentTurnKey?: string | null;
  encounterKey: string;
  artifactGeneration: string | null;
  pending: boolean;
  recovery?: React.ReactNode;
  conditionsDisabled: boolean;
  conditionError: Error | null;
  onRetryConditions: () => void;
  error?: string;
  spellError?: string;
  conditionDefinitions: EncounterConditionDefinitionView[];
  damageTypes?: Array<{ value: string; label: string }>;
  standardRecoveryDc?: bigint;
  onAddCondition: (
    condition: Omit<AddEncounterParticipantConditionRequest, "request_id">,
  ) => Promise<boolean>;
  onReference: import("../../shared/records/PreparedContent").RecordReferenceHandler;
  onRemoveCondition: (
    participantKey: string,
    conditionId: bigint,
    expectedRevision: bigint,
  ) => Promise<boolean>;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => Promise<boolean>;
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
    expectedRevision?: bigint,
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
    condition: Omit<UpdateEncounterParticipantConditionRequest, "request_id">,
  ) => Promise<boolean>;
  participant: EncounterParticipantView;
  onResolveHpLifecycle: React.ComponentProps<typeof EncounterLifeStatus>["onResolve"];
  onBeginStrike: (request: import("./useEncounterStrike").StrikeLaunch) => void;
  renderStrikeContext?: (requestId: string) => React.ReactNode;
  requestedConditionEdit?: import("./conditionEditRequest").ConditionEditRequest;
  participants: EncounterParticipantView[];
}) {
  const updateParticipant = (changes: Partial<UpdateEncounterParticipantRequest>) =>
    onUpdate(participantUpdate(participant, changes));
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
          error={error}
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
        <Alert
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
        error={error}
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
    </div>
  );
  const contextualDefenses = (
    <>
      {actor?.runtime && actor.runtime.contextual_armor_class.length > 0 && (
        <div className="record-fact-list">
          {actor.runtime.contextual_armor_class.map((defense) => (
            <FactRow
              key={defense.opponent_key}
              label={`AC against ${participants.find((p) => p.participant_key === defense.opponent_key)?.display_name ?? defense.opponent_key}`}
            >
              {factText(defense.value)}
            </FactRow>
          ))}
        </div>
      )}
    </>
  );

  return (
    <section className="encounter-reference">
      <div className="encounter-reference__header">
        <div className="encounter-reference__identity">
          <div className="encounter-reference__title">
            <div className="encounter-reference__name">
              <h2>{participant.display_name}</h2>
              <EncounterParticipantSettings
                participant={participant}
                disabled={disabled}
                pending={pending}
                error={error}
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
          </div>
          {presentation &&
          (presentation.identity.traits.length > 0 ||
            (presentation.identity.rarity &&
              presentation.identity.rarity !== "common")) ? (
            <div className="badge-row">
              <RecordRarity rarity={presentation.identity.rarity} />
              {presentation.identity.traits
                .filter((trait) => trait.kind !== "rarity")
                .map((trait) => (
                  <TraitBadge key={`${trait.kind}-${trait.value}`} trait={trait} />
                ))}
            </div>
          ) : null}
          {presentation?.identity.publication?.trim() && (
            <p className="muted record-reference__source">
              Source: <span>{presentation.identity.publication}</span>
            </p>
          )}
        </div>
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
      {participant.status === "unresolved" && (
        <p>Reference unavailable. Saved participant settings remain editable.</p>
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
                  durability: (
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
                  ),
                  conditions: conditionControls,
                }
              : undefined
          }
          reading={
            artifactGeneration && participant.record_key
              ? {
                  generation: artifactGeneration,
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
