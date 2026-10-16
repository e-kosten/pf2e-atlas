import { Alert, Button, Dropdown, Form, Input, InputNumber, Modal, Tag } from "antd";
import { MoreHorizontal, Play } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import type {
  AddEncounterManualParticipantRequest,
  AddEncounterRecordParticipantRequest,
  EncounterParticipantView,
  SearchResultsRequest,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import { PrimaryToolbar } from "../../shared/ui/PrimaryToolbar";
import { EditableCommitField } from "../../shared/ui/forms/EditableCommitField";
import { SearchPickerModal } from "../../shared/ui/pickers/SearchPickerModal";
import { optionalBigIntInput, participantUpdate } from "./participantEdits";

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
  recovery,
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
  onAddRecord: (request: AddEncounterRecordParticipantRequest) => Promise<boolean>;
  onAddManual: (request: AddEncounterManualParticipantRequest) => Promise<boolean>;
  disabled: boolean;
  error?: string;
  recovery?: React.ReactNode;
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
  const formId = useId();
  const [added, setAdded] = useState("");
  const [candidateName, setCandidateName] = useState("");
  const [committedAdd, setCommittedAdd] = useState<string | null>(null);
  const [committedPc, setCommittedPc] = useState(false);
  const pendingFormReset = useRef({ record: false, pc: false });
  const [recordOpen, setRecordOpen] = useState(false);
  const [pcOpen, setPcOpen] = useState(false);
  const [selectedRecordKey, setSelectedRecordKey] = useState<string | null>(null);
  const [recordForm] = Form.useForm<AddRecordForm>();
  const [pcForm] = Form.useForm<AddPcForm>();
  useEffect(() => {
    if (disabled) return;
    if (pendingFormReset.current.record) {
      recordForm.resetFields();
      pendingFormReset.current.record = false;
    }
    if (pendingFormReset.current.pc) {
      pcForm.resetFields();
      pendingFormReset.current.pc = false;
    }
  }, [disabled, committedAdd, committedPc, recordForm, pcForm]);
  const addRecord = async (values: AddRecordForm) => {
    if (disabled) return;
    setCommittedAdd(null);
    setAdded("");
    try {
      const saved = await onAddRecord({
        encounter_ref: slug,
        record_ref: selectedRecordKey ?? "",
        quantity: values.quantity ?? 1,
        ...(values.initiative === undefined
          ? {}
          : { initiative: BigInt(values.initiative) }),
      });
      if (saved) {
        setAdded(`Added ${values.quantity ?? 1} × ${candidateName}.`);
        recordForm.resetFields();
      } else {
        pendingFormReset.current.record = true;
        setCommittedAdd(`Added ${values.quantity ?? 1} × ${candidateName}.`);
      }
    } catch {
      /* The owner displays the failure; retain the form and selection. */
    }
  };
  const addPc = async (values: AddPcForm) => {
    if (disabled) return;
    try {
      const saved = await onAddManual({
        encounter_ref: slug,
        display_name: values.name,
        ...(values.maxHp === undefined ? {} : { max_hp: BigInt(values.maxHp) }),
        ...(values.initiative === undefined
          ? {}
          : { initiative: BigInt(values.initiative) }),
      });
      if (saved) {
        setPcOpen(false);
        pcForm.resetFields();
      } else {
        pendingFormReset.current.pc = true;
        setCommittedPc(true);
      }
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
  const renderParticipant = (participant: EncounterParticipantView) => {
    const index = participants.findIndex(
      (p) => p.participant_key === participant.participant_key,
    );
    const move = (offset: number) => {
      const target = participants[index + offset];
      if (target)
        onReorder(
          participant.participant_key,
          target.participant_key,
          offset < 0 ? "before" : "after",
        );
    };
    return (
      <div
        className={rosterRowClass(
          participant,
          participant.participant_key === currentTurnParticipantKey,
          participant.participant_key === selectedParticipantKey,
        )}
        key={participant.participant_key}
      >
        <EditableCommitField
          disabled={disabled}
          ariaLabel={`${participant.display_name} initiative`}
          className="encounter-roster__initiative-input"
          inputMode="numeric"
          size="small"
          onCommit={(value) => commitRosterInitiative(participant, value, onUpdate)}
          value={inputNumberValue(participant.initiative)}
        />
        <Button
          type="text"
          className="encounter-roster__select"
          data-participant-key={participant.participant_key}
          aria-pressed={participant.participant_key === selectedParticipantKey}
          onClick={() => onSelect(participant.participant_key)}
        >
          <span className="encounter-roster__name">{participant.display_name}</span>
          {participant.participant_key === currentTurnParticipantKey && (
            <Tag color="processing">Current turn</Tag>
          )}
          {participant.defeated && (
            <span className="encounter-roster__state">Defeated</span>
          )}
        </Button>
        <span
          className="encounter-roster__hp"
          aria-label={`${participant.display_name} HP`}
        >
          {inputNumberValue(participant.current_hp) || "—"}/
          {inputNumberValue(participant.max_hp) || "—"}
          {participant.temporary_hp > 0n && (
            <small>+{participant.temporary_hp.toString()} temp</small>
          )}
        </span>
        <Dropdown
          trigger={["click"]}
          disabled={disabled}
          menu={{
            items: [
              { key: "turn", label: "Set turn", onClick: () => onSetTurn(participant) },
              {
                key: "up",
                label: `Move before ${participants[index - 1]?.display_name ?? "previous participant"}${participants[index - 1]?.initiative === undefined ? " (unset initiative)" : ` (initiative ${participants[index - 1].initiative})`}`,
                disabled: index === 0,
                onClick: () => move(-1),
              },
              {
                key: "down",
                label: `Move after ${participants[index + 1]?.display_name ?? "next participant"}${participants[index + 1]?.initiative === undefined ? " (unset initiative)" : ` (initiative ${participants[index + 1].initiative})`}`,
                disabled: index === participants.length - 1,
                onClick: () => move(1),
              },
              { type: "divider" },
              {
                key: "remove",
                label: "Remove",
                danger: true,
                onClick: () => onRemove(participant),
              },
            ],
          }}
        >
          <Button
            size="small"
            type="text"
            aria-label={`Actions for ${participant.display_name}`}
            icon={<MoreHorizontal size={16} aria-hidden="true" />}
          />
        </Dropdown>
      </div>
    );
  };

  return (
    <section className="encounter-pane encounter-roster">
      <header className="encounter-pane__header">
        <PrimaryToolbar className="encounter-actions">
          <Button
            disabled={disabled}
            icon={<Play size={14} />}
            type="primary"
            onClick={onAdvanceTurn}
          >
            {currentTurnParticipantKey ? "Next turn" : "Start turn"}
          </Button>
          <Button
            disabled={disabled}
            onClick={() => {
              setAdded("");
              setRecordOpen(true);
            }}
          >
            Add Creature
          </Button>
          <Button
            disabled={disabled}
            onClick={() => {
              setCommittedPc(false);
              setPcOpen(true);
            }}
          >
            Add PC
          </Button>
        </PrimaryToolbar>
      </header>
      {loading ? (
        <div className="detail-empty">Loading encounter...</div>
      ) : participants.length === 0 ? (
        <div className="detail-empty">Add a creature, hazard, or PC.</div>
      ) : (
        <div className="encounter-roster__list">
          <div className="encounter-roster__columns" aria-hidden="true">
            <span>Init</span>
            <span>Participant</span>
            <span>HP</span>
            <span />
          </div>
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
        dismissLabel="Done"
        onSelectedRecordNameChange={setCandidateName}
        status={
          added || (!disabled && committedAdd) ? (
            <span role="status" className="encounter-add-receipt">
              {committedAdd ?? added}
            </span>
          ) : undefined
        }
      >
        {recovery}
        {error && (
          <Alert
            type="error"
            title="Participant could not be added"
            description={error}
          />
        )}
        <Form.Item name="quantity" label="Quantity" initialValue={1}>
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
        open={pcOpen && !(committedPc && !disabled)}
        onCancel={() => setPcOpen(false)}
        onOk={() => pcForm.submit()}
      >
        <Form
          name={formId}
          disabled={disabled}
          form={pcForm}
          layout="vertical"
          onFinish={(v) => void addPc(v)}
        >
          {recovery}
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
function inputNumberValue(value: bigint | undefined): string {
  return value === undefined ? "" : value.toString();
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
