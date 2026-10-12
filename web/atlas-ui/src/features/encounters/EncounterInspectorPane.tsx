import { Button, Collapse, Form, Input, Select, Modal } from "antd";
import { Pencil } from "lucide-react";
import { useState } from "react";
import type { getRecordDetail } from "../../api/atlasApi";
import type {
  AddEncounterParticipantConditionRequest,
  EncounterParticipantVariantView,
  EncounterParticipantView,
  EncounterConditionDefinitionView,
  UpdateEncounterParticipantConditionRequest,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import {
  applyParticipantUpdate,
  optionalBigIntInput,
  participantUpdate,
} from "./participantEdits";
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
  onCloseRecordPreview,
  onOpenRecordFullPage,
  onReference,
  onAddCondition,
  onRemoveCondition,
  onUpdateCondition,
  onUpdate,
  participant,
  participants,
  conditionDefinitions,
  previewDetail,
  previewLoading,
  previewError,
  onRetryPreview,
  previewRecordKey,
  previewAnchor,
  previewSelection,
}: {
  onCloseRecordPreview: () => void;
  onOpenRecordFullPage: (recordKey: string) => void;
  onReference: import("../../shared/records/PreparedContent").RecordReferenceHandler;
  onAddCondition: (condition: AddEncounterParticipantConditionRequest) => void;
  onRemoveCondition: (participantKey: string, conditionId: bigint) => void;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => void;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => void;
  participant: EncounterParticipantView | undefined;
  participants: EncounterParticipantView[];
  conditionDefinitions: EncounterConditionDefinitionView[];
  previewSelection?: import("../../generated/atlas").RecordNavigationView;
  previewDetail: Awaited<ReturnType<typeof getRecordDetail>> | undefined;
  previewLoading: boolean;
  previewError?: Error | null;
  onRetryPreview?: () => void;
  previewRecordKey: string | null;
  previewAnchor: RecordPreviewAnchor | null;
}) {
  if (!participant) {
    return (
      <section className="encounter-pane detail-empty">Select a participant.</section>
    );
  }
  return (
    <section className="encounter-pane encounter-record-pane">
      {
        <EncounterParticipantSurface
          key={`participant-${participant.participant_key}`}
          conditionDefinitions={conditionDefinitions}
          onAddCondition={onAddCondition}
          onReference={onReference}
          onRemoveCondition={onRemoveCondition}
          onUpdate={onUpdate}
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
          anchor={previewAnchor}
          detail={previewDetail}
          fullPageHref={atlasRoutePath({
            kind: "record",
            recordKey: previewRecordKey,
            selection: previewSelection,
          })}
          loading={previewLoading}
          error={previewError}
          onRetry={onRetryPreview}
          onClose={onCloseRecordPreview}
          onOpenFullPage={() => onOpenRecordFullPage(previewRecordKey)}
          onReference={onReference}
        />
      )}
    </section>
  );
}

function EncounterParticipantSurface({
  conditionDefinitions,
  onAddCondition,
  onReference,
  onRemoveCondition,
  onUpdate,
  onUpdateCondition,
  participant,
  participants,
}: {
  conditionDefinitions: EncounterConditionDefinitionView[];
  onAddCondition: (condition: AddEncounterParticipantConditionRequest) => void;
  onReference: import("../../shared/records/PreparedContent").RecordReferenceHandler;
  onRemoveCondition: (participantKey: string, conditionId: bigint) => void;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => void;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => void;
  participant: EncounterParticipantView;
  participants: EncounterParticipantView[];
}) {
  const [projectedCurrent, setProjectedCurrent] = useState<{
    source: EncounterParticipantView | null;
    participant: EncounterParticipantView | null;
  }>({ source: null, participant: null });
  const [noteOpen, setNoteOpen] = useState(false);
  const activeCurrent =
    projectedCurrent.source === participant &&
    projectedCurrent.participant?.participant_key === participant.participant_key
      ? projectedCurrent.participant
      : participant;
  const updateParticipant = (changes: Partial<UpdateEncounterParticipantRequest>) => {
    const request = participantUpdate(activeCurrent, changes);
    setProjectedCurrent({
      source: participant,
      participant: applyParticipantUpdate(activeCurrent, request),
    });
    onUpdate(request);
  };
  const presentation = activeCurrent.presentation ?? participant.presentation;
  return (
    <section className="encounter-reference">
      <div className="encounter-reference__header">
        {presentation ? (
          <RecordIdentity
            identity={{ ...presentation.identity, title: activeCurrent.display_name }}
            classification={presentation.classification}
          />
        ) : (
          <h2>{activeCurrent.display_name}</h2>
        )}
        <div className="encounter-reference__header-actions">
          <Button
            aria-label="Participant note"
            icon={<Pencil size={14} aria-hidden="true" />}
            onClick={() => setNoteOpen((open) => !open)}
            size="small"
            type={noteOpen ? "primary" : "default"}
          />
          <ParticipantVariantControl
            participant={activeCurrent}
            onUpdate={updateParticipant}
          />
        </div>
      </div>
      {activeCurrent.status === "unresolved" && (
        <p>Authored record unavailable. Saved participant settings remain editable.</p>
      )}
      <div className="encounter-reference__controls">
        <EncounterHpControls current={activeCurrent} onUpdate={updateParticipant} />
        <EncounterConditionControls
          conditionDefinitions={conditionDefinitions}
          current={activeCurrent}
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
            label: "Edit details",
            styles: { header: { paddingInline: 0 }, body: { paddingInline: 0 } },
            children: (
              <ParticipantEditStrip
                participant={activeCurrent}
                onUpdate={updateParticipant}
              />
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
      {noteOpen && (
        <ParticipantNoteEditor
          participant={activeCurrent}
          onUpdate={updateParticipant}
        />
      )}
    </section>
  );
}

function ParticipantVariantControl({
  onUpdate,
  participant,
}: {
  onUpdate: (changes: Partial<UpdateEncounterParticipantRequest>) => void;
  participant: EncounterParticipantView;
}) {
  if (participant.participant_kind !== "creature") {
    return null;
  }
  return (
    <div className="encounter-variant-control">
      <span>Variant</span>
      <Select
        aria-label="Variant"
        className="encounter-variant-select"
        onChange={(value: EncounterParticipantVariantView) =>
          onUpdate({ participant_variant: value })
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
  onUpdate,
  participant,
}: {
  onUpdate: (changes: Partial<UpdateEncounterParticipantRequest>) => void;
  participant: EncounterParticipantView;
}) {
  return (
    <section className="encounter-participant-strip">
      <Form.Item style={{ marginBottom: 0 }} label="Name" layout="vertical">
        <EditableCommitField
          ariaLabel="Participant name"
          onCommit={(displayName) => onUpdate({ display_name: displayName })}
          value={participant.display_name}
        />
      </Form.Item>
      <Form.Item style={{ marginBottom: 0 }} label="Initiative" layout="vertical">
        <EditableCommitField
          ariaLabel="Participant initiative"
          inputMode="numeric"
          onCommit={(value) => {
            const initiative = optionalBigIntInput(value);
            if (initiative !== null) {
              onUpdate({ initiative });
            }
          }}
          value={participant.initiative?.toString() ?? ""}
        />
      </Form.Item>
      <Form.Item style={{ marginBottom: 0 }} label="Side" layout="vertical">
        <Select
          aria-label="Participant side"
          value={participant.side}
          onChange={(side) => onUpdate({ side })}
          options={["pc", "ally", "enemy", "neutral", "hazard"].map((value) => ({
            value,
            label: value,
          }))}
        />
      </Form.Item>
      <Button onClick={() => onUpdate({ defeated: !participant.defeated })}>
        {participant.defeated ? "Mark active" : "Mark defeated"}
      </Button>
    </section>
  );
}

function ParticipantNoteEditor({
  onUpdate,
  participant,
}: {
  onUpdate: (changes: Partial<UpdateEncounterParticipantRequest>) => void;
  participant: EncounterParticipantView;
}) {
  return (
    <Input.TextArea
      aria-label="Participant note text"
      key={`note-${participant.participant_key}-${participant.note ?? ""}`}
      defaultValue={participant.note ?? ""}
      onBlur={(event) => {
        const nextNote = event.currentTarget.value;
        if (nextNote !== (participant.note ?? "")) {
          onUpdate({ note: nextNote });
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
      <Button onClick={() => setOpen(true)}>Source content</Button>
      <Modal
        open={open}
        onCancel={() => setOpen(false)}
        footer={null}
        title="Authored source content"
      >
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
