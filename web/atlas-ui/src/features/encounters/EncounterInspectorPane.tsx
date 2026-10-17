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
  pending,
  recovery,
  conditionsDisabled,
  conditionError,
  onRetryConditions,
  error,
  onCloseRecordPreview,
  onOpenRecordFullPage,
  onReference,
  onAddCondition,
  onRemoveCondition,
  onUpdateCondition,
  onUpdate,
  onHpAction,
  participant,
  participants,
  conditionDefinitions,
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
}: {
  disabled: boolean;
  encounterRevision: bigint;
  pending: boolean;
  recovery?: React.ReactNode;
  conditionsDisabled: boolean;
  conditionError: Error | null;
  onRetryConditions: () => void;
  error?: string;
  onCloseRecordPreview: () => void;
  onOpenRecordFullPage: (recordKey: string) => void;
  onReference: import("../../shared/records/PreparedContent").RecordReferenceHandler;
  onAddCondition: (
    condition: AddEncounterParticipantConditionRequest,
  ) => Promise<boolean>;
  onRemoveCondition: (participantKey: string, conditionId: bigint) => Promise<boolean>;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => Promise<boolean>;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => Promise<boolean>;
  onHpAction: (
    participant: EncounterParticipantView,
    action: EncounterHpAction,
  ) => Promise<boolean>;
  participant: EncounterParticipantView | undefined;
  participants: EncounterParticipantView[];
  conditionDefinitions: EncounterConditionDefinitionView[];
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
          pending={pending}
          recovery={recovery}
          conditionsDisabled={conditionsDisabled}
          conditionError={conditionError}
          onRetryConditions={onRetryConditions}
          error={error}
          conditionDefinitions={conditionDefinitions}
          onAddCondition={onAddCondition}
          onReference={onReference}
          onRemoveCondition={onRemoveCondition}
          onUpdate={onUpdate}
          onHpAction={onHpAction}
          onUpdateCondition={onUpdateCondition}
          participant={participant}
          participants={participants}
        />
      }
      {previewRecordKey && (
        <RecordPreviewPopover
          context="encounter"
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
  pending,
  recovery,
  conditionsDisabled,
  conditionError,
  onRetryConditions,
  error,
  conditionDefinitions,
  onAddCondition,
  onReference,
  onRemoveCondition,
  onUpdate,
  onHpAction,
  onUpdateCondition,
  participant,
  participants,
}: {
  disabled: boolean;
  encounterRevision: bigint;
  pending: boolean;
  recovery?: React.ReactNode;
  conditionsDisabled: boolean;
  conditionError: Error | null;
  onRetryConditions: () => void;
  error?: string;
  conditionDefinitions: EncounterConditionDefinitionView[];
  onAddCondition: (
    condition: AddEncounterParticipantConditionRequest,
  ) => Promise<boolean>;
  onReference: import("../../shared/records/PreparedContent").RecordReferenceHandler;
  onRemoveCondition: (participantKey: string, conditionId: bigint) => Promise<boolean>;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => Promise<boolean>;
  onHpAction: (
    participant: EncounterParticipantView,
    action: EncounterHpAction,
  ) => Promise<boolean>;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => Promise<boolean>;
  participant: EncounterParticipantView;
  participants: EncounterParticipantView[];
}) {
  const updateParticipant = (changes: Partial<UpdateEncounterParticipantRequest>) =>
    onUpdate(participantUpdate(participant, changes));
  const presentation = participant.presentation;
  const body = presentation?.body;
  const hasHp = !(
    body?.kind === "hazard" &&
    body.value.defense_block.state === "value" &&
    body.value.defense_block.value === false
  );
  const statusControl = (
    <Button
      size="small"
      disabled={disabled}
      onClick={() =>
        void updateParticipant({ defeated: !participant.defeated }).catch(() => {})
      }
    >
      {participant.defeated ? "Mark active" : "Mark defeated"}
    </Button>
  );
  return (
    <section className="encounter-reference">
      <div className="encounter-reference__header">
        <div className="encounter-reference__identity">
          <h2>{participant.display_name}</h2>
          <p className="record-reference__kind">
            {presentation?.classification ?? participant.participant_kind}
            {participant.participant_variant !== "normal" && (
              <span className="encounter-reference__variant">
                {participant.participant_variant === "elite" ? "Elite" : "Weak"}
              </span>
            )}
          </p>
        </div>
        <div className="encounter-reference__actions">
          {!hasHp && statusControl}
          <EncounterParticipantSettings
            participant={participant}
            disabled={disabled}
            pending={pending}
            error={error}
            recovery={recovery}
            onUpdate={onUpdate}
          />
        </div>
      </div>
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
          encounterRevision={encounterRevision}
          pending={pending}
          recovery={recovery}
          disabled={disabled}
          addDisabled={conditionsDisabled}
          error={error}
          conditionDefinitions={conditionDefinitions}
          current={participant}
          onAddCondition={onAddCondition}
          onReference={onReference}
          onRemoveCondition={onRemoveCondition}
          onUpdateCondition={onUpdateCondition}
          participants={participants}
        />
      </div>
      {presentation && (
        <RecordPresentation
          presentation={presentation}
          onReference={onReference}
          encounter
          hideIdentity
        />
      )}
    </section>
  );
}
