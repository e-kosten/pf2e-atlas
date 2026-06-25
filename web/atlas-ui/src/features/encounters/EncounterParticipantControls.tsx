import { Button, Form, Input, Select } from "antd";
import { useState } from "react";
import type {
  AddEncounterParticipantConditionRequest,
  EncounterConditionDefinitionView,
  EncounterParticipantSideView,
  EncounterParticipantView,
  UpdateEncounterParticipantConditionRequest,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import {
  applyParticipantUpdate,
  optionalBigIntInput,
  participantUpdate,
} from "./participantEdits";
import { EditableCommitField } from "../../shared/ui/forms/EditableCommitField";
import { EncounterConditionControls } from "./EncounterConditionControls";
import { EncounterHpControls } from "./EncounterHpControls";

export function EncounterParticipantControls({
  current,
  participants,
  conditionDefinitions,
  onAddCondition,
  onRemoveCondition,
  onReference,
  onUpdateCondition,
  onUpdate,
}: {
  current: EncounterParticipantView | null;
  participants: EncounterParticipantView[];
  conditionDefinitions: EncounterConditionDefinitionView[];
  onAddCondition: (condition: AddEncounterParticipantConditionRequest) => void;
  onRemoveCondition: (participantKey: string, conditionId: bigint) => void;
  onReference: (recordKey: string, anchorRect?: DOMRect) => void;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => void;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => void;
}) {
  const [projectedCurrent, setProjectedCurrent] = useState<{
    source: EncounterParticipantView | null;
    participant: EncounterParticipantView | null;
  }>({ source: null, participant: null });
  const activeCurrent =
    projectedCurrent.source === current &&
    projectedCurrent.participant?.participant_key === current?.participant_key
      ? projectedCurrent.participant
      : current;
  const updateParticipant = (changes: Partial<UpdateEncounterParticipantRequest>) => {
    if (!activeCurrent) {
      return;
    }
    const request = participantUpdate(activeCurrent, changes);
    setProjectedCurrent({
      source: current,
      participant: applyParticipantUpdate(activeCurrent, request),
    });
    onUpdate(request);
  };

  return (
    <section className="encounter-participant-controls">
      {activeCurrent ? (
        <div
          key={activeCurrent.participant_key}
          className="encounter-participant-controls__body"
        >
          <div className="encounter-form-grid">
            <Form.Item label="Name" layout="vertical">
              <EditableCommitField
                ariaLabel="Participant name"
                onCommit={(displayName) =>
                  updateParticipant({ display_name: displayName })
                }
                value={activeCurrent.display_name}
              />
            </Form.Item>
            <Form.Item label="Initiative" layout="vertical">
              <EditableCommitField
                ariaLabel="Participant initiative"
                inputMode="numeric"
                onCommit={(value) => {
                  const initiative = optionalBigIntInput(value);
                  if (initiative !== null) {
                    updateParticipant({ initiative });
                  }
                }}
                value={inputNumberValue(activeCurrent.initiative)}
              />
            </Form.Item>
          </div>
          <EncounterHpControls current={activeCurrent} onUpdate={updateParticipant} />
          <Form.Item label="Participant note" layout="vertical">
            <Input.TextArea
              key={`note-${activeCurrent.participant_key}-${activeCurrent.note ?? ""}`}
              defaultValue={activeCurrent.note ?? ""}
              onBlur={(event) => {
                const nextNote = event.currentTarget.value;
                if (nextNote !== (activeCurrent.note ?? "")) {
                  updateParticipant({ note: nextNote });
                }
              }}
            />
          </Form.Item>
          <Form.Item label="Side" layout="vertical">
            <Select<EncounterParticipantSideView>
              value={activeCurrent.side}
              onChange={(side) => updateParticipant({ side })}
              options={["pc", "ally", "enemy", "neutral", "hazard"].map((value) => ({
                value: value as EncounterParticipantSideView,
                label: value,
              }))}
            />
          </Form.Item>
          <Button
            onClick={() => updateParticipant({ defeated: !activeCurrent.defeated })}
          >
            {activeCurrent.defeated ? "Mark active" : "Mark defeated"}
          </Button>
          <EncounterConditionControls
            current={activeCurrent}
            onAddCondition={onAddCondition}
            onRemoveCondition={onRemoveCondition}
            onReference={onReference}
            onUpdateCondition={onUpdateCondition}
            participants={participants}
            conditionDefinitions={conditionDefinitions}
          />
        </div>
      ) : (
        <div className="detail-empty">Select a participant to edit.</div>
      )}
    </section>
  );
}

function inputNumberValue(value: bigint | undefined): string {
  return value === undefined ? "" : value.toString();
}
