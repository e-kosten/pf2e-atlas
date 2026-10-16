import { Alert, Button, Form, Input, InputNumber, Modal, Tag } from "antd";
import { GripVertical, Play, Trash2 } from "lucide-react";
import { useState } from "react";
import type {
  AddEncounterManualParticipantRequest,
  AddEncounterRecordParticipantRequest,
  EncounterParticipantView,
  SearchResultsRequest,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import { EditableCommitField } from "../../shared/ui/forms/EditableCommitField";
import { SearchPickerModal } from "../../shared/ui/pickers/SearchPickerModal";
import {
  clampCurrentHp,
  optionalBigIntInput,
  optionalHpFormulaInput,
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
  onAddRecord,
  onAddManual,
  disabled,
  error,
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
  onAddRecord: (request: AddEncounterRecordParticipantRequest) => Promise<unknown>;
  onAddManual: (request: AddEncounterManualParticipantRequest) => Promise<unknown>;
  disabled: boolean;
  error?: string;
  onAdvanceTurn: () => void;
  onRemove: (participant: EncounterParticipantView) => void;
  onReorder: (
    participantKey: string,
    targetParticipantKey: string,
    placement: "before" | "after",
  ) => void;
  onSelect: (participantKey: string) => void;
  onSetTurn: (participant: EncounterParticipantView) => void;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => Promise<unknown>;
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
  const addRecord = async (values: AddRecordForm) => {
    if (disabled) return;
    try {
      await onAddRecord({
        encounter_ref: slug,
        record_ref: selectedRecordKey ?? "",
        quantity: values.quantity ?? 1,
        ...(values.initiative === undefined
          ? {}
          : { initiative: BigInt(values.initiative) }),
      });
      setRecordOpen(false);
      recordForm.resetFields();
      setSelectedRecordKey(null);
    } catch {
      /* The owner displays the failure; retain the form and selection. */
    }
  };
  const addPc = async (values: AddPcForm) => {
    if (disabled) return;
    try {
      await onAddManual({
        encounter_ref: slug,
        display_name: values.name,
        ...(values.maxHp === undefined ? {} : { max_hp: BigInt(values.maxHp) }),
        ...(values.initiative === undefined
          ? {}
          : { initiative: BigInt(values.initiative) }),
      });
      setPcOpen(false);
      pcForm.resetFields();
    } catch {
      /* The owner displays the failure; retain this draft. */
    }
  };
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
      draggable={!disabled}
      onDragOver={(event) => event.preventDefault()}
      onDragStart={() => {
        if (!disabled) setDraggingParticipantKey(participant.participant_key);
      }}
      onDragEnd={() => setDraggingParticipantKey(null)}
      onDrop={(event) => {
        event.preventDefault();
        if (
          disabled ||
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
      <div className="encounter-roster__initiative">
        <span>Initiative</span>
        <EditableCommitField
          disabled={disabled}
          ariaLabel={`${participant.display_name} initiative`}
          className="encounter-roster__initiative-input"
          inputMode="numeric"
          onCommit={(value) => commitRosterInitiative(participant, value, onUpdate)}
          size="small"
          stopPropagation
          value={inputNumberValue(participant.initiative)}
        />
      </div>
      <span className="encounter-roster__main">
        <span className="encounter-roster__name">{participant.display_name}</span>
        {participant.participant_key === currentTurnParticipantKey && (
          <Tag color="processing">Current turn</Tag>
        )}
        <small>{participant.note_hint ?? participant.side}</small>
      </span>
      <div className="encounter-roster__hp">
        <span>HP</span>
        <EditableCommitField
          disabled={disabled}
          ariaLabel={`${participant.display_name} current HP`}
          className="encounter-roster__hp-input"
          inputMode="numeric"
          onCommit={(value) => commitRosterHp(participant, value, onUpdate)}
          placeholder={participant.max_hp === undefined ? "" : "HP"}
          size="small"
          stopPropagation
          value={inputNumberValue(participant.current_hp)}
        />
      </div>
      <span className="encounter-roster__row-actions">
        <Button
          disabled={disabled}
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
          disabled={disabled}
          danger
          aria-label={`Remove ${participant.display_name}`}
          icon={<Trash2 size={12} aria-hidden="true" />}
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
            disabled={disabled}
            icon={<Play size={14} />}
            size="small"
            type="primary"
            onClick={onAdvanceTurn}
          >
            {currentTurnParticipantKey ? "Next" : "Play"}
          </Button>
          <Button disabled={disabled} size="small" onClick={() => setRecordOpen(true)}>
            Add Creature
          </Button>
          <Button disabled={disabled} size="small" onClick={() => setPcOpen(true)}>
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
        disabled={disabled}
        okButtonProps={{ disabled: disabled || !selectedRecordKey, loading: disabled }}
        onCancel={() => setRecordOpen(false)}
        onFinish={(values) => void addRecord(values)}
        onSelectedRecordKeyChange={setSelectedRecordKey}
        open={recordOpen}
        selectedRecordKey={selectedRecordKey}
        title="Add creature or hazard"
      >
        {error && (
          <Alert
            type="error"
            title="Participant could not be added"
            description={error}
          />
        )}
        <Form.Item name="quantity" label="Quantity">
          <InputNumber aria-label="Quantity" min={1} max={50} />
        </Form.Item>
        <Form.Item name="initiative" label="Initiative">
          <InputNumber aria-label="Initiative" />
        </Form.Item>
      </SearchPickerModal>
      <Modal
        title="Add PC"
        confirmLoading={disabled}
        cancelButtonProps={{ disabled }}
        closable={!disabled}
        keyboard={!disabled}
        mask={{ closable: !disabled }}
        open={pcOpen}
        onCancel={() => setPcOpen(false)}
        onOk={() => pcForm.submit()}
      >
        <Form
          disabled={disabled}
          form={pcForm}
          layout="vertical"
          onFinish={(v) => void addPc(v)}
        >
          {error && (
            <Alert
              type="error"
              title="Participant could not be added"
              description={error}
            />
          )}
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

function encounterRecordPickerRequest(query: string): SearchResultsRequest {
  const filter = {
    kind: "in" as const,
    field: "record.kind",
    values: ["creature", "hazard"],
  };
  const trimmed = query.trim();
  return {
    expected_artifact_generation: null,
    criteria:
      trimmed.length > 0
        ? {
            kind: "text_search",
            query: trimmed,
            filter,
            mode: "hybrid",
          }
        : {
            kind: "list_records",
            filter,
          },
    page: { number: 1, size: 25 },
  };
}

function commitRosterInitiative(
  participant: EncounterParticipantView,
  value: string,
  onUpdate: (participant: UpdateEncounterParticipantRequest) => Promise<unknown>,
) {
  const initiative = optionalBigIntInput(value);
  if (initiative === null) {
    return;
  }
  return onUpdate(participantUpdate(participant, { initiative }));
}
function commitRosterHp(
  participant: EncounterParticipantView,
  value: string,
  onUpdate: (participant: UpdateEncounterParticipantRequest) => Promise<unknown>,
) {
  const hp = optionalHpFormulaInput(value);
  if (hp === null) {
    return;
  }
  const clampedHp =
    hp === undefined ? undefined : BigInt(clampCurrentHp(participant, Number(hp)));
  return onUpdate(
    participantUpdate(participant, {
      current_hp: clampedHp,
      defeated: clampedHp === BigInt(0) ? true : participant.defeated,
    }),
  );
}
function inputNumberValue(value: bigint | undefined): string {
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
