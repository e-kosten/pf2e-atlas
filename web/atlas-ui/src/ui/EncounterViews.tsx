import { Edit2, Play, Plus, Trash2 } from "lucide-react";
import {
  keepPreviousData,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { Button, Checkbox, Form, Input, InputNumber, Modal, Select, Table } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useEffect, useMemo, useState } from "react";
import {
  addEncounterManualParticipant,
  addEncounterParticipantCondition,
  addEncounterRecordParticipant,
  createEncounter,
  deleteEncounter,
  getEncounter,
  getEncounters,
  getRecordDetail,
  openResultWindow,
  removeEncounterParticipant,
  removeEncounterParticipantCondition,
  reorderEncounterParticipant,
  setEncounterTurn,
  updateEncounter,
  updateEncounterParticipantCondition,
  updateEncounterParticipant,
} from "../api/atlasApi";
import type {
  AddEncounterParticipantConditionRequest,
  EncounterParticipantConditionView,
  EncounterParticipantSideView,
  EncounterParticipantView,
  EncounterStatusView,
  EncounterSummaryView,
  OpenResultWindowRequest,
  ResultWindowRow,
  UpdateEncounterParticipantConditionRequest,
  UpdateEncounterParticipantRequest,
  UpdateEncounterRequest,
} from "../generated/atlas";
import { RecordPresentation } from "./recordPresentation";
import { encounterPath, navigateToAtlasRoute, type AtlasRoute } from "./routes";
import { WorkspaceLayout } from "./WorkspaceLayout";

type EncounterIndexViewProps = {
  route: Extract<AtlasRoute, { kind: "encounters" }>;
};

type EncounterDetailViewProps = {
  route: Extract<AtlasRoute, { kind: "encounter" }>;
};

type CreateEncounterForm = {
  name: string;
};

type EditEncounterForm = {
  name: string;
  slug: string;
  description?: string;
  note?: string;
  status: EncounterStatusView;
};

type AddRecordForm = {
  quantity?: number;
  initiative?: number;
};

type AddPcForm = {
  name: string;
  maxHp?: number;
  initiative?: number;
};

type AddConditionForm = {
  name: string;
  value?: number;
  duration?: number;
  sourceParticipantKey?: string;
  note?: string;
  sourceNote?: string;
};

const ENCOUNTER_WIDTH_SPECS = {
  filter: { defaultWidth: 360, minWidth: 300 },
  results: { defaultWidth: 620, minWidth: 420 },
  detail: { defaultWidth: 360, minWidth: 320 },
};
const ENCOUNTER_RECORD_PICKER_DEBOUNCE_MS = 250;

export function EncounterIndexView(_props: EncounterIndexViewProps) {
  const queryClient = useQueryClient();
  const [createOpen, setCreateOpen] = useState(false);
  const [showArchived, setShowArchived] = useState(false);
  const encounters = useQuery({ queryKey: ["encounters"], queryFn: getEncounters });
  const visible = (encounters.data?.encounters ?? []).filter(
    (encounter) => showArchived || encounter.status !== "archived",
  );
  const deleteMutation = useMutation({
    mutationFn: deleteEncounter,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["encounters"] }),
  });

  return (
    <main className="encounter-index-view">
      <section className="list-index-view__toolbar">
        <div>
          <h2>Encounters</h2>
          <p>{visible.length} active encounters</p>
        </div>
        <div className="encounter-actions">
          <Checkbox
            checked={showArchived}
            onChange={(event) => setShowArchived(event.target.checked)}
          >
            Archived
          </Checkbox>
          <Button
            icon={<Plus size={16} />}
            onClick={() => setCreateOpen(true)}
            type="primary"
          >
            New Encounter
          </Button>
        </div>
      </section>
      <section className="list-index-view__table">
        <Table
          columns={encounterColumns((encounter) => {
            if (confirm(`Delete ${encounter.name}?`)) {
              deleteMutation.mutate(encounter.slug);
            }
          })}
          dataSource={visible}
          loading={encounters.isLoading || encounters.isFetching}
          locale={{ emptyText: "No encounters" }}
          pagination={false}
          rowKey={(encounter) => encounter.encounter_key}
          size="middle"
        />
      </section>
      <CreateEncounterModal
        open={createOpen}
        onCancel={() => setCreateOpen(false)}
        onCreated={(slug) => {
          setCreateOpen(false);
          navigateToAtlasRoute({ kind: "encounter", slug });
        }}
      />
    </main>
  );
}

export function EncounterDetailView({ route }: EncounterDetailViewProps) {
  const queryClient = useQueryClient();
  const [selectedParticipantKey, setSelectedParticipantKey] = useState<string | null>(
    null,
  );
  const encounter = useQuery({
    queryKey: ["encounter", route.slug],
    queryFn: () => getEncounter(route.slug),
  });
  const selected =
    encounter.data?.participants.find(
      (participant) => participant.participant_key === selectedParticipantKey,
    ) ??
    encounter.data?.participants.find(
      (participant) =>
        participant.participant_key === encounter.data?.current_turn_participant_key,
    ) ??
    encounter.data?.participants[0];
  const detail = useQuery({
    queryKey: ["encounter-record-detail", selected?.record_key],
    enabled: Boolean(selected?.record_key && selected.status === "active"),
    queryFn: () => getRecordDetail(selected?.record_key ?? ""),
  });
  const invalidateEncounter = async () => {
    await queryClient.invalidateQueries({ queryKey: ["encounter", route.slug] });
    await queryClient.invalidateQueries({ queryKey: ["encounters"] });
  };
  const updateParticipant = useMutation({
    mutationFn: (request: UpdateEncounterParticipantRequest) =>
      updateEncounterParticipant(route.slug, request),
    onSuccess: invalidateEncounter,
  });
  const updateEncounterMutation = useMutation({
    mutationFn: updateEncounter,
    onSuccess: invalidateEncounter,
  });
  const reorderParticipant = useMutation({
    mutationFn: (request: {
      participantKey: string;
      targetParticipantKey: string;
      placement: "before" | "after";
    }) =>
      reorderEncounterParticipant(route.slug, {
        participant_key: request.participantKey,
        target_participant_key: request.targetParticipantKey,
        placement: request.placement,
      }),
    onSuccess: invalidateEncounter,
  });
  const removeParticipant = useMutation({
    mutationFn: (participantKey: string) =>
      removeEncounterParticipant(route.slug, participantKey),
    onSuccess: invalidateEncounter,
  });
  const startTurn = useMutation({
    mutationFn: (participantKey: string | null) =>
      setEncounterTurn({
        encounter_ref: route.slug,
        ...(participantKey ? { participant_key: participantKey } : {}),
      }),
    onSuccess: invalidateEncounter,
  });
  const addCondition = useMutation({
    mutationFn: (request: AddEncounterParticipantConditionRequest) =>
      addEncounterParticipantCondition(route.slug, request),
    onSuccess: invalidateEncounter,
  });
  const updateCondition = useMutation({
    mutationFn: (request: {
      participantKey: string;
      condition: UpdateEncounterParticipantConditionRequest;
    }) =>
      updateEncounterParticipantCondition(
        route.slug,
        request.participantKey,
        request.condition,
      ),
    onSuccess: invalidateEncounter,
  });
  const removeCondition = useMutation({
    mutationFn: (request: { participantKey: string; conditionId: bigint }) =>
      removeEncounterParticipantCondition(
        route.slug,
        request.participantKey,
        request.conditionId,
      ),
    onSuccess: invalidateEncounter,
  });

  return (
    <WorkspaceLayout
      filter={
        <EncounterRosterPane
          currentTurnParticipantKey={
            encounter.data?.current_turn_participant_key ?? null
          }
          loading={encounter.isLoading}
          onAddComplete={invalidateEncounter}
          onRemove={(participant) => {
            if (confirm(`Remove ${participant.display_name}?`)) {
              removeParticipant.mutate(participant.participant_key);
            }
          }}
          onSelect={setSelectedParticipantKey}
          onReorder={(participantKey, targetParticipantKey, placement) =>
            reorderParticipant.mutate({
              participantKey,
              targetParticipantKey,
              placement,
            })
          }
          onSetTurn={(participant) => startTurn.mutate(participant.participant_key)}
          onUpdate={(participant) => updateParticipant.mutate(participant)}
          participants={encounter.data?.participants ?? []}
          selectedParticipantKey={selected?.participant_key ?? null}
          slug={route.slug}
        />
      }
      results={
        <EncounterInspectorPane
          detailLoading={detail.isLoading || detail.isFetching}
          participant={selected}
          recordDetail={detail.data}
        />
      }
      detail={
        <EncounterTurnPane
          current={
            encounter.data?.participants.find(
              (participant) =>
                participant.participant_key ===
                encounter.data?.current_turn_participant_key,
            ) ?? null
          }
          encounter={encounter.data?.encounter ?? null}
          encounterNote={encounter.data?.note}
          participants={encounter.data?.participants ?? []}
          onDeleteEncounter={() => {
            if (confirm(`Delete ${encounter.data?.encounter.name ?? route.slug}?`)) {
              deleteEncounter(route.slug).then(() =>
                navigateToAtlasRoute({ kind: "encounters" }),
              );
            }
          }}
          onStart={() => startTurn.mutate(null)}
          onUpdateEncounter={(request) => updateEncounterMutation.mutate(request)}
          onAddCondition={(request) => addCondition.mutate(request)}
          onRemoveCondition={(participantKey, conditionId) =>
            removeCondition.mutate({ participantKey, conditionId })
          }
          onUpdateCondition={(participantKey, condition) =>
            updateCondition.mutate({ participantKey, condition })
          }
          onUpdate={(participant) => updateParticipant.mutate(participant)}
        />
      }
      labels={{ filter: "Roster", results: "Record", detail: "Turn" }}
      selectedRecordKey={selected?.record_key ?? selected?.participant_key ?? null}
      widthSpecs={ENCOUNTER_WIDTH_SPECS}
    />
  );
}

function encounterColumns(
  onDelete: (encounter: EncounterSummaryView) => void,
): ColumnsType<EncounterSummaryView> {
  return [
    {
      title: "Name",
      dataIndex: "name",
      render: (_value, encounter) => (
        <a href={encounterPath(encounter.slug)}>{encounter.name}</a>
      ),
    },
    { title: "Status", dataIndex: "status", width: 120 },
    { title: "Round", dataIndex: "round_number", width: 90 },
    { title: "Participants", dataIndex: "participant_count", width: 120 },
    { title: "Updated", dataIndex: "updated_at", width: 220 },
    {
      title: "",
      width: 72,
      render: (_value, encounter) => (
        <Button
          danger
          aria-label={`Delete ${encounter.name}`}
          icon={<Trash2 size={14} />}
          size="small"
          onClick={() => onDelete(encounter)}
        />
      ),
    },
  ];
}

function EncounterRosterPane({
  currentTurnParticipantKey,
  loading,
  onAddComplete,
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

  return (
    <section className="encounter-pane encounter-roster">
      <header className="encounter-pane__header">
        <div>
          <h2>Initiative</h2>
          <p>{participants.length} participants</p>
        </div>
        <div className="encounter-actions">
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
          {participants.map((participant) => (
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
                onReorder(
                  draggingParticipantKey,
                  participant.participant_key,
                  placement,
                );
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
                    commitRosterInitiative(
                      participant,
                      event.currentTarget.value,
                      onUpdate,
                    );
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
                onBlur={(event) =>
                  commitRosterHp(participant, event.target.value, onUpdate)
                }
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
              <span className="encounter-roster__max-hp">
                {participant.max_hp === undefined
                  ? ""
                  : `/ ${participant.max_hp.toString()}`}
              </span>
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
          ))}
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
              value={recordSearch}
              onChange={(event) => {
                setRecordSearch(event.target.value);
                setSelectedRecordKey(null);
              }}
            />
          </Form.Item>
          <Table
            columns={encounterRecordPickerColumns()}
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
            <InputNumber min={1} max={50} />
          </Form.Item>
          <Form.Item name="initiative" label="Initiative">
            <InputNumber />
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
            <InputNumber min={0} />
          </Form.Item>
          <Form.Item name="initiative" label="Initiative">
            <InputNumber />
          </Form.Item>
        </Form>
      </Modal>
    </section>
  );
}

function encounterRecordPickerColumns(): ColumnsType<ResultWindowRow> {
  return [
    {
      title: "Name",
      dataIndex: ["record", "title"],
      render: (_, row) => (
        <span className="encounter-record-picker__title">
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

function EncounterInspectorPane({
  detailLoading,
  participant,
  recordDetail,
}: {
  detailLoading: boolean;
  participant: EncounterParticipantView | undefined;
  recordDetail: Awaited<ReturnType<typeof getRecordDetail>> | undefined;
}) {
  if (!participant) {
    return (
      <section className="encounter-pane detail-empty">Select a participant.</section>
    );
  }
  if (participant.participant_kind === "pc") {
    return (
      <section className="encounter-pane manual-participant">
        <p className="eyebrow">PC</p>
        <h2>{participant.display_name}</h2>
        <dl>
          <dt>Initiative</dt>
          <dd>{displayNumber(participant.initiative)}</dd>
          <dt>HP</dt>
          <dd>{hpLabel(participant)}</dd>
          <dt>Side</dt>
          <dd>{participant.side}</dd>
        </dl>
        {participant.note && <p>{participant.note}</p>}
      </section>
    );
  }
  return (
    <section className="encounter-pane">
      <RecordPresentation
        detail={recordDetail}
        loading={detailLoading}
        onReference={() => undefined}
      />
    </section>
  );
}

function EncounterTurnPane({
  current,
  encounter,
  encounterNote,
  participants,
  onDeleteEncounter,
  onStart,
  onUpdateEncounter,
  onAddCondition,
  onRemoveCondition,
  onUpdateCondition,
  onUpdate,
}: {
  current: EncounterParticipantView | null;
  encounter: EncounterSummaryView | null;
  encounterNote: string | undefined;
  participants: EncounterParticipantView[];
  onDeleteEncounter: () => void;
  onStart: () => void;
  onUpdateEncounter: (encounter: UpdateEncounterRequest) => void;
  onAddCondition: (condition: AddEncounterParticipantConditionRequest) => void;
  onRemoveCondition: (participantKey: string, conditionId: bigint) => void;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => void;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => void;
}) {
  const [hpInput, setHpInput] = useState("");
  const [amount, setAmount] = useState<number | null>(null);
  const [note, setNote] = useState("");
  const [editEncounterOpen, setEditEncounterOpen] = useState(false);
  const [conditionForm] = Form.useForm<AddConditionForm>();
  const encounterName = encounter?.name ?? "Encounter";

  return (
    <section className="encounter-pane encounter-turn">
      <header className="encounter-pane__header">
        <div>
          <h2>{encounterName}</h2>
          <p>{current ? `Turn: ${current.display_name}` : "Not started"}</p>
        </div>
        <div className="encounter-actions">
          <Button
            aria-label="Edit encounter"
            icon={<Edit2 size={14} />}
            onClick={() => setEditEncounterOpen(true)}
          />
          <Button icon={<Play size={16} />} onClick={onStart} type="primary">
            {current ? "Next" : "Play"}
          </Button>
        </div>
      </header>
      {current ? (
        <div key={current.participant_key} className="encounter-turn__body">
          <div className="encounter-form-grid">
            <Form.Item label="Name" layout="vertical">
              <Input
                defaultValue={current.display_name}
                onBlur={(event) =>
                  onUpdate(
                    participantUpdate(current, { display_name: event.target.value }),
                  )
                }
              />
            </Form.Item>
            <Form.Item label="Initiative" layout="vertical">
              <InputNumber
                defaultValue={optionalNumber(current.initiative)}
                onBlur={(event) =>
                  onUpdate(
                    participantUpdate(current, {
                      initiative:
                        event.target.value === ""
                          ? undefined
                          : BigInt(Number(event.target.value)),
                    }),
                  )
                }
              />
            </Form.Item>
            <Form.Item label="Temp HP" layout="vertical">
              <InputNumber
                min={0}
                defaultValue={optionalNumber(current.temporary_hp)}
                onBlur={(event) =>
                  onUpdate(
                    participantUpdate(current, {
                      temporary_hp: BigInt(
                        Math.max(0, Number(event.target.value || 0)),
                      ),
                    }),
                  )
                }
              />
            </Form.Item>
          </div>
          <div className="encounter-control-row">
            <Form.Item label="HP or formula" layout="vertical">
              <Input
                value={hpInput}
                onChange={(event) => setHpInput(event.target.value)}
              />
            </Form.Item>
            <Button
              onClick={() => {
                const hp = evaluateHpFormula(hpInput);
                if (hp !== null) {
                  onUpdate(participantUpdate(current, { current_hp: BigInt(hp) }));
                  setHpInput("");
                }
              }}
            >
              Set
            </Button>
          </div>
          <div className="encounter-control-row">
            <Form.Item label="Amount" layout="vertical">
              <InputNumber
                min={0}
                value={amount}
                onChange={(value) => setAmount(value)}
              />
            </Form.Item>
            <Button
              onClick={() => {
                if (amount !== null) {
                  onUpdate(applyDamage(current, amount));
                  setAmount(null);
                }
              }}
            >
              Damage
            </Button>
            <Button
              onClick={() => {
                if (amount !== null) {
                  onUpdate(
                    participantUpdate(current, {
                      current_hp: BigInt(
                        Math.max(0, asNumber(current.current_hp) + amount),
                      ),
                    }),
                  );
                  setAmount(null);
                }
              }}
            >
              Heal
            </Button>
          </div>
          <Form.Item label="Participant note" layout="vertical">
            <Input.TextArea
              value={note || current.note || ""}
              onChange={(event) => setNote(event.target.value)}
              onBlur={() => onUpdate(participantUpdate(current, { note }))}
            />
          </Form.Item>
          <Form.Item label="Side" layout="vertical">
            <Select<EncounterParticipantSideView>
              value={current.side}
              onChange={(side) => onUpdate(participantUpdate(current, { side }))}
              options={["pc", "ally", "enemy", "neutral", "hazard"].map((value) => ({
                value: value as EncounterParticipantSideView,
                label: value,
              }))}
            />
          </Form.Item>
          <Button
            onClick={() =>
              onUpdate(participantUpdate(current, { defeated: !current.defeated }))
            }
          >
            {current.defeated ? "Mark active" : "Mark defeated"}
          </Button>
          <section className="encounter-conditions">
            <h3>Conditions</h3>
            {current.conditions.length === 0 ? (
              <div className="detail-empty">No conditions</div>
            ) : (
              <div className="encounter-condition-list">
                {current.conditions.map((condition) => (
                  <ConditionEditor
                    condition={condition}
                    key={condition.condition_id.toString()}
                    participantKey={current.participant_key}
                    participants={participants}
                    onRemove={onRemoveCondition}
                    onUpdate={onUpdateCondition}
                  />
                ))}
              </div>
            )}
            <Form
              form={conditionForm}
              layout="vertical"
              onFinish={(values) => {
                onAddCondition({
                  participant_key: current.participant_key,
                  name: values.name,
                  ...(values.value === undefined
                    ? {}
                    : { value: BigInt(values.value) }),
                  ...(values.duration === undefined
                    ? {}
                    : { duration_rounds: BigInt(values.duration) }),
                  ...(values.sourceParticipantKey
                    ? { source_participant_key: values.sourceParticipantKey }
                    : {}),
                  ...(values.note ? { note: values.note } : {}),
                  ...(values.sourceNote ? { source_note: values.sourceNote } : {}),
                });
                conditionForm.resetFields();
              }}
            >
              <Form.Item name="name" label="Condition" rules={[{ required: true }]}>
                <Input />
              </Form.Item>
              <div className="encounter-control-row">
                <Form.Item name="value" label="Value">
                  <InputNumber min={0} />
                </Form.Item>
                <Form.Item name="duration" label="Rounds">
                  <InputNumber min={0} />
                </Form.Item>
              </div>
              <Form.Item name="note" label="Note">
                <Input />
              </Form.Item>
              <Form.Item name="sourceParticipantKey" label="Source">
                <Select
                  allowClear
                  options={participants.map((participant) => ({
                    value: participant.participant_key,
                    label: participant.display_name,
                  }))}
                />
              </Form.Item>
              <Form.Item name="sourceNote" label="Source Note">
                <Input />
              </Form.Item>
              <Button onClick={() => conditionForm.submit()}>Add Condition</Button>
            </Form>
          </section>
        </div>
      ) : (
        <div className="detail-empty">Press play after setting initiative.</div>
      )}
      {encounter && (
        <div className="encounter-actions">
          <Button
            onClick={() =>
              onUpdateEncounter({
                ...encounterUpdate(encounter),
                note: encounterNote,
                status: "complete",
              })
            }
          >
            Complete
          </Button>
          <Button
            onClick={() =>
              onUpdateEncounter({
                ...encounterUpdate(encounter),
                note: encounterNote,
                status: "archived",
              })
            }
          >
            Archive
          </Button>
        </div>
      )}
      <Button danger onClick={onDeleteEncounter}>
        Delete encounter
      </Button>
      {encounter && (
        <EditEncounterModal
          encounter={encounter}
          encounterNote={encounterNote}
          open={editEncounterOpen}
          onCancel={() => setEditEncounterOpen(false)}
          onSave={(request) => {
            onUpdateEncounter(request);
            setEditEncounterOpen(false);
          }}
        />
      )}
    </section>
  );
}

function CreateEncounterModal({
  open,
  onCancel,
  onCreated,
}: {
  open: boolean;
  onCancel: () => void;
  onCreated: (slug: string) => void;
}) {
  const [form] = Form.useForm<CreateEncounterForm>();
  const queryClient = useQueryClient();
  const mutation = useMutation({
    mutationFn: (values: CreateEncounterForm) => createEncounter({ name: values.name }),
    onSuccess: async (view) => {
      await queryClient.invalidateQueries({ queryKey: ["encounters"] });
      form.resetFields();
      onCreated(view.encounter.slug);
    },
  });
  return (
    <Modal
      title="New encounter"
      open={open}
      onCancel={onCancel}
      onOk={() => form.submit()}
    >
      <Form form={form} layout="vertical" onFinish={(v) => mutation.mutate(v)}>
        <Form.Item name="name" label="Name" rules={[{ required: true }]}>
          <Input />
        </Form.Item>
      </Form>
    </Modal>
  );
}

function EditEncounterModal({
  encounter,
  encounterNote,
  open,
  onCancel,
  onSave,
}: {
  encounter: EncounterSummaryView;
  encounterNote: string | undefined;
  open: boolean;
  onCancel: () => void;
  onSave: (encounter: UpdateEncounterRequest) => void;
}) {
  const [form] = Form.useForm<EditEncounterForm>();
  return (
    <Modal
      title="Edit encounter"
      open={open}
      onCancel={onCancel}
      onOk={() => form.submit()}
      afterOpenChange={(visible) => {
        if (visible) {
          form.setFieldsValue({
            name: encounter.name,
            slug: encounter.slug,
            description: encounter.description,
            note: encounterNote,
            status: encounter.status,
          });
        }
      }}
    >
      <Form
        form={form}
        layout="vertical"
        onFinish={(values) =>
          onSave({
            encounter_key: encounter.encounter_key,
            slug: values.slug,
            name: values.name,
            ...(values.description ? { description: values.description } : {}),
            ...(values.note ? { note: values.note } : {}),
            status: values.status,
          })
        }
      >
        <Form.Item name="name" label="Name" rules={[{ required: true }]}>
          <Input />
        </Form.Item>
        <Form.Item name="slug" label="Slug" rules={[{ required: true }]}>
          <Input />
        </Form.Item>
        <Form.Item name="description" label="Description">
          <Input />
        </Form.Item>
        <Form.Item name="note" label="Note">
          <Input.TextArea />
        </Form.Item>
        <Form.Item name="status" label="Status" rules={[{ required: true }]}>
          <Select<EncounterStatusView>
            options={["draft", "running", "complete", "archived"].map((value) => ({
              value: value as EncounterStatusView,
              label: value,
            }))}
          />
        </Form.Item>
      </Form>
    </Modal>
  );
}

function ConditionEditor({
  condition,
  participantKey,
  participants,
  onRemove,
  onUpdate,
}: {
  condition: EncounterParticipantConditionView;
  participantKey: string;
  participants: EncounterParticipantView[];
  onRemove: (participantKey: string, conditionId: bigint) => void;
  onUpdate: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => void;
}) {
  const update = (changes: Partial<UpdateEncounterParticipantConditionRequest>) =>
    onUpdate(participantKey, {
      condition_id: condition.condition_id,
      condition_key: condition.condition_key,
      name: condition.name,
      value: condition.value,
      source_participant_key: condition.source_participant_key,
      duration_rounds: condition.duration_rounds,
      note: condition.note,
      source_note: condition.source_note,
      ...changes,
    });
  return (
    <div className="encounter-condition-row">
      <Form.Item label="Condition" layout="vertical">
        <Input
          defaultValue={condition.name}
          onBlur={(event) => update({ name: event.target.value })}
        />
      </Form.Item>
      <Form.Item label="Value" layout="vertical">
        <InputNumber
          min={0}
          defaultValue={optionalNumber(condition.value)}
          onBlur={(event) =>
            update({
              value:
                event.target.value === ""
                  ? undefined
                  : BigInt(Number(event.target.value)),
            })
          }
        />
      </Form.Item>
      <Form.Item label="Rounds" layout="vertical">
        <InputNumber
          min={0}
          defaultValue={optionalNumber(condition.duration_rounds)}
          onBlur={(event) =>
            update({
              duration_rounds:
                event.target.value === ""
                  ? undefined
                  : BigInt(Number(event.target.value)),
            })
          }
        />
      </Form.Item>
      <Form.Item label="Source" layout="vertical">
        <Select
          allowClear
          defaultValue={condition.source_participant_key}
          onChange={(value) => update({ source_participant_key: value })}
          options={participants.map((participant) => ({
            value: participant.participant_key,
            label: participant.display_name,
          }))}
        />
      </Form.Item>
      <Form.Item label="Note" layout="vertical">
        <Input
          defaultValue={condition.note}
          onBlur={(event) => update({ note: event.target.value || undefined })}
        />
      </Form.Item>
      <Form.Item label="Source Note" layout="vertical">
        <Input
          defaultValue={condition.source_note}
          onBlur={(event) => update({ source_note: event.target.value || undefined })}
        />
      </Form.Item>
      <Button
        danger
        aria-label={`Remove ${condition.name}`}
        icon={<Trash2 size={12} />}
        size="small"
        onClick={() => onRemove(participantKey, condition.condition_id)}
      />
    </div>
  );
}

function encounterUpdate(encounter: EncounterSummaryView): UpdateEncounterRequest {
  return {
    encounter_key: encounter.encounter_key,
    slug: encounter.slug,
    name: encounter.name,
    description: encounter.description,
    status: encounter.status,
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
  onUpdate(participantUpdate(participant, { current_hp: hp }));
}

function applyDamage(
  participant: EncounterParticipantView,
  amount: number,
): UpdateEncounterParticipantRequest {
  const temporaryHp = asNumber(participant.temporary_hp);
  const currentHp = asNumber(participant.current_hp);
  const tempDamage = Math.min(temporaryHp, amount);
  const remaining = amount - tempDamage;
  return participantUpdate(participant, {
    temporary_hp: BigInt(temporaryHp - tempDamage),
    current_hp: BigInt(Math.max(0, currentHp - remaining)),
  });
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

function hpLabel(participant: EncounterParticipantView): string {
  if (
    participant.current_hp === undefined &&
    participant.max_hp === undefined &&
    asNumber(participant.temporary_hp) === 0
  ) {
    return "";
  }
  const current = displayNumber(participant.current_hp);
  const max = displayNumber(participant.max_hp);
  const temporaryHp = asNumber(participant.temporary_hp);
  return `${current}/${max}${temporaryHp > 0 ? ` +${temporaryHp}` : ""}`;
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

function asNumber(value: bigint | undefined): number {
  return value === undefined ? 0 : Number(value);
}

function optionalNumber(value: bigint | undefined): number | undefined {
  return value === undefined ? undefined : Number(value);
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
