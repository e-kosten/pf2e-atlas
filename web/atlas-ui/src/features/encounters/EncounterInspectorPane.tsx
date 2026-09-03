import { Button, Form, Input, Select, Tag } from "antd";
import { useState } from "react";
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
  optionalIntegerInput,
  participantUpdate,
} from "./participantEdits";
import { RecordPreviewScope } from "../../shared/records/RecordPreviewScope";
import { RecordSurface } from "../../shared/records/RecordSurface";
import { EditableCommitField } from "../../shared/ui/forms/EditableCommitField";
import { EncounterConditionControls } from "./EncounterConditionControls";
import { EncounterHpControls } from "./EncounterHpControls";

export function EncounterInspectorPane({
  onOpenRecordFullPage,
  onAddCondition,
  onRemoveCondition,
  onUpdateCondition,
  onUpdate,
  participant,
  participants,
  conditionDefinitions,
  currentTurnParticipantKey,
}: {
  onOpenRecordFullPage: (recordKey: string) => void;
  onAddCondition: (condition: AddEncounterParticipantConditionRequest) => void;
  onRemoveCondition: (participantKey: string, conditionId: number) => void;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => void;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => void;
  participant: EncounterParticipantView | undefined;
  participants: EncounterParticipantView[];
  conditionDefinitions: EncounterConditionDefinitionView[];
  currentTurnParticipantKey: string | null;
}) {
  if (!participant) {
    return (
      <section className="encounter-pane detail-empty">Select a participant.</section>
    );
  }
  const surface = participant.record_view;
  return (
    <section className="encounter-pane encounter-record-pane">
      {surface ? (
        <RecordPreviewScope onOpenFullPage={onOpenRecordFullPage}>
          <EncounterParticipantSurface
            conditionDefinitions={conditionDefinitions}
            currentTurnParticipantKey={currentTurnParticipantKey}
            onAddCondition={onAddCondition}
            onReference={onOpenRecordFullPage}
            onRemoveCondition={onRemoveCondition}
            onUpdate={onUpdate}
            onUpdateCondition={onUpdateCondition}
            participant={participant}
            participants={participants}
          />
        </RecordPreviewScope>
      ) : (
        <SurfaceUnavailable participant={participant} />
      )}
    </section>
  );
}

function SurfaceUnavailable({
  participant,
}: {
  participant: EncounterParticipantView;
}) {
  return (
    <section className="encounter-surface-unavailable">
      <p className="eyebrow">{participantKindLabel(participant.participant_kind)}</p>
      <h2>{participant.display_name}</h2>
      <p>
        This participant does not have a composed record surface yet. This is a
        projection gap rather than a fallback UI.
      </p>
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
  currentTurnParticipantKey,
}: {
  conditionDefinitions: EncounterConditionDefinitionView[];
  onAddCondition: (condition: AddEncounterParticipantConditionRequest) => void;
  onReference: (recordKey: string) => void;
  onRemoveCondition: (participantKey: string, conditionId: number) => void;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => void;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => void;
  participant: EncounterParticipantView;
  participants: EncounterParticipantView[];
  currentTurnParticipantKey: string | null;
}) {
  const [projectedCurrent, setProjectedCurrent] = useState<{
    source: EncounterParticipantView | null;
    participant: EncounterParticipantView | null;
  }>({ source: null, participant: null });
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
  const surface = activeCurrent.record_view ?? participant.record_view;
  if (!surface) {
    return null;
  }
  return (
    <RecordSurface
      onReference={onReference}
      surface={surface}
      slots={{
        header: (
          <ParticipantEditStrip
            currentTurn={activeCurrent.participant_key === currentTurnParticipantKey}
            participant={activeCurrent}
            onUpdate={updateParticipant}
          />
        ),
        header_actions: (
          <ParticipantVariantControl
            participant={activeCurrent}
            onUpdate={updateParticipant}
          />
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
        notes: (
          <ParticipantNoteEditor
            participant={activeCurrent}
            onUpdate={updateParticipant}
          />
        ),
      }}
    />
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
  currentTurn,
  onUpdate,
  participant,
}: {
  currentTurn: boolean;
  onUpdate: (changes: Partial<UpdateEncounterParticipantRequest>) => void;
  participant: EncounterParticipantView;
}) {
  return (
    <section className="encounter-participant-strip">
      <div className="encounter-participant-strip__state">
        <Tag>{currentTurn ? "Current turn" : "Not current turn"}</Tag>
        {participant.defeated && <Tag>Defeated</Tag>}
      </div>
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
            const initiative = optionalIntegerInput(value);
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
      aria-label="Participant note"
      autoSize={{ minRows: 3, maxRows: 10 }}
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

function participantKindLabel(
  kind: EncounterParticipantView["participant_kind"],
): string {
  if (kind === "pc") {
    return "PC";
  }
  return kind.charAt(0).toUpperCase() + kind.slice(1);
}
