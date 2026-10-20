import { FactRow, factText } from "../../shared/records/presentationFacts";
import { EncounterStrikeControl } from "./EncounterStrikeControl";
import { Alert, Button } from "antd";
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
import { RecordPreviewPopover } from "../../shared/records/RecordPreviewPopover";
import { RecordPresentation } from "../../shared/records/RecordPresentation";
import type { RecordPreviewAnchor } from "../../shared/records/recordPreviewTypes";
import { EncounterParticipantSettings } from "./EncounterParticipantSettings";
import { EncounterConditionControls } from "./EncounterConditionControls";
import { EncounterHpControls } from "./EncounterHpControls";
import { atlasRoutePath } from "../../app/routes";

export function EncounterInspectorPane({
  disabled,
  encounterRevision,
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
  onHpAction,
  onSpellAction,
  conditionActions,
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
  conditionActions?: React.ReactNode;
  onBeginStrike: (request: import("./useEncounterStrike").StrikeLaunch) => void;
  renderStrikeContext?: (requestId: string) => React.ReactNode;
  requestedConditionEdit?: import("./conditionEditRequest").ConditionEditRequest;
  onSpellAction: (
    participant: EncounterParticipantView,
    action: import("../../generated/atlas").EncounterSpellAction,
    expectedRevision?: bigint,
  ) => Promise<boolean>;
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
      {
        <EncounterParticipantSurface
          key={`participant-${participant.participant_key}`}
          disabled={disabled}
          encounterRevision={encounterRevision}
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
          onHpAction={onHpAction}
          onSpellAction={onSpellAction}
          onUpdateCondition={onUpdateCondition}
          participant={participant}
          conditionActions={conditionActions}
          onBeginStrike={onBeginStrike}
          renderStrikeContext={renderStrikeContext}
          requestedConditionEdit={requestedConditionEdit}
          participants={participants}
        />
      }
      {previewRecordKey && (
        <RecordPreviewPopover
          actions={
            previewDetail && (
              <EncounterSpellControls
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
  onHpAction,
  onSpellAction,
  onUpdateCondition,
  participant,
  conditionActions,
  onBeginStrike,
  renderStrikeContext,
  requestedConditionEdit,
  participants,
}: {
  disabled: boolean;
  encounterRevision: bigint;
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
  onHpAction: (
    participant: EncounterParticipantView,
    action: EncounterHpAction,
  ) => Promise<boolean>;
  onUpdateCondition: (
    participantKey: string,
    condition: Omit<UpdateEncounterParticipantConditionRequest, "request_id">,
  ) => Promise<boolean>;
  participant: EncounterParticipantView;
  conditionActions?: React.ReactNode;
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
      <Button
        size="small"
        disabled={disabled}
        onClick={() =>
          void updateParticipant({ defeated: !participant.defeated }).catch(() => {})
        }
      >
        {participant.defeated ? "Clear defeated" : "Mark defeated"}
      </Button>
    );
  return (
    <section className="encounter-reference">
      <div className="encounter-reference__header">
        <div className="encounter-reference__identity">
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
            {presentation?.classification ?? participant.participant_kind}
            {participant.participant_variant !== "normal" && (
              <span className="encounter-reference__variant">
                {participant.participant_variant === "elite" ? "Elite" : "Weak"}
              </span>
            )}
          </p>
        </div>
        {!hasHp && <div className="encounter-reference__actions">{statusControl}</div>}
      </div>
      {conditionActions}
      {participant.status === "unresolved" && (
        <p>Original record unavailable. Saved participant settings remain editable.</p>
      )}
      <div
        className={`encounter-reference__controls${hasHp ? "" : " encounter-reference__controls--without-hp"}`}
      >
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
        {conditionError && (
          <Alert
            type="error"
            title="Conditions could not load"
            description={conditionError.message}
            action={<Button onClick={onRetryConditions}>Retry conditions</Button>}
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
      {presentation && (
        <RecordPresentation
          activityRolls={(activity) =>
            activity.kind === "strike" && artifactGeneration ? (
              <EncounterStrikeControl
                key={JSON.stringify([
                  activity.navigation,
                  participant.participant_key,
                  participant.revision.toString(),
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
