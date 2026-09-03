import { Button, Form, Input, InputNumber, Popover, Select } from "antd";
import { Minus, MoreHorizontal, Plus, Trash2 } from "lucide-react";
import { useState } from "react";
import type {
  AddEncounterParticipantConditionRequest,
  EncounterConditionDefinitionView,
  EncounterRuntimeConditionView,
  EncounterParticipantView,
  UpdateEncounterParticipantConditionRequest,
} from "../../generated/atlas";
import { useRecordPreviewRenderer } from "../../shared/records/RecordPreviewContext";
import { EditableCommitField } from "../../shared/ui/forms/EditableCommitField";
import { optionalNumber } from "./participantEdits";

type AddConditionForm = {
  conditionRef: string;
  value?: number;
  duration?: number;
  sourceParticipantKey?: string;
  note?: string;
};

type ConditionDetailsForm = {
  duration?: number;
  sourceParticipantKey?: string;
  note?: string;
};

export function EncounterConditionControls({
  current,
  conditionDefinitions,
  onAddCondition,
  onRemoveCondition,
  onReference,
  onUpdateCondition,
  participants,
}: {
  current: EncounterParticipantView;
  conditionDefinitions: EncounterConditionDefinitionView[];
  onAddCondition: (condition: AddEncounterParticipantConditionRequest) => void;
  onRemoveCondition: (participantKey: string, conditionId: number) => void;
  onReference: (recordKey: string) => void;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => void;
  participants: EncounterParticipantView[];
}) {
  const [conditionForm] = Form.useForm<AddConditionForm>();
  const [addConditionOpen, setAddConditionOpen] = useState(false);
  const [addDetailsOpen, setAddDetailsOpen] = useState(false);
  const addConditionRef = Form.useWatch("conditionRef", conditionForm);
  const addCondition = conditionDefinition(addConditionRef, conditionDefinitions);
  const addConditionHasValue = addCondition?.has_value ?? false;
  const submitAddCondition = () => {
    const values = conditionForm.getFieldsValue();
    const condition = conditionDefinition(values.conditionRef, conditionDefinitions);
    if (!condition) {
      return;
    }
    onAddCondition({
      participant_key: current.participant_key,
      condition_ref: condition.condition_ref,
      ...(!condition.has_value || values.value === undefined
        ? {}
        : { value: values.value }),
      ...(values.duration === undefined ? {} : { duration_rounds: values.duration }),
      ...(values.sourceParticipantKey
        ? { source_participant_key: values.sourceParticipantKey }
        : {}),
      ...(values.note ? { note: values.note } : {}),
    });
    conditionForm.resetFields();
    setAddDetailsOpen(false);
    setAddConditionOpen(false);
  };

  return (
    <section className="creature-sheet__panel record-surface-card--conditions encounter-conditions">
      <div className="encounter-conditions__header">
        <h3>Conditions</h3>
        <Button
          aria-expanded={addConditionOpen}
          onClick={() => setAddConditionOpen((open) => !open)}
          size="small"
        >
          Add Condition
        </Button>
      </div>
      {addConditionOpen && (
        <Form className="encounter-add-condition" form={conditionForm}>
          <div className="encounter-add-condition__row">
            <Form.Item name="conditionRef" rules={[{ required: true }]}>
              <Select
                aria-label="Add condition"
                showSearch
                optionFilterProp="label"
                options={conditionDefinitions.map((condition) => ({
                  label: condition.name,
                  value: condition.condition_ref,
                }))}
                placeholder="Condition"
                onChange={(conditionRef) => {
                  const condition = conditionDefinition(
                    conditionRef,
                    conditionDefinitions,
                  );
                  conditionForm.setFieldValue(
                    "value",
                    condition?.has_value
                      ? (optionalNumber(condition.default_value) ?? 1)
                      : undefined,
                  );
                }}
              />
            </Form.Item>
            {addConditionHasValue && (
              <Form.Item name="value">
                <InputNumber aria-label="Condition value" min={0} />
              </Form.Item>
            )}
            <Button
              aria-expanded={addDetailsOpen}
              aria-label="Condition details"
              icon={<MoreHorizontal size={14} />}
              onClick={() => setAddDetailsOpen((open) => !open)}
            />
            <Button type="primary" onClick={submitAddCondition}>
              Add
            </Button>
          </div>
          {addDetailsOpen && <ConditionDetailsFields participants={participants} />}
        </Form>
      )}
      {(current.record_view.encounter?.conditions ?? []).length === 0 ? (
        <p className="encounter-empty-note">No conditions</p>
      ) : (
        <div className="encounter-condition-list">
          {(current.record_view.encounter?.conditions ?? []).map((condition) => (
            <ConditionEditor
              condition={condition}
              key={condition.condition_id.toString()}
              participantKey={current.participant_key}
              participants={participants}
              conditionDefinitions={conditionDefinitions}
              onReference={onReference}
              onRemove={onRemoveCondition}
              onUpdate={onUpdateCondition}
            />
          ))}
        </div>
      )}
    </section>
  );
}

function ConditionDetailsFields({
  participants,
}: {
  participants: EncounterParticipantView[];
}) {
  return (
    <div className="encounter-condition-details">
      <Form.Item name="duration" label="Duration">
        <InputNumber aria-label="Duration rounds" min={0} />
      </Form.Item>
      <Form.Item name="sourceParticipantKey" label="Source">
        <Select
          allowClear
          aria-label="Source"
          options={participants.map((participant) => ({
            value: participant.participant_key,
            label: participant.display_name,
          }))}
        />
      </Form.Item>
      <Form.Item name="note" label="Note">
        <Input.TextArea
          aria-label="Condition note"
          autoSize={{ minRows: 2, maxRows: 8 }}
          className="encounter-condition-note-input"
        />
      </Form.Item>
    </div>
  );
}

function ConditionEditor({
  condition,
  participantKey,
  participants,
  conditionDefinitions,
  onReference,
  onRemove,
  onUpdate,
}: {
  condition: EncounterRuntimeConditionView;
  participantKey: string;
  participants: EncounterParticipantView[];
  conditionDefinitions: EncounterConditionDefinitionView[];
  onReference: (recordKey: string) => void;
  onRemove: (participantKey: string, conditionId: number) => void;
  onUpdate: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => void;
}) {
  const renderRecordPreview = useRecordPreviewRenderer();
  const [detailsForm] = Form.useForm<ConditionDetailsForm>();
  const [detailsOpen, setDetailsOpen] = useState(false);
  const hasValue = conditionTakesValue(condition, conditionDefinitions);
  const hasDetails = Boolean(
    condition.duration_rounds !== undefined ||
    condition.source_participant_key ||
    condition.note,
  );
  const update = (changes: Partial<UpdateEncounterParticipantConditionRequest>) =>
    onUpdate(participantKey, {
      condition_id: condition.condition_id,
      name: condition.name,
      value: condition.value,
      source_participant_key: condition.source_participant_key,
      duration_rounds: condition.duration_rounds,
      note: condition.note,
      ...changes,
    });
  const saveDetails = () => {
    const values = detailsForm.getFieldsValue();
    update({
      duration_rounds: values.duration,
      source_participant_key: values.sourceParticipantKey,
      note: values.note || undefined,
    });
    setDetailsOpen(false);
  };
  const openDetails = (open: boolean) => {
    if (open) {
      detailsForm.setFieldsValue({
        duration: optionalNumber(condition.duration_rounds),
        sourceParticipantKey: condition.source_participant_key,
        note: condition.note,
      });
    }
    setDetailsOpen(open);
  };
  return (
    <div
      className={[
        "encounter-condition-row",
        hasValue ? "" : "encounter-condition-row--no-value",
      ]
        .filter(Boolean)
        .join(" ")}
    >
      {condition.condition_key ? (
        renderRecordPreview ? (
          renderRecordPreview(condition.condition_key, (open) => (
            <button
              aria-expanded={open}
              aria-haspopup="dialog"
              className="encounter-condition-row__name encounter-condition-row__name-button"
              type="button"
            >
              {condition.name}
            </button>
          ))
        ) : (
          <button
            className="encounter-condition-row__name encounter-condition-row__name-button"
            onClick={() => onReference(condition.condition_key!)}
            type="button"
          >
            {condition.name}
          </button>
        )
      ) : (
        <span className="encounter-condition-row__name">{condition.name}</span>
      )}
      {hasValue && (
        <div
          aria-label={`${condition.name} value controls`}
          className="encounter-condition-value-controls"
          role="group"
        >
          <Button
            aria-label={`Decrease ${condition.name} value`}
            disabled={(condition.value ?? 0) <= 0}
            icon={<Minus size={12} />}
            onClick={() => stepConditionValue(condition.value, -1, update)}
            size="small"
          />
          <EditableCommitField
            ariaLabel={`${condition.name} value`}
            inputMode="numeric"
            onCommit={(value) => commitConditionValue(value, update)}
            size="small"
            value={inputNumberValue(condition.value)}
          />
          <Button
            aria-label={`Increase ${condition.name} value`}
            icon={<Plus size={12} />}
            onClick={() => stepConditionValue(condition.value, 1, update)}
            size="small"
          />
        </div>
      )}
      {condition.duration_rounds !== undefined && (
        <span className="encounter-condition-chip">
          {condition.duration_rounds.toString()} rounds
        </span>
      )}
      {hasDetails && <span className="encounter-condition-meta">Details</span>}
      <Popover
        content={
          <Form form={detailsForm} layout="vertical">
            <ConditionDetailsFields participants={participants} />
            <div className="encounter-condition-details__actions">
              <Button type="primary" onClick={saveDetails}>
                Save
              </Button>
            </div>
          </Form>
        }
        open={detailsOpen}
        onOpenChange={openDetails}
        placement="bottomRight"
        trigger="click"
      >
        <Button
          aria-label={`Edit ${condition.name} details`}
          icon={<MoreHorizontal size={14} />}
          size="small"
        />
      </Popover>
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

function conditionDefinition(
  conditionRef: string | undefined,
  definitions: EncounterConditionDefinitionView[],
): EncounterConditionDefinitionView | undefined {
  return definitions.find((condition) => condition.condition_ref === conditionRef);
}

function conditionTakesValue(
  condition: EncounterRuntimeConditionView,
  definitions: EncounterConditionDefinitionView[],
): boolean {
  const definition = conditionDefinition(condition.condition_key, definitions);
  return definition?.has_value ?? condition.value !== undefined;
}

function commitConditionValue(
  value: string,
  update: (changes: Partial<UpdateEncounterParticipantConditionRequest>) => void,
) {
  const trimmed = value.trim();
  if (trimmed.length === 0) {
    update({ value: undefined });
    return;
  }
  if (!/^\d+$/.test(trimmed)) {
    return;
  }
  update({ value: Number(trimmed) });
}

function inputNumberValue(value: number | undefined): string {
  return value === undefined ? "" : value.toString();
}

function stepConditionValue(
  value: number | undefined,
  step: -1 | 1,
  update: (changes: Partial<UpdateEncounterParticipantConditionRequest>) => void,
) {
  update({ value: Math.max(0, (value ?? 0) + step) });
}
