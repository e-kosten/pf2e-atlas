import { Button, Form, Input, Select, Modal } from "antd";
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
import { RecordSurface } from "../../shared/records/RecordSurface";
import type { RecordPreviewAnchor } from "../../shared/records/recordPreviewTypes";
import { EditableCommitField } from "../../shared/ui/forms/EditableCommitField";
import { EncounterConditionControls } from "./EncounterConditionControls";
import { EncounterHpControls } from "./EncounterHpControls";

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
  previewRecordKey,
  previewAnchor,
}: {
  onCloseRecordPreview: () => void;
  onOpenRecordFullPage: (recordKey: string) => void;
  onReference: (recordKey: string, anchorRect?: DOMRect) => void;
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
  previewDetail: Awaited<ReturnType<typeof getRecordDetail>> | undefined;
  previewLoading: boolean;
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
          recordKey={participant.record_key}
          onReference={onReference}
        />
      )}
      {previewRecordKey && (
        <RecordPreviewPopover
          anchor={previewAnchor}
          detail={previewDetail}
          loading={previewLoading}
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
  onReference: (recordKey: string, anchorRect?: DOMRect) => void;
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
  const surface = activeCurrent.surface ?? participant.surface;
  const slots = {
    header: (
      <ParticipantEditStrip participant={activeCurrent} onUpdate={updateParticipant} />
    ),
    header_actions: (
      <>
        <Button
          aria-label="Participant note"
          icon={<Pencil size={14} />}
          onClick={() => setNoteOpen((open) => !open)}
          size="small"
          type={noteOpen ? "primary" : "default"}
        />
        <ParticipantVariantControl
          participant={activeCurrent}
          onUpdate={updateParticipant}
        />
      </>
    ),
    vitals: (
      <EncounterHpControls current={activeCurrent} onUpdate={updateParticipant} />
    ),
    conditions: (
      <EncounterConditionControls
        conditionDefinitions={conditionDefinitions}
        current={activeCurrent}
        onAddCondition={onAddCondition}
        onReference={onReference}
        onRemoveCondition={onRemoveCondition}
        onUpdateCondition={onUpdateCondition}
        participants={participants}
      />
    ),
    ...(noteOpen
      ? {
          notes: (
            <ParticipantNoteEditor
              participant={activeCurrent}
              onUpdate={updateParticipant}
            />
          ),
        }
      : {}),
  };
  if (!surface)
    return (
      <section aria-label="Participant controls">
        <h2>{activeCurrent.display_name}</h2>
        {activeCurrent.status === "unresolved" && (
          <p>
            Authored record unavailable. Saved participant settings remain editable.
          </p>
        )}
        {slots.header}
        {slots.header_actions}
        {slots.vitals}
        {slots.conditions}
        {slots.notes}
      </section>
    );
  return <RecordSurface onReference={onReference} surface={surface} slots={slots} />;
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
      <Form.Item label="Name" layout="vertical">
        <EditableCommitField
          ariaLabel="Participant name"
          onCommit={(displayName) => onUpdate({ display_name: displayName })}
          value={participant.display_name}
        />
      </Form.Item>
      <Form.Item label="Initiative" layout="vertical">
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
      <Form.Item label="Side" layout="vertical">
        <Select
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
  onReference: (key: string, anchorRect?: DOMRect) => void;
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
          loading={detail.isLoading}
          errors={[detail.error]}
          onReference={onReference}
        />
      </Modal>
    </>
  );
}
