import { EncounterModal } from "./EncounterModal";
import {
  Alert,
  Button,
  Dropdown,
  Form,
  Input,
  InputNumber,
  Select,
  Tooltip,
} from "antd";
import { DndContext, DragOverlay } from "@dnd-kit/core";
import { SortableContext, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { SortableRow, useSortableList } from "../../shared/ui/sortable/SortableList";
import { Dice5, Ellipsis, GripVertical, Play, Trash2 } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import type {
  AddEncounterManualParticipantRequest,
  AddEncounterRecordParticipantRequest,
  EncounterParticipantView,
  EncounterParticipantVariantView,
  SearchResultsRequest,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import { EncounterAddMenu } from "./EncounterAddMenu";
import { PrimaryToolbar } from "../../shared/ui/PrimaryToolbar";
import { InitiativeValueField } from "./InitiativeValueField";
import { participantUpdate } from "./participantEdits";
import { InitiativePopover } from "./InitiativeRollModal";
import { SearchPickerModal } from "../../shared/ui/pickers/SearchPickerModal";

type AddRecordForm = {
  quantity?: number | null;
  initiative?: number | null;
  adjustment?: "as_listed" | EncounterParticipantVariantView;
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
  onAddParty,
  disabled,
  error,
  recovery,
  onAdvanceTurn,
  advanceDisabled = false,
  advanceLoading = false,
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
  onAddParty: (
    request: import("../../generated/atlas").AddEncounterPartyRequest,
  ) => Promise<void>;
  onAddManual: (request: AddEncounterManualParticipantRequest) => Promise<boolean>;
  disabled: boolean;
  error?: string;
  recovery?: React.ReactNode;
  onAdvanceTurn: () => void;
  advanceDisabled?: boolean;
  advanceLoading?: boolean;
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
  const currentParticipant = participants.find(
    (participant) => participant.participant_key === currentTurnParticipantKey,
  );
  const sortable = useSortableList({
    items: participants,
    getKey: (participant) => participant.participant_key,
    disabled,
    onReorder,
  });
  const [added, setAdded] = useState("");
  const [candidateName, setCandidateName] = useState("");
  const [committedAdd, setCommittedAdd] = useState<string | null>(null);
  const [committedPc, setCommittedPc] = useState(false);
  const pendingFormReset = useRef({ record: false, pc: false });
  const [recordOpen, setRecordOpen] = useState(false);
  const [pcOpen, setPcOpen] = useState(false);
  const [selectedRecordKey, setSelectedRecordKey] = useState<string | null>(null);
  const [selectedIsCreature, setSelectedIsCreature] = useState(false);
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
        ...(selectedIsCreature && values.adjustment && values.adjustment !== "as_listed"
          ? { participant_variant: values.adjustment }
          : {}),
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
      <SortableRow
        id={participant.participant_key}
        disabled={sortable.disabled}
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
              aria-label={[
                participant.display_name,
                participant.participant_key === currentTurnParticipantKey &&
                  "Current turn",
                participant.hazard_disabled && "Disabled",
                participant.defeated && "Defeated",
              ]
                .filter(Boolean)
                .join(", ")}
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
              className={`encounter-roster__hp ${rosterHpTone(participant)}`}
              aria-label={`${participant.display_name} HP`}
            >
              {participant.current_hp == null
                ? "—"
                : (participant.current_hp + participant.temporary_hp).toString()}
            </span>
            <ParticipantActions
              participant={participant}
              disabled={disabled}
              onSetTurn={() => onSetTurn(participant)}
              onRemove={() => onRemove(participant)}
            />
          </>
        )}
      </SortableRow>
    );
  };

  return (
    <section className="encounter-pane encounter-roster">
      <header className="encounter-roster__header">
        <PrimaryToolbar className="encounter-roster__turn-bar">
          <div className="encounter-roster__turn-summary">
            <span className="encounter-roster__turn-caption">Current turn</span>
            <strong>
              {currentParticipant?.display_name ??
                (loading ? "Loading…" : "Not started")}
            </strong>
          </div>
          <Button
            disabled={
              disabled || advanceDisabled || loading || participants.length === 0
            }
            loading={advanceLoading ? { delay: 200 } : false}
            icon={<Play size={14} aria-hidden="true" />}
            type="primary"
            onClick={onAdvanceTurn}
          >
            {currentTurnParticipantKey ? "Next turn" : "Start turn"}
          </Button>
        </PrimaryToolbar>
        <PrimaryToolbar className="encounter-roster__management">
          <EncounterAddMenu
            onAdd={onAddParty}
            disabled={disabled}
            onAddCreature={() => {
              setAdded("");
              setRecordOpen(true);
            }}
            onAddPc={() => {
              setCommittedPc(false);
              setPcOpen(true);
            }}
          />
          {onRollInitiative && (
            <Button
              disabled={
                disabled || participants.every((p) => p.participant_kind === "pc")
              }
              icon={<Dice5 size={14} />}
              type="text"
              onClick={() => onRollInitiative()}
            >
              Roll initiative
            </Button>
          )}
        </PrimaryToolbar>
      </header>
      {loading ? (
        <div className="detail-empty">Loading encounter...</div>
      ) : participants.length === 0 ? (
        <div className="detail-empty">Add a creature, hazard, or PC.</div>
      ) : (
        <DndContext {...sortable.context}>
          <SortableContext
            items={sortable.items.map((p) => p.participant_key)}
            strategy={verticalListSortingStrategy}
          >
            <div className="encounter-roster__list">
              <div className="encounter-roster__columns" aria-hidden="true">
                <span />
                <span className="encounter-roster__initiative-heading">Init</span>
                <span className="encounter-roster__name-heading">Participant</span>
                <span className="encounter-roster__hp-heading">HP</span>
                <span />
              </div>
              {sortable.items.map(renderParticipant)}
            </div>
          </SortableContext>
          <DragOverlay dropAnimation={null}>
            {sortable.activeItem && (
              <div
                className="encounter-roster__row encounter-roster__drag-preview"
                aria-hidden
              >
                <GripVertical size={16} />
                <span>{inputNumberValue(sortable.activeItem.initiative) || "—"}</span>
                <span className="encounter-roster__name">
                  {sortable.activeItem.display_name}
                </span>
                <span
                  className={`encounter-roster__hp ${rosterHpTone(sortable.activeItem)}`}
                >
                  {sortable.activeItem.current_hp == null
                    ? "—"
                    : (
                        sortable.activeItem.current_hp +
                        sortable.activeItem.temporary_hp
                      ).toString()}
                </span>
                <Ellipsis size={16} />
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
        onSelectionChange={(selection) => {
          setSelectedRecordKey(selection?.record.record_key ?? null);
          setSelectedIsCreature(selection?.record.kind === "creature");
          if (selection) setCandidateName(selection.record.title);
        }}
        open={recordOpen}
        selectedRecordKey={selectedRecordKey}
        title="Add creature or hazard"
        dismissLabel="Done"
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
        {selectedIsCreature && (
          <Form.Item
            name="adjustment"
            label="Adjustment"
            initialValue="as_listed"
            preserve={false}
          >
            <Select
              aria-label="Adjustment"
              options={[
                { value: "as_listed", label: "As listed" },
                { value: "normal", label: "Normal" },
                { value: "elite", label: "Elite" },
                { value: "weak", label: "Weak" },
              ]}
            />
          </Form.Item>
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
      <EncounterModal
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
      </EncounterModal>
    </section>
  );
}

const initiativeRule = {
  type: "integer" as const,
  min: Number.MIN_SAFE_INTEGER,
  max: Number.MAX_SAFE_INTEGER,
  message: "Enter a whole number within the supported range.",
};

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

function ParticipantActions({
  participant,
  disabled,
  onSetTurn,
  onRemove,
}: {
  participant: EncounterParticipantView;
  disabled: boolean;
  onSetTurn: () => void;
  onRemove: () => void;
}) {
  const trigger = useRef<HTMLButtonElement>(null);
  return (
    <div className="encounter-roster__actions">
      <Dropdown
        trigger={["click"]}
        menu={{
          "aria-label": `${participant.display_name} actions`,
          items: [
            {
              key: "turn",
              label: `Set turn to ${participant.display_name}`,
              icon: <Play size={14} aria-hidden="true" />,
              disabled,
              onClick: () => {
                trigger.current?.focus();
                onSetTurn();
              },
            },
          ],
        }}
      >
        <Button
          ref={trigger}
          type="text"
          size="small"
          disabled={disabled}
          className="encounter-roster__more"
          aria-label={`Actions for ${participant.display_name}`}
          aria-haspopup="menu"
          icon={<Ellipsis size={16} aria-hidden="true" />}
        />
      </Dropdown>
      <Tooltip
        title={`Remove ${participant.display_name}`}
        trigger={["hover", "focus"]}
      >
        <Button
          type="text"
          size="small"
          danger
          disabled={disabled}
          aria-label={`Remove ${participant.display_name}`}
          icon={<Trash2 size={14} aria-hidden="true" />}
          onClick={onRemove}
        />
      </Tooltip>
    </div>
  );
}

function rosterHpTone(participant: EncounterParticipantView): string {
  const maximum = participant.effective_max_hp;
  if (
    participant.current_hp == null ||
    maximum.state !== "value" ||
    maximum.value == null ||
    maximum.value <= 0
  )
    return "";
  const ratio =
    (Number(participant.current_hp) + Number(participant.temporary_hp)) / maximum.value;
  return ratio <= 0.25
    ? "encounter-roster__hp--critical"
    : ratio <= 0.5
      ? "encounter-roster__hp--bloodied"
      : "encounter-roster__hp--healthy";
}
