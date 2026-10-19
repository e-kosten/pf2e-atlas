import { Alert, Button, Form, Input, InputNumber, Modal } from "antd";
import {
  DndContext,
  DragOverlay,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import {
  SortableContext,
  arrayMove,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { Dice5, GripVertical, Play, Trash2 } from "lucide-react";
import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import type {
  AddEncounterManualParticipantRequest,
  AddEncounterRecordParticipantRequest,
  EncounterParticipantView,
  SearchResultsRequest,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import { PrimaryToolbar } from "../../shared/ui/PrimaryToolbar";
import { InitiativeValueField } from "./InitiativeValueField";
import { participantUpdate } from "./participantEdits";
import { InitiativePopover } from "./InitiativeRollModal";
import { SearchPickerModal } from "../../shared/ui/pickers/SearchPickerModal";

type AddRecordForm = {
  quantity?: number | null;
  initiative?: number | null;
};

type AddPcForm = {
  name: string;
  maxHp?: number | null;
  initiative?: number | null;
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
  onRollInitiative,
  onRemove,
  onReorder,
  onSelect,
  onSetTurn,
  onUpdate,
  onRoll,
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
  onRollInitiative?: (participantKey?: string) => void;
  onRemove: (participant: EncounterParticipantView) => void;
  onReorder: (
    participantKey: string,
    targetParticipantKey: string,
    placement: "before" | "after",
  ) => Promise<boolean>;
  onSelect: (participantKey: string) => void;
  onSetTurn: (participant: EncounterParticipantView) => void;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => Promise<unknown>;
  onRoll: (
    request: import("../../generated/atlas").RollInitiativeRequest,
  ) => Promise<import("../../generated/atlas").InitiativeRollView>;
  participants: EncounterParticipantView[];
  selectedParticipantKey: string | null;
  slug: string;
}) {
  const formId = useId();
  const [draggingKey, setDraggingKey] = useState<string | null>(null);
  // Hold only visual order during the guarded write/read. Values and gameplay
  // ordering still come from the canonical refreshed encounter.
  const [droppedOrder, setDroppedOrder] = useState<{
    base: EncounterParticipantView[];
    keys: string[];
  } | null>(null);
  const displayedParticipants =
    droppedOrder?.base === participants
      ? [...participants].sort(
          (a, b) =>
            droppedOrder.keys.indexOf(a.participant_key) -
            droppedOrder.keys.indexOf(b.participant_key),
        )
      : participants;
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 6 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );
  const draggedParticipant = participants.find(
    (p) => p.participant_key === draggingKey,
  );
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
        ...(values.initiative == null ? {} : { initiative: BigInt(values.initiative) }),
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
        ...(values.maxHp == null ? {} : { max_hp: BigInt(values.maxHp) }),
        ...(values.initiative == null ? {} : { initiative: BigInt(values.initiative) }),
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
  const renderParticipant = (participant: EncounterParticipantView) => {
    return (
      <SortableRosterRow
        id={participant.participant_key}
        disabled={disabled}
        className={rosterRowClass(
          participant,
          participant.participant_key === currentTurnParticipantKey,
          participant.participant_key === selectedParticipantKey,
        )}
        key={participant.participant_key}
      >
        {({ attributes, listeners, setActivatorNodeRef }) => (
          <>
            <Button
              {...attributes}
              {...listeners}
              ref={setActivatorNodeRef}
              type="text"
              size="small"
              disabled={disabled}
              className="encounter-roster__drag-handle"
              aria-label={`Reorder ${participant.display_name}`}
              title="Drag to reorder. With a keyboard, press Space then use the arrow keys."
              icon={<GripVertical size={16} aria-hidden="true" />}
            />
            {participant.participant_kind === "pc" ? (
              <InitiativeValueField
                name={participant.display_name}
                value={participant.initiative}
                disabled={disabled}
                compact
                onSave={(value) =>
                  onUpdate(
                    participantUpdate(participant, {
                      initiative: value === undefined ? undefined : BigInt(value),
                    }),
                  )
                }
              />
            ) : (
              <InitiativePopover
                participant={participant}
                encounterKey={slug}
                disabled={disabled}
                onRoll={onRoll}
                onUpdate={onUpdate}
              />
            )}
            <Button
              type="text"
              className="encounter-roster__select"
              data-participant-key={participant.participant_key}
              aria-pressed={participant.participant_key === selectedParticipantKey}
              aria-current={
                participant.participant_key === currentTurnParticipantKey
                  ? "true"
                  : undefined
              }
              onClick={() => onSelect(participant.participant_key)}
            >
              <span className="encounter-roster__name">{participant.display_name}</span>
              {participant.hazard_disabled && (
                <span className="encounter-roster__state">Disabled</span>
              )}
              {participant.defeated && (
                <span className="encounter-roster__state">Defeated</span>
              )}
            </Button>
            <span
              className="encounter-roster__hp"
              aria-label={`${participant.display_name} HP`}
            >
              {participant.current_hp == null
                ? "—"
                : (participant.current_hp + participant.temporary_hp).toString()}
            </span>
            <div className="encounter-roster__row-actions">
              <Button
                type="text"
                size="small"
                disabled={disabled}
                aria-label={`Set turn to ${participant.display_name}`}
                title={`Set turn to ${participant.display_name}`}
                icon={<Play size={14} aria-hidden="true" />}
                onClick={() => onSetTurn(participant)}
              />
              <Button
                type="text"
                size="small"
                danger
                disabled={disabled}
                aria-label={`Remove ${participant.display_name}`}
                title={`Remove ${participant.display_name}`}
                icon={<Trash2 size={14} aria-hidden="true" />}
                onClick={() => onRemove(participant)}
              />
            </div>
          </>
        )}
      </SortableRosterRow>
    );
  };

  return (
    <section className="encounter-pane encounter-roster">
      <header className="encounter-pane__header">
        <PrimaryToolbar className="encounter-actions">
          {onRollInitiative && (
            <Button
              disabled={
                disabled || participants.every((p) => p.participant_kind === "pc")
              }
              icon={<Dice5 size={14} />}
              onClick={() => onRollInitiative()}
            >
              Initiative
            </Button>
          )}
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
        <DndContext
          sensors={sensors}
          onDragStart={({ active }) => setDraggingKey(String(active.id))}
          onDragCancel={() => setDraggingKey(null)}
          onDragEnd={async ({ active, over }) => {
            setDraggingKey(null);
            if (disabled || !over || active.id === over.id) return;
            const from = participants.findIndex((p) => p.participant_key === active.id);
            const to = participants.findIndex((p) => p.participant_key === over.id);
            if (from < 0 || to < 0) return;
            setDroppedOrder({
              base: participants,
              keys: arrayMove(
                participants.map((p) => p.participant_key),
                from,
                to,
              ),
            });
            try {
              const refreshed = await onReorder(
                String(active.id),
                String(over.id),
                from < to ? "after" : "before",
              );
              // Canonical query notifications may render after this promise.
              // Keep the visual order until the participants snapshot changes.
              if (!refreshed) setDroppedOrder(null);
            } catch {
              /* The mutation owner displays failures; restore canonical order. */
              setDroppedOrder(null);
            }
          }}
        >
          <SortableContext
            items={displayedParticipants.map((p) => p.participant_key)}
            strategy={verticalListSortingStrategy}
          >
            <div className="encounter-roster__list">
              <div className="encounter-roster__columns" aria-hidden="true">
                <span />
                <span>Init</span>
                <span>Participant</span>
                <span>HP</span>
                <span />
              </div>
              {displayedParticipants.map(renderParticipant)}
            </div>
          </SortableContext>
          <DragOverlay dropAnimation={null}>
            {draggedParticipant && (
              <div
                className="encounter-roster__row encounter-roster__drag-preview"
                aria-hidden
              >
                <GripVertical size={16} />
                <span>{inputNumberValue(draggedParticipant.initiative) || "—"}</span>
                <span className="encounter-roster__name">
                  {draggedParticipant.display_name}
                </span>
                <span className="encounter-roster__hp">
                  {draggedParticipant.current_hp == null
                    ? "—"
                    : (
                        draggedParticipant.current_hp + draggedParticipant.temporary_hp
                      ).toString()}
                </span>
                <span className="encounter-roster__row-actions">
                  <Play size={14} />
                  <Trash2 size={14} />
                </span>
              </div>
            )}
          </DragOverlay>
        </DndContext>
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
        <Form.Item
          name="quantity"
          label="Quantity"
          initialValue={1}
          rules={[
            {
              required: true,
              type: "integer",
              min: 1,
              max: 50,
              message: "Enter a whole number from 1 to 50.",
            },
          ]}
        >
          <InputNumber aria-label="Quantity" min={1} max={50} />
        </Form.Item>
        <Form.Item name="initiative" label="Initiative" rules={[initiativeRule]}>
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
          <Form.Item
            name="maxHp"
            label="Max HP"
            rules={[
              {
                type: "integer",
                min: 0,
                max: Number.MAX_SAFE_INTEGER,
                message: "Enter a nonnegative whole number within the supported range.",
              },
            ]}
          >
            <InputNumber aria-label="Max HP" min={0} />
          </Form.Item>
          <Form.Item name="initiative" label="Initiative" rules={[initiativeRule]}>
            <InputNumber aria-label="Initiative" />
          </Form.Item>
        </Form>
      </Modal>
    </section>
  );
}

const initiativeRule = {
  type: "integer" as const,
  min: Number.MIN_SAFE_INTEGER,
  max: Number.MAX_SAFE_INTEGER,
  message: "Enter a whole number within the supported range.",
};

function SortableRosterRow({
  id,
  disabled,
  className,
  children,
}: {
  id: string;
  disabled: boolean;
  className: string;
  children: (
    handle: Pick<
      ReturnType<typeof useSortable>,
      "attributes" | "listeners" | "setActivatorNodeRef"
    >,
  ) => ReactNode;
}) {
  const {
    attributes,
    listeners,
    setActivatorNodeRef,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id, disabled });
  return (
    <div
      ref={setNodeRef}
      className={className}
      style={{
        transform: CSS.Transform.toString(transform),
        transition,
        opacity: isDragging ? 0.25 : undefined,
      }}
    >
      {children({ attributes, listeners, setActivatorNodeRef })}
    </div>
  );
}

function encounterRecordPickerRequest(query: string): SearchResultsRequest {
  const filter = {
    kind: "in" as const,
    field: "kind",
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
    participant.defeated || participant.hazard_disabled
      ? "encounter-roster__row--inactive"
      : "",
  ]
    .filter(Boolean)
    .join(" ");
}
