import { Alert, Button, Select, Modal } from "antd";
import { useState } from "react";
import type { getRecordDetail } from "../../api/atlasApi";
import type {
  AddEncounterParticipantConditionRequest,
  EncounterParticipantVariantView,
  EncounterHpAction,
  EncounterParticipantView,
  EncounterConditionDefinitionView,
  UpdateEncounterParticipantConditionRequest,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import { participantUpdate } from "./participantEdits";
import { RecordPreviewPopover } from "../../shared/records/RecordPreviewPopover";
import { RecordDetailPane } from "../../shared/records/RecordDetailPane";
import { useRecordDetail } from "../../shared/records/useRecordDetail";
import {
  RecordIdentity,
  RecordPresentation,
} from "../../shared/records/RecordPresentation";
import type { RecordPreviewAnchor } from "../../shared/records/recordPreviewTypes";
import { EncounterParticipantSettings } from "./EncounterParticipantSettings";
import { useRecordPreview } from "../../shared/records/useRecordPreview";
import { EncounterConditionControls } from "./EncounterConditionControls";
import { EncounterHpControls } from "./EncounterHpControls";
import { atlasRoutePath, navigateToAtlasRoute } from "../../app/routes";

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
  return (
    <section className="encounter-reference">
      <div className="encounter-reference__header">
        {presentation ? (
          <RecordIdentity
            identity={{ ...presentation.identity, title: participant.display_name }}
            classification={presentation.classification}
          />
        ) : (
          <h2>{participant.display_name}</h2>
        )}
        <div className="encounter-reference__header-actions">
          {participant.record_key && (
            <ParticipantSourceContent recordKey={participant.record_key} />
          )}
          <EncounterParticipantSettings
            participant={participant}
            disabled={disabled}
            pending={pending}
            error={error}
            recovery={recovery}
            onUpdate={onUpdate}
          />
        </div>
        <div className="encounter-reference__adjustment">
          <ParticipantVariantControl
            disabled={disabled}
            participant={participant}
            onUpdate={updateParticipant}
          />
          <Button
            size="small"
            disabled={disabled}
            onClick={() =>
              void updateParticipant({ defeated: !participant.defeated }).catch(
                () => {},
              )
            }
          >
            {participant.defeated ? "Mark active" : "Mark defeated"}
          </Button>
        </div>
      </div>
      {participant.status === "unresolved" && (
        <p>Authored record unavailable. Saved participant settings remain editable.</p>
      )}
      <div className="encounter-reference__controls">
        <EncounterHpControls
          pending={pending}
          recovery={recovery}
          error={error}
          disabled={disabled}
          current={participant}
          onAction={(action) => onHpAction(participant, action)}
        />
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

function ParticipantVariantControl({
  disabled,
  onUpdate,
  participant,
}: {
  onUpdate: (changes: Partial<UpdateEncounterParticipantRequest>) => Promise<boolean>;
  participant: EncounterParticipantView;
  disabled: boolean;
}) {
  if (participant.participant_kind !== "creature") {
    return null;
  }
  return (
    <div className="encounter-variant-control">
      <span>Adjustment</span>
      <Select
        disabled={disabled}
        aria-label="Adjustment"
        className="encounter-variant-select"
        onChange={(value: EncounterParticipantVariantView) =>
          void onUpdate({ participant_variant: value }).catch(() => {})
        }
        options={[
          { label: "Normal", value: "normal" },
          { label: "Elite", value: "elite" },
          { label: "Weak", value: "weak" },
        ]}
        size="small"
        value={participant.participant_variant}
      />
    </div>
  );
}

function ParticipantSourceContent({ recordKey }: { recordKey: string }) {
  const [open, setOpen] = useState(false);
  const detail = useRecordDetail(open ? recordKey : null);
  const preview = useRecordPreview();
  const close = () => {
    preview.close();
    setOpen(false);
  };
  return (
    <>
      <Button size="small" onClick={() => setOpen(true)}>
        Source
      </Button>
      <Modal open={open} onCancel={close} footer={null} title="Source record">
        <p className="muted">Authored reference; encounter changes are not applied.</p>
        <RecordDetailPane
          detail={detail.data}
          loading={detail.isLoading || detail.isFetching}
          error={detail.error}
          onRetry={() => {
            void detail.refetch();
          }}
          onBack={close}
          backLabel="Close source content"
          onReference={preview.open}
        />
        {preview.recordKey && (
          <RecordPreviewPopover
            visibleOwnerRecordKey={recordKey}
            anchor={preview.anchor}
            detail={preview.detail}
            loading={preview.loading}
            error={preview.error}
            onRetry={preview.retry}
            onClose={preview.close}
            onReference={preview.open}
            readingHistory={preview.readingHistory}
            landing={preview.landing}
            onOpenFullPage={() => {
              if (preview.recordKey)
                navigateToAtlasRoute({
                  kind: "record",
                  recordKey: preview.recordKey,
                  selection: preview.selection,
                  landing: preview.landing,
                });
            }}
          />
        )}
      </Modal>
    </>
  );
}
