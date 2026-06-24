import { keepPreviousData, useMutation, useQuery } from "@tanstack/react-query";
import { Button, Form, Input, InputNumber, Modal, Table } from "antd";
import type { ColumnsType } from "antd/es/table";
import { GripVertical, Play, Trash2 } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import {
  addEncounterManualParticipant,
  addEncounterRecordParticipant,
  openResultWindow,
} from "../../api/atlasApi";
import type {
  EncounterParticipantView,
  OpenResultWindowRequest,
  ResultWindowRow,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";

const ENCOUNTER_RECORD_PICKER_DEBOUNCE_MS = 250;

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
  const [recordSearch, setRecordSearch] = useState("");
  const [activeRecordSearch, setActiveRecordSearch] = useState("");
  const [selectedRecordKey, setSelectedRecordKey] = useState<string | null>(null);
  const [recordForm] = Form.useForm<AddRecordForm>();
  const [pcForm] = Form.useForm<AddPcForm>();
  useEffect(() => {
    if (!recordOpen) {
      return;
    }
    const timeout = window.setTimeout(
      () => setActiveRecordSearch(recordSearch),
      ENCOUNTER_RECORD_PICKER_DEBOUNCE_MS,
    );
    return () => window.clearTimeout(timeout);
  }, [recordOpen, recordSearch]);
  const recordResults = useQuery({
    queryKey: ["encounter-record-picker", activeRecordSearch],
    enabled: recordOpen && activeRecordSearch.trim().length > 0,
    placeholderData: keepPreviousData,
    queryFn: () => openResultWindow(encounterRecordPickerRequest(activeRecordSearch)),
  });
  const selectedRecord = useMemo(
    () =>
      recordResults.data?.rows.find(
        (row) => row.record.record_key === selectedRecordKey,
      ) ?? null,
    [recordResults.data?.rows, selectedRecordKey],
  );
  const addRecord = useMutation({
    mutationFn: (values: AddRecordForm) =>
      addEncounterRecordParticipant({
        encounter_ref: slug,
        record_ref: selectedRecordKey ?? "",
        quantity: values.quantity ?? 1,
        ...(values.initiative === undefined
          ? {}
          : { initiative: BigInt(values.initiative) }),
      }),
    onSuccess: () => {
      setRecordOpen(false);
      recordForm.resetFields();
      setRecordSearch("");
      setActiveRecordSearch("");
      setSelectedRecordKey(null);
      onAddComplete();
    },
  });
  const addPc = useMutation({
    mutationFn: (values: AddPcForm) =>
      addEncounterManualParticipant({
        encounter_ref: slug,
        display_name: values.name,
        ...(values.maxHp === undefined ? {} : { max_hp: BigInt(values.maxHp) }),
        ...(values.initiative === undefined
          ? {}
          : { initiative: BigInt(values.initiative) }),
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
      <Input
        aria-label={`${participant.display_name} initiative`}
        className="encounter-roster__initiative-input"
        defaultValue={inputNumberValue(participant.initiative)}
        key={`initiative-${participant.participant_key}-${displayNumber(participant.initiative)}`}
        onBlur={(event) =>
          commitRosterInitiative(participant, event.target.value, onUpdate)
        }
        onClick={(event) => event.stopPropagation()}
        onKeyDown={(event) => {
          event.stopPropagation();
          if (event.key === "Enter") {
            commitRosterInitiative(participant, event.currentTarget.value, onUpdate);
          }
        }}
        size="small"
      />
      <span className="encounter-roster__main">
        <span>{participant.display_name}</span>
        <small>{participant.note_hint ?? participant.side}</small>
      </span>
      <Input
        aria-label={`${participant.display_name} current HP`}
        className="encounter-roster__hp-input"
        defaultValue={inputNumberValue(participant.current_hp)}
        key={`hp-${participant.participant_key}-${displayNumber(participant.current_hp)}`}
        onBlur={(event) => commitRosterHp(participant, event.target.value, onUpdate)}
        onClick={(event) => event.stopPropagation()}
        onKeyDown={(event) => {
          event.stopPropagation();
          if (event.key === "Enter") {
            commitRosterHp(participant, event.currentTarget.value, onUpdate);
          }
        }}
        placeholder={participant.max_hp === undefined ? "" : "HP"}
        size="small"
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
          <p>{participants.length} participants</p>
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
      <Modal
        title="Add creature or hazard"
        open={recordOpen}
        onCancel={() => {
          setRecordOpen(false);
          setRecordSearch("");
          setActiveRecordSearch("");
          setSelectedRecordKey(null);
        }}
        onOk={() => recordForm.submit()}
        okButtonProps={{ disabled: !selectedRecordKey }}
      >
        <Form form={recordForm} layout="vertical" onFinish={(v) => addRecord.mutate(v)}>
          <Form.Item label="Search" layout="vertical">
            <Input
              allowClear
              aria-label="Search"
              value={recordSearch}
              onChange={(event) => {
                setRecordSearch(event.target.value);
                setSelectedRecordKey(null);
              }}
            />
          </Form.Item>
          <Table
            columns={encounterRecordPickerColumns((recordKey) =>
              setSelectedRecordKey(recordKey),
            )}
            dataSource={
              activeRecordSearch.trim().length > 0
                ? (recordResults.data?.rows ?? [])
                : []
            }
            loading={recordResults.isLoading || recordResults.isFetching}
            locale={{
              emptyText:
                activeRecordSearch.trim().length > 0
                  ? "No creatures or hazards"
                  : "Search for a creature or hazard",
            }}
            pagination={false}
            rowClassName={(row) =>
              row.record.record_key === selectedRecordKey
                ? "encounter-record-picker__row encounter-record-picker__row--selected"
                : "encounter-record-picker__row"
            }
            onRow={(row) => ({
              onClick: () => setSelectedRecordKey(row.record.record_key),
            })}
            rowKey={(row) => row.record.record_key}
            scroll={{ y: 280 }}
            size="small"
          />
          <p className="encounter-record-picker__selection">
            {selectedRecord
              ? `Selected: ${selectedRecord.record.title}`
              : "Select a creature or hazard."}
          </p>
          <Form.Item name="quantity" label="Quantity">
            <InputNumber aria-label="Quantity" min={1} max={50} />
          </Form.Item>
          <Form.Item name="initiative" label="Initiative">
            <InputNumber aria-label="Initiative" />
          </Form.Item>
        </Form>
      </Modal>
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

function encounterRecordPickerColumns(
  onSelectRecord: (recordKey: string) => void,
): ColumnsType<ResultWindowRow> {
  return [
    {
      title: "Name",
      dataIndex: ["record", "title"],
      render: (_, row) => (
        <span
          className="encounter-record-picker__title"
          onClick={(event) => {
            event.stopPropagation();
            onSelectRecord(row.record.record_key);
          }}
          onKeyDown={(event) => {
            if (event.key === "Enter" || event.key === " ") {
              event.preventDefault();
              onSelectRecord(row.record.record_key);
            }
          }}
          role="button"
          tabIndex={0}
        >
          <span>{row.record.title}</span>
          {row.record.preview ? <small>{row.record.preview}</small> : null}
        </span>
      ),
    },
    {
      title: "Kind",
      dataIndex: ["record", "kind_label"],
      width: 100,
    },
    {
      title: "Level",
      dataIndex: ["record", "level_label"],
      width: 90,
      render: (value) => value ?? "",
    },
  ];
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

function participantUpdate(
  participant: EncounterParticipantView,
  changes: Partial<UpdateEncounterParticipantRequest>,
): UpdateEncounterParticipantRequest {
  return {
    participant_key: participant.participant_key,
    display_name: participant.display_name,
    side: participant.side,
    participant_variant: participant.participant_variant,
    initiative: participant.initiative,
    max_hp: participant.max_hp,
    current_hp: participant.current_hp,
    temporary_hp: participant.temporary_hp,
    defeated: participant.defeated,
    hidden: participant.hidden,
    note: participant.note,
    ...changes,
  };
}
function commitRosterInitiative(
  participant: EncounterParticipantView,
  value: string,
  onUpdate: (participant: UpdateEncounterParticipantRequest) => void,
) {
  const initiative = optionalBigIntInput(value);
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
  onUpdate(
    participantUpdate(participant, {
      current_hp: hp,
      defeated: hp === BigInt(0) ? true : participant.defeated,
    }),
  );
}
function evaluateHpFormula(value: string): number | null {
  const trimmed = value.trim();
  if (!/^\d+(\s*[+-]\s*\d+)*$/.test(trimmed)) {
    return null;
  }
  const tokens = trimmed.match(/\d+|[+-]/g);
  if (!tokens || tokens.length === 0) {
    return null;
  }
  let result = Number(tokens[0]);
  for (let index = 1; index < tokens.length; index += 2) {
    const operator = tokens[index];
    const next = Number(tokens[index + 1]);
    result = operator === "-" ? result - next : result + next;
  }
  return Math.max(0, result);
}
function optionalBigIntInput(value: string): bigint | undefined | null {
  const trimmed = value.trim();
  if (trimmed.length === 0) {
    return undefined;
  }
  if (!/^-?\d+$/.test(trimmed)) {
    return null;
  }
  return BigInt(trimmed);
}
function optionalHpFormulaInput(value: string): bigint | undefined | null {
  const trimmed = value.trim();
  if (trimmed.length === 0) {
    return undefined;
  }
  const hp = evaluateHpFormula(trimmed);
  return hp === null ? null : BigInt(hp);
}
function inputNumberValue(value: bigint | undefined): string {
  return value === undefined ? "" : value.toString();
}
function displayNumber(value: bigint | undefined): string {
  return value === undefined ? "--" : value.toString();
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
