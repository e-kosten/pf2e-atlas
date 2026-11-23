import { restoreOverlayFocus } from "../../shared/ui/overlays/restoreOverlayFocus";
import type { ReorderPlacement } from "../../generated/atlas";
import { EncounterRecordAdjustment } from "./EncounterRecordAdjustment";
import { isUnconfirmedMutation } from "../../api/apiError";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { encounterTurnText } from "./encounterSummary";
import {
  InitiativeEntryEditor,
  type InitiativeEntryFields,
} from "./InitiativeEntryEditor";
import { CharacterPicker } from "../parties/CharacterPicker";
import { EncounterPcAdd } from "../parties/EncounterPcAdd";
import { Button, Dropdown, Form, InputNumber, Tooltip, Typography } from "antd";
import { DndContext, DragOverlay } from "@dnd-kit/core";
import { SortableContext, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { SortableRow, useSortableList } from "../../shared/ui/sortable/SortableList";
import { Ellipsis, Eye, EyeOff, GripVertical, Play, Trash2 } from "lucide-react";
import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import type {
  AddEncounterRecordParticipantRequest,
  EncounterParticipantView,
  UpdateEncounterParticipantEdit,
} from "../../generated/atlas";
import { EncounterAddMenu, type EncounterPcAddMode } from "./EncounterAddMenu";
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
  recordAdditionRequest,
  encounterPreviewBasis,
  type PreparedCreatureAddition,
  type EncounterRecordAddForm as AddRecordForm,
} from "./EncounterAddDifficulty";

export function EncounterRosterPane({
  currentTurnParticipantKey,
  encounterSummary,
  encounterRevision,
  difficulty,
  removeError,
  removingKey,
  loading,
  onAddRecord,
  onAddInitiativeEntry,
  initiativeEntryError,
  pending,
  onAddParty,
  disabled,
  error,
  recovery,
  turnAction,
  needsInitiative = false,
  rollInitiativeAction,
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
  encounterRevision: number;
  difficulty?: EncounterDifficultyProps;
  removingKey: string | null;
  removeError?: { participantKey: string; message: string };
  loading: boolean;
  onAddRecord: (
    encounterKey: string,
    request: AddEncounterRecordParticipantRequest,
  ) => Promise<boolean>;
  onAddParty: (
    request: import("../../generated/atlas").AddEncounterPartyRequest,
  ) => Promise<void>;
  onAddInitiativeEntry?: (request: InitiativeEntryFields) => Promise<boolean>;
  initiativeEntryError?: string;
  pending: boolean;
  disabled: boolean;
  error?: string;
  recovery?: React.ReactNode;
  turnAction: React.ReactNode;
  needsInitiative?: boolean;
  rollInitiativeAction: React.ReactNode;
  onRemove: (participant: EncounterParticipantView, opener: HTMLElement) => void;
  onReorder: (
    participantKey: string,
    targetParticipantKey: string,
    placement: ReorderPlacement,
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
  const turnText = encounterSummary && encounterTurnText(encounterSummary);
  const sortable = useSortableList({
    items: participants,
    getKey: (participant) => participant.participant_key,
    disabled,
    onReorder,
  });
  const [added, setAdded] = useState("");
  const [committedAdd, setCommittedAdd] = useState<string | null>(null);
  const pendingFormReset = useRef({ record: false });
  const [recordOpen, setRecordOpen] = useState(false);
  const [pcAddMode, setPcAddMode] = useState<EncounterPcAddMode>();
  const PcAddDialog = pcAddMode === "existing" ? CharacterPicker : EncounterPcAdd;
  const [entryOpen, setEntryOpen] = useState(false);
  const entryOpener = useRef<HTMLElement | null>(null);
  const roster = useRef<HTMLElement>(null);
  const [compactActions, setCompactActions] = useState(false);
  useLayoutEffect(() => {
    const pane = roster.current;
    if (!pane) return;
    const measure = () => {
      if (pane.clientWidth) setCompactActions(pane.clientWidth < 360);
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(pane);
    return () => observer.disconnect();
  }, []);
  const [candidate, setCandidate] = useState<PickerSelection | null>(null);
  const selectedRecordKey = candidate?.record.record_key ?? null;
  const [preparedCreature, setPreparedCreature] =
    useState<PreparedCreatureAddition | null>(null);
  const scenario = candidate?.detail.record.creature_display?.uses.length
    ? { artifact_generation: candidate.detail.artifact_generation }
    : undefined;
  const selectedIsCreature = candidate?.record.kind === "creature";
  const defaultAdjustment = selectedIsCreature
    ? candidate?.detail.encounter_adjustment
    : undefined;
  const [recordForm] = Form.useForm<AddRecordForm>();
  Form.useWatch("quantity", recordForm);
  Form.useWatch("adjustment", recordForm);
  const previewBasis = difficulty
    ? encounterPreviewBasis(difficulty.encounterKey, difficulty.revision)
    : undefined;
  const currentAddition = () =>
    selectedRecordKey
      ? encounterRecordAddition(
          selectedRecordKey,
          selectedIsCreature,
          recordForm,
          scenario,
          preparedCreature,
          previewBasis,
        )
      : null;
  const waitingForCreaturePreview = Boolean(
    scenario && !currentAddition()?.creature_scenario?.expected,
  );
  const [unconfirmed, setUnconfirmed] = useState<{
    encounterKey: string;
    request: AddEncounterRecordParticipantRequest;
    receipt: string;
    title: string;
  } | null>(null);
  const adding = useRef(false);
  useEffect(() => {
    if (disabled) return;
    if (pendingFormReset.current.record) {
      recordForm.setFields([
        { name: "quantity", value: 1, touched: false, errors: [] },
        { name: "adjustment", value: defaultAdjustment, touched: false, errors: [] },
      ]);
      pendingFormReset.current.record = false;
    }
  }, [disabled, committedAdd, recordForm, defaultAdjustment]);
  const addRecord = async () => {
    if (disabled || adding.current) return;
    const addition = currentAddition();
    const attempt =
      unconfirmed ??
      (addition &&
      candidate &&
      encounterSummary &&
      (!scenario || addition.creature_scenario?.expected)
        ? {
            encounterKey: encounterSummary.encounter_key,
            request: recordAdditionRequest(addition),
            title: candidate.record.title,
            receipt: `Added ${addition.quantity} × ${candidate.record.title}.`,
          }
        : null);
    if (!attempt) return;
    const { receipt } = attempt;
    adding.current = true;
    setCommittedAdd(null);
    setAdded("");
    try {
      const saved = await onAddRecord(attempt.encounterKey, attempt.request);
      setUnconfirmed(null);
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
    } catch (error) {
      setUnconfirmed(isUnconfirmedMutation(error) ? attempt : null);
    } finally {
      adding.current = false;
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
              icon={<GripVertical size={16} aria-hidden="true" />}
            />
            {participant.participant_kind === "initiative_entry" ? (
              <span
                className="encounter-roster__fixed-initiative"
                aria-label={`Initiative ${participant.initiative}`}
              >
                {participant.initiative}
              </span>
            ) : participant.participant_kind === "pc" ? (
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
              {participant.participant_kind === "initiative_entry" && (
                <span className="encounter-roster__state">Custom turn</span>
              )}
              {participant.hazard_disabled && (
                <span className="encounter-roster__state">Disabled</span>
              )}
              {participant.defeated && (
                <span className="encounter-roster__state">Defeated</span>
              )}
            </Button>
            <span
              className={`encounter-roster__hp ${rosterHpTone(participant)}`}
              aria-hidden={
                participant.participant_kind === "initiative_entry" || undefined
              }
              aria-label={
                participant.participant_kind === "initiative_entry"
                  ? undefined
                  : `${participant.display_name} HP`
              }
            >
              {participant.participant_kind === "initiative_entry"
                ? null
                : participant.current_hp == null
                  ? "—"
                  : (participant.current_hp + participant.temporary_hp).toString()}
            </span>
            <ParticipantActions
              participant={participant}
              disabled={disabled}
              compact={compactActions}
              removing={removingKey === participant.participant_key}
              onSetTurn={() => onSetTurn(participant)}
              onToggleDim={() =>
                onUpdate(participant.participant_key, {
                  expected_revision: participant.revision,
                  hidden: !participant.hidden,
                })
              }
              onRemove={(opener) => onRemove(participant, opener)}
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
    <section
      ref={roster}
      tabIndex={-1}
      className="encounter-pane encounter-roster"
      data-compact-actions={compactActions || undefined}
    >
      <header className="encounter-roster__header">
        <PrimaryToolbar className="encounter-roster__turn-bar">
          <div className="encounter-roster__turn-summary">
            {turnText && <strong>{turnText}</strong>}
            {needsInitiative && participants.length > 0 && (
              <span className="encounter-roster__turn-caption">Initiative not set</span>
            )}
          </div>
          {turnAction}
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
          <div className="detail-empty">Add a participant or custom turn.</div>
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
            encounterRevision={encounterRevision}
            disabled={disabled}
            onAddCreature={() => {
              setAdded("");
              setRecordOpen(true);
            }}
            onAddInitiativeEntry={
              onAddInitiativeEntry
                ? (opener) => {
                    entryOpener.current = opener;
                    setEntryOpen(true);
                  }
                : undefined
            }
            onAddPc={setPcAddMode}
          />
          {rollInitiativeAction}
        </PrimaryToolbar>
      </footer>
      <SearchPickerModal<AddRecordForm>
        fixedFilter={{ kind: "in", field: "kind", values: ["creature", "hazard"] }}
        emptyPrompt="Search for a creature or hazard"
        emptyResults="No creatures or hazards"
        form={recordForm}
        disabled={disabled}
        okButtonProps={{
          disabled:
            disabled ||
            Boolean(unconfirmed) ||
            waitingForCreaturePreview ||
            !selectedRecordKey,
          loading: disabled,
        }}
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
        compactOptions
        summary={
          difficulty && recordOpen ? (
            <EncounterAddDifficulty
              {...difficulty}
              form={recordForm}
              selectedRecordKey={selectedRecordKey}
              creature={selectedIsCreature}
              scenario={scenario}
              onPreparedCreature={setPreparedCreature}
              recordName={candidate?.record.title ?? ""}
              disabled={disabled}
            />
          ) : undefined
        }
        status={
          (recovery ||
            error ||
            unconfirmed ||
            added ||
            (!disabled && committedAdd)) && (
            <>
              {recovery}
              {unconfirmed && (
                <ErrorNotice
                  error={error}
                  title="Addition could not be confirmed"
                  outcome="uncertain"
                  description={`Confirm the original addition of ${unconfirmed.request.quantity} × ${unconfirmed.title}.`}
                  action={
                    <Button disabled={disabled} onClick={() => void addRecord()}>
                      Retry to confirm
                    </Button>
                  }
                />
              )}
              {error && !unconfirmed && (
                <ErrorNotice
                  error={error}
                  title="Participant could not be added"
                  description={error}
                />
              )}
              {(added || (!disabled && committedAdd)) && (
                <span role="status" className="encounter-add-receipt">
                  {committedAdd ?? added}
                </span>
              )}
            </>
          )
        }
      >
        <div
          className="encounter-add-fields"
          inert={!selectedRecordKey}
          style={selectedRecordKey ? undefined : { visibility: "hidden", opacity: 0 }}
        >
          {(!selectedRecordKey || selectedIsCreature) && <EncounterRecordAdjustment />}
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
      </SearchPickerModal>
      {entryOpen && onAddInitiativeEntry && (
        <InitiativeEntryEditor
          disabled={disabled}
          pending={pending}
          error={initiativeEntryError}
          onSave={onAddInitiativeEntry}
          onClose={() => setEntryOpen(false)}
          getDismissFocus={() => ({
            opener: entryOpener.current,
            pane: roster.current,
          })}
        />
      )}
      {pcAddMode && encounterSummary && (
        <PcAddDialog
          encounter={{
            encounterKey: encounterSummary.encounter_key,
            revision: encounterRevision,
          }}
          onClose={() => setPcAddMode(undefined)}
        />
      )}
    </section>
  );
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
  compact,
  removing,
  onSetTurn,
  onToggleDim,
  onRemove,
}: {
  participant: EncounterParticipantView;
  disabled: boolean;
  compact: boolean;
  removing: boolean;
  onSetTurn: () => void;
  onToggleDim: () => Promise<unknown>;
  onRemove: (opener: HTMLElement) => void;
}) {
  const trigger = useRef<HTMLButtonElement>(null);
  const focusAfterRemoveButtonHides = useCallback(
    (button: HTMLButtonElement | null) => {
      if (!button) return;
      return () => {
        if (button.ownerDocument.activeElement === button)
          restoreOverlayFocus(() => trigger.current, button);
      };
    },
    [],
  );
  function remove(opener: HTMLElement | null) {
    if (disabled || removing || !opener) return;
    opener.focus({ preventScroll: true });
    onRemove(opener);
  }
  return (
    <div className="encounter-roster__actions">
      <Dropdown
        disabled={disabled}
        trigger={["click"]}
        styles={{
          itemContent: {
            maxWidth: "min(18rem, calc(100vw - 64px))",
            whiteSpace: "normal",
            overflowWrap: "anywhere",
          },
        }}
        menu={{
          "aria-label": `${participant.display_name} actions`,
          onClick: ({ domEvent }) => domEvent.preventDefault(),
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
            ...(compact
              ? [
                  {
                    key: "remove",
                    label: "Remove participant",
                    danger: true,
                    icon: <Trash2 size={14} aria-hidden="true" />,
                    disabled: disabled || removing,
                    onClick: () => remove(trigger.current),
                  },
                ]
              : []),
          ],
        }}
      >
        <Button
          ref={trigger}
          type="text"
          size="small"
          disabled={disabled && !removing}
          aria-disabled={removing || undefined}
          className="encounter-roster__more"
          aria-label={`Actions for ${participant.display_name}`}
          aria-haspopup="menu"
          icon={<Ellipsis size={16} aria-hidden="true" />}
        />
      </Dropdown>
      {!compact && (
        <Tooltip
          title={`Remove ${participant.display_name}`}
          trigger={["hover", "focus"]}
        >
          <Button
            ref={focusAfterRemoveButtonHides}
            type="text"
            size="small"
            danger
            disabled={disabled && !removing}
            aria-disabled={removing || undefined}
            aria-label={`Remove ${participant.display_name}`}
            icon={<Trash2 size={14} aria-hidden="true" />}
            onClick={(event) => remove(event.currentTarget)}
          />
        </Tooltip>
      )}
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
