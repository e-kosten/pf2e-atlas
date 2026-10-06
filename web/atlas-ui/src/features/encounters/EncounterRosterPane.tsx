import { useMutation } from "@tanstack/react-query";
import { Button, Form, Input, InputNumber, Modal } from "antd";
import { GripVertical, Play, Trash2 } from "lucide-react";
import { useState } from "react";
import {
  addEncounterManualParticipant,
  addEncounterRecordParticipant,
} from "../../api/atlasApi";
import type {
  EncounterParticipantView,
  OpenResultWindowRequest,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import { EditableCommitField } from "../../shared/ui/forms/EditableCommitField";
import { SearchPickerModal } from "../../shared/ui/pickers/SearchPickerModal";
import {
  clampCurrentHp,
  optionalIntegerInput,
  optionalHpFormulaInput,
  participantCurrentHp,
  participantMaximumHp,
  participantUpdate,
} from "./participantEdits";

type AddRecordForm = {
  quantity?: number;
  initiative?: number;
};

type AddPcForm = {
  name: string;
  maxHp?: number;
  initiative?: number;
};

export function EncounterRosterPane({
  currentTurnParticipantKey,
  loading,
  onAddComplete,
  onAdvanceTurn,
  onRemove,
  onReorder,
  onSelect,
  onSetTurn,
  onUpdate,
  participants,
  selectedParticipantKey,
  slug,
}: {
  currentTurnParticipantKey: string | null;
  loading: boolean;
  onAddComplete: () => void;
  onAdvanceTurn: () => void;
  onRemove: (participant: EncounterParticipantView) => void;
  onReorder: (
    participantKey: string,
    targetParticipantKey: string,
    placement: "before" | "after",
  ) => void;
  onSelect: (participantKey: string) => void;
  onSetTurn: (participant: EncounterParticipantView) => void;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => void;
  participants: EncounterParticipantView[];
  selectedParticipantKey: string | null;
  slug: string;
}) {
  const [recordOpen, setRecordOpen] = useState(false);
  const [pcOpen, setPcOpen] = useState(false);
  const [draggingParticipantKey, setDraggingParticipantKey] = useState<string | null>(
    null,
  );
  const [selectedRecordKey, setSelectedRecordKey] = useState<string | null>(null);
  const [recordForm] = Form.useForm<AddRecordForm>();
  const [pcForm] = Form.useForm<AddPcForm>();
  const addRecord = useMutation({
    mutationFn: (values: AddRecordForm) =>
      addEncounterRecordParticipant({
        encounter_ref: slug,
        record_ref: selectedRecordKey ?? "",
        quantity: values.quantity ?? 1,
        ...(values.initiative === undefined ? {} : { initiative: values.initiative }),
      }),
    onSuccess: () => {
      setRecordOpen(false);
      recordForm.resetFields();
      setSelectedRecordKey(null);
      onAddComplete();
    },
  });
  const addPc = useMutation({
    mutationFn: (values: AddPcForm) =>
      addEncounterManualParticipant({
        encounter_ref: slug,
        display_name: values.name,
        ...(values.maxHp === undefined ? {} : { max_hp: values.maxHp }),
        ...(values.initiative === undefined ? {} : { initiative: values.initiative }),
      }),
    onSuccess: () => {
      setPcOpen(false);
      pcForm.resetFields();
      onAddComplete();
    },
  });
  const setInitiativeParticipants = participants.filter(
    (participant) => participant.initiative !== undefined,
  );
  const unsetInitiativeParticipants = participants.filter(
    (participant) => participant.initiative === undefined,
  );
  const renderParticipant = (participant: EncounterParticipantView) => (
    <div
      className={rosterRowClass(
        participant,
        participant.participant_key === currentTurnParticipantKey,
        participant.participant_key === selectedParticipantKey,
      )}
      key={participant.participant_key}
      draggable
      onDragOver={(event) => event.preventDefault()}
      onDragStart={() => setDraggingParticipantKey(participant.participant_key)}
      onDragEnd={() => setDraggingParticipantKey(null)}
      onDrop={(event) => {
        event.preventDefault();
        if (
          !draggingParticipantKey ||
          draggingParticipantKey === participant.participant_key
        ) {
          return;
        }
        const bounds = event.currentTarget.getBoundingClientRect();
        const placement =
          event.clientY < bounds.top + bounds.height / 2 ? "before" : "after";
        onReorder(draggingParticipantKey, participant.participant_key, placement);
        setDraggingParticipantKey(null);
      }}
      onClick={() => onSelect(participant.participant_key)}
      onKeyDown={(event) => {
        if (isInteractiveEventTarget(event.target)) {
          return;
        }
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onSelect(participant.participant_key);
        }
      }}
      role="button"
      tabIndex={0}
    >
      <span className="encounter-roster__drag-handle" aria-hidden="true">
        <GripVertical size={16} />
      </span>
      <EditableCommitField
        ariaLabel={`${participant.display_name} initiative`}
        className="encounter-roster__initiative-input"
        inputMode="numeric"
        onCommit={(value) => commitRosterInitiative(participant, value, onUpdate)}
        size="small"
        stopPropagation
        value={inputNumberValue(participant.initiative)}
      />
      <span className="encounter-roster__main">
        <span>{participant.display_name}</span>
        <small>{participant.note_hint ?? participant.side}</small>
      </span>
      <EditableCommitField
        ariaLabel={`${participant.display_name} current HP`}
        className="encounter-roster__hp-input"
        inputMode="numeric"
        onCommit={(value) => commitRosterHp(participant, value, onUpdate)}
        placeholder={participantMaximumHp(participant) === undefined ? "" : "HP"}
        size="small"
        stopPropagation
        value={inputNumberValue(participantCurrentHp(participant))}
      />
      <span className="encounter-roster__row-actions">
        <Button
          aria-label={`Set turn to ${participant.display_name}`}
          size="small"
          onClick={(event) => {
            event.stopPropagation();
            onSetTurn(participant);
          }}
        >
          Turn
        </Button>
        <Button
          danger
          aria-label={`Remove ${participant.display_name}`}
          icon={<Trash2 size={12} />}
          size="small"
          onClick={(event) => {
            event.stopPropagation();
            onRemove(participant);
          }}
        />
      </span>
    </div>
  );

  return (
    <section className="encounter-pane encounter-roster">
      <header className="encounter-pane__header">
        <div>
          <h2>Initiative</h2>
        </div>
        <div className="encounter-actions">
          <Button
            icon={<Play size={14} />}
            size="small"
            type="primary"
            onClick={onAdvanceTurn}
          >
            {currentTurnParticipantKey ? "Next" : "Play"}
          </Button>
          <Button size="small" onClick={() => setRecordOpen(true)}>
            Add Creature
          </Button>
          <Button size="small" onClick={() => setPcOpen(true)}>
            Add PC
          </Button>
        </div>
      </header>
      {loading ? (
        <div className="detail-empty">Loading encounter...</div>
      ) : participants.length === 0 ? (
        <div className="detail-empty">Add a creature, hazard, or PC.</div>
      ) : (
        <div className="encounter-roster__list">
          {setInitiativeParticipants.length > 0 && (
            <div className="encounter-roster__group">
              <span className="encounter-roster__group-label">Initiative set</span>
              {setInitiativeParticipants.map(renderParticipant)}
            </div>
          )}
          {unsetInitiativeParticipants.length > 0 && (
            <div className="encounter-roster__group">
              <span className="encounter-roster__group-label">Unset initiative</span>
              {unsetInitiativeParticipants.map(renderParticipant)}
            </div>
          )}
        </div>
      )}
      <SearchPickerModal<AddRecordForm>
        buildRequest={encounterRecordPickerRequest}
        emptyPrompt="Search for a creature or hazard"
        emptyResults="No creatures or hazards"
        form={recordForm}
        okButtonProps={{ disabled: !selectedRecordKey }}
        onCancel={() => setRecordOpen(false)}
        onFinish={(values) => addRecord.mutate(values)}
        onSelectedRecordKeyChange={setSelectedRecordKey}
        open={recordOpen}
        selectedLabel={(row) => `Selected: ${row.record.surface.metadata.title}`}
        selectedPrompt="Select a creature or hazard."
        selectedRecordKey={selectedRecordKey}
        title="Add creature or hazard"
      >
        <Form.Item name="quantity" label="Quantity">
          <InputNumber aria-label="Quantity" min={1} max={50} />
        </Form.Item>
        <Form.Item name="initiative" label="Initiative">
          <InputNumber aria-label="Initiative" />
        </Form.Item>
      </SearchPickerModal>
      <Modal
        title="Add PC"
        open={pcOpen}
        onCancel={() => setPcOpen(false)}
        onOk={() => pcForm.submit()}
      >
        <Form form={pcForm} layout="vertical" onFinish={(v) => addPc.mutate(v)}>
          <Form.Item name="name" label="Name" rules={[{ required: true }]}>
            <Input />
          </Form.Item>
          <Form.Item name="maxHp" label="Max HP">
            <InputNumber aria-label="Max HP" min={0} />
          </Form.Item>
          <Form.Item name="initiative" label="Initiative">
            <InputNumber aria-label="Initiative" />
          </Form.Item>
        </Form>
      </Modal>
    </section>
  );
}

function encounterRecordPickerRequest(query: string): OpenResultWindowRequest {
  const filter = {
    clauses: [
      {
        id: "encounter-participant-kind",
        field: "kind",
        operator: "include_any" as const,
        values: ["creature", "hazard"],
      },
    ],
  };
  const trimmed = query.trim();
  return {
    mode:
      trimmed.length > 0
        ? {
            kind: "text_search",
            query: trimmed,
            filter,
          }
        : {
            kind: "list_records",
            filter,
            sort: { kind: "alphabetical" },
          },
    page: { number: 1, size: 25 },
    include_diagnostics: false,
  };
}

function commitRosterInitiative(
  participant: EncounterParticipantView,
  value: string,
  onUpdate: (participant: UpdateEncounterParticipantRequest) => void,
) {
  const initiative = optionalIntegerInput(value);
  if (initiative === null) {
    return;
  }
  onUpdate(participantUpdate(participant, { initiative }));
}
function commitRosterHp(
  participant: EncounterParticipantView,
  value: string,
  onUpdate: (participant: UpdateEncounterParticipantRequest) => void,
) {
  const hp = optionalHpFormulaInput(value);
  if (hp === null) {
    return;
  }
  const clampedHp = hp === undefined ? undefined : clampCurrentHp(participant, hp);
  onUpdate(
    participantUpdate(participant, {
      current_hp: clampedHp,
      defeated: clampedHp === 0 ? true : participant.defeated,
    }),
  );
}
function inputNumberValue(value: number | undefined): string {
  return value === undefined ? "" : value.toString();
}
function isInteractiveEventTarget(target: EventTarget): boolean {
  return (
    target instanceof HTMLElement &&
    Boolean(target.closest("button,input,textarea,select,a"))
  );
}
function rosterRowClass(
  participant: EncounterParticipantView,
  current: boolean,
  selected: boolean,
): string {
  return [
    "encounter-roster__row",
    current ? "encounter-roster__row--current" : "",
    selected ? "encounter-roster__row--selected" : "",
    participant.defeated ? "encounter-roster__row--defeated" : "",
  ]
    .filter(Boolean)
    .join(" ");
}
