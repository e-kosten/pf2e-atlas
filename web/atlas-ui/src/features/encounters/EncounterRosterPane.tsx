import { EncounterRecordAdjustment } from "./EncounterRecordAdjustment";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { encounterStatusLabels } from "./encounterStatus";
import { EncounterModal } from "./EncounterModal";
import { Button, Dropdown, Form, Input, InputNumber, Tooltip, Typography } from "antd";
import { DndContext, DragOverlay } from "@dnd-kit/core";
import { SortableContext, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { SortableRow, useSortableList } from "../../shared/ui/sortable/SortableList";
import { Dice5, Ellipsis, Eye, EyeOff, GripVertical, Play, Trash2 } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import type {
  AddEncounterManualParticipantRequest,
  AddEncounterRecordParticipantRequest,
  EncounterParticipantView,
  SearchResultsRequest,
  UpdateEncounterParticipantEdit,
} from "../../generated/atlas";
import { EncounterAddMenu } from "./EncounterAddMenu";
import { PrimaryToolbar } from "../../shared/ui/PrimaryToolbar";
import { InitiativeValueField } from "./InitiativeValueField";
import { InitiativePopover } from "./InitiativeRollModal";
import {
  SearchPickerModal,
  type PickerSelection,
} from "../../shared/ui/pickers/SearchPickerModal";

import type { EncounterDifficultyProps } from "./EncounterDifficulty";
import {
  EncounterAddDifficulty,
  encounterRecordAddition,
  type EncounterRecordAddForm as AddRecordForm,
} from "./EncounterAddDifficulty";

type AddPcForm = {
  name: string;
  maxHp?: number | null;
  initiative?: number | null;
};

export function EncounterRosterPane({
  currentTurnParticipantKey,
  encounterSummary,
  difficulty,
  removeError,
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
  needsInitiative = false,
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
  encounterSummary?: import("../../generated/atlas").EncounterSummaryView;
  difficulty?: EncounterDifficultyProps;
  removeError?: { participantKey: string; message: string };
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
  needsInitiative?: boolean;
  onRollInitiative?: (participantKey?: string) => void;
  onRemove: (participant: EncounterParticipantView) => void;
  onReorder: (
    participantKey: string,
    targetParticipantKey: string,
    placement: "before" | "after",
  ) => Promise<boolean>;
  onSelect: (participantKey: string) => void;
  onSetTurn: (participant: EncounterParticipantView) => void;
  onUpdate: (
    participantKey: string,
    request: UpdateEncounterParticipantEdit,
  ) => Promise<unknown>;
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
  const [committedAdd, setCommittedAdd] = useState<string | null>(null);
  const [committedPc, setCommittedPc] = useState(false);
  const pendingFormReset = useRef({ record: false, pc: false });
  const [recordOpen, setRecordOpen] = useState(false);
  const [pcOpen, setPcOpen] = useState(false);
  const [candidate, setCandidate] = useState<PickerSelection | null>(null);
  const selectedRecordKey = candidate?.record.record_key ?? null;
  const selectedIsCreature = candidate?.record.kind === "creature";
  const defaultAdjustment = selectedIsCreature
    ? candidate?.detail.encounter_adjustment
    : undefined;
  const [recordForm] = Form.useForm<AddRecordForm>();
  const [pcForm] = Form.useForm<AddPcForm>();
  useEffect(() => {
    if (disabled) return;
    if (pendingFormReset.current.record) {
      recordForm.setFields([
        { name: "quantity", value: 1, touched: false, errors: [] },
        { name: "adjustment", value: defaultAdjustment, touched: false, errors: [] },
      ]);
      pendingFormReset.current.record = false;
    }
    if (pendingFormReset.current.pc) {
      pcForm.resetFields();
      pendingFormReset.current.pc = false;
    }
  }, [disabled, committedAdd, committedPc, recordForm, pcForm, defaultAdjustment]);
  const addRecord = async () => {
    if (disabled || !candidate) return;
    const { record_ref, quantity, participant_variant } = encounterRecordAddition(
      candidate.record.record_key,
      selectedIsCreature,
      recordForm,
    );
    const receipt = `Added ${quantity} × ${candidate.record.title}.`;
    setCommittedAdd(null);
    setAdded("");
    try {
      const saved = await onAddRecord({
        record_ref,
        quantity,
        ...(participant_variant === null ? {} : { participant_variant }),
      });
      if (saved) {
        setAdded(receipt);
        recordForm.setFields([
          { name: "quantity", value: 1, touched: false, errors: [] },
          { name: "adjustment", value: defaultAdjustment, touched: false, errors: [] },
        ]);
      } else {
        pendingFormReset.current.record = true;
        setCommittedAdd(receipt);
      }
    } catch {
      /* The owner displays the failure; retain the form and selection. */
    }
  };
  const addPc = async (values: AddPcForm) => {
    if (disabled) return;
    try {
      const saved = await onAddManual({
        display_name: values.name,
        ...(values.maxHp == null ? {} : { max_hp: values.maxHp }),
        ...(values.initiative == null ? {} : { initiative: values.initiative }),
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
            <Tooltip
              title="Drag to reorder. With a keyboard, press Space then use the arrow keys."
              trigger={["hover", "focus"]}
            >
              <Button
                {...attributes}
                {...listeners}
                ref={setActivatorNodeRef}
                type="text"
                size="small"
                disabled={disabled}
                className="encounter-roster__drag-handle"
                aria-label={`Reorder ${participant.display_name}`}
                icon={<GripVertical size={16} aria-hidden="true" />}
              />
            </Tooltip>
            {participant.participant_kind === "pc" ? (
              <InitiativeValueField
                name={participant.display_name}
                value={participant.initiative}
                disabled={disabled}
                compact
                onSave={(value) =>
                  onUpdate(participant.participant_key, {
                    expected_revision: participant.revision,
                    initiative: value ?? null,
                  })
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
                participant.hidden && "Dimmed",
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
              <span className="encounter-roster__name">
                {participant.hidden && (
                  <EyeOff
                    className="encounter-roster__dimmed-icon"
                    size={14}
                    aria-hidden="true"
                  />
                )}
                {participant.display_name}
              </span>
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
              onToggleDim={() =>
                onUpdate(participant.participant_key, {
                  expected_revision: participant.revision,
                  hidden: !participant.hidden,
                })
              }
              onRemove={() => onRemove(participant)}
            />
            {removeError?.participantKey === participant.participant_key && (
              <Typography.Text
                type="danger"
                role="alert"
                className="encounter-roster__remove-error"
              >
                <span>{removeError.message}</span>
                <span>Try removing again.</span>
              </Typography.Text>
            )}
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
            <span className="encounter-roster__turn-caption">
              {encounterSummary
                ? `${encounterStatusLabels[encounterSummary.status]} · Round ${encounterSummary.round_number}`
                : "Current turn"}
            </span>
            <strong>
              {currentParticipant?.display_name ??
                (loading
                  ? "Loading…"
                  : needsInitiative && participants.length > 0
                    ? "Initiative not set"
                    : "Not started")}
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
      </header>
      <div
        className="encounter-roster__participants"
        role="region"
        aria-label="Encounter participants"
        // eslint-disable-next-line jsx-a11y/no-noninteractive-tabindex -- The bounded roster needs keyboard scrolling, including in WebKit.
        tabIndex={0}
      >
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
      </div>
      <footer className="encounter-roster__footer">
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
      </footer>
      <SearchPickerModal<AddRecordForm>
        buildRequest={encounterRecordPickerRequest}
        emptyPrompt="Search for a creature or hazard"
        emptyResults="No creatures or hazards"
        form={recordForm}
        disabled={disabled}
        okButtonProps={{ disabled: disabled || !selectedRecordKey, loading: disabled }}
        onCancel={() => setRecordOpen(false)}
        onFinish={() => void addRecord()}
        onSelectionChange={(selection) => {
          if (selection) {
            const quantity = recordForm.getFieldValue("quantity");
            recordForm.setFields([
              { name: "quantity", value: quantity === undefined ? 1 : quantity },
              {
                name: "adjustment",
                value:
                  selection.record.kind === "creature"
                    ? selection.detail.encounter_adjustment
                    : undefined,
                touched: false,
                errors: [],
              },
            ]);
          }
          setCandidate(selection);
        }}
        open={recordOpen}
        selectedRecordKey={selectedRecordKey}
        title="Add creature or hazard"
        dismissLabel="Done"
        summary={
          difficulty && recordOpen ? (
            <EncounterAddDifficulty
              {...difficulty}
              form={recordForm}
              selectedRecordKey={selectedRecordKey}
              creature={selectedIsCreature}
              recordName={candidate?.record.title ?? ""}
              disabled={disabled}
            />
          ) : undefined
        }
        status={
          added || (!disabled && committedAdd) ? (
            <span role="status" className="encounter-add-receipt">
              {committedAdd ?? added}
            </span>
          ) : undefined
        }
      >
        <>
          {recovery}
          {error && (
            <ErrorNotice
              error={error}
              title="Participant could not be added"
              description={error}
            />
          )}
          <div
            className="encounter-add-fields"
            inert={!selectedRecordKey}
            style={selectedRecordKey ? undefined : { visibility: "hidden", opacity: 0 }}
          >
            {(!selectedRecordKey || selectedIsCreature) && (
              <EncounterRecordAdjustment />
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
              <InputNumber
                className="encounter-add-quantity"
                aria-label="Quantity"
                min={1}
                max={50}
              />
            </Form.Item>
          </div>
        </>
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
            <ErrorNotice
              error={error}
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

function inputNumberValue(value: number | undefined): string {
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
    participant.hidden ? "encounter-roster__row--dimmed" : "",
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
  onToggleDim,
  onRemove,
}: {
  participant: EncounterParticipantView;
  disabled: boolean;
  onSetTurn: () => void;
  onToggleDim: () => Promise<unknown>;
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
            {
              key: "dim",
              label: participant.hidden ? "Show normally" : "Dim in roster",
              icon: participant.hidden ? (
                <Eye size={14} aria-hidden="true" />
              ) : (
                <EyeOff size={14} aria-hidden="true" />
              ),
              disabled,
              onClick: () => {
                trigger.current?.focus({ preventScroll: true });
                void onToggleDim().catch(() => {
                  /* The encounter mutation owner retains failure feedback. */
                });
              },
            },
          ],
        }}
      >
        <Tooltip
          title={`Actions for ${participant.display_name}`}
          trigger={["hover", "focus"]}
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
        </Tooltip>
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
  const ratio = (participant.current_hp + participant.temporary_hp) / maximum.value;
  return ratio <= 0.25
    ? "encounter-roster__hp--critical"
    : ratio <= 0.5
      ? "encounter-roster__hp--bloodied"
      : "encounter-roster__hp--healthy";
}
