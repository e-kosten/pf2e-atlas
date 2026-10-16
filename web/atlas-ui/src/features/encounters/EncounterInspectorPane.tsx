import { Alert, Button, Collapse, Form, Input, Select, Modal } from "antd";
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
import { optionalBigIntInput, participantUpdate } from "./participantEdits";
import { RecordPreviewPopover } from "../../shared/records/RecordPreviewPopover";
import { RecordDetailPane } from "../../shared/records/RecordDetailPane";
import { useRecordDetail } from "../../shared/records/useRecordDetail";
import {
  RecordIdentity,
  RecordPresentation,
} from "../../shared/records/RecordPresentation";
import type { RecordPreviewAnchor } from "../../shared/records/recordPreviewTypes";
import { EditableCommitField } from "../../shared/ui/forms/EditableCommitField";
import { EncounterConditionControls } from "./EncounterConditionControls";
import { EncounterHpControls } from "./EncounterHpControls";
import { atlasRoutePath } from "../../app/routes";

export function EncounterInspectorPane({
  disabled,
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
  conditionsDisabled: boolean;
  conditionError: Error | null;
  onRetryConditions: () => void;
  error?: string;
  onCloseRecordPreview: () => void;
  onOpenRecordFullPage: (recordKey: string) => void;
  onReference: import("../../shared/records/PreparedContent").RecordReferenceHandler;
  onAddCondition: (
    condition: AddEncounterParticipantConditionRequest,
  ) => Promise<unknown>;
  onRemoveCondition: (participantKey: string, conditionId: bigint) => Promise<unknown>;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => Promise<unknown>;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => Promise<unknown>;
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
      {participant.record_key && (
        <ParticipantSourceContent
          key={`source-${participant.participant_key}`}
          recordKey={participant.record_key}
          onReference={onReference}
        />
      )}
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
  conditionsDisabled: boolean;
  conditionError: Error | null;
  onRetryConditions: () => void;
  error?: string;
  conditionDefinitions: EncounterConditionDefinitionView[];
  onAddCondition: (
    condition: AddEncounterParticipantConditionRequest,
  ) => Promise<unknown>;
  onReference: import("../../shared/records/PreparedContent").RecordReferenceHandler;
  onRemoveCondition: (participantKey: string, conditionId: bigint) => Promise<unknown>;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => Promise<unknown>;
  onHpAction: (
    participant: EncounterParticipantView,
    action: EncounterHpAction,
  ) => Promise<boolean>;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => Promise<unknown>;
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
          <ParticipantVariantControl
            disabled={disabled}
            participant={participant}
            onUpdate={updateParticipant}
          />
        </div>
      </div>
      {participant.status === "unresolved" && (
        <p>Authored record unavailable. Saved participant settings remain editable.</p>
      )}
      <div className="encounter-reference__controls">
        <EncounterHpControls
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
      <Collapse
        ghost
        size="small"
        items={[
          {
            key: "participant-details",
            label: "Participant settings",
            styles: { header: { paddingInline: 0 }, body: { paddingInline: 0 } },
            children: (
              <>
                <ParticipantEditStrip
                  disabled={disabled}
                  participant={participant}
                  onUpdate={updateParticipant}
                />
                <ParticipantNoteEditor
                  disabled={disabled}
                  participant={participant}
                  onUpdate={updateParticipant}
                />
              </>
            ),
          },
        ]}
      />
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
  onUpdate: (changes: Partial<UpdateEncounterParticipantRequest>) => Promise<unknown>;
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

function ParticipantEditStrip({
  disabled,
  onUpdate,
  participant,
}: {
  onUpdate: (changes: Partial<UpdateEncounterParticipantRequest>) => Promise<unknown>;
  participant: EncounterParticipantView;
  disabled: boolean;
}) {
  return (
    <section className="encounter-participant-strip">
      <Form.Item style={{ marginBottom: 0 }} label="Name" layout="vertical">
        <EditableCommitField
          disabled={disabled}
          ariaLabel="Participant name"
          onCommit={(displayName) => onUpdate({ display_name: displayName })}
          value={participant.display_name}
        />
      </Form.Item>
      <Form.Item style={{ marginBottom: 0 }} label="Initiative" layout="vertical">
        <EditableCommitField
          disabled={disabled}
          ariaLabel="Participant initiative"
          inputMode="numeric"
          onCommit={(value) => {
            const initiative = optionalBigIntInput(value);
            if (initiative !== null) {
              return onUpdate({ initiative });
            }
          }}
          value={participant.initiative?.toString() ?? ""}
        />
      </Form.Item>
      <Form.Item style={{ marginBottom: 0 }} label="Side" layout="vertical">
        <Select
          disabled={disabled}
          aria-label="Participant side"
          value={participant.side}
          onChange={(side) => void onUpdate({ side }).catch(() => {})}
          options={["pc", "ally", "enemy", "neutral", "hazard"].map((value) => ({
            value,
            label: value,
          }))}
        />
      </Form.Item>
      <Button
        disabled={disabled}
        onClick={() =>
          void onUpdate({ defeated: !participant.defeated }).catch(() => {})
        }
      >
        {participant.defeated ? "Mark active" : "Mark defeated"}
      </Button>
    </section>
  );
}

function ParticipantNoteEditor({
  disabled,
  onUpdate,
  participant,
}: {
  disabled: boolean;
  onUpdate: (changes: Partial<UpdateEncounterParticipantRequest>) => Promise<unknown>;
  participant: EncounterParticipantView;
}) {
  const [draft, setDraft] = useState<string | null>(null);
  const value = draft ?? participant.note ?? "";
  return (
    <Input.TextArea
      placeholder="Participant notes…"
      disabled={disabled}
      aria-label="Participant note text"
      value={value}
      onChange={(event) => setDraft(event.currentTarget.value)}
      onBlur={() => {
        if (!disabled && value !== (participant.note ?? "")) {
          void onUpdate({ note: value })
            .then(() => setDraft(null))
            .catch(() => {});
        }
      }}
    />
  );
}

function ParticipantSourceContent({
  recordKey,
  onReference,
}: {
  recordKey: string;
  onReference: import("../../shared/records/PreparedContent").RecordReferenceHandler;
}) {
  const [open, setOpen] = useState(false);
  const detail = useRecordDetail(open ? recordKey : null);
  return (
    <>
      <Button onClick={() => setOpen(true)}>Read source record</Button>
      <Modal
        open={open}
        onCancel={() => setOpen(false)}
        footer={null}
        title="Source record"
      >
        <p>Authored reference; encounter changes are not applied.</p>
        <RecordDetailPane
          detail={detail.data}
          loading={detail.isLoading || detail.isFetching}
          error={detail.error}
          onRetry={() => {
            void detail.refetch();
          }}
          onBack={() => setOpen(false)}
          backLabel="Close source content"
          onReference={onReference}
        />
      </Modal>
    </>
  );
}
